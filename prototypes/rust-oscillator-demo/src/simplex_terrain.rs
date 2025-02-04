use crate::{
    interface::{ParamType, SoundAlgorithm, SoundParameter},
    utils::PerlinSequence1D,
};

pub struct SimplexOscillator {
    sample_rate: u32,
    phase: f32,
    drift: [(PerlinSequence1D, PerlinSequence1D); 8],
    debug_rollover_flag: bool,

    frequency: f32,
    radius: f32,
    aspect: f32,
    drift_speed: f32,
    drift_depth: f32,
    seed: i32,
    voices: i32,
}

impl SimplexOscillator {
    pub fn new() -> Self {
        Self {
            sample_rate: 0,
            phase: 0.0,
            drift: [(PerlinSequence1D::new(), PerlinSequence1D::new()); 8],
            debug_rollover_flag: false,

            frequency: 220.0,
            radius: 1.0,
            aspect: 1.0,
            drift_speed: 0.0,
            drift_depth: 0.0,
            seed: 0,
            voices: 1,
        }
    }
}

impl SoundAlgorithm for SimplexOscillator {
    fn get_name(&self) -> &'static str {
        "Simplex terrain"
    }

    fn debug_get_freq(&mut self) -> f32 {
        self.frequency
    }

    fn debug_get_and_clear_cycle_flag(&mut self) -> bool {
        let flag = self.debug_rollover_flag;
        self.debug_rollover_flag = false;
        flag
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    fn generate_sample(&mut self) -> f32 {
        debug_assert_ne!(self.sample_rate, 0);
        self.phase += self.frequency / self.sample_rate as f32;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.debug_rollover_flag = true;
        }

        let mut output = 0.0;

        for i in 0..self.voices {
            let x_offset = self.drift[i as usize]
                .0
                .next_sample(self.drift_speed, self.sample_rate)
                * self.drift_depth;
            let y_offset = self.drift[i as usize]
                .1
                .next_sample(self.drift_speed, self.sample_rate)
                * self.drift_depth;

            let x = (self.phase * 2.0 * std::f32::consts::PI).sin() * self.radius + x_offset;
            let y = (self.phase * 2.0 * std::f32::consts::PI).sin() * self.radius * self.aspect
                + y_offset;
            output +=
                opensimplex2::fast::noise3_ImproveXY(self.seed.into(), x.into(), y.into(), 0.0)
        }

        output / self.voices as f32
    }

    fn parameters(&self) -> Vec<SoundParameter> {
        vec![
            SoundParameter {
                value: self.frequency,
                name: "Frequency",
                param_type: ParamType::Float {
                    min: 22.0,
                    max: 880.0,
                },
            },
            SoundParameter {
                value: self.radius,
                name: "Radius",
                param_type: ParamType::Float {
                    min: 0.0,
                    max: 10.0,
                },
            },
            SoundParameter {
                value: self.aspect,
                name: "Aspect",
                param_type: ParamType::Float { min: 1.0, max: 5.0 },
            },
            SoundParameter {
                value: self.drift_speed,
                name: "Drift_speed",
                param_type: ParamType::Float { min: 0.0, max: 4.0 },
            },
            SoundParameter {
                value: self.drift_depth,
                name: "Drift_depth",
                param_type: ParamType::Float { min: 0.0, max: 4.0 },
            },
            SoundParameter {
                value: self.seed as f32,
                name: "Seed",
                param_type: ParamType::Float { min: 0.0, max: 4.0 },
            },
            SoundParameter {
                value: self.voices as f32,
                name: "Voices",
                param_type: ParamType::Float { min: 1.0, max: 4.0 },
            },
        ]
    }

    fn update_parameter(&mut self, name: &str, value: f32) {
        match name {
            "Frequency" => {
                self.frequency = value;
            }
            "Radius" => {
                self.radius = value;
            }
            "Aspect" => {
                self.aspect = value;
            }
            "Drift_speed" => {
                self.drift_speed = value;
            }
            "Drift_depth" => {
                self.drift_depth = value;
            }
            "Seed" => {
                self.seed = value.round() as i32;
            }
            "Voices" => {
                self.voices = value.round() as i32;
            }
            _ => panic!("Unexpected parameter name: {}", name),
        }
    }
}
