use core::u8;

use super::mailbox::{mbox_call, mbox_get, mbox_set};
use crate::motherboards::display::DisplayBuffer;

use crate::{
    color::Color,
    geometry, println,
    types::{KernelError, KernelResult},
};

pub struct HdmiBuffer;

impl HdmiBuffer {
    pub fn new() -> KernelResult<Self> {
        if init_buffer() {
            Ok(Self)
        } else {
            Err(KernelError::BufferInitError)
        }
    }
}

impl DisplayBuffer for HdmiBuffer {
    fn clear(&self, color: &Color) {
        clear(color);
    }

    fn draw_rect(&self, p1: geometry::Point, p2: geometry::Point, color: &Color) {
        draw_rect(p1.x(), p1.y(), p2.x(), p2.y(), color);
    }

    fn draw_triangle(
        &self,
        p0: geometry::Point,
        p1: geometry::Point,
        p2: geometry::Point,
        color: &Color,
    ) {
        draw_triangle(p0.x(), p0.y(), p1.x(), p1.y(), p2.x(), p2.y(), color);
    }

    fn debug(&self) {
        debug_dump();
    }
}

#[cfg(feature = "device")]
const PHYSICAL_WIDTH: u32 = 3440;
#[cfg(feature = "device")]
const PHYSICAL_HEIGHT: u32 = 1440;

#[cfg(feature = "emulator")]
const PHYSICAL_WIDTH: u32 = 1500;
#[cfg(feature = "emulator")]
const PHYSICAL_HEIGHT: u32 = 900;

const MBOX_REQUEST: u32 = 0;
const MBOX_CH_PROP: u32 = 8;
const MBOX_TAG_LAST: u32 = 0;

struct FrameConfig {
    buffer: usize,
    size: usize,
    width: u32,
    height: u32,
    pitch: u32,
    depth: u32,
    bits_per_pixel: u32,
    is_rgb: u32,
}

impl FrameConfig {
    const fn new() -> Self {
        FrameConfig {
            buffer: 0,
            size: 0,
            width: 0,
            height: 0,
            pitch: 0,
            depth: 0,
            bits_per_pixel: 4,
            is_rgb: 0,
        }
    }
}

static mut FRAME_CONFIG: FrameConfig = FrameConfig::new();

pub struct ScreenDimensions {
    pub height: u32,
    pub width: u32,
}

pub fn get_screen_dimensions() -> ScreenDimensions {
    unsafe {
        ScreenDimensions {
            width: FRAME_CONFIG.width,
            height: FRAME_CONFIG.height,
        }
    }
}

const PHYSICAL_SIZE_TAG: u32 = 0x0004_8003;
const VIRTUAL_SIZE_TAG: u32 = 0x0004_8004;
const VIRTUAL_OFFSET_TAG: u32 = 0x0004_8009;
const DEPTH_TAG: u32 = 0x0004_8005;
const PIXEL_ORDER_TAG: u32 = 0x0004_8006;
const FRAME_BUFFER_TAG: u32 = 0x0004_0001;
const PITCH_TAG: u32 = 0x0004_0008;

fn init_buffer() -> bool {
    const BUFFER_SIZE: u32 = 35 * 4;

    mbox_set(0, BUFFER_SIZE);
    mbox_set(1, MBOX_REQUEST);

    mbox_set(2, PHYSICAL_SIZE_TAG);
    mbox_set(3, 8);
    mbox_set(4, 8);
    mbox_set(5, PHYSICAL_WIDTH);
    mbox_set(6, PHYSICAL_HEIGHT);

    mbox_set(7, VIRTUAL_SIZE_TAG);
    mbox_set(8, 8);
    mbox_set(9, 8);
    mbox_set(10, PHYSICAL_WIDTH);
    mbox_set(11, PHYSICAL_HEIGHT);

    mbox_set(12, VIRTUAL_OFFSET_TAG);
    mbox_set(13, 8);
    mbox_set(14, 8);
    mbox_set(15, 0);
    mbox_set(16, 0);

    mbox_set(17, DEPTH_TAG);
    mbox_set(18, 4);
    mbox_set(19, 4);
    mbox_set(20, 32);

    mbox_set(21, PIXEL_ORDER_TAG);
    mbox_set(22, 4);
    mbox_set(23, 4);
    mbox_set(24, 1); // 1 = RGB

    mbox_set(25, FRAME_BUFFER_TAG);
    mbox_set(26, 8);
    mbox_set(27, 8);
    mbox_set(28, 4096); // request 4096-byte alignment; returns frame buffer ptr here
    mbox_set(29, 0); //    returns size here

    mbox_set(30, PITCH_TAG);
    mbox_set(31, 4);
    mbox_set(32, 4);
    mbox_set(33, 0); // returns pitch here

    mbox_set(34, MBOX_TAG_LAST);

    if mbox_call(MBOX_CH_PROP) && mbox_get(20) == 32 && mbox_get(28) != 0 {
        unsafe {
            // Convert the GPU bus address to an ARM physical address and store the buffer
            FRAME_CONFIG.buffer = (mbox_get(28) & 0x3FFF_FFFF) as usize;
            FRAME_CONFIG.size = mbox_get(29) as usize;
            FRAME_CONFIG.width = mbox_get(5);
            FRAME_CONFIG.height = mbox_get(6);
            FRAME_CONFIG.pitch = mbox_get(33);
            FRAME_CONFIG.depth = mbox_get(20);
            FRAME_CONFIG.bits_per_pixel = if FRAME_CONFIG.width != 0 {
                FRAME_CONFIG.pitch / FRAME_CONFIG.width
            } else {
                4
            };
            FRAME_CONFIG.is_rgb = mbox_get(24);
        }
        true
    } else {
        false
    }
}

