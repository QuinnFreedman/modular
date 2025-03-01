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

mod menu;
mod quantizer;
mod resistor_ladder_buttons;

use core::arch::asm;
use core::{cell::Cell, panic::PanicInfo};

use arduino_hal::delay_ms;
use arduino_hal::hal::port;
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use arduino_hal::{hal::port::PB0, prelude::*, Peripherals};
use avr_device::interrupt::{self, Mutex};
use embedded_hal::digital::v2::{InputPin, OutputPin};
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
use menu::{LedColor, MenuState};
use quantizer::QuantizerState;
use resistor_ladder_buttons::{ButtonEvent, ButtonLadderState};
use ufmt::uwriteln;

static SYSTEM_CLOCK_STATE: GlobalSystemClockState<{ ClockPrecision::MS16 }> =
    GlobalSystemClockState::new();
handle_system_clock_interrupt!(&SYSTEM_CLOCK_STATE);

static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<3> = new_async_adc_state();

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let (mut spi, mut d10) = arduino_hal::spi::Spi::new(
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
    let a5 = pins.a5.into_analog_input(&mut adc);
    let mut led_driver_cs_pin = pins.d9.into_output_high();
    // TODO replace this with pullup resistor to BLANK pin
    led_driver_cs_pin.set_low();
    spi.transfer(&mut [0x00u8; 36]).unwrap_infallible();
    led_driver_cs_pin.set_high();
    delay_ms(1);

    unsafe {
        avr_device::interrupt::enable();
    };

    init_async_adc(
        adc,
        &GLOBAL_ASYNC_ADC_STATE,
        [
            a5.into_channel(),
            arduino_hal::adc::channel::ADC6.into_channel(),
            arduino_hal::adc::channel::ADC7.into_channel(),
        ],
    );

    let shift_btn_pin = pins.d8.into_pull_up_input();

    let mut quantizer_state = QuantizerState::new();
    let mut menu_state = MenuState::new();

    let mut update_leds = |leds: &[LedColor; 12]| {
        led_driver_cs_pin.set_low();
        for led in leds[6..12].iter().chain(leds[0..6].iter()) {
            let mut bytes = led.to_bytes();
            spi.transfer(&mut bytes).unwrap_infallible();
        }
        led_driver_cs_pin.set_high();
    };

    let sys_clock = SystemClock::init_system_clock(dp.TC0, &SYSTEM_CLOCK_STATE);
    let mut button_state = ButtonLadderState::new();

    loop {
        let cv = interrupt::free(|cs| GLOBAL_ASYNC_ADC_STATE.get_inner(cs).get_all());
        let button_event = button_state.sample_adc_value(sys_clock.millis(), cv[0]);
        let leds = menu_state.handle_button_input_and_render_display(
            &mut quantizer_state,
            &button_event,
            shift_btn_pin.is_low(),
        );

        update_leds(&leds);
        delay_ms(1);
    }
}

const RED_LEVEL: u16 = 0xFFF;
const GREEN_LEVEL: u16 = 0x04F;

impl LedColor {
    const fn to_bytes(&self) -> [u8; 3] {
        match self {
            LedColor::GREEN => concat_u12s(0, GREEN_LEVEL),
            LedColor::RED => concat_u12s(RED_LEVEL, 0),
            LedColor::AMBER => concat_u12s(RED_LEVEL, GREEN_LEVEL),
            LedColor::OFF => concat_u12s(0, 0),
        }
    }
}

const fn concat_u12s(left: u16, right: u16) -> [u8; 3] {
    assert!(left <= 0xFFF);
    assert!(right <= 0xFFF);
    let bytes = ((right as u32) | ((left as u32) << 12)).to_be_bytes();
    [bytes[1], bytes[2], bytes[3]]
}
