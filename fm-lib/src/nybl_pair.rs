use core::marker::PhantomData;

use crate::const_traits::{ConstInto, ConstFrom};

#[derive(Clone, Copy)]
#[repr(transparent)]
/**
An abstraction to hold two 4-bit values in a single byte
*/
pub struct NyblPair<A, B>
where
    A: ConstFrom<u8>,
    B: ConstFrom<u8>,
    A: ConstInto<u8>,
    B: ConstInto<u8>,
{
    data: u8,
    // I'm not sure if the PhantomData is strictly necesary since anything that
    // is 4 bits is probably always pass-by-value but it doesn't hurt.
    a: PhantomData<A>,
    b: PhantomData<B>,
}

#[const_trait]
pub trait ConstNyblPair<A, B>
where
    A: ~const ConstFrom<u8>,
    B: ~const ConstFrom<u8>,
    A: ~const ConstInto<u8>,
    B: ~const ConstInto<u8>,
 {
    /**
    Gets the "right" value of the pair, stored in the least-significant four bits
    */
    fn lsbs(&self) -> B ;

    /**
    Gets the "left" value of the pair, stored in the most-significant four bits
    */
    fn msbs(&self) -> A ;
    /**
    Returns both items as tuple: (left, right)
    */
    fn as_tuple(&self) -> (A, B) ;

    fn new(msb: A, lsb: B) -> Self;
}

impl<A, B> const ConstNyblPair<A, B>  for NyblPair<A, B>
where
    A: ~const ConstFrom<u8>,
    B: ~const ConstFrom<u8>,
    A: ~const ConstInto<u8>,
    B: ~const ConstInto<u8>,
{
    /**
    Gets the "right" value of the pair, stored in the least-significant four bits
    */
    #[inline(always)]
    fn lsbs(&self) -> B {
        let value = self.data & 0x0f;
        ConstFrom::<u8>::const_from(value)
    }
    /**
    Gets the "left" value of the pair, stored in the most-significant four bits
    */
    #[inline(always)]
    fn msbs(&self) -> A {
        let value = self.data >> 4;
        ConstFrom::<u8>::const_from(value)
    }
    /**
    Returns both items as tuple: (left, right)
    */
    #[inline(always)]
    #[allow(dead_code)]
     fn as_tuple(&self) -> (A, B) {
        (self.msbs(), self.lsbs())
    }
    #[inline(always)]
     fn new(msb: A, lsb: B) -> Self {
        let a_value: u8 = msb.const_into();
        let b_value: u8 = lsb.const_into();
        debug_assert!(a_value < 16);
        debug_assert!(b_value < 16);
        Self {
            data: a_value << 4 | b_value,
            a: PhantomData,
            b: PhantomData,
        }
    }
}
