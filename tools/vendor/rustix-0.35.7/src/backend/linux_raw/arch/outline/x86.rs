//! Syscall wrappers for 32-bit x86.
//!
//! This module is similar to the `nr_last` module, except specialized for
//! 32-bit x86.
//!
//! The syscall convention passes all arguments in registers. The closest we
//! can easily get to that from Rust is to use the fastcall convention which
//! passes the first two arguments in `ecx` and `edx`, which are the second
//! and third Linux syscall arguments. To line them up, this function passes
//! the second and third syscall argument as the first and second argument to
//! the outline assembly, followed by the first syscall argument, and then the
//! rest of the syscall arguments. The assembly code still has to do some work,
//! but at least we can get up to two arguments into the right place for it.

#![allow(dead_code, unused_imports)]

use crate::backend::reg::{ArgReg, RetReg, SyscallNumber, A0, A1, A2, A3, A4, A5, R0};
use crate::backend::vdso_wrappers::SyscallType;

// First we declare the actual assembly routines with `*_nr_last_fastcall`
// names and reordered arguments. If the signatures or calling conventions are
// ever changed, the symbol names should also be updated accordingly, to avoid
// collisions with other versions of this crate.
//
// We don't define `_readonly` versions of these because we have no way to tell
// Rust that calls to our outline assembly are readonly.
extern "fastcall" {
    fn rustix_syscall0_nr_last_fastcall(nr: SyscallNumber<'_>) -> RetReg<R0>;
    fn rustix_syscall1_nr_last_fastcall(a0: ArgReg<'_, A0>, nr: SyscallNumber<'_>) -> RetReg<R0>;
    fn rustix_syscall1_noreturn_nr_last_fastcall(a0: ArgReg<'_, A0>, nr: SyscallNumber<'_>) -> !;
    fn rustix_syscall2_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a0: ArgReg<'_, A0>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    fn rustix_syscall3_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a0: ArgReg<'_, A0>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    fn rustix_syscall4_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a0: ArgReg<'_, A0>,
        a3: ArgReg<'_, A3>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    fn rustix_syscall5_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a0: ArgReg<'_, A0>,
        a3: ArgReg<'_, A3>,
        a4: ArgReg<'_, A4>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
    fn rustix_syscall6_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a0: ArgReg<'_, A0>,
        a3: ArgReg<'_, A3>,
        a4: ArgReg<'_, A4>,
        a5: ArgReg<'_, A5>,
        nr: SyscallNumber<'_>,
    ) -> RetReg<R0>;
}

// Then we define inline wrapper functions that do the reordering.

