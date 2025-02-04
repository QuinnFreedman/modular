use crate::interface::{ParamType, SoundAlgorithm, SoundParameter};
use crate::utils::{lerp, PerlinSequence1D, SinWaveVoice};

const NUM_VOICES: usize = 12;

pub struct HarmonicWtOsc {
    sample_rate: u32,
    frequency: f32,
    voices: [SinWaveVoice; NUM_VOICES],
    perlins: [PerlinSequence1D; NUM_VOICES],
    detune: [PerlinSequence1D; NUM_VOICES],
    sub: SinWaveVoice,
    perlin_freq: f32,
    perlin_amp: f32,
    harmonics: f32,
    harmonics_table: HarmonicsTable,
    ht_offset: f32,
    sub_amp: f32,
    detune_amount: f32,
    detune_speed: f32,
}

type HarmonicsTable = [[f32; NUM_VOICES]; 3];

fn make_table() -> HarmonicsTable {
    let mut saw = [0f32; NUM_VOICES];
    for i in 0..NUM_VOICES {
        let h = i + 1;
        saw[i] = 1.0 / h as f32
    }

    let mut square = [0f32; NUM_VOICES];
    for i in 0..NUM_VOICES {
        let h = i + 1;
        square[i] = if h % 2 == 0 { 0.0 } else { 1.0 / h as f32 };
    }

    let mut double_saw = [0f32; NUM_VOICES];
    for i in 0..NUM_VOICES {
        let h = i + 1;
        double_saw[i] = 1.0 / h as f32;
        if h % 2 == 0 {
            double_saw[i] += 2.0 / h as f32;
        }
        double_saw[i] /= 2.0;
    }

    let mut sin = [0f32; NUM_VOICES];
    sin[0] = 1.0;

    let mut even = [0f32; NUM_VOICES];
    for i in 0..NUM_VOICES {
        let h = i + 1;
        if h == 1 || h % 2 == 0 {
            even[i] = 1.0 / h as f32;
        }
    }

    [square, saw, double_saw]
}

fn get_coeffs(table: &HarmonicsTable, x: f32) -> [f32; NUM_VOICES] {
    let slice_size: f32 = 1.0 / table.len() as f32;
    let whole_part = x.div_euclid(slice_size);
    let frac_part = x.rem_euclid(slice_size);
    let max_idx = table.len() - 1;
    let idx_low = usize::min(whole_part as usize, max_idx);
    let idx_high = usize::min(idx_low + 1, max_idx);
    let u = frac_part * table.len() as f32;

    let mut result = [0f32; NUM_VOICES];
    for i in 0..result.len() {
        result[i] = lerp(table[idx_low][i]..=table[idx_high][i], u);
    }
    result
}

impl HarmonicWtOsc {
    pub fn new() -> Self {
        Self {
            sample_rate: 0,
            frequency: 220.0,
            voices: [SinWaveVoice::new(); NUM_VOICES],
            perlins: [PerlinSequence1D::new(); NUM_VOICES],
            detune: [PerlinSequence1D::new_random(); NUM_VOICES],
            sub: SinWaveVoice::new(),
            perlin_freq: 0.5,
            perlin_amp: 0.0,
            harmonics_table: make_table(),
            harmonics: 0.5,
            ht_offset: 0.0,
            sub_amp: 0.0,
            detune_amount: 0.0,
            detune_speed: 0.0,
        }
    }
}

impl SoundAlgorithm for HarmonicWtOsc {
    fn get_name(&self) -> &'static str {
        "Harmonic WT"
    }

    fn debug_get_freq(&mut self) -> f32 {
        self.frequency
    }

    fn debug_get_and_clear_cycle_flag(&mut self) -> bool {
        let flag = self.voices[0].debug_overflow_flag;
        self.voices[0].debug_overflow_flag = false;
        flag
    }

    fn generate_sample(&mut self) -> f32 {
        debug_assert_ne!(self.sample_rate, 0);
        let mut amplitudes = get_coeffs(&self.harmonics_table, self.ht_offset);

        for (i, amp) in amplitudes.iter_mut().enumerate() {
            if i != 0 {
                if self.harmonics < 0.5 {
                    *amp = lerp(0.0..=*amp, self.harmonics * 2.0);
                } else {
                    // TODO do something more interesting
                    *amp = lerp(*amp..=1.0, ((self.harmonics - 0.5) * 2.0).powi(2));
                }
            }

            let u = self.perlins[i].next_sample(self.perlin_freq, self.sample_rate) / 8.0 + 0.5;
            let delta = self.perlin_amp * *amp;
            let min_amp = f32::max(0.0, *amp - delta);
            let max_amp = *amp + delta;
            *amp = lerp(min_amp..=max_amp, u);
        }

        let total_amplitude: f32 = amplitudes.iter().sum();
        let mut sum = 0.0;
        for (i, voice) in self.voices.iter_mut().enumerate() {
            let amp = amplitudes[i];
            const MAX_DETUNE: f32 = 0.5;
            let detune = self.detune[i].next_sample(self.detune_speed, self.sample_rate) / 4.0
                * self.detune_amount
                * MAX_DETUNE;
            let freq = self.frequency * ((i + 1) as f32) * (detune + 1.0);
            sum += voice.next_sample(freq, self.sample_rate) * amp;
        }
        let mut mix = sum / total_amplitude;

        mix += self.sub.next_sample(self.frequency / 4.0, self.sample_rate) * self.sub_amp;

        mix
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    fn parameters(&self) -> Vec<SoundParameter> {
        vec![
            SoundParameter {
                name: "Frequency",
                value: self.frequency,
                param_type: ParamType::Float {
                    min: 22.0,
                    max: 1100.0,
                },
            },
            SoundParameter {
                name: "Shape",
                value: self.ht_offset,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Harmonics",
                value: self.harmonics,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Drift Speed",
                value: self.perlin_freq,
                param_type: ParamType::Float { min: 0.0, max: 0.5 },
            },
            SoundParameter {
                name: "Drift Amp",
                value: self.perlin_amp,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Sub",
                value: self.sub_amp,
                param_type: ParamType::Float { min: 0.0, max: 2.0 },
            },
            SoundParameter {
                name: "Detune Amount",
                value: self.detune_amount,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Detune Drift",
                value: self.detune_speed,
                param_type: ParamType::Float { min: 0.0, max: 0.5 },
            },
        ]
    }

    fn update_parameter(&mut self, name: &str, value: f32) {
        match name {
            "Frequency" => self.frequency = value,
            "Shape" => self.ht_offset = value,
            "Drift Speed" => self.perlin_freq = value,
            "Drift Amp" => self.perlin_amp = value,
            "Harmonics" => self.harmonics = value,
            "Sub" => self.sub_amp = value,
            "Detune Amount" => self.detune_amount = value,
            "Detune Drift" => self.detune_speed = value,
            _ => panic!("Unexpected parameter name: {}", name),
        }
    }
}
