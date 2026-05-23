#[path = "__board__/raspberrypi/raspberrypi.rs"]
mod raspberrypi;

#[path = "__board__/raspberrypi/screen.rs"]
pub mod screen;

#[path = "__board__/raspberrypi/console.rs"]
pub mod console;

pub use console::console;