#[inline]
pub(in crate::backend) unsafe fn syscall0(nr: SyscallNumber<'_>) -> RetReg<R0> {
    rustix_syscall0_nr_last_fastcall(nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall1(nr: SyscallNumber<'_>, a0: ArgReg<'_, A0>) -> RetReg<R0> {
    rustix_syscall1_nr_last_fastcall(a0, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall1_noreturn(nr: SyscallNumber<'_>, a0: ArgReg<'_, A0>) -> ! {
    rustix_syscall1_noreturn_nr_last_fastcall(a0, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall2(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
) -> RetReg<R0> {
    rustix_syscall2_nr_last_fastcall(a1, a0, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall3(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
) -> RetReg<R0> {
    rustix_syscall3_nr_last_fastcall(a1, a2, a0, nr)
}
#[inline]
pub(in crate::backend) unsafe fn syscall4(
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
    a3: ArgReg<'_, A3>,
) -> RetReg<R0> {
    rustix_syscall4_nr_last_fastcall(a1, a2, a0, a3, nr)
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
    rustix_syscall5_nr_last_fastcall(a1, a2, a0, a3, a4, nr)
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
    rustix_syscall6_nr_last_fastcall(a1, a2, a0, a3, a4, a5, nr)
}

// Then we define the `_readonly` versions of the wrappers. We don't have
// separate `_readonly` implementations, so these can just be aliases to
// their non-`_readonly` counterparts.
pub(in crate::backend) use {
    syscall0 as syscall0_readonly, syscall1 as syscall1_readonly, syscall2 as syscall2_readonly,
    syscall3 as syscall3_readonly, syscall4 as syscall4_readonly, syscall5 as syscall5_readonly,
    syscall6 as syscall6_readonly,
};

// x86 prefers to route all syscalls through the vDSO, though this isn't
// always possible, so it also has a special form for doing the dispatch.
//
// First we declare the actual assembly routines with `*_nr_last_fastcall`
// names and reordered arguments. If the signatures or calling conventions are
// ever changed, the symbol names should also be updated accordingly, to avoid
// collisions with other versions of this crate.
extern "fastcall" {
    fn rustix_indirect_syscall0_nr_last_fastcall(
        nr: SyscallNumber<'_>,
        callee: SyscallType,
    ) -> RetReg<R0>;
    fn rustix_indirect_syscall1_nr_last_fastcall(
        a0: ArgReg<'_, A0>,
        nr: SyscallNumber<'_>,
        callee: SyscallType,
    ) -> RetReg<R0>;
    fn rustix_indirect_syscall1_noreturn_nr_last_fastcall(
        a0: ArgReg<'_, A0>,
        nr: SyscallNumber<'_>,
        callee: SyscallType,
    ) -> !;
    fn rustix_indirect_syscall2_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a0: ArgReg<'_, A0>,
        nr: SyscallNumber<'_>,
        callee: SyscallType,
    ) -> RetReg<R0>;
    fn rustix_indirect_syscall3_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a0: ArgReg<'_, A0>,
        nr: SyscallNumber<'_>,
        callee: SyscallType,
    ) -> RetReg<R0>;
    fn rustix_indirect_syscall4_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a0: ArgReg<'_, A0>,
        a3: ArgReg<'_, A3>,
        nr: SyscallNumber<'_>,
        callee: SyscallType,
    ) -> RetReg<R0>;
    fn rustix_indirect_syscall5_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a0: ArgReg<'_, A0>,
        a3: ArgReg<'_, A3>,
        a4: ArgReg<'_, A4>,
        nr: SyscallNumber<'_>,
        callee: SyscallType,
    ) -> RetReg<R0>;
    fn rustix_indirect_syscall6_nr_last_fastcall(
        a1: ArgReg<'_, A1>,
        a2: ArgReg<'_, A2>,
        a0: ArgReg<'_, A0>,
        a3: ArgReg<'_, A3>,
        a4: ArgReg<'_, A4>,
        a5: ArgReg<'_, A5>,
        nr: SyscallNumber<'_>,
        callee: SyscallType,
    ) -> RetReg<R0>;
}

// Then we define inline wrapper functions that do the reordering.

#[inline]
pub(in crate::backend) unsafe fn indirect_syscall0(
    callee: SyscallType,
    nr: SyscallNumber<'_>,
) -> RetReg<R0> {
    rustix_indirect_syscall0_nr_last_fastcall(nr, callee)
}
#[inline]
pub(in crate::backend) unsafe fn indirect_syscall1(
    callee: SyscallType,
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
) -> RetReg<R0> {
    rustix_indirect_syscall1_nr_last_fastcall(a0, nr, callee)
}
#[inline]
pub(in crate::backend) unsafe fn indirect_syscall1_noreturn(
    callee: SyscallType,
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
) -> ! {
    rustix_indirect_syscall1_noreturn_nr_last_fastcall(a0, nr, callee)
}
#[inline]
pub(in crate::backend) unsafe fn indirect_syscall2(
    callee: SyscallType,
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
) -> RetReg<R0> {
    rustix_indirect_syscall2_nr_last_fastcall(a1, a0, nr, callee)
}
#[inline]
pub(in crate::backend) unsafe fn indirect_syscall3(
    callee: SyscallType,
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
) -> RetReg<R0> {
    rustix_indirect_syscall3_nr_last_fastcall(a1, a2, a0, nr, callee)
}
#[inline]
pub(in crate::backend) unsafe fn indirect_syscall4(
    callee: SyscallType,
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
    a3: ArgReg<'_, A3>,
) -> RetReg<R0> {
    rustix_indirect_syscall4_nr_last_fastcall(a1, a2, a0, a3, nr, callee)
}
#[inline]
pub(in crate::backend) unsafe fn indirect_syscall5(
    callee: SyscallType,
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
    a3: ArgReg<'_, A3>,
    a4: ArgReg<'_, A4>,
) -> RetReg<R0> {
    rustix_indirect_syscall5_nr_last_fastcall(a1, a2, a0, a3, a4, nr, callee)
}
#[inline]
pub(in crate::backend) unsafe fn indirect_syscall6(
    callee: SyscallType,
    nr: SyscallNumber<'_>,
    a0: ArgReg<'_, A0>,
    a1: ArgReg<'_, A1>,
    a2: ArgReg<'_, A2>,
    a3: ArgReg<'_, A3>,
    a4: ArgReg<'_, A4>,
    a5: ArgReg<'_, A5>,
) -> RetReg<R0> {
    rustix_indirect_syscall6_nr_last_fastcall(a1, a2, a0, a3, a4, a5, nr, callee)
}
