pub struct BitVec<const SIZE: usize>
where
    [(); SIZE.div_ceil(8)]: Sized,
{
    data: [u8; SIZE.div_ceil(8)],
}

impl<const SIZE: usize> BitVec<SIZE>
where
    [(); SIZE.div_ceil(8)]: Sized,
{
    pub fn get(&self, i: u8) -> bool {
        assert!((i as usize) < SIZE);
        let byte_index = i / 8;
        let bit_index = i % 8;
        ((self.data[byte_index as usize] >> (bit_index)) & 1) != 0
    }

    pub fn set(&mut self, i: u8, value: bool) {
        assert!((i as usize) < SIZE);
        let byte_index = i / 8;
        let bit_index = i % 8;
        let bit_value = if value { 1 } else { 0 };
        self.data[byte_index as usize] |= bit_value << bit_index;
    }

    pub fn new() -> Self {
        Self {
            data: [0u8; SIZE.div_ceil(8)],
        }
    }

    pub fn from_bytes(data: [u8; SIZE.div_ceil(8)]) -> Self {
        Self { data }
    }

    pub fn get_bytes<'a>(&'a self) -> &'a [u8; SIZE.div_ceil(8)] {
        &self.data
    }
}
