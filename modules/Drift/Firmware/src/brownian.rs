use fixed::{types::extra::U16, FixedU16};
use fm_lib::rng::ParallelLfsr;

use crate::shared::DriftModule;

pub struct BrownianModuleState {
    target_value: u16,
    current_value: u16,
    rng: ParallelLfsr,
}

impl BrownianModuleState {
    pub fn new(random_seed: u16) -> Self {
        let rng = ParallelLfsr::new(random_seed);
        Self {
            target_value: 0,
            current_value: 0,
            rng,
        }
    }

    fn step_target_value(&mut self, cv: u16) {
        // TODO dial in step size after sample rate is locked in
        let step_size = 128;

        let random = self.rng.next();
        let cutoff = cv << 6;

        if random < cutoff {
            const CENTERING_MARGIN: u16 = 5;
            const CENTERING_STRENGTH: u16 = 256;
            let cutoff2 = if self.target_value < u16::MAX / CENTERING_MARGIN {
                (cutoff / 2) - (cutoff / CENTERING_STRENGTH)
            } else if self.target_value > u16::MAX - (u16::MAX / CENTERING_MARGIN) {
                (cutoff / 2) + (cutoff / CENTERING_STRENGTH)
            } else {
                cutoff / 2
            };

            if random >= cutoff2 {
                self.target_value = self.target_value.saturating_add(step_size);
            } else {
                self.target_value = self.target_value.saturating_sub(step_size);
            }
        }
    }

    fn step_smoothed_value(&mut self, cv: u16) {
        if cv >= 1020 {
            self.current_value = self.target_value;
            return;
        }

        let cv_fixed = FixedU16::<U16>::from_bits(cv);
        let delta = FixedU16::<U16>::from_bits(self.current_value.abs_diff(self.target_value));

        const MAX_STEP_SIZE: FixedU16<U16> = FixedU16::<U16>::from_bits(u16::MAX / 8);
        const MIN_STEP_SIZE: FixedU16<U16> = FixedU16::<U16>::from_bits(u16::MAX / 4096);

        let step_size = cv_fixed.lerp(MIN_STEP_SIZE, MAX_STEP_SIZE);

        if self.current_value < self.target_value {
            self.current_value += (step_size * delta).to_bits();
        } else if self.current_value > self.target_value {
            self.current_value -= (step_size * delta).to_bits();
        }
    }
}

impl DriftModule for BrownianModuleState {
    fn step(&mut self, cv: &[u16; 4]) -> u16 {
        self.step_target_value(cv[2] /* TODO read cv*/);
        self.step_smoothed_value(cv[3]);
        self.current_value >> 4
    }
}
