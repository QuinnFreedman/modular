use core::panic;

use crate::{
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
}

pub struct MenuState {
    selected_channel: Channel,
    menu_page: MenuPage,
    shift_was_pressed: bool,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            selected_channel: Channel::A,
            menu_page: MenuPage::MainMenu,
            shift_was_pressed: false,
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
                active_channel.notes.rotate_left(1);
            }
            1 => {
                active_channel.notes.rotate_right(1);
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
                active_channel.sample_mode = match active_channel.sample_mode {
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
        button_event: &ButtonEvent,
        shift_pressed: bool,
        active_notes: &QuantizationResult,
    ) -> [LedColor; 12] {
        if self.shift_was_pressed && !shift_pressed {
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
        self.shift_was_pressed = shift_pressed;

        match button_event {
            ButtonEvent::ButtonJustPressed(n) => match self.menu_page {
                MenuPage::ScalarSubMenu(ref mut status, ref menu) => {
                    let selected_channel =
                        &mut quantizer_state.channels[self.selected_channel.index()];
                    match handle_sub_menu_button_press(
                        selected_channel,
                        status,
                        menu,
                        *n,
                        shift_pressed,
                    ) {
                        Some(new_menu_status) => *status = new_menu_status,
                        None => self.menu_page = MenuPage::MainMenu,
                    }
                }
                MenuPage::MainMenu => {
                    if shift_pressed {
                        self.handle_shift_button_press(quantizer_state, *n);
                    } else {
                        let selected_channel =
                            &mut quantizer_state.channels[self.selected_channel.index()];
                        selected_channel.notes[*n as usize] = !selected_channel.notes[*n as usize];
                    }
                }
                MenuPage::ShowChangedBoolOption(_) => {}
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
            if selected_channel.notes[i] {
                leds[i] = color;
            }
        }

        match self.selected_channel {
            Channel::A => leds[(active_notes.channel_a as usize) % 12] = LedColor::AMBER,
            Channel::B => leds[(active_notes.channel_b as usize) % 12] = LedColor::AMBER,
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
            leds[4] = match channel.sample_mode {
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

    match menu {
        ScalarSubMenu::Glide => {
            channel_state.glide_amount = button_idx;
        }
        ScalarSubMenu::Delay => {
            channel_state.trigger_delay_amount = button_idx;
        }
        ScalarSubMenu::PreShift => {
            channel_state.pre_shift = button_idx_to_i8(button_idx);
        }
        ScalarSubMenu::ScaleShift => {
            channel_state.scale_shift = button_idx_to_i8(button_idx);
        }
        ScalarSubMenu::PostShift => {
            channel_state.post_shift = button_idx_to_i8(button_idx);
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
        ScalarSubMenu::Glide => render_sub_menu_unsigned(state.glide_amount),
        ScalarSubMenu::Delay => render_sub_menu_unsigned(state.trigger_delay_amount),
        ScalarSubMenu::PreShift => render_sub_menu_signed(state.pre_shift),
        ScalarSubMenu::ScaleShift => render_sub_menu_signed(state.scale_shift),
        ScalarSubMenu::PostShift => render_sub_menu_signed(state.post_shift),
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
