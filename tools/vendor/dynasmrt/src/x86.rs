//! Runtime support for the x86 architecture assembling target.
//!
//! The x86 instruction set features variable-length instructions and
//! relative relocations up to 16 bits in size, or absolute relocations of 32 bits in size.
//!
//! The core relocation behaviour for this architecture is provided by the [`X86Relocation`] type.
//!
//! Next to that, this module contains the following:
//!
//! ## Type aliases
//!
//! Several specialized type aliases of the generic [`Assembler`] are provided as these are by far the most common usecase.
//!
//! ## Enums
//!
//! There are enumerator of every logically distinct register family usable in x86. 
//! These enums implement the [`Register`] trait and their discriminant values match their numeric encoding in dynamic register literals.
//!
//! *Note: The presence of some registers listed here is purely what is encodable. Check the relevant architecture documentation to find what is architecturally valid.*


use crate::Register;
use crate::relocations::{Relocation, RelocationSize, RelocationKind, ImpossibleRelocation};


/// Relocation implementation for the x86 architecture.
#[derive(Debug, Clone)]
pub struct X86Relocation {
    size: RelocationSize,
    kind: RelocationKind,
}

impl Relocation for X86Relocation {
    type Encoding = (u8, u8);
    fn from_encoding(encoding: Self::Encoding) -> Self {
        Self {
            size: RelocationSize::from_encoding(encoding.0),
            kind: RelocationKind::from_encoding(encoding.1),
        }
    }
    fn from_size(size: RelocationSize) -> Self {
        Self {
            size,
            kind: RelocationKind::Relative,
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
        self.kind
    }
    fn page_size() -> usize {
        4096
    }
}


/// An x86 Assembler. This is aliased here for backwards compatability.
pub type Assembler = crate::Assembler<X86Relocation>;
/// An x86 AssemblyModifier. This is aliased here for backwards compatability.
pub type AssemblyModifier<'a> = crate::Modifier<'a, X86Relocation>;
/// An x86 UncommittedModifier. This is aliased here for backwards compatability.
pub type UncommittedModifier<'a> = crate::UncommittedModifier<'a>;


/// 1, 2 or 4-byte general purpose "double-word" registers.
///
/// EIP does not appear here as it cannot be addressed dynamically.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rd {
    EAX = 0x00, ECX = 0x01, EDX = 0x02, EBX = 0x03,
    ESP = 0x04, EBP = 0x05, ESI = 0x06, EDI = 0x07,
}
reg_impls!(Rd);

/// High-byte general purpose registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rh {
    AH = 0x4, CH = 0x5, DH = 0x6, BH = 0x7,
}
reg_impls!(Rh);

/// 10-byte floating point registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rf {
    ST0 = 0x0, ST1 = 0x1, ST2 = 0x2, ST3 = 0x3,
    ST4 = 0x4, ST5 = 0x5, ST6 = 0x6, ST7 = 0x7,
}
reg_impls!(Rf);

/// 8-byte MMX registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rm {
    MMX0 = 0x0, MMX1 = 0x1, MMX2 = 0x2, MMX3 = 0x3,
    MMX4 = 0x4, MMX5 = 0x5, MMX6 = 0x6, MMX7 = 0x7,
}
reg_impls!(Rm);

/// 16 or 32-byte SSE registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rx {
    XMM0  = 0x0, XMM1  = 0x1, XMM2  = 0x2, XMM3  = 0x3,
    XMM4  = 0x4, XMM5  = 0x5, XMM6  = 0x6, XMM7  = 0x7,
}
reg_impls!(Rx);

/// 2-byte segment registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rs {
    ES = 0x0, CS = 0x1, SS = 0x2, DS = 0x3,
    FS = 0x4, GS = 0x5,
}
reg_impls!(Rs);

/// 4-byte control registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RC {
    CR0  = 0x0, CR1  = 0x1, CR2  = 0x2, CR3  = 0x3,
    CR4  = 0x4, CR5  = 0x5, CR6  = 0x6, CR7  = 0x7,
}
reg_impls!(RC);

/// 4-byte debug registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RD {
    DR0  = 0x0, DR1  = 0x1, DR2  = 0x2, DR3  = 0x3,
    DR4  = 0x4, DR5  = 0x5, DR6  = 0x6, DR7  = 0x7,
    DR8  = 0x8, DR9  = 0x9, DR10 = 0xA, DR11 = 0xB,
    DR12 = 0xC, DR13 = 0xD, DR14 = 0xE, DR15 = 0xF,
}
reg_impls!(RD);

/// 16-byte bound registers.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RB {
    BND0 = 0x0, BND1 = 0x1, BND2 = 0x2, BND3 = 0x3
}
reg_impls!(RB);

#[cfg(test)]
mod tests {
    use super::Rd::*;
    use crate::Register;

    #[test]
    fn reg_code() {
        assert_eq!(EAX.code(), 0);
    }

    #[test]
    fn reg_code_from() {
        assert_eq!(u8::from(ECX), 1);
    }
}
