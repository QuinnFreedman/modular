pub struct ButtonLadderState {
    candidate_value: Option<u8>,
    debounced_value: Option<u8>,
    last_change_time: u32,
    debouncing: bool,
}

const DEBOUNCE_TIME_MS: u32 = 20;

impl ButtonLadderState {
    pub fn new() -> Self {
        Self {
            candidate_value: None,
            debounced_value: None,
            last_change_time: 0,
            debouncing: false,
        }
    }

    pub fn sample_adc_value(&mut self, current_time: u32, adc_value: u16) -> ButtonEvent {
        let new_value = get_closest_button_index(adc_value);
        if new_value != self.candidate_value {
            self.candidate_value = new_value;
            self.last_change_time = current_time;
            self.debouncing = true;
        } else if self.debouncing && current_time - self.last_change_time > DEBOUNCE_TIME_MS {
            self.debouncing = false;
            self.debounced_value = self.candidate_value;
            return match self.debounced_value {
                Some(n) => ButtonEvent::ButtonJustPressed(n),
                None => ButtonEvent::ButtonJustReleased,
            };
        }

        match self.debounced_value {
            Some(n) => ButtonEvent::ButtonHeld(n),
            None => ButtonEvent::NoEvent,
        }
    }
}

fn get_closest_button_index(adc_value: u16) -> Option<u8> {
    // (n / (R1 + n)) * 1024 = adc_value;
    const R1_VALUE: u16 = 10;
    let divisor = 1024 - adc_value;
    let idx = (R1_VALUE * adc_value + (divisor - 1)) / divisor;
    if idx >= 12 {
        None
    } else {
        Some(idx as u8)
    }
}

pub enum ButtonEvent {
    NoEvent,
    ButtonHeld(u8),
    ButtonJustReleased,
    ButtonJustPressed(u8),
}
