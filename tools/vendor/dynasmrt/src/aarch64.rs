//! Runtime support for the aarch64 architecture assembling target.
//!
//! The aarch64 instruction set features fixed-width 32-bit instructions and relative relocations up to 28 bits in size.
//!
//! The core relocation behaviour for this architecture is provided by the [`Aarch64Relocation`] type.
//!
//! Next to that, this module contains the following:
//!
//! ## Type aliases
//!
//! Several specialized type aliases of the generic [`Assembler`] are provided as these are by far the most common usecase.
//!
//! ## Enums
//!
//! There are enumerator of every logically distinct register family usable in aarch64. 
//! These enums implement the [`Register`] trait and their discriminant values match their numeric encoding in dynamic register literals.
//!
//! *Note: The presence of some registers listed here is purely what is encodable. Check the relevant architecture documentation to find what is architecturally valid.*
//!
//! ## Functions
//!
//! The aarch64 architecture allows encoding several special types of immediates. The encoding implementations for these immediate types have been exposed to assist the user
//! in correctly using these instructions. They will return `Some(encoding)` only if the given value can be encoded losslessly in that immediate type.

use crate::Register;
use crate::relocations::{Relocation, RelocationSize, RelocationKind, ImpossibleRelocation, fits_signed_bitfield};
use byteorder::{ByteOrder, LittleEndian};
use std::convert::TryFrom;

/// Relocation implementation for the aarch64 architecture.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum Aarch64Relocation {
    // b, bl 26 bits, dword aligned
    B,
    // b.cond, cbnz, cbz, ldr, ldrsw, prfm: 19 bits, dword aligned
    BCOND,
    // adr split 21 bit, byte aligned
    ADR,
    // adrp split 21 bit, 4096-byte aligned
    ADRP,
    // tbnz, tbz: 14 bits, dword aligned
    TBZ,
    // Anything in directives
    Plain(RelocationSize),
}

impl Aarch64Relocation {
    fn op_mask(&self) -> u32 {
        match self {
            Self::B => 0xFC00_0000,
            Self::BCOND => 0xFF00_001F,
            Self::ADR => 0x9F00_001F,
            Self::ADRP => 0x9F00_001F,
            Self::TBZ => 0xFFF8_001F,
            Self::Plain(_) => 0
        }
    }

    fn encode(&self, value: isize) -> Result<u32, ImpossibleRelocation> {
        let value = i64::try_from(value).map_err(|_| ImpossibleRelocation { } )?;
        Ok(match self {
            Self::B => {
                if value & 3 != 0 || !fits_signed_bitfield(value >> 2, 26) {
                    return Err(ImpossibleRelocation { } );
                }
                let value = (value >> 2) as u32;
                value & 0x3FF_FFFF
            },
            Self::BCOND => {
                if value & 3 != 0 || !fits_signed_bitfield(value >> 2, 19) {
                    return Err(ImpossibleRelocation { } );
                }
                let value = (value >> 2) as u32;
                (value & 0x7FFFF) << 5
            },
            Self::ADR => {
                if !fits_signed_bitfield(value, 21) {
                    return Err(ImpossibleRelocation { } );
                }
                let low = (value) as u32;
                let high = (value >> 2) as u32;
                ((high & 0x7FFFF) << 5) | ((low & 3) << 29)
            },
            Self::ADRP => {
                let value = value + 0xFFF;
                if !fits_signed_bitfield(value >> 12, 21) {
                    return Err(ImpossibleRelocation { } );
                }
                let low = (value >> 12) as u32;
                let high = (value >> 14) as u32;
                ((high & 0x7FFFF) << 5) | ((low & 3) << 29)
            },
            Self::TBZ => {
                if value & 3 != 0 || !fits_signed_bitfield(value >> 2, 14) {
                    return Err(ImpossibleRelocation { } );
                }
                let value = (value >> 2) as u32;
                (value & 0x3FFF) << 5
            },
            Self::Plain(_) => return Err(ImpossibleRelocation { } )
        })
    }
}

