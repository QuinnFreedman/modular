#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]
#![feature(int_roundings)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]

use core::panic::PanicInfo;

use arduino_hal::{delay_ms, prelude::*};
use fm_lib::rotary_encoder::RotaryEncoderHandler;
use led_driver::TLC5940;
use ufmt::uwriteln;

mod led_driver;

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

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);
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

    let led_driver = TLC5940::<7>::new(&mut spi, pwm_ref, d10, xlatch, dp.TC1, dp.TC2);
    // turn on interrupts
    unsafe {
        avr_device::interrupt::enable();
    };

    loop {
        let mut data = [0u16; 7];
        led_driver.sync_write(&mut spi, &data);
        delay_ms(300);
        for i in 0..7 {
            data[i] = 0xfff;
            led_driver.sync_write(&mut spi, &data);
            delay_ms(300);
        }
        for i in 0..7 {
            data[i] = 0x111;
            led_driver.sync_write(&mut spi, &data);
            delay_ms(300);
        }
    }
}
