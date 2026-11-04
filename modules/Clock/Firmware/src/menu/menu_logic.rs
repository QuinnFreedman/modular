use arduino_hal::port::PinOps;
use fm_lib::button_debouncer::{ButtonWithLongPress, LongPressButtonState};
use fm_lib::rotary_encoder::RotaryEncoderHandler;

use crate::clock::{ClockChannelConfig, ClockConfig};
use crate::eeprom::PersistanceManager;

use super::{
    menu_state::*,
    utils::{single_step_clock_division, step_clock_division},
};

#[inline(never)]
pub fn update_menu<BtnPin, const BTN_DEBOUNCE: u32, const BTN_LONG_PRESS: u32>(
    menu_or_ss_state: &mut MenuOrScreenSaverState,
    clock_state: &mut ClockConfig,
    button: &mut ButtonWithLongPress<BtnPin, BTN_DEBOUNCE, BTN_LONG_PRESS>,
    rotary_encoder: &RotaryEncoderHandler,
    current_time_ms: u32,
    did_rollover: bool,
    persistance_manager: &mut PersistanceManager,
) -> MenuUpdate
where
    BtnPin: PinOps,
{
    match menu_or_ss_state {
        MenuOrScreenSaverState::ScreenSaver(ref mut ss_state) => {
            let button_state = button.sample(current_time_ms);
            match button_state {
                LongPressButtonState::ButtonJustDown => {
                    *menu_or_ss_state =
                        MenuOrScreenSaverState::Menu(MenuState::new(current_time_ms));
                    return MenuUpdate::SwitchScreens;
                }
                _ => {}
            };

            let rotary_encoder_delta = rotary_encoder.sample_and_reset();
            if rotary_encoder_delta != 0 {
                *menu_or_ss_state = MenuOrScreenSaverState::Menu(MenuState::new(current_time_ms));
                return MenuUpdate::SwitchScreens;
            }

            if did_rollover {
                let (col, did_finish_screen) = {
                    const COL_MAX: u8 = 7;

                    let mut starting_col = ss_state.rng.next() % 16;
                    let mut one_col_has_space = false;
                    for i in 0..16 {
                        let idx = (starting_col + i) % 16;
                        if ss_state.y_offsets[idx as usize] < COL_MAX {
                            starting_col = idx;
                            one_col_has_space = true;
                            break;
                        }
                    }

                    if !one_col_has_space {
                        (starting_col, true)
                    } else {
                        let mut num_steps = (ss_state.rng.next() % 16) + 1;
                        let mut step = 0;
                        let mut idx: u8 = 0;
                        while num_steps > 0 {
                            idx = (starting_col + step) % 16;
                            step += 1;
                            if ss_state.y_offsets[idx as usize] < COL_MAX {
                                num_steps -= 1;
                            }
                        }
                        (idx % 16, false)
                    }
                };
                if did_finish_screen {
                    ss_state.y_offsets = [0; 16];
                    ss_state.color = !ss_state.color;
                }
                ss_state.y_offsets[col as usize] += 1;
                // if ss_state.y_offsets[col as usize] >= 8 {
                //     ss_state.y_offsets[col as usize] = 0;
                //     ss_state.colors ^= 1 << col;
                // }
                return MenuUpdate::ScreenSaverStep(col);
            }

            MenuUpdate::NoUpdate
        }
        MenuOrScreenSaverState::Menu(ref mut menu_state) => {
            let button_state = button.sample(current_time_ms);
            match button_state {
                LongPressButtonState::ButtonJustDown => {
                    menu_state.last_input_time_ms = current_time_ms;
                    return handle_short_press(menu_state, clock_state);
                }
                LongPressButtonState::ButtonJustClickedLong => {
                    menu_state.last_input_time_ms = current_time_ms;
                    return handle_long_press(menu_state, clock_state);
                }
                LongPressButtonState::ButtonIsUp => {}
                _ => {
                    menu_state.last_input_time_ms = current_time_ms;
                }
            }

            let rotary_encoder_delta = rotary_encoder.sample_and_reset();
            if rotary_encoder_delta != 0 {
                menu_state.last_input_time_ms = current_time_ms;
                return handle_rotary_knob_change(
                    menu_state,
                    clock_state,
                    rotary_encoder_delta,
                    persistance_manager,
                );
            }

            const SCREENSAVER_TIMEOUT_MS: u32 = 5000;
            if current_time_ms > menu_state.last_input_time_ms + SCREENSAVER_TIMEOUT_MS {
                persistance_manager.flush();
                *menu_or_ss_state =
                    MenuOrScreenSaverState::ScreenSaver(ScreenSaverState::new(current_time_ms));
                return MenuUpdate::SwitchScreens;
            }

            MenuUpdate::NoUpdate
        }
    }
}

