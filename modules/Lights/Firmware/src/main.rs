#![no_std]
#![no_main]

mod bp_filter;
mod peak_detector;
mod ringbuffer;

use bp_filter::BiquadBandPass;
use core::panic::PanicInfo;
use defmt_rtt as _;
use embedded_alloc::LlffHeap as Heap;
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;
use fixed::types::I1F31;
use peak_detector::PeakDetector;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::adc::AdcPin;
use rp_pico::hal::dma::double_buffer;
use rp_pico::hal::dma::DMAExt;
use rp_pico::hal::pac;
use rp_pico::hal::pio::PIOExt;
use rp_pico::hal::prelude::*;
use rp_pico::hal::Adc;
use rp_pico::hal::Timer;
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_pio::Ws2812;

use ringbuffer::RingBuffer;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const BUF_LEN: usize = 160;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let sio = hal::Sio::new(pac.SIO);
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led = pins.led.into_push_pull_output();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    loop {
        led.set_high().unwrap();
        delay.delay_ms(100);
        led.set_low().unwrap();
        delay.delay_ms(100);
    }
}

#[entry]
fn main() -> ! {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 64_000;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut adc_pin0 = AdcPin::new(pins.gpio26.into_floating_input()).unwrap();
    let mut adc_fifo = adc
        .build_fifo()
        .clock_divider(4799, 0) // ~10 kS/s
        .set_channel(&mut adc_pin0)
        // TODO, use round robin
        .enable_dma()
        .start_paused();
    let dma = pac.DMA.dyn_split(&mut pac.RESETS);
    static mut BUF_A: [u16; BUF_LEN] = [0; BUF_LEN];
    static mut BUF_B: [u16; BUF_LEN] = [0; BUF_LEN];

    let mut transfer = double_buffer::Config::new(
        (dma.ch0.unwrap(), dma.ch1.unwrap()),
        adc_fifo.dma_read_target(),
        #[allow(static_mut_refs)]
        unsafe {
            &mut BUF_A
        },
    )
    .start();

    // Start ADC after DMA is armed to avoid losing samples.
    adc_fifo.resume();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let mut ws = Ws2812::new(
        pins.gpio5.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let pwm = &mut pwm_slices.pwm1;
    pwm.set_ph_correct();
    pwm.set_div_int(1u8);
    pwm.enable();
    let channel1 = &mut pwm.channel_a;
    channel1.output_to(pins.gpio2);
    let channel2 = &mut pwm.channel_b;
    channel2.output_to(pins.gpio3);
    let pwm2 = &mut pwm_slices.pwm2;
    pwm2.set_ph_correct();
    pwm2.set_div_int(1u8);
    pwm2.enable();
    let channel3 = &mut pwm2.channel_a;
    channel3.output_to(pins.gpio4);

    let strip_brightness = 64u8; // out of 256
    let mut double_buffer_idx = false;

    let mut filter = BiquadBandPass::new(10000.0, 500.0, 5.0);
    let mut input_peak_detector = PeakDetector::new();

    let mut debug_pin = pins.gpio0.into_push_pull_output();
    loop {
        #[allow(static_mut_refs)]
        let (full_buffer, t) = transfer
            .write_next(if double_buffer_idx {
                unsafe { &mut BUF_B }
            } else {
                unsafe { &mut BUF_A }
            })
            .wait();
        double_buffer_idx = !double_buffer_idx;

        let level: u8 = 64;
        channel1.set_duty_cycle_fraction(level as u16, 255).unwrap();
        channel2.set_duty_cycle_fraction(level as u16, 255).unwrap();
        channel3.set_duty_cycle_fraction(level as u16, 255).unwrap();
        let leds = calculate_leds(full_buffer, &mut filter, &mut input_peak_detector);
        ws.write(brightness(leds.iter().copied(), strip_brightness))
            .unwrap();
        debug_pin.set_high().unwrap();
        delay.delay_us(80); // ~60 FPS
        debug_pin.set_low().unwrap();
        transfer = t;
    }
}

/// convert an ADC sample with range 0-4095 to a signed fixed number
fn sample_to_fixed(sample: u16) -> I1F31 {
    let centered: i32 = sample as i32 - 2048;
    I1F31::from_bits(centered << 20)
}

const STRIP_LEN: usize = 10;
fn calculate_leds(
    samples: &[u16],
    filter: &mut BiquadBandPass,
    peak_detector: &mut PeakDetector,
) -> [RGB8; STRIP_LEN] {
    let mut peak = I1F31::ZERO;
    let mut result = I1F31::ZERO;
    for sample in samples.iter().copied() {
        let sample_fixed = sample_to_fixed(sample);
        peak = peak_detector.step(sample_fixed);
        result = filter.step(sample_fixed);
    }

    let mut leds: [RGB8; STRIP_LEN] = [(0, 0, 0).into(); STRIP_LEN];
    // for (i, led) in leds.iter_mut().enumerate() {
    //     let red = if peak > (I1F31::MAX / STRIP_LEN as i32) * i as i32 {
    //         255
    //     } else {
    //         0
    //     };
    //     *led = (red, 0, 0).into();
    //     // *led = hsv2rgb_u8(360.0 / STRIP_LEN as f32 * i as f32, 1.0, 1.0).into();
    // }

    leds[0] = (255, 0, 0).into();
    leds[1] = (0, 255, 0).into();
    leds[2] = (0, 0, 255).into();

    leds
}

pub fn hsv2rgb(hue: f32, sat: f32, val: f32) -> (f32, f32, f32) {
    let c = val * sat;
    let v = (hue / 60.0) % 2.0 - 1.0;
    let v = if v < 0.0 { -v } else { v };
    let x = c * (1.0 - v);
    let m = val - c;
    let (r, g, b) = if hue < 60.0 {
        (c, x, 0.0)
    } else if hue < 120.0 {
        (x, c, 0.0)
    } else if hue < 180.0 {
        (0.0, c, x)
    } else if hue < 240.0 {
        (0.0, x, c)
    } else if hue < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (r + m, g + m, b + m)
}

pub fn hsv2rgb_u8(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let r = hsv2rgb(h, s, v);

    (
        (r.0 * 255.0) as u8,
        (r.1 * 255.0) as u8,
        (r.2 * 255.0) as u8,
    )
}
