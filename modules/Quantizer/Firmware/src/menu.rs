use core::panic;

use crate::{quantizer::QuantizerState, resistor_ladder_buttons::ButtonEvent};

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

impl Into<usize> for &Channel {
    fn into(self) -> usize {
        match self {
            Channel::A => 0,
            Channel::B => 1,
        }
    }
}

impl Channel {
    pub fn index(&self) -> usize {
        Into::<usize>::into(self)
    }
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

    fn handle_shift_button_press(
        &mut self,
        quantizer_state: &mut QuantizerState,
        button_index: u8,
    ) {
        match button_index {
            0 => {}
            1 => {}
            2 => {}
            3 => {}
            4 => {}
            5 => {}
            6 => {}
            7 => {}
            8 => {}
            9 => {}
            10 => {
                self.selected_channel = Channel::A;
            }
            11 => {
                self.selected_channel = Channel::B;
            }
            _ => panic!(),
        }
    }

    pub fn handle_button_input_and_render_display(
        &mut self,
        quantizer_state: &mut QuantizerState,
        button_event: &ButtonEvent,
        shift_pressed: bool,
    ) -> [LedColor; 12] {
        match button_event {
            ButtonEvent::ButtonJustPressed(n) => {
                if shift_pressed {
                    self.handle_shift_button_press(quantizer_state, *n);
                } else {
                    let selected_channel =
                        &mut quantizer_state.channels[self.selected_channel.index()];
                    selected_channel.notes[*n as usize] = !selected_channel.notes[*n as usize];
                }
            }
            _ => {}
        }

        self.render_notes_display(quantizer_state)
    }

    fn render_notes_display(&self, quantizer_state: &QuantizerState) -> [LedColor; 12] {
        let selected_channel = &quantizer_state.channels[self.selected_channel.index()];
        let color = match self.selected_channel {
            Channel::A => LedColor::GREEN,
            Channel::B => LedColor::RED,
        };
        let mut leds = [LedColor::OFF; 12];
        for i in 0..12 {
            if selected_channel.notes[i] {
                leds[i] = color;
            }
        }
        leds
    }
}
