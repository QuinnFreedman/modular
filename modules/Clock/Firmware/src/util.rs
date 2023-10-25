/*
trait DivideWithRemainderU8 {
    fn divide_with_remainder<const N: u8>(self) -> (u8, u8);
}
trait DivideWithRemainderUsize {
    fn divide_with_remainder<const N: usize>(self) -> (usize, usize);
}

impl DivideWithRemainderU8 for u8 {
    fn divide_with_remainder<const DIVISOR: u8>(self) -> (u8, u8) {
        (self / DIVISOR, self % DIVISOR)
    }
}

impl DivideWithRemainderUsize for usize {
    fn divide_with_remainder<const DIVISOR: usize>(self) -> (usize, usize) {
        (self / DIVISOR, self % DIVISOR)
    }
}
*/
