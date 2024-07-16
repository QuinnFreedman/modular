use core::cell;
use core::marker::ConstParamTy;
use core::ops::Deref;

use crate::asynchronous::{assert_interrupts_disabled, unsafe_access_mutex};
use crate::const_traits::ConstInto;

#[allow(dead_code)]
#[derive(PartialEq, Eq, ConstParamTy)]
pub enum ClockPrecision {
    MS1,
    MS2,
    MS4,
    MS8,
    MS16,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq)]
enum Prescaler {
    PS8,
    PS64,
    PS256,
    PS1024,
}

impl const ConstInto<u32> for Prescaler {
    fn const_into(self) -> u32 {
        match self {
            Prescaler::PS8 => 8,
            Prescaler::PS64 => 64,
            Prescaler::PS256 => 256,
            Prescaler::PS1024 => 1024,
        }
    }
}

impl ClockPrecision {
    const fn prescaler(&self) -> Prescaler {
        match self {
            ClockPrecision::MS1 => Prescaler::PS64,
            ClockPrecision::MS2 => Prescaler::PS256,
            ClockPrecision::MS4 => Prescaler::PS256,
            ClockPrecision::MS8 => Prescaler::PS1024,
            ClockPrecision::MS16 => Prescaler::PS1024,
        }
    }

    const fn rollover(&self) -> u8 {
        const DOUBLE: u8 = 125;
        const SINGLE: u8 = 250;
        match self {
            ClockPrecision::MS1 => SINGLE,
            ClockPrecision::MS2 => DOUBLE,
            ClockPrecision::MS4 => SINGLE,
            ClockPrecision::MS8 => DOUBLE,
            ClockPrecision::MS16 => SINGLE,
        }
    }

    const fn ctr_units_to_us(&self, counter_value: u8) -> u32 {
        (ConstInto::<u32>::const_into(self.prescaler()) * counter_value as u32) / 16
    }

    const fn ctr_units_to_ms(&self, counter_value: u8) -> u32 {
        (ConstInto::<u32>::const_into(self.prescaler()) * counter_value as u32) / 16000
    }

    const fn ms_increment(&self) -> u32 {
        self.ctr_units_to_ms(self.rollover())
    }
}

pub struct GlobalSystemClockState<const PRECISION: ClockPrecision>(
    avr_device::interrupt::Mutex<cell::Cell<u32>>,
);

impl<const PRECISION: ClockPrecision> Deref for GlobalSystemClockState<{ PRECISION }> {
    type Target = avr_device::interrupt::Mutex<cell::Cell<u32>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const PRECISION: ClockPrecision> GlobalSystemClockState<{ PRECISION }> {
    pub const fn new() -> Self {
        GlobalSystemClockState(avr_device::interrupt::Mutex::new(cell::Cell::new(0)))
    }
}

#[macro_export]
macro_rules! handle_system_clock {
    ($precision:expr, $global_state:expr) => {
        #[avr_device::interrupt(atmega328p)]
        fn TIMER0_COMPA() {
            fm_lib::system_clock::increment_global_counter::<{ $precision }>($global_state);
        }
    };
}

/**
Intended to be called every time there is a timer interrupt for the system clock timer.
*/
pub fn increment_global_counter<const PRECISION: ClockPrecision>(
    global_state: &'static GlobalSystemClockState<{ PRECISION }>,
) {
    assert_interrupts_disabled(|cs| {
        let global_state: &'static GlobalSystemClockState<PRECISION> = global_state;
        let counter_cell = global_state.borrow(cs);
        let counter = counter_cell.get();
        counter_cell.set(counter + PRECISION.ms_increment());
    });
}

// static DEBUG_MILLIS_COUNTER: GlobalSystemClockState = GlobalSystemClockState::new();
// handle_system_clock!(ClockPrecision::MS16, &DEBUG_MILLIS_COUNTER);

pub struct SystemClock<TIMER: AtmegaTimerSubset, const PRECISION: ClockPrecision> {
    timer: TIMER,
    global_millis_ctr: &'static GlobalSystemClockState<{ PRECISION }>,
}

impl<TIMER: AtmegaTimerSubset, const PRECISION: ClockPrecision> SystemClock<TIMER, PRECISION> {
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        avr_device::interrupt::free(|cs| {
            self.global_millis_ctr.borrow(cs).set(0);
            self.timer.reset_tcnt();
        });
    }
    fn read_timer_register(&self) -> u8 {
        self.timer.get_tcnt()
    }
    /**
    Running system clock in ms. Precision is between 1ms and 16ms depending on
    clock configuration.
    */
    #[allow(dead_code)]
    #[inline(always)]
    pub fn millis(&self) -> u32 {
        avr_device::interrupt::free(|cs| self.global_millis_ctr.borrow(cs).get())
    }

