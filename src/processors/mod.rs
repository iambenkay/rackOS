mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::cpu::wait_forever;
