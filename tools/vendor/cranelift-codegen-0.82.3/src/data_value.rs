//! This module gives users to instantiate values that Cranelift understands. These values are used,
//! for example, during interpretation and for wrapping immediates.
use crate::ir::immediates::{Ieee32, Ieee64, Offset32};
use crate::ir::{types, ConstantData, Type};
use core::convert::TryInto;
use core::fmt::{self, Display, Formatter};

/// Represent a data value. Where [Value] is an SSA reference, [DataValue] is the type + value
/// that would be referred to by a [Value].
///
/// [Value]: crate::ir::Value
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum DataValue {
    B(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(Ieee32),
    F64(Ieee64),
    V128([u8; 16]),
}

impl DataValue {
    /// Try to cast an immediate integer (a wrapped `i64` on most Cranelift instructions) to the
    /// given Cranelift [Type].
    pub fn from_integer(imm: i128, ty: Type) -> Result<DataValue, DataValueCastFailure> {
        match ty {
            types::I8 => Ok(DataValue::I8(imm as i8)),
            types::I16 => Ok(DataValue::I16(imm as i16)),
            types::I32 => Ok(DataValue::I32(imm as i32)),
            types::I64 => Ok(DataValue::I64(imm as i64)),
            types::I128 => Ok(DataValue::I128(imm)),
            _ => Err(DataValueCastFailure::FromInteger(imm, ty)),
        }
    }

    /// Return the Cranelift IR [Type] for this [DataValue].
    pub fn ty(&self) -> Type {
        match self {
            DataValue::B(_) => types::B8, // A default type.
            DataValue::I8(_) | DataValue::U8(_) => types::I8,
            DataValue::I16(_) | DataValue::U16(_) => types::I16,
            DataValue::I32(_) | DataValue::U32(_) => types::I32,
            DataValue::I64(_) | DataValue::U64(_) => types::I64,
            DataValue::I128(_) | DataValue::U128(_) => types::I128,
            DataValue::F32(_) => types::F32,
            DataValue::F64(_) => types::F64,
            DataValue::V128(_) => types::I8X16, // A default type.
        }
    }

    /// Return true if the value is a vector (i.e. `DataValue::V128`).
    pub fn is_vector(&self) -> bool {
        match self {
            DataValue::V128(_) => true,
            _ => false,
        }
    }

    /// Return true if the value is a bool (i.e. `DataValue::B`).
    pub fn is_bool(&self) -> bool {
        match self {
            DataValue::B(_) => true,
            _ => false,
        }
    }

    /// Write a [DataValue] to a slice.
    ///
    /// # Panics:
    ///
    /// Panics if the slice does not have enough space to accommodate the [DataValue]
    pub fn write_to_slice(&self, dst: &mut [u8]) {
        match self {
            DataValue::B(true) => dst[..16].copy_from_slice(&[u8::MAX; 16][..]),
            DataValue::B(false) => dst[..16].copy_from_slice(&[0; 16][..]),
            DataValue::I8(i) => dst[..1].copy_from_slice(&i.to_ne_bytes()[..]),
            DataValue::I16(i) => dst[..2].copy_from_slice(&i.to_ne_bytes()[..]),
            DataValue::I32(i) => dst[..4].copy_from_slice(&i.to_ne_bytes()[..]),
            DataValue::I64(i) => dst[..8].copy_from_slice(&i.to_ne_bytes()[..]),
            DataValue::I128(i) => dst[..16].copy_from_slice(&i.to_ne_bytes()[..]),
            DataValue::F32(f) => dst[..4].copy_from_slice(&f.bits().to_ne_bytes()[..]),
            DataValue::F64(f) => dst[..8].copy_from_slice(&f.bits().to_ne_bytes()[..]),
            DataValue::V128(v) => dst[..16].copy_from_slice(&v[..]),
            _ => unimplemented!(),
        };
    }

    /// Read a [DataValue] from a slice using a given [Type].
    ///
    /// # Panics:
    ///
    /// Panics if the slice does not have enough space to accommodate the [DataValue]
    pub fn read_from_slice(src: &[u8], ty: Type) -> Self {
        match ty {
            types::I8 => DataValue::I8(i8::from_ne_bytes(src[..1].try_into().unwrap())),
            types::I16 => DataValue::I16(i16::from_ne_bytes(src[..2].try_into().unwrap())),
            types::I32 => DataValue::I32(i32::from_ne_bytes(src[..4].try_into().unwrap())),
            types::I64 => DataValue::I64(i64::from_ne_bytes(src[..8].try_into().unwrap())),
            types::I128 => DataValue::I128(i128::from_ne_bytes(src[..16].try_into().unwrap())),
            types::F32 => DataValue::F32(Ieee32::with_bits(u32::from_ne_bytes(
                src[..4].try_into().unwrap(),
            ))),
            types::F64 => DataValue::F64(Ieee64::with_bits(u64::from_ne_bytes(
                src[..8].try_into().unwrap(),
            ))),
            _ if ty.is_bool() => {
                // Only `ty.bytes()` are guaranteed to be written
                // so we can only test the first n bytes of `src`

                let size = ty.bytes() as usize;
                DataValue::B(src[..size].iter().any(|&i| i != 0))
            }
            _ if ty.is_vector() && ty.bytes() == 16 => {
                DataValue::V128(src[..16].try_into().unwrap())
            }
            _ => unimplemented!(),
        }
    }

    /// Write a [DataValue] to a memory location.
    pub unsafe fn write_value_to(&self, p: *mut u128) {
        // Since `DataValue` does not have type info for bools we always
        // write out a full 16 byte slot.
        let size = match self.ty() {
            ty if ty.is_bool() => 16,
            ty => ty.bytes() as usize,
        };

        self.write_to_slice(std::slice::from_raw_parts_mut(p as *mut u8, size));
    }

    /// Read a [DataValue] from a memory location using a given [Type].
    pub unsafe fn read_value_from(p: *const u128, ty: Type) -> Self {
        DataValue::read_from_slice(
            std::slice::from_raw_parts(p as *const u8, ty.bytes() as usize),
            ty,
        )
    }
}

/// Record failures to cast [DataValue].
#[derive(Debug, PartialEq)]
#[allow(missing_docs)]
pub enum DataValueCastFailure {
    TryInto(Type, Type),
    FromInteger(i128, Type),
}

// This is manually implementing Error and Display instead of using thiserror to reduce the amount
// of dependencies used by Cranelift.
impl std::error::Error for DataValueCastFailure {}

impl Display for DataValueCastFailure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DataValueCastFailure::TryInto(from, to) => {
                write!(
                    f,
                    "unable to cast data value of type {} to type {}",
                    from, to
                )
            }
            DataValueCastFailure::FromInteger(val, to) => {
                write!(
                    f,
                    "unable to cast i64({}) to a data value of type {}",
                    val, to
                )
            }
        }
    }
}

