use core::arch::asm;
use core::panic;

use arduino_hal::Eeprom;
use fm_lib::button_debouncer::{ButtonState, LongPressButtonState};

use crate::{
    bitvec::BitVec,
    persistence::{
        check_save_slots, erase_all_save_slots, read_config, read_scale, write_config, write_scale,
    },
    quantizer::{PitchMode, QuantizationResult, QuantizerChannel, QuantizerState, SampleMode},
    resistor_ladder_buttons::ButtonEvent,
};

#[derive(Clone, Copy)]
pub enum LedColor {
    GREEN,
    RED,
    AMBER,
    OFF,
}

pub enum Channel {
    A,
    B,
}

impl Into<usize> for &Channel {
    fn into(self) -> usize {
        match self {
            Channel::A => 0,
            Channel::B => 1,
        }
    }
}

impl Channel {
    pub fn index(&self) -> usize {
        Into::<usize>::into(self)
    }
}

enum ScalarSubMenuStatus {
    AwaitingFirstInput,
    ExitOnShiftRelease,
    ExitOnButtonRelease,
}

enum ScalarSubMenu {
    Glide,
    Delay,
    PreShift,
    ScaleShift,
    PostShift,
}

enum BoolOption {
    TrackAndHold,
    RelativePitch,
    ChannelsLinked,
}

enum MenuPage {
    MainMenu,
    ScalarSubMenu(ScalarSubMenuStatus, ScalarSubMenu),
    ShowChangedBoolOption(BoolOption),
    SelectSaveSlot(SaveSlotType),
    SelectLoadSlot(SaveSlotType),
    ConfirmSaveSlot(u8, SaveSlotType, u32),
    ConfirmErase(u32),
}

#[derive(Clone, Copy)]
enum SaveSlotType {
    Scale,
    FullConfig,
}

pub struct MenuState {
    selected_channel: Channel,
    menu_page: MenuPage,
    shift_was_pressed: bool,
    scale_save_slots_in_use: BitVec<12>,
    config_save_slots_in_use: BitVec<12>,
}

impl MenuState {
    pub fn new(eeprom: &mut Eeprom) -> Self {
        let (scale_save_slots_in_use, config_save_slots_in_use) = check_save_slots(eeprom);
        Self {
            selected_channel: Channel::A,
            menu_page: MenuPage::MainMenu,
            shift_was_pressed: false,
            scale_save_slots_in_use,
            config_save_slots_in_use,
        }
    }

    fn handle_shift_button_press(
        &mut self,
        quantizer_state: &mut QuantizerState,
        button_index: u8,
    ) {
        let active_channel = &mut quantizer_state.channels[self.selected_channel.index()];
        match button_index {
            0 => {
                active_channel.config.notes.rotate_left(1);
            }
            1 => {
                active_channel.config.notes.rotate_right(1);
            }
            2 => {
                self.menu_page = MenuPage::ScalarSubMenu(
                    ScalarSubMenuStatus::AwaitingFirstInput,
                    ScalarSubMenu::Glide,
                )
            }
            3 => {
                self.menu_page = MenuPage::ScalarSubMenu(
                    ScalarSubMenuStatus::AwaitingFirstInput,
                    ScalarSubMenu::Delay,
                )
            }
            4 => {
                active_channel.config.sample_mode = match active_channel.config.sample_mode {
                    SampleMode::TrackAndHold => SampleMode::SampleAndHold,
                    SampleMode::SampleAndHold => SampleMode::TrackAndHold,
                };
                self.menu_page = MenuPage::ShowChangedBoolOption(BoolOption::TrackAndHold);
            }
            5 => {
                self.menu_page = MenuPage::ScalarSubMenu(
                    ScalarSubMenuStatus::AwaitingFirstInput,
                    ScalarSubMenu::PostShift,
                )
            }
            6 => {
                self.menu_page = MenuPage::ScalarSubMenu(
                    ScalarSubMenuStatus::AwaitingFirstInput,
                    ScalarSubMenu::ScaleShift,
                )
            }
            7 => {
                self.menu_page = MenuPage::ScalarSubMenu(
                    ScalarSubMenuStatus::AwaitingFirstInput,
                    ScalarSubMenu::PreShift,
                )
            }
            8 => {
                quantizer_state.channel_b_mode = match quantizer_state.channel_b_mode {
                    PitchMode::Relative => PitchMode::Absolute,
                    PitchMode::Absolute => PitchMode::Relative,
                };
                self.menu_page = MenuPage::ShowChangedBoolOption(BoolOption::RelativePitch);
            }
            9 => {
                quantizer_state.channels_linked = !quantizer_state.channels_linked;
                self.menu_page = MenuPage::ShowChangedBoolOption(BoolOption::ChannelsLinked);
            }
            10 => {
                self.selected_channel = Channel::A;
            }
            11 => {
                self.selected_channel = Channel::B;
            }
            _ => panic!(),
        }
    }

