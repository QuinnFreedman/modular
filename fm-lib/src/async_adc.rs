//! This module provides an interface to continuously and asynchronously poll the
//! analog values of any number of pins at the maximum possible speed. The most
//! recently converted value for each channel can be read back at any time.
//!
//! The ATmega uses a successive approximation ADC. It takes 13 ADC cycles to
//! read a single value, at a max ADC clock speed of 200kHz. This module takes
//! advantage of the ATmega ADC's "free running mode," which automatically starts
//! the next ADC conversion after each one completes. Additionally, it uses the
//! ADC interrupt to run an interrupt handler after each successful conversion. The
//! interrupt handler stores the most recently converted result in a buffer and
//! updates the ADC to switch to reading the next channel, reading all configured
//! channels in a loop.
//!
//! The ADC will not change any of its parameters once a conversion has begun.
//! Changes to the input channel or other options are buffered until the current
//! conversion completes. This means that timing is not an issue; as long as the
//! interrupt is handled before the next conversion starts, there is no delay or
//! lost information. It also means that the new channel set in the interrupt
//! handler will not be resolved until the result after next, which means this
//! module has to essentially plan 2 cycles ahead.

use core::{cell::UnsafeCell, mem::MaybeUninit};

use arduino_hal::{
    adc::{AdcChannel, Channel},
    Adc,
};
use avr_device::interrupt::Mutex;

use crate::asynchronous::{assert_interrupts_disabled, unsafe_access_mutex, Borrowable as _};

pub struct AsyncAdcState<const CHANNELS: usize, const WINDOW: usize = 1> {
    channels: MaybeUninit<[Channel; CHANNELS]>,
    values: [[u16; WINDOW]; CHANNELS],
    channel_cursor: u8,
    time_cursor: u8,
}

/**
An AsyncAdcState wrapped in the necessary synchronization primitives so it is
thread-safe.

The thread safety is not just to appease the borrow checker; it is
necessary to always access the ADC values through a critical section guard because
the values are 16 bit ints, which cannot be written to atomically. So if you try
to read a value unsafely, it could be updated by the ADC interrupt mid read, which
could cause spikes e.g. when going from 255 -> 256 would actually register as 0 or
273 depending on write order.
 */
pub type AsyncAdc<const CHANNELS: usize, const WINDOW: usize = 1> =
    Mutex<UnsafeCell<Option<AsyncAdcState<CHANNELS, WINDOW>>>>;

pub const fn new_async_adc_state<const N: usize>() -> AsyncAdc<N, 1> {
    Mutex::new(UnsafeCell::new(None))
}

pub const fn new_averaging_async_adc_state<const CHANNELS: usize, const WINDOW: usize>(
) -> AsyncAdc<CHANNELS, WINDOW> {
    Mutex::new(UnsafeCell::new(None))
}

pub trait Indexable {
    type Result;
    fn get<T>(&self, i: T) -> Self::Result
    where
        T: Into<usize>;
}

impl<const CHANNELS: usize, const WINDOW: usize> Indexable
    for Option<AsyncAdcState<CHANNELS, WINDOW>>
{
    type Result = u16;
    /**
    Get the value at `index`. This is checked in debug mode but unchecked in release
    */
    #[inline(always)]
    fn get<T>(&self, index: T) -> u16
    where
        T: Into<usize>,
    {
        let i: usize = index.into();
        debug_assert!(i < CHANNELS);
        debug_assert!(self.is_some());
        let adc = unsafe { self.as_ref().unwrap_unchecked() };
        unsafe { average(adc.values.get_unchecked(i as usize)) }
    }
}

fn average<const N: usize>(values: &[u16; N]) -> u16 {
    if N == 1 {
        return values[0];
    }

    let mut sorted = values.clone();
    for i in 1..sorted.len() {
        let mut j = i;
        while j > 0 && sorted[j - 1] > sorted[j] {
            sorted.swap(j - 1, j);
            j -= 1;
        }
    }

    sorted[N / 2]
}

pub trait GetAdcValues<const N: usize> {
    fn get_all(&self) -> [u16; N];
}