fn handle_long_press(menu_state: &mut MenuState, _clock_state: &mut ClockConfig) -> MenuUpdate {
    match menu_state.page {
        MenuPage::Bpm => MenuUpdate::NoUpdate,
        MenuPage::Main { cursor } => {
            menu_state.page = MenuPage::SubMenu {
                channel: cursor,
                cursor: 0,
                scroll: 0,
            };
            menu_state.editing = EditingState::Navigating;
            MenuUpdate::SwitchScreens
        }
        MenuPage::SubMenu {
            channel,
            cursor: _,
            scroll: _,
        } => {
            menu_state.page = MenuPage::Main { cursor: channel };
            menu_state.editing = EditingState::Navigating;
            MenuUpdate::SwitchScreens
        }
    }
}

fn handle_short_press(menu_state: &mut MenuState, _clock_state: &mut ClockConfig) -> MenuUpdate {
    match menu_state.page {
        MenuPage::Bpm => {
            menu_state.editing = menu_state.editing.toggle();
            MenuUpdate::ToggleEditingAtCursor
        }
        MenuPage::Main { cursor: _ } => {
            menu_state.editing = menu_state.editing.toggle();
            MenuUpdate::ToggleEditingAtCursor
        }
        MenuPage::SubMenu {
            channel,
            cursor,
            scroll: _,
        } => match SubMenuItem::from(cursor) {
            SubMenuItem::Exit => {
                menu_state.page = MenuPage::Main { cursor: channel };
                MenuUpdate::SwitchScreens
            }
            _ => {
                menu_state.editing = menu_state.editing.toggle();
                MenuUpdate::ToggleEditingAtCursor
            }
        },
    }
}

pub const MIN_BPM: u8 = 30;
pub const MAX_BPM: u8 = 250;

