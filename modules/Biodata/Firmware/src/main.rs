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
use arduino_hal::delay_us;
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
use fixed::types::U0F16;
use fixed::types::U16F16;
use fixed::types::U32F32;
use fm_lib::async_adc::new_averaging_async_adc_state;
use fm_lib::asynchronous::assert_interrupts_disabled;
use fm_lib::asynchronous::AtomicRead as _;
use fm_lib::button_debouncer::ButtonWithLongPress;
use fm_lib::display::show_float;
use fm_lib::mcp4922::DacChannel;
use fm_lib::mcp4922::MCP4922;
use fm_lib::{
    async_adc::{handle_conversion_result, init_async_adc, AsyncAdc, GetAdcValues},
    asynchronous::Borrowable,
    handle_system_clock_interrupt,
    system_clock::{ClockPrecision, GlobalSystemClockState, SystemClock},
};
use ufmt::uDisplay as _;
use ufmt::uwrite;
use ufmt::uwriteln;

static SYSTEM_CLOCK_STATE: GlobalSystemClockState<{ ClockPrecision::MS16 }> =
    GlobalSystemClockState::new();
handle_system_clock_interrupt!(&SYSTEM_CLOCK_STATE);

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

#[derive(PartialEq, Eq)]
enum NoteState {
    GateOn(u32),
    GateCooldown(u32),
    GateOff,
}

impl NoteState {
    fn is_on_cooldown(&self) -> bool {
        match self {
            NoteState::GateCooldown(_) => true,
            _ => false,
        }
    }
}

