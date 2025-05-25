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

    pub const fn new() -> Self {
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

impl<'a, const SIZE: usize> IntoIterator for &'a BitVec<SIZE>
where
    [(); SIZE.div_ceil(8)]: Sized,
{
    type Item = bool;
    type IntoIter = BitVecIterator<'a, SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        BitVecIterator {
            bit_vec: self,
            index: 0,
        }
    }
}
pub struct BitVecIterator<'a, const N: usize>
where
    [(); N.div_ceil(8)]: Sized,
{
    bit_vec: &'a BitVec<N>,
    index: u8,
}

impl<'a, const N: usize> Iterator for BitVecIterator<'a, N>
where
    [(); N.div_ceil(8)]: Sized,
{
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if (self.index as usize) >= N {
            return None;
        }

        let result = self.bit_vec.get(self.index);
        self.index += 1;
        Some(result)
    }
}
