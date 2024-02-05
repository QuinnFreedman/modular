use core::mem::MaybeUninit;

use arduino_hal::{
    adc::{AdcChannel, Channel},
    Adc,
};
use fm_lib::asynchronous::assert_interrupts_disabled;

pub struct AsyncAdcState<const N: usize> {
    channels: MaybeUninit<[Channel; N]>,
    values: [u16; N],
    cursor: u8,
}

pub type AsyncAdc<const N: usize> = Option<AsyncAdcState<N>>;

pub trait Indexable {
    fn get(&self, i: u8) -> u16;
}

impl<const N: usize> Indexable for AsyncAdc<N> {
    fn get(&self, i: u8) -> u16 {
        debug_assert!(i < N as u8);
        debug_assert!(self.is_some());
        let adc = unsafe { self.as_ref().unwrap_unchecked() };
        adc.values[i as usize]
    }
}

pub fn init_async_adc<const N: usize>(
    mut adc: Adc,
    async_adc_state: &'static mut AsyncAdc<N>,
    channels: [Channel; N],
) {
    let ch1 = channels[0].channel();
    let ch2 = channels[1].channel();

    let mut values = unsafe { MaybeUninit::<[u16; N]>::uninit().assume_init() };
    for i in 0..N {
        values[i] = adc.read_blocking(&channels[i]);
    }
    debug_assert!(async_adc_state.is_none());
    *async_adc_state = Some(AsyncAdcState {
        channels: MaybeUninit::new(channels),
        values,
        cursor: 0,
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
pub fn handle_conversion_result<const N: usize>(adc: &'static mut AsyncAdc<N>) {
    assert_interrupts_disabled(|_cs| {
        debug_assert!(adc.is_some());
        let adc = unsafe { adc.as_mut().unwrap_unchecked() };
        let dp = unsafe { arduino_hal::Peripherals::steal() };

        let result = dp.ADC.adc.read().bits();
        adc.values[adc.cursor as usize] = result;

        adc.cursor = (adc.cursor + 1) % N as u8;
        let next_channel = unsafe { adc.channels.assume_init_ref() }
            [((adc.cursor + 1) % N as u8) as usize]
            .channel();
        dp.ADC
            .admux
            .modify(|r, w| unsafe { w.bits(r.bits()) }.mux().variant(next_channel));
    })
}
