use embedded_graphics::pixelcolor::BinaryColor;

use crate::{
    clock::ClockConfig,
    display_buffer::{Justify, MiniBuffer, TextColor},
    font::PRO_FONT_22,
    menu::{menu_state::EditingState, MenuUpdate},
    render_nubers::u8_to_str_b10,
};

#[inline(never)]
pub fn render_bpm_page<DI, SIZE>(
    editing: EditingState,
    clock_state: &ClockConfig,
    menu_update: &MenuUpdate,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let mut buffer: [u8; 3] = [0u8; 3];
    let text = u8_to_str_b10(&mut buffer, clock_state.bpm);
    let mut mini_buffer = MiniBuffer::<64, 32>::new();

    if editing == EditingState::Editing {
        mini_buffer.fast_fill(0, 0, 64, 32, BinaryColor::On);
    }

    mini_buffer.fast_draw_ascii_text(
        Justify::Center(32),
        Justify::Center(16),
        text,
        &PRO_FONT_22,
        match editing {
            EditingState::Editing => &TextColor::BinaryOffTransparent,
            EditingState::Navigating => &TextColor::BinaryOn,
        },
    );
    if *menu_update == MenuUpdate::SwitchScreens {
        display.clear().unwrap();
    }
    mini_buffer.blit(display, 32, 16).unwrap();
}
