use arduino_hal::port::{mode, Pin, PinOps};

pub struct ButtonDebouncer<PIN, const DEBOUNCE_TIME_MS: u32>
where
    PIN: PinOps,
{
    pin: Pin<mode::Input<mode::PullUp>, PIN>,
    last_pin_change_time: u32,
    candidate_button_value: bool,
    debounced_button_value: bool,
    debouncing: bool,
}

impl<PIN, const DEBOUNCE_TIME_MS: u32> ButtonDebouncer<PIN, DEBOUNCE_TIME_MS>
where
    PIN: PinOps,
{
    pub fn new(pin: Pin<mode::Input<mode::PullUp>, PIN>) -> Self {
        ButtonDebouncer {
            pin,
            last_pin_change_time: 0,
            candidate_button_value: false,
            debounced_button_value: false,
            debouncing: false,
        }
    }

    pub fn sample(&mut self, current_time_ms: u32) -> ButtonState {
        let button_value = self.pin.is_low();
        if button_value != self.candidate_button_value {
            self.candidate_button_value = button_value;
            self.last_pin_change_time = current_time_ms;
            self.debouncing = true;
        } else if self.debouncing && current_time_ms - self.last_pin_change_time > DEBOUNCE_TIME_MS
        {
            self.debouncing = false;
            self.debounced_button_value = self.candidate_button_value;
            return match self.debounced_button_value {
                true => ButtonState::ButtonJustPressed,
                false => ButtonState::ButtonJustReleased,
            };
        }

        match self.debounced_button_value {
            false => ButtonState::ButtonIsUp,
            true => ButtonState::ButtonHeldDown,
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum ButtonState {
    ButtonJustPressed,
    ButtonJustReleased,
    ButtonHeldDown,
    ButtonIsUp,
}

pub struct ButtonWithLongPress<PIN, const DEBOUNCE_TIME_MS: u32, const LONG_PRESS_TIME_MS: u32>
where
    PIN: PinOps,
{
    base: ButtonDebouncer<PIN, DEBOUNCE_TIME_MS>,
    state: LPBInternalState,
}

#[derive(Eq, PartialEq)]
pub enum LongPressButtonState {
    ButtonJustDown,
    ButtonJustClickedShort,
    ButtonJustClickedLong,
    ButtonHeldDownShort,
    ButtonHeldDownLong,
    ButtonIsUp,
}

enum LPBInternalState {
    WaitingForLongPressSince(u32),
    NotWaiting,
}

impl<PIN, const DEBOUNCE_TIME_MS: u32, const LONG_PRESS_TIME_MS: u32>
    ButtonWithLongPress<PIN, DEBOUNCE_TIME_MS, LONG_PRESS_TIME_MS>
where
    PIN: PinOps,
{
    pub fn new(pin: Pin<mode::Input<mode::PullUp>, PIN>) -> Self {
        Self {
            base: ButtonDebouncer::<PIN, DEBOUNCE_TIME_MS>::new(pin),
            state: LPBInternalState::NotWaiting,
        }
    }

    pub fn sample(&mut self, current_time_ms: u32) -> LongPressButtonState {
        // let dp = unsafe { arduino_hal::Peripherals::steal() };
        // let pins = arduino_hal::pins!(dp);
        // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
        // serial.write_byte('s' as u8);
        // serial.write_byte('\n' as u8);
        let button_state = self.base.sample(current_time_ms);
        match button_state {
            ButtonState::ButtonJustPressed => {
                self.state = LPBInternalState::WaitingForLongPressSince(current_time_ms);
                LongPressButtonState::ButtonJustDown
            }
            ButtonState::ButtonJustReleased => match self.state {
                LPBInternalState::WaitingForLongPressSince(_) => {
                    LongPressButtonState::ButtonJustClickedShort
                }
                LPBInternalState::NotWaiting => LongPressButtonState::ButtonIsUp,
            },
            ButtonState::ButtonHeldDown => match self.state {
                LPBInternalState::WaitingForLongPressSince(start_time) => {
                    let held_time = current_time_ms.saturating_sub(start_time);
                    if held_time > LONG_PRESS_TIME_MS {
                        self.state = LPBInternalState::NotWaiting;
                        LongPressButtonState::ButtonJustClickedLong
                    } else {
                        LongPressButtonState::ButtonHeldDownShort
                    }
                }
                LPBInternalState::NotWaiting => LongPressButtonState::ButtonHeldDownLong,
            },
            ButtonState::ButtonIsUp => LongPressButtonState::ButtonIsUp,
        }
    }
}
