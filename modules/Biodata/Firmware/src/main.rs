#![allow(incomplete_features)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]
#![feature(int_roundings)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(cell_update)]

use core::borrow::BorrowMut;
use core::cell::Cell;
use core::cell::RefCell;
use core::default;

use arduino_hal::adc::AdcSettings;
use arduino_hal::delay_ms;
use arduino_hal::hal::port::PD6;
use arduino_hal::hal::port::PD7;
use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use arduino_hal::Spi;
use avr_device::interrupt;
use avr_device::interrupt::Mutex;
use embedded_hal::digital::v2::OutputPin;
use fixed::traits::FromFixed as _;
use fixed::types::I1F15;
use fixed::types::I8F8;
use fixed::types::U16F16;
use fm_lib::async_adc::new_averaging_async_adc_state;
use fm_lib::asynchronous::assert_interrupts_disabled;
use fm_lib::asynchronous::AtomicRead as _;
use fm_lib::button_debouncer::ButtonWithLongPress;
use fm_lib::mcp4922::DacChannel;
use fm_lib::mcp4922::MCP4922;
use fm_lib::{
    async_adc::{handle_conversion_result, init_async_adc, AsyncAdc, GetAdcValues},
    asynchronous::Borrowable,
    handle_system_clock_interrupt,
    system_clock::{ClockPrecision, GlobalSystemClockState, SystemClock},
};
use ufmt::uwrite;
use ufmt::uwriteln;

const ARRAY_SIZE: usize = 16;

static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<3, 3> = new_averaging_async_adc_state();
static SAMPLE_ARRAY: Mutex<RefCell<[u16; ARRAY_SIZE]>> = Mutex::new(RefCell::new([0; ARRAY_SIZE]));
static SAMPLE_ARRAY_INDEX: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0));

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
}

// pin d2
#[avr_device::interrupt(atmega328p)]
fn INT0() {
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let mut delta = dp.TC1.tcnt1.read().bits();
    dp.TC1.tcnt1.write(|w| w.bits(0));
    let has_overflowed = dp.TC1.tifr1.read().tov1().bit_is_set();
    if has_overflowed {
        delta = u16::MAX;
        dp.TC1.tifr1.write(|w| w.tov1().set_bit());
    }
    assert_interrupts_disabled(|cs| {
        let mut idx = SAMPLE_ARRAY_INDEX.borrow(cs).borrow_mut();
        let mut array = SAMPLE_ARRAY.borrow(cs).borrow_mut();

        if *idx >= ARRAY_SIZE as u8 {
            return;
        }

        array[*idx as usize] = delta;
        *idx += 1;
    })
}

/// Enable external interrupts for INT0 (digital pin 2)
fn enable_external_interrupts(dp: &Peripherals) {
    // set pins d2 as an input
    dp.PORTD
        .ddrd
        .modify(|r, w| unsafe { w.bits(r.bits()) }.pd2().clear_bit());
    // enable external interrupt 0
    dp.EXINT.eimsk.write(|r| r.int0().set_bit());
    // trigger interrupt on rising edge
    dp.EXINT.eicra.write(|r| r.isc0().val_0x03());
}

fn configure_timer(tc1: &arduino_hal::pac::TC1) {
    tc1.tccr1b.write(|w| w.cs1().prescale_64());
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    enable_external_interrupts(&dp);
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, AdcSettings::default());
    // let (mut spi, d10) = arduino_hal::spi::Spi::new(
    //     dp.SPI,
    //     pins.d13.into_output(),        // Clock
    //     pins.d11.into_output(),        // MOSI
    //     pins.d12.into_pull_up_input(), // MISO
    //     pins.d10.into_output(),        // CS
    //     arduino_hal::spi::Settings {
    //         data_order: arduino_hal::spi::DataOrder::MostSignificantFirst,
    //         clock: arduino_hal::spi::SerialClockRate::OscfOver2,
    //         mode: embedded_hal::spi::MODE_0,
    //     },
    // );

    // let d2 = pins.d2.into_floating_input();
    let a5 = pins.a5.into_analog_input(&mut adc);
    let mut d13 = pins.d13.into_output();

    init_async_adc(
        adc,
        &GLOBAL_ASYNC_ADC_STATE,
        [
            a5.into_channel(),
            arduino_hal::adc::channel::ADC6.into_channel(),
            arduino_hal::adc::channel::ADC7.into_channel(),
        ],
    );

    configure_timer(&dp.TC1);

    unsafe {
        avr_device::interrupt::enable();
    };

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    // let mut dac = MCP4922::new(d10);

    loop {
        // d13.set_high();
        // delay_ms(100);
        // d13.set_low();
        // delay_ms(1000);

        let mut array = [0u16; ARRAY_SIZE];
        let ready_to_analyze = interrupt::free(|cs| {
            let mut idx = SAMPLE_ARRAY_INDEX.borrow(cs).borrow_mut();
            if *idx == ARRAY_SIZE as u8 {
                *idx = 0;
                array = *SAMPLE_ARRAY.borrow(cs).borrow_mut();
                true
            } else {
                false
            }
        });
        if ready_to_analyze {
            let (mean, std_dev) = analyze_samples(&array);
            uwriteln!(&mut serial, "{}, {}", mean, std_dev).unwrap_infallible();
        }
    }
}

fn analyze_samples(samples: &[u16; ARRAY_SIZE]) -> (u16, u16) {
    // let mut mean: u16 = 0;
    let mut sum: i32 = 0;
    for sample in samples {
        sum += *sample as i32;
        // mean += sample / ARRAY_SIZE as u16;
    }

    let mean = (sum / ARRAY_SIZE as i32) as u16;

    let mut variance: u32 = 0;
    for sample in samples {
        let delta = mean.abs_diff(*sample);
        let delta_squared = (delta as u32) * (delta as u32);
        variance += delta_squared;
    }
    let std_dev = variance.isqrt() as u16;

    (mean, std_dev)
}

fn semitones_to_dac(semitones: I8F8) -> u16 {
    debug_assert!(semitones >= 0);
    debug_assert!(semitones <= 120);
    let bits = (U16F16::from_fixed(semitones) / U16F16::from_num(120))
        .to_bits()
        .min(0xFFFF);
    (bits as u16) >> 4
}
