use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    crate::processors::wait_forever()
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
