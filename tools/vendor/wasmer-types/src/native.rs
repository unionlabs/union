//! This module permits to create native functions
//! easily in Rust, thanks to its advanced typing system.

use crate::extern_ref::VMExternRef;
use crate::lib::std::fmt;
use crate::types::Type;
use crate::values::{Value, WasmValueType};

/// `NativeWasmType` represents a Wasm type that has a direct
/// representation on the host (hence the “native” term).
///
/// It uses the Rust Type system to automatically detect the
/// Wasm type associated with a native Rust type.
///
/// ```
/// use wasmer_types::{NativeWasmType, Type};
///
/// let wasm_type = i32::WASM_TYPE;
/// assert_eq!(wasm_type, Type::I32);
/// ```
///
/// > Note: This strategy will be needed later to
/// > automatically detect the signature of a Rust function.
pub trait NativeWasmType: Sized {
    /// The ABI for this type (i32, i64, f32, f64)
    type Abi: Copy + fmt::Debug;

    /// Type for this `NativeWasmType`.
    const WASM_TYPE: Type;

    #[doc(hidden)]
    fn from_abi(abi: Self::Abi) -> Self;

    #[doc(hidden)]
    fn into_abi(self) -> Self::Abi;

    /// Convert self to i128 binary representation.
    fn to_binary(self) -> i128;

    /// Convert self to a `Value`.
    fn to_value<T: WasmValueType>(self) -> Value<T> {
        let binary = self.to_binary();
        // we need a store, we're just hoping we don't actually use it via funcref
        // TODO(reftypes): we need an actual solution here
        let hack = 3;

        unsafe { Value::read_value_from(&hack, &binary, Self::WASM_TYPE) }
    }

    /// Convert to self from i128 binary representation.
    fn from_binary(binary: i128) -> Self;
}

impl NativeWasmType for i32 {
    const WASM_TYPE: Type = Type::I32;
    type Abi = Self;

    #[inline]
    fn from_abi(abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self) -> Self::Abi {
        self
    }

    #[inline]
    fn to_binary(self) -> i128 {
        self as _
    }

    #[inline]
    fn from_binary(bits: i128) -> Self {
        bits as _
    }
}

impl NativeWasmType for i64 {
    const WASM_TYPE: Type = Type::I64;
    type Abi = Self;

    #[inline]
    fn from_abi(abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self) -> Self::Abi {
        self
    }

    #[inline]
    fn to_binary(self) -> i128 {
        self as _
    }

    #[inline]
    fn from_binary(bits: i128) -> Self {
        bits as _
    }
}

impl NativeWasmType for f32 {
    const WASM_TYPE: Type = Type::F32;
    type Abi = Self;

    #[inline]
    fn from_abi(abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self) -> Self::Abi {
        self
    }

    #[inline]
    fn to_binary(self) -> i128 {
        self.to_bits() as _
    }

    #[inline]
    fn from_binary(bits: i128) -> Self {
        Self::from_bits(bits as _)
    }
}

impl NativeWasmType for f64 {
    const WASM_TYPE: Type = Type::F64;
    type Abi = Self;

    #[inline]
    fn from_abi(abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self) -> Self::Abi {
        self
    }

    #[inline]
    fn to_binary(self) -> i128 {
        self.to_bits() as _
    }

    #[inline]
    fn from_binary(bits: i128) -> Self {
        Self::from_bits(bits as _)
    }
}

impl NativeWasmType for u128 {
    const WASM_TYPE: Type = Type::V128;
    type Abi = Self;

    #[inline]
    fn from_abi(abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self) -> Self::Abi {
        self
    }

    #[inline]
    fn to_binary(self) -> i128 {
        self as _
    }

    #[inline]
    fn from_binary(bits: i128) -> Self {
        bits as _
    }
}

impl NativeWasmType for VMExternRef {
    const WASM_TYPE: Type = Type::ExternRef;
    type Abi = Self;

    #[inline]
    fn from_abi(abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self) -> Self::Abi {
        self
    }

    #[inline]
    fn to_binary(self) -> i128 {
        self.to_binary()
    }

    #[inline]
    fn from_binary(bits: i128) -> Self {
        // TODO(reftypes): ensure that the safety invariants are actually upheld here
        unsafe { Self::from_binary(bits) }
    }
}

#[cfg(test)]
mod test_native_type {
    use super::*;
    use crate::types::Type;

    #[test]
    fn test_wasm_types() {
        assert_eq!(i32::WASM_TYPE, Type::I32);
        assert_eq!(i64::WASM_TYPE, Type::I64);
        assert_eq!(f32::WASM_TYPE, Type::F32);
        assert_eq!(f64::WASM_TYPE, Type::F64);
        assert_eq!(u128::WASM_TYPE, Type::V128);
    }

    #[test]
    fn test_roundtrip() {
        assert_eq!(i32::from_binary(42i32.to_binary()), 42i32);
        assert_eq!(i64::from_binary(42i64.to_binary()), 42i64);
        assert_eq!(f32::from_binary(42f32.to_binary()), 42f32);
        assert_eq!(f64::from_binary(42f64.to_binary()), 42f64);
        assert_eq!(u128::from_binary(42u128.to_binary()), 42u128);
    }
}

// pub trait IntegerAtomic
// where
//     Self: Sized
// {
//     type Primitive;

//     fn add(&self, other: Self::Primitive) -> Self::Primitive;
//     fn sub(&self, other: Self::Primitive) -> Self::Primitive;
//     fn and(&self, other: Self::Primitive) -> Self::Primitive;
//     fn or(&self, other: Self::Primitive) -> Self::Primitive;
//     fn xor(&self, other: Self::Primitive) -> Self::Primitive;
//     fn load(&self) -> Self::Primitive;
//     fn store(&self, other: Self::Primitive) -> Self::Primitive;
//     fn compare_exchange(&self, expected: Self::Primitive, new: Self::Primitive) -> Self::Primitive;
//     fn swap(&self, other: Self::Primitive) -> Self::Primitive;
// }

/// Trait for a Value type. A Value type is a type that is always valid and may
/// be safely copied.
///
/// That is, for all possible bit patterns a valid Value type can be constructed
/// from those bits.
///
/// Concretely a `u32` is a Value type because every combination of 32 bits is
/// a valid `u32`. However a `bool` is _not_ a Value type because any bit patterns
/// other than `0` and `1` are invalid in Rust and may cause undefined behavior if
/// a `bool` is constructed from those bytes.
pub unsafe trait ValueType: Copy
where
    Self: Sized,
{
}

macro_rules! impl_value_type_for {
    ( $($type:ty),* ) => {
        $(
            unsafe impl ValueType for $type {}
        )*
    };
}

impl_value_type_for!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