    pub fn handle_button_input_and_render_display(
        &mut self,
        quantizer_state: &mut QuantizerState,
        buttons: &ButtonInput,
        active_notes: &QuantizationResult,
        current_time_ms: u32,
        eeprom: &mut Eeprom,
    ) -> [LedColor; 12] {
        if self.shift_was_pressed && !buttons.shift_pressed {
            if let MenuPage::ScalarSubMenu(ref mut menu_status, _) = self.menu_page {
                match menu_status {
                    ScalarSubMenuStatus::AwaitingFirstInput => {
                        *menu_status = ScalarSubMenuStatus::ExitOnShiftRelease
                    }
                    ScalarSubMenuStatus::ExitOnShiftRelease => self.menu_page = MenuPage::MainMenu,
                    ScalarSubMenuStatus::ExitOnButtonRelease => {}
                }
            }
        }
        self.shift_was_pressed = buttons.shift_pressed;

        if buttons.save_button == LongPressButtonState::ButtonJustDown {
            match self.menu_page {
                MenuPage::SelectSaveSlot(_) | MenuPage::SelectLoadSlot(_) => {
                    self.menu_page = MenuPage::MainMenu
                }
                MenuPage::ConfirmSaveSlot(_, _, _) => {}
                _ => {
                    self.menu_page = MenuPage::SelectSaveSlot(if self.shift_was_pressed {
                        SaveSlotType::FullConfig
                    } else {
                        SaveSlotType::Scale
                    })
                }
            };
        } else if buttons.load_button == LongPressButtonState::ButtonJustDown {
            match self.menu_page {
                MenuPage::SelectSaveSlot(_) | MenuPage::SelectLoadSlot(_) => {
                    self.menu_page = MenuPage::MainMenu;
                }
                MenuPage::ConfirmSaveSlot(_, _, _) => {}
                _ => {
                    self.menu_page = MenuPage::SelectLoadSlot(if self.shift_was_pressed {
                        SaveSlotType::FullConfig
                    } else {
                        SaveSlotType::Scale
                    });
                }
            };
        } else if (buttons.load_button == LongPressButtonState::ButtonHeldDownLong
            && buttons.save_button == LongPressButtonState::ButtonJustClickedLong)
            || (buttons.load_button == LongPressButtonState::ButtonJustClickedLong
                && buttons.save_button == LongPressButtonState::ButtonHeldDownLong)
        {
            self.menu_page = MenuPage::ConfirmErase(current_time_ms);
            self.scale_save_slots_in_use = BitVec::new();
            self.config_save_slots_in_use = BitVec::new();
            erase_all_save_slots(eeprom);
        }

        match buttons.key_event {
            ButtonEvent::ButtonJustPressed(n) => match self.menu_page {
                MenuPage::ScalarSubMenu(ref mut status, ref menu) => {
                    let selected_channel =
                        &mut quantizer_state.channels[self.selected_channel.index()];
                    match handle_sub_menu_button_press(
                        selected_channel,
                        status,
                        menu,
                        n,
                        buttons.shift_pressed,
                    ) {
                        Some(new_menu_status) => *status = new_menu_status,
                        None => self.menu_page = MenuPage::MainMenu,
                    }
                }
                MenuPage::MainMenu => {
                    if buttons.shift_pressed {
                        self.handle_shift_button_press(quantizer_state, n);
                    } else {
                        let selected_channel =
                            &mut quantizer_state.channels[self.selected_channel.index()];
                        selected_channel.config.notes[n as usize] =
                            !selected_channel.config.notes[n as usize];
                    }
                }
                MenuPage::ShowChangedBoolOption(_) => {}
                MenuPage::SelectSaveSlot(slot_type) => {
                    match slot_type {
                        SaveSlotType::Scale => {
                            write_scale(eeprom, n, quantizer_state, &self.selected_channel);
                            self.scale_save_slots_in_use.set(n, true);
                        }
                        SaveSlotType::FullConfig => {
                            write_config(eeprom, n, quantizer_state);
                            self.config_save_slots_in_use.set(n, true);
                        }
                    }
                    self.menu_page = MenuPage::ConfirmSaveSlot(n, slot_type, current_time_ms);
                }
                MenuPage::SelectLoadSlot(slot_type) => {
                    match slot_type {
                        SaveSlotType::Scale => {
                            read_scale(eeprom, n, quantizer_state, &self.selected_channel);
                        }
                        SaveSlotType::FullConfig => {
                            read_config(eeprom, n, quantizer_state);
                        }
                    }
                    self.menu_page = MenuPage::MainMenu;
                }
                MenuPage::ConfirmSaveSlot(_, _, _) => {}
                MenuPage::ConfirmErase(_) => {}
            },
            ButtonEvent::ButtonJustReleased => match self.menu_page {
                MenuPage::ScalarSubMenu(ScalarSubMenuStatus::ExitOnButtonRelease, _) => {
                    self.menu_page = MenuPage::MainMenu;
                }
                MenuPage::ShowChangedBoolOption(_) => {
                    self.menu_page = MenuPage::MainMenu;
                }
                _ => {}
            },
            _ => {}
        }

        match self.menu_page {
            MenuPage::MainMenu => self.render_notes_display(quantizer_state, active_notes),
            MenuPage::ScalarSubMenu(_, ref menu) => {
                let selected_channel = &quantizer_state.channels[self.selected_channel.index()];
                render_sub_menu(menu, &selected_channel)
            }
            MenuPage::ShowChangedBoolOption(ref option) => {
                render_bool_option(&quantizer_state, &self.selected_channel, option)
            }
            MenuPage::SelectSaveSlot(ref slot_type) | MenuPage::SelectLoadSlot(ref slot_type) => {
                let slots = match slot_type {
                    SaveSlotType::Scale => &self.scale_save_slots_in_use,
                    SaveSlotType::FullConfig => &self.config_save_slots_in_use,
                };
                render_save_menu(&self.selected_channel, slot_type, slots)
            }
            MenuPage::ConfirmSaveSlot(slot, slot_type, start) => {
                let time = current_time_ms - start;
                if time >= 1024 {
                    self.menu_page = MenuPage::MainMenu
                }
                render_confirm_save(&self.selected_channel, &slot_type, &time, slot)
            }
            MenuPage::ConfirmErase(start) => {
                let time = current_time_ms - start;
                if time >= 832 {
                    self.menu_page = MenuPage::MainMenu
                }
                render_confirm_erse(&time)
            }
        }
    }