impl Relocation for Aarch64Relocation {
    type Encoding = (u8,);
    fn from_encoding(encoding: Self::Encoding) -> Self {
        match encoding.0 {
            0 => Self::B,
            1 => Self::BCOND,
            2 => Self::ADR,
            3 => Self::ADRP,
            4 => Self::TBZ,
            x  => Self::Plain(RelocationSize::from_encoding(x - 4))
        }
    }
    fn from_size(size: RelocationSize) -> Self {
        Self::Plain(size)
    }
    fn size(&self) -> usize {
        match self {
            Self::Plain(s) => s.size(),
            _ => RelocationSize::DWord.size(),
        }
    }
    fn write_value(&self, buf: &mut [u8], value: isize) -> Result<(), ImpossibleRelocation> {
        if let Self::Plain(s) = self {
            return s.write_value(buf, value);
        };

        let mask = self.op_mask();
        let template = LittleEndian::read_u32(buf) & mask;

        let packed = self.encode(value)?;

        LittleEndian::write_u32(buf, template | packed);
        Ok(())
    }
    fn read_value(&self, buf: &[u8]) -> isize {
        if let Self::Plain(s) = self {
            return s.read_value(buf);
        };

        let mask = !self.op_mask();
        let value = LittleEndian::read_u32(buf);
        let unpacked = match self {
            Self::B => u64::from(
                value & mask
            ) << 2,
            Self::BCOND => u64::from(
                (value & mask) >> 5
            ) << 2,
            Self::ADR  => u64::from(
                (((value >> 5 ) & 0x7FFFF) << 2) |
                ((value >> 29) & 3 )
            ),
            Self::ADRP => u64::from(
                (((value >> 5 ) & 0x7FFFF) << 2) |
                ((value >> 29) & 3 )
            ) << 12,
            Self::TBZ => u64::from(
                (value & mask) >> 5
            ) << 2,
            Self::Plain(_) => unreachable!()
        };

        // Sign extend.
        let bits = match self {
            Self::B => 26,
            Self::BCOND => 19,
            Self::ADR => 21,
            Self::ADRP => 33,
            Self::TBZ => 14,
            Self::Plain(_) => unreachable!()
        };
        let offset = 1u64 << (bits - 1);
        let value: u64 = (unpacked ^ offset) - offset;

        value as i64 as isize
    }
    fn kind(&self) -> RelocationKind {
        RelocationKind::Relative
    }
    fn page_size() -> usize {
        4096
    }
}

/// An aarch64 Assembler. This is aliased here for backwards compatability.
pub type Assembler = crate::Assembler<Aarch64Relocation>;
/// An aarch64 AssemblyModifier. This is aliased here for backwards compatability.
pub type AssemblyModifier<'a> = crate::Modifier<'a, Aarch64Relocation>;
/// An aarch64 UncommittedModifier. This is aliased here for backwards compatability.
pub type UncommittedModifier<'a> = crate::UncommittedModifier<'a>;


/// Helper function for validating that a given value can be encoded as a 32-bit logical immediate
pub fn encode_logical_immediate_32bit(value: u32) -> Option<u16> {
    let transitions = value ^ value.rotate_right(1);
    let element_size = (64u32).checked_div(transitions.count_ones())?;

    // confirm that the elements are identical
    if value != value.rotate_left(element_size) {
        return None;
    }

    let element = value & 1u32.checked_shl(element_size).unwrap_or(0).wrapping_sub(1);
    let ones = element.count_ones();
    let imms = (!((element_size << 1) - 1) & 0x3F) | (ones - 1);

    let immr = if (element & 1) != 0 {
        ones - (!element).trailing_zeros()
    } else {
        element_size - element.trailing_zeros()
    };

    Some(((immr as u16) << 6) | (imms as u16))
}

/// Helper function for validating that a given value can be encoded as a 64-bit logical immediate
pub fn encode_logical_immediate_64bit(value: u64) -> Option<u16> {
    let transitions = value ^ value.rotate_right(1);
    let element_size = (128u32).checked_div(transitions.count_ones())?;

    // confirm that the elements are identical
    if value != value.rotate_left(element_size) {
        return None;
    }

    let element = value & 1u64.checked_shl(element_size).unwrap_or(0).wrapping_sub(1);
    let ones = element.count_ones();
    let imms = (!((element_size << 1) - 1) & 0x7F) | (ones - 1);

    let immr = if (element & 1) != 0 {
        ones - (!element).trailing_zeros()
    } else {
        element_size - element.trailing_zeros()
    };

    let n = imms & 0x40 == 0;
    let imms = imms & 0x3F;

    Some(((n as u16) << 12) | ((immr as u16) << 6) | (imms as u16))
}

