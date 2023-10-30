//! Inline asm for making system calls.
//!
//! Compilers should really have intrinsics for making system calls. They're
//! much like regular calls, with custom calling conventions, and calling
//! conventions are otherwise the compiler's job. But for now, use inline asm.

#[cfg_attr(target_arch = "aarch64", path = "aarch64.rs")]
#[cfg_attr(target_arch = "arm", path = "arm.rs")]
#[cfg_attr(target_arch = "mips", path = "mips.rs")]
#[cfg_attr(target_arch = "mips64", path = "mips64.rs")]
#[cfg_attr(target_arch = "powerpc64", path = "powerpc64.rs")]
#[cfg_attr(target_arch = "riscv64", path = "riscv64.rs")]
#[cfg_attr(target_arch = "x86", path = "x86.rs")]
#[cfg_attr(target_arch = "x86_64", path = "x86_64.rs")]
mod target_arch;

pub(in crate::backend) use self::target_arch::*;
