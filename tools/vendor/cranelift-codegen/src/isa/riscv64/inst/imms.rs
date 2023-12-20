//! Riscv64 ISA definitions: immediate constants.

// Some variants are never constructed, but we still want them as options in the future.
use super::Inst;
#[allow(dead_code)]
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Copy, Clone, Debug, Default)]
pub struct Imm12 {
    pub bits: i16,
}

impl Imm12 {
    pub(crate) const FALSE: Self = Self { bits: 0 };
    pub(crate) const TRUE: Self = Self { bits: 1 };
    pub fn maybe_from_u64(val: u64) -> Option<Imm12> {
        let sign_bit = 1 << 11;
        if val == 0 {
            Some(Imm12 { bits: 0 })
        } else if (val & sign_bit) != 0 && (val >> 12) == 0xffff_ffff_ffff_f {
            Some(Imm12 {
                bits: (val & 0xffff) as i16,
            })
        } else if (val & sign_bit) == 0 && (val >> 12) == 0 {
            Some(Imm12 {
                bits: (val & 0xffff) as i16,
            })
        } else {
            None
        }
    }
    #[inline]
    pub fn from_bits(bits: i16) -> Self {
        Self { bits: bits & 0xfff }
    }
    /// Create a zero immediate of this format.
    #[inline]
    pub fn zero() -> Self {
        Imm12 { bits: 0 }
    }
    #[inline]
    pub fn as_i16(self) -> i16 {
        self.bits
    }
    #[inline]
    pub fn as_u32(&self) -> u32 {
        (self.bits as u32) & 0xfff
    }
}

impl Into<i64> for Imm12 {
    fn into(self) -> i64 {
        self.bits as i64
    }
}

impl Display for Imm12 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:+}", self.bits)
    }
}

impl std::ops::Neg for Imm12 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self { bits: -self.bits }
    }
}

// singed
#[derive(Clone, Copy, Default)]
pub struct Imm20 {
    /// The immediate bits.
    pub bits: i32,
}

impl Imm20 {
    #[inline]
    pub fn from_bits(bits: i32) -> Self {
        Self {
            bits: bits & 0xf_ffff,
        }
    }
    #[inline]
    pub fn as_u32(&self) -> u32 {
        (self.bits as u32) & 0xf_ffff
    }
}

impl Debug for Imm20 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.bits)
    }
}

impl Display for Imm20 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.bits)
    }
}

#[derive(Clone, Copy)]
pub struct Uimm5 {
    bits: u8,
}

impl Uimm5 {
    pub fn from_bits(bits: u8) -> Self {
        Self { bits }
    }
    /// Create a zero immediate of this format.
    pub fn zero() -> Self {
        Self { bits: 0 }
    }
    pub fn as_u32(&self) -> u32 {
        (self.bits as u32) & 0b1_1111
    }
}

impl Debug for Uimm5 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.bits)
    }
}

impl Display for Uimm5 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.bits)
    }
}

impl Inst {
    pub(crate) fn imm_min() -> i64 {
        let imm20_max: i64 = (1 << 19) << 12;
        let imm12_max = 1 << 11;
        -imm20_max - imm12_max
    }
    pub(crate) fn imm_max() -> i64 {
        let imm20_max: i64 = ((1 << 19) - 1) << 12;
        let imm12_max = (1 << 11) - 1;
        imm20_max + imm12_max
    }

    /// An imm20 immediate and an Imm12 immediate can generate a 32-bit immediate.
    /// This helper produces an imm12, imm20, or both to generate the value.
    ///
    /// `value` must be between `imm_min()` and `imm_max()`, or else
    /// this helper returns `None`.
    pub(crate) fn generate_imm<R>(
        value: u64,
        mut handle_imm: impl FnMut(Option<Imm20>, Option<Imm12>) -> R,
    ) -> Option<R> {
        if let Some(imm12) = Imm12::maybe_from_u64(value) {
            // can be load using single imm12.
            let r = handle_imm(None, Some(imm12));
            return Some(r);
        }
        let value = value as i64;
        if !(value >= Self::imm_min() && value <= Self::imm_max()) {
            // not in range, return None.
            return None;
        }
        const MOD_NUM: i64 = 4096;
        let (imm20, imm12) = if value > 0 {
            let mut imm20 = value / MOD_NUM;
            let mut imm12 = value % MOD_NUM;
            if imm12 >= 2048 {
                imm12 -= MOD_NUM;
                imm20 += 1;
            }
            assert!(imm12 >= -2048 && imm12 <= 2047);
            (imm20, imm12)
        } else {
            // this is the abs value.
            let value_abs = value.abs();
            let imm20 = value_abs / MOD_NUM;
            let imm12 = value_abs % MOD_NUM;
            let mut imm20 = -imm20;
            let mut imm12 = -imm12;
            if imm12 < -2048 {
                imm12 += MOD_NUM;
                imm20 -= 1;
            }
            (imm20, imm12)
        };
        assert!(imm20 >= -(0x7_ffff + 1) && imm20 <= 0x7_ffff);
        assert!(imm20 != 0 || imm12 != 0);
        Some(handle_imm(
            if imm20 != 0 {
                Some(Imm20::from_bits(imm20 as i32))
            } else {
                None
            },
            if imm12 != 0 {
                Some(Imm12::from_bits(imm12 as i16))
            } else {
                None
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_imm12() {
        let x = Imm12::zero();
        assert_eq!(0, x.as_u32());
        Imm12::maybe_from_u64(0xffff_ffff_ffff_ffff).unwrap();
    }

    #[test]
    fn imm20_and_imm12() {
        assert!(Inst::imm_max() == (i32::MAX - 2048) as i64);
        assert!(Inst::imm_min() == i32::MIN as i64 - 2048);
    }
}
