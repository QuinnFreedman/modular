use crate::shared::{get_delta_t, DriftModule};

pub struct LfoModuleState {
    time: u32,
}

impl LfoModuleState {
    pub fn new() -> Self {
        Self { time: 0 }
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

impl DriftModule for LfoModuleState {
    fn step(&mut self, cv: &[u16; 4]) -> u16 {
        let (t, _) = self.step_time(cv[2]);

        if t < u32::MAX / 2 {
            (t >> 19) as u16
        } else {
            0xFFFu16.saturating_sub(((t - u32::MAX / 2) >> 19) as u16)
        }
    }
}
