use crate::board_core::screen::{self, Color};

pub fn entrypoint() -> ! {
    if screen::init() {
        let black = Color::black();
        screen::clear(&black);

        screen::draw_rect(100, 100, 200, 200, &Color::red());
        screen::draw_rect(300, 300, 400, 400, &Color::green());
        screen::draw_rect(500, 500, 600, 600, &Color::blue());

        screen::debug_dump();
    }

    loop {}
}
