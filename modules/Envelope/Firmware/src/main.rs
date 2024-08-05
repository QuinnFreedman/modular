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

use arduino_hal::hal::port;
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use arduino_hal::{hal::port::PB0, prelude::*, Peripherals};
use avr_device::interrupt::{self, Mutex};
use embedded_hal::digital::v2::OutputPin;
use envelope::{
    ui_show_mode, ui_show_stage, update, AcrcLoopState, AcrcState, AdsrState, AhrdState,
    EnvelopeMode,
};
use fm_lib::asynchronous::{AtomicRead, Borrowable};
use fm_lib::{
    async_adc::{
        handle_conversion_result, init_async_adc, new_async_adc_state, AsyncAdc, GetAdcValues,
    },
    asynchronous::{assert_interrupts_disabled, unsafe_access_mutex},
    button_debouncer::ButtonDebouncer,
    handle_system_clock_interrupt,
    mcp4922::{DacChannel, MCP4922},
    system_clock::{ClockPrecision, GlobalSystemClockState, SystemClock},
};
use ufmt::uwriteln;

use crate::aux::{update_aux, AuxMode};
use crate::envelope::{EnvelopeState, GateState, Input};

mod aux;
mod envelope;
mod exponential_curves;

static SYSTEM_CLOCK_STATE: GlobalSystemClockState<{ ClockPrecision::MS16 }> =
    GlobalSystemClockState::new();
handle_system_clock_interrupt!(&SYSTEM_CLOCK_STATE);

const UI_SHOW_ENVELOPE_MODE_MS: u32 = 2000;
#[derive(PartialEq, Eq)]
enum DisplayMode {
    ShowEnvelopeMode { until: u32 },
    ShowEnvelopeSegment,
}

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

