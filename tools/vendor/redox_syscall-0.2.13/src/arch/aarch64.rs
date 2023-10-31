use core::{mem, slice};
use core::ops::{Deref, DerefMut};

use super::error::{Error, Result};

macro_rules! syscall {
    ($($name:ident($a:ident, $($b:ident, $($c:ident, $($d:ident, $($e:ident, $($f:ident, )?)?)?)?)?);)+) => {
        $(
            pub unsafe fn $name($a: usize, $($b: usize, $($c: usize, $($d: usize, $($e: usize, $($f: usize)?)?)?)?)?) -> Result<usize> {
                let ret: usize;

                core::arch::asm!(
                    "svc 0",
                    in("x8") $a,
                    $(
                        in("x0") $b,
                        $(
                            in("x1") $c,
                            $(
                                in("x2") $d,
                                $(
                                    in("x3") $e,
                                    $(
                                        in("x4") $f,
                                    )?
                                )?
                            )?
                        )?
                    )?
                    lateout("x0") ret,
                    options(nostack),
                );

                Error::demux(ret)
            }
        )+
    };
}

syscall! {
    syscall0(a,);
    syscall1(a, b,);
    syscall2(a, b, c,);
    syscall3(a, b, c, d,);
    syscall4(a, b, c, d, e,);
    syscall5(a, b, c, d, e, f,);
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct IntRegisters {
    pub elr_el1: usize,
    pub tpidr_el0: usize,
    pub tpidrro_el0: usize,
    pub spsr_el1: usize,
    pub esr_el1: usize,
    pub sp_el0: usize,      // Shouldn't be used if interrupt occurred at EL1
    pub padding: usize,     // To keep the struct even number aligned
    pub x30: usize,
    pub x29: usize,
    pub x28: usize,
    pub x27: usize,
    pub x26: usize,
    pub x25: usize,
    pub x24: usize,
    pub x23: usize,
    pub x22: usize,
    pub x21: usize,
    pub x20: usize,
    pub x19: usize,
    pub x18: usize,
    pub x17: usize,
    pub x16: usize,
    pub x15: usize,
    pub x14: usize,
    pub x13: usize,
    pub x12: usize,
    pub x11: usize,
    pub x10: usize,
    pub x9: usize,
    pub x8: usize,
    pub x7: usize,
    pub x6: usize,
    pub x5: usize,
    pub x4: usize,
    pub x3: usize,
    pub x2: usize,
    pub x1: usize,
    pub x0: usize
}

impl Deref for IntRegisters {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self as *const IntRegisters as *const u8, mem::size_of::<IntRegisters>())
        }
    }
}

impl DerefMut for IntRegisters {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self as *mut IntRegisters as *mut u8, mem::size_of::<IntRegisters>())
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(packed)]
pub struct FloatRegisters {
    pub fp_simd_regs: [u128; 32],
    pub fpsr: u32,
    pub fpcr: u32
}

impl Deref for FloatRegisters {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self as *const FloatRegisters as *const u8, mem::size_of::<FloatRegisters>())
        }
    }
}

impl DerefMut for FloatRegisters {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self as *mut FloatRegisters as *mut u8, mem::size_of::<FloatRegisters>())
        }
    }
}