    fn render_notes_display(
        &self,
        quantizer_state: &QuantizerState,
        active_notes: &QuantizationResult,
    ) -> [LedColor; 12] {
        let selected_channel = &quantizer_state.channels[self.selected_channel.index()];
        let color = match self.selected_channel {
            Channel::A => LedColor::GREEN,
            Channel::B => LedColor::RED,
        };
        let mut leds = [LedColor::OFF; 12];
        for i in 0..12 {
            if selected_channel.config.notes[i] {
                leds[i] = color;
            }
        }

        match self.selected_channel {
            Channel::A => {
                leds[(active_notes.channel_a.nominal_semitones as usize) % 12] = LedColor::AMBER
            }
            Channel::B => {
                leds[(active_notes.channel_b.nominal_semitones as usize) % 12] = LedColor::AMBER
            }
        }

        leds
    }
}

fn render_bool_option(
    quantizer_state: &QuantizerState,
    selected_channel: &Channel,
    option: &BoolOption,
) -> [LedColor; 12] {
    let mut leds = [LedColor::OFF; 12];
    match option {
        BoolOption::TrackAndHold => {
            let channel = &quantizer_state.channels[selected_channel.index()];
            leds[4] = match channel.config.sample_mode {
                SampleMode::TrackAndHold => LedColor::GREEN,
                SampleMode::SampleAndHold => LedColor::RED,
            };
        }
        BoolOption::RelativePitch => {
            leds[8] = match quantizer_state.channel_b_mode {
                PitchMode::Relative => LedColor::GREEN,
                PitchMode::Absolute => LedColor::RED,
            };
        }
        BoolOption::ChannelsLinked => {
            leds[9] = match quantizer_state.channels_linked {
                true => LedColor::GREEN,
                false => LedColor::RED,
            }
        }
    }
    leds
}

