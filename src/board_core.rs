#[path = "__board__/raspberrypi/raspberrypi.rs"]
mod raspberrypi;

#[path = "__board__/raspberrypi/screen.rs"]
mod screen;

#[path = "__board__/raspberrypi/console.rs"]
mod console;

pub use console::console;
