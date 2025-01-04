#![allow(incomplete_features)]
#![no_std]
#![feature(const_trait_impl)]
#![feature(inline_const_pat)]
#![feature(asm_experimental_arch)]
#![feature(adt_const_params)]
#![feature(abi_avr_interrupt)]

pub mod async_adc;
pub mod asynchronous;
pub mod bit_ops;
pub mod button_debouncer;
pub mod const_traits;
pub mod debug_unwrap;
pub mod eeprom;
pub mod mcp4922;
pub mod number_utils;
pub mod nybl_pair;
pub mod rng;
pub mod rotary_encoder;
pub mod system_clock;

#[cfg(feature = "panic")]
pub mod panic;