/**
 Pin-change interrupt handler pin d3 (external interrupt 1)
*/
#[avr_device::interrupt(atmega328p)]
fn INT1() {
    assert_interrupts_disabled(|cs| {
        QUEUED_TRIGGER.borrow(cs).set(true);
    });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {
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

/// Enable external interrupts for INT0 (digital pin 2)
fn enable_external_interrupts(dp: &Peripherals) {
    // set pin d3 as an input
    dp.PORTD
        .ddrd
        .modify(|r, w| unsafe { w.bits(r.bits()) }.pd3().clear_bit());

    // enable pullup resistors for pin d3
    dp.PORTD
        .portd
        .modify(|r, w| unsafe { w.bits(r.bits()) }.pd3().set_bit());
    // enable external interrupt 1
    dp.EXINT.eimsk.write(|r| r.int1().set_bit());
    // trigger interrupt 1 on falling edge (trigger inputs are inverted)
    dp.EXINT.eicra.write(|r| r.isc1().val_0x02());
}

static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<4> = new_async_adc_state();

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
}

impl EnvelopeMode {
    fn next(self) -> Self {
        match self {
            EnvelopeMode::Adsr(_) => EnvelopeMode::Acrc(AcrcState::default()),
            EnvelopeMode::Acrc(_) => EnvelopeMode::AcrcLoop(AcrcLoopState::default()),
            EnvelopeMode::AcrcLoop(_) => EnvelopeMode::AhrdLoop(AhrdState::default()),
            EnvelopeMode::AhrdLoop(_) => EnvelopeMode::Adsr(AdsrState::default()),
        }
    }
}

impl EnvelopeState {
    fn cycle_mode(self) -> Self {
        Self {
            mode: self.mode.next(),
            time: 0,
            last_value: 0,
            artificial_gate: false,
        }
    }
}

static DAC_WRITE_QUEUED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static QUEUED_TRIGGER: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[cfg(feature = "debug")]
static DEBUG_SKIPPED_WRITE_COUNT: Mutex<Cell<u8>> = Mutex::new(Cell::new(0));

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    enable_external_interrupts(&dp);

    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let a4 = pins.a4.into_analog_input(&mut adc);
    let a5 = pins.a5.into_analog_input(&mut adc);
    let gate_pin = pins.d2.into_pull_up_input();
    let config_pin_1 = pins.a2.into_pull_up_input();
    let config_pin_2 = pins.a1.into_pull_up_input();

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

    let ui = UI::new(
        pins.d4.into_output(),
        pins.d5.into_output(),
        pins.d6.into_output(),
        pins.d7.into_output(),
    );

    let d8 = pins.d8.into_pull_up_input();
    let mut button = ButtonDebouncer::<PB0, 32>::new(d8);

    let config = match (config_pin_1.is_high(), config_pin_2.is_high()) {
        (true, true) => AuxMode::EndOfRise,
        (false, true) => AuxMode::EndOfFall,
        (true, false) => AuxMode::NonZero,
        (false, false) => AuxMode::FollowGate,
    };

    let _ = config_pin_1.into_floating_input();
    let _ = config_pin_2.into_floating_input();

    let sys_clock = SystemClock::init_system_clock(dp.TC0, &SYSTEM_CLOCK_STATE);

    let mut envelope_state = EnvelopeState {
        mode: EnvelopeMode::Adsr(AdsrState::default()),
        time: 0,
        last_value: 0,
        artificial_gate: false,
    };

    let mut display = DisplayMode::ShowEnvelopeMode {
        until: UI_SHOW_ENVELOPE_MODE_MS,
    };

    ui.update(ui_show_mode(&envelope_state.mode));

    configure_timer(&dp.TC2);
    let mut dac = MCP4922::new(d10);
    dac.shutdown_channel(&mut spi, DacChannel::ChannelB);

    let mut aux_output_pin = pins.d9.into_output();

    const LED_BLINK_INTERVAL_MS: u32 = 100;
    let mut led_blink_timer: u32 = 0;
    let mut led_blink_state: bool = false;

    let mut gate_was_high = false;

    loop {
        let cv = interrupt::free(|cs| GLOBAL_ASYNC_ADC_STATE.get_inner(cs).get_all());

        let current_time = sys_clock.millis_exact();
        let button_state = button.sample(current_time);
        let button_was_pressed = match button_state {
            fm_lib::button_debouncer::ButtonState::ButtonJustPressed => true,
            _ => false,
        };

        if button_was_pressed {
            envelope_state = envelope_state.cycle_mode();
            display = DisplayMode::ShowEnvelopeMode {
                until: current_time + UI_SHOW_ENVELOPE_MODE_MS,
            };
            led_blink_timer = current_time + LED_BLINK_INTERVAL_MS;
            led_blink_state = true;
            ui.update(ui_show_mode(&envelope_state.mode));
        }

        if let DisplayMode::ShowEnvelopeMode { until } = display {
            if current_time > until {
                display = DisplayMode::ShowEnvelopeSegment;
                ui.update(ui_show_stage(&envelope_state.mode));
            }

            if current_time > led_blink_timer {
                led_blink_timer = current_time + LED_BLINK_INTERVAL_MS;
                led_blink_state = !led_blink_state;
                if led_blink_state {
                    ui.update(ui_show_mode(&envelope_state.mode));
                } else {
                    ui.update(0);
                }
            }
        }

        if !DAC_WRITE_QUEUED.atomic_read() {
            let gate_is_high = gate_pin.is_low();
            let gate = match (gate_was_high, gate_is_high) {
                (true, true) => GateState::High,
                (true, false) => GateState::Falling,
                (false, true) => GateState::Rising,
                (false, false) => GateState::Low,
            };
            gate_was_high = gate_is_high;
            let trigger = interrupt::free(|cs| {
                let mutex = QUEUED_TRIGGER.borrow(cs);
                let value = mutex.get();
                mutex.set(false);
                value
            });
            let input = Input { gate, trigger };
            let (value, did_change_phase) = update(&mut envelope_state, &input, &cv);
            dac.write_keep_cs_pin_low(&mut spi, DacChannel::ChannelA, value, Default::default());
            unsafe_access_mutex(|cs| DAC_WRITE_QUEUED.borrow(cs).set(true));

            if did_change_phase {
                aux_output_pin
                    .set_state(update_aux(&envelope_state.mode, &config).into())
                    .unwrap_infallible();
                if display == DisplayMode::ShowEnvelopeSegment {
                    ui.update(ui_show_stage(&envelope_state.mode));
                }
            }
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

struct UI {
    _d4: Pin<Output, port::PD4>,
    _d5: Pin<Output, port::PD5>,
    _d6: Pin<Output, port::PD6>,
    _d7: Pin<Output, port::PD7>,
}

impl UI {
    fn new(
        d4: Pin<Output, port::PD4>,
        d5: Pin<Output, port::PD5>,
        d6: Pin<Output, port::PD6>,
        d7: Pin<Output, port::PD7>,
    ) -> Self {
        UI {
            _d4: d4,
            _d5: d5,
            _d6: d6,
            _d7: d7,
        }
    }

    fn update(&self, ui_state: u8) {
        debug_assert!(ui_state & 0xf == 0);
        unsafe {
            let dp = arduino_hal::Peripherals::steal();
            dp.PORTD
                .portd
                .modify(|r, w| w.bits(r.bits() & 0xf | ui_state))
        }
    }
}
