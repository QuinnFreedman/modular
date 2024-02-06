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

use core::panic::PanicInfo;

use arduino_hal::{delay_ms, prelude::*, Peripherals};
use fm_lib::{configure_system_clock, rng::ParallelLfsr, rotary_encoder::RotaryEncoderHandler};
use ufmt::uwriteln;

use crate::{
    async_dac::{handle_conversion_result, init_async_adc, AsyncAdc, Indexable},
    led_driver::TLC5940,
    rng::RngModule,
};

mod async_dac;
mod led_driver;
mod rng;

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
    // TODO switch connections
    let b = port.pd4().bit_is_set();
    let a = port.pd5().bit_is_set();
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

static mut GLOBAL_ASYNC_ADC_STATE: AsyncAdc<4> = None;

#[avr_device::interrupt(atmega328p)]
fn ADC() {
    handle_conversion_result(unsafe { &mut GLOBAL_ASYNC_ADC_STATE });
}

enum AnalogChannel {
    Chance,
    Bias,
    BiasCV,
    GateTrigSwitch,
}

impl Into<u16> for AnalogChannel {
    fn into(self) -> u16 {
        self as u16
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    // Enable interrupts for rotary encoder
    enable_portd_pc_interrupts(&dp);

    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
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
        unsafe { &mut GLOBAL_ASYNC_ADC_STATE },
        [
            arduino_hal::adc::channel::ADC7.into_channel(),
            a4.into_channel(),
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
    let sys_clock = system_clock::init_system_clock(dp.TC0);

    uwriteln!(&mut serial, "Seed: {}", seed).unwrap_infallible();

    let mut prng = ParallelLfsr::new(seed);
    let mut rng_module = RngModule::<MAX_BUFFER_SIZE, NUM_LEDS>::new(&mut prng);

    loop {
        uwriteln!(
            &mut serial,
            "Chance: {}, Bias: {}, ChanceCV {}, Trig: {}",
            unsafe { GLOBAL_ASYNC_ADC_STATE.get(AnalogChannel::Chance) },
            unsafe { GLOBAL_ASYNC_ADC_STATE.get(AnalogChannel::Bias) },
            unsafe { GLOBAL_ASYNC_ADC_STATE.get(AnalogChannel::BiasCV) },
            unsafe { GLOBAL_ASYNC_ADC_STATE.get(AnalogChannel::GateTrigSwitch) },
        )
        .unwrap_infallible();
        delay_ms(100);

        // let current_time = sys_clock.millis();
        // let re_delta = ROTARY_ENCODER.sample_and_reset();
        // if re_delta != 0 {
        //     let size_change = if encoder_switch.is_low() {
        //         SizeAdjustment::ExactDelta(re_delta)
        //     } else {
        //         SizeAdjustment::PowersOfTwo(re_delta)
        //     };
        //     rng_module.adjust_buffer_size(size_change, current_time);
        // }

        // rng_module.render_display_if_needed(
        //     current_time,
        //     |buffer: &[u16; NUM_LEDS as usize]| -> Result<(), ()> {
        //         led_driver.write(&mut spi, buffer)
        //     },
        // );
    }
}
