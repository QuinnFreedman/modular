use core::{cell::UnsafeCell, mem::MaybeUninit};

use arduino_hal::{
    adc::{AdcChannel, Channel},
    Adc,
};
use avr_device::interrupt::{CriticalSection, Mutex};
use fm_lib::asynchronous::{assert_interrupts_disabled, unsafe_access_mutex};

pub struct AsyncAdcState<const N: usize> {
    channels: MaybeUninit<[Channel; N]>,
    values: [u16; N],
    cursor: u8,
}

pub type AsyncAdc<const N: usize> = Mutex<UnsafeCell<Option<AsyncAdcState<N>>>>;

pub const fn new_async_adc_state<const N: usize>() -> AsyncAdc<N> {
    Mutex::new(UnsafeCell::new(None))
}

pub trait Indexable {
    fn get<T>(&self, i: T) -> u16
    where
        T: Into<usize>;
}

impl<const N: usize> Indexable for Option<AsyncAdcState<N>> {
    #[inline(always)]
    fn get<T>(&self, index: T) -> u16
    where
        T: Into<usize>,
    {
        let i: usize = index.into();
        debug_assert!(i < N);
        debug_assert!(self.is_some());
        let adc = unsafe { self.as_ref().unwrap_unchecked() };
        unsafe { *adc.values.get_unchecked(i as usize) }
    }
}

pub trait Borrowable {
    type Inner;
    fn get<'cs>(&self, cs: CriticalSection<'cs>) -> &'cs Self::Inner;
}

impl<T> Borrowable for Mutex<UnsafeCell<T>> {
    type Inner = T;
    fn get<'cs>(&self, cs: CriticalSection<'cs>) -> &'cs Self::Inner {
        let ptr = self.borrow(cs).get();
        let option_ref = unsafe { ptr.as_ref().unwrap_unchecked() };
        option_ref
    }
}

pub fn init_async_adc<const N: usize>(
    mut adc: Adc,
    async_adc_state: &AsyncAdc<N>,
    channels: [Channel; N],
) {
    let ch1 = channels[0].channel();
    let ch2 = channels[1].channel();

    let mut values = unsafe { MaybeUninit::<[u16; N]>::uninit().assume_init() };
    for i in 0..N {
        values[i] = adc.read_blocking(&channels[i]);
    }

    unsafe_access_mutex(|cs| {
        let cell = async_adc_state.borrow(cs);
        let inner = unsafe { &mut *cell.get() };
        debug_assert!(inner.is_none());
        *inner = Some(AsyncAdcState {
            channels: MaybeUninit::new(channels),
            values,
            cursor: 0,
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

#[inline(always)]
pub fn handle_conversion_result<const N: usize>(adc: &AsyncAdc<N>) {
    assert_interrupts_disabled(|cs| {
        let cell = adc.borrow(cs);
        let inner = unsafe { &mut *cell.get() }.as_mut();
        debug_assert!(inner.is_some());
        let adc = unsafe { inner.unwrap_unchecked() };
        let dp = unsafe { arduino_hal::Peripherals::steal() };

        let result = dp.ADC.adc.read().bits();
        debug_assert!(adc.cursor < N as u8);
        unsafe {
            *adc.values.get_unchecked_mut(adc.cursor as usize) = result;
        };

        adc.cursor = (adc.cursor + 1) % N as u8;
        // original cursor + 1 is already being read once the interrupt is called,
        // and setting ADMUX won't take effect until the current conversion is done,
        // so we have to look 2 ahead when setting the next channel
        let next_channel_index = (adc.cursor + 1) % N as u8;
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
