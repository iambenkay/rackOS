use crate::color::Color;
use crate::geometry::Point;
use crate::motherboards::display::{self, DisplayBuffer};

pub fn main() -> ! {
    if let Ok(screen_buffer) = display::display_buffer() {
        let black = Color::black();
        screen_buffer.clear(&black);
        screen_buffer.draw_rect(Point::xy(100), Point::xy(200), &Color::red());
        screen_buffer.draw_rect(Point::xy(300), Point::xy(400), &Color::green());
        screen_buffer.draw_rect(Point::xy(500), Point::xy(600), &Color::blue());
        screen_buffer.draw_rect(
            Point::new(700, 500),
            Point::new(900, 700),
            &Color::from_rgb(0xFF, 0xFF, 0),
        );
        screen_buffer.draw_triangle(
            Point::new(1100, 400),
            Point::new(1100, 600),
            Point::new(1200, 500),
            &Color::from_rgb(0, 0xFF, 0xFF),
        );
        screen_buffer.debug();
    }

    loop {}
}
