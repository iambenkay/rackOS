use crate::board_core::screen::{self, Color};

pub fn main() -> ! {
    if screen::init_buffer() {
        let black = Color::black();
        screen::clear(&black);
        screen::draw_rect(100, 100, 200, 200, &Color::red());
        screen::draw_rect(300, 300, 400, 400, &Color::green());
        screen::draw_rect(500, 500, 600, 600, &Color::blue());
        screen::draw_rect(700, 500, 900, 700, &Color::from_rgb(0xFF, 0xFF, 0));

        screen::debug::debug_dump();
    }

    loop {}
}
