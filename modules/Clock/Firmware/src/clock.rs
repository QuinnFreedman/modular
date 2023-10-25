// TOOD maybe have candidate value in menu state for actively edited field so
// changes aren't applied until commit
pub struct ClockChannelConfig {
    pub division: i8,
    pub swing: u8,
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
                swing: 0,
                pulse_width: 50,
                phase_shift: 0,
            }),
        }
    }
}

pub struct ClockState {
    last_cycle_start_time: u32,
    cycle_count: u32,
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
    ms_into_current_core_cycle: u32,
    ms_per_core_cycle: u32,
    core_cycle_count: u32,
) -> bool {
    let (ms_per_channel_period, mut ms_into_current_channel_period, mut channel_period_count) =
        if channel.division <= 1 {
            let core_cycles_per_period = channel.division.abs() as u32;
            (
                core_cycles_per_period * ms_per_core_cycle,
                (core_cycle_count % core_cycles_per_period) * ms_per_core_cycle
                    + ms_into_current_core_cycle,
                core_cycle_count / core_cycles_per_period,
            )
        } else {
            // TODO
            // (ms_into_current_core_cycle * 100) / ms_per_core_cycle < 50
            return false;
        };

    // handle phase shift with wrap around
    // this could be much simpler but we have to keep channel_period_count updated to implement swing
    let phase_shift_percent = -channel.phase_shift;
    if phase_shift_percent < 0 {
        let phase_shift_ms = ms_per_channel_period * (-phase_shift_percent as u32) / 100;
        if ms_into_current_channel_period >= phase_shift_ms {
            ms_into_current_channel_period -= phase_shift_ms;
        } else {
            channel_period_count = channel_period_count.saturating_sub(1);
            ms_into_current_channel_period =
                ms_per_channel_period + ms_into_current_channel_period - phase_shift_ms;
        }
    } else if phase_shift_percent > 0 {
        let phase_shift_ms = ms_per_channel_period * (phase_shift_percent as u32) / 100;
        ms_into_current_channel_period += phase_shift_ms;
        if ms_into_current_channel_period > ms_per_channel_period {
            ms_into_current_channel_period -= ms_per_channel_period;
            channel_period_count += 1;
        }
    }

    const TRIG_WIDTH_MS: u32 = 10;
    let max_pw_ms = ms_per_channel_period - TRIG_WIDTH_MS;
    let pulse_width_ms = if channel.pulse_width == 0 {
        TRIG_WIDTH_MS
    } else if channel.pulse_width == 100 {
        max_pw_ms
    } else {
        (ms_per_channel_period * channel.pulse_width as u32 / 100).clamp(TRIG_WIDTH_MS, max_pw_ms)
    };

    if channel_period_count % 2 == 0 {
        ms_into_current_channel_period < pulse_width_ms
    } else {
        let swing_ms = ms_per_channel_period * channel.swing as u32 / 100;
        ms_into_current_channel_period > swing_ms
            && ms_into_current_channel_period < swing_ms + pulse_width_ms
            && ms_into_current_channel_period < max_pw_ms
    }
}
