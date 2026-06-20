use super::hdmi::{draw_char, get_screen_dimensions};

pub struct RaspiWriter;

const CON_SCALE: u32 = 1;
static mut CON_X: u32 = 0;
static mut CON_Y: u32 = 0;

impl core::fmt::Write for RaspiWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &b in s.as_bytes() {
            con_put(b);
        }
        Ok(())
    }
}

fn con_newline() {
    let dim = get_screen_dimensions();
    unsafe {
        CON_X = 0;
        CON_Y += 8 * CON_SCALE + 2;
        if CON_Y + 8 * CON_SCALE > dim.height {
            CON_Y = 0;
        }
    }
}

fn con_put(b: u8) {
    let dim = get_screen_dimensions();
    let cell = 8 * CON_SCALE;
    match b {
        b'\n' => con_newline(),
        b'\r' => unsafe { CON_X = 0 },
        _ => unsafe {
            if CON_X + cell > dim.width {
                con_newline();
            }
            // Font is uppercase-only; fold lowercase so messages stay legible.
            let c = if b.is_ascii_lowercase() { b - 32 } else { b };
            draw_char(c, CON_X, CON_Y, CON_SCALE);
            CON_X += cell;
        },
    }
}
