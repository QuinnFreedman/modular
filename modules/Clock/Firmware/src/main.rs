#![allow(incomplete_features)]
#![no_std]
#![no_main]
#![feature(generic_const_exprs)]
#![feature(int_roundings)]
#![feature(abi_avr_interrupt)]
#![feature(inline_const_pat)]
#![feature(const_trait_impl)]
#![feature(adt_const_params)]

mod clock;
mod display_buffer;
mod eeprom;
mod font;
mod menu;
mod random;
mod render_numbers;

use arduino_hal::hal::port::{PC3, PC4};
use avr_device::interrupt;
use clock::{ClockConfig, ClockState};
use core::panic::PanicInfo;
use eeprom::PersistanceManager;
use fm_lib::button_debouncer::{ButtonWithLongPress, LongPressButtonState};
use fm_lib::debug_unwrap::DebugUnwrap;
use fm_lib::handle_system_clock_interrupt;
use fm_lib::rotary_encoder::RotaryEncoderHandler;
use fm_lib::system_clock::{ClockPrecision, GlobalSystemClockState, SystemClock};
use menu::{render_menu, update_menu, MenuOrScreenSaverState, MenuUpdate};
use ssd1306::{prelude::*, Ssd1306};

static SYSTEM_CLOCK_STATE: GlobalSystemClockState<{ ClockPrecision::MS16 }> =
    GlobalSystemClockState::new();
handle_system_clock_interrupt!(&SYSTEM_CLOCK_STATE);

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    interrupt::disable();

    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    const SHORT: u16 = 100;
    const LONG: u16 = 500;
    let mut led = pins.d13.into_output();
    loop {
        for len in [SHORT, LONG] {
            for _ in 0..3u8 {
                led.set_high();
                arduino_hal::delay_ms(len);
                led.set_low();
                arduino_hal::delay_ms(SHORT);
            }
        }
    }
}

static ROTARY_ENCODER: RotaryEncoderHandler = RotaryEncoderHandler::new();

/**
 Pin-change interrupt handler for port B (pins d8-d13)
*/
#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT0() {
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let port = dp.PORTB.pinb.read();
    let a = port.pb0().bit_is_set();
    let b = port.pb1().bit_is_set();
    ROTARY_ENCODER.update(a, b);
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().assert_ok();

    let mut clock_config = ClockConfig::new();
    let mut persistance_manager = PersistanceManager::new(dp.EEPROM, &mut clock_config);

    // start system clock
    let sys_clock = SystemClock::init_system_clock(dp.TC0, &SYSTEM_CLOCK_STATE);

    // set pins d0-d7 as output
    dp.PORTD.ddrd.write(|w| unsafe { w.bits(0xff) });

    // Enable pin-change interrupts on pins d8 and d9
    dp.PORTB.ddrb.write(|w| w.pb0().bit(false).pb1().bit(false));
    dp.PORTB.portb.write(|w| w.pb0().bit(true).pb1().bit(true));
    dp.EXINT.pcifr.reset();
    dp.EXINT.pcmsk0.write(|w| w.pcint().bits(0b00000011));
    dp.EXINT.pcicr.write(|w| w.pcie().bits(0b001));

    // turn on interrupts
    unsafe {
        avr_device::interrupt::enable();
    };

    // setup display
    let pins = arduino_hal::pins!(dp);
    let mut display = {
        let (spi, _) = arduino_hal::spi::Spi::new(
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
        let interface = display_interface_spi::SPIInterface::new(
            spi,
            pins.a1.into_output(),
            pins.a2.into_output(),
        );

        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0);
        display
            .reset(&mut pins.a0.into_output(), &mut arduino_hal::Delay::new())
            .assert_ok();
        display
            .init_with_addr_mode(ssd1306::command::AddrMode::Vertical)
            .assert_ok();
        display.clear().assert_ok();
        display
    };

    let encoder_button_pin = pins.a4.into_pull_up_input();
    let pause_button_pin = pins.a3.into_pull_up_input();

    if encoder_button_pin.is_low() {
        clock_config.invert_encoder = !clock_config.invert_encoder;
        persistance_manager.set_invert_encoder(clock_config.invert_encoder);
    }

    if clock_config.invert_encoder {
        ROTARY_ENCODER.invert();
    } 

    // set up app state
    let mut encoder_button = ButtonWithLongPress::<PC4, 32, 500>::new(encoder_button_pin);
    let mut pause_button = ButtonWithLongPress::<PC3, 32, 2500>::new(pause_button_pin);
    let mut menu_state = MenuOrScreenSaverState::new(0);
    let mut clock_state = ClockState::new();
    let mut is_paused = false;
    let mut start_time: u64 = 0;

    render_menu(
        &menu_state,
        &clock_config,
        &MenuUpdate::SwitchScreens,
        &mut display,
    );

    // I want to use direct port manipulation for performance but also the
    // individual pins from the board support library for convenience at the
    // same time, which isn't allowed by the borrow checker, so I have to use
    // an unsafe copy of the references to the ports here
    let unsafe_peripherals = unsafe { arduino_hal::Peripherals::steal() };

    // Main loop. Will run for the rest of the program
    loop {
        // use ms for menu logic but use micros for clock to reduce aliasing
        let current_time_us = sys_clock.micros();
        let current_time_ms = (current_time_us / 1000) as u32;
        // Handle pause button
        let pause_button_state = pause_button.sample(current_time_ms);
        match pause_button_state {
            LongPressButtonState::ButtonJustDown => {
                is_paused = !is_paused;
                if !is_paused {
                    clock_state.reset();
                    start_time = current_time_us;
                }
                menu_state = MenuOrScreenSaverState::new(current_time_ms);
                render_menu(
                    &menu_state,
                    &clock_config,
                    // TODO this causes a slight flicker. Calculate what the actual
                    // update should be
                    &MenuUpdate::SwitchScreens,
                    &mut display,
                );
            }
            LongPressButtonState::ButtonJustClickedLong => {
                clock_config = ClockConfig::new();
                clock_state.reset();
                start_time = current_time_us;
                menu_state = MenuOrScreenSaverState::new(current_time_ms);
                is_paused = false;
                render_menu(
                    &menu_state,
                    &clock_config,
                    &MenuUpdate::SwitchScreens,
                    &mut display,
                );
                persistance_manager.overwrite(&clock_config);
            }
            _ => {}
        }

        // Handle clock logic and write clock state to output pins
        let clock_time = current_time_us.wrapping_sub(start_time);
        let (pin_state, did_rollover) =
            clock::sample(&clock_config, &mut clock_state, clock_time, is_paused);
        unsafe_peripherals
            .PORTD
            .portd
            .write(|w| unsafe { w.bits(pin_state) });

        // Handle menu logic
        let menu_update = update_menu(
            &mut menu_state,
            &mut clock_config,
            &mut encoder_button,
            &ROTARY_ENCODER,
            current_time_ms,
            did_rollover,
            &mut persistance_manager,
        );

        // Only re-render the part of the screen that needs to be updated, if any
        if menu_update != MenuUpdate::NoUpdate {
            render_menu(&menu_state, &clock_config, &menu_update, &mut display);
        }
    }
}
