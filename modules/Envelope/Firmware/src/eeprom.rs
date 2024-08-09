use core::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
};

use arduino_hal::Eeprom;
use avr_device::atmega328p::EEPROM;

struct WareLevelledEepromWriter<'a, const SIZE: u16, T>
where
    &'a T: Into<[u8; SIZE as usize]>,
    T: From<[u8; SIZE as usize]>,
{
    address: u16,
    generation: u16,
    eeprom: Eeprom,
    _t: PhantomData<&'a T>,
}

impl<'a, const SIZE: u16, T> WareLevelledEepromWriter<'a, SIZE, T>
where
    &'a T: Into<[u8; SIZE as usize]>,
    T: From<[u8; SIZE as usize]>,
{
    const TOTAL_SIZE: u16 = 2 + SIZE;

    pub fn init_and_advance(eeprom: EEPROM, object: &'a mut T) -> Self {
        let eep = arduino_hal::Eeprom::new(eeprom);

        let mut max_gen: u16 = 0;
        let mut max_gen_addr: u16 = 0;
        let mut found_address = false;

        for address in (0..eep.capacity() - Self::TOTAL_SIZE).step_by(Self::TOTAL_SIZE as usize) {
            let mut gen_bytes = [0u8; 2];
            eep.read(address, &mut gen_bytes).unwrap();
            let gen = u16::from_le_bytes(gen_bytes);
            if gen_bytes[1] != 0xFF && gen > max_gen {
                max_gen = gen;
                max_gen_addr = address;
                found_address = true;
            }
        }

        let mut writer = Self {
            address: max_gen_addr,
            generation: max_gen_addr,
            eeprom: eep,
            _t: PhantomData::<&'a T>::default(),
        };

        if !found_address {
            writer.write_data(object);
        } else {
            *object = writer.advance_and_copy();
        }

        writer
    }

    fn write_data(&mut self, data: &'a T) {
        let marker_bytes = self.generation.to_le_bytes();
        self.eeprom.erase_byte(self.address + 1);
        self.eeprom.write(self.address + 2, &data.into()).unwrap();
        self.eeprom.write_byte(self.address + 0, marker_bytes[0]);
        self.eeprom.write_byte(self.address + 1, marker_bytes[1]);
    }

    fn advance_and_copy(&mut self) -> T {
        let mut gen_bytes = [0u8; 2];
        self.eeprom.read(self.address, &mut gen_bytes).unwrap();
        let old_gen = u16::from_le_bytes(gen_bytes);
        debug_assert!(old_gen == self.generation);

        let mut new_address = self.address + Self::TOTAL_SIZE;
        if new_address + Self::TOTAL_SIZE > self.eeprom.capacity() {
            new_address = 0;
        }
        let new_gen = self.generation + 1;
        if new_gen >> 8 == 0xFF {
            self.clear();
        }

        let mut data: [u8; SIZE as usize] = unsafe { MaybeUninit::uninit().assume_init() };
        self.eeprom.read(self.address + 2, &mut data).unwrap();

        self.eeprom.erase_byte(new_address + 1);
        self.eeprom.write(new_address + 2, &data).unwrap();
        let new_gen_bytes = new_gen.to_le_bytes();
        self.eeprom.write_byte(new_address, new_gen_bytes[0]);
        self.eeprom.write_byte(new_address + 1, new_gen_bytes[1]);

        self.address = new_address;
        self.generation = new_gen;

        data.into()
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
        self.eeprom.write_byte(self.address + 2 + offset, byte);
    }
}
