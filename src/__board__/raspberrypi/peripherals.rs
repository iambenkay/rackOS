const PERIPHERAL_BASE: u32 = 0x4000_0000;

fn mmio_write(addr: u32, value: u32) {
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, value);
    }
}

fn mmio_read(addr: u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}
