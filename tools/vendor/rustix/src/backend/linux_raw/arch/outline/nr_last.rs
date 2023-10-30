//! Syscall wrappers for platforms which pass the syscall number specially.
//!
//! Rustix aims to minimize the amount of assembly code it needs. To that end,
//! this code reorders syscall arguments as close as feasible to the actual
//! syscall convention before calling the assembly functions.
//!
//! Many architectures use a convention where the syscall number is passed in a
//! special register, with the regular syscall arguments passed in either the
//! same or similar registers as the platform C convention. This code
//! approximates that order by passing the regular syscall arguments first, and
//! the syscall number last. That way, the outline assembly code typically just
//! needs to move the syscall number to its special register, and leave the
//! other arguments mostly as they are.

#[cfg(target_arch = "mips")]
use crate::backend::reg::A6;
use crate::backend::reg::{ArgReg, RetReg, SyscallNumber, A0, A1, A2, A3, A4, A5, R0};

// First we declare the actual assembly routines with `*_nr_last` names and
// reordered arguments. If the signatures or calling conventions are ever
// changed, the symbol names should also be updated accordingly, to avoid
// collisions with other versions of this crate.
//
// We don't define `_readonly` versions of these because we have no way to tell
// Rust that calls to our outline assembly are readonly.
extern "C" {
    fn rustix_syscall0_nr_last(nr: SyscallNumber<'_>) -> RetReg<R0>;
    fn rustix_syscall1_nr_last(a0: ArgReg<'_, A0>, nr: SyscallNumber<'_>) -> RetReg<R0>;
    fn rustix_syscall1_noreturn_nr_last(a0: ArgReg<'_, A0>, nr: SyscallNumber<'_>) -> !;
    fn rustix_syscall2_nr_last(
        a0: ArgReg<'_, A0>,
        a1: ArgReg<'_, A1>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    fn rustix_syscall3_nr_last(
        a0: ArgReg<'_, A0>,
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    fn rustix_syscall4_nr_last(
        a0: ArgReg<'_, A0>,
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a3: ArgReg<'_, A3>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    fn rustix_syscall5_nr_last(
        a0: ArgReg<'_, A0>,
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a3: ArgReg<'_, A3>,
        a4: ArgReg<'_, A4>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    fn rustix_syscall6_nr_last(
        a0: ArgReg<'_, A0>,
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a3: ArgReg<'_, A3>,
        a4: ArgReg<'_, A4>,
        a5: ArgReg<'_, A5>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    #[cfg(target_arch = "mips")]
    fn rustix_syscall7_nr_last(
        a0: ArgReg<'_, A0>,
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a3: ArgReg<'_, A3>,
        a4: ArgReg<'_, A4>,
        a5: ArgReg<'_, A5>,
        a6: ArgReg<'_, A6>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
}

// Then we define inline wrapper functions that do the reordering.

#[inline]
pub(in crate::backend) unsafe fn syscall0(nr: SyscallNumber<'_>) -> RetReg<R0> {
    rustix_syscall0_nr_last(nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall1(nr: SyscallNumber<'_>, a0: ArgReg<'_, A0>) -> RetReg<R0> {
    rustix_syscall1_nr_last(a0, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall1_noreturn(nr: SyscallNumber<'_>, a0: ArgReg<'_, A0>) -> ! {
    rustix_syscall1_noreturn_nr_last(a0, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall2(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
) -> RetReg<R0> {
    rustix_syscall2_nr_last(a0, a1, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall3(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
) -> RetReg<R0> {
    rustix_syscall3_nr_last(a0, a1, a2, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall4(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
    a3: ArgReg<'_, A3>,
) -> RetReg<R0> {
    rustix_syscall4_nr_last(a0, a1, a2, a3, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall5(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
    a3: ArgReg<'_, A3>,
    a4: ArgReg<'_, A4>,
) -> RetReg<R0> {
    rustix_syscall5_nr_last(a0, a1, a2, a3, a4, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall6(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
    a3: ArgReg<'_, A3>,
    a4: ArgReg<'_, A4>,
    a5: ArgReg<'_, A5>,
) -> RetReg<R0> {
    rustix_syscall6_nr_last(a0, a1, a2, a3, a4, a5, nr)
}
#[cfg(target_arch = "mips")]
#[inline]
pub(in crate::backend) unsafe fn syscall7(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
    a3: ArgReg<'_, A3>,
    a4: ArgReg<'_, A4>,
    a5: ArgReg<'_, A5>,
    a6: ArgReg<'_, A6>,
) -> RetReg<R0> {
    rustix_syscall7_nr_last(a0, a1, a2, a3, a4, a5, a6, nr)
}

// Then we define the `_readonly` versions of the wrappers. We don't have
// separate `_readonly` implementations, so these can just be aliases to
// their non-`_readonly` counterparts.
#[cfg(target_arch = "mips")]
pub(in crate::backend) use syscall7 as syscall7_readonly;
pub(in crate::backend) use {
    syscall0 as syscall0_readonly, syscall1 as syscall1_readonly, syscall2 as syscall2_readonly,
    syscall3 as syscall3_readonly, syscall4 as syscall4_readonly, syscall5 as syscall5_readonly,
    syscall6 as syscall6_readonly,
};
