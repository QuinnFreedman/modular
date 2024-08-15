use fixed::{types::extra::U12, FixedU16};
use fm_lib::rng::ParallelLfsr;

use crate::shared::{get_delta_t, DriftModule};

pub struct BezierModuleState {
    time: u32,
    value_a: FixedU16<U12>,
    value_b: FixedU16<U12>,
    rng: ParallelLfsr,
}

impl BezierModuleState {
    pub fn new(random_seed: u16) -> Self {
        let mut rng = ParallelLfsr::new(random_seed);
        Self {
            time: 0,
            value_a: FixedU16::<U12>::from_bits(0),
            value_b: FixedU16::<U12>::from_bits(rng.next() >> 4),
            rng,
        }
    }

    fn step_time(&mut self, cv: u16) -> (u32, bool) {
        let dt = get_delta_t(cv);
        self.time = self.time.saturating_add(dt);
        let rollover = self.time == u32::MAX;
        let before_rollover = self.time;
        if rollover {
            self.time = 0;
        }
        (before_rollover, rollover)
    }
}

fn bezier_curve(x: FixedU16<U12>) -> FixedU16<U12> {
    let x_2 = x * x;
    let x_3 = x_2 * x;
    3 * (x_2 - x_3) + x_3
}

fn bezier_interpolate(x: FixedU16<U12>, a: FixedU16<U12>, b: FixedU16<U12>) -> FixedU16<U12> {
    bezier_curve(x).lerp(a, b)
}

impl DriftModule for BezierModuleState {
    fn step(&mut self, cv: &[u16; 4]) -> u16 {
        let (t, rollover) = self.step_time(cv[2]);

        if rollover {
            self.value_a = self.value_b;
            self.value_b = FixedU16::<U12>::from_bits(self.rng.next() >> 4);
            return self.value_a.to_bits();
        }

        let t_fixed = FixedU16::<U12>::from_bits((t >> 20) as u16);

        let result = bezier_interpolate(t_fixed, self.value_a, self.value_b);
        debug_assert!(result <= 1);

        if result == 1 {
            0xFFF
        } else {
            result.to_bits()
        }
    }
}
