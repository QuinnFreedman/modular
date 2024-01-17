/**

Given the sequence of all powers of 2 and all negative powers of 2 (i.e. powers of 2
made negative, not negative exponents) excluding -1, like this:

...-8, -4, -2, 1, 2, 4, 8,...

This function steps along that sequence starting from `start` in a number and
direction given by `delta`. If `start` is not a power of 2, the first "step" will be
getting to the next power of 2 in the given direction.

E.g.

step(1, 2)   // -> 4
step(7, 3)   // -> 32
step(1, -2)  // -> -4
step(-4, 2)  // -> 1
step(-4, -2) // -> -16
step(-5, -3) // -> -32
step(-2, 1)  // -> 1
step(-1, 1)  // -> 1
step(0, 1)   // -> 1
step(0, -1)  // -> -2

*/
pub fn step_in_powers_of_2(mut start: i8, mut delta: i8) -> i8 {
    debug_assert!(delta != 0);

    // zero can cause subtraction overflow so handle it with special case
    if start == 0 || start == -1 {
        if delta > 0 {
            delta -= 1;
        }
        start = 1;
    }

    // (Exponent of) the largest power of 2 smaller than starting number
    let mut power_of_two = (i8::BITS - start.unsigned_abs().leading_zeros() - 1) as i8;

    // If we don't start at a power of 2, we "use up" one step getting to first power of 2
    if !start.unsigned_abs().is_power_of_two() {
        if (start >= 0) == (delta >= 0) {
            // If the delta is increasing the magnitude of the number, start at the next
            // larger power of 2
            power_of_two += 1;
            delta -= delta.signum();
        } else {
            delta += delta.signum()
        }
    }

    if start < 0 {
        power_of_two = -power_of_two;
    }

    power_of_two += delta;

    let result = (1 << power_of_two.unsigned_abs()) as i8;
    if power_of_two < 0 {
        -result
    } else {
        result
    }
}
