#[path = "__board__/raspberrypi/cpu.rs"]
mod cpu;

#[path = "__board__/raspberrypi/console.rs"]
mod console;

pub fn console() -> impl core::fmt::Write {
    #[cfg(any(feature = "rpi4", feature = "rpi5"))]
    console::RaspiWriter
}
