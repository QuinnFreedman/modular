use core::cell::Cell;

use arduino_hal::{
    hal::port::{PB1, PB2, PD3},
    port::{mode::Output, Pin},
    prelude::*,
    spi::ChipSelectPin,
    Spi,
};
use avr_device::interrupt::CriticalSection;
use embedded_hal::digital::v2::OutputPin;

/**
Determines how long each PWM period should be, in clocks.
f_PWM = f_osc/(2 * TLC_PWM_PERIOD) Hz
TLC_PWM_PERIOD = f_osc/(2 * f_PWM)

TLC_PWM_PERIOD = ((TLC_GSCLK_PERIOD + 1) * 4096)/2
The default of 8192 means the PWM frequency is 976.5625Hz
*/
const TLC_GSCLK_PERIOD: u8 = 3;
const TLC_PWM_PERIOD: u16 = 4096 * 2; // Period of 4096 steps counting up and down

#[allow(dead_code)]
static WAITING_FOR_XLAT: avr_device::interrupt::Mutex<Cell<bool>> =
    avr_device::interrupt::Mutex::new(Cell::new(false));

pub struct TLC5940<const NUM_OUTPUTS: usize> {
    _xlatch: Pin<Output, PB1>,
    _blank: ChipSelectPin<PB2>,
    _pwm_ref: Pin<Output, PD3>,
    tc1: arduino_hal::pac::TC1,
    _tc2: arduino_hal::pac::TC2,
}

impl<const NUM_OUTPUTS: usize> TLC5940<NUM_OUTPUTS> {
    /**
    Initializes the TLC5940 device; Takes ownership of the pins and timers
    used to drive the device since they shouldn't be used for other purposes
    while controlling the device. Only one instance of the driver should
    exist at a time.
    */
    pub fn new(
        spi: &mut Spi,
        pwm_ref: Pin<Output, PD3>,
        mut blank: ChipSelectPin<PB2>,
        mut xlatch: Pin<Output, PB1>,
        tc1: arduino_hal::pac::TC1,
        tc2: arduino_hal::pac::TC2,
    ) -> Self {
        blank.set_high().ok();
        xlatch.set_low();

        // The TPS5940 GS operating mode and shift register values are not defined
        // after power up. One solution is to switch to DC mode first and then back
        // to PWM mode, but that requires an extra IO pin. The other solution is to
        // overflow the input shift register with 193 bits of dummy data and latch
        // it while the TLS540 is in GS PWM mode.
        for _ in 0..193 {
            nb::block!(spi.send(0)).void_unwrap();
        }
        xlatch.set_high();
        xlatch.set_low();

        // write initial values
        for _ in 0..NUM_OUTPUTS * 2 {
            nb::block!(spi.send(0xff)).void_unwrap();
        }
        xlatch.set_high();
        xlatch.set_low();

        avr_device::interrupt::free(|cs| WAITING_FOR_XLAT.borrow(cs).set(false));

        // Timer 1: BLANK and XLAT
        // Clear OC1B on compare match when up-counting. Set OC1B on compare match when down counting
        tc1.tccr1a.write(|w| w.com1b().match_clear());
        // phase and frequency correct PWM with ICR1 TOP
        tc1.tccr1b.write(|w| w.wgm1().bits(0b10));
        // pulse width for XLAT
        tc1.ocr1a.write(|w| w.bits(1));
        // pulse width for BLANK (> XLAT)
        tc1.ocr1b.write(|w| w.bits(2));
        // set frequency for blank and xlat
        tc1.icr1.write(|w| w.bits(TLC_PWM_PERIOD));

        // Timer 2: GSCLK
        tc2.tccr2a.write(|w| {
            w
                // Clear OC2B on compare match
                .com2b()
                .match_clear()
                // Fast PWM mode with OCRA TOP
                .wgm2()
                .pwm_fast()
        });
        // Lowest possible PWM (clear OC2B immediately)
        tc2.ocr2b.write(|w| w.bits(0));
        // Controls PWM frequency
        tc2.ocr2a.write(|w| w.bits(TLC_GSCLK_PERIOD));
        tc2.tccr2b.write(|w| {
            w
                // Reset PWM on TOP instead of MAX (set by OCR2A) (allows for freq control)
                .wgm22()
                .set_bit()
                // No prescale (start PWM output 2)
                .cs2()
                .direct()
        });

        // no prescale, (start PWM output 1)
        tc1.tccr1b
            .modify(|r, w| w.wgm1().bits(r.wgm1().bits()).cs1().direct());

        TLC5940 {
            _xlatch: xlatch,
            _blank: blank,
            _pwm_ref: pwm_ref,
            tc1,
            _tc2: tc2,
        }
    }

