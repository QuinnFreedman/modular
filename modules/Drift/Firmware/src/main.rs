#![allow(incomplete_features)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]
#![feature(adt_const_params)]

use arduino_hal::prelude::*;
use core::{cell::Cell, panic::PanicInfo};

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

    let mut state = ModuleState { time: 0 };

    loop {
        let cv = interrupt::free(|cs| GLOBAL_ASYNC_ADC_STATE.get_inner(cs).get_all());
        // uwriteln!(&mut serial, "{}, {}", cv[2], cv[3]).unwrap_infallible(); continue;

        if !DAC_WRITE_QUEUED.atomic_read() {
            let value = update(&mut state, &cv);

            dp.TC2.ocr2b.write(|w| w.bits((value >> 4) as u8));

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
}

// TODO have unique states and update functions for each mode
struct ModuleState {
    time: u32,
}

fn get_delta_t(cv: u16) -> u32 {
    // 10 seconds
    const MAX_PHASE_TIME_MICROS: u32 = 10 * 1000 * 1000;
    // ~2.27kHz == .48 ms / period
    const MICROS_PER_STEP: u32 = 480;
    const MAX_STEPS_PER_CYCLE: u16 = (MAX_PHASE_TIME_MICROS / MICROS_PER_STEP) as u16;
    let cv_fraction = read_cv::<{ CvType::Exponential }>(cv);
    let mut actual_steps_per_cycle = (cv_fraction.numerator as u32 * MAX_STEPS_PER_CYCLE as u32)
        / cv_fraction.denominator as u32;
    if actual_steps_per_cycle == 0 {
        actual_steps_per_cycle = 1;
    }

    u32::MAX / actual_steps_per_cycle
}

fn update(state: &mut ModuleState, cv: &[u16; 4]) -> u16 {
    let dt = get_delta_t(cv[2]);
    state.time = state.time.saturating_add(dt);
    let rollover = state.time == u32::MAX;
    let before_rollover = state.time;
    if rollover {
        state.time = 0;
    }

    // Placeholder linear ramp
    (before_rollover >> 20) as u16
}

#[derive(Copy, Clone)]
pub struct Fraction<T> {
    pub numerator: T,
    pub denominator: T,
}

#[derive(PartialEq, Eq)]
pub enum CvType {
    Linear,
    Exponential,
}

impl core::marker::ConstParamTy for CvType {}

/**
Transforms a raw cv value into a usable fraction of the maximum.
- Inverts value to compensate for the inverting amplifier in hardware
- Shifts values slightly to account for the fact that the input voltage
    is limited to a slightly smaller range than the DAC can read
- Applies a simple piecewise exponential curve to make the knobs more usable
*/
pub fn read_cv<const CURVE: CvType>(cv: u16) -> Fraction<u16> {
    let numerator = match CURVE {
        CvType::Linear => cv,
        CvType::Exponential => {
            if cv < 512 {
                cv / 4
            } else if cv < 768 {
                cv - 384
            } else {
                3 * cv - 1920
            }
        }
    };

    let denominator = match CURVE {
        CvType::Linear => 1024,
        // the piecewise function isn't perfect, the range is a little larger
        // than the domain. It actually goes to 1011. Round to 1024 for performance
        CvType::Exponential => 1024,
    };

    Fraction {
        numerator: u16::min(numerator, denominator),
        denominator,
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
