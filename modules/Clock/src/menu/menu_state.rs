pub struct MenuState {
    pub page: MenuPage,
    pub editing: EditingState,
}

impl MenuState {
    pub fn new() -> Self {
        MenuState {
            page: MenuPage::Bpm,
            editing: EditingState::Navigating,
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
    Exit = 4,
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
            const { Self::Exit as u8 } => Self::Exit,
            _ => panic!(),
        }
    }
}
