#[path = "mailbox.rs"]
mod mailbox;

#[path = "color.rs"]
mod color;

use core::u8;

pub use color::Color;
use mailbox::{mbox_call, mbox_get, mbox_set};

#[cfg(feature = "rpi5")]
const PHYSICAL_WIDTH: u32 = 3440;
#[cfg(feature = "rpi5")]
const PHYSICAL_HEIGHT: u32 = 1440;

#[cfg(feature = "rpi4")]
const PHYSICAL_WIDTH: u32 = 1500;
#[cfg(feature = "rpi4")]
const PHYSICAL_HEIGHT: u32 = 900;

const MBOX_REQUEST: u32 = 0;
const MBOX_CH_PROP: u32 = 8;
const MBOX_TAG_LAST: u32 = 0;

static mut FRAME_BUFFER: usize = 0;
static mut SIZE: usize = 0;
static mut WIDTH: u32 = 0;
static mut HEIGHT: u32 = 0;
static mut PITCH: u32 = 0;
static mut DEPTH: u32 = 0;
static mut BPP: u32 = 4;
static mut ISRGB: u32 = 0;

const PHYSICAL_SIZE_TAG: u32 = 0x0004_8003;
const VIRTUAL_SIZE_TAG: u32 = 0x0004_8004;
const VIRTUAL_OFFSET_TAG: u32 = 0x0004_8009;
const DEPTH_TAG: u32 = 0x0004_8005;
const PIXEL_ORDER_TAG: u32 = 0x0004_8006;
const FRAME_BUFFER_TAG: u32 = 0x0004_0001;
const PITCH_TAG: u32 = 0x0004_0008;

pub fn init_buffer() -> bool {
    const BUFFER_SIZE: u32 = 35 * 4;

    unsafe {
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
            // Convert the GPU bus address to an ARM physical address.
            FRAME_BUFFER = (mbox_get(28) & 0x3FFF_FFFF) as usize;
            SIZE = mbox_get(29) as usize;
            WIDTH = mbox_get(5);
            HEIGHT = mbox_get(6);
            PITCH = mbox_get(33);
            DEPTH = mbox_get(20);

            BPP = if WIDTH != 0 { PITCH / WIDTH } else { 4 };
            ISRGB = mbox_get(24);
            true
        } else {
            false
        }
    }
}

pub fn clear(color: &Color) {
    let (base, size) = unsafe { (FRAME_BUFFER, SIZE) };
    if base == 0 || size == 0 {
        return;
    }

    for i in 0..size / 4 {
        write_color(base + i * 4, color);
    }
}

/// Write a color to a single pixel, honoring the framebuffer's bytes-per-pixel.
pub fn draw_pixel(x: u32, y: u32, color: &Color) {
    unsafe {
        if FRAME_BUFFER == 0 || x >= WIDTH || y >= HEIGHT {
            return;
        }
        let addr = FRAME_BUFFER + (y * PITCH + x * BPP) as usize;
        write_color(addr, color);
    }
}

/// Fill a solid rectangle with the given color.
pub fn draw_rect(x1: u32, y1: u32, x2: u32, y2: u32, color: &Color) {
    for y in y1..=y2 {
        for x in x1..=x2 {
            draw_pixel(x, y, color);
        }
    }
}

// Signed area of the triangle (a, b, p); sign tells which side of edge a->b p is on.
fn edge(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

/// Fill a triangle given by three vertices, using half-space rasterization.
pub fn draw_triangle(x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: &Color) {
    let (w, h) = unsafe { (WIDTH as i32, HEIGHT as i32) };

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
                draw_pixel(x as u32, y as u32, color);
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

fn put_raw(x: u32, y: u32, color: &Color) {
    unsafe {
        if FRAME_BUFFER == 0 || x >= WIDTH || y >= HEIGHT {
            return;
        }
        let addr = FRAME_BUFFER + (y * PITCH + x * BPP) as usize;

        write_color(addr, color);
    }
}

fn draw_char(c: u8, px: u32, py: u32, scale: u32) {
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
                    put_raw(px + col * scale + sx, py + row * scale + sy, &pixel_color);
                }
            }
        }
    }
}

fn draw_str(s: &[u8], px: u32, py: u32, scale: u32) {
    let mut cx = px;
    for &c in s {
        draw_char(c, cx, py, scale);
        cx += 8 * scale;
    }
}

fn write_color(addr: usize, color: &Color) {
    unsafe {
        if BPP == 2 {
            core::ptr::write_volatile(addr as *mut u16, color.into());
        } else {
            core::ptr::write_volatile(addr as *mut u32, color.into());
        }
    }
}

pub mod debug {
    static mut LOG_Y: u32 = 8;

    pub fn log_u32(key: &[u8], value: u32) {
        let scale = 1u32;
        let mut buf = [b' '; 50];
        let mut n = 0;
        for &c in key {
            buf[n] = c;
            n += 1;
        }
        buf[n] = b':';
        n += 1;
        for shift in (0..8).rev() {
            let nib = ((value >> (shift * 4)) & 0xF) as u8;
            buf[n] = if nib < 10 {
                b'0' + nib
            } else {
                b'A' + nib - 10
            };
            n += 1;
        }
        n += 1;
        buf[n] = b'O';
        n += 1;
        buf[n] = b'R';
        n += 2;

        let (digits, len) = number_to_char_array::<12>(value);
        for i in 0..len {
            buf[n] = digits[i];
            n += 1;
        }

        let width_px = 27 * 8 * scale; // reserve 25 chars
        let x = unsafe { super::WIDTH }.saturating_sub(width_px + 8);
        let y = unsafe { LOG_Y };
        super::draw_str(&buf[..n], x, y, scale);
        unsafe { LOG_Y += 8 * scale + 4 };
    }

    /// Dump the firmware-returned framebuffer geometry to the top-right corner.
    pub fn debug_dump() {
        unsafe {
            log_u32(b"W", super::WIDTH);
            log_u32(b"H", super::HEIGHT);
            log_u32(b"P", super::PITCH);
            log_u32(b"S", super::SIZE as u32);
            log_u32(b"DEPTH", super::DEPTH);
            log_u32(b"FB", super::FRAME_BUFFER as u32);
        }
    }

    fn number_to_char_array<const N: usize>(mut num: u32) -> ([u8; N], usize) {
        let mut buffer = [b'\0'; N];
        let mut index = N;

        if num == 0 {
            index -= 1;
            buffer[index] = b'0';
            return (buffer, N - index);
        }

        while num != 0 {
            let remainder = num % 10;
            index -= 1;
            buffer[index] = b'0' + remainder as u8;
            num /= 10;
        }

        let mut result = [b'\0'; N];
        let len = N - index;
        result[..len].copy_from_slice(&buffer[index..N]);

        (result, len)
    }
}
