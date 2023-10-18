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

#[derive(PartialEq, Eq)]
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
