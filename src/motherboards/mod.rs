pub mod display;
mod raspberrypi;

pub fn console() -> impl core::fmt::Write {
    #[cfg(any(feature = "rpi4", feature = "rpi5"))]
    raspberrypi::console::RaspiWriter
}
