use arduino_hal::Eeprom;

const SENTINEL_VALUE: u8 = 0b10101010;

const STORAGE_OFFSET: u8 = 1;

use crate::{
    bitvec::BitVec,
    menu::Channel,
    quantizer::{ChannelConfig, PitchMode, QuantizerChannel, QuantizerState, SampleMode},
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

pub fn check_save_slots(eeprom: &mut Eeprom) -> (BitVec<12>, BitVec<12>) {
    let mut scales = BitVec::<12>::new();
    for i in 0..12 {
        let address = STORAGE_OFFSET + i * 3;
        scales.set(i, eeprom.read_byte(address as u16) == SENTINEL_VALUE);
    }
    let offset = STORAGE_OFFSET + 12 * 3;
    let mut configs = BitVec::<12>::new();
    for i in 0..12 {
        let address = offset + i * 18;
        configs.set(i, eeprom.read_byte(address as u16) == SENTINEL_VALUE);
    }
    (scales, configs)
}

pub fn erase_all_save_slots(eeprom: &mut Eeprom) {
    for i in 0..12 {
        let address = STORAGE_OFFSET + i * 3;
        eeprom.erase_byte(address as u16);
    }
    let offset = STORAGE_OFFSET + 12 * 3;
    for i in 0..12 {
        let address = offset + i * 18;
        eeprom.erase_byte(address as u16);
    }
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
    if bytes[0] != SENTINEL_VALUE {
        return;
    }
    quantizer_state.channels[channel.index()].config.notes = parse_notes(&bytes[1..3]);
}

pub fn write_config(eeprom: &mut Eeprom, slot: u8, quantizer_state: &QuantizerState) {
    let offset = STORAGE_OFFSET + 12 * 3;
    let address = offset + slot * 18;
    let buff = quantizer_state.to_bytes();
    eeprom.write_byte(address as u16, SENTINEL_VALUE);
    eeprom.write((address + 1) as u16, &buff).unwrap();
}

pub fn read_config(eeprom: &mut Eeprom, slot: u8, quantizer_state: &mut QuantizerState) {
    let offset = STORAGE_OFFSET + 12 * 3;
    let address = offset + slot * 18;
    let sentinel = eeprom.read_byte(address as u16);
    if sentinel != SENTINEL_VALUE {
        return;
    }
    let mut buff = [0u8; 17];
    eeprom.read((address + 1) as u16, &mut buff).unwrap();
    *quantizer_state = QuantizerState::from_bytes(&buff);
}

impl ChannelConfig {
    fn from_bytes(bytes: &[u8; 8]) -> Self {
        ChannelConfig {
            notes: parse_notes(&bytes[0..2]),
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

    fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0..2].copy_from_slice(&encode_notes(&self.notes));
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

impl QuantizerState {
    fn from_bytes(bytes: &[u8; 17]) -> Self {
        Self {
            channels_linked: bytes[0] & 1 != 0,
            channel_b_mode: if bytes[0] & 2 == 0 {
                PitchMode::Relative
            } else {
                PitchMode::Absolute
            },
            channels: [
                QuantizerChannel::from_config(ChannelConfig::from_bytes(
                    bytes[1..9].try_into().unwrap(),
                )),
                QuantizerChannel::from_config(ChannelConfig::from_bytes(
                    bytes[9..17].try_into().unwrap(),
                )),
            ],
        }
    }

    fn to_bytes(&self) -> [u8; 17] {
        let mut flags = 0u8;
        if self.channels_linked {
            flags |= 1;
        }
        if self.channel_b_mode == PitchMode::Absolute {
            flags |= 2;
        }

        let mut bytes = [0u8; 17];
        bytes[0] = flags;
        bytes[1..9].clone_from_slice(&self.channels[0].config.to_bytes());
        bytes[9..17].clone_from_slice(&self.channels[1].config.to_bytes());
        bytes
    }
}
