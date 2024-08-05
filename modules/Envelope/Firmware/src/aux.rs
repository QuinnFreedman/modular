use crate::envelope::{AcrcLoopState, AcrcState, AdsrState, AhrdState, EnvelopeMode};

pub enum AuxMode {
    EndOfRise,
    EndOfFall,
    NonZero,
    FollowGate,
}

pub fn update_aux(env_mode: EnvelopeMode, config: AuxMode) -> bool {
    match config {
        AuxMode::EndOfRise => match env_mode {
            EnvelopeMode::Adsr(phase) => match phase {
                AdsrState::Wait => false,
                AdsrState::Attack => false,
                AdsrState::Decay => true,
                AdsrState::Sustain => true,
                AdsrState::Release => true,
            },
            EnvelopeMode::Acrc(phase) => match phase {
                AcrcState::Wait => false,
                AcrcState::Attack => false,
                AcrcState::Hold => true,
                AcrcState::Release => true,
            },
            EnvelopeMode::AcrcLoop(phase) => match phase {
                AcrcLoopState::Attack => false,
                AcrcLoopState::Release => true,
            },
            EnvelopeMode::AhrdLoop(phase) => match phase {
                AhrdState::Attack => false,
                AhrdState::Hold => true,
                AhrdState::Release => true,
                AhrdState::Delay => false,
            },
        },
        AuxMode::EndOfFall => match env_mode {
            EnvelopeMode::Adsr(phase) => match phase {
                AdsrState::Wait => true,
                AdsrState::Attack => false,
                AdsrState::Decay => false,
                AdsrState::Sustain => false,
                AdsrState::Release => false,
            },
            EnvelopeMode::Acrc(phase) => match phase {
                AcrcState::Wait => true,
                AcrcState::Attack => false,
                AcrcState::Hold => false,
                AcrcState::Release => false,
            },
            EnvelopeMode::AcrcLoop(phase) => match phase {
                AcrcLoopState::Attack => true,
                AcrcLoopState::Release => false,
            },
            EnvelopeMode::AhrdLoop(phase) => match phase {
                AhrdState::Attack => false,
                AhrdState::Hold => false,
                AhrdState::Release => false,
                AhrdState::Delay => true,
            },
        },
        AuxMode::NonZero => match env_mode {
            EnvelopeMode::Adsr(phase) => match phase {
                AdsrState::Wait => false,
                AdsrState::Attack => true,
                AdsrState::Decay => true,
                AdsrState::Sustain => true,
                AdsrState::Release => true,
            },
            EnvelopeMode::Acrc(phase) => match phase {
                AcrcState::Wait => false,
                AcrcState::Attack => true,
                AcrcState::Hold => true,
                AcrcState::Release => true,
            },
            EnvelopeMode::AcrcLoop(phase) => match phase {
                AcrcLoopState::Attack => true,
                AcrcLoopState::Release => true,
            },
            EnvelopeMode::AhrdLoop(phase) => match phase {
                AhrdState::Attack => true,
                AhrdState::Hold => true,
                AhrdState::Release => true,
                AhrdState::Delay => false,
            },
        },
        AuxMode::FollowGate => match env_mode {
            EnvelopeMode::Adsr(phase) => match phase {
                AdsrState::Wait => false,
                AdsrState::Attack => true,
                AdsrState::Decay => true,
                AdsrState::Sustain => true,
                AdsrState::Release => false,
            },
            EnvelopeMode::Acrc(phase) => match phase {
                AcrcState::Wait => false,
                AcrcState::Attack => true,
                AcrcState::Hold => true,
                AcrcState::Release => false,
            },
            EnvelopeMode::AcrcLoop(_) => false,
            EnvelopeMode::AhrdLoop(_) => false,
        },
    }
}
