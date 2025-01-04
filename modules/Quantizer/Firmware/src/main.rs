#![allow(incomplete_features)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]
#![feature(int_roundings)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(inline_const)]
#![feature(cell_update)]

use core::arch::asm;
use core::{cell::Cell, panic::PanicInfo};

use arduino_hal::delay_ms;
use arduino_hal::hal::port;
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use arduino_hal::{hal::port::PB0, prelude::*, Peripherals};
use avr_device::interrupt::{self, Mutex};
use embedded_hal::digital::v2::OutputPin;
use fm_lib::{
    async_adc::{
        handle_conversion_result, init_async_adc, new_async_adc_state, AsyncAdc, GetAdcValues,
    },
    asynchronous::{assert_interrupts_disabled, unsafe_access_mutex},
    asynchronous::{AtomicRead, Borrowable},
    button_debouncer::ButtonDebouncer,
    eeprom::WearLevelledEepromWriter,
    handle_system_clock_interrupt,
    mcp4922::{DacChannel, MCP4922},
    system_clock::{ClockPrecision, GlobalSystemClockState, SystemClock},
};
use ufmt::uwriteln;


static SYSTEM_CLOCK_STATE: GlobalSystemClockState<{ ClockPrecision::MS16 }> =
    GlobalSystemClockState::new();
handle_system_clock_interrupt!(&SYSTEM_CLOCK_STATE);

static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<2> = new_async_adc_state();

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
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

    unsafe {
        avr_device::interrupt::enable();
    };

    init_async_adc(
        adc,
        &GLOBAL_ASYNC_ADC_STATE,
        [
            // a4.into_channel(),
            // a5.into_channel(),
            arduino_hal::adc::channel::ADC6.into_channel(),
            arduino_hal::adc::channel::ADC7.into_channel(),
        ],
    );

    let mut d13 = pins.d13.into_output();

    loop {
        d13.set_high();
        delay_ms(200);
        d13.set_low();
        delay_ms(600);
    }
}
