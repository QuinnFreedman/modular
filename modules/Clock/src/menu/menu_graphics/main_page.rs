use embedded_graphics::pixelcolor::BinaryColor;

use crate::{
    clock::ClockConfig,
    display_buffer::{Justify, MiniBuffer, TextColor},
    font::PRO_FONT_22,
    menu::{menu_state::EditingState, MenuUpdate},
    render_nubers::i8_to_str_b10,
};

#[inline(never)]
pub fn render_main_page<DI, SIZE>(
    cursor: u8,
    editing: EditingState,
    clock_state: &ClockConfig,
    menu_update: &MenuUpdate,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    match menu_update {
        MenuUpdate::NoUpdate => (),
        MenuUpdate::UpdateValueAtCursor | MenuUpdate::ToggleEditingAtCursor => {
            draw_top_level_menu_item(
                cursor,
                clock_state.channels[cursor as usize].division,
                match editing {
                    EditingState::Editing => ChannelStyle::Editing,
                    EditingState::Navigating => ChannelStyle::Selected,
                },
                display,
            )
        }
        MenuUpdate::MoveCursorFrom(old_cursor) => {
            let old_page = old_cursor / 4;
            let new_page = cursor / 4;
            if old_page == new_page {
                for (index, style) in [
                    (*old_cursor, ChannelStyle::Deselected),
                    (cursor, ChannelStyle::Selected),
                ] {
                    draw_top_level_menu_item(
                        index,
                        clock_state.channels[index as usize].division,
                        style,
                        display,
                    )
                }
            } else {
                full_render_main_page(editing, clock_state, cursor, display);
            }
        }
        MenuUpdate::SwitchScreens | MenuUpdate::Scroll(_) => {
            full_render_main_page(editing, clock_state, cursor, display);
        }
    }
}

#[derive(PartialEq, Eq)]
enum ChannelStyle {
    Editing,
    Selected,
    Deselected,
}

#[inline(never)]
fn full_render_main_page<DI, SIZE>(
    editing: EditingState,
    clock_state: &ClockConfig,
    cursor: u8,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let page_offset = (cursor / 4) * 4;
    for i in 0..4 {
        let channel_idx = page_offset + i;
        let style = if channel_idx == cursor {
            match editing {
                EditingState::Editing => ChannelStyle::Editing,
                EditingState::Navigating => ChannelStyle::Selected,
            }
        } else {
            ChannelStyle::Deselected
        };
        draw_top_level_menu_item(
            channel_idx,
            clock_state.channels[channel_idx as usize].division,
            style,
            display,
        )
    }
}

#[inline(never)]
fn draw_top_level_menu_item<DI, SIZE>(
    channel_index: u8,
    value: i8,
    state: ChannelStyle,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let idx_in_page = channel_index % 4;
    let x = idx_in_page % 2;
    let y = idx_in_page / 2;
    let screen_x = x * 64;
    let screen_y = y * 32;

    let mut buffer: [u8; 4] = [0u8; 4];
    let text = i8_to_str_b10(&mut buffer, value);

    let mut mini_buffer = MiniBuffer::<64, 32>::new();

    let margin = 5usize;
    if state == ChannelStyle::Editing {
        mini_buffer.fast_fill(
            margin,
            margin,
            64 - margin * 2,
            32 - margin * 2,
            BinaryColor::On,
        );
    }

    mini_buffer.fast_draw_ascii_text(
        Justify::Center(32),
        Justify::Center(16),
        text,
        &PRO_FONT_22,
        match state {
            ChannelStyle::Editing => &TextColor::BinaryOffTransparent,
            _ => &TextColor::BinaryOn,
        },
    );

    if state == ChannelStyle::Selected {
        mini_buffer.fast_rect(
            margin,
            margin,
            64 - margin * 2,
            32 - margin * 2,
            BinaryColor::On,
            2,
        );
    }
    mini_buffer.blit(display, screen_x, screen_y).unwrap();
}