/// Helper function for validating that a given value can be encoded as a floating point immediate
pub fn encode_floating_point_immediate(value: f32) -> Option<u8> {
    // floating point ARM immediates are encoded as
    // abcdefgh => aBbbbbbc defgh000 00000000 00000000
    // where B = !b
    // which means we can just slice out "a" and "bcdefgh" and assume the rest was correct

    let bits = value.to_bits();

    let check = (bits >> 25) & 0x3F;
    if (check == 0b10_0000 || check == 0b01_1111) && (bits & 0x7_FFFF) == 0 {
        Some((((bits >> 24) & 0x80) | ((bits >> 19) & 0x7F)) as u8)
    } else {
        None
    }
}


/// 4 or 8-byte general purpopse registers, where X31 is the zero register.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RX {
    X0 = 0x00, X1 = 0x01, X2 = 0x02, X3 = 0x03,
    X4 = 0x04, X5 = 0x05, X6 = 0x06, X7 = 0x07,
    X8 = 0x08, X9 = 0x09, X10= 0x0A, X11= 0x0B,
    X12= 0x0C, X13= 0x0D, X14= 0x0E, X15= 0x0F,
    X16= 0x10, X17= 0x11, X18= 0x12, X19= 0x13,
    X20= 0x14, X21= 0x15, X22= 0x16, X23= 0x17,
    X24= 0x18, X25= 0x19, X26= 0x1A, X27= 0x1B,
    X28= 0x1C, X29= 0x1D, X30= 0x1E, XZR= 0x1F,
}
reg_impls!(RX);

/// 0x1F addresses both XZR and SP (disambiguated by context). This enum is a mirror of RX just
/// with the SP in place of XZR.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RXSP {
    X0 = 0x00, X1 = 0x01, X2 = 0x02, X3 = 0x03,
    X4 = 0x04, X5 = 0x05, X6 = 0x06, X7 = 0x07,
    X8 = 0x08, X9 = 0x09, X10= 0x0A, X11= 0x0B,
    X12= 0x0C, X13= 0x0D, X14= 0x0E, X15= 0x0F,
    X16= 0x10, X17= 0x11, X18= 0x12, X19= 0x13,
    X20= 0x14, X21= 0x15, X22= 0x16, X23= 0x17,
    X24= 0x18, X25= 0x19, X26= 0x1A, X27= 0x1B,
    X28= 0x1C, X29= 0x1D, X30= 0x1E, SP = 0x1F,
}
reg_impls!(RXSP);

/// 1, 2, 4, 8 or 16-bytes scalar FP / vector SIMD registers. 
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RV {
    V0 = 0x00, V1 = 0x01, V2 = 0x02, V3 = 0x03,
    V4 = 0x04, V5 = 0x05, V6 = 0x06, V7 = 0x07,
    V8 = 0x08, V9 = 0x09, V10= 0x0A, V11= 0x0B,
    V12= 0x0C, V13= 0x0D, V14= 0x0E, V15= 0x0F,
    V16= 0x10, V17= 0x11, V18= 0x12, V19= 0x13,
    V20= 0x14, V21= 0x15, V22= 0x16, V23= 0x17,
    V24= 0x18, V25= 0x19, V26= 0x1A, V27= 0x1B,
    V28= 0x1C, V29= 0x1D, V30= 0x1E, V31= 0x1F,
}
reg_impls!(RV);

#[cfg(test)]
mod tests {
    use super::RX::*;
    use crate::Register;

    #[test]
    fn reg_code() {
        assert_eq!(X2.code(), 2);
    }

    #[test]
    fn reg_code_from() {
        assert_eq!(u8::from(X24), 0x18);
    }
}
