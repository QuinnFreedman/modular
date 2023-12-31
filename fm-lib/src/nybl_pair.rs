use core::marker::PhantomData;

#[derive(Clone, Copy)]
#[repr(transparent)]
/**
An abstraction to hold two 4-bit values in a single byte
*/
pub struct NyblPair<A, B>
where
    A: From<u8>,
    B: From<u8>,
    A: ~const Into<u8>,
    B: ~const Into<u8>,
{
    data: u8,
    // I'm not sure if the PhantomData is strictly necesary since anything that
    // is 4 bits is probably always pass-by-value but it doesn't hurt.
    a: PhantomData<A>,
    b: PhantomData<B>,
}

impl<A, B> NyblPair<A, B>
where
    A: From<u8>,
    B: From<u8>,
    A: ~const Into<u8>,
    B: ~const Into<u8>,
{
    /**
    Gets the "right" value of the pair, stored in the least-significant four bits
    */
    #[inline(always)]
    pub fn lsbs(&self) -> B {
        let value = self.data & 0x0f;
        value.into()
    }
    /**
    Gets the "left" value of the pair, stored in the most-significant four bits
    */
    #[inline(always)]
    pub fn msbs(&self) -> A {
        let value = self.data >> 4;
        value.into()
    }
    /**
    Returns both items as tuple: (left, right)
    */
    #[inline(always)]
    #[allow(dead_code)]
    pub fn as_tuple(&self) -> (A, B) {
        (self.msbs(), self.lsbs())
    }
    #[inline(always)]
    pub const fn new(msb: A, lsb: B) -> Self {
        let a_value: u8 = msb.into();
        let b_value: u8 = lsb.into();
        debug_assert!(a_value < 16);
        debug_assert!(b_value < 16);
        Self {
            data: a_value << 4 | b_value,
            a: PhantomData,
            b: PhantomData,
        }
    }
}
