use avr_progmem::{progmem, progmem_str as F};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::DrawTarget};

use crate::{
    clock::ClockChannelConfig,
    display_buffer::{Justify, MiniBuffer, TextColor},
    font::PRO_FONT_22,
    menu::{
        menu_state::{EditingState, SubMenuItem},
        MenuUpdate,
    },
    render_nubers::{i8_to_str_b10, tempo_to_str, u8_to_str_b10},
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
    match menu_update {
        MenuUpdate::UpdateValueAtCursor | MenuUpdate::ToggleEditingAtCursor => {
            draw_submenu_item(
                cursor,
                true,
                scroll,
                editing.into(),
                false,
                channel,
                display,
            );
        }
        MenuUpdate::MoveCursorFrom(_) | MenuUpdate::Scroll(_) | MenuUpdate::SwitchScreens => {
            let at_top = scroll == 0;
            let at_bottom = scroll >= 3;
            draw_arrows(true, !at_top, display);
            draw_arrows(false, !at_bottom, display);
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
                draw_submenu_item(i, selected, scroll, editing_item, true, channel, display);
            }
        }
        MenuUpdate::NoUpdate => {}
    }
}

progmem! {
    static progmem string DIVISION_TEXT = "DIVIS";
    static progmem string PULSEWIDTH_TEXT = "PULSE";
    static progmem string PHASESHIFT_TEXT = "PHASE";
    static progmem string SWING_TEXT = "SWING";
    static progmem string EXIT_TEXT = "EXIT";
    static progmem TRIANGLE_DOWN: [u8; 11] = [
        0b00000100,
        0b00001100,
        0b00011100,
        0b00111100,
        0b01111100,
        0b11111100,
        0b01111100,
        0b00111100,
        0b00011100,
        0b00001100,
        0b00000100,
    ];
    static progmem TRIANGLE_UP: [u8; 11] = [
        0b00100000,
        0b00110000,
        0b00111000,
        0b00111100,
        0b00111110,
        0b00111111,
        0b00111110,
        0b00111100,
        0b00111000,
        0b00110000,
        0b00100000,
    ];
}

#[inline(never)]
fn draw_arrows<DI, SIZE>(
    top: bool,
    draw_arrows: bool,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let mut buffer = MiniBuffer::<128, 8>::new();
    let y = if top { 0 } else { 64 - 8 };

    if draw_arrows {
        let img_progmem = if top { TRIANGLE_UP } else { TRIANGLE_DOWN };
        let img = &img_progmem.load();

        let img_width: u8 = img.len() as u8;
        let spacing: u8 = 30;

        let offsets = [
            128 / 2 - img_width / 2,
            128 / 2 - spacing - img_width / 2,
            128 / 2 + spacing - img_width / 2,
        ];
        for offset in offsets {
            buffer.fast_draw_image(offset as usize, 0, img_width, 8, img, &TextColor::BinaryOn);
        }
    }

    buffer.blit(display, 0, y).unwrap();
}

#[inline(always)]
fn draw_submenu_item<DI, SIZE>(
    index: u8,
    selected: bool,
    scroll: u8,
    editing: EditingState,
    full_update: bool,
    channel: &ClockChannelConfig,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    let offset_y = (index.saturating_sub(scroll)) * 24 + 8;
    let menu_item: SubMenuItem = index.into();
    if full_update {
        draw_submenu_item_label(offset_y, selected, menu_item, display);
    }
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
    let mut text_buffer = [0u8; 4];
    let text = match item {
        SubMenuItem::Division => tempo_to_str(&mut text_buffer, channel.division),
        SubMenuItem::PulseWidth => u8_to_str_b10(&mut text_buffer, channel.pulse_width),
        SubMenuItem::PhaseShift => i8_to_str_b10(&mut text_buffer, channel.phase_shift),
        SubMenuItem::Swing => i8_to_str_b10(&mut text_buffer, channel.swing),
        SubMenuItem::Exit => &text_buffer[0..0],
    };
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
            F!("Tempo").as_bytes(),
            &PRO_FONT_22,
            text_color,
        ),
        SubMenuItem::PulseWidth => buffer.fast_draw_ascii_text(
            Justify::Start(2),
            Justify::Start(1),
            F!("PulseW").as_bytes(),
            &PRO_FONT_22,
            text_color,
        ),
        SubMenuItem::PhaseShift => buffer.fast_draw_ascii_text(
            Justify::Start(2),
            Justify::Start(1),
            F!("Phase").as_bytes(),
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
