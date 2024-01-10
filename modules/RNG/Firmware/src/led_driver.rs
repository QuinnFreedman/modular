use core::{
    arch::asm,
    cell::Cell,
    sync::atomic::{compiler_fence, Ordering},
};

use arduino_hal::{
    hal::port::{PB1, PB2, PD3},
    port::{mode::Output, Pin},
    prelude::*,
    spi::ChipSelectPin,
    Spi,
};
use avr_device::{asm::delay_cycles, interrupt::CriticalSection};
use embedded_hal::digital::v2::OutputPin;

/**
Determines how long each PWM period should be, in clocks.
f_PWM = f_osc/(2 * TLC_PWM_PERIOD) Hz
TLC_PWM_PERIOD = f_osc/(2 * f_PWM)

TLC_PWM_PERIOD = ((TLC_GSCLK_PERIOD + 1) * 4096)/2
The default of 8192 means the PWM frequency is 976.5625Hz
*/
const TLC_GSCLK_PERIOD: u8 = 3;
const TLC_PWM_PERIOD: u16 = 8192; // Doubled because it counts up and down

#[allow(dead_code)]
static WAITING_FOR_XLAT: avr_device::interrupt::Mutex<Cell<bool>> =
    avr_device::interrupt::Mutex::new(Cell::new(false));

/**
This provides a way to access a static mutex that would otherwise be disallowed.
It functions like avr_device::interrupt::free except that it doesn't actually
block interrupts while the function is running.

This should not be used unless you are sure that an interrupt will not happen
or being interrupted while accessing the mutex will not cause a race condition.
*/
#[inline(always)]
fn unsafe_access_mutex<F, R>(f: F) -> R
where
    F: FnOnce(CriticalSection) -> R,
{
    // unsafe { asm!("nop") };
    compiler_fence(Ordering::SeqCst);

    let r = f(unsafe { CriticalSection::new() });

    compiler_fence(Ordering::SeqCst);

    r
}

/**
This struct is responsible for driving the TLC5940. The pins used are not configurable
because they are all special pins that are internally connected to the ATmega's timers
PWM outputs. A lot of this implementation is very hardware-specific and so it will not
work on anything except an ATmega328P

The TLC5940 doesn't have any internal oscilator so it must recieve constant clock
triggers from the microcontroller. It requires 3 different clock signals in addition
to the SPI data and clock lines.

BLANK can be held HIGH to turn off the output. But, on the falling edge, it also
triggers a new PWM cycle. Durring normal operation, BLANK must be pulsed at the
desired PWM frequency. BLANK is connected to pin D10 (aka PB2) wich is internally
connected to OC1B (the 2nd PWM output channel of the ATmega's Timer 1).

GSLK is the high-resolution clock. Each pulse of BLANK, all outputs are enabled and
all counters are reset. Each pulse on GSLK, each counter is decremented and any
outputs are turned off if their counter hits zero. GSLK should pulse 4096 times
per BLANK pulse if you want the full pulse width resoulution. GSLK is connected to
D3 (PD3) which is the output of Timer 2.

The TLC5940 does not have a chip-select pin for the SPI protocol. Instead, it is
constantly reading SPI input into a shift register. You can send a pulse to the latch
(XLAT) pin to lock the current contents of the shift register into the device's
memory. You can write SPI data at any time, but you can only latch it between PWM
cycles while the BLANK input is held high. BLANK must be set high first, then XLAT
brought high, and then both brought low in the reverse order, with some time in
between. To achieve this, the XLAT pin is connected to D9 (PB1) which is the other
PWM output of Timer 1 which controlls the BLANK pin. The timer is configured in
"Phase and Frequency Correct PWM Mode", which means that the shorter XLAT duty cycle
will be centered in the BLANK pulse. The output on OC1A (PB1) is disabled at startup.
When data is written to the SPI bus, a flat is set and then the output trigger is
enabled. Additionally, the interrupt handler for Timer 1 is enabled. The next time
the timer loops, it will pulse the XLAT pin with the correct timing and then call
the interrupt. The interrupt clears the flag, disables the XLAT pulses, and also
disables the interrupt iteself.

This implementation is heavily based on the [SparkFun Arduino Library](https://github.com/sparkfun/SparkFun_TLC5940_Arduino_Library)
as well is [Paul Stoffregen's version](https://github.com/PaulStoffregen/Tlc5940)
which is very similar. However, it is made somewhat simpler by only supporting
a single architecture and ignoring the TCL5940's dot correction mode, error reporting
mode, and other features.
*/
pub struct TLC5940<const NUM_OUTPUTS: usize> {
    _xlatch: Pin<Output, PB1>,
    _blank: ChipSelectPin<PB2>,
    _pwm_ref: Pin<Output, PD3>,
    tc1: arduino_hal::pac::TC1,
    _tc2: arduino_hal::pac::TC2,
}

pub const fn get_u12_payload_size(num_u16s: usize) -> usize {
    usize::div_ceil(num_u16s * 3, 2)
}

