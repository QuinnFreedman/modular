#![allow(incomplete_features)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]
#![feature(int_roundings)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(effects)]
#![feature(inline_const)]
#![feature(cell_update)]

use core::{cell::Cell, panic::PanicInfo};

use arduino_hal::{
    hal::port,
    port::{
        mode::{Floating, Input, Output},
        Pin, PinOps,
    },
    prelude::*,
    Peripherals,
};

use avr_device::interrupt::{self, Mutex};
use fm_lib::{
    async_adc::{
        handle_conversion_result, init_async_adc, new_async_adc_state, AsyncAdc, AsyncAdcState,
        Indexable,
    },
    asynchronous::{assert_interrupts_disabled, unsafe_access_mutex, Borrowable},
    mcp4922::{DacChannel, MCP4922},
    rng::ParallelLfsr,
    rotary_encoder::RotaryEncoderHandler,
    system_clock::{ClockPrecision, GlobalSystemClockState},
};
use fm_lib::{handle_system_clock_interrupt, system_clock::SystemClock};
use rng::{RngModuleInputShort, RngModuleOutput};
use ufmt::uwriteln;

use crate::{
    led_driver::TLC5940,
    rng::{RngModule, RngModuleInput, SizeAdjustment},
};

mod led_driver;
mod rng;

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
    serial.write_byte(b'\n');
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

static ROTARY_ENCODER: RotaryEncoderHandler = RotaryEncoderHandler::new();

/**
 Pin-change interrupt handler for port B (pins d8-d13)
*/
#[avr_device::interrupt(atmega328p)]
fn PCINT2() {
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let port = dp.PORTD.pind.read();
    let a = port.pd4().bit_is_set();
    let b = port.pd5().bit_is_set();
    ROTARY_ENCODER.update(a, b);
}

static UNHANDLED_STEP_COUNT: Mutex<Cell<u8>> = Mutex::new(Cell::new(0));
/**
 Pin-change interrupt handler pin d2 (external interrupt 0)
*/
#[avr_device::interrupt(atmega328p)]
fn INT0() {
    assert_interrupts_disabled(|cs| UNHANDLED_STEP_COUNT.borrow(cs).update(|x| x + 1));
}

/// Enable pin-change interrupts on Port D, specifically digital pins 4 and 5
fn enable_portd_pc_interrupts(dp: &Peripherals) {
    // set pins d4 and d5 as inputs (PCINT20 and 21)
    dp.PORTD.ddrd.modify(|r, w| {
        unsafe { w.bits(r.bits()) }
            .pd4()
            .clear_bit()
            .pd5()
            .clear_bit()
    });
    // set pull-up for d4 and d5
    dp.PORTD
        .portd
        .modify(|r, w| unsafe { w.bits(r.bits()) }.pd4().set_bit().pd5().set_bit());
    // Enable interrupts for pins 4 and 5 in port D
    dp.EXINT
        .pcmsk2
        .modify(|r, w| w.pcint().bits(r.pcint().bits() | 0b00110000));
    // Enable pin-change interrupts for port D
    dp.EXINT
        .pcicr
        .modify(|r, w| w.pcie().bits(r.pcie().bits() | 0b100));
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

static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<3> = new_async_adc_state();

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
}

enum AnalogChannel {
    Chance,
    Bias,
    BiasCV,
}

impl Into<usize> for AnalogChannel {
    fn into(self) -> usize {
        self as usize
    }
}

