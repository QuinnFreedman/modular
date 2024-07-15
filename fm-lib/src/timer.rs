/**
Sets up a system real-time clock using TIMER0.
This entire module is wrapped in a macro. This is because I wanted the frequency to
be configurable but it needs to be available both in the SystemClock struct and in
the interrupt handler and I wasn't willing to pay the performance hit of storing it
in a thread-safe global variable and I didn't want the error prone-ness of having to
set it twice to the same value. I will probably try to refactor this sometime later.
*/
#[macro_export]
macro_rules! configure_system_clock {
    ($precision:expr) => {
        mod system_clock {
            use core::cell;
            use core::marker::ConstParamTy;

            use fm_lib::asynchronous::unsafe_access_mutex;
            use fm_lib::const_traits::ConstInto;

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

            #[allow(dead_code)]
            const CLOCK_PRECISION: ClockPrecision = $precision;
            #[allow(dead_code)]
            static MILLIS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u32>> =
                avr_device::interrupt::Mutex::new(cell::Cell::new(0));

            #[avr_device::interrupt(atmega328p)]
            fn TIMER0_COMPA() {
                fm_lib::asynchronous::assert_interrupts_disabled(|cs| {
                    let counter_cell = MILLIS_COUNTER.borrow(cs);
                    let counter = counter_cell.get();
                    counter_cell.set(counter + CLOCK_PRECISION.ms_increment());
                })
            }

            pub struct SystemClock<const PRECISION: ClockPrecision>(arduino_hal::pac::TC0);

            impl<const PRECISION: ClockPrecision> SystemClock<PRECISION> {
                #[allow(dead_code)]
                pub fn reset(&mut self) {
                    avr_device::interrupt::free(|cs| {
                        MILLIS_COUNTER.borrow(cs).set(0);
                        self.0.tcnt0.write(|w| w.bits(0));
                    });
                }
                fn read_timer_register(&self) -> u8 {
                    self.0.tcnt0.read().bits()
                }
                /**
                Running system clock in ms. Precision is between 1ms and 16ms depending on
                clock configuration.
                */
                #[allow(dead_code)]
                #[inline(always)]
                pub fn millis(&self) -> u32 {
                    avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
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
                        ms_counter = unsafe_access_mutex(|cs| MILLIS_COUNTER.borrow(cs).get());
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
            }

            #[allow(dead_code)]
            pub fn init_system_clock(
                tc0: arduino_hal::pac::TC0,
            ) -> SystemClock<{ CLOCK_PRECISION }> {
                // Configure the timer for the above interval (in CTC mode)
                // and enable its interrupt.
                tc0.tccr0a.write(|w| w.wgm0().ctc());
                tc0.ocr0a.write(|w| w.bits(CLOCK_PRECISION.rollover()));
                tc0.tccr0b.write(|w| match CLOCK_PRECISION.prescaler() {
                    Prescaler::PS8 => w.cs0().prescale_8(),
                    Prescaler::PS64 => w.cs0().prescale_64(),
                    Prescaler::PS256 => w.cs0().prescale_256(),
                    Prescaler::PS1024 => w.cs0().prescale_1024(),
                });
                tc0.timsk0.write(|w| w.ocie0a().set_bit());
                tc0.tcnt0.write(|w| w.bits(0));

                SystemClock::<{ CLOCK_PRECISION }>(tc0)
            }
        }
    };
}
