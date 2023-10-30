//! Runtime support for the x64 architecture assembling target.
//!
//! The x64 instruction set features variable-length instructions and
//! relative relocations up to 32 bits in size.
//!
//! The core relocation behaviour for this architecture is provided by the [`X64Relocation`] type.
//!
//! Next to that, this module contains the following:
//!
//! ## Type aliases
//!
//! Several specialized type aliases of the generic [`Assembler`] are provided as these are by far the most common usecase.
//!
//! ## Enums
//!
//! There are enumerator of every logically distinct register family usable in x64. 
//! These enums implement the [`Register`] trait and their discriminant values match their numeric encoding in dynamic register literals.
//! Some of these are re-exported from the x86 architecture.
//!
//! *Note: The presence of some registers listed here is purely what is encodable. Check the relevant architecture documentation to find what is architecturally valid.*

use crate::relocations::{Relocation, RelocationSize, RelocationKind, ImpossibleRelocation};
use crate::Register;

use std::hash::Hash;


/// Relocation implementation for the x64 architecture.
#[derive(Debug, Clone)]
pub struct X64Relocation {
    size: RelocationSize,
}

impl Relocation for X64Relocation {
    type Encoding = (u8,);
    fn from_encoding(encoding: Self::Encoding) -> Self {
        Self {
            size: RelocationSize::from_encoding(encoding.0),
        }
    }
    fn from_size(size: RelocationSize) -> Self {
        Self {
            size,
        }
    }
    fn size(&self) -> usize {
        self.size.size()
    }
    fn write_value(&self, buf: &mut [u8], value: isize) -> Result<(), ImpossibleRelocation> {
        self.size.write_value(buf, value)
    }
    fn read_value(&self, buf: &[u8]) -> isize {
        self.size.read_value(buf)
    }
    fn kind(&self) -> RelocationKind {
        RelocationKind::Relative
    }
    fn page_size() -> usize {
        4096
    }
}

/// An x64 Assembler. This is aliased here for backwards compatability.
pub type Assembler = crate::Assembler<X64Relocation>;
/// An x64 AssemblyModifier. This is aliased here for backwards compatability.
pub type AssemblyModifier<'a> = crate::Modifier<'a, X64Relocation>;
/// An x64 UncommittedModifier. This is aliased here for backwards compatability.
pub type UncommittedModifier<'a> = crate::UncommittedModifier<'a>;


/// 1, 2, 4 or 8-byte general purpose "quad-word" registers.
///
/// RIP does not appear here as it cannot be addressed dynamically.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rq {
    RAX = 0x0, RCX = 0x1, RDX = 0x2, RBX = 0x3,
    RSP = 0x4, RBP = 0x5, RSI = 0x6, RDI = 0x7,
    R8  = 0x8, R9  = 0x9, R10 = 0xA, R11 = 0xB,
    R12 = 0xC, R13 = 0xD, R14 = 0xE, R15 = 0xF,
}
reg_impls!(Rq);

/// 16 or 32-byte SSE registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rx {
    XMM0  = 0x0, XMM1  = 0x1, XMM2  = 0x2, XMM3  = 0x3,
    XMM4  = 0x4, XMM5  = 0x5, XMM6  = 0x6, XMM7  = 0x7,
    XMM8  = 0x8, XMM9  = 0x9, XMM10 = 0xA, XMM11 = 0xB,
    XMM12 = 0xC, XMM13 = 0xD, XMM14 = 0xE, XMM15 = 0xF,
}
reg_impls!(Rx);

/// 8-byte control registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RC {
    CR0  = 0x0, CR1  = 0x1, CR2  = 0x2, CR3  = 0x3,
    CR4  = 0x4, CR5  = 0x5, CR6  = 0x6, CR7  = 0x7,
    CR8  = 0x8, CR9  = 0x9, CR10 = 0xA, CR11 = 0xB,
    CR12 = 0xC, CR13 = 0xD, CR14 = 0xE, CR15 = 0xF,
}
reg_impls!(RC);

// The other register families are the same as 32-bit X86. (Although access size for Debug regs is 8-byte)
pub use crate::x86::{Rh, Rf, Rm, Rs, RD, RB};

#[cfg(test)]
mod tests {
    use super::Rq::*;
    use crate::Register;

    #[test]
    fn reg_code() {
        assert_eq!(RAX.code(), 0);
    }

    #[test]
    fn reg_code_from() {
        assert_eq!(u8::from(R11), 11);
    }
}
