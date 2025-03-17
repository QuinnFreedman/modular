use crate::interface::{SoundAlgorithm, SoundParameter};

pub struct Antialiased<Osc>
where
    Osc: SoundAlgorithm,
{
    osc: Osc,
    filter: ButterworthFilter,
}

impl<Osc> Antialiased<Osc>
where
    Osc: SoundAlgorithm,
{
    pub fn new(osc: Osc) -> Self {
        Self {
            osc,
            filter: ButterworthFilter::new(),
        }
    }
}

impl<Osc> SoundAlgorithm for Antialiased<Osc>
where
    Osc: SoundAlgorithm,
{
    fn get_name(&self) -> &'static str {
        self.osc.get_name()
    }

    fn debug_get_freq(&mut self) -> f32 {
        self.osc.debug_get_freq()
    }

    fn debug_get_and_clear_cycle_flag(&mut self) -> bool {
        self.osc.debug_get_and_clear_cycle_flag()
    }

    fn generate_sample(&mut self) -> f32 {
        const CUTOFF_FREQ: f32 = 21_000.0;
        let input = self.osc.generate_sample();
        self.filter.step(input, CUTOFF_FREQ);
        let input = self.osc.generate_sample();
        self.filter.step(input, CUTOFF_FREQ)
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.filter.sample_rate = sample_rate;
        self.osc.set_sample_rate(sample_rate * 2)
    }

    fn parameters(&self) -> Vec<SoundParameter> {
        self.osc.parameters()
    }

    fn update_parameter(&mut self, name: &str, value: f32) {
        self.osc.update_parameter(name, value)
    }
}

fn get_butterworth_lowpass_coefficients(
    cutoff_freq: f32,
    sampling_freq: f32,
) -> (f32, f32, f32, f32, f32) {
    let ff = cutoff_freq / sampling_freq;
    let alpha = 1.0 / f32::tan(core::f32::consts::PI * ff);
    let sqrt2 = f32::sqrt(2.0);
    let b0 = 1.0 / (1.0 + sqrt2 * alpha + alpha.powi(2));
    let b1 = 2.0 * b0;
    let b2 = b0;
    let a1 = 2.0 * (alpha.powi(2) - 1.0) * b0;
    let a2 = -(1.0 - sqrt2 * alpha + alpha.powi(2)) * b0;

    (b0, b1, b2, a1, a2)
}

struct ButterworthFilter {
    last_inputs: [f32; 2],
    last_outputs: [f32; 2],
    sample_rate: u32,
}

impl ButterworthFilter {
    fn new() -> Self {
        Self {
            last_inputs: [0.0; 2],
            last_outputs: [0.0; 2],
            sample_rate: 0,
        }
    }

    fn step(&mut self, input: f32, cutoff_freq: f32) -> f32 {
        let (b0, b1, b2, a1, a2) =
            get_butterworth_lowpass_coefficients(cutoff_freq, self.sample_rate as f32);

        let output = b0 * input
            + b1 * self.last_outputs[0]
            + b2 * self.last_outputs[1]
            + a1 * self.last_outputs[0]
            + a2 * self.last_outputs[1];

        self.last_inputs[1] = self.last_inputs[0];
        self.last_inputs[0] = input;
        self.last_outputs[1] = self.last_outputs[0];
        self.last_outputs[0] = output;

        output
    }
}
