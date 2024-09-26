use core::mem::{self, offset_of};

use arduino_hal::{prelude::_unwrap_infallible_UnwrapInfallible, Eeprom};
use avr_device::atmega328p::EEPROM;
use fm_lib::debug_unwrap::DebugUnwrap;
use ufmt::uwriteln;

use crate::clock::{ClockChannelConfig, ClockConfig};

pub struct PersistanceManager {}
impl PersistanceManager {
    #[inline(never)]
    pub fn new(_eeprom: EEPROM, _data: &mut [u8; mem::size_of::<ClockConfig>()]) -> Self {
        Self {}
    }

    pub fn flush(&mut self) {}

    #[inline(always)]
    pub fn set_bpm(&mut self, _: u8) {}

    #[inline(always)]
    pub fn set_division(&mut self, _: u8, _: i8) {}

    #[inline(always)]
    pub fn set_pulse_width(&mut self, _: u8, _: u8) {}

    #[inline(always)]
    pub fn set_phase_shift(&mut self, _: u8, _: i8) {}

    #[inline(always)]
    pub fn set_swing(&mut self, _: u8, _: u8) {}
}

/*

#[repr(packed)]
struct EepromWrite {
    offset: u8,
    value: u8,
}

#[repr(packed)]
pub struct PersistanceManager {
    eeprom: Eeprom,
    offset: u16,
    queued_write: EepromWrite,
}

impl PersistanceManager {
    #[inline(never)]
    pub fn new(eeprom: EEPROM, data: &mut [u8; mem::size_of::<ClockConfig>()]) -> Self {
        let mut eep = arduino_hal::Eeprom::new(eeprom);

        // for i in 0..eep.capacity() {
        //     eep.write_byte(i, 0xFF);
        // }

        const BLOCK_SIZE: u16 = mem::size_of::<ClockConfig>() as u16 + 1;

        let mut latest_version: u8 = 0xFF;
        let mut latest_version_index: u16 = 0;

        // let dp = unsafe { arduino_hal::Peripherals::steal() };
        // let pins = arduino_hal::pins!(dp);
        // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

        for i in (0..eep.capacity()).step_by(BLOCK_SIZE as usize) {
            let version = eep.read_byte(i);
            // uwriteln!(&mut serial, "check addr: {} version: {}", i, version).unwrap_infallible();
            if version != 0xFF && (version > latest_version || latest_version == 0xFF) {
                latest_version = version;
                latest_version_index = i;
            }
        }

        // uwriteln!(
        //     &mut serial,
        //     "old version: {} old index: {}",
        //     latest_version,
        //     latest_version_index
        // )
        // .unwrap_infallible();

        let mut new_index = 0;

        if latest_version == 0xFF {
            // uwriteln!(&mut serial, "no save; writing",).unwrap_infallible();

            eep.write_byte(new_index, 0);
            eep.write(new_index + 1, data).assert_ok();
        } else {
            let mut new_version = latest_version + 1;
            if new_version == 0xFF {
                new_version = 0;
                for i in (0..eep.capacity()).step_by(BLOCK_SIZE as usize) {
                    eep.erase_byte(i);
                }
            }
            new_index = latest_version_index + BLOCK_SIZE;
            if new_index + BLOCK_SIZE > eep.capacity() {
                new_index = 0;
            }
            // uwriteln!(&mut serial, "read data at {}", latest_version_index + 1).unwrap_infallible();
            eep.read(latest_version_index + 1, data).assert_ok();
            // uwriteln!(&mut serial, "data[0] = {}", data[0]).unwrap_infallible();
            // uwriteln!(
            //     &mut serial,
            //     "data[{}] = {}",
            //     data.len() - 1,
            //     data[data.len() - 1]
            // )
            // .unwrap_infallible();
            eep.write_byte(new_index, new_version);
            eep.write(new_index + 1, data).assert_ok();
        }

        Self {
            eeprom: eep,
            offset: new_index + 1,
            queued_write: EepromWrite {
                offset: 0xFF,
                value: 0,
            },
        }
    }

    pub fn flush(&mut self) {
        if self.queued_write.offset != 0xFF {
            self.eeprom.write_byte(
                self.offset + self.queued_write.offset as u16,
                self.queued_write.value,
            );
            self.queued_write.offset = 0xFF;
        }
    }

    #[inline(always)]
    fn queue_write(&mut self, offset: u8, value: u8) {
        if self.queued_write.offset != 0xFF && self.queued_write.offset != offset {
            self.eeprom.write_byte(
                self.offset + self.queued_write.offset as u16,
                self.queued_write.value,
            );
        }
        debug_assert!((offset as usize) < mem::size_of::<ClockConfig>());
        self.queued_write = EepromWrite { offset, value }
    }

    #[inline(always)]
    pub fn set_bpm(&mut self, tempo: u8) {
        self.queue_write(offset_of!(ClockConfig, bpm) as u8, tempo);
    }

    #[inline(always)]
    pub fn set_division(&mut self, channel: u8, division: i8) {
        let offset = offset_of!(ClockConfig, channels) as u8
            + (channel * offset_of!(ClockChannelConfig, division) as u8);
        self.queue_write(offset, unsafe { mem::transmute(division) });
    }

    #[inline(always)]
    pub fn set_pulse_width(&mut self, channel: u8, pw: u8) {
        let offset = offset_of!(ClockConfig, channels) as u8
            + (channel * offset_of!(ClockChannelConfig, pulse_width) as u8);
        self.queue_write(offset, pw);
    }

    #[inline(always)]
    pub fn set_phase_shift(&mut self, channel: u8, ps: i8) {
        let offset = offset_of!(ClockConfig, channels) as u8
            + (channel * offset_of!(ClockChannelConfig, phase_shift) as u8);
        self.queue_write(offset, unsafe { mem::transmute(ps) });
    }

    #[inline(always)]
    pub fn set_swing(&mut self, channel: u8, sw: u8) {
        let offset = offset_of!(ClockConfig, channels) as u8
            + (channel * offset_of!(ClockChannelConfig, swing) as u8);
        self.queue_write(offset, sw);
    }
}
*/
