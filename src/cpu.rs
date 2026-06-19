#[cfg(target_arch = "aarch64")]
#[path = "__arch__/aarch64/cpu.rs"]
mod arch_cpu;

pub use arch_cpu::wait_forever;