/// Helper for creating conversion implementations for [DataValue].
macro_rules! build_conversion_impl {
    ( $rust_ty:ty, $data_value_ty:ident, $cranelift_ty:ident ) => {
        impl From<$rust_ty> for DataValue {
            fn from(data: $rust_ty) -> Self {
                DataValue::$data_value_ty(data)
            }
        }

        impl TryInto<$rust_ty> for DataValue {
            type Error = DataValueCastFailure;
            fn try_into(self) -> Result<$rust_ty, Self::Error> {
                if let DataValue::$data_value_ty(v) = self {
                    Ok(v)
                } else {
                    Err(DataValueCastFailure::TryInto(
                        self.ty(),
                        types::$cranelift_ty,
                    ))
                }
            }
        }
    };
}
build_conversion_impl!(bool, B, B8);
build_conversion_impl!(i8, I8, I8);
build_conversion_impl!(i16, I16, I16);
build_conversion_impl!(i32, I32, I32);
build_conversion_impl!(i64, I64, I64);
build_conversion_impl!(i128, I128, I128);
build_conversion_impl!(u8, U8, I8);
build_conversion_impl!(u16, U16, I16);
build_conversion_impl!(u32, U32, I32);
build_conversion_impl!(u64, U64, I64);
build_conversion_impl!(u128, U128, I128);
build_conversion_impl!(Ieee32, F32, F32);
build_conversion_impl!(Ieee64, F64, F64);
build_conversion_impl!([u8; 16], V128, I8X16);
impl From<Offset32> for DataValue {
    fn from(o: Offset32) -> Self {
        DataValue::from(Into::<i32>::into(o))
    }
}

impl Display for DataValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::B(dv) => write!(f, "{}", dv),
            DataValue::I8(dv) => write!(f, "{}", dv),
            DataValue::I16(dv) => write!(f, "{}", dv),
            DataValue::I32(dv) => write!(f, "{}", dv),
            DataValue::I64(dv) => write!(f, "{}", dv),
            DataValue::I128(dv) => write!(f, "{}", dv),
            DataValue::U8(dv) => write!(f, "{}", dv),
            DataValue::U16(dv) => write!(f, "{}", dv),
            DataValue::U32(dv) => write!(f, "{}", dv),
            DataValue::U64(dv) => write!(f, "{}", dv),
            DataValue::U128(dv) => write!(f, "{}", dv),
            // The Ieee* wrappers here print the expected syntax.
            DataValue::F32(dv) => write!(f, "{}", dv),
            DataValue::F64(dv) => write!(f, "{}", dv),
            // Again, for syntax consistency, use ConstantData, which in this case displays as hex.
            DataValue::V128(dv) => write!(f, "{}", ConstantData::from(&dv[..])),
        }
    }
}

/// Helper structure for printing bracket-enclosed vectors of [DataValue]s.
/// - for empty vectors, display `[]`
/// - for single item vectors, display `42`, e.g.
/// - for multiple item vectors, display `[42, 43, 44]`, e.g.
pub struct DisplayDataValues<'a>(pub &'a [DataValue]);

impl<'a> Display for DisplayDataValues<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0.len() == 1 {
            write!(f, "{}", self.0[0])
        } else {
            write!(f, "[")?;
            write_data_value_list(f, &self.0)?;
            write!(f, "]")
        }
    }
}

/// Helper function for displaying `Vec<DataValue>`.
pub fn write_data_value_list(f: &mut Formatter<'_>, list: &[DataValue]) -> fmt::Result {
    match list.len() {
        0 => Ok(()),
        1 => write!(f, "{}", list[0]),
        _ => {
            write!(f, "{}", list[0])?;
            for dv in list.iter().skip(1) {
                write!(f, ", {}", dv)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn type_conversions() {
        assert_eq!(DataValue::B(true).ty(), types::B8);
        assert_eq!(
            TryInto::<bool>::try_into(DataValue::B(false)).unwrap(),
            false
        );
        assert_eq!(
            TryInto::<i32>::try_into(DataValue::B(false)).unwrap_err(),
            DataValueCastFailure::TryInto(types::B8, types::I32)
        );

        assert_eq!(DataValue::V128([0; 16]).ty(), types::I8X16);
        assert_eq!(
            TryInto::<[u8; 16]>::try_into(DataValue::V128([0; 16])).unwrap(),
            [0; 16]
        );
        assert_eq!(
            TryInto::<i32>::try_into(DataValue::V128([0; 16])).unwrap_err(),
            DataValueCastFailure::TryInto(types::I8X16, types::I32)
        );
    }
}
