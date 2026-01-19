use crate::random::Rng;

// TODO maybe have candidate value in menu state for actively edited field so
// changes aren't applied until commit
#[repr(C)]
pub struct ClockChannelConfig {
    pub division: i8,
    pub swing: u8,
    pub pulse_width: u8,
    pub phase_shift: i8,
    pub probability: u8,
}

#[repr(C)]
pub struct ClockConfig {
    pub channels: [ClockChannelConfig; 8],
    pub bpm: u8,
    // This isn't used right now, but reserving the space for a future feature
    pub _is_follower: bool,
    pub invert_encoder: bool,
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
                probability: 100,
            }),
            _is_follower: false,
            invert_encoder: false,
        }
    }
}

#[repr(packed)]
pub struct ClockChannelState {
    pub is_on: bool,
    pub enable_output: bool
}

#[repr(packed)]
pub struct ClockState {
    pub channels: [ClockChannelState; 8],
    last_cycle_start_time: u64,
    cycle_count: u32,
    rng: Rng,
}

impl ClockState {
    pub fn new() -> Self {
        let rng = Rng::new(12345);
        
        Self {
            channels: [false; 8].map(|i| ClockChannelState {
                is_on: i,
                enable_output: i
            }),
            last_cycle_start_time: 0,
            cycle_count: 0,
            rng,
        }
    }

    pub fn reset(&mut self) {
        self.last_cycle_start_time = 0;
        self.cycle_count = 0;
    }
}

const NUM_CHANNELS: u8 = 8;
const MICROS_PER_MINUTE: u32 = 1000 * 1000 * 60;

/**
This is the main logic loop for the actual clock iteslf. It takes in the current time
and configs, and calculates whether each clock channel should currently be HIGH or LOW.

Returns a tuple of u8 and bool. The u8 is a bit vector representing the state of the
eight clock channels. The bool indicates whether or not the core clock rolled over
at this sample point. This is used to render the screensaver.
*/
#[inline(never)]
pub fn sample(
    config: &ClockConfig,
    state: &mut ClockState,
    current_time_micros: u64,
    is_paused: bool,
) -> (u8, bool) {
    if is_paused {
        let mut result: u8 = 0;
        for i in 0..NUM_CHANNELS {
            let channel = &config.channels[i as usize];
            let is_on = channel.division == -65;
            result |= (is_on as u8) << i;
        }
        return (result, false);
    }
    let mut did_rollover = false;
    let mut micros_in_current_cycle = (current_time_micros - state.last_cycle_start_time) as u32;
    let micros_per_cycle = MICROS_PER_MINUTE / config.bpm as u32;
    if micros_in_current_cycle > micros_per_cycle {
        micros_in_current_cycle -= micros_per_cycle;
        state.last_cycle_start_time += micros_per_cycle as u64;
        state.cycle_count += 1;
        did_rollover = true;
    }

    let mut result: u8 = 0;
    for i in 0..NUM_CHANNELS {
        let channel = &config.channels[i as usize];
        let channel_state = &mut state.channels[i as usize];
        // if the tempo is too slow, micros counts will overflow a u32 at some points in
        // the math, so fall back to lower temporal resolution
        let (time_in_current_cycle, time_per_cycle) = if channel.division < -32 && config.bpm < 50 {
            (micros_in_current_cycle / 10, micros_per_cycle / 10)
        } else {
            (micros_in_current_cycle, micros_per_cycle)
        };
        const TRIG_WIDTH_MICROS: u32 = 5000; // 5ms is the minimum pulse width

        let is_on = channel_is_on(
            channel,
            time_in_current_cycle,
            time_per_cycle,
            state.cycle_count,
            TRIG_WIDTH_MICROS,
        );

        if is_on && !channel_state.is_on {
            let n = state.rng.next();

            channel_state.enable_output = (channel.probability == 100) || (n < ((channel.probability as f32 / 100.0) * 255.0) as u8) ;
        }

        channel_state.is_on = is_on;

        result |= ((channel_state.is_on && channel_state.enable_output) as u8) << i;
    }
    (result, did_rollover)
}

/**
Determine if a given clock channel should be in its HIGH or LOW state based on its
pulse width, phase shift, swing, and the current master clock time

Although the time units are all written as ms in this function, they can be scaled
up or down to any resolution by just multiplying the input time in current cycle and
time per cycle by a constant factor (i.e. to work in us precision)

In this function "core cycle" refers to the amount of time for the master clock to
loop, while "channel period" refers to the amount of time for the individual channel
to loop.
 */
