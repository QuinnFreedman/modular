use crate::{
    bezier::BezierModuleState, brownian::BrownianModuleState, lfo::LfoModuleState,
    perlin::PerlinModuleState, shared::DriftModule,
};

pub enum DriftAlgorithm {
    Perlin(PerlinModuleState),
    Bezier(BezierModuleState),
    Brownian(BrownianModuleState),
    Lfo(LfoModuleState),
}

impl DriftAlgorithm {
    pub fn new(config: [bool; 2], random_seed: u16) -> Self {
        match (config[0], config[1]) {
            (true, true) => Self::Perlin(PerlinModuleState::new(random_seed)),
            (false, true) => Self::Brownian(BrownianModuleState::new(random_seed)),
            (true, false) => Self::Bezier(BezierModuleState::new(random_seed)),
            (false, false) => Self::Lfo(LfoModuleState::new()),
        }
    }

    pub fn step(&mut self, cv: &[u16; 4]) -> u16 {
        match self {
            DriftAlgorithm::Perlin(ref mut state) => state.step(cv),
            DriftAlgorithm::Bezier(ref mut state) => state.step(cv),
            DriftAlgorithm::Brownian(ref mut state) => state.step(cv),
            DriftAlgorithm::Lfo(ref mut state) => state.step(cv),
        }
    }
}
