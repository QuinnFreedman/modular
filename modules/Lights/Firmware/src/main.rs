#![no_std]
#![no_main]

mod bp_filter;
mod peak_detector;
mod ringbuffer;

use bp_filter::BiquadDF2;
use core::panic::PanicInfo;
use defmt_rtt as _;
use embedded_alloc::LlffHeap as Heap;
use embedded_hal::digital::OutputPin;
use embedded_hal::pwm::SetDutyCycle;
use fixed::traits::Fixed;
use fixed::traits::ToFixed as _;
use fixed::types::I1F31;
use fixed::types::I32F32;
use fixed::types::U32F32;
use peak_detector::PeakDetector;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::adc::AdcChannel;
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

#[global_allocator]
static HEAP: Heap = Heap::empty();

const BUF_LEN: usize = 160 * 2;

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
    let mut adc_pin1 = AdcPin::new(pins.gpio27.into_floating_input()).unwrap();
    let mut adc_fifo = adc
        .build_fifo()
        .clock_divider(9599, 0) // ~10 kS/s, doubled for round robin
        // .set_channel(&mut adc_pin0)
        .round_robin((&adc_pin0, &adc_pin1))
        .enable_dma()
        .start_paused();
    // fn assert_adc_channel<T>(channel: T)
    // where
    //     T: AdcChannel,
    // {
    // }
    // assert_adc_channel(adc_pin1);
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

    let sample_rate = 10000;
    let q_value = 2.0;
    let mut filters = [
        BiquadDF2::new_bandpass(100.0, q_value, sample_rate),
        BiquadDF2::new_bandpass(200.0, q_value, sample_rate),
        BiquadDF2::new_bandpass(400.0, q_value, sample_rate),
        BiquadDF2::new_bandpass(800.0, q_value, sample_rate),
        BiquadDF2::new_bandpass(1600.0, q_value, sample_rate),
        BiquadDF2::new_bandpass(3200.0, q_value, sample_rate),
    ];
    let mut peak_detectors: [PeakDetector; 6] = core::array::from_fn(|_| PeakDetector::new());

    let mut hue: f32 = 0.0;

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

        let mut samples = [[0u16; BUF_LEN / 2]; 2];
        for (i, sample) in full_buffer.iter().enumerate() {
            samples[i % 2][i / 2] = *sample;
        }

        let level = samples[1]
            .iter()
            .map(|x| 4095u16.saturating_sub(*x) as u32)
            .sum::<u32>()
            / (BUF_LEN as u32 / 2);
        hue += 0.1;
        let (r, g, b) = hsv_to_rgb_f32(hue, 1.0, level as f32 / 4095.0);
        channel1
            .set_duty_cycle_fraction((b * 4095.0) as u16, 4095)
            .unwrap();
        channel2
            .set_duty_cycle_fraction((g * 4095.0) as u16, 4095)
            .unwrap();
        channel3
            .set_duty_cycle_fraction((r * 4095.0) as u16, 4095)
            .unwrap();
        let leds = calculate_leds(&samples[0], &mut filters, &mut peak_detectors);
        // ws.write(brightness(leds.iter().copied(), strip_brightness))
        //     .unwrap();
        ws.write(leds.iter().copied()).unwrap();
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

const STRIP_LEN: usize = 30;
fn calculate_leds<const N: usize>(
    samples: &[u16],
    filters: &mut [BiquadDF2; N],
    peak_detectors: &mut [PeakDetector; N],
) -> [RGB8; STRIP_LEN] {
    const GAMMA8: [u8; 256] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4,
        4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12,
        13, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24,
        24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40,
        41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
        64, 66, 67, 68, 69, 70, 72, 73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93,
        95, 96, 98, 99, 101, 102, 104, 105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124,
        126, 127, 129, 131, 133, 135, 137, 138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158,
        160, 162, 164, 167, 169, 171, 173, 175, 177, 180, 182, 184, 186, 189, 191, 193, 196, 198,
        200, 203, 205, 208, 210, 213, 215, 218, 220, 223, 225, 228, 231, 233, 236, 239, 241, 244,
        247, 249, 252, 255,
    ];

    let mut peaks = [I1F31::ZERO; N];
    for sample in samples.iter().copied() {
        let sample_fixed = sample_to_fixed(sample);
        for i in 0..N {
            let result = filters[i].process_sample(sample_fixed);
            peaks[i] = peak_detectors[i].step(result);
        }
    }

    let mut leds: [RGB8; STRIP_LEN] = [(0, 0, 0).into(); STRIP_LEN];
    for (i, led) in leds.iter_mut().enumerate() {
        let value = interpolate::<N, STRIP_LEN>(peaks, i);
        *led = hsv_to_rgb(
            (255 * i as u32 / STRIP_LEN as u32) as u8,
            255,
            // GAMMA8[(value.to_bits() >> 23) as usize],
            (value.to_bits() >> 23) as u8,
        )
        .into();
        // *led = hsv_to_rgb((value.to_bits() >> 23) as u8, 255, 255).into();
    }

    leds
}

fn interpolate<const N: usize, const M: usize>(values: [I1F31; N], i: usize) -> I1F31 {
    // const scale =
    let scale = U32F32::const_from_int((N as u64) - 1) / U32F32::const_from_int((M as u64) - 1);
    let t = i as u64 * scale;
    let k = t.int().to_num::<usize>();
    let alpha = t.frac().to_fixed::<I1F31>();
    if k >= N - 1 {
        values[N - 1]
    } else {
        (I1F31::MAX - alpha) * values[k] + alpha * values[k + 1]
    }
}

pub fn hsv_to_rgb(h: u8, s: u8, v: u8) -> (u8, u8, u8) {
    if s == 0 {
        return (v, v, v);
    }

    // Hue scaled to [0..=1535] (6 * 256)
    let region = (h as u16 * 6) >> 8; // [0..=5]
    let remainder = (h as u16 * 6) & 0xFF; // [0..=255]

    let p = (v as u16 * (255 - s as u16)) >> 8;
    let q = (v as u16 * (255 - ((s as u16 * remainder) >> 8))) >> 8;
    let t = (v as u16 * (255 - ((s as u16 * (255 - remainder)) >> 8))) >> 8;

    let (r, g, b) = match region {
        0 => (v as u16, t, p),
        1 => (q, v as u16, p),
        2 => (p, v as u16, t),
        3 => (p, q, v as u16),
        4 => (t, p, v as u16),
        _ => (v as u16, p, q),
    };

    (r as u8, g as u8, b as u8)
}

pub fn hsv_to_rgb_f32(hue: f32, sat: f32, val: f32) -> (f32, f32, f32) {
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
