use crate::quantizer::QuantizerState;

#[derive(Clone, Copy)]
pub enum LedColor {
    GREEN,
    RED,
    AMBER,
    OFF,
}

pub enum Channel {
    A,
    B,
}

pub struct MenuState {
    selected_channel: Channel,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            selected_channel: Channel::A,
        }
    }

    pub fn handle_input_and_render(
        &mut self,
        quantizer_state: &QuantizerState,
        shift_pressed: bool,
    ) -> [LedColor; 12] {
        if shift_pressed {
            [LedColor::AMBER; 12]
        } else {
            let mut leds = [LedColor::OFF; 12];
            for i in 0..12 {
                if quantizer_state.notes[i] {
                    leds[i] = LedColor::GREEN;
                }
            }
            leds
        }
    }
}
