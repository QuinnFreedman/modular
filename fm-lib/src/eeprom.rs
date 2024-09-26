use core::mem::MaybeUninit;

use arduino_hal::{prelude::_unwrap_infallible_UnwrapInfallible, Eeprom};
use avr_device::atmega328p::EEPROM;
use ufmt::uwriteln;

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

    #[inline(never)]
    pub fn init_and_advance<Validator>(
        eeprom: EEPROM,
        memory: &mut [u8; SIZE],
        clear: bool,
        is_valid: Validator,
    ) -> Self
    where
        Validator: Fn(&[u8; SIZE]) -> bool,
    {
        let mut eep = arduino_hal::Eeprom::new(eeprom);

        if clear {
            Self::clear_all(&mut eep);
        }

        let (address, version) = Self::binary_search_for_monotonic_ringbuffer_head(&eep);

        let mut writer = Self {
            address,
            version,
            eeprom: eep,
        };

        if version == 0xFFFF {
            writer.address = 0;
            writer.version = 0;
            writer.write_data(memory);
        } else {
            let temp = writer.advance_and_copy();
            if !is_valid(&temp) {
                let dp = unsafe { arduino_hal::Peripherals::steal() };
                let pins = arduino_hal::pins!(dp);
                let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
                uwriteln!(&mut serial, "invalid EEPROM; resetting").unwrap_infallible();

                Self::clear_all(&mut writer.eeprom);
                writer.address = 0;
                writer.version = 0;
                writer.write_data(memory);
            } else {
                *memory = temp;
            }
        }

        writer
    }

    fn write_data(&mut self, data: &[u8; SIZE]) {
        let marker_bytes = self.version.to_le_bytes();
        self.eeprom.erase_byte(self.address + 1);
        self.eeprom.write(self.address + 2, data).unwrap();
        self.eeprom.write_byte(self.address + 0, marker_bytes[0]);
        self.eeprom.write_byte(self.address + 1, marker_bytes[1]);
    }

    fn advance_and_copy(&mut self) -> [u8; SIZE] {
        let mut new_address = self.address + Self::TOTAL_SIZE;
        if new_address + Self::TOTAL_SIZE > self.eeprom.capacity() {
            new_address = 0;
        }
        let new_version = self.version + 1;
        if new_version >> 8 == 0xFF {
            self.clear();
            // NOTE resetting the address on version overflow is not ideal,
            // but it lets us keep the order invariant which loads much faster
            self.address = 0;
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

    fn clear_all(eeprom: &mut Eeprom) {
        for address in (0..=eeprom.capacity() - Self::TOTAL_SIZE).step_by(Self::TOTAL_SIZE as usize)
        {
            // eeprom.erase_byte(address);
            eeprom.erase_byte(address + 1);
        }
    }

    fn clear(&mut self) {
        for address in
            (0..=self.eeprom.capacity() - Self::TOTAL_SIZE).step_by(Self::TOTAL_SIZE as usize)
        {
            if address != self.address {
                self.eeprom.erase_byte(address + 1);
            }
        }
        self.eeprom.erase_byte(self.address);
        self.eeprom.erase_byte(self.address + 1);
    }

    pub fn update_byte(&mut self, offset: u16, byte: u8) {
        debug_assert!(offset < Self::DATA_SIZE);
        self.eeprom.write_byte(self.address + 2 + offset, byte);
    }

    fn load_version_number(eeprom: &Eeprom, index: u16) -> u16 {
        let address = index * Self::TOTAL_SIZE;
        let msb = eeprom.read_byte(address + 1);
        if msb == 0xFF {
            return 0xFFFF;
        }

        let lsb = eeprom.read_byte(address);
        u16::from_le_bytes([lsb, msb])
    }

    pub fn binary_search_for_monotonic_ringbuffer_head(eeprom: &Eeprom) -> (u16, u16) {
        let midpoint = |low, high| {
            debug_assert!(low <= high);
            low + (high - low) / 2
        };

        let gt = |a, b| {
            if b == 0xFFFF && a != 0xFFFF {
                true
            } else {
                a > b
            }
        };

        let len = eeprom.capacity() / Self::TOTAL_SIZE;
        let mut low_idx = 0;
        let mut high_idx = len - 1;
        let mut mid_idx = midpoint(low_idx, high_idx);
        let mut low_value = Self::load_version_number(eeprom, low_idx);
        let mut mid_value = Self::load_version_number(eeprom, mid_idx);
        let mut high_value = Self::load_version_number(eeprom, high_idx);

        loop {
            if gt(low_value, mid_value) {
                if low_idx + 1 == mid_idx {
                    return (low_idx * Self::TOTAL_SIZE, low_value);
                }
                high_idx = mid_idx;
                high_value = mid_value;
            } else if gt(mid_value, high_value) {
                if mid_idx + 1 == high_idx {
                    return (mid_idx * Self::TOTAL_SIZE, mid_idx);
                }
                low_idx = mid_idx;
                low_value = mid_value;
            } else {
                return (low_idx * Self::TOTAL_SIZE, low_value);
            }

            mid_idx = midpoint(low_idx, high_idx);
            mid_value = Self::load_version_number(eeprom, mid_idx);
        }
    }
}
