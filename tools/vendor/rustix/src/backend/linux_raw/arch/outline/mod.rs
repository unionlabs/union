//! Declare functions defined in out-of-line ("outline") asm files.
//!
//! Kernel calling conventions differ from userspace calling conventions, so we
//! also define inline function wrappers which reorder the arguments so that
//! they match with the kernel convention as closely as possible, to minimize
//! the amount of out-of-line code we need.
//!
//! This is needed because as of our MSRV of 1.63, inline asm and naked
//! functions are experimental.

#[cfg(target_arch = "x86")]
mod x86;
// For these architectures, pass the `nr` argument last.
#[cfg(any(
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "powerpc64",
))]
mod nr_last;

#[cfg(any(
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "powerpc64",
))]
pub(in crate::backend) use nr_last::*;
#[cfg(target_arch = "x86")]
pub(in crate::backend) use x86::*;
