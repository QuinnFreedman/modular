use crate::random::Rng;

pub enum MenuOrScreenSaverState {
    ScreenSaver(ScreenSaverState),
    Menu(MenuState),
}

#[repr(packed)]
pub struct ScreenSaverState {
    pub y_offsets: [u8; 16],
    pub color: bool,
    pub rng: Rng,
}

#[repr(packed)]
pub struct MenuState {
    pub page: MenuPage,
    pub editing: EditingState,
    pub last_input_time_ms: u32,
}

impl MenuOrScreenSaverState {
    pub fn new(current_time_ms: u32) -> Self {
        MenuOrScreenSaverState::Menu(MenuState::new(current_time_ms))
    }
}

impl MenuState {
    pub fn new(time: u32) -> Self {
        MenuState {
            page: MenuPage::Bpm,
            editing: EditingState::Navigating,
            last_input_time_ms: time,
        }
    }
}

impl ScreenSaverState {
    pub fn new(seed: u32) -> Self {
        let rng = Rng::new(seed);
        let y_offsets = [0u8; 16];
        // for i in 0..16 {
        //     y_offsets[i] = rng.next() % 8;
        // }
        Self {
            y_offsets,
            color: true,
            rng,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EditingState {
    Editing,
    Navigating,
}

impl EditingState {
    pub fn toggle(&self) -> Self {
        match self {
            EditingState::Editing => EditingState::Navigating,
            EditingState::Navigating => EditingState::Editing,
        }
    }
}

impl Into<bool> for EditingState {
    fn into(self) -> bool {
        match self {
            EditingState::Editing => true,
            EditingState::Navigating => false,
        }
    }
}

pub enum MenuPage {
    Bpm,
    Main { cursor: u8 },
    SubMenu { cursor: u8, scroll: u8, channel: u8 },
}

#[derive(PartialEq, Eq)]
pub enum MenuUpdate {
    NoUpdate,
    UpdateValueAtCursor,
    ToggleEditingAtCursor,
    MoveCursorFrom(u8),
    Scroll(ScrollDirection),
    SwitchScreens,
    ScreenSaverStep(u8),
}

#[derive(PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SubMenuItem {
    Division = 0,
    PulseWidth = 1,
    PhaseShift = 2,
    Swing = 3,
    Probability = 4,
    Exit = 5,
}

impl Into<u8> for SubMenuItem {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for SubMenuItem {
    fn from(value: u8) -> Self {
        match value {
            const { Self::Division as u8 } => Self::Division,
            const { Self::PulseWidth as u8 } => Self::PulseWidth,
            const { Self::PhaseShift as u8 } => Self::PhaseShift,
            const { Self::Swing as u8 } => Self::Swing,
            const { Self::Probability as u8 } => Self::Probability,
            const { Self::Exit as u8 } => Self::Exit,
            _ => panic!(),
        }
    }
}
