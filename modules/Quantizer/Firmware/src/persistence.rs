use arduino_hal::Eeprom;

const SENTINEL_VALUE: u8 = 0b10101010;

const STORAGE_OFFSET: u8 = 1;

use crate::{
    menu::Channel,
    quantizer::{ChannelConfig, QuantizerState, SampleMode},
};

fn parse_notes(bytes: &[u8]) -> [bool; 12] {
    debug_assert!(bytes.len() == 2);
    let mut notes = [false; 12];
    for i in 0..12 {
        let byte_index = i / 8;
        let bit_index = i % 8;
        notes[i] = ((bytes[byte_index] >> (bit_index)) & 1) != 0;
    }
    notes
}

fn encode_notes(notes: &[bool; 12]) -> [u8; 2] {
    let mut bytes = [0u8; 2];
    for i in 0..12 {
        let byte_index = i / 8;
        let bit_index = i % 8;
        let bit_value = if notes[i] { 1 } else { 0 };
        bytes[byte_index] |= bit_value << bit_index;
    }
    bytes
}

pub fn check_scale_save_slots(eeprom: &mut Eeprom) -> [bool; 12] {
    let mut full = [false; 12];
    for i in 0..12 {
        let address = STORAGE_OFFSET + i * 3;
        full[i as usize] = eeprom.read_byte(address as u16) == SENTINEL_VALUE;
    }
    full
}

pub fn write_scale(
    eeprom: &mut Eeprom,
    slot: u8,
    quantizer_state: &QuantizerState,
    channel: &Channel,
) {
    let notes = &quantizer_state.channels[channel.index()].config.notes;
    let address = STORAGE_OFFSET + slot * 3;
    let buff = encode_notes(notes);
    eeprom.write_byte(address as u16, SENTINEL_VALUE);
    eeprom.write((address + 1) as u16, &buff).unwrap();
}

pub fn read_scale(
    eeprom: &mut Eeprom,
    slot: u8,
    quantizer_state: &mut QuantizerState,
    channel: &Channel,
) {
    let address = STORAGE_OFFSET + slot * 3;
    let mut bytes = [0u8; 3];
    eeprom.read(address as u16, &mut bytes).unwrap();
    quantizer_state.channels[channel.index()].config.notes = parse_notes(&bytes[1..3]);
}

impl ChannelConfig {
    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        ChannelConfig {
            notes: parse_notes(&bytes[0..1]),
            sample_mode: if bytes[2] == 0 {
                SampleMode::TrackAndHold
            } else {
                SampleMode::SampleAndHold
            },
            glide_amount: bytes[3],
            trigger_delay_amount: bytes[4],
            pre_shift: unsafe { core::mem::transmute(bytes[5]) },
            scale_shift: unsafe { core::mem::transmute(bytes[6]) },
            post_shift: unsafe { core::mem::transmute(bytes[7]) },
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0..1].copy_from_slice(&encode_notes(&self.notes));
        bytes[2] = match self.sample_mode {
            SampleMode::TrackAndHold => 0,
            SampleMode::SampleAndHold => 1,
        };
        bytes[3] = self.glide_amount;
        bytes[4] = self.trigger_delay_amount;
        bytes[5] = unsafe { core::mem::transmute(self.pre_shift) };
        bytes[6] = unsafe { core::mem::transmute(self.scale_shift) };
        bytes[7] = unsafe { core::mem::transmute(self.post_shift) };
        bytes
    }
}
