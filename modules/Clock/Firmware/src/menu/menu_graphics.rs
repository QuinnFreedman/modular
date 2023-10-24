mod bpm_page;
mod main_page;
mod submenu;

use crate::clock::ClockConfig;

use self::{bpm_page::render_bpm_page, main_page::render_main_page, submenu::render_submenu_page};

use super::{
    menu_state::{MenuPage, MenuUpdate},
    MenuState,
};

#[inline(always)]
pub fn render_menu<DI, SIZE>(
    menu_state: &MenuState,
    clock_state: &ClockConfig,
    menu_update: &MenuUpdate,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    match menu_state.page {
        MenuPage::Bpm => render_bpm_page(menu_state.editing, clock_state, menu_update, display),
        MenuPage::Main { cursor } => render_main_page(
            cursor,
            menu_state.editing,
            clock_state,
            menu_update,
            display,
        ),
        MenuPage::SubMenu {
            cursor,
            scroll,
            channel,
        } => {
            render_submenu_page(
                cursor,
                scroll,
                &clock_state.channels[channel as usize],
                menu_state.editing,
                menu_update,
                display,
            );
        }
    }
}
