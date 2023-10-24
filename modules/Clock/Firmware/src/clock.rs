// TOOD maybe have candidate value in menu state for actively edited field so
// changes aren't applied until commit
pub struct ClockChannelConfig {
    pub division: i8,
    pub swing: i8,
    pub pulse_width: u8,
    pub phase_shift: i8,
}

pub struct ClockConfig {
    pub channels: [ClockChannelConfig; 8],
    pub bpm: u8,
}

impl ClockConfig {
    pub fn new() -> Self {
        // Maybe move this to PROGMEM if the loop isn't unrolled
        const DEFAULT_DIVISIONS: [i8; 8] = [1, 2, 4, 8, -2, -4, -8, -16];
        ClockConfig {
            bpm: 128,
            channels: DEFAULT_DIVISIONS.map(|i| ClockChannelConfig {
                division: i,
                swing: 50,
                pulse_width: 0,
                phase_shift: 0,
            }),
        }
    }
}

pub struct ClockState {
    last_cycle_start_time: u32,
    cycle_count: u8,
}

impl ClockState {
    pub fn new() -> Self {
        Self {
            last_cycle_start_time: 0,
            cycle_count: 0,
        }
    }
}

const NUM_CHANNELS: u8 = 8;
const MS_PER_MINUTE: u32 = 1000 * 60;

#[inline(never)]
pub fn sample(config: &ClockConfig, state: &mut ClockState, current_time_ms: u32) -> u8 {
    let mut time_in_current_cycle = current_time_ms - state.last_cycle_start_time;
    // TODO: maybe convert to micros here for better accuracy if it never overflows at min BPM
    let ms_per_cycle = MS_PER_MINUTE / config.bpm as u32;
    if time_in_current_cycle > ms_per_cycle {
        time_in_current_cycle -= ms_per_cycle;
        state.last_cycle_start_time += ms_per_cycle;
        state.cycle_count = (state.cycle_count + 1) % 128;
    }

    let mut result: u8 = 0;
    for i in 0..NUM_CHANNELS {
        let is_on = channel_is_on(
            &config.channels[i as usize],
            time_in_current_cycle,
            ms_per_cycle,
            state.cycle_count,
        );
        result |= (is_on as u8) << i;
    }
    result
}

fn channel_is_on(
    channel: &ClockChannelConfig,
    time_in_current_cycle: u32,
    ms_per_cycle: u32,
    cycle_count: u8,
) -> bool {
    let percent_complete = (time_in_current_cycle * 100) / ms_per_cycle;
    // TODO implement channel config options
    percent_complete > 50
}