fn handle_rotary_knob_change(
    menu_state: &mut MenuState,
    clock_state: &mut ClockConfig,
    rotary_encoder_delta: i8,
    persistance_manager: &mut PersistanceManager,
) -> MenuUpdate {
    match menu_state.page {
        MenuPage::Bpm => match menu_state.editing {
            EditingState::Editing => {
                clock_state.bpm = clock_state
                    .bpm
                    .saturating_add_signed(rotary_encoder_delta)
                    .clamp(MIN_BPM, MAX_BPM);
                persistance_manager.set_bpm(clock_state.bpm);
                MenuUpdate::UpdateValueAtCursor
            }
            EditingState::Navigating => {
                if rotary_encoder_delta > 0 {
                    menu_state.page = MenuPage::Main {
                        cursor: (rotary_encoder_delta as u8 - 1).min(7),
                    };
                    MenuUpdate::SwitchScreens
                } else {
                    MenuUpdate::NoUpdate
                }
            }
        },
        MenuPage::Main { ref mut cursor } => match menu_state.editing {
            EditingState::Navigating => {
                let old_cursor = *cursor;
                let new_cursor = (old_cursor as i8) + rotary_encoder_delta;

                if new_cursor < 0 {
                    menu_state.page = MenuPage::Bpm;
                    MenuUpdate::SwitchScreens
                } else {
                    *cursor = (new_cursor as u8).min(7);
                    MenuUpdate::MoveCursorFrom(old_cursor)
                }
            }
            EditingState::Editing => {
                let new_value = step_clock_division(
                    clock_state.channels[(*cursor) as usize].division,
                    rotary_encoder_delta,
                );
                clock_state.channels[(*cursor) as usize].division = new_value;
                persistance_manager.set_division(*cursor, new_value);
                MenuUpdate::UpdateValueAtCursor
            }
        },
        MenuPage::SubMenu {
            channel: channel_idx,
            ref mut cursor,
            ref mut scroll,
        } => {
            let channel: &mut ClockChannelConfig = &mut clock_state.channels[channel_idx as usize];
            match menu_state.editing {
                EditingState::Editing => match SubMenuItem::from(*cursor) {
                    SubMenuItem::Division => {
                        let old_division = channel.division;
                        channel.division =
                            single_step_clock_division(channel.division, rotary_encoder_delta);
                        persistance_manager.set_division(channel_idx, channel.division);
                        if channel.division != old_division
                            && (old_division == -65 || channel.division == -65)
                        {
                            MenuUpdate::SwitchScreens
                        } else {
                            MenuUpdate::UpdateValueAtCursor
                        }
                    }
                    SubMenuItem::PulseWidth => {
                        channel.pulse_width = channel
                            .pulse_width
                            .saturating_add_signed(rotary_encoder_delta)
                            .clamp(0, 100);
                        persistance_manager.set_pulse_width(channel_idx, channel.pulse_width);
                        MenuUpdate::UpdateValueAtCursor
                    }
                    SubMenuItem::PhaseShift => {
                        channel.phase_shift = channel
                            .phase_shift
                            .saturating_add(rotary_encoder_delta)
                            .clamp(-32, 32);
                        persistance_manager.set_phase_shift(channel_idx, channel.phase_shift);
                        MenuUpdate::UpdateValueAtCursor
                    }
                    SubMenuItem::Swing => {
                        channel.swing = channel
                            .swing
                            .saturating_add_signed(rotary_encoder_delta)
                            .min(32);
                        persistance_manager.set_swing(channel_idx, channel.swing);
                        MenuUpdate::UpdateValueAtCursor
                    }
                    SubMenuItem::Probability => {
                        channel.probability = channel
                            .probability
                            .saturating_add_signed(rotary_encoder_delta)
                            .clamp(0,100);
                        persistance_manager.set_probability(channel_idx, channel.probability);
                        MenuUpdate::UpdateValueAtCursor
                    }
                    SubMenuItem::Exit => MenuUpdate::NoUpdate,
                },
                EditingState::Navigating => {
                    let old_cursor = *cursor;
                    if channel.division == -65 {
                        if rotary_encoder_delta < 0 {
                            *cursor = 0;
                        } else if rotary_encoder_delta > 0 {
                            *cursor = 5;
                        }
                        if old_cursor == *cursor {
                            MenuUpdate::NoUpdate
                        } else {
                            MenuUpdate::MoveCursorFrom(old_cursor)
                        }
                    } else {
                        *cursor = cursor.saturating_add_signed(rotary_encoder_delta).min(5);
                        if old_cursor == *cursor {
                            MenuUpdate::NoUpdate
                        } else if *cursor < *scroll {
                            *scroll = *cursor;
                            MenuUpdate::Scroll(ScrollDirection::Up)
                        } else if *cursor > *scroll + 1 {
                            *scroll = *cursor - 1;
                            MenuUpdate::Scroll(ScrollDirection::Down)
                        } else {
                            MenuUpdate::MoveCursorFrom(old_cursor)
                        }
                    }
                }
            }
        }
    }
}