    /**
    Returns FALSE if data has been written to the device that has not yet
    been latched. Latches can only happen at the end of a PWM cycle.
    Don't use the SPI buss for any device until is_ready() returns TRUE.
    */
    pub fn is_ready(&self) -> bool {
        // reads from global WAITING_FOR_XLAT mutex in an unsafe way, but
        // it's ok since it's just a read and WAITING_FOR_XLAT will only
        // be set TRUE from the main thread
        let waiting = WAITING_FOR_XLAT
            .borrow(unsafe { CriticalSection::new() })
            .get();
        true
        // !avr_device::interrupt::free(|cs| WAITING_FOR_XLAT.borrow(cs).get())
    }

    pub fn sync_write(&self, spi: &mut Spi, data: &[u16; NUM_OUTPUTS]) {
        while !self.is_ready() {}
        self.write(spi, data).ok();
        while !self.is_ready() {}
    }

    /**
    Writes PWM values for all channels to the device. The values will be latched
    into the TLC5940's memory at the start of the next PWM cycle.

    If the driver already has a write queued for this PWM cycle, this function
    will return Err.

    If you write to the SPI bus immediately after calling this function before
    the values have been latched, that could cause the values to be overwritten.
    If that might be the case, wait for `is_ready()` before using SPI or just
    use `sync_write`.
    */
    pub fn write(&self, spi: &mut Spi, data: &[u16; NUM_OUTPUTS]) -> Result<(), ()> {
        // The whole function doesn't need to be wrapped in interrupt::free
        // because an interrupt will only ever be clear WAITING_FOR_XLAT, not
        // set it, so a race condition couldn't result in a duplicate write
        let waiting_for_xlat = avr_device::interrupt::free(|cs| WAITING_FOR_XLAT.borrow(cs).get());
        if waiting_for_xlat {
            return Err(());
        }

        avr_device::interrupt::free(|cs| WAITING_FOR_XLAT.borrow(cs).set(true));

        for word in data {
            let hbyte: u8 = (word >> 8) as u8 & 0x0fu8;
            let lbyte: u8 = (word & 0xff) as u8;
            nb::block!(spi.send(hbyte)).ok();
            nb::block!(spi.send(lbyte)).ok();
        }

        enable_xlat_trigger(&self.tc1);
        Ok(())
    }
}

fn enable_xlat_trigger(tc1: &arduino_hal::pac::TC1) {
    // Enable pulses on XLAT pin along with BLANK pin (timer 1)
    tc1.tccr1a
        .write(|w| w.com1b().match_clear().com1a().match_clear());

    // Enable TIMER1_OVF interupt to run on next XLAT pulse to turn off the trigger
    tc1.tifr1.write(|w| w.tov1().set_bit());
    tc1.timsk1.write(|w| w.toie1().set_bit());
}

fn disable_xlat_trigger(tc1: &arduino_hal::pac::TC1) {
    // Disable pulses on XLAT pin
    tc1.tccr1a.write(|w| w.com1b().match_clear());
    // Disable timer1 interupt
    tc1.timsk1.write(|w| w.toie1().clear_bit());
}

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn TIMER1_OVF() {
    // Is called after an XLAT trigger has happened. Turns off XLAT
    // triggers so it doesn't repeat
    let tc1 = unsafe { arduino_hal::Peripherals::steal().TC1 };
    disable_xlat_trigger(&tc1);
    avr_device::interrupt::free(|cs| WAITING_FOR_XLAT.borrow(cs).set(false));
}
