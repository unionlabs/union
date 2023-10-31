use core::{mem, slice};
use core::ops::{Deref, DerefMut};

use super::error::{Error, Result};

macro_rules! syscall {
    ($($name:ident($a:ident, $($b:ident, $($c:ident, $($d:ident, $($e:ident, $($f:ident, )?)?)?)?)?);)+) => {
        $(
            pub unsafe fn $name($a: usize, $($b: usize, $($c: usize, $($d: usize, $($e: usize, $($f: usize)?)?)?)?)?) -> Result<usize> {
                let ret: usize;

                asm!(
                    "ecall",
                    in("a7") $a,
                    $(
                        in("a0") $b,
                        $(
                            in("a1") $c,
                            $(
                                in("a2") $d,
                                $(
                                    in("a3") $e,
                                    $(
                                        in("a4") $f,
                                    )?
                                )?
                            )?
                        )?
                    )?
                    lateout("a0") ret,
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
    //TODO
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
    //TODO
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