    /**
    Reads the timer register and the running ms count simultaneously.
    This has to be done in a synchronized way. Otherwise, the timer register
    could roll over (causing the ms counter to increment) between the time the
    two values are read, causing the sum to jump ahead and then jump back
    on subsequent reads. This function is intended to preserve monotonicity.

    The two returned values can be summed after adjusting for scale factor to
    get the exact current system time.
    */
    fn safe_read_counter_value(&self) -> (u32, u8) {
        let mut timer_register = self.read_timer_register();
        let mut ms_counter: u32;
        loop {
            ms_counter = unsafe_access_mutex(|cs| self.global_millis_ctr.borrow(cs).get());
            let timer_register_check = self.read_timer_register();
            let ok = timer_register_check > timer_register;
            timer_register = timer_register_check;
            if ok {
                break;
            }
        }

        (ms_counter, timer_register)
    }

    /**
    Running system clock in ms. Augments the running counter with live data from
    the hardware timer register for more precision at the cost of some more
    computation. Precision is still limited by the prescale factor, which is a
    property of the clock precision configuration.
    */
    #[allow(dead_code)]
    pub fn millis_exact(&self) -> u32 {
        let (ms_counter, timer_register) = self.safe_read_counter_value();
        ms_counter + PRECISION.ctr_units_to_ms(timer_register)
    }

    #[allow(dead_code)]
    pub fn micros(&self) -> u64 {
        let (ms_counter, timer_register) = self.safe_read_counter_value();
        (ms_counter as u64 * 1000) + PRECISION.ctr_units_to_us(timer_register) as u64
    }

    pub fn init_system_clock(
        tc: TIMER,
        global_state: &'static GlobalSystemClockState<{ PRECISION }>,
    ) -> Self {
        tc.init_timer_for_system_clock(&PRECISION);

        SystemClock::<TIMER, { PRECISION }> {
            timer: tc,
            global_millis_ctr: global_state,
        }
    }
}

pub trait AtmegaTimerSubset {
    fn init_timer_for_system_clock(&self, precision: &ClockPrecision);
    fn reset_tcnt(&self);
    fn get_tcnt(&self) -> u8;
}

impl AtmegaTimerSubset for arduino_hal::pac::TC0 {
    fn init_timer_for_system_clock(&self, precision: &ClockPrecision) {
        // Configure the timer in CTC mode with reset at OCRnA
        self.tccr0a.write(|w| w.wgm0().ctc());
        // Set the timer to the given precision and frequency
        self.ocr0a.write(|w| w.bits(precision.rollover()));
        self.tccr0b.write(|w| match precision.prescaler() {
            Prescaler::PS8 => w.cs0().prescale_8(),
            Prescaler::PS64 => w.cs0().prescale_64(),
            Prescaler::PS256 => w.cs0().prescale_256(),
            Prescaler::PS1024 => w.cs0().prescale_1024(),
        });
        // Enable interrupts on OCRnA match
        self.timsk0.write(|w| w.ocie0a().set_bit());

        self.reset_tcnt();
    }

    fn reset_tcnt(&self) {
        self.tcnt0.write(|w| w.bits(0));
    }

    fn get_tcnt(&self) -> u8 {
        self.tcnt0.read().bits()
    }
}

impl AtmegaTimerSubset for arduino_hal::pac::TC2 {
    fn init_timer_for_system_clock(&self, precision: &ClockPrecision) {
        // Configure the timer in CTC mode with reset at OCRnA
        self.tccr2a.write(|w| w.wgm2().ctc());
        // Set the timer to the given precision and frequency
        self.ocr2a.write(|w| w.bits(precision.rollover()));
        self.tccr2b.write(|w| match precision.prescaler() {
            Prescaler::PS8 => w.cs2().prescale_8(),
            Prescaler::PS64 => w.cs2().prescale_64(),
            Prescaler::PS256 => w.cs2().prescale_256(),
            Prescaler::PS1024 => w.cs2().prescale_1024(),
        });
        // Enable interrupts on OCRnA match
        self.timsk2.write(|w| w.ocie2a().set_bit());

        self.reset_tcnt();
    }

    fn reset_tcnt(&self) {
        self.tcnt2.write(|w| w.bits(0));
    }

    fn get_tcnt(&self) -> u8 {
        self.tcnt2.read().bits()
    }
}
