pub struct ButtonLadderState {
    candidate_value: Option<u8>,
    debounced_value: Option<u8>,
    last_change_time: u32,
    debouncing: bool,
}

const DEBOUNCE_TIME_MS: u32 = 64;

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

// expected voltages readings resulting from a voltage divider resistor ladder with
// a 10k resistor to 5V against up to 12 1k resistors to ground, measured by a
// 10-bit dac referenced to 5V
const EXPECTED_BUTTON_VALUES: [u16; 13] =
    [0, 93, 171, 236, 292, 341, 384, 421, 455, 485, 512, 536, 558];

fn get_closest_button_index(adc_value: u16) -> Option<u8> {
    let mut closest_idx: u8 = 12;
    let mut best_delta: u16 = adc_value.abs_diff(EXPECTED_BUTTON_VALUES[12]);

    for i in (0..=11u8).rev() {
        let delta = adc_value.abs_diff(EXPECTED_BUTTON_VALUES[i as usize]);
        if delta < best_delta {
            closest_idx = i;
            best_delta = delta;
        }
    }

    if closest_idx < 12 {
        Some(closest_idx)
    } else {
        None
    }
}

pub enum ButtonEvent {
    NoEvent,
    ButtonHeld(u8),
    ButtonJustReleased,
    ButtonJustPressed(u8),
}