const DEBUG_AUTO_STEP: bool = false;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    // Enable interrupts for rotary encoder
    enable_portd_pc_interrupts(&dp);
    enable_external_interrupts(&dp);

    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let a0 = pins.a0.into_analog_input(&mut adc);
    let a2 = pins.a2.into_analog_input(&mut adc);
    let a3 = pins.a3.into_analog_input(&mut adc);
    let a4 = pins.a4.into_analog_input(&mut adc);
    let a5 = pins.a5.into_analog_input(&mut adc);
    let seed: u16 = {
        // initialize random seed by reading voltage of floating pins
        let mut seed: u16 = 0;
        seed |= a5.analog_read(&mut adc) & 0xf;
        seed |= (a3.analog_read(&mut adc) & 0xf) << 4;
        seed |= (a2.analog_read(&mut adc) & 0xf) << 8;
        seed |= (adc.read_blocking(&arduino_hal::adc::channel::ADC7) & 0xf) << 12;
        seed
    };
    let encoder_switch = a5.into_digital(&mut adc).into_pull_up_input();
    let a2_digital = a2.into_digital(&mut adc).into_output();
    let a3_digital = a3.into_digital(&mut adc).into_output();
    let a4_digital = a4.into_digital(&mut adc).into_output();

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

    unsafe {
        avr_device::interrupt::enable();
    };

    init_async_adc(
        adc,
        &GLOBAL_ASYNC_ADC_STATE,
        [
            arduino_hal::adc::channel::ADC7.into_channel(),
            a0.into_channel(),
            arduino_hal::adc::channel::ADC6.into_channel(),
        ],
    );

    let xlatch = pins.d9.into_output();
    let pwm_ref = pins.d3.into_output();

    const NUM_LEDS: u8 = 7;
    const MAX_BUFFER_SIZE: u8 = 32;

    let led_driver =
        TLC5940::<{ NUM_LEDS as usize }>::new(&mut spi, pwm_ref, d10, xlatch, dp.TC1, dp.TC2);
    let sys_clock = SystemClock::init_system_clock(dp.TC0, &SYSTEM_CLOCK_STATE);
    let prng = ParallelLfsr::new(seed);
    let mut rng_module = RngModule::<MAX_BUFFER_SIZE, NUM_LEDS>::new(prng);
    let mut dac = MCP4922::new(pins.d8.into_output_high());

    let mut output_pins = OutputPins {
        enabled_led: a2_digital,
        clock_led: a3_digital,
        gate_a: pins.d7.into_output(),
        gate_b: pins.d6.into_output(),
    };

    let input_pins = DigitalInputPins {
        enabled_cv: pins.a1,
        gate_trig_switch: a4_digital.into_floating_input(),
    };

    let mut last_step_time: u32 = 0;

    loop {
        let current_time = sys_clock.millis();

        // Step the buffer forward if there have been any unhandled clock pulses
        {
            let steps_to_handle = unsafe_access_mutex(|cs| UNHANDLED_STEP_COUNT.borrow(cs).get());
            for _ in 0..steps_to_handle {
                handle_clock_step(
                    &mut rng_module,
                    current_time,
                    &input_pins,
                    &mut output_pins,
                    |value| dac.write(&mut spi, DacChannel::ChannelA, value),
                );
            }
            interrupt::free(|cs| {
                UNHANDLED_STEP_COUNT
                    .borrow(cs)
                    .update(|x| x.saturating_sub(steps_to_handle))
            });
        }

        // Handle rotary encoder
        {
            let re_delta = ROTARY_ENCODER.sample_and_reset();
            if re_delta != 0 {
                let size_change = if encoder_switch.is_low() {
                    SizeAdjustment::ExactDelta(re_delta)
                } else {
                    SizeAdjustment::PowersOfTwo(re_delta)
                };
                rng_module.adjust_buffer_size(size_change, current_time);
            }
        }

        // Handle time passing
        {
            let chance_pot_value = unsafe_access_mutex(|cs| {
                GLOBAL_ASYNC_ADC_STATE
                    .get_inner(cs)
                    .get(AnalogChannel::Chance)
            });
            let output = rng_module.time_step(
                current_time,
                &RngModuleInputShort {
                    chance_pot: chance_pot_value << 2,
                    enable_cv: input_pins.enabled_cv.is_high(),
                },
            );
            // optimization: we know that DAC value can't change between clock
            // pulses, so just skip writing it in this update (pass noop closure)
            write_module_output(&output, &mut output_pins, |_value| {});
        }

        // Handle LED bar display updates
        {
            let bias = unsafe_access_mutex(|cs| get_bias(GLOBAL_ASYNC_ADC_STATE.get_inner(cs)));

            rng_module.render_display_if_needed(
                bias,
                |buffer: &[u16; NUM_LEDS as usize]| -> Result<(), ()> {
                    led_driver.write(&mut spi, buffer)
                },
            );
        }

        if DEBUG_AUTO_STEP {
            debug_assert!(current_time >= last_step_time);
            if current_time - last_step_time >= 1000u32 {
                last_step_time += 1000;
                interrupt::free(|cs| UNHANDLED_STEP_COUNT.borrow(cs).set(1));
            }
        }
    }
}