fn clear(color: &Color) {
    let (base, size) = unsafe { (FRAME_CONFIG.buffer, FRAME_CONFIG.size) };
    if base == 0 || size == 0 {
        return;
    }

    for i in 0..size / 4 {
        write_color(base + i * 4, color);
    }
}

/// Write a color to a single pixel, honoring the framebuffer's bits-per-pixel.
/// Coordinates outside the screen (including negative) are silently dropped.
fn draw_pixel(x: i32, y: i32, color: &Color) {
    unsafe {
        if FRAME_CONFIG.buffer == 0
            || x < 0
            || y < 0
            || (x as u32) >= FRAME_CONFIG.width
            || (y as u32) >= FRAME_CONFIG.height
        {
            return;
        }
        let x = x as u32;
        let y = y as u32;
        let addr = FRAME_CONFIG.buffer
            + (y * FRAME_CONFIG.pitch + x * FRAME_CONFIG.bits_per_pixel) as usize;
        write_color(addr, color);
    }
}

/// Fill a solid rectangle with the given color. The rect is clipped to the
/// screen, so off-screen (including negative) corners are fine.
fn draw_rect(x1: i32, y1: i32, x2: i32, y2: i32, color: &Color) {
    let (w, h) = unsafe { (FRAME_CONFIG.width as i32, FRAME_CONFIG.height as i32) };
    let xa = x1.min(x2).max(0);
    let xb = x1.max(x2).min(w - 1);
    let ya = y1.min(y2).max(0);
    let yb = y1.max(y2).min(h - 1);
    if xa > xb || ya > yb {
        return; // fully off-screen
    }
    for y in ya..=yb {
        for x in xa..=xb {
            draw_pixel(x, y, color);
        }
    }
}

// Signed area of the triangle (a, b, p); sign tells which side of edge a->b p is on.
fn edge(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

/// Fill a triangle given by three vertices, using half-space rasterization.
fn draw_triangle(x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: &Color) {
    let (w, h) = unsafe { (FRAME_CONFIG.width as i32, FRAME_CONFIG.height as i32) };

    // Bounding box of the triangle, clamped to the screen.
    let min_x = x0.min(x1).min(x2).max(0);
    let min_y = y0.min(y1).min(y2).max(0);
    let max_x = x0.max(x1).max(x2).min(w - 1);
    let max_y = y0.max(y1).max(y2).min(h - 1);

    // Orientation of the whole triangle; a point is inside when all three edge
    // functions share this sign (covers both clockwise and counter-clockwise).
    let area = edge(x0, y0, x1, y1, x2, y2);
    if area == 0 {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let w0 = edge(x1, y1, x2, y2, x, y);
            let w1 = edge(x2, y2, x0, y0, x, y);
            let w2 = edge(x0, y0, x1, y1, x, y);
            let inside = if area > 0 {
                w0 >= 0 && w1 >= 0 && w2 >= 0
            } else {
                w0 <= 0 && w1 <= 0 && w2 <= 0
            };
            if inside {
                draw_pixel(x, y, color);
            }
        }
    }
}

