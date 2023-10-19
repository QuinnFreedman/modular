// TOOD maybe have candidate value in menu state for actively edited field so
// changes aren't applied until commit
pub struct ClockChannelConfig {
    pub division: i8,
    pub swing: i8,
    pub pulse_width: i8,
    pub phase_shift: i8,
}

pub struct ClockConfig {
    pub channels: [ClockChannelConfig; 8],
    pub bpm: u8,
}

impl ClockConfig {
    pub fn new() -> Self {
        // Maybe move this to PROGMEM
        const DEFAULT_DIVISIONS: [i8; 8] = [1, 2, 4, 8, -2, -4, -8, -16];
        ClockConfig {
            bpm: 128,
            channels: DEFAULT_DIVISIONS.map(|i| ClockChannelConfig {
                division: i,
                swing: 0,
                pulse_width: 0,
                phase_shift: 0,
            }),
        }
    }
}

// NOTE: keep track of ms_into_period. When > MS_PER_PERIOD, subtract and carry over.
// when MS_PER_PERIOD is changed, it's not a problem, cary over might just be more
// (what happens if multiple carry overs? -- probably ignore; rare event and fine)
