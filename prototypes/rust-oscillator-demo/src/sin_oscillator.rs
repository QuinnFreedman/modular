use crate::interface::{ParamType, SoundAlgorithm, SoundParameter};

pub struct SinOscillator {
    sample_rate: u32,
    frequency: f32,
    phase: f32,
    debug_cycle_flag: bool,
}

impl SinOscillator {
    pub fn new() -> Self {
        Self {
            sample_rate: 0,
            frequency: 220.0,
            phase: 0.0,
            debug_cycle_flag: false,
        }
    }
}

impl SoundAlgorithm for SinOscillator {
    fn get_name(&self) -> &'static str {
        "Sine Wave"
    }

    fn debug_get_freq(&mut self) -> f32 {
        self.frequency
    }

    fn debug_get_and_clear_cycle_flag(&mut self) -> bool {
        let flag = self.debug_cycle_flag;
        self.debug_cycle_flag = false;
        flag
    }

    fn generate_sample(&mut self) -> f32 {
        assert_ne!(self.sample_rate, 0);
        self.phase += self.frequency / self.sample_rate as f32;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.debug_cycle_flag = true;
        }

        (self.phase * 2.0 * std::f32::consts::PI).sin()
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    fn parameters(&self) -> Vec<SoundParameter> {
        vec![SoundParameter {
            name: "Frequency",
            value: self.frequency,
            param_type: ParamType::Float {
                min: 22.0,
                max: 1100.0,
            },
        }]
    }

    fn update_parameter(&mut self, name: &str, value: f32) {
        match name {
            "Frequency" => self.frequency = value,
            _ => panic!("Unexpected parameter name: {}", name),
        }
    }
}
