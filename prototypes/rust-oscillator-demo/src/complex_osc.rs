use rand::random;

use crate::{
    interface::{ParamType, SoundAlgorithm, SoundParameter},
    utils::{lerp, FirstOrderIIRLowpassFilter, SinWaveVoice},
};

const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

pub struct ComplexOscillator {
    carrier: Voice,
    modulator: Voice,
    osc_c: SinWaveVoice,
    last_noise_sample: f32,
    sample_rate: u32,
    sync_counter: u32,
    debug_cycle_flag: bool,
    lpf: FirstOrderIIRLowpassFilter,

    carrier_freq: f32,
    carrier_morph: f32,
    mod_freq_ratio: f32,
    mod_morph: f32,
    mod_sync: SyncMode,
    alpha: f32,
    mode: ComplexMode,
    osc_c_freq: f32,
    osc_c_amp: f32,
    osc_c_mode: OscCMode,
    noise_level: f32,
    lowpass_freq: f32,
}

impl ComplexOscillator {
    pub fn new() -> Self {
        Self {
            carrier: Voice::new(),
            modulator: Voice::new(),
            osc_c: SinWaveVoice::new(),
            last_noise_sample: 0.0,
            sync_counter: 0,
            sample_rate: 0,
            carrier_freq: 220.0,
            carrier_morph: 0.0,
            mod_freq_ratio: 0.0,
            mod_morph: 0.0,
            mod_sync: SyncMode::Free,
            alpha: 0.0,
            mode: ComplexMode::Crossfade,
            osc_c_freq: 0.0,
            osc_c_amp: 0.0,
            osc_c_mode: OscCMode::Vibrato,
            noise_level: 0.0,
            debug_cycle_flag: false,
            lpf: FirstOrderIIRLowpassFilter::new(),
            lowpass_freq: 22000.0,
        }
    }
}

