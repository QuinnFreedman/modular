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

use arduino_hal::{hal::port::PB0, prelude::*, Peripherals};
use avr_device::interrupt::{self, Mutex};
use embedded_hal::digital::v2::OutputPin;
use envelope::{
    ui_show_mode, ui_show_stage, update, AcrcLoopState, AcrcState, AdsrState, AhrdState,
    EnvelopeState,
};
use fm_lib::asynchronous::Borrowable;
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

mod envelope;

static SYSTEM_CLOCK_STATE: GlobalSystemClockState<{ ClockPrecision::MS16 }> =
    GlobalSystemClockState::new();
handle_system_clock_interrupt!(&SYSTEM_CLOCK_STATE);

const UI_SHOW_ENVELOPE_MODE_MS: u32 = 2000;
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

/**
 Pin-change interrupt handler pin d2 (external interrupt 0)
*/
#[avr_device::interrupt(atmega328p)]
fn INT0() {
    // assert_interrupts_disabled(|cs| UNHANDLED_STEP_COUNT.borrow(cs).update(|x| x + 1));
}

#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {
    let dp = unsafe { arduino_hal::Peripherals::steal() };

    assert_interrupts_disabled(|cs| {
        if DAC_WRITE_READY.borrow(cs).get() {
            DAC_WRITE_READY.borrow(cs).set(false);
        } else {
            DEBUG_SKIPPED_WRITE_COUNT.borrow(cs).update(|x| x + 1);
        }
    });

    dp.PORTB
        .portb
        .modify(|r, w| unsafe { w.bits(r.bits()) }.pb2().set_bit());
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

static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<4> = new_async_adc_state();

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
}

enum CvChannel {
    CV1,
    CV2,
    CV3,
    CV4,
}

impl Into<usize> for CvChannel {
    fn into(self) -> usize {
        self as usize
    }
}

impl EnvelopeState {
    fn next(self) -> Self {
        match self {
            EnvelopeState::Adsr(_) => EnvelopeState::Acrc(AcrcState::default()),
            EnvelopeState::Acrc(_) => EnvelopeState::AcrcLoop(AcrcLoopState::default()),
            EnvelopeState::AcrcLoop(_) => EnvelopeState::AhrdLoop(AhrdState::default()),
            EnvelopeState::AhrdLoop(_) => EnvelopeState::Adsr(AdsrState::default()),
        }
    }
}

static DAC_WRITE_READY: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static DEBUG_SKIPPED_WRITE_COUNT: Mutex<Cell<u16>> = Mutex::new(Cell::new(0));

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    enable_external_interrupts(&dp);

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

    // let mut dac = MCP4922::new(pins.d8.into_output_high());

    let mut leds = [
        pins.d4.into_output().downgrade(),
        pins.d5.into_output().downgrade(),
        pins.d6.into_output().downgrade(),
        pins.d7.into_output().downgrade(),
    ];

    let d8 = pins.d8.into_pull_up_input();
    let mut button = ButtonDebouncer::<PB0, 32>::new(d8);

    let sys_clock = SystemClock::init_system_clock(dp.TC0, &SYSTEM_CLOCK_STATE);

    let mut envelope_state = EnvelopeState::Adsr(AdsrState::default());
    let mut t: u32 = 0;

    let mut display = DisplayMode::ShowEnvelopeMode {
        until: UI_SHOW_ENVELOPE_MODE_MS,
    };

    let ui_state = ui_show_mode(&envelope_state);
    for i in 0..4u8 {
        leds[i as usize]
            .set_state(ui_state[i as usize].into())
            .unwrap();
    }

    configure_timer(&dp.TC2);
    let mut dac = MCP4922::new(d10);

    let mut debug_skip_count_last = 0u16;

    let mut debug_last_log_time: u32 = 0;
    const DEBUG_LOG_INTERVAL: u32 = 500;

    loop {
        let cv = interrupt::free(|cs| GLOBAL_ASYNC_ADC_STATE.get_inner(cs).get_all());

        let current_time = sys_clock.millis_exact();
        let button_state = button.sample(current_time);
        let button_was_pressed = match button_state {
            fm_lib::button_debouncer::ButtonState::ButtonJustPressed => true,
            _ => false,
        };

        if button_was_pressed {
            envelope_state = envelope_state.next();
            t = 0;
            display = DisplayMode::ShowEnvelopeMode {
                until: current_time + UI_SHOW_ENVELOPE_MODE_MS,
            };
            let ui_state = ui_show_mode(&envelope_state);
            for i in 0..4u8 {
                leds[i as usize]
                    .set_state(ui_state[i as usize].into())
                    .unwrap();
            }
        }

        // if current_time - debug_last_log_time >= DEBUG_LOG_INTERVAL {
        //     debug_last_log_time = current_time;
        //     // uwriteln!(&mut serial, "{}", current_time).unwrap_infallible();
        //     uwriteln!(&mut serial, "{}, {}, {}, {}", cv[0], cv[1], cv[2], cv[3])
        //         .unwrap_infallible();
        // }

        if let DisplayMode::ShowEnvelopeMode { until } = display {
            if current_time > until {
                display = DisplayMode::ShowEnvelopeSegment;
                let ui_state = ui_show_stage(&envelope_state);
                for i in 0..4u8 {
                    leds[i as usize]
                        .set_state(ui_state[i as usize].into())
                        .unwrap();
                }
            }
        }

        if !unsafe_access_mutex(|cs| DAC_WRITE_READY.borrow(cs).get()) {
            let (value, did_change_phase) = update(&mut envelope_state, &mut t, cv);
            dac.write_keep_cs_pin_low(&mut spi, DacChannel::ChannelA, value, Default::default());
            unsafe_access_mutex(|cs| DAC_WRITE_READY.borrow(cs).set(true));

            if did_change_phase {
                let ui_state = ui_show_stage(&envelope_state);
                for i in 0..4u8 {
                    leds[i as usize]
                        .set_state(ui_state[i as usize].into())
                        .unwrap();
                }
            }
        }

        if unsafe_access_mutex(|cs| DEBUG_SKIPPED_WRITE_COUNT.borrow(cs).get())
            != debug_skip_count_last
        {
            interrupt::free(|cs| {
                let num_skipped = DEBUG_SKIPPED_WRITE_COUNT.borrow(cs).get();
                uwriteln!(&mut serial, "skipped: {}", num_skipped).unwrap_infallible();
                debug_skip_count_last = num_skipped;
            })
        }
    }
}

fn configure_timer(tc2: &arduino_hal::pac::TC2) {
    // reset timer counter at TOP set by OCRA
    tc2.tccr2a.write(|w| w.wgm2().ctc());
    // set timer frequency to cycle at 5kHz
    // (16MHz clock speed / 64 prescale factor / 50 count/reset )
    tc2.tccr2b.write(|w| w.cs2().prescale_64());
    tc2.ocr2a.write(|w| w.bits(50));

    // enable interrupt on match to compare register A
    tc2.timsk2.write(|w| w.ocie2a().set_bit());
}
