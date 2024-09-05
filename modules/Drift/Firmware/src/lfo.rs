use fixed::{types::extra::U16, FixedU32};

use crate::shared::{get_delta_t, DriftModule};

pub struct LfoModuleState {
    time: u32,
    apex: u32,
}

impl LfoModuleState {
    pub fn new() -> Self {
        Self {
            time: 0,
            apex: u32::MAX / 2,
        }
    }

    fn step_time(&mut self, knob: u16, cv: u16) -> (u32, bool) {
        let dt = get_delta_t(knob, cv, 0);
        self.time = self.time.saturating_add(dt);
        let rollover = self.time == u32::MAX;
        let before_rollover = self.time;
        if rollover {
            self.time = 0;
        }
        (before_rollover, rollover)
    }
}

impl DriftModule for LfoModuleState {
    fn step(&mut self, cv: &[u16; 4]) -> u16 {
        let (t, rollover) = self.step_time(cv[2], cv[0]);

        if rollover {
            // TODO this isn't great
            // This only updates the shape of the wave between cycles.
            // Ideally, the shape would be continuously adjustable for FM and
            // better user feedback. But you can't do that in closed form without
            // getting discontinuities when you sweep skew, and when I tried to
            // do it numerically it messed with the overall cycle time at high freq
            // due to integer precision issues. This is definitely solvable but
            // just doing this as a simple stopgap for now
            self.apex = (u16::min(1023, cv[3] + cv[1]) as u32) << 22;
        }

        let value = if t < self.apex {
            let completed = FixedU32::<U16>::from_bits(t >> 16);
            let out_of = FixedU32::<U16>::from_bits(self.apex >> 16);
            (completed / out_of).to_bits() as u16
        } else {
            let completed = FixedU32::<U16>::from_bits((t - self.apex) >> 16);
            let out_of = FixedU32::<U16>::from_bits((u32::MAX - self.apex) >> 16);
            let y = u32::min((completed / out_of).to_bits(), u16::MAX as u32) as u16;
            u16::MAX - y
        };

        value >> 4
    }
}
