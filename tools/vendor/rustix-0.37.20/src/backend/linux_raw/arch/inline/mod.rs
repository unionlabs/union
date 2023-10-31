//! Inline asm for making system calls.
//!
//! Compilers should really have intrinsics for making system calls. They're
//! much like regular calls, with custom calling conventions, and calling
//! conventions are otherwise the compiler's job. But for now, use inline asm.
//!
//! The calling conventions for Linux syscalls are [documented here].
//!
//! [documented here]: https://man7.org/linux/man-pages/man2/syscall.2.html

#[cfg_attr(target_arch = "aarch64", path = "aarch64.rs")]
#[cfg_attr(all(target_arch = "arm", not(thumb_mode)), path = "arm.rs")]
#[cfg_attr(all(target_arch = "arm", thumb_mode), path = "thumb.rs")]
#[cfg_attr(target_arch = "mips", path = "mips.rs")]
#[cfg_attr(target_arch = "mips64", path = "mips64.rs")]
#[cfg_attr(target_arch = "powerpc64", path = "powerpc64.rs")]
#[cfg_attr(target_arch = "riscv64", path = "riscv64.rs")]
#[cfg_attr(target_arch = "x86", path = "x86.rs")]
#[cfg_attr(target_arch = "x86_64", path = "x86_64.rs")]
mod target_arch;

pub(in crate::backend) use self::target_arch::*;
