mod aarch64;

pub fn wait_forever() -> ! {
    #[cfg(target_arch = "aarch64")]
    aarch64::cpu::wait_forever()
}
