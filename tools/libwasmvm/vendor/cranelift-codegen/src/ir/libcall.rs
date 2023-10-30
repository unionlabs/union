//! Naming well-known routines in the runtime library.

use crate::ir::{types, ExternalName, FuncRef, Function, Opcode, Type};
use core::fmt;
use core::str::FromStr;
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// The name of a runtime library routine.
///
/// Runtime library calls are generated for Cranelift IR instructions that don't have an equivalent
/// ISA instruction or an easy macro expansion. A `LibCall` is used as a well-known name to refer to
/// the runtime library routine. This way, Cranelift doesn't have to know about the naming
/// convention in the embedding VM's runtime library.
///
/// This list is likely to grow over time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum LibCall {
    /// probe for stack overflow. These are emitted for functions which need
    /// when the `enable_probestack` setting is true.
    Probestack,
    /// udiv.i64
    UdivI64,
    /// sdiv.i64
    SdivI64,
    /// urem.i64
    UremI64,
    /// srem.i64
    SremI64,
    /// ishl.i64
    IshlI64,
    /// ushr.i64
    UshrI64,
    /// sshr.i64
    SshrI64,
    /// ceil.f32
    CeilF32,
    /// ceil.f64
    CeilF64,
    /// floor.f32
    FloorF32,
    /// floor.f64
    FloorF64,
    /// trunc.f32
    TruncF32,
    /// frunc.f64
    TruncF64,
    /// nearest.f32
    NearestF32,
    /// nearest.f64
    NearestF64,
    /// libc.memcpy
    Memcpy,
    /// libc.memset
    Memset,
    /// libc.memmove
    Memmove,
    /// libc.memcmp
    Memcmp,

    /// Elf __tls_get_addr
    ElfTlsGetAddr,
    // When adding a new variant make sure to add it to `all_libcalls` too.
}

impl fmt::Display for LibCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl FromStr for LibCall {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Probestack" => Ok(Self::Probestack),
            "UdivI64" => Ok(Self::UdivI64),
            "SdivI64" => Ok(Self::SdivI64),
            "UremI64" => Ok(Self::UremI64),
            "SremI64" => Ok(Self::SremI64),
            "IshlI64" => Ok(Self::IshlI64),
            "UshrI64" => Ok(Self::UshrI64),
            "SshrI64" => Ok(Self::SshrI64),
            "CeilF32" => Ok(Self::CeilF32),
            "CeilF64" => Ok(Self::CeilF64),
            "FloorF32" => Ok(Self::FloorF32),
            "FloorF64" => Ok(Self::FloorF64),
            "TruncF32" => Ok(Self::TruncF32),
            "TruncF64" => Ok(Self::TruncF64),
            "NearestF32" => Ok(Self::NearestF32),
            "NearestF64" => Ok(Self::NearestF64),
            "Memcpy" => Ok(Self::Memcpy),
            "Memset" => Ok(Self::Memset),
            "Memmove" => Ok(Self::Memmove),
            "Memcmp" => Ok(Self::Memcmp),

            "ElfTlsGetAddr" => Ok(Self::ElfTlsGetAddr),
            _ => Err(()),
        }
    }
}

impl LibCall {
    /// Get the well-known library call name to use as a replacement for an instruction with the
    /// given opcode and controlling type variable.
    ///
    /// Returns `None` if no well-known library routine name exists for that instruction.
    pub fn for_inst(opcode: Opcode, ctrl_type: Type) -> Option<Self> {
        Some(match ctrl_type {
            types::I64 => match opcode {
                Opcode::Udiv => Self::UdivI64,
                Opcode::Sdiv => Self::SdivI64,
                Opcode::Urem => Self::UremI64,
                Opcode::Srem => Self::SremI64,
                Opcode::Ishl => Self::IshlI64,
                Opcode::Ushr => Self::UshrI64,
                Opcode::Sshr => Self::SshrI64,
                _ => return None,
            },
            types::F32 => match opcode {
                Opcode::Ceil => Self::CeilF32,
                Opcode::Floor => Self::FloorF32,
                Opcode::Trunc => Self::TruncF32,
                Opcode::Nearest => Self::NearestF32,
                _ => return None,
            },
            types::F64 => match opcode {
                Opcode::Ceil => Self::CeilF64,
                Opcode::Floor => Self::FloorF64,
                Opcode::Trunc => Self::TruncF64,
                Opcode::Nearest => Self::NearestF64,
                _ => return None,
            },
            _ => return None,
        })
    }

    /// Get a list of all known `LibCall`'s.
    pub fn all_libcalls() -> &'static [LibCall] {
        use LibCall::*;
        &[
            Probestack,
            UdivI64,
            SdivI64,
            UremI64,
            SremI64,
            IshlI64,
            UshrI64,
            SshrI64,
            CeilF32,
            CeilF64,
            FloorF32,
            FloorF64,
            TruncF32,
            TruncF64,
            NearestF32,
            NearestF64,
            Memcpy,
            Memset,
            Memmove,
            Memcmp,
            ElfTlsGetAddr,
        ]
    }
}

/// Get a function reference for the probestack function in `func`.
///
/// If there is an existing reference, use it, otherwise make a new one.
pub fn get_probestack_funcref(func: &mut Function) -> Option<FuncRef> {
    find_funcref(LibCall::Probestack, func)
}

/// Get the existing function reference for `libcall` in `func` if it exists.
fn find_funcref(libcall: LibCall, func: &Function) -> Option<FuncRef> {
    // We're assuming that all libcall function decls are at the end.
    // If we get this wrong, worst case we'll have duplicate libcall decls which is harmless.
    for (fref, func_data) in func.dfg.ext_funcs.iter().rev() {
        match func_data.name {
            ExternalName::LibCall(lc) => {
                if lc == libcall {
                    return Some(fref);
                }
            }
            _ => break,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn display() {
        assert_eq!(LibCall::CeilF32.to_string(), "CeilF32");
        assert_eq!(LibCall::NearestF64.to_string(), "NearestF64");
    }

    #[test]
    fn parsing() {
        assert_eq!("FloorF32".parse(), Ok(LibCall::FloorF32));
    }

    #[test]
    fn all_libcalls_to_from_string() {
        for &libcall in LibCall::all_libcalls() {
            assert_eq!(libcall.to_string().parse(), Ok(libcall));
        }
    }
}