fn channel_is_on(
    channel: &ClockChannelConfig,
    ms_into_current_core_cycle: u32,
    ms_per_core_cycle: u32,
    core_cycle_count: u32,
    min_trig_width_ms: u32,
) -> bool {
    // NOTE: maybe it would be more accurate to use f32 seconds for time calculations
    // instead of effectively fixed-point (u32 us). float math is probably slower but
    // there is some slight aliasing at high tempo. Alternatively, could supersample
    // to an even higher resolution (ns, 100xus) at higher BPM. If different resolution
    // is used for different channels it can cause some desync. The core clock BPM
    // range is only 1 OoM, so maybe not a huge win there.

    // convert from core clock time to chanel period time (calculated differently
    // depending on if channel is a multiple or division)
    let (ms_per_channel_period, mut ms_into_current_channel_period, mut is_even_channel_period) =
        if channel.division == -65 {
            return false;
        } else if channel.division <= 1 {
            let core_cycles_per_period = channel.division.abs() as u32;
            (
                core_cycles_per_period * ms_per_core_cycle,
                (core_cycle_count % core_cycles_per_period) * ms_per_core_cycle
                    + ms_into_current_core_cycle,
                (core_cycle_count / core_cycles_per_period) % 2 == 0,
            )
        } else {
            let periods_per_core_cycle = channel.division as u32;
            let ms_per_channel_period = (ms_per_core_cycle / periods_per_core_cycle).max(1);
            let is_even_period = {
                let was_even_last_cycle = (core_cycle_count * periods_per_core_cycle) % 2 == 0;
                let periods_this_cycle = ms_into_current_core_cycle / ms_per_channel_period;
                let even_in_current_cycle = periods_this_cycle % 2 == 0;
                was_even_last_cycle == even_in_current_cycle
            };
            let ms_into_current_channel_period = ms_into_current_core_cycle % ms_per_channel_period;
            (
                ms_per_channel_period,
                ms_into_current_channel_period,
                is_even_period,
            )
        };

    // handle phase shift with wrap around
    // this could be a simple signed modulus addition, but we have to keep
    // is_even_channel_period updated to implement swing, which adds complication
    let phase_shift_fraction = -channel.phase_shift;
    if phase_shift_fraction < 0 {
        let phase_shift_ms = ms_per_channel_period * (-phase_shift_fraction as u32) / 64;
        if ms_into_current_channel_period >= phase_shift_ms {
            ms_into_current_channel_period -= phase_shift_ms;
        } else {
            is_even_channel_period = !is_even_channel_period;
            ms_into_current_channel_period =
                ms_per_channel_period + ms_into_current_channel_period - phase_shift_ms;
        }
    } else if phase_shift_fraction > 0 {
        let phase_shift_ms = ms_per_channel_period * (phase_shift_fraction as u32) / 64;
        ms_into_current_channel_period += phase_shift_ms;
        if ms_into_current_channel_period > ms_per_channel_period {
            ms_into_current_channel_period -= ms_per_channel_period;
            is_even_channel_period = !is_even_channel_period;
        }
    }

    // calculate pulse width, taking into account min trigger lengths
    let max_pw_ms = ms_per_channel_period.saturating_sub(min_trig_width_ms);

    let pulse_width_ms = if min_trig_width_ms >= max_pw_ms {
        // If period gets very small, ignore pulse width
        ms_per_channel_period / 2
    } else if channel.pulse_width == 0 {
        min_trig_width_ms
    } else if channel.pulse_width == 100 {
        max_pw_ms
    } else {
        (ms_per_channel_period * channel.pulse_width as u32 / 100)
            .clamp(min_trig_width_ms, max_pw_ms)
    };

    if is_even_channel_period {
        // normal (even) output
        ms_into_current_channel_period < pulse_width_ms
    } else {
        // handle swing (odd cycles)
        let swing_ms = ms_per_channel_period * channel.swing as u32 / 64;
        ms_into_current_channel_period > swing_ms
            && ms_into_current_channel_period < swing_ms + pulse_width_ms
            && ms_into_current_channel_period < max_pw_ms
    }
}
