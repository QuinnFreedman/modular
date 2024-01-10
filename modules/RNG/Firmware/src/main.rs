#![allow(incomplete_features)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]
#![feature(int_roundings)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]

use core::panic::PanicInfo;

use arduino_hal::{prelude::*, Peripherals};
use fm_lib::{configure_system_clock, rotary_encoder::RotaryEncoderHandler};
use led_driver::TLC5940;
use ufmt::uwriteln;

mod led_driver;

configure_system_clock!(ClockPrecision::MS16);

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
        .void_unwrap();
    } else {
        uwriteln!(&mut serial, "Panic occurred").void_unwrap();
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
#[allow(non_snake_case)]
fn PCINT2() {
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let port = dp.PORTD.pind.read();
    let a = port.pd4().bit_is_set();
    let b = port.pd5().bit_is_set();
    ROTARY_ENCODER.update(a, b);
}

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

enum DisplayMode {
    ShowBuffer,
    ShowBufferLengthSince(u32),
}

const NUM_LEDS: u8 = 7;
fn binary_representation_as_display_buffer(n: u8) -> [u16; NUM_LEDS as usize] {
    let mut result = [0u16; NUM_LEDS as usize];
    for i in 0..NUM_LEDS - 1 {
        result[i as usize] = if n as u16 & (1 << i as u16) == 0 {
            0
        } else {
            0xfff
        };
    }
    result
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    // Enable interrupts for rotary encoder
    enable_portd_pc_interrupts(&dp);

    let pins = arduino_hal::pins!(dp);
    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
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
    let xlatch = pins.d9.into_output();
    let pwm_ref = pins.d3.into_output();

    let led_driver =
        TLC5940::<{ NUM_LEDS as usize }>::new(&mut spi, pwm_ref, d10, xlatch, dp.TC1, dp.TC2);
    let sys_clock = system_clock::init_system_clock(dp.TC0);

    unsafe {
        avr_device::interrupt::enable();
    };

    let mut display_needs_update: bool = true;
    let mut display_mode: DisplayMode = DisplayMode::ShowBuffer;
    let mut buffer_size: u8 = 8;
    const MAX_BUFFER_SIZE: u8 = 32;
    const BUFFER_LEN_DISPLAY_TIME_MS: u32 = 3000;
    loop {
        let current_time = sys_clock.millis();
        let re_delta = ROTARY_ENCODER.sample_and_reset();
        if re_delta != 0 {
            buffer_size = buffer_size
                .saturating_add_signed(re_delta)
                .clamp(0, MAX_BUFFER_SIZE);
            display_mode = DisplayMode::ShowBufferLengthSince(current_time);
            display_needs_update = true;
        }

        if let DisplayMode::ShowBufferLengthSince(start_time) = display_mode {
            if current_time > start_time + BUFFER_LEN_DISPLAY_TIME_MS {
                display_mode = DisplayMode::ShowBuffer;
                display_needs_update = true;
            }
        }

        if display_needs_update {
            let to_write = match display_mode {
                DisplayMode::ShowBuffer => [0xfffu16; NUM_LEDS as usize], // DEBUG PLACEHOLDER
                DisplayMode::ShowBufferLengthSince(_) => {
                    binary_representation_as_display_buffer(buffer_size)
                }
            };
            if let Ok(()) = led_driver.write(&mut spi, &to_write) {
                display_needs_update = false;
            }
        }
    }
}
