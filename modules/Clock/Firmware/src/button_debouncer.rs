use arduino_hal::port::{mode, Pin, PinOps};

/**
When a mechanical button is pressed, it might quickly flicker on and off a few
times before it settles to fully on, either because electricity can jump the gap
as the contact gets close or because the button is litterally bouncing a tiny bit.
This wouldn't be noticable to a human but it can cause the microcontroller to
register extra unwanted button presses. To fix this we don't accept the button's
new value as soon as it changes, but instead wait until it has been constant at
the same value for a few ms.

This debouncing implementation is very principled -- it does exactly what we
theoretically would want. Using a trick like a shift register (effectively just
requiring getting the same value a few times in a row) would be slightly more
efficient, but the behavior would depend on the sampling frequency which would
depend on what else is happening in the loop.

Confirming the value with a single delayed resample would also be a little simpler
but could theoretically give incorrect results with a lot of noise.
 */
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

    /**
    Should be called continuously in the main loop. Checks if the button is
    currently pressed and if its state has changed. Returns the debounced
    state of the button.
    */
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

/**
Builds on top of `ButtonDebouncer` but additionally keeps track of how long the
button has been held down so it can return more button states
*/
pub struct ButtonWithLongPress<PIN, const DEBOUNCE_TIME_MS: u32, const LONG_PRESS_TIME_MS: u32>
where
    PIN: PinOps,
{
    base: ButtonDebouncer<PIN, DEBOUNCE_TIME_MS>,
    state: LPBInternalState,
}

#[derive(Eq, PartialEq)]
pub enum LongPressButtonState {
    /**
    Button has just been depressed
    */
    ButtonJustDown,
    /**
    Button has just been released before the long press timer ran out
    */
    ButtonJustClickedShort,
    /**
    Button is being held down and the long press timer just ran out
    */
    ButtonJustClickedLong,
    /**
    Button has just been released after the long press timer had run out
    */
    ButtonJustReleasedLong,
    /**
    Button is being held down. No change since last sample. Long press timer
    has not yet run out
    */
    ButtonHeldDownShort,
    /**
    Button is being held down. No change since last sample. Long press already
    ran out out
    */
    ButtonHeldDownLong,
    /**
    Button is not pressed; no change since last sample.
    */
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

    /**
    Should be called continuously in the main loop. Checks if the button is
    currently pressed and if its state has changed. Returns the debounced
    state of the button.
    */
    pub fn sample(&mut self, current_time_ms: u32) -> LongPressButtonState {
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
                LPBInternalState::NotWaiting => LongPressButtonState::ButtonJustReleasedLong,
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
