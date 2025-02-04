use std::f32::consts::PI;

use crate::oscillator::Oscillator;

pub struct Downsampler<Osc, P>
where
    Osc: Oscillator<Params = P>,
{
    synth: Osc,
    oversampling_factor: usize,
    fir_coefficients: Vec<f32>, // Precomputed FIR filter coefficients
    fir_state: Vec<f32>,        // Circular buffer for filter state
    filter_index: usize,
    sample_counter: usize,
}

impl<Osc, P> Downsampler<Osc, P>
where
    Osc: Oscillator<Params = P>,
{
    pub fn new(synth: Osc, oversampling_factor: usize, fir_coefficients: Vec<f32>) -> Self {
        let filter_len = fir_coefficients.len();
        Self {
            synth,
            oversampling_factor,
            fir_coefficients,
            fir_state: vec![0.0; filter_len],
            filter_index: 0,
            sample_counter: 0,
        }
    }

    pub fn next_sample(&mut self, synth_params: &P) -> f32 {
        for _ in 0..self.oversampling_factor {
            // Get the next high-rate sample
            let sample = self.synth.step(synth_params);

            // Update filter state (circular buffer)
            self.fir_state[self.filter_index] = sample;
            self.filter_index = (self.filter_index + 1) % self.fir_state.len();
        }

        // Apply FIR filter to the state
        let mut filtered_sample = 0.0;
        for (i, coeff) in self.fir_coefficients.iter().enumerate() {
            let state_index = (self.filter_index + i) % self.fir_state.len();
            filtered_sample += self.fir_state[state_index] * coeff;
        }

        filtered_sample
    }
}

pub fn generate_fir_coefficients(downsampling_factor: usize, num_taps: usize) -> Vec<f32> {
    let cutoff_frequency = 1.0 / downsampling_factor as f32; // Normalized cutoff frequency
    let half_taps = (num_taps - 1) as isize / 2; // Center of the filter
    let mut coefficients = Vec::with_capacity(num_taps);

    for n in 0..num_taps as isize {
        // Calculate the sinc function value
        let x = n - half_taps;
        let sinc = if x == 0 {
            2.0 * PI * cutoff_frequency
        } else {
            (2.0 * PI * cutoff_frequency * x as f32).sin() / (x as f32)
        };

        // Apply the Hamming window
        let hamming = 0.54 - 0.46 * ((2.0 * PI * n as f32) / (num_taps as f32 - 1.0)).cos();

        // Combine sinc and window
        coefficients.push(sinc * hamming);
    }

    // Normalize the filter coefficients to ensure unity gain at DC
    let sum: f32 = coefficients.iter().sum();
    coefficients.iter_mut().for_each(|c| *c /= sum);

    coefficients
}
