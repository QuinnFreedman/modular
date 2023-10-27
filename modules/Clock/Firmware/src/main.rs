#![no_std]
#![no_main]
#![feature(generic_const_exprs)]
#![feature(int_roundings)]
#![feature(panic_info_message)]
#![feature(abi_avr_interrupt)]
#![feature(inline_const_pat)]
#![feature(const_trait_impl)]
#![feature(const_convert)]

mod button_debouncer;
mod clock;
mod display_buffer;
mod font;
mod menu;
mod random;
mod render_nubers;
mod rotary_encoder;
mod system_clock;

use arduino_hal::hal::port::PC0;
use button_debouncer::ButtonWithLongPress;
use clock::{ClockConfig, ClockState};
use core::panic::PanicInfo;
use menu::{render_menu, update_menu, MenuOrScreenSaverState, MenuUpdate};
use rotary_encoder::RotaryEncoderHandler;
use ssd1306::{prelude::*, Ssd1306};
use system_clock::{millis, millis_init};

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        #[allow(non_snake_case)]
        let SPCR: *mut u8 = 0x4C as *mut u8;
        let spe_mask = 1u8 << 6u8;
        *SPCR = *SPCR & !spe_mask;
    }

    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    // serial.write_str(F!("PANIC: ")).unwrap();
    // serial
    //     .write_str(
    //         info.message()
    //             .and_then(|x| x.as_str())
    //             .unwrap_or(F!("(no message)")),
    //     )
    //     .unwrap();
    // serial.write_byte('\n' as u8);

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
    let dp = arduino_hal::Peripherals::take().unwrap();

    // start system clock
    millis_init(dp.TC0);

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
            pins.a3.into_output(),
            pins.a4.into_output(),
        );

        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0);
        display
            .reset(&mut pins.a2.into_output(), &mut arduino_hal::Delay::new())
            .unwrap();
        display
            .init_with_addr_mode(ssd1306::command::AddrMode::Vertical)
            .unwrap();
        display.clear().unwrap();
        display
    };

    let mut button = ButtonWithLongPress::<PC0, 50, 500>::new(pins.a0.into_pull_up_input());
    let mut menu_state = MenuOrScreenSaverState::new();
    let mut clock_config = ClockConfig::new();
    let mut clock_state = ClockState::new();

    render_menu(
        &menu_state,
        &clock_config,
        &MenuUpdate::SwitchScreens,
        &mut display,
    );

    let unsafe_peripherals = unsafe { arduino_hal::Peripherals::steal() };
    loop {
        let current_time_ms = millis();
        let (pin_state, did_rollover) =
            clock::sample(&clock_config, &mut clock_state, current_time_ms);
        unsafe_peripherals
            .PORTD
            .portd
            .write(|w| unsafe { w.bits(pin_state) });

        let menu_update = update_menu(
            &mut menu_state,
            &mut clock_config,
            &mut button,
            &ROTARY_ENCODER,
            current_time_ms,
            did_rollover,
        );

        if menu_update != MenuUpdate::NoUpdate {
            render_menu(&menu_state, &clock_config, &menu_update, &mut display);
        }
    }
}
