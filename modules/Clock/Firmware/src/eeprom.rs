use core::mem::{self, offset_of};

use arduino_hal::Eeprom;
use avr_device::atmega328p::EEPROM;
use fm_lib::debug_unwrap::DebugUnwrap;

use crate::{
    clock::{ClockChannelConfig, ClockConfig},
    menu::{MAX_BPM, MIN_BPM},
};

struct EepromWrite {
    offset: u8,
    value: u8,
}

/**
On module startup, the previous clock config is loaded from EEPROM, if it was saved.
Then, the whole config is copied to another spot in EEPROM in order to spread out
the writes since each EEPROM bit can only take a limited number of writes in its
lifetime. All subsequent changes to clock state are written to the new location.

To save writes and clock cycles, updates are not written to EEPROM immediately.
Instead, one update at a time can be queued. Queue is flushed after a short delay
or when a change to a different field is queued.

This is very similar to `WearLevelledEepromWriter` in `fm-lib`, but because both
flash storage and memory are *very* constrained in this module, this is a slightly
simplified and specialized version to address only the needs of this module.
*/
pub struct PersistanceManager {
    eeprom: Eeprom,
    offset: u16,
    queued_write: EepromWrite,
}

fn is_valid_clock_config(data: &[u8; mem::size_of::<ClockConfig>()]) -> bool {
    let config: &ClockConfig = unsafe { mem::transmute(data) };
    if config.bpm < MIN_BPM || config.bpm > MAX_BPM {
        return false;
    }

    for channel in &config.channels {
        if channel.division > 64 || channel.division < -65 {
            return false;
        }

        if channel.pulse_width > 100 {
            return false;
        }

        if channel.phase_shift > 32 || channel.phase_shift < -32 {
            return false;
        }

        if channel.swing > 32 {
            return false;
        }
    }

    true
}

const NULL_OFFSET: u8 = u8::MAX;
const UNINITIALIZED: u8 = u8::MAX;

impl PersistanceManager {
    #[inline(never)]
    pub fn new(eeprom: EEPROM, clock_config: &mut ClockConfig) -> Self {
        let raw_data: &mut [u8; mem::size_of::<ClockConfig>()] =
            unsafe { mem::transmute(clock_config) };
        let mut eep = arduino_hal::Eeprom::new(eeprom);

        const BLOCK_SIZE: u16 = mem::size_of::<ClockConfig>() as u16 + 1;

        let mut latest_version: u8 = UNINITIALIZED;
        let mut latest_version_index: u16 = 0;

        for i in (0..eep.capacity()).step_by(BLOCK_SIZE as usize) {
            let version = eep.read_byte(i);
            if version != UNINITIALIZED
                && (version > latest_version || latest_version == UNINITIALIZED)
            {
                latest_version = version;
                latest_version_index = i;
            }
        }

        let mut new_index = 0;

        if latest_version == UNINITIALIZED {
            eep.write_byte(new_index, 0);
            eep.write(new_index + 1, raw_data).assert_ok();
        } else {
            let mut new_version = latest_version + 1;
            if new_version == UNINITIALIZED {
                new_version = 0;
                for i in (0..eep.capacity()).step_by(BLOCK_SIZE as usize) {
                    eep.erase_byte(i);
                }
            }
            new_index = latest_version_index + BLOCK_SIZE;
            if new_index + BLOCK_SIZE > eep.capacity() {
                new_index = 0;
            }
            eep.read(latest_version_index + 1, raw_data).assert_ok();

            if !is_valid_clock_config(&raw_data) {
                // If the loaded clock state is invalid (either because of a bug in
                // this code or because the EEPROM has been corrupted somehow, or
                // because it was modified before loading this firmware) the config
                // should be reset to default

                // TODO indicate to user somehow that this has happened
                *raw_data = unsafe { mem::transmute(ClockConfig::new()) }
            }

            eep.write_byte(new_index, new_version);
            eep.write(new_index + 1, raw_data).assert_ok();
        }

        Self {
            eeprom: eep,
            offset: new_index + 1,
            queued_write: EepromWrite {
                offset: NULL_OFFSET,
                value: 0,
            },
        }
    }

    pub fn overwrite(&mut self, new_config: &ClockConfig) {
        let raw_data: &[u8; mem::size_of::<ClockConfig>()] = unsafe { mem::transmute(new_config) };
        self.eeprom.write(self.offset, raw_data).assert_ok();
        self.queued_write.offset = NULL_OFFSET;
    }

    pub fn flush(&mut self) {
        if self.queued_write.offset != NULL_OFFSET {
            self.eeprom.write_byte(
                self.offset + self.queued_write.offset as u16,
                self.queued_write.value,
            );
            self.queued_write.offset = NULL_OFFSET;
        }
    }

    #[inline(always)]
    fn queue_write(&mut self, offset: u8, value: u8) {
        if self.queued_write.offset != NULL_OFFSET && self.queued_write.offset != offset {
            self.eeprom.write_byte(
                self.offset + self.queued_write.offset as u16,
                self.queued_write.value,
            );
        }
        debug_assert!((offset as usize) < mem::size_of::<ClockConfig>());
        self.queued_write = EepromWrite { offset, value }
    }

    fn write_channel_attribute(&mut self, channel: u8, offset: u8, value: u8) {
        let eeprom_offset = offset_of!(ClockConfig, channels) as u8
            + (channel * mem::size_of::<ClockChannelConfig>() as u8)
            + offset;
        self.queue_write(eeprom_offset, value);
    }

    #[inline(always)]
    pub fn set_bpm(&mut self, tempo: u8) {
        self.queue_write(offset_of!(ClockConfig, bpm) as u8, tempo);
    }

    #[inline(always)]
    pub fn set_invert_encoder(&mut self, invert: bool) {
        self.queue_write(
            offset_of!(ClockConfig, invert_encoder) as u8, 
            unsafe { mem::transmute(invert) },
        );
    }

    #[inline(always)]
    pub fn set_division(&mut self, channel: u8, division: i8) {
        self.write_channel_attribute(
            channel,
            offset_of!(ClockChannelConfig, division) as u8,
            unsafe { mem::transmute(division) },
        );
    }

    #[inline(always)]
    pub fn set_pulse_width(&mut self, channel: u8, pw: u8) {
        self.write_channel_attribute(
            channel,
            offset_of!(ClockChannelConfig, pulse_width) as u8,
            pw,
        );
    }

    #[inline(always)]
    pub fn set_phase_shift(&mut self, channel: u8, ps: i8) {
        self.write_channel_attribute(
            channel,
            offset_of!(ClockChannelConfig, phase_shift) as u8,
            unsafe { mem::transmute(ps) },
        );
    }

    #[inline(always)]
    pub fn set_swing(&mut self, channel: u8, sw: u8) {
        self.write_channel_attribute(channel, offset_of!(ClockChannelConfig, swing) as u8, sw);
    }

    #[inline(always)]
    pub fn set_probability(&mut self, channel: u8, probability: u8) {
        self.write_channel_attribute(
            channel,
            offset_of!(ClockChannelConfig, probability) as u8,
            probability
        );
    }
}