fn glyph(c: u8) -> [u8; 8] {
    match c {
        b'0' => [0x70, 0x88, 0x98, 0xA8, 0xC8, 0x88, 0x70, 0x00],
        b'1' => [0x20, 0x60, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00],
        b'2' => [0x70, 0x88, 0x08, 0x10, 0x20, 0x40, 0xF8, 0x00],
        b'3' => [0x70, 0x88, 0x08, 0x30, 0x08, 0x88, 0x70, 0x00],
        b'4' => [0x10, 0x30, 0x50, 0x90, 0xF8, 0x10, 0x10, 0x00],
        b'5' => [0xF8, 0x80, 0xF0, 0x08, 0x08, 0x88, 0x70, 0x00],
        b'6' => [0x30, 0x40, 0x80, 0xF0, 0x88, 0x88, 0x70, 0x00],
        b'7' => [0xF8, 0x08, 0x10, 0x20, 0x40, 0x40, 0x40, 0x00],
        b'8' => [0x70, 0x88, 0x88, 0x70, 0x88, 0x88, 0x70, 0x00],
        b'9' => [0x70, 0x88, 0x88, 0x78, 0x08, 0x10, 0x60, 0x00],
        b'A' => [0x70, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88, 0x00],
        b'B' => [0xF0, 0x88, 0x88, 0xF0, 0x88, 0x88, 0xF0, 0x00],
        b'C' => [0x70, 0x88, 0x80, 0x80, 0x80, 0x88, 0x70, 0x00],
        b'D' => [0xF0, 0x88, 0x88, 0x88, 0x88, 0x88, 0xF0, 0x00],
        b'E' => [0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0xF8, 0x00],
        b'F' => [0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0x80, 0x00],
        b'G' => [0x70, 0x88, 0x80, 0xB8, 0x88, 0x88, 0x70, 0x00],
        b'H' => [0x88, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88, 0x00],
        b'I' => [0x70, 0x20, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00],
        b'J' => [0x38, 0x10, 0x10, 0x10, 0x90, 0x90, 0x60, 0x00],
        b'K' => [0x88, 0x90, 0xA0, 0xC0, 0xA0, 0x90, 0x88, 0x00],
        b'L' => [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xF8, 0x00],
        b'M' => [0x88, 0xD8, 0xA8, 0xA8, 0x88, 0x88, 0x88, 0x00],
        b'N' => [0x88, 0xC8, 0xA8, 0x98, 0x88, 0x88, 0x88, 0x00],
        b'O' => [0x70, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00],
        b'P' => [0xF0, 0x88, 0x88, 0xF0, 0x80, 0x80, 0x80, 0x00],
        b'Q' => [0x70, 0x88, 0x88, 0x88, 0xA8, 0x90, 0x68, 0x00],
        b'R' => [0xF0, 0x88, 0x88, 0xF0, 0xA0, 0x90, 0x88, 0x00],
        b'S' => [0x70, 0x88, 0x80, 0x70, 0x08, 0x88, 0x70, 0x00],
        b'T' => [0xF8, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00],
        b'U' => [0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00],
        b'V' => [0x88, 0x88, 0x88, 0x88, 0x88, 0x50, 0x20, 0x00],
        b'W' => [0x88, 0x88, 0x88, 0xA8, 0xA8, 0xD8, 0x88, 0x00],
        b'X' => [0x88, 0x88, 0x50, 0x20, 0x50, 0x88, 0x88, 0x00],
        b'Y' => [0x88, 0x88, 0x50, 0x20, 0x20, 0x20, 0x20, 0x00],
        b'Z' => [0xF8, 0x08, 0x10, 0x20, 0x40, 0x80, 0xF8, 0x00],
        b':' => [0x00, 0x20, 0x20, 0x00, 0x00, 0x20, 0x20, 0x00],
        _ => [0x00; 8],
    }
}

/// Draw a text glyph to the screen based on a bitmap
pub fn draw_char(c: u8, px: u32, py: u32, scale: u32) {
    let g = glyph(c);
    for row in 0..8u32 {
        let bits = g[row as usize];
        for col in 0..8u32 {
            let pixel_color = if (bits >> (7 - col)) & 1 == 1 {
                Color::white()
            } else {
                Color::black()
            };
            for sy in 0..scale {
                for sx in 0..scale {
                    draw_pixel(
                        (px + col * scale + sx) as i32,
                        (py + row * scale + sy) as i32,
                        &pixel_color,
                    );
                }
            }
        }
    }
}

fn write_color(addr: usize, color: &Color) {
    unsafe {
        if FRAME_CONFIG.bits_per_pixel == 2 {
            core::ptr::write_volatile(addr as *mut u16, color.into());
        } else {
            core::ptr::write_volatile(addr as *mut u32, color.into());
        }
    }
}

fn debug_dump() {
    let (w, h, p, s, bpp, d) = unsafe {
        (
            FRAME_CONFIG.width,
            FRAME_CONFIG.height,
            FRAME_CONFIG.pitch,
            FRAME_CONFIG.size,
            FRAME_CONFIG.bits_per_pixel,
            FRAME_CONFIG.depth,
        )
    };
    println!("Width: {}", w);
    println!("Height: {}", h);
    println!("Pitch: {}", p);
    println!("Size: {}", s);
    println!("Bits per pixel: {}", bpp);
    println!("Depth: {}", d);
}
