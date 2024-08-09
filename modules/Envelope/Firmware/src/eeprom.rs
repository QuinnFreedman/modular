use core::mem::MaybeUninit;

use arduino_hal::Eeprom;
use avr_device::atmega328p::EEPROM;

/**
Every time the writer is initialized (i.e. on device startup) the full object is
copied to a new location in EEPROM. From then on, updates can be made quickly in
place, while a mirrored copy of any relevant state is kept in memory by the user.
EEPROM should not be read from again until the next startup.
*/
pub struct WearLevelledEepromWriter<const SIZE: usize> {
    pub address: u16,
    pub version: u16,
    eeprom: Eeprom,
}

impl<const SIZE: usize> WearLevelledEepromWriter<SIZE> {
    const DATA_SIZE: u16 = SIZE as u16;
    const TOTAL_SIZE: u16 = 2 + Self::DATA_SIZE;

    pub fn init_and_advance(eeprom: EEPROM, memory: &mut [u8; SIZE]) -> Self {
        let mut eep = arduino_hal::Eeprom::new(eeprom);

        let mut max_version: u16 = 0;
        let mut max_version_addr: u16 = 0;
        let mut found_address = false;

        for address in (0..eep.capacity() - Self::TOTAL_SIZE).step_by(Self::TOTAL_SIZE as usize) {
            let mut version_bytes = [0u8; 2];
            eep.read(address, &mut version_bytes).unwrap();
            let version = u16::from_le_bytes(version_bytes);
            if version_bytes[1] != 0xFF && (max_version == 0 || version > max_version) {
                max_version = version;
                max_version_addr = address;
                found_address = true;
            }
        }

        let mut writer = Self {
            address: max_version_addr,
            version: max_version,
            eeprom: eep,
        };

        if !found_address {
            writer.write_data(memory);
        } else {
            *memory = writer.advance_and_copy();
        }

        writer
    }

    // TODO assume EEPROM is in order, use bin search
    // fn binary_search_for_sector_with_highest_version(&self) {}

    fn write_data(&mut self, data: &[u8; SIZE]) {
        let marker_bytes = self.version.to_le_bytes();
        self.eeprom.erase_byte(self.address + 1);
        self.eeprom.write(self.address + 2, data).unwrap();
        self.eeprom.write_byte(self.address + 0, marker_bytes[0]);
        self.eeprom.write_byte(self.address + 1, marker_bytes[1]);
    }

    fn advance_and_copy(&mut self) -> [u8; SIZE] {
        let mut version_bytes = [0u8; 2];
        self.eeprom.read(self.address, &mut version_bytes).unwrap();
        let old_version = u16::from_le_bytes(version_bytes);
        debug_assert!(old_version == self.version);

        let mut new_address = self.address + Self::TOTAL_SIZE;
        if new_address + Self::TOTAL_SIZE > self.eeprom.capacity() {
            new_address = 0;
        }
        let new_version = self.version + 1;
        if new_version >> 8 == 0xFF {
            self.clear();
        }

        let mut data: [u8; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        self.eeprom.read(self.address + 2, &mut data).unwrap();

        self.eeprom.erase_byte(new_address + 1);
        self.eeprom.write(new_address + 2, &data).unwrap();
        let new_version_bytes = new_version.to_le_bytes();
        self.eeprom.write_byte(new_address, new_version_bytes[0]);
        self.eeprom
            .write_byte(new_address + 1, new_version_bytes[1]);

        self.address = new_address;
        self.version = new_version;

        data
    }

    fn clear(&mut self) {
        for address in
            (0..self.eeprom.capacity() - Self::TOTAL_SIZE).step_by(Self::TOTAL_SIZE as usize)
        {
            if address != self.address {
                self.eeprom.erase_byte(address + 1);
            }
        }
        self.eeprom.erase_byte(self.address);
        self.eeprom.erase_byte(self.address + 1);
    }

    /*
    pub fn read(&mut self) -> T {
        // TODO make T implement default and return default if error
        debug_assert!(self.address + Self::TOTAL_SIZE <= self.eeprom.capacity());
        let mut gen_bytes = [0u8; 2];
        self.eeprom.read(self.address, &mut gen_bytes).unwrap();
        let gen = u16::from_le_bytes(gen_bytes);
        debug_assert!(gen >> 8 != 0xFF);
        debug_assert!(gen == self.generation);

        let mut data_bytes = [0u8; SIZE as usize];
        self.eeprom.read(self.address + 2, &mut data_bytes).unwrap();
        T::from(data_bytes)
    }
    */

    pub fn update_byte(&mut self, offset: u16, byte: u8) {
        debug_assert!(offset < Self::DATA_SIZE);
        self.eeprom.write_byte(self.address + 2 + offset, byte);
    }
}
