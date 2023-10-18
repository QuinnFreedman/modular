// pub fn divide_with_remainder_u8<const DIVISOR: u8>(n: u8) -> (u8, u8) {
//     (n / DIVISOR, n % DIVISOR)
// }
pub fn divide_with_remainder_usize<const DIVISOR: usize>(n: usize) -> (usize, usize) {
    (n / DIVISOR, n % DIVISOR)
}

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