/**
Reads bias pot and cv analog inputs, undoes functions applied to CV in hardware,
sums them, and upscales the result to match the RNG modules 12-bit scale
*/
fn get_bias<const N: usize>(adc: &Option<AsyncAdcState<N>>) -> u16 {
    let pot_value = adc.get(AnalogChannel::Bias);
    let cv_value = adc.get(AnalogChannel::BiasCV);
    const HALF_ADC_SCALE: i16 = 1024 / 2;
    let adjusted_cv = -(HALF_ADC_SCALE - (cv_value as i16)) * 2;
    let sum = pot_value.saturating_add_signed(adjusted_cv);
    // discard the 2 lsbs of the input to reduce display flickering. The difference
    // should not be musically noticeable
    (sum & !0b11u16) << 2
}

fn handle_clock_step<const MAX_BUFFER_SIZE: u8, const NUM_LEDS: u8, WriteToDac>(
    rng: &mut RngModule<MAX_BUFFER_SIZE, NUM_LEDS>,
    current_time: u32,
    input_pins: &SpecifiedDigitalInputPins,
    output_pins: &mut SpecifiedOutputPins,
    write_to_dac: WriteToDac,
) where
    [(); NUM_LEDS as usize]: Sized,
    [(); MAX_BUFFER_SIZE as usize]: Sized,
    WriteToDac: FnOnce(u16),
{
    let input = unsafe_access_mutex(|cs| {
        let adc = GLOBAL_ASYNC_ADC_STATE.get_inner(cs);
        RngModuleInput {
            chance_cv: adc.get(AnalogChannel::Chance) << 2,
            bias_cv: get_bias(adc),
            trig_mode: input_pins.gate_trig_switch.is_high(),
            enable_cv: input_pins.enabled_cv.is_high(),
        }
    });
    let output = rng.handle_clock_trigger(current_time, &input);

    write_module_output(&output, output_pins, write_to_dac);
}

fn write_module_output<WriteToDac>(
    state: &RngModuleOutput,
    output_pins: &mut SpecifiedOutputPins,
    write_to_dac: WriteToDac,
) where
    WriteToDac: FnOnce(u16),
{
    if state.enable_led_on {
        output_pins.enabled_led.set_high();
    } else {
        output_pins.enabled_led.set_low();
    }

    if state.clock_led_on {
        output_pins.clock_led.set_high();
    } else {
        output_pins.clock_led.set_low();
    }

    if !state.output_a {
        output_pins.gate_a.set_low();
    }
    if !state.output_b {
        output_pins.gate_b.set_low();
    }

    write_to_dac(state.analog_out);

    if state.output_a {
        output_pins.gate_a.set_high();
    }
    if state.output_b {
        output_pins.gate_b.set_high();
    }
}

type SpecifiedOutputPins = OutputPins<port::PC2, port::PC3, port::PD7, port::PD6>;
struct OutputPins<EnabledLedPin, ClockLedPin, GateAPin, GateBPin>
where
    EnabledLedPin: PinOps,
    ClockLedPin: PinOps,
    GateAPin: PinOps,
    GateBPin: PinOps,
{
    enabled_led: Pin<Output, EnabledLedPin>,
    clock_led: Pin<Output, ClockLedPin>,
    gate_a: Pin<Output, GateAPin>,
    gate_b: Pin<Output, GateBPin>,
}

type SpecifiedDigitalInputPins = DigitalInputPins<port::PC1, port::PC4>;
struct DigitalInputPins<EnabledCvPin, GateTrigSwitchPin>
where
    EnabledCvPin: PinOps,
    GateTrigSwitchPin: PinOps,
{
    enabled_cv: Pin<Input<Floating>, EnabledCvPin>,
    gate_trig_switch: Pin<Input<Floating>, GateTrigSwitchPin>,
}
