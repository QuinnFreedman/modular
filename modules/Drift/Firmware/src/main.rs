#![allow(incomplete_features)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]

use arduino_hal::prelude::*;
use core::{cell::Cell, panic::PanicInfo};

use avr_device::interrupt::{self, Mutex};
use fm_lib::asynchronous::{AtomicRead, Borrowable};
use fm_lib::{
    async_adc::{
        handle_conversion_result, init_async_adc, new_async_adc_state, AsyncAdc, GetAdcValues,
    },
    handle_system_clock_interrupt,
    mcp4922::{DacChannel, MCP4922},
    system_clock::{ClockPrecision, GlobalSystemClockState, SystemClock},
};
use ufmt::uwriteln;

static SYSTEM_CLOCK_STATE: GlobalSystemClockState<{ ClockPrecision::MS16 }> =
    GlobalSystemClockState::new();
handle_system_clock_interrupt!(&SYSTEM_CLOCK_STATE);

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    serial.flush();
    serial.write_byte(b'\r');
    serial.write_byte(b'\n');
    serial.write_byte(b'\r');
    serial.write_byte(b'\n');
    if let Some(location) = info.location() {
        uwriteln!(
            &mut serial,
            "Panic occurred in file '{}' at line {}",
            location.file(),
            location.line()
        )
        .unwrap_infallible();
    } else {
        uwriteln!(&mut serial, "Panic occurred").unwrap_infallible();
    }

    let short = 100;
    let long = 500;
    let mut led = pins.d13.into_output();
    loop {
        for len in [short, long] {
            for _ in 0..3u8 {
                led.set_high();
                arduino_hal::delay_ms(len);
                led.set_low();
                arduino_hal::delay_ms(short);
            }
        }
    }
}

static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<4> = new_async_adc_state();

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
}

static DAC_WRITE_QUEUED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[cfg(feature = "debug")]
static DEBUG_SKIPPED_WRITE_COUNT: Mutex<Cell<u8>> = Mutex::new(Cell::new(0));

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let a4 = pins.a4.into_analog_input(&mut adc);
    let a5 = pins.a5.into_analog_input(&mut adc);

    let (mut spi, d10) = arduino_hal::spi::Spi::new(
        dp.SPI,
        pins.d13.into_output(),        // Clock
        pins.d11.into_output(),        // MOSI
        pins.d12.into_pull_up_input(), // MISO
        pins.d10.into_output(),        // CS
        arduino_hal::spi::Settings {
            data_order: arduino_hal::spi::DataOrder::MostSignificantFirst,
            clock: arduino_hal::spi::SerialClockRate::OscfOver2,
            mode: embedded_hal::spi::MODE_0,
        },
    );

    #[cfg(feature = "debug")]
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    unsafe {
        avr_device::interrupt::enable();
    };

    init_async_adc(
        adc,
        &GLOBAL_ASYNC_ADC_STATE,
        [
            a4.into_channel(),
            a5.into_channel(),
            arduino_hal::adc::channel::ADC6.into_channel(),
            arduino_hal::adc::channel::ADC7.into_channel(),
        ],
    );

    // let config = match (config_pin_1.is_high(), config_pin_2.is_high()) {
    //     (true, true) => AuxMode::EndOfRise,
    //     (false, true) => AuxMode::EndOfFall,
    //     (true, false) => AuxMode::NonZero,
    //     (false, false) => AuxMode::FollowGate,
    // };

    let sys_clock = SystemClock::init_system_clock(dp.TC0, &SYSTEM_CLOCK_STATE);

    // configure_timer(&dp.TC2);
    let mut dac = MCP4922::new(d10);
    dac.shutdown_channel(&mut spi, DacChannel::ChannelB);

    let mut debug_led_value = 0u8;
    let mut led_pin = pins.d3.into_output();
    configure_timer_for_pwm(&dp.TC2);

    // dp.TC2.ocr2a.write(|w| w.bits(10));

    loop {
        // dp.TC2.ocr2a.write(|w| w.bits(debug_led_value));
        dp.TC2.ocr2b.write(|w| w.bits(debug_led_value));
        debug_led_value = debug_led_value.wrapping_add(1);
        arduino_hal::delay_ms(10);
    }

    /*
    loop {
        let cv = interrupt::free(|cs| GLOBAL_ASYNC_ADC_STATE.get_inner(cs).get_all());

        let current_time = sys_clock.millis_exact();


        if !DAC_WRITE_QUEUED.atomic_read() {
            // let value = update(&mut envelope_state, &input, &cv);
            let value = 0;

            dac.write_keep_cs_pin_low(&mut spi, DacChannel::ChannelA, value, Default::default());
            DAC_WRITE_QUEUED.atomic_write(true);
        }

        #[cfg(feature = "debug")]
        {
            use ufmt::uwrite;
            let num_skipped = DEBUG_SKIPPED_WRITE_COUNT.atomic_read();
            if num_skipped != 0 {
                unsafe_access_mutex(|cs| DEBUG_SKIPPED_WRITE_COUNT.borrow(cs).set(0));
                for _ in 0..num_skipped {
                    uwrite!(&mut serial, ".").unwrap_infallible();
                }
            }
        }
    }
    */
}

fn configure_timer_for_pwm(tc2: &arduino_hal::pac::TC2) {
    tc2.tccr2a.write(|w| {
        w
            // Set mode to PWM fast (asymmetric rising-only)
            .wgm2()
            .pwm_fast()
            // disconnect OCR2A pin
            .com2a()
            .disconnected()
            // set OCR2B to be the output pin in non-inverting mode
            .com2b()
            .match_clear()
    });
    tc2.tccr2b.write(|w| {
        w.wgm22()
            .clear_bit()
            // set PWM speed
            .cs2()
            .prescale_64()
    });
}

/*
fn configure_timer(tc2: &arduino_hal::pac::TC2) {
    // reset timer counter at TOP set by OCRA
    tc2.tccr2a.write(|w| w.wgm2().ctc());
    // set timer frequency to cycle at ~2.2727kHz
    // (16MHz clock speed / 64 prescale factor / 120 count/reset )
    tc2.tccr2b.write(|w| w.cs2().prescale_64());
    tc2.ocr2a.write(|w| w.bits(120));

    // enable interrupt on match to compare register A
    tc2.timsk2.write(|w| w.ocie2a().set_bit());
}
*/
