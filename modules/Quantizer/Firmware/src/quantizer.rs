pub struct QuantizerState {
    pub notes: [bool; 12],
    pub sample_mode: SampleMode,
    pub glide_amount: u8,
    pub trigger_delay_amount: u8,
    pub pre_shift: i8,
    pub scale_shift: i8,
    pub post_shift: i8,
    pub channel_b_mode: PitchMode,
    pub channels_linked: bool,
}

pub enum PitchMode {
    Relative,
    Absolute,
}

pub enum SampleMode {
    TrackAndHold,
    SampleAndHold,
}

impl QuantizerState {
    pub fn new() -> Self {
        Self {
            notes: [false; 12],
            sample_mode: SampleMode::TrackAndHold,
            glide_amount: 0,
            trigger_delay_amount: 0,
            pre_shift: 0,
            scale_shift: 0,
            post_shift: 0,
            channel_b_mode: PitchMode::Absolute,
            channels_linked: false,
        }
    }
}