impl<const NUM_OUTPUTS: usize> TLC5940<NUM_OUTPUTS>
where
    [(); get_u12_payload_size(NUM_OUTPUTS)]: Sized,
{
    // const fn payload_size() -> usize {
    //     usize::div_ceil(NUM_OUTPUTS * 3, 2)
    // }
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
        for _ in 0..get_u12_payload_size(NUM_OUTPUTS) {
            nb::block!(spi.send(0)).void_unwrap();
        }
        xlatch.set_high();
        xlatch.set_low();

        unsafe_access_mutex(|cs| WAITING_FOR_XLAT.borrow(cs).set(false));

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
        // Reset PWM on TOP instead of MAX (set by OCR2A) (allows for freq control)
        tc2.tccr2b.write(|w| w.wgm22().set_bit());
        // Lowest possible PWM (clear OC2B immediately)
        tc2.ocr2b.write(|w| w.bits(0));
        // Controls PWM frequency
        tc2.ocr2a.write(|w| w.bits(TLC_GSCLK_PERIOD));

        // Start PWM output 2 with no prescale
        tc2.tccr2b
            .modify(|r, w| unsafe { w.bits(r.bits()) }.cs2().direct());

        // Need to allow at least 10ns after BLANK goes LOW before GSCLK goes HIGH
        // and also 10ns after GSCLK goes HIGH before BLANK goes HIGH. This delay
        // staggers the two PWM outputs so this is true. If this is set up incorrectly,
        // it can cause slight (or intense) flickering in the LEDs, since there will be
        // a race condition that seems to cause whole PWM cycles to be skipped. A delay
        // does not seem to be needed in the C/C++ version. I'm not sure why. I'm
        // guessing Rust optimizes differently and so takes more cycles to modify the
        // registers. The ideal phase offset would be around 94ns. At 16Mhz, the arduino
        // gets ~62.5ns/cycle, so skipping 2 cycles puts us in spec. (Luckilly, the
        // TLC5940 seems to ignore all GLSCLK pulses that happen while BLANK is fully
        // HIGH, although the datasheet implies you shouldn't send them).
        unsafe { asm!("nop") };
        unsafe { asm!("nop") };

        // Start PWM output 1 with no prescale
        tc1.tccr1b
            .modify(|r, w| unsafe { w.bits(r.bits()) }.cs1().direct());

        blank.set_low().ok();

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
    After calling `write()`, on't use the SPI bus again for any device until
    `is_ready()` returns TRUE.
    */
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        // reads from global WAITING_FOR_XLAT mutex in an unsafe way, but
        // it's ok since WAITING_FOR_XLAT will only ever be set TRUE from
        // the main thread so this can return a "false positive" but not
        // a "false negative"
        !unsafe_access_mutex(|cs| WAITING_FOR_XLAT.borrow(cs).get())
    }

    #[allow(dead_code)]
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
        // There is no possible race condition here because and interrupt will only
        // ever set WAITING_FOR_XLAT to FALSE, never TRUE, so an interrupt happening
        // right before or after this check could only result in a false rejection,
        // never in a duplicate write. Disabling interrupts is not needed
        let waiting_for_xlat = unsafe_access_mutex(|cs| WAITING_FOR_XLAT.borrow(cs).get());
        if waiting_for_xlat {
            return Err(());
        }

        unsafe_access_mutex(|cs| WAITING_FOR_XLAT.borrow(cs).set(true));

        // The TLC5940 requires data to be transfered in tightly-packed 12-bit words
        // and to be transmitted msb-first (i.e. big endian)
        let mut u12_be_buffer = [0u8; get_u12_payload_size(NUM_OUTPUTS)];
        for i in 0..data.len() {
            let n = data[i];
            if i % 2 == 0 {
                let msb_idx = (i * 3) / 2;
                let high_byte = (n >> 4) as u8;
                let low_half_byte = (n << 4) as u8;
                u12_be_buffer[msb_idx] = high_byte;
                u12_be_buffer[msb_idx + 1] |= low_half_byte;
            } else {
                let msb_idx = ((i - 1) * 3) / 2 + 1;
                let high_half_byte = (n >> 8) as u8 & 0xf;
                let low_byte = n as u8;
                u12_be_buffer[msb_idx] |= high_half_byte;
                u12_be_buffer[msb_idx + 1] = low_byte;
            }
        }

        // Sometimes we need to write a non-integer number of bytes. Most other
        // implementations I have seen use bit-bang SPI and write the data bit-by-bit.
        // But, it seems faster and easier to use the hardware SPI and just write
        // 4 extra bits of garbage before the real transmission. We do this by shifting
        // everything in the buffer 4 bits to the right. There is probably a more clever
        // way to do this as part of the packing above instead of as a separate step.
        if NUM_OUTPUTS % 2 == 1 {
            for i in (1..u12_be_buffer.len()).rev() {
                u12_be_buffer[i] = u12_be_buffer[i] >> 4 | (u12_be_buffer[i - 1] & 0x0f) << 4;
            }
            u12_be_buffer[0] = u12_be_buffer[0] >> 4;
        }

        for byte in u12_be_buffer {
            nb::block!(spi.send(byte)).ok();
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
    unsafe_access_mutex(|cs| WAITING_FOR_XLAT.borrow(cs).set(false));
}