fn handle_sub_menu_button_press(
    channel_state: &mut QuantizerChannel,
    status: &ScalarSubMenuStatus,
    menu: &ScalarSubMenu,
    button_idx: u8,
    shift_pressed: bool,
) -> Option<ScalarSubMenuStatus> {
    fn button_idx_to_i8(idx: u8) -> i8 {
        if idx <= 6 {
            idx as i8
        } else {
            (idx as i8) - 12
        }
    }

    // I could not begin to tell you why this is necessary, but if I don't
    // convince the compiler that button_idx is important, it just sets it to 0.
    // It's either a compiler bug or just the optimization lottery that is
    // avoiding a memory overflow/corruption error
    unsafe {
        asm!("and {0}, {0}", in(reg) button_idx);
    }

    match menu {
        ScalarSubMenu::Glide => {
            channel_state.config.glide_amount = button_idx;
        }
        ScalarSubMenu::Delay => {
            channel_state.config.trigger_delay_amount = button_idx;
        }
        ScalarSubMenu::PreShift => {
            channel_state.config.pre_shift = button_idx_to_i8(button_idx);
        }
        ScalarSubMenu::ScaleShift => {
            channel_state.config.scale_shift = button_idx_to_i8(button_idx);
        }
        ScalarSubMenu::PostShift => {
            channel_state.config.post_shift = button_idx_to_i8(button_idx);
        }
    }

    match status {
        ScalarSubMenuStatus::AwaitingFirstInput => Some(ScalarSubMenuStatus::ExitOnShiftRelease),
        ScalarSubMenuStatus::ExitOnShiftRelease => {
            if shift_pressed {
                Some(ScalarSubMenuStatus::ExitOnShiftRelease)
            } else {
                Some(ScalarSubMenuStatus::ExitOnButtonRelease)
            }
        }
        ScalarSubMenuStatus::ExitOnButtonRelease => None,
    }
}

fn render_sub_menu(sub_menu: &ScalarSubMenu, state: &QuantizerChannel) -> [LedColor; 12] {
    match sub_menu {
        ScalarSubMenu::Glide => render_sub_menu_unsigned(state.config.glide_amount),
        ScalarSubMenu::Delay => render_sub_menu_unsigned(state.config.trigger_delay_amount),
        ScalarSubMenu::PreShift => render_sub_menu_signed(state.config.pre_shift),
        ScalarSubMenu::ScaleShift => render_sub_menu_signed(state.config.scale_shift),
        ScalarSubMenu::PostShift => render_sub_menu_signed(state.config.post_shift),
    }
}

fn render_sub_menu_unsigned(n: u8) -> [LedColor; 12] {
    let mut leds = [LedColor::OFF; 12];
    leds[0] = LedColor::AMBER;
    for i in 1..n + 1 {
        leds[i as usize] = LedColor::GREEN;
    }

    leds
}

fn render_sub_menu_signed(n: i8) -> [LedColor; 12] {
    let mut leds = [LedColor::OFF; 12];
    leds[0] = LedColor::AMBER;
    if n < 0 {
        for i in (12 + n)..12 {
            leds[i as usize] = LedColor::RED;
        }
    } else {
        for i in 1..n + 1 {
            leds[i as usize] = LedColor::GREEN;
        }
    }

    leds
}

fn render_save_menu(
    selected_channel: &Channel,
    slot_type: &SaveSlotType,
    save_slots: &BitVec<12>,
) -> [LedColor; 12] {
    let color = match slot_type {
        SaveSlotType::FullConfig => LedColor::AMBER,
        SaveSlotType::Scale => match selected_channel {
            Channel::A => LedColor::GREEN,
            Channel::B => LedColor::RED,
        },
    };

    let mut result = [LedColor::OFF; 12];

    for i in 0..12u8 {
        if save_slots.get(i) {
            result[i as usize] = color;
        }
    }

    result
}

fn render_confirm_save(
    selected_channel: &Channel,
    slot_type: &SaveSlotType,
    time: &u32,
    slot: u8,
) -> [LedColor; 12] {
    let mut result = [LedColor::OFF; 12];

    if time & 128 == 0 {
        let color = match slot_type {
            SaveSlotType::FullConfig => LedColor::AMBER,
            SaveSlotType::Scale => match selected_channel {
                Channel::A => LedColor::GREEN,
                Channel::B => LedColor::RED,
            },
        };
        result[slot as usize] = color;
    }

    result
}

fn render_confirm_erse(time: &u32) -> [LedColor; 12] {
    let time_lsbs = (time & u16::MAX as u32) as u16;

    let index = (time_lsbs / 64).min(12) % 12;
    let mut leds = [LedColor::OFF; 12];
    leds[index as usize] = LedColor::AMBER;
    leds
}

pub struct ButtonInput {
    pub key_event: ButtonEvent,
    pub load_button: LongPressButtonState,
    pub save_button: LongPressButtonState,
    pub shift_pressed: bool,
}
