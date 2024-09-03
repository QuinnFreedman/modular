use arduino_hal::prelude::*;
use core::panic::PanicInfo;
use ufmt::uwriteln;

use avr_device::interrupt;

/**
This is a generic panic handler suitable for most modules. When a panic occurs, it will:
1. Halt main execution without stack trace or unwind
2. Turn off all interrupts and stop SPI communication
3. Write debug info to UART if available (in release builds debug info will be removed for performance & size)
4. Flash onboard LED in alternating short/long pulses to indicate error
*/
#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interrupt::disable();
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    dp.SPI.spcr.write(|w| w.spe().clear_bit());

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