const X: bool = true;
const O: bool = false;
const CIRCLE_OF_FIFTHS: [[bool; 12]; 11] = [
    [X, O, O, O, O, O, O, O, O, O, O, O],
    [X, O, O, O, O, X, O, O, O, O, O, O],
    [X, O, O, O, O, X, O, O, O, O, X, O],
    [X, O, O, X, O, X, O, O, O, O, X, O],
    [X, O, O, X, O, X, O, O, X, O, X, O],
    [X, X, O, X, O, X, X, O, X, O, X, O],
    [X, X, O, X, O, X, X, O, X, O, X, X],
    [X, X, O, X, X, X, X, O, X, O, X, X],
    [X, X, O, X, X, X, X, O, X, X, X, X],
    [X, X, X, X, X, X, X, O, X, X, X, X],
    [X, X, X, X, X, X, X, X, X, X, X, X],
];

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    enable_external_interrupts(&dp);
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, AdcSettings::default());
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

    let a0 = pins.a0.into_analog_input(&mut adc);
    let a1 = pins.a1.into_analog_input(&mut adc);
    let a2 = pins.a2.into_analog_input(&mut adc);
    // let a3 = pins.a3.into_analog_input(&mut adc);
    // let a4 = pins.a4.into_analog_input(&mut adc);
    // let a5 = pins.a5.into_analog_input(&mut adc);
    let mut gate_out_a_pin = pins.d6.into_output();
    init_async_adc(
        adc,
        &GLOBAL_ASYNC_ADC_STATE,
        [
            a0.into_channel(),
            a1.into_channel(),
            a2.into_channel(),
            // a3.into_channel(),
            // a4.into_channel(),
            // a5.into_channel(),
            // arduino_hal::adc::channel::ADC6.into_channel(),
            // arduino_hal::adc::channel::ADC7.into_channel(),
        ],
    );

    configure_timer(&dp.TC1);
    let sys_clock = SystemClock::init_system_clock(dp.TC0, &SYSTEM_CLOCK_STATE);

    delay_ms(1);

    unsafe {
        avr_device::interrupt::enable();
    };

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut dac = MCP4922::new(d10);

    let mut note_state = NoteState::GateOff;

    let mut cv_output_current: u16 = 0;
    let mut cv_output_target: u16 = 0;

    let mut last_loop_time: u64 = 0;

    loop {
        let loop_start_time = sys_clock.micros();
        let cv = interrupt::free(|cs| GLOBAL_ASYNC_ADC_STATE.get_inner(cs).get_all());

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

        if ready_to_analyze && !note_state.is_on_cooldown() {
            let (mean, std_dev, spread) = analyze_samples(&array);

            cv_output_target = (std_dev * 16.0).min(4095.0) as u16;
            let delta = U0F16::from_bits(cv_output_current.abs_diff(cv_output_target));
            let delta_time = (loop_start_time - last_loop_time) as u32;
            last_loop_time = loop_start_time;
            let step_size = U0F16::from_bits((delta_time / 16).min(u16::MAX as u32) as u16);
            if cv_output_current < cv_output_target {
                cv_output_current += (step_size * delta).to_bits();
            } else if cv_output_current > cv_output_target {
                cv_output_current -= (step_size * delta).to_bits();
            }
            // uwriteln!(&mut serial, "{}->{}", cv_output_current, cv_output_target,)
            //     .unwrap_infallible();

            // These constants (and this algorithm) is copied directly from the MIDI Sprout
            // (https://github.com/electricityforprogress/MIDIsprout)
            // This method seems a little arbitrary. I'll probably change it before version 1.0
            let thresh = lerp(5.0, 1.61, (cv[0] as f32) / 1023.0);
            let change = spread as f32 > std_dev.max(1.0) * thresh;

            // uwriteln!(
            //     &mut serial,
            //     "{}, {}, {} {}",
            //     spread,
            //     show_float(std_dev),
            //     show_float(thresh),
            //     if change { '*' } else { ' ' }
            // )
            // .unwrap_infallible();

            if change {
                let gate_len = lerp_u7_to_u16((spread & 127) as u8, 100, 2500);
                if let NoteState::GateOn(_) = note_state {
                    gate_out_a_pin.set_low();
                    delay_ms(4);
                }
                note_state = NoteState::GateOn(sys_clock.millis() + gate_len as u32);

                let absolute_max_semitones: u8 = 60;
                let note_range = ((cv[2] * (absolute_max_semitones / 2) as u16) / 1024) as u8;
                let min_semitones: u8 = absolute_max_semitones / 2 - note_range;
                let max_semitones: u8 = absolute_max_semitones / 2 + note_range;

                let semitones = lerp_u7_to_u16(
                    (mean & 127) as u8,
                    min_semitones as u16,
                    max_semitones as u16,
                );
                let quant_table = &CIRCLE_OF_FIFTHS[(cv[1] / 93).min(10) as usize];
                let quantized_note = get_nearest_active_note(quant_table, semitones as u8);

                // uwriteln!(
                //     &mut serial,
                //     "{}({}.{})",
                //     quantized_note,
                //     quantized_note / 12,
                //     quantized_note % 12,
                // )
                // .unwrap_infallible();

                let dac_bits = semitones_to_dac(quantized_note.clamp(min_semitones, max_semitones));
                dac.write(&mut spi, DacChannel::ChannelA, dac_bits);

                delay_us(5);

                gate_out_a_pin.set_high();
            }
        }

        dac.write(&mut spi, DacChannel::ChannelB, cv_output_current.min(4095));

        let time = sys_clock.millis();
        match note_state {
            NoteState::GateOn(until) => {
                if time > until {
                    note_state = NoteState::GateCooldown(time + 5);
                    gate_out_a_pin.set_low();
                }
            }
            NoteState::GateCooldown(until) => {
                if time > until {
                    note_state = NoteState::GateOff
                }
            }
            NoteState::GateOff => {}
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

#[inline]
pub fn lerp_u7_to_u16(x: u8, out_min: u16, out_max: u16) -> u16 {
    debug_assert!(x <= 127);
    debug_assert!(out_min <= out_max);
    let range = (out_max - out_min) as u32;
    let scaled = (x as u32 * range + 63) / 127;
    out_min.wrapping_add(scaled as u16)
}

fn analyze_samples(samples: &[u16; ARRAY_SIZE]) -> (u16, f32, u16) {
    let mut min: u16 = samples[0];
    let mut max: u16 = samples[0];
    let mut sum: u32 = 0;
    for sample in samples.iter().copied() {
        sum += sample as u32;
        if sample < min {
            min = sample;
        } else if sample > max {
            max = sample;
        }
    }
    let delta = max - min;

    let mean = (sum / ARRAY_SIZE as u32) as u16;

    let mut variance: u32 = 0;
    for sample in samples {
        let delta = mean.abs_diff(*sample);
        let delta_squared = (delta as u32) * (delta as u32);
        variance = variance.saturating_add(delta_squared);
    }
    let std_dev = softfloat::F32::from_u32(variance / ARRAY_SIZE as u32)
        .sqrt()
        .to_native_f32();

    (mean, std_dev, delta)
}

fn get_nearest_active_note(notes: &[bool; 12], starting_note: u8) -> u8 {
    debug_assert!(starting_note < 120);

    for delta in 0..12 {
        let note_up = starting_note + delta;
        if notes[(note_up % 12) as usize] {
            return note_up;
        }
        let note_down = starting_note.saturating_sub(delta);
        if notes[(note_down % 12) as usize] {
            return note_down;
        }
    }

    debug_assert!(false);
    return starting_note;
}

fn semitones_to_dac(semitones: u8) -> u16 {
    // const MAX_SEMITONES: u8 = 120;
    // TODO: should be 120, but 60 for demo because the output is only 0-5V
    const MAX_SEMITONES: u8 = 60;
    debug_assert!(semitones <= MAX_SEMITONES);
    let bits = (U16F16::const_from_int(semitones as u32)
        / U16F16::const_from_int(MAX_SEMITONES as u32))
    .to_bits()
    .min(0xFFFF);
    (bits as u16) >> 4
}
