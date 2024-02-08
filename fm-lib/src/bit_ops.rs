pub trait BitOps {
    fn set_bit(&mut self, i: u8);
    fn clear_bit(&mut self, i: u8);
    fn write_bit(&mut self, i: u8, value: bool);
    fn get_bit(&self, i: u8) -> bool;
}

impl BitOps for u8 {
    #[inline(always)]
    fn set_bit(&mut self, i: u8) {
        *self |= 1 << i;
    }

    #[inline(always)]
    fn clear_bit(&mut self, i: u8) {
        *self &= !(1 << i);
    }

    #[inline(always)]
    fn write_bit(&mut self, i: u8, value: bool) {
        if value {
            self.set_bit(i);
        } else {
            self.clear_bit(i);
        }
    }

    #[inline(always)]
    fn get_bit(&self, i: u8) -> bool {
        (self & (0b1 << i)) != 0
    }
}
