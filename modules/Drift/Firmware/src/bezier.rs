use fixed::{types::extra::U12, FixedI16, FixedU16};
use fm_lib::rng::ParallelLfsr;

use crate::shared::{get_delta_t, DriftModule};

pub struct BezierModuleState {
    time: u32,
    speed_adjust: i16,
    value_a: FixedU16<U12>,
    value_b: FixedU16<U12>,
    rng: ParallelLfsr,
}

impl BezierModuleState {
    pub fn new(random_seed: u16) -> Self {
        let mut rng = ParallelLfsr::new(random_seed);
        Self {
            time: 0,
            speed_adjust: 0,
            value_a: FixedU16::<U12>::from_bits(0),
            value_b: FixedU16::<U12>::from_bits(rng.next() >> 4),
            rng,
        }
    }

    fn step_time(&mut self, knob: u16, cv: u16) -> (u32, bool) {
        let dt = get_delta_t(knob, cv, self.speed_adjust);
        self.time = self.time.saturating_add(dt);
        let rollover = self.time == u32::MAX;
        let before_rollover = self.time;
        if rollover {
            self.time = 0;
        }
        (before_rollover, rollover)
    }

    fn get_speed_adjust(&mut self, knob: u16, cv: u16) -> i16 {
        let sum = u16::max(knob + cv, 1023);
        const HALF: u16 = 1023 / 2;
        const DEAD_ZONE: u16 = 200;
        const RANGE: u16 = HALF - DEAD_ZONE;
        let magnitude =
            (if sum > HALF { sum - HALF } else { HALF - sum }).saturating_sub(DEAD_ZONE);

        if magnitude == 0 {
            return 0;
        }

        let random = bipolar_random(&mut self.rng);
        ((random as i32 * magnitude as i32) / (RANGE as i32)) as i16
    }
}

// I thought about letting the texture knob control the "c" parameter to smoothly
// transition between bezier easing to lerp to reverse bezier, but this method of
// computing the curve (by first incrementing the time variable and then evaluating
// the curve as a closed-form expression of time) is discontinuous with regard to c,
// turning the knob quickly caused a jagged section of output. You could compute
// the derivative of the bezier curve and then calculate the output incrementally,
// but I worry about numerical precision and the derivative == 0 endpoints
fn parametric_bezier_curve(x: FixedU16<U12>, c: FixedU16<U12>) -> FixedU16<U12> {
    debug_assert!(x <= 1);
    debug_assert!(c <= 2);
    let x_2 = x * x;
    let x_3 = x_2 * x;
    let inv_x = FixedU16::<U12>::from_bits(1 << 12) - x;
    let inv_x_2 = inv_x * inv_x;
    let three = FixedU16::<U12>::from_num(3);

    c * inv_x_2 * x + (three - c) * inv_x * x_2 + x_3
}

/**
A special case of the bezier smoothing function

```
c * (1-x)^2 * x + (3-c) * (1-x) * x^2 + x^3
```

where c=0 to make the slope horizontal at the endpoints
for a smooth easing function
*/
fn smooth_bezier_curve(x: FixedU16<U12>) -> FixedU16<U12> {
    let x_2 = x * x;
    let x_3 = x_2 * x;
    3 * (x_2 - x_3) + x_3
}

/**
A special case of the bezier smoothing function

```
c * (1-x)^2 * x + (3-c) * (1-x) * x^2 + x^3
```

where c=2 to make an "inverted"/"spiky" easing curve
*/
fn unsmooth_bezier_curve(x: FixedU16<U12>) -> FixedU16<U12> {
    let x_2 = x * x;
    let x_3 = x_2 * x;
    2 * x_3 + 2 * x - 3 * x_2
}

fn bezier_interpolate(x: FixedU16<U12>, a: FixedU16<U12>, b: FixedU16<U12>) -> FixedU16<U12> {
    smooth_bezier_curve(x).lerp(a, b)
}

fn reverse_bezier_interpolate(
    x: FixedU16<U12>,
    a: FixedU16<U12>,
    b: FixedU16<U12>,
) -> FixedU16<U12> {
    unsmooth_bezier_curve(x).lerp(a, b)
}

impl DriftModule for BezierModuleState {
    fn step(&mut self, cv: &[u16; 4]) -> u16 {
        let (t, rollover) = self.step_time(cv[2], 0 /* TODO read cv */);

        if rollover {
            self.value_a = self.value_b;
            self.value_b = FixedU16::<U12>::from_bits(self.rng.next() >> 4);
            self.speed_adjust = self.get_speed_adjust(cv[3], 0 /* TODO read cv */);
            return self.value_a.to_bits();
        }

        let t_fixed = FixedU16::<U12>::from_bits((t >> 20) as u16);

        let result = if cv[3] < 1024 / 2 {
            reverse_bezier_interpolate(t_fixed, self.value_a, self.value_b)
        } else {
            bezier_interpolate(t_fixed, self.value_a, self.value_b)
        };
        debug_assert!(result <= 1);

        if result == 1 {
            0xFFF
        } else {
            result.to_bits()
        }
    }
}

fn bipolar_random(rng: &mut ParallelLfsr) -> i16 {
    let half = u16::MAX / 2;
    let random = rng.next();
    if random > half {
        (random - half) as i16
    } else {
        -((half - random) as i16)
    }
}

/*
fn sample_from_distribution<const MEAN: u32, const RANGE: u32>(
    stdev_frac: FixedU16<U12>,
    rng: &mut ParallelLfsr,
) -> u32 {
    let random = gaussian_random(rng, stdev_frac);
    if random > 0 {
        MEAN + RANGE / (1 << 12) * random.to_bits() as u32
    } else {
        MEAN - RANGE / (1 << 12) * random.neg().to_bits() as u32
    }
}

fn gaussian_random(rng: &mut ParallelLfsr, stdev: FixedU16<U12>) -> FixedI16<U12> {
    // TODO placeholder
    // always return a value between -1 and 1
    debug_assert!(stdev <= 1);
    let random_uniform = FixedI16::<U12>::from_bits(rng.next() as i16) / 16;
    debug_assert!(random_uniform > -1);
    debug_assert!(random_uniform < 1);
    random_uniform
}
*/
