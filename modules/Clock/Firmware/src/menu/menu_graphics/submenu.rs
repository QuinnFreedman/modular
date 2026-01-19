use avr_progmem::{progmem, progmem_str as F};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::DrawTarget};
use fm_lib::debug_unwrap::DebugUnwrap;

use crate::{
    clock::ClockChannelConfig,
    display_buffer::{Justify, MiniBuffer, TextColor},
    font::PRO_FONT_22,
    menu::{
        menu_state::{EditingState, SubMenuItem},
        MenuUpdate,
    },
    render_numbers::{i8_to_str_b10, tempo_to_str, u8_to_str_b10},
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
            draw_submenu_item(cursor, true, scroll, editing.into(), false, channel, display);
        }
        MenuUpdate::MoveCursorFrom(_) | MenuUpdate::Scroll(_) | MenuUpdate::SwitchScreens => {
            if channel.division == -65 {
                let editing_division = match editing {
                    EditingState::Editing => {
                        if cursor == 0 {
                            EditingState::Editing
                        } else {
                            EditingState::Navigating
                        }
                    }
                    EditingState::Navigating => EditingState::Navigating,
                };
                draw_arrows(true, false, display);
                draw_arrows(false, false, display);
                draw_submenu_item(
                    0,
                    cursor == 0,
                    scroll,
                    editing_division,
                    true,
                    channel,
                    display,
                );
                let exit_y_offset = 24 + 8;
                draw_submenu_item_label(exit_y_offset, cursor == 5, SubMenuItem::Exit, display);
                let mut buffer = MiniBuffer::<54, 24>::new();
                if cursor == 5 {
                    buffer.clear(BinaryColor::On).assert_ok();
                }
                buffer.blit(display, 74, exit_y_offset).assert_ok();
            } else {
                let at_top = scroll == 0;
                let at_bottom = scroll >= 4;
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
        }
        MenuUpdate::NoUpdate | MenuUpdate::ScreenSaverStep(_) => {}
    }
}

progmem! {
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
    static progmem RETURN_ARROW: [u8; 57] = *include_bytes!("../../../assets/back_arrow.bin");
    static progmem SLASH_64: [u8; 36] = *include_bytes!("../../../assets/div_64.bin");
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
        let center: u8 = 128 / 2;

        let offsets = [
            center - img_width / 2,
            center - spacing - img_width / 2,
            center + spacing - img_width / 2,
        ];
        for offset in offsets {
            buffer.fast_draw_image(offset as usize, 0, img_width, 8, img, &TextColor::BinaryOn);
        }
    }

    buffer.blit(display, 0, y).assert_ok();
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
        buffer.clear(BinaryColor::On).assert_ok();
        TextColor::BinaryOff
    } else {
        TextColor::BinaryOn
    };
    let mut text_buffer = [0u8; 4];
    let (text, symbol): (&[u8], Option<[u8; 36]>) = match item {
        SubMenuItem::Division => (tempo_to_str(&mut text_buffer, channel.division), None),
        SubMenuItem::PulseWidth => match channel.pulse_width {
            0 => {
                text_buffer.copy_from_slice("TRIG".as_bytes());
                (&text_buffer, None)
            }
            100 => {
                text_buffer.copy_from_slice("INVT".as_bytes());
                (&text_buffer, None)
            }
            pulse_width => {
                let text = u8_to_str_b10(&mut text_buffer, pulse_width);
                // This could be done by appending '%' to the text buffer but
                // this is a little easier
                // Using custom code page to save space, '_' is mapped to '/'
                (text, Some(PRO_FONT_22.get_glyph(b'^')))
            }
        },
        SubMenuItem::PhaseShift => (
            i8_to_str_b10(&mut text_buffer, channel.phase_shift),
            Some(SLASH_64.load()),
        ),
        SubMenuItem::Swing => (
            u8_to_str_b10(&mut text_buffer, channel.swing),
            Some(SLASH_64.load())
        ),
        SubMenuItem::Probability => (
            u8_to_str_b10(&mut text_buffer, channel.probability),
            Some(PRO_FONT_22.get_glyph(b'^')),
        ),
        SubMenuItem::Exit => (&text_buffer[0..0], None),
    };
    let mut align_to: u8 = 52;
    if let Some(img) = symbol {
        align_to -= 12;
        buffer.fast_draw_image(align_to as usize, 1, 12, 24, &img, &text_color);
    }
    buffer.fast_draw_ascii_text(
        Justify::End(align_to as usize),
        Justify::Start(1),
        text,
        &PRO_FONT_22,
        &text_color,
    );
    if editing == EditingState::Editing {
        buffer.fast_rect(0, 0, 54, 24, BinaryColor::On, 2);
    }
    buffer.blit(display, 74, y_offset).assert_ok();
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
        buffer.clear(BinaryColor::On).assert_ok();
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
        SubMenuItem::Probability => buffer.fast_draw_ascii_text(
            Justify::Start(2),
            Justify::Start(1),
            F!("Prob").as_bytes(),
            &PRO_FONT_22,
            text_color,
        ),
        SubMenuItem::Exit => {
            let img = RETURN_ARROW.load();
            buffer.fast_draw_image(2, 0, 19, 24, &img, text_color);
            buffer.fast_draw_ascii_text(
                Justify::Start(26),
                Justify::Start(1),
                F!("Exit").as_bytes(),
                &PRO_FONT_22,
                text_color,
            );
        }
    }
    buffer.blit(display, 0, y_offset).assert_ok();
}
