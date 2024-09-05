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
        let step_size = 256 + cv >> 1;
        let cutoff = cv << 6;

        let random = self.rng.next();
        if random < cutoff {
            const CENTERING_MARGIN: u16 = 5;
            const CENTERING_STRENGTH: u16 = 64;
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
        // TODO: These controls would maybe be more useful if they had an exponential curve (especially texture)
        self.step_target_value(u16::min(1023, cv[2] + cv[0]));
        self.step_smoothed_value(u16::min(1023, cv[3] + cv[1]));
        self.current_value >> 4
    }
}
