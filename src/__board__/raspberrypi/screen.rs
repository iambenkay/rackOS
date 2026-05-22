use core::{
    ptr::{addr_of, addr_of_mut, read_volatile, write_volatile},
    u8,
};

#[repr(C, align(16))]
struct Mailbox([u32; 36]);

impl Mailbox {
    const fn new() -> Mailbox {
        Mailbox([0; 36])
    }
}

static mut MBOX: Mailbox = Mailbox::new();

#[cfg(feature = "rpi5")]
const VIDEOCORE_MBOX: usize = 0x10_7C01_3880;

#[cfg(feature = "rpi4")]
const VIDEOCORE_MBOX: usize = 0xFE00_B880;

#[cfg(feature = "rpi5")]
const PHYSICAL_WIDTH: u32 = 3440;
#[cfg(feature = "rpi5")]
const PHYSICAL_HEIGHT: u32 = 1440;

#[cfg(feature = "rpi4")]
const PHYSICAL_WIDTH: u32 = 1500;
#[cfg(feature = "rpi4")]
const PHYSICAL_HEIGHT: u32 = 900;

const MBOX_READ: usize = VIDEOCORE_MBOX + 0x00;
const MBOX_STATUS: usize = VIDEOCORE_MBOX + 0x18;
const MBOX_WRITE: usize = VIDEOCORE_MBOX + 0x20;

const MBOX_RESPONSE: u32 = 0x8000_0000;
const MBOX_FULL: u32 = 0x8000_0000;
const MBOX_EMPTY: u32 = 0x4000_0000;
const MBOX_REQUEST: u32 = 0;
const MBOX_CH_PROP: u32 = 8;
const MBOX_TAG_LAST: u32 = 0;

static mut FRAME_BUFFER: usize = 0;
static mut SIZE: usize = 0;
static mut WIDTH: u32 = 0;
static mut HEIGHT: u32 = 0;
static mut PITCH: u32 = 0;
static mut ISRGB: u32 = 0;

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color::from_rgba(r, g, b, u8::MAX)
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    pub fn red() -> Color {
        Color::from_rgb(255, 0, 0)
    }

    pub fn green() -> Color {
        Color::from_rgb(0, 255, 0)
    }

    pub fn blue() -> Color {
        Color::from_rgb(0, 0, 255)
    }
}

impl Into<u32> for &Color {
    fn into(self) -> u32 {
        (self.r as u32) | (self.g as u32) << 8 | (self.b as u32) << 16 | (self.a as u32) << 24
    }
}

fn mbox_set(i: usize, v: u32) {
    unsafe {
        let base = addr_of_mut!(MBOX.0) as *mut u32;
        write_volatile(base.add(i), v);
    }
}

fn mbox_get(i: usize) -> u32 {
    unsafe {
        let base = addr_of!(MBOX.0) as *const u32;
        read_volatile(base.add(i))
    }
}

fn mbox_call(ch: u32) -> bool {
    let addr = addr_of!(MBOX) as usize as u32;
    let r = (addr & !0xF) | (ch & 0xF);

    unsafe {
        while read_volatile(MBOX_STATUS as *const u32) & MBOX_FULL != 0 {
            core::hint::spin_loop();
        }
    }

    unsafe { write_volatile(MBOX_WRITE as *mut u32, r) }

    // Wait for a response to our specific message and confirm success.
    loop {
        unsafe {
            while read_volatile(MBOX_STATUS as *const u32) & MBOX_EMPTY != 0 {
                core::hint::spin_loop();
            }
        }
        unsafe {
            if read_volatile(MBOX_READ as *const u32) == r {
                return mbox_get(1) == MBOX_RESPONSE;
            }
        }
    }
}

pub fn init() -> bool {
    unsafe {
        mbox_set(0, 35 * 4); // total buffer size in bytes
        mbox_set(1, MBOX_REQUEST);

        mbox_set(2, 0x0004_8003); // tag: set physical width/height
        mbox_set(3, 8);
        mbox_set(4, 8);
        mbox_set(5, PHYSICAL_WIDTH);
        mbox_set(6, PHYSICAL_HEIGHT);

        mbox_set(7, 0x0004_8004); // tag: set virtual width/height
        mbox_set(8, 8);
        mbox_set(9, 8);
        mbox_set(10, PHYSICAL_WIDTH);
        mbox_set(11, PHYSICAL_HEIGHT);

        mbox_set(12, 0x0004_8009); // tag: set virtual offset
        mbox_set(13, 8);
        mbox_set(14, 8);
        mbox_set(15, 0);
        mbox_set(16, 0);

        mbox_set(17, 0x0004_8005); // tag: set depth
        mbox_set(18, 4);
        mbox_set(19, 4);
        mbox_set(20, 32);

        mbox_set(21, 0x0004_8006); // tag: set pixel order
        mbox_set(22, 4);
        mbox_set(23, 4);
        mbox_set(24, 1); // 1 = RGB

        mbox_set(25, 0x0004_0001); // tag: get framebuffer (asks for alignment)
        mbox_set(26, 8);
        mbox_set(27, 8);
        mbox_set(28, 4096); // request 4096-byte alignment; returns ptr here
        mbox_set(29, 0); //    returns size here

        mbox_set(30, 0x0004_0008); // tag: get pitch
        mbox_set(31, 4);
        mbox_set(32, 4);
        mbox_set(33, 0); // returns pitch here

        mbox_set(34, MBOX_TAG_LAST);

        if mbox_call(MBOX_CH_PROP) && mbox_get(20) == 32 && mbox_get(28) != 0 {
            // Convert the GPU bus address to an ARM physical address.
            let fb = mbox_get(28) & 0x3FFF_FFFF;
            FRAME_BUFFER = fb as usize;
            SIZE = mbox_get(29) as usize;
            WIDTH = mbox_get(5);
            HEIGHT = mbox_get(6);
            PITCH = mbox_get(33);
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
    let words = size / 4;
    for i in 0..words {
        unsafe {
            write_volatile((base + i * 4) as *mut u32, color.into());
        }
    }
}

/// Write a 32-bit ARGB color to a single pixel.
pub fn draw_pixel(x: u32, y: u32, color: &Color) {
    unsafe {
        if FRAME_BUFFER == 0 || x >= WIDTH || y >= HEIGHT {
            return;
        }
        let offset = (y * PITCH + x * 4) as usize;
        write_volatile((FRAME_BUFFER + offset) as *mut u32, color.into());
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
