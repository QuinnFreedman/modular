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

mod bitvec;
mod menu;
mod persistence;
mod quantizer;
mod resistor_ladder_buttons;

use core::cell::Cell;

use arduino_hal::delay_ms;
use arduino_hal::hal::port::PD6;
use arduino_hal::hal::port::PD7;
use arduino_hal::prelude::*;
use arduino_hal::Spi;
use avr_device::interrupt;
use avr_device::interrupt::Mutex;
use embedded_hal::digital::v2::OutputPin;
use fixed::types::I1F15;
use fixed::types::I8F8;
use fixed::types::U16F16;
use fm_lib::async_adc::new_averaging_async_adc_state;
use fm_lib::asynchronous::assert_interrupts_disabled;
use fm_lib::asynchronous::AtomicRead;
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
use menu::ButtonInput;
use menu::{LedColor, MenuState};
use quantizer::QuantizationResult;
use quantizer::QuantizerState;
use resistor_ladder_buttons::ButtonLadderState;
use ufmt::uwrite;

static SYSTEM_CLOCK_STATE: GlobalSystemClockState<{ ClockPrecision::MS16 }> =
    GlobalSystemClockState::new();
handle_system_clock_interrupt!(&SYSTEM_CLOCK_STATE);

static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<3, 3> = new_averaging_async_adc_state();
static SAMPLE_READY: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut eeprom = arduino_hal::Eeprom::new(dp.EEPROM);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
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
    let a5 = pins.a5.into_analog_input(&mut adc);
    let mut led_driver_cs_pin = pins.d9.into_output_high();
    let mut input_led_a_pin = pins.a1.into_output();
    let mut output_led_a_pin = pins.a2.into_output();
    let mut input_led_b_pin = pins.a3.into_output();
    let mut output_led_b_pin = pins.a4.into_output();
    let mut led_blank_pin = pins.a0.into_output_high();
    pins.d4.into_output(); // Trigger output A
    pins.d5.into_output(); // Trigger output B

    led_driver_cs_pin.set_low();
    spi.transfer(&mut [0x00u8; 36]).unwrap_infallible();
    led_driver_cs_pin.set_high();
    delay_ms(1);

    led_blank_pin.set_low();

    configure_timer(&dp.TC2);

    init_async_adc(
        adc,
        &GLOBAL_ASYNC_ADC_STATE,
        [
            a5.into_channel(),
            arduino_hal::adc::channel::ADC6.into_channel(),
            arduino_hal::adc::channel::ADC7.into_channel(),
        ],
    );

    unsafe {
        avr_device::interrupt::enable();
    };

    let shift_btn_pin = pins.d8.into_pull_up_input();
    let trig_input_pin_a = pins.d2.into_floating_input();
    let trig_input_pin_b = pins.d3.into_floating_input();

    let mut quantizer_state = QuantizerState::new();
    let mut menu_state = MenuState::new(&mut eeprom);

    let mut update_leds = |spi: &mut Spi, leds: &[LedColor; 12]| {
        led_driver_cs_pin.set_low();
        for led in leds[6..12].iter().chain(leds[0..6].iter()) {
            let mut bytes = led.to_bytes();
            spi.transfer(&mut bytes).unwrap_infallible();
        }
        led_driver_cs_pin.set_high();
    };

    let sys_clock = SystemClock::init_system_clock(dp.TC0, &SYSTEM_CLOCK_STATE);
    let mut button_state = ButtonLadderState::new();
    let mut save_button = ButtonWithLongPress::<PD7, 32, 2000>::new(pins.d7.into_pull_up_input());
    let mut load_button = ButtonWithLongPress::<PD6, 32, 2000>::new(pins.d6.into_pull_up_input());

    let mut last_output = QuantizationResult::zero();

    let mut dac = MCP4922::new(d10);

    let mut cached_led_state = [LedColor::OFF; 12];

    loop {
        while !SAMPLE_READY.atomic_read() {}
        SAMPLE_READY.atomic_write(false);
        let cv = interrupt::free(|cs| GLOBAL_ASYNC_ADC_STATE.get_inner(cs).get_all());
        let current_time_ms = sys_clock.millis();
        let button_event = button_state.sample_adc_value(current_time_ms, cv[0]);
        let save_button_state = save_button.sample(current_time_ms);
        let load_button_state = load_button.sample(current_time_ms);

        let adc_value_a = I1F15::from_bits((cv[1] << 5) as i16);
        let adc_value_b = I1F15::from_bits((cv[2] << 5) as i16);
        let result = quantizer_state.step(
            adc_to_semitones(adc_value_a),
            adc_to_semitones(adc_value_b),
            trig_input_pin_a.is_high(),
            trig_input_pin_b.is_high(),
        );

        let leds = menu_state.handle_button_input_and_render_display(
            &mut quantizer_state,
            &ButtonInput {
                key_event: button_event,
                load_button: load_button_state,
                save_button: save_button_state,
                shift_pressed: shift_btn_pin.is_low(),
            },
            &result,
            current_time_ms,
            &mut eeprom,
        );

        if result.channel_a.actual_semitones != last_output.channel_a.actual_semitones {
            dac.write(
                &mut spi,
                DacChannel::ChannelA,
                semitones_to_dac(result.channel_a.actual_semitones),
            );
            dac.end_write();
        }
        if result.channel_b.actual_semitones != last_output.channel_b.actual_semitones {
            dac.write(
                &mut spi,
                DacChannel::ChannelB,
                semitones_to_dac(result.channel_b.actual_semitones),
            );
        }

        last_output = result;

        {
            let dp = unsafe { arduino_hal::Peripherals::steal() };
            dp.PORTD.portd.modify(|r, w| {
                unsafe { w.bits(r.bits()) }
                    .pd4()
                    .bit(last_output.channel_a.output_trigger)
                    .pd5()
                    .bit(last_output.channel_b.output_trigger)
            });
        }

        output_led_a_pin
            .set_state(last_output.channel_a.output_trigger_ui.into())
            .unwrap_infallible();
        output_led_b_pin
            .set_state(last_output.channel_b.output_trigger_ui.into())
            .unwrap_infallible();
        input_led_a_pin
            .set_state(last_output.channel_a.input_trigger_ui.into())
            .unwrap_infallible();
        input_led_b_pin
            .set_state(last_output.channel_b.input_trigger_ui.into())
            .unwrap_infallible();
        if !all_equal(leds, cached_led_state) {
            update_leds(&mut spi, &leds);
            cached_led_state = leds;
        }
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

fn adc_to_semitones(raw_adc_value: I1F15) -> I8F8 {
    // NOTE: this assumes readings can go all the way from 0 to 0x7FFF but in fact
    // the scale max is 0x7FE0. Idk if there any accuracy to be gained from
    // including that at this level of precision
    raw_adc_value.lerp(I8F8::ZERO, I8F8::from_bits(120 << 8))
}

fn semitones_to_dac(semitones: I8F8) -> u16 {
    assert!(semitones >= 0);
    assert!(semitones <= 120);
    let volts = U16F16::from_num(semitones / 12);
    let bits = (volts / U16F16::from_num(10)).to_bits();
    assert!(bits < u16::MAX as u32);
    (bits as u16) >> 4
}

fn configure_timer(tc2: &arduino_hal::pac::TC2) {
    // reset timer counter at TOP set by OCRA
    tc2.tccr2a.write(|w| w.wgm2().ctc());
    // set timer frequency to cycle at 1kHz
    // (16MHz clock speed / 128 prescale factor / 125 count/reset )
    tc2.tccr2b.write(|w| w.cs2().prescale_128());
    tc2.ocr2a.write(|w| w.bits(125));

    // enable interrupt on match to compare register A
    tc2.timsk2.write(|w| w.ocie2a().set_bit());
}

#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {
    let dp = unsafe { arduino_hal::Peripherals::steal() };

    assert_interrupts_disabled(|cs| {
        if !SAMPLE_READY.borrow(cs).get() {
            SAMPLE_READY.borrow(cs).set(true);
        } else {
            let pins = arduino_hal::pins!(dp);
            let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
            uwrite!(&mut serial, ".").unwrap_infallible();
        }
    });
}

fn all_equal<T, const N: usize>(a: [T; N], b: [T; N]) -> bool
where
    T: PartialEq,
{
    for i in 0..N {
        if a[i] != b[i] {
            return false;
        }
    }

    true
}
