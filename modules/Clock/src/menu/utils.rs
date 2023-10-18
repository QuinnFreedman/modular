pub trait AddWithoutOverflow {
    fn add_without_overflow(&self, delta: i8) -> Self;
}

impl AddWithoutOverflow for u8 {
    fn add_without_overflow(&self, delta: i8) -> Self {
        if delta < 0 {
            self.saturating_sub((-delta) as u8)
        } else {
            self.saturating_add(delta as u8)
        }
    }
}

trait LowerPowerOfTwo {
    /**
    Returns the largest power of two less than the given number
    */
    fn lower_power_of_two(self) -> Self;
}

impl LowerPowerOfTwo for u8 {
    fn lower_power_of_two(self) -> Self {
        if self <= 1 {
            return 0;
        }
        let n = self - 1;

        let first_set_bit_index = Self::BITS - n.leading_zeros() - 1;

        1u8 << first_set_bit_index
    }
}

pub fn step_clock_division(mut current_value: i8, mut delta: i8) -> i8 {
    while delta != 0 {
        let sign = current_value.signum();
        let delta_sign = delta.signum();
        let abs_value = current_value.abs() as u8;
        current_value = if (delta_sign * sign) > 0 {
            (abs_value + 1).next_power_of_two().min(64)
        } else {
            abs_value.lower_power_of_two()
        } as i8
            * sign;
        if current_value == 0 {
            current_value = delta_sign;
        }
        delta -= delta_sign;
    }
    current_value
}
