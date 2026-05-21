// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! Processor code.

#[cfg(target_arch = "aarch64")]
#[path = "__arch__/aarch64/cpu.rs"]
mod arch_cpu;

//--------------------------------------------------------------------------------------------------
// Architectural Public Reexports
//--------------------------------------------------------------------------------------------------
pub use arch_cpu::wait_forever;
