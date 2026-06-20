use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};

#[cfg(feature = "rpi5")]
pub const VIDEOCORE_MBOX: usize = 0x10_7C01_3880;

#[cfg(feature = "rpi4")]
pub const VIDEOCORE_MBOX: usize = 0xFE00_B880;

const MBOX_RESPONSE: u32 = 0x8000_0000;
const MBOX_FULL: u32 = 0x8000_0000;
const MBOX_EMPTY: u32 = 0x4000_0000;

struct Flags;

impl Flags {
    fn read() -> *const u32 {
        (VIDEOCORE_MBOX + 0x00) as *const u32
    }

    fn status() -> *const u32 {
        (VIDEOCORE_MBOX + 0x18) as *const u32
    }

    fn write() -> *mut u32 {
        (VIDEOCORE_MBOX + 0x20) as *mut u32
    }
}

/// This is allocated for mailbox request-response flow.
/// The mailbox interface is the recommended way for interacting with VideoCore.
/// Without it, drawing to the screen would be needlessly complex
#[repr(C, align(16))]
struct Mailbox([u32; 36]);

impl Mailbox {
    const fn new() -> Mailbox {
        Mailbox([0; 36])
    }
}

static mut MBOX: Mailbox = Mailbox::new();

/// set value at mailbox index
pub fn mbox_set(i: usize, v: u32) {
    unsafe {
        let base = addr_of_mut!(MBOX.0) as *mut u32;
        write_volatile(base.add(i), v);
    }
}

/// get value at mailbox index
pub fn mbox_get(i: usize) -> u32 {
    unsafe {
        let base = addr_of!(MBOX.0) as *const u32;
        read_volatile(base.add(i))
    }
}

/// Send mailbox request on channel `channel` and wait for response
pub fn mbox_call(channel: u32) -> bool {
    let addr = addr_of!(MBOX) as usize as u32;
    let channel = (addr & !0xF) | (channel & 0xF);

    while is_mbox_full() {
        core::hint::spin_loop();
    }

    write_mbox_request(channel);

    loop {
        while is_mbox_empty() {
            core::hint::spin_loop();
        }
        if does_mbox_response_match(channel) {
            return mbox_get(1) == MBOX_RESPONSE;
        }
    }
}

fn is_mbox_full() -> bool {
    unsafe { (read_volatile(Flags::status()) & MBOX_FULL) != 0 }
}

fn is_mbox_empty() -> bool {
    unsafe { (read_volatile(Flags::status()) & MBOX_EMPTY) != 0 }
}

fn write_mbox_request(request: u32) {
    unsafe { write_volatile(Flags::write(), request) }
}

fn does_mbox_response_match(request: u32) -> bool {
    unsafe { read_volatile(Flags::read()) == request }
}
