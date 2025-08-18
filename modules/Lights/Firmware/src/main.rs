#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embedded_hal::pwm::SetDutyCycle;
use rp_pico::entry;

use panic_halt as _;

use rp_pico::hal::prelude::*;

use rp_pico::hal::pac;

use rp_pico::hal;

use rp_pico::hal::pio::PIOExt;
use rp_pico::hal::Timer;
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_pio::Ws2812;

#[entry]
fn main() -> ! {
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

    // let _led: hal::gpio::Pin<_, hal::gpio::FunctionPio0, hal::gpio::PullNone> =
    //     pins.led.reconfigure();
    // let _led: hal::gpio::Pin<_, hal::gpio::FunctionPio0, hal::gpio::PullNone> =
    //     pins.gpio5.reconfigure();
    // let led_pin_id = 5;

    let sin = hal::rom_data::float_funcs::fsin::ptr();
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let mut ws = Ws2812::new(
        pins.gpio6.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    const STRIP_LEN: usize = 100;
    let mut leds: [RGB8; STRIP_LEN] = [(0, 0, 0).into(); STRIP_LEN];
    let mut t = 0.0;

    let strip_brightness = 64u8; // Limit brightness to 64/256
    let animation_speed = 0.1;

    // WS2812 demo
    loop {
        for (i, led) in leds.iter_mut().enumerate() {
            // An offset to give 3 consecutive LEDs a different color:
            let hue_offs = match i % 3 {
                1 => 0.25,
                2 => 0.5,
                _ => 0.0,
            };

            let sin_11 = sin((t + hue_offs) * 2.0 * core::f32::consts::PI);
            // Bring -1..1 sine range to 0..1 range:
            let sin_01 = (sin_11 + 1.0) * 0.5;

            let hue = 360.0 * sin_01;
            let sat = 1.0;
            let val = 1.0;

            let rgb = hsv2rgb_u8(hue, sat, val);
            *led = rgb.into();
        }

        ws.write(brightness(leds.iter().copied(), strip_brightness))
            .unwrap();

        delay.delay_ms(16); // ~60 FPS

        t += (16.0 / 1000.0) * animation_speed;
        while t > 1.0 {
            t -= 1.0;
        }
    }

    // PWM demo
    /*
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

    let mut level = 0;
    loop {
        level = (level + 1) % 256;
        channel1.set_duty_cycle_fraction(level as u16, 255).unwrap();
        channel2.set_duty_cycle_fraction(level as u16, 255).unwrap();
        channel3.set_duty_cycle_fraction(level as u16, 255).unwrap();
        delay.delay_ms(10);
    }
    */
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
