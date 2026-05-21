#[cfg(any(feature = "rpi3", feature = "rpi4"))]
#[path = "__board__/raspberrypi/raspberrypi.rs"]
mod raspberrypi;

#[path = "__board__/raspberrypi/console.rs"]
mod console;

#[path = "__board__/raspberrypi/peripherals.rs"]
pub mod peripherals;

pub use console::console;
