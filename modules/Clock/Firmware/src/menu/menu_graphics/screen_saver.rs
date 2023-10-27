use crate::menu::{menu_state::ScreenSaverState, MenuUpdate};

pub fn render_screensaver<DI, SIZE>(
    ss_state: &ScreenSaverState,
    menu_update: &MenuUpdate,
    display: &mut ssd1306::Ssd1306<DI, SIZE, ssd1306::mode::BasicMode>,
) where
    DI: display_interface::WriteOnlyDataCommand,
    SIZE: ssd1306::size::DisplaySize,
{
    match menu_update {
        MenuUpdate::ScreenSaverStep(col) => {
            let row = ss_state.y_offsets[*col as usize];
            debug_assert!(row < 8);
            debug_assert!(*col < 16);
            let row_px = row * 8;
            let col_px = col * 8;
            display
                .set_draw_area((col_px, row_px), (col_px + 8, row_px + 8))
                .unwrap();
            let color = if ss_state.color { 0xff } else { 0x00 };
            for _ in 0..8 {
                display.draw(&[color]).unwrap();
            }
        }
        MenuUpdate::SwitchScreens => display.clear().unwrap(),
        _ => {}
    }
}
