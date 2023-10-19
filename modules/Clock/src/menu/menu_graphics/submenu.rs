use avr_progmem::{progmem, progmem_str as F, wrapper::ProgMem};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::DrawTarget};

use crate::{
    clock::{ClockChannelConfig, ClockConfig},
    display_buffer::{Justify, MiniBuffer, TextColor},
    font::PRO_FONT_22,
    menu::{
        menu_state::{EditingState, SubMenuItem},
        MenuState, MenuUpdate,
    },
    render_nubers::{i8_to_str_b10, u8_to_str_b10},
};

#[inline(always)]
pub fn render_submenu_page<DI, SIZE>(
    cursor: u8,
    scroll: u8,
    channel: &ClockChannelConfig,
    editing: EditingState,
    menu_update: &MenuUpdate,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    display.clear().unwrap();
    for i in scroll..scroll + 2 {
        let selected = cursor == i;
        let editing_item = match editing {
            EditingState::Editing => {
                if selected {
                    EditingState::Editing
                } else {
                    EditingState::Navigating
                }
            }
            EditingState::Navigating => EditingState::Navigating,
        };
        draw_submenu_item(i, selected, scroll, editing_item, channel, display);
    }
}

progmem! {
    static progmem string DIVISION_TEXT = "DIVIS";
    static progmem string PULSEWIDTH_TEXT = "PULSE";
    static progmem string PHASESHIFT_TEXT = "PHASE";
    static progmem string SWING_TEXT = "SWING";
    static progmem string EXIT_TEXT = "EXIT";
}

#[inline(always)]
fn draw_submenu_item<DI, SIZE>(
    index: u8,
    selected: bool,
    scroll: u8,
    editing: EditingState,
    channel: &ClockChannelConfig,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let offset_y = (index.saturating_sub(scroll)) * 24 + 8;
    let menu_item: SubMenuItem = index.into();
    draw_submenu_item_label(offset_y, selected, menu_item, display);
    draw_submenu_item_value(offset_y, selected, editing, menu_item, channel, display);
}

#[inline(never)]
fn draw_submenu_item_value<DI, SIZE>(
    y_offset: u8,
    selected: bool,
    editing: EditingState,
    item: SubMenuItem,
    channel: &ClockChannelConfig,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let mut buffer = MiniBuffer::<54, 24>::new();
    let text_color = if selected && !Into::<bool>::into(editing) {
        buffer.clear(BinaryColor::On).unwrap();
        TextColor::BinaryOff
    } else {
        TextColor::BinaryOn
    };
    let value: i8 = match item {
        SubMenuItem::Division => channel.division,
        SubMenuItem::PulseWidth => channel.pulse_width,
        SubMenuItem::PhaseShift => channel.phase_shift,
        SubMenuItem::Swing => channel.swing,
        SubMenuItem::Exit => 0,
    };
    let mut text_buffer = [0u8; 4];
    let text = i8_to_str_b10(&mut text_buffer, value);
    buffer.fast_draw_ascii_text(
        Justify::End(52),
        Justify::Start(1),
        text,
        &PRO_FONT_22,
        &text_color,
    );
    if editing == EditingState::Editing {
        buffer.fast_rect(0, 0, 54, 24, BinaryColor::On, 2);
    }
    buffer.blit(display, 74, y_offset).unwrap();
}

#[inline(never)]
fn draw_submenu_item_label<DI, SIZE>(
    y_offset: u8,
    invert: bool,
    item: SubMenuItem,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let mut buffer = MiniBuffer::<74, 24>::new();

    if invert {
        buffer.clear(BinaryColor::On).unwrap();
    }

    let text_color = match invert {
        true => &TextColor::BinaryOff,
        false => &TextColor::BinaryOn,
    };

    match item {
        SubMenuItem::Division => buffer.fast_draw_ascii_text(
            Justify::Start(2),
            Justify::Start(1),
            F!("Divis").as_bytes(),
            &PRO_FONT_22,
            text_color,
        ),
        SubMenuItem::PulseWidth => buffer.fast_draw_ascii_text(
            Justify::Start(2),
            Justify::Start(1),
            F!("PWidth").as_bytes(),
            &PRO_FONT_22,
            text_color,
        ),
        SubMenuItem::PhaseShift => buffer.fast_draw_ascii_text(
            Justify::Start(2),
            Justify::Start(1),
            F!("PShift").as_bytes(),
            &PRO_FONT_22,
            text_color,
        ),
        SubMenuItem::Swing => buffer.fast_draw_ascii_text(
            Justify::Start(2),
            Justify::Start(1),
            F!("Swing").as_bytes(),
            &PRO_FONT_22,
            text_color,
        ),
        SubMenuItem::Exit => buffer.fast_draw_ascii_text(
            Justify::Start(2),
            Justify::Start(1),
            F!("Exit").as_bytes(),
            &PRO_FONT_22,
            text_color,
        ),
    }
    buffer.blit(display, 0, y_offset).unwrap();
}
