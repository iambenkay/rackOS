use crate::board_core::screen::{self, Color};

pub fn entrypoint() -> ! {
    if screen::init() {
        let black = Color::from_rgb(0, 0, 0);
        screen::clear(&black);

        screen::draw_rect(100, 100, 200, 200, &Color::red());
    }

    loop {}
}
