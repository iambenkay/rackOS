use crate::color;
use crate::geometry;
use crate::types::KernelResult;

use super::raspberrypi::hdmi::HdmiBuffer;

pub fn display_buffer() -> KernelResult<impl DisplayBuffer> {
    #[cfg(any(feature = "rpi5", feature = "rpi4"))]
    HdmiBuffer::new()
}

pub trait DisplayBuffer {
    fn clear(&self, color: &color::Color);
    fn draw_rect(&self, p1: geometry::Point, p2: geometry::Point, color: &color::Color);
    fn draw_triangle(
        &self,
        p0: geometry::Point,
        p1: geometry::Point,
        p2: geometry::Point,
        color: &color::Color,
    );
    fn debug(&self);
}