impl<const N: usize, const W: usize> GetAdcValues<N> for Option<AsyncAdcState<N, W>> {
    fn get_all(&self) -> [u16; N] {
        debug_assert!(self.is_some());
        let adc = unsafe { self.as_ref().unwrap_unchecked() };
        adc.values.map(|x| average(&x))
    }
}

/**
Creates the ADC state struct and stores it as a static global. Also initializes
the hardware ADC, enabling free running mode and starting the conversion.
 */
pub fn init_async_adc<const CHANNELS: usize, const WINDOW: usize>(
    mut adc: Adc,
    async_adc_state: &AsyncAdc<CHANNELS, WINDOW>,
    channels: [Channel; CHANNELS],
) {
    let ch1 = channels[0].channel();
    let ch2 = channels[1].channel();

    let mut values = unsafe { MaybeUninit::<[[u16; WINDOW]; CHANNELS]>::uninit().assume_init() };
    for j in 0..WINDOW {
        for i in 0..CHANNELS {
            values[i][j] = adc.read_blocking(&channels[i]);
        }
    }

    unsafe_access_mutex(|cs| {
        let inner = async_adc_state.get_inner_mut(cs);
        debug_assert!(inner.is_none());
        *inner = Some(AsyncAdcState {
            channels: MaybeUninit::new(channels),
            values,
            channel_cursor: 0,
            time_cursor: 0,
        });
    });

    let dp = unsafe { arduino_hal::Peripherals::steal() };

    dp.ADC.admux.write(|w| w.mux().variant(ch1).refs().avcc());
    // set auto trigger source to free run mode
    dp.ADC.adcsrb.write(|w| w.adts().val_0x00());
    dp.ADC.adcsra.write(|w| {
        w.aden().set_bit(); // enable ADC
        w.adsc().set_bit(); // start conversion
        w.adate().set_bit(); // auto trigger (free run)
        w.adie().set_bit(); // enable interrupt on conversion end
        w.adps().prescaler_128() // prescaler of 128 required for full accuracy. 64 works fine at the cost of some LSBs
    });
    dp.ADC
        .admux
        .modify(|r, w| unsafe { w.bits(r.bits()) }.mux().variant(ch2));
}

fn advance_cursor<const N: usize, const W: usize>(adc: &AsyncAdcState<N, W>) -> (u8, u8) {
    let mut next_channel = adc.channel_cursor + 1;
    let mut next_time_step = adc.time_cursor;
    if next_channel >= N as u8 {
        next_channel = 0;
        next_time_step += 1;
        if next_time_step >= W as u8 {
            next_time_step = 0;
        }
    }
    (next_channel, next_time_step)
}

/**
This function must be called after an ADC conversion completes, in the ADC
interrupt handler. It reads the most recent ADC conversion result and stores it,
then advances the ADC input channel by one.
*/
#[inline(always)]
pub fn handle_conversion_result<const N: usize, const W: usize>(adc: &AsyncAdc<N, W>) {
    assert_interrupts_disabled(|cs| {
        let inner = adc.get_inner_mut(cs).as_mut();
        debug_assert!(inner.is_some());
        let adc = unsafe { inner.unwrap_unchecked() };
        let dp = unsafe { arduino_hal::Peripherals::steal() };

        let result = dp.ADC.adc.read().bits();
        debug_assert!(adc.channel_cursor < N as u8);
        unsafe {
            *adc.values
                .get_unchecked_mut(adc.channel_cursor as usize)
                .get_unchecked_mut(adc.time_cursor as usize) = result;
        };

        (adc.channel_cursor, adc.time_cursor) = advance_cursor(&adc);
        // original cursor + 1 is already being read once the interrupt is called,
        // and setting ADMUX won't take effect until the current conversion is done,
        // so we have to look 2 ahead when setting the next channel
        let next_channel_index = (adc.channel_cursor + 1) % N as u8;
        let next_channel = unsafe {
            adc.channels
                .assume_init_ref()
                .get_unchecked(next_channel_index as usize)
                .channel()
        };
        dp.ADC
            .admux
            .modify(|r, w| unsafe { w.bits(r.bits()) }.mux().variant(next_channel));
    })
}