impl SoundAlgorithm for ComplexOscillator {
    fn get_name(&self) -> &'static str {
        "Complex Pair"
    }

    fn debug_get_freq(&mut self) -> f32 {
        self.carrier_freq
    }

    fn debug_get_and_clear_cycle_flag(&mut self) -> bool {
        let flag = self.debug_cycle_flag;
        self.debug_cycle_flag = false;
        flag
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    fn generate_sample(&mut self) -> f32 {
        debug_assert_ne!(self.sample_rate, 0);

        let osc_freq = match self.osc_c_mode {
            OscCMode::Vibrato => lerp(0.1..=220.0, self.osc_c_freq),
            OscCMode::Sub => {
                let osc_c_freq_coeff = if self.osc_c_freq > 0.5 {
                    ((self.osc_c_freq - 0.5) * 14.0).round() + 1.0
                } else {
                    1.0 / (((0.5 - self.osc_c_freq) * 14.0).round() + 1.0)
                };
                self.carrier_freq * osc_c_freq_coeff
            }
            _ => {
                let osc_c_freq_coeff = if self.osc_c_freq > 0.5 {
                    (self.osc_c_freq - 0.5) * 14.0 + 1.0
                } else {
                    1.0 / ((0.5 - self.osc_c_freq) * 14.0 + 1.0)
                };
                self.carrier_freq * osc_c_freq_coeff
            }
        };

        let osc_c_value = self.osc_c.next_sample(osc_freq, self.sample_rate) * self.osc_c_amp;

        const MAX_VIB_DEPTH: f32 = 0.2;
        let vibrato_coeff = match self.osc_c_mode {
            OscCMode::Vibrato | OscCMode::FM => 1.0 + osc_c_value * MAX_VIB_DEPTH,
            _ => 1.0,
        };

        let alpha = if self.osc_c_mode == OscCMode::Alpha {
            (self.alpha + osc_c_value).clamp(0.0, 1.0)
        } else {
            self.alpha
        };

        let mod_freq_coeff = if self.mod_freq_ratio > 0.0 {
            let quantized = if self.mod_sync == SyncMode::Quantize {
                quantize(self.mod_freq_ratio)
            } else {
                self.mod_freq_ratio
            };
            quantized + 1.0
        } else {
            let positive = -self.mod_freq_ratio;
            let quantized = if self.mod_sync == SyncMode::Quantize {
                quantize(positive)
            } else {
                positive
            };
            1.0 / (quantized + 1.0)
        };

        let b_freq = self.carrier_freq * mod_freq_coeff * vibrato_coeff;

        let b_morph = if self.osc_c_mode == OscCMode::OscBWt {
            (self.mod_morph + osc_c_value).clamp(0.0, 1.0)
        } else {
            self.mod_morph
        };

        let b = self.modulator.step(b_freq, b_morph, self.sample_rate).0;

        let a_freq = if self.mode == ComplexMode::FM {
            self.carrier_freq + b * self.carrier_freq * alpha * 2.0
        } else {
            self.carrier_freq * vibrato_coeff
        };

        let a_morph = if self.osc_c_mode == OscCMode::OscAWt {
            (self.carrier_morph + osc_c_value).clamp(0.0, 1.0)
        } else {
            self.carrier_morph
        };

        let (a, did_rollover) = self.carrier.step(a_freq, a_morph, self.sample_rate);

        if did_rollover {
            self.debug_cycle_flag = true;
        }

        if did_rollover && self.mod_sync == SyncMode::Sync {
            self.modulator.phase = 0.0;
        }

        let result = match self.mode {
            ComplexMode::Crossfade => (1.0 - alpha) * a + alpha * b,
            ComplexMode::And => match a > alpha / 2.0 && b > alpha / 2.0 {
                true => 1.0,
                false => -1.0,
            },
            ComplexMode::AM => ((1.0 - alpha) * a) + (alpha * a * b),
            ComplexMode::PWM => {
                if self.carrier.phase > alpha {
                    a
                } else {
                    b
                }
            }
            ComplexMode::FM => a,
        };

        let noise = random::<f32>();

        let noisy = result * (1.0 - (noise + self.last_noise_sample) / 2.0 * self.noise_level);

        self.last_noise_sample = noise;

        let filtered = self
            .lpf
            .process_sample(noisy, self.lowpass_freq, self.sample_rate);

        let with_sub = if self.osc_c_mode == OscCMode::Sub {
            filtered + osc_c_value
        } else {
            filtered
        };

        return with_sub;
    }

    fn parameters(&self) -> Vec<SoundParameter> {
        vec![
            SoundParameter {
                name: "Carrier freq",
                value: self.carrier_freq,
                param_type: ParamType::Float {
                    min: 55.0,
                    max: 880.0,
                },
            },
            SoundParameter {
                name: "Carrier table",
                value: self.carrier.table.into(),
                param_type: ParamType::Select(&[SIN_SAW, ODD_HARMONICS, SIN_SHAPED, SIN_FOLDED]),
            },
            SoundParameter {
                name: "Carrier morph",
                value: self.carrier_morph,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Mod freq ratio",
                value: self.mod_freq_ratio,
                param_type: ParamType::Float {
                    min: -7.0,
                    max: 7.0,
                },
            },
            SoundParameter {
                name: "Sync Mode",
                value: self.mod_sync.into(),
                param_type: ParamType::Select(&[FREE, QUANTIZE, SYNC]),
            },
            SoundParameter {
                name: "Mod table",
                value: self.modulator.table.into(),
                param_type: ParamType::Select(&[SIN_SAW, ODD_HARMONICS, SIN_SHAPED, SIN_FOLDED]),
            },
            SoundParameter {
                name: "Mod morph",
                value: self.mod_morph,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Blend Mode",
                value: self.mode.into(),
                param_type: ParamType::Select(&[CROSSFADE, AND, AM, PWM, FM]),
            },
            SoundParameter {
                name: "Alpha",
                value: self.alpha,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Osc C freq",
                value: self.osc_c_freq,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Osc C amp",
                value: self.osc_c_amp,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Osc C mode",
                value: self.osc_c_mode.into(),
                param_type: ParamType::Select(&[VIBRATO, FM, BLEND, OSC_A_WT, OSC_B_WT, SUB]),
            },
            SoundParameter {
                name: "Noise level",
                value: self.noise_level,
                param_type: ParamType::Float { min: 0.0, max: 1.0 },
            },
            SoundParameter {
                name: "Low pass filter",
                value: self.lowpass_freq,
                param_type: ParamType::Float {
                    min: 0.0,
                    max: 21000.0,
                },
            },
        ]
    }

    fn update_parameter(&mut self, name: &str, value: f32) {
        match name {
            "Carrier freq" => {
                self.carrier_freq = value;
            }
            "Carrier morph" => {
                self.carrier_morph = value;
            }
            "Mod freq ratio" => {
                self.mod_freq_ratio = value;
            }
            "Mod morph" => {
                self.mod_morph = value;
            }
            "Alpha" => {
                self.alpha = value;
            }
            "Osc C freq" => {
                self.osc_c_freq = value;
            }
            "Osc C amp" => {
                self.osc_c_amp = value;
            }
            "Noise level" => {
                self.noise_level = value;
            }
            "Blend Mode" => {
                self.mode = value.into();
            }
            "Sync Mode" => {
                self.mod_sync = value.into();
                self.sync_counter = 0;
            }
            "Mod table" => {
                self.modulator.table = value.into();
            }
            "Carrier table" => {
                self.carrier.table = value.into();
            }
            "Osc C mode" => {
                self.osc_c_mode = value.into();
                self.osc_c.phase = 0.0;
            }
            "Low pass filter" => {
                self.lowpass_freq = value;
            }
            _ => panic!("Unexpected parameter name: {}", name),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ComplexMode {
    Crossfade,
    And,
    AM,
    FM,
    PWM,
}

const CROSSFADE: &str = &"Crossfade";
const AND: &str = &"And";
const AM: &str = &"AM";
const PWM: &str = &"PWM";
const FM: &str = &"FM";

impl Into<f32> for ComplexMode {
    fn into(self) -> f32 {
        match self {
            ComplexMode::Crossfade => 0.0,
            ComplexMode::And => 1.0,
            ComplexMode::AM => 2.0,
            ComplexMode::PWM => 3.0,
            ComplexMode::FM => 4.0,
        }
    }
}

impl From<f32> for ComplexMode {
    fn from(value: f32) -> Self {
        if value == 0.0 {
            ComplexMode::Crossfade
        } else if value == 1.0 {
            ComplexMode::And
        } else if value == 2.0 {
            ComplexMode::AM
        } else if value == 3.0 {
            ComplexMode::PWM
        } else if value == 4.0 {
            ComplexMode::FM
        } else {
            panic!()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SyncMode {
    Free,
    Quantize,
    Sync,
}

const FREE: &str = &"Free";
const QUANTIZE: &str = &"Quantize";
const SYNC: &str = &"Sync";

impl Into<f32> for SyncMode {
    fn into(self) -> f32 {
        match self {
            SyncMode::Free => 0.0,
            SyncMode::Quantize => 1.0,
            SyncMode::Sync => 2.0,
        }
    }
}

impl From<f32> for SyncMode {
    fn from(value: f32) -> Self {
        if value == 0.0 {
            SyncMode::Free
        } else if value == 1.0 {
            SyncMode::Quantize
        } else if value == 2.0 {
            SyncMode::Sync
        } else {
            panic!()
        }
    }
}

#[derive(Clone, Copy)]
pub enum WaveTable {
    SinSaw,
    OddHarmonics,
    SinEnveloped,
    FoldedSin,
}

const SIN_SAW: &str = &"sin/saw";
const ODD_HARMONICS: &str = &"odd harmonics";
const SIN_SHAPED: &str = &"shaped sin";
const SIN_FOLDED: &str = &"folded sins";

impl Into<f32> for WaveTable {
    fn into(self) -> f32 {
        match self {
            WaveTable::SinSaw => 0.0,
            WaveTable::OddHarmonics => 1.0,
            WaveTable::SinEnveloped => 2.0,
            WaveTable::FoldedSin => 3.0,
        }
    }
}

impl From<f32> for WaveTable {
    fn from(value: f32) -> Self {
        if value == 0.0 {
            WaveTable::SinSaw
        } else if value == 1.0 {
            WaveTable::OddHarmonics
        } else if value == 2.0 {
            WaveTable::SinEnveloped
        } else if value == 3.0 {
            WaveTable::FoldedSin
        } else {
            panic!()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OscCMode {
    Vibrato,
    FM,
    Alpha,
    OscAWt,
    OscBWt,
    Sub,
}

impl Into<f32> for OscCMode {
    fn into(self) -> f32 {
        match self {
            OscCMode::Vibrato => 0.0,
            OscCMode::FM => 1.0,
            OscCMode::Alpha => 2.0,
            OscCMode::OscAWt => 3.0,
            OscCMode::OscBWt => 4.0,
            OscCMode::Sub => 5.0,
        }
    }
}

impl From<f32> for OscCMode {
    fn from(value: f32) -> Self {
        if value == 0.0 {
            OscCMode::Vibrato
        } else if value == 1.0 {
            OscCMode::FM
        } else if value == 2.0 {
            OscCMode::Alpha
        } else if value == 3.0 {
            OscCMode::OscAWt
        } else if value == 4.0 {
            OscCMode::OscBWt
        } else if value == 5.0 {
            OscCMode::Sub
        } else {
            panic!()
        }
    }
}

const VIBRATO: &str = &"Vibrato";
const BLEND: &str = &"Blend";
const OSC_A_WT: &str = &"Osc A WT";
const OSC_B_WT: &str = &"Osc B WT";
const SUB: &str = &"Sub";

struct Voice {
    phase: f32,
    current_wt_idx: f32,
    table: WaveTable,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            current_wt_idx: -1.0,
            table: WaveTable::SinSaw,
        }
    }

    pub fn step(&mut self, frequency: f32, morph: f32, sample_rate: u32) -> (f32, bool) {
        assert!(morph >= 0.0);
        assert!(morph <= 1.0);

        let mut did_rollover = false;
        if self.current_wt_idx == -1.0 {
            self.current_wt_idx = morph;
        }

        self.phase += frequency / sample_rate as f32;

        while self.phase >= 1.0 {
            self.phase -= 1.0;
            self.current_wt_idx = morph;
            did_rollover = true;
        }

        while self.phase < 0.0 {
            self.phase += 1.0;
        }

        let result = match self.table {
            WaveTable::SinSaw => self.sin_saw_addative_wavetable(morph),
            WaveTable::OddHarmonics => self.odd_harmonics_wavetable(morph),
            WaveTable::SinEnveloped => self.sin_enveloped(morph),
            WaveTable::FoldedSin => self.folded_sin(morph),
        };

        (result, did_rollover)
    }

    fn sin_saw_wavetable(&self, morph: f32) -> f32 {
        let a = 1.0 - morph;

        let x = if self.phase < a / 2.0 {
            self.phase / a
        } else {
            (self.phase + 1.0 - a) / (2.0 - a)
        };

        let cos = (x * TWO_PI).cos();
        let tri = if x < 0.5 {
            1.0 - 4.0 * x
        } else {
            4.0 * (x - 0.5) - 1.0
        };

        (1.0 - a) * tri + a * cos
    }

    fn sin_saw_addative_wavetable(&self, morph: f32) -> f32 {
        let x = self.phase * TWO_PI;
        let mut result = x.sin();

        for n in 2..=12 {
            result += (1.0 / n as f32) * morph * (n as f32 * x).sin();
        }

        result * (2.0 / std::f32::consts::PI)
    }

    fn odd_harmonics_wavetable(&self, morph: f32) -> f32 {
        const MAX_HARMONICS: f32 = 4.0;
        const MAX_VALUE: f32 = 3.0;
        let step_size = 1.0 / (MAX_HARMONICS - 1.0);
        let index_low = usize::min(
            MAX_HARMONICS as usize - 2,
            morph.div_euclid(step_size) as usize,
        );
        let index_high = index_low + 1;
        let frac = morph.rem_euclid(step_size) / step_size;

        let x = self.phase * TWO_PI;

        let mut result = 0.0;
        for i in 0..=index_low {
            result += ((2 * i + 1) as f32 * x).sin();
        }
        result += ((2 * index_high + 1) as f32 * x).sin() * frac * frac;
        result /= lerp(1.0..=MAX_VALUE, morph);

        result
    }

    fn sin_enveloped(&self, morph: f32) -> f32 {
        fn scaled_sin(x: f32) -> f32 {
            0.5 + f32::sin(TWO_PI * (x - 0.25)) / 2.0
        }

        let envelope = if self.phase < 0.25 {
            scaled_sin(self.phase * 2.0)
        } else if self.phase <= 0.75 {
            1.0
        } else {
            scaled_sin(self.phase * 2.0)
        };

        let x = self.phase - 0.5;

        f32::sin(TWO_PI * (morph * 9.0 + 1.0) * x) * envelope
    }

    fn folded_sin(&self, morph: f32) -> f32 {
        fn sigmoid(x: f32) -> f32 {
            let f = |x: f32| 2.0 * x - x.powi(2);
            if x < -1.0 {
                -1.0
            } else if x < 0.0 {
                -f(-x)
            } else if x < 1.0 {
                f(x)
            } else {
                1.0
            }
        }
        const MAX_FOLDS: u32 = 3;
        let mut result = f32::sin(self.phase * TWO_PI);
        result *= lerp(1.0..=10.0, morph);
        for _ in 0..MAX_FOLDS {
            if result > 1.0 {
                result = 1.0 - (result - 1.0);
            } else if result < -1.0 {
                result = -1.0 - (result + 1.0);
            } else {
                break;
            }
        }
        sigmoid(result)
    }
}

fn quantize(x: f32) -> f32 {
    let fractions = [2.0f32, 3.0];

    fractions
        .iter()
        .map(|n| (x * n).round() / n)
        .min_by(|a, b| (x - a).abs().partial_cmp(&(x - b).abs()).unwrap())
        .unwrap()
}
