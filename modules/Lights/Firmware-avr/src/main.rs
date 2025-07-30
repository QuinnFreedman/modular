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

use core::arch::asm;
use core::borrow::BorrowMut;
use core::cell::Cell;
use core::cell::RefCell;
use core::default;

use arduino_hal::adc::AdcSettings;
use arduino_hal::delay_ms;
use arduino_hal::delay_us;
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
use fixed::types::U16F16;
use fm_lib::async_adc::new_averaging_async_adc_state;
use fm_lib::asynchronous::assert_interrupts_disabled;
use fm_lib::asynchronous::AtomicRead as _;
use fm_lib::button_debouncer::ButtonWithLongPress;
use fm_lib::mcp4922::DacChannel;
use fm_lib::mcp4922::MCP4922;
use fm_lib::{
    async_adc::{handle_conversion_result, init_async_adc, AsyncAdc, GetAdcValues},
    asynchronous::Borrowable,
    handle_system_clock_interrupt,
    system_clock::{ClockPrecision, GlobalSystemClockState, SystemClock},
};
use ufmt::uwrite;
use ufmt::uwriteln;

// const ARRAY_SIZE: usize = 16;

// static GLOBAL_ASYNC_ADC_STATE: AsyncAdc<3, 3> = new_averaging_async_adc_state();
// static SAMPLE_ARRAY: Mutex<RefCell<[u16; ARRAY_SIZE]>> = Mutex::new(RefCell::new([0; ARRAY_SIZE]));
// static SAMPLE_ARRAY_INDEX: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0));

// #[avr_device::interrupt(atmega328p)]
// fn ADC() {
//     handle_conversion_result(&GLOBAL_ASYNC_ADC_STATE);
// }

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    // let mut adc = arduino_hal::Adc::new(dp.ADC, AdcSettings::default());
    // let (mut spi, d10) = arduino_hal::spi::Spi::new(
    //     dp.SPI,
    //     pins.d13.into_output(),        // Clock
    //     pins.d11.into_output(),        // MOSI
    //     pins.d12.into_pull_up_input(), // MISO
    //     pins.d10.into_output(),        // CS
    //     arduino_hal::spi::Settings {
    //         data_order: arduino_hal::spi::DataOrder::MostSignificantFirst,
    //         clock: arduino_hal::spi::SerialClockRate::OscfOver2,
    //         mode: embedded_hal::spi::MODE_0,
    //     },
    // );

    // let d2 = pins.d2.into_floating_input();
    let mut d13 = pins.d13.into_output();

    // init_async_adc(
    //     adc,
    //     &GLOBAL_ASYNC_ADC_STATE,
    //     [
    //         a5.into_channel(),
    //         arduino_hal::adc::channel::ADC6.into_channel(),
    //         arduino_hal::adc::channel::ADC7.into_channel(),
    //     ],
    // );

    unsafe {
        avr_device::interrupt::enable();
    };

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    // let mut dac = MCP4922::new(d10);

    let mut led_pin = pins.d6.into_output(); // PD6

    let data: [u8; 12] = [
        0x00, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0xFF,
    ]; // GRB = red

    unsafe {
        send_ws2811(&data);
    }

    loop {
        d13.set_high();
        delay_ms(100);
        d13.set_low();
        delay_ms(1000);

        // let mut array = [0u16; ARRAY_SIZE];
        // let ready_to_analyze = interrupt::free(|cs| {
        //     let mut idx = SAMPLE_ARRAY_INDEX.borrow(cs).borrow_mut();
        //     if *idx == ARRAY_SIZE as u8 {
        //         *idx = 0;
        //         array = *SAMPLE_ARRAY.borrow(cs).borrow_mut();
        //         true
        //     } else {
        //         false
        //     }
        // });
        // if ready_to_analyze {
        //     let (mean, std_dev) = analyze_samples(&array);
        //     uwriteln!(&mut serial, "{}, {}", mean, std_dev).unwrap_infallible();
        // }
    }
}

unsafe fn send_ws2811(values: &[u8]) {
    // Pin: PD6, Bit 6
    const PORTD_ADDR: u8 = 0x0B; // I/O address of PORTD
    const PIN: u8 = 6;

    let data = values.as_ptr();
    let len = values.len() as u16;

    // Disable interrupts
    avr_device::interrupt::disable();

    asm!(
        "ldi  r18, 8",                 // bit counter
        "ld   r16, Z+",                // load first byte from *data into r16

        "1:",                          // start outer loop (bytes)
        "ldi  r18, 8",                 // reset bit counter

        "2:",                          // start inner loop (bits)
        "sbrc r16, 7",                 // test MSB
        "rjmp 3f",

        // --- bit = 0 ---
        "sbi  {port}, {pin}",          // 2 cycles HIGH
        "nop",                         // 1
        "cbi  {port}, {pin}",          // 2 cycles LOW
        "nop", "nop", "nop", "nop",    // 4
        "nop", "nop", "nop", "nop",    // 4
        "nop", "nop", "nop", "nop",    // 4
        "rjmp 4f",                     // 2 (total ~20)

        // --- bit = 1 ---
        "3:",
        "sbi  {port}, {pin}",          // 2
        "nop", "nop", "nop", "nop",    // 4
        "nop", "nop", "nop", "nop",    // 4
        "nop", "nop",                  // 2 (12 total high)
        "cbi  {port}, {pin}",          // 2
        "nop", "nop", "nop", "nop",    // 4
        "nop", "nop",                  // 2

        "4:",
        "lsl  r16",                    // shift left
        "dec  r18",                    // bit count--
        "brne 2b",                     // repeat 8 bits

        "sbiw {len_lo}, 1",           // len--
        "brne 5f",

        "rjmp 6f",

        "5:",
        "ld   r16, Z+",               // load next byte
        "rjmp 1b",

        "6:",
        port = const PORTD_ADDR,
        pin  = const PIN,

        len_lo = inout(reg_pair) len => _,
        in("Z") data,
        out("r16") _,
        out("r18") _,
    );

    // Re-enable interrupts
    avr_device::interrupt::enable();

    // Reset time
    delay_us(60);
}
