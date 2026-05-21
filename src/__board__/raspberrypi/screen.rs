use crate::io;

#[repr(C, align(16))]
struct Mailbox {
    data: [u32; 36],
}

impl Mailbox {
    const fn new() -> Mailbox {
        Mailbox { data: [0; 36] }
    }
}

static mut MBOX: Mailbox = Mailbox::new();
