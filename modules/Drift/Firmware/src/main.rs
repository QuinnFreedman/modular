#![allow(incomplete_features)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]
#![feature(adt_const_params)]
#![feature(cell_update)]

mod bezier;
mod shared;

use arduino_hal::prelude::*;
use bezier::BezierModuleState;
use core::{cell::Cell, panic::PanicInfo};
use shared::DriftModule;

use avr_device::interrupt::{self, Mutex};
use fm_lib::asynchronous::{assert_interrupts_disabled, AtomicRead, Borrowable};
use fm_lib::{
    async_adc::{
        handle_conversion_result, init_async_adc, new_async_adc_state, AsyncAdc, GetAdcValues,
    },
    mcp4922::{DacChannel, MCP4922},
};
use ufmt::uwriteln;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interrupt::disable();
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

    configure_timer_interrupt(&dp.TC0);
    let mut dac = MCP4922::new(d10);
    dac.shutdown_channel(&mut spi, DacChannel::ChannelB);

    let _ = pins.d3.into_output();
    configure_timer_for_pwm(&dp.TC2);

    // TODO load different module depending on configuration
    // TODO read floating analog pins to get RNG seed
    let module: &mut dyn DriftModule = &mut BezierModuleState::new(0);

    loop {
        let cv = interrupt::free(|cs| GLOBAL_ASYNC_ADC_STATE.get_inner(cs).get_all());
        // uwriteln!(&mut serial, "{}, {}", cv[2], cv[3]).unwrap_infallible(); continue;

        if !DAC_WRITE_QUEUED.atomic_read() {
            let value = module.step(&cv);

            dp.TC2.ocr2b.write(|w| w.bits((value >> 4) as u8));

            dac.write_keep_cs_pin_low(&mut spi, DacChannel::ChannelA, value, Default::default());
            DAC_WRITE_QUEUED.atomic_write(true);
        }

        #[cfg(feature = "debug")]
        {
            use ufmt::uwrite;
            let num_skipped = DEBUG_SKIPPED_WRITE_COUNT.atomic_read();
            if num_skipped != 0 {
                DEBUG_SKIPPED_WRITE_COUNT.atomic_write(0);
                for _ in 0..num_skipped {
                    uwrite!(&mut serial, ".").unwrap_infallible();
                }
            }
        }
    }
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

fn configure_timer_interrupt(tc0: &arduino_hal::pac::TC0) {
    // reset timer counter at TOP set by OCRA
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    // set timer frequency to cycle at ~2.2727kHz
    // (16MHz clock speed / 64 prescale factor / 120 count/reset )
    tc0.tccr0b.write(|w| w.cs0().prescale_64());
    tc0.ocr0a.write(|w| w.bits(120));

    // enable interrupt on match to compare register A
    tc0.timsk0.write(|w| w.ocie0a().set_bit());
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    let dp = unsafe { arduino_hal::Peripherals::steal() };

    assert_interrupts_disabled(|cs| {
        if DAC_WRITE_QUEUED.borrow(cs).get() {
            DAC_WRITE_QUEUED.borrow(cs).set(false);
            dp.PORTB
                .portb
                .modify(|r, w| unsafe { w.bits(r.bits()) }.pb2().set_bit());
        } else {
            #[cfg(feature = "debug")]
            DEBUG_SKIPPED_WRITE_COUNT.borrow(cs).update(|x| x + 1);
        }
    });
}
