//! This module permits to create native functions
//! easily in Rust, thanks to its advanced typing system.

use wasmer_types::{NativeWasmType, RawValue, Type};

use crate::store::AsStoreRef;
use crate::vm::{VMExternRef, VMFuncRef};

use crate::{ExternRef, Function, TypedFunction};
use std::array::TryFromSliceError;
use std::convert::Infallible;
use std::convert::TryInto;
use std::error::Error;

use crate::store::AsStoreMut;

/// `NativeWasmTypeInto` performs conversions from and into `NativeWasmType`
/// types with a context.
pub trait NativeWasmTypeInto: NativeWasmType + Sized {
    #[doc(hidden)]
    fn into_abi(self, store: &mut impl AsStoreMut) -> Self::Abi;

    #[doc(hidden)]
    unsafe fn from_abi(store: &mut impl AsStoreMut, abi: Self::Abi) -> Self;

    /// Convert self to raw value representation.
    fn into_raw(self, store: &mut impl AsStoreMut) -> RawValue;

    /// Convert to self from raw value representation.
    ///
    /// # Safety
    ///
    unsafe fn from_raw(store: &mut impl AsStoreMut, raw: RawValue) -> Self;
}

impl NativeWasmTypeInto for i32 {
    #[inline]
    unsafe fn from_abi(_store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self, _store: &mut impl AsStoreMut) -> Self::Abi {
        self
    }

    #[inline]
    fn into_raw(self, _store: &mut impl AsStoreMut) -> RawValue {
        RawValue { i32: self }
    }

    #[inline]
    unsafe fn from_raw(_store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        raw.i32
    }
}

impl NativeWasmTypeInto for u32 {
    #[inline]
    unsafe fn from_abi(_store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self, _store: &mut impl AsStoreMut) -> Self::Abi {
        self
    }

    #[inline]
    fn into_raw(self, _store: &mut impl AsStoreMut) -> RawValue {
        RawValue { i32: self as _ }
    }

    #[inline]
    unsafe fn from_raw(_store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        raw.i32 as _
    }
}

impl NativeWasmTypeInto for i64 {
    #[inline]
    unsafe fn from_abi(_store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self, _store: &mut impl AsStoreMut) -> Self::Abi {
        self
    }

    #[inline]
    fn into_raw(self, _store: &mut impl AsStoreMut) -> RawValue {
        RawValue { i64: self }
    }

    #[inline]
    unsafe fn from_raw(_store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        raw.i64
    }
}

impl NativeWasmTypeInto for u64 {
    #[inline]
    unsafe fn from_abi(_store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self, _store: &mut impl AsStoreMut) -> Self::Abi {
        self
    }

    #[inline]
    fn into_raw(self, _store: &mut impl AsStoreMut) -> RawValue {
        RawValue { i64: self as _ }
    }

    #[inline]
    unsafe fn from_raw(_store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        raw.i64 as _
    }
}

impl NativeWasmTypeInto for f32 {
    #[inline]
    unsafe fn from_abi(_store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self, _store: &mut impl AsStoreMut) -> Self::Abi {
        self
    }

    #[inline]
    fn into_raw(self, _store: &mut impl AsStoreMut) -> RawValue {
        RawValue { f32: self }
    }

    #[inline]
    unsafe fn from_raw(_store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        raw.f32
    }
}

impl NativeWasmTypeInto for f64 {
    #[inline]
    unsafe fn from_abi(_store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self, _store: &mut impl AsStoreMut) -> Self::Abi {
        self
    }

    #[inline]
    fn into_raw(self, _store: &mut impl AsStoreMut) -> RawValue {
        RawValue { f64: self }
    }

    #[inline]
    unsafe fn from_raw(_store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        raw.f64
    }
}

impl NativeWasmTypeInto for u128 {
    #[inline]
    unsafe fn from_abi(_store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        abi
    }

    #[inline]
    fn into_abi(self, _store: &mut impl AsStoreMut) -> Self::Abi {
        self
    }

    #[inline]
    fn into_raw(self, _store: &mut impl AsStoreMut) -> RawValue {
        RawValue { u128: self }
    }

    #[inline]
    unsafe fn from_raw(_store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        raw.u128
    }
}

impl NativeWasmType for ExternRef {
    const WASM_TYPE: Type = Type::ExternRef;
    type Abi = usize;
}

impl NativeWasmTypeInto for Option<ExternRef> {
    #[inline]
    unsafe fn from_abi(store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        VMExternRef::from_raw(RawValue { externref: abi })
            .map(|e| ExternRef::from_vm_externref(store, e))
    }

    #[inline]
    fn into_abi(self, _store: &mut impl AsStoreMut) -> Self::Abi {
        self.map_or(0, |e| unsafe { e.vm_externref().into_raw().externref })
    }

    #[inline]
    fn into_raw(self, _store: &mut impl AsStoreMut) -> RawValue {
        self.map_or(RawValue { externref: 0 }, |e| e.vm_externref().into_raw())
    }

    #[inline]
    unsafe fn from_raw(store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        VMExternRef::from_raw(raw).map(|e| ExternRef::from_vm_externref(store, e))
    }
}

impl<Args, Rets> From<TypedFunction<Args, Rets>> for Function
where
    Args: WasmTypeList,
    Rets: WasmTypeList,
{
    fn from(other: TypedFunction<Args, Rets>) -> Self {
        other.into_function()
    }
}

impl NativeWasmType for Function {
    const WASM_TYPE: Type = Type::FuncRef;
    type Abi = usize;
}

impl NativeWasmTypeInto for Option<Function> {
    #[inline]
    unsafe fn from_abi(store: &mut impl AsStoreMut, abi: Self::Abi) -> Self {
        VMFuncRef::from_raw(RawValue { funcref: abi }).map(|f| Function::from_vm_funcref(store, f))
    }

    #[inline]
    fn into_abi(self, store: &mut impl AsStoreMut) -> Self::Abi {
        self.map_or(0, |f| unsafe { f.vm_funcref(store).into_raw().externref })
    }

    #[inline]
    fn into_raw(self, store: &mut impl AsStoreMut) -> RawValue {
        self.map_or(RawValue { externref: 0 }, |e| {
            e.vm_funcref(store).into_raw()
        })
    }

    #[inline]
    unsafe fn from_raw(store: &mut impl AsStoreMut, raw: RawValue) -> Self {
        VMFuncRef::from_raw(raw).map(|f| Function::from_vm_funcref(store, f))
    }
}

/// A trait to convert a Rust value to a `WasmNativeType` value,
/// or to convert `WasmNativeType` value to a Rust value.
///
/// This trait should ideally be split into two traits:
/// `FromNativeWasmType` and `ToNativeWasmType` but it creates a
/// non-negligible complexity in the `WasmTypeList`
/// implementation.
///
/// # Safety
/// This trait is unsafe given the nature of how values are written and read from the native
/// stack
pub unsafe trait FromToNativeWasmType
where
    Self: Sized,
{
    /// Native Wasm type.
    type Native: NativeWasmTypeInto;

    /// Convert a value of kind `Self::Native` to `Self`.
    ///
    /// # Panics
    ///
    /// This method panics if `native` cannot fit in the `Self`
    /// type`.
    fn from_native(native: Self::Native) -> Self;

    /// Convert self to `Self::Native`.
    ///
    /// # Panics
    ///
    /// This method panics if `self` cannot fit in the
    /// `Self::Native` type.
    fn to_native(self) -> Self::Native;

    /// Returns whether the given value is from the given store.
    ///
    /// This always returns true for primitive types that can be used with
    /// any context.
    fn is_from_store(&self, _store: &impl AsStoreRef) -> bool {
        true
    }
}

macro_rules! from_to_native_wasm_type {
    ( $( $type:ty => $native_type:ty ),* ) => {
        $(
            #[allow(clippy::use_self)]
            unsafe impl FromToNativeWasmType for $type {
                type Native = $native_type;

                #[inline]
                fn from_native(native: Self::Native) -> Self {
                    native as Self
                }

                #[inline]
                fn to_native(self) -> Self::Native {
                    self as Self::Native
                }
            }
        )*
    };
}

macro_rules! from_to_native_wasm_type_same_size {
    ( $( $type:ty => $native_type:ty ),* ) => {
        $(
            #[allow(clippy::use_self)]
            unsafe impl FromToNativeWasmType for $type {
                type Native = $native_type;

                #[inline]
                fn from_native(native: Self::Native) -> Self {
                    Self::from_ne_bytes(Self::Native::to_ne_bytes(native))
                }

                #[inline]
                fn to_native(self) -> Self::Native {
                    Self::Native::from_ne_bytes(Self::to_ne_bytes(self))
                }
            }
        )*
    };
}

from_to_native_wasm_type!(
    i8 => i32,
    u8 => i32,
    i16 => i32,
    u16 => i32
);

from_to_native_wasm_type_same_size!(
    i32 => i32,
    u32 => i32,
    i64 => i64,
    u64 => i64,
    f32 => f32,
    f64 => f64
);

unsafe impl FromToNativeWasmType for Option<ExternRef> {
    type Native = Self;

    fn to_native(self) -> Self::Native {
        self
    }
    fn from_native(n: Self::Native) -> Self {
        n
    }
    fn is_from_store(&self, store: &impl AsStoreRef) -> bool {
        self.as_ref().map_or(true, |e| e.is_from_store(store))
    }
}

unsafe impl FromToNativeWasmType for Option<Function> {
    type Native = Self;

    fn to_native(self) -> Self::Native {
        self
    }
    fn from_native(n: Self::Native) -> Self {
        n
    }
    fn is_from_store(&self, store: &impl AsStoreRef) -> bool {
        self.as_ref().map_or(true, |f| f.is_from_store(store))
    }
}

#[cfg(test)]
mod test_from_to_native_wasm_type {
    use super::*;

    #[test]
    fn test_to_native() {
        assert_eq!(7i8.to_native(), 7i32);
        assert_eq!(7u8.to_native(), 7i32);
        assert_eq!(7i16.to_native(), 7i32);
        assert_eq!(7u16.to_native(), 7i32);
        assert_eq!(u32::MAX.to_native(), -1);
    }

    #[test]
    fn test_to_native_same_size() {
        assert_eq!(7i32.to_native(), 7i32);
        assert_eq!(7u32.to_native(), 7i32);
        assert_eq!(7i64.to_native(), 7i64);
        assert_eq!(7u64.to_native(), 7i64);
        assert_eq!(7f32.to_native(), 7f32);
        assert_eq!(7f64.to_native(), 7f64);
    }
}

/// The `WasmTypeList` trait represents a tuple (list) of Wasm
/// typed values. It is used to get low-level representation of
/// such a tuple.
pub trait WasmTypeList
where
    Self: Sized,
{
    /// The C type (a struct) that can hold/represent all the
    /// represented values.
    type CStruct;

    /// The array type that can hold all the represented values.
    ///
    /// Note that all values are stored in their binary form.
    type Array: AsMut<[RawValue]>;

    /// The size of the array
    fn size() -> u32;

    /// Constructs `Self` based on an array of values.
    ///
    /// # Safety
    unsafe fn from_array(store: &mut impl AsStoreMut, array: Self::Array) -> Self;

    /// Constructs `Self` based on a slice of values.
    ///
    /// `from_slice` returns a `Result` because it is possible
    /// that the slice doesn't have the same size than
    /// `Self::Array`, in which circumstance an error of kind
    /// `TryFromSliceError` will be returned.
    ///
    /// # Safety
    unsafe fn from_slice(
        store: &mut impl AsStoreMut,
        slice: &[RawValue],
    ) -> Result<Self, TryFromSliceError>;

    /// Builds and returns an array of type `Array` from a tuple
    /// (list) of values.
    ///
    /// # Safety
    unsafe fn into_array(self, store: &mut impl AsStoreMut) -> Self::Array;

    /// Allocates and return an empty array of type `Array` that
    /// will hold a tuple (list) of values, usually to hold the
    /// returned values of a WebAssembly function call.
    fn empty_array() -> Self::Array;

    /// Builds a tuple (list) of values from a C struct of type
    /// `CStruct`.
    ///
    /// # Safety
    unsafe fn from_c_struct(store: &mut impl AsStoreMut, c_struct: Self::CStruct) -> Self;

    /// Builds and returns a C struct of type `CStruct` from a
    /// tuple (list) of values.
    ///
    /// # Safety
    unsafe fn into_c_struct(self, store: &mut impl AsStoreMut) -> Self::CStruct;

    /// Writes the contents of a C struct to an array of `RawValue`.
    ///
    /// # Safety
    unsafe fn write_c_struct_to_ptr(c_struct: Self::CStruct, ptr: *mut RawValue);

    /// Get the Wasm types for the tuple (list) of currently
    /// represented values.
    fn wasm_types() -> &'static [Type];
}

/// The `IntoResult` trait turns a `WasmTypeList` into a
/// `Result<WasmTypeList, Self::Error>`.
///
/// It is mostly used to turn result values of a Wasm function
/// call into a `Result`.
pub trait IntoResult<T>
where
    T: WasmTypeList,
{
    /// The error type for this trait.
    type Error: Error + Sync + Send + 'static;

    /// Transforms `Self` into a `Result`.
    fn into_result(self) -> Result<T, Self::Error>;
}

impl<T> IntoResult<T> for T
where
    T: WasmTypeList,
{
    // `T` is not a `Result`, it's already a value, so no error
    // can be built.
    type Error = Infallible;

    fn into_result(self) -> Result<Self, Infallible> {
        Ok(self)
    }
}

impl<T, E> IntoResult<T> for Result<T, E>
where
    T: WasmTypeList,
    E: Error + Sync + Send + 'static,
{
    type Error = E;

    fn into_result(self) -> Self {
        self
    }
}

#[cfg(test)]
mod test_into_result {
    use super::*;
    use std::convert::Infallible;

    #[test]
    fn test_into_result_over_t() {
        let x: i32 = 42;
        let result_of_x: Result<i32, Infallible> = x.into_result();

        assert_eq!(result_of_x.unwrap(), x);
    }

    #[test]
    fn test_into_result_over_result() {
        {
            let x: Result<i32, Infallible> = Ok(42);
            let result_of_x: Result<i32, Infallible> = x.into_result();

            assert_eq!(result_of_x, x);
        }

        {
            use std::{error, fmt};

            #[derive(Debug, PartialEq)]
            struct E;

            impl fmt::Display for E {
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(formatter, "E")
                }
            }

            impl error::Error for E {}

            let x: Result<Infallible, E> = Err(E);
            let result_of_x: Result<Infallible, E> = x.into_result();

            assert_eq!(result_of_x.unwrap_err(), E);
        }
    }
}

// Implement `WasmTypeList` on `Infallible`, which means that
// `Infallible` can be used as a returned type of a host function
// to express that it doesn't return, or to express that it cannot
// fail (with `Result<_, Infallible>`).
impl WasmTypeList for Infallible {
    type CStruct = Self;
    type Array = [RawValue; 0];

    fn size() -> u32 {
        0
    }

    unsafe fn from_array(_: &mut impl AsStoreMut, _: Self::Array) -> Self {
        unreachable!()
    }

    unsafe fn from_slice(
        _: &mut impl AsStoreMut,
        _: &[RawValue],
    ) -> Result<Self, TryFromSliceError> {
        unreachable!()
    }

    unsafe fn into_array(self, _: &mut impl AsStoreMut) -> Self::Array {
        []
    }

    fn empty_array() -> Self::Array {
        []
    }

    unsafe fn from_c_struct(_: &mut impl AsStoreMut, self_: Self::CStruct) -> Self {
        self_
    }

    unsafe fn into_c_struct(self, _: &mut impl AsStoreMut) -> Self::CStruct {
        self
    }

    unsafe fn write_c_struct_to_ptr(_: Self::CStruct, _: *mut RawValue) {}

    fn wasm_types() -> &'static [Type] {
        &[]
    }
}

macro_rules! impl_wasmtypelist {
    ( [$c_struct_representation:ident]
       $c_struct_name:ident,
       $( $x:ident ),* ) => {

        /// A structure with a C-compatible representation that can hold a set of Wasm values.
        /// This type is used by `WasmTypeList::CStruct`.
        #[repr($c_struct_representation)]
        pub struct $c_struct_name< $( $x ),* > ( $( <<$x as FromToNativeWasmType>::Native as NativeWasmType>::Abi ),* )
        where
            $( $x: FromToNativeWasmType ),*;

        // Implement `WasmTypeList` for a specific tuple.
        #[allow(unused_parens, dead_code)]
        impl< $( $x ),* >
            WasmTypeList
        for
            ( $( $x ),* )
        where
            $( $x: FromToNativeWasmType ),*
        {
            type CStruct = $c_struct_name< $( $x ),* >;

            type Array = [RawValue; count_idents!( $( $x ),* )];

            fn size() -> u32 {
                count_idents!( $( $x ),* ) as _
            }

            #[allow(unused_mut)]
            #[allow(clippy::unused_unit)]
            #[allow(clippy::missing_safety_doc)]
            unsafe fn from_array(mut _store: &mut impl AsStoreMut, array: Self::Array) -> Self {
                // Unpack items of the array.
                #[allow(non_snake_case)]
                let [ $( $x ),* ] = array;

                // Build the tuple.
                (
                    $(
                        FromToNativeWasmType::from_native(NativeWasmTypeInto::from_raw(_store, $x))
                    ),*
                )
            }

            #[allow(clippy::missing_safety_doc)]
            unsafe fn from_slice(store: &mut impl AsStoreMut, slice: &[RawValue]) -> Result<Self, TryFromSliceError> {
                Ok(Self::from_array(store, slice.try_into()?))
            }

            #[allow(unused_mut)]
            #[allow(clippy::missing_safety_doc)]
            unsafe fn into_array(self, mut _store: &mut impl AsStoreMut) -> Self::Array {
                // Unpack items of the tuple.
                #[allow(non_snake_case)]
                let ( $( $x ),* ) = self;

                // Build the array.
                [
                    $(
                        FromToNativeWasmType::to_native($x).into_raw(_store)
                    ),*
                ]
            }

            fn empty_array() -> Self::Array {
                // Build an array initialized with `0`.
                [RawValue { i32: 0 }; count_idents!( $( $x ),* )]
            }

            #[allow(unused_mut)]
            #[allow(clippy::unused_unit)]
            #[allow(clippy::missing_safety_doc)]
            unsafe fn from_c_struct(mut _store: &mut impl AsStoreMut, c_struct: Self::CStruct) -> Self {
                // Unpack items of the C structure.
                #[allow(non_snake_case)]
                let $c_struct_name( $( $x ),* ) = c_struct;

                (
                    $(
                        FromToNativeWasmType::from_native(NativeWasmTypeInto::from_abi(_store, $x))
                    ),*
                )
            }

            #[allow(unused_parens, non_snake_case, unused_mut)]
            #[allow(clippy::missing_safety_doc)]
            unsafe fn into_c_struct(self, mut _store: &mut impl AsStoreMut) -> Self::CStruct {
                // Unpack items of the tuple.
                let ( $( $x ),* ) = self;

                // Build the C structure.
                $c_struct_name(
                    $(
                        FromToNativeWasmType::to_native($x).into_abi(_store)
                    ),*
                )
            }

            #[allow(non_snake_case)]
            unsafe fn write_c_struct_to_ptr(c_struct: Self::CStruct, _ptr: *mut RawValue) {
                // Unpack items of the tuple.
                let $c_struct_name( $( $x ),* ) = c_struct;

                let mut _n = 0;
                $(
                    *_ptr.add(_n).cast() = $x;
                    _n += 1;
                )*
            }

            fn wasm_types() -> &'static [Type] {
                &[
                    $(
                        $x::Native::WASM_TYPE
                    ),*
                ]
            }
        }

    };
}

// Black-magic to count the number of identifiers at compile-time.
macro_rules! count_idents {
    ( $($idents:ident),* ) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            enum Idents { $( $idents, )* __CountIdentsLast }
            const COUNT: usize = Idents::__CountIdentsLast as usize;
            COUNT
        }
    };
}

// Here we go! Let's generate all the C struct and `WasmTypeList`
// implementations.
impl_wasmtypelist!([C] S0,);
impl_wasmtypelist!([transparent] S1, A1);
impl_wasmtypelist!([C] S2, A1, A2);
impl_wasmtypelist!([C] S3, A1, A2, A3);
impl_wasmtypelist!([C] S4, A1, A2, A3, A4);
impl_wasmtypelist!([C] S5, A1, A2, A3, A4, A5);
impl_wasmtypelist!([C] S6, A1, A2, A3, A4, A5, A6);
impl_wasmtypelist!([C] S7, A1, A2, A3, A4, A5, A6, A7);
impl_wasmtypelist!([C] S8, A1, A2, A3, A4, A5, A6, A7, A8);
impl_wasmtypelist!([C] S9, A1, A2, A3, A4, A5, A6, A7, A8, A9);
impl_wasmtypelist!([C] S10, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
impl_wasmtypelist!([C] S11, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
impl_wasmtypelist!([C] S12, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
impl_wasmtypelist!([C] S13, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
impl_wasmtypelist!([C] S14, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
impl_wasmtypelist!([C] S15, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
impl_wasmtypelist!([C] S16, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);
impl_wasmtypelist!([C] S17, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17);
impl_wasmtypelist!([C] S18, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18);
impl_wasmtypelist!([C] S19, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19);
impl_wasmtypelist!([C] S20, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20);
impl_wasmtypelist!([C] S21, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21);
impl_wasmtypelist!([C] S22, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22);
impl_wasmtypelist!([C] S23, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23);
impl_wasmtypelist!([C] S24, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24);
impl_wasmtypelist!([C] S25, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25);
impl_wasmtypelist!([C] S26, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26);

#[cfg(test)]
mod test_wasm_type_list {
    use super::*;
    use wasmer_types::Type;
    /*
    #[test]
    fn test_from_array() {
        let mut store = Store::default();
        let env = FunctionEnv::new(&mut store, ());
        assert_eq!(<()>::from_array(&mut env, []), ());
        assert_eq!(<i32>::from_array(&mut env, [RawValue{i32: 1}]), (1i32));
        assert_eq!(<(i32, i64)>::from_array(&mut env, [RawValue{i32:1}, RawValue{i64:2}]), (1i32, 2i64));
        assert_eq!(
            <(i32, i64, f32, f64)>::from_array(&mut env, [
                RawValue{i32:1},
                RawValue{i64:2},
                RawValue{f32: 3.1f32},
                RawValue{f64: 4.2f64}
            ]),
            (1, 2, 3.1f32, 4.2f64)
        );
    }

    #[test]
    fn test_into_array() {
        let mut store = Store::default();
        let env = FunctionEnv::new(&mut store, ());
        assert_eq!(().into_array(&mut store), [0i128; 0]);
        assert_eq!((1i32).into_array(&mut store), [1i32]);
        assert_eq!((1i32, 2i64).into_array(&mut store), [RawValue{i32: 1}, RawValue{i64: 2}]);
        assert_eq!(
            (1i32, 2i32, 3.1f32, 4.2f64).into_array(&mut store),
            [RawValue{i32: 1}, RawValue{i32: 2}, RawValue{ f32: 3.1f32}, RawValue{f64: 4.2f64}]
        );
    }
    */
    #[test]
    fn test_empty_array() {
        assert_eq!(<()>::empty_array().len(), 0);
        assert_eq!(<i32>::empty_array().len(), 1);
        assert_eq!(<(i32, i64)>::empty_array().len(), 2);
    }
    /*
    #[test]
    fn test_from_c_struct() {
        let mut store = Store::default();
        let env = FunctionEnv::new(&mut store, ());
        assert_eq!(<()>::from_c_struct(&mut store, S0()), ());
        assert_eq!(<i32>::from_c_struct(&mut store, S1(1)), (1i32));
        assert_eq!(<(i32, i64)>::from_c_struct(&mut store, S2(1, 2)), (1i32, 2i64));
        assert_eq!(
            <(i32, i64, f32, f64)>::from_c_struct(&mut store, S4(1, 2, 3.1, 4.2)),
            (1i32, 2i64, 3.1f32, 4.2f64)
        );
    }
    */
    #[test]
    fn test_wasm_types_for_uni_values() {
        assert_eq!(<i32>::wasm_types(), [Type::I32]);
        assert_eq!(<i64>::wasm_types(), [Type::I64]);
        assert_eq!(<f32>::wasm_types(), [Type::F32]);
        assert_eq!(<f64>::wasm_types(), [Type::F64]);
    }

    #[test]
    fn test_wasm_types_for_multi_values() {
        assert_eq!(<(i32, i32)>::wasm_types(), [Type::I32, Type::I32]);
        assert_eq!(<(i64, i64)>::wasm_types(), [Type::I64, Type::I64]);
        assert_eq!(<(f32, f32)>::wasm_types(), [Type::F32, Type::F32]);
        assert_eq!(<(f64, f64)>::wasm_types(), [Type::F64, Type::F64]);

        assert_eq!(
            <(i32, i64, f32, f64)>::wasm_types(),
            [Type::I32, Type::I64, Type::F32, Type::F64]
        );
    }
}
/*
    #[allow(non_snake_case)]
    #[cfg(test)]
    mod test_function {
        use super::*;
        use crate::Store;
        use crate::FunctionEnv;
        use wasmer_types::Type;

        fn func() {}
        fn func__i32() -> i32 {
            0
        }
        fn func_i32( _a: i32) {}
        fn func_i32__i32( a: i32) -> i32 {
            a * 2
        }
        fn func_i32_i32__i32( a: i32, b: i32) -> i32 {
            a + b
        }
        fn func_i32_i32__i32_i32( a: i32, b: i32) -> (i32, i32) {
            (a, b)
        }
        fn func_f32_i32__i32_f32( a: f32, b: i32) -> (i32, f32) {
            (b, a)
        }

        #[test]
        fn test_function_types() {
            let mut store = Store::default();
            let env = FunctionEnv::new(&mut store, ());
            use wasmer_types::FunctionType;
            assert_eq!(
                StaticFunction::new(func).ty(&mut store),
                FunctionType::new(vec![], vec![])
            );
            assert_eq!(
                StaticFunction::new(func__i32).ty(&mut store),
                FunctionType::new(vec![], vec![Type::I32])
            );
            assert_eq!(
                StaticFunction::new(func_i32).ty(),
                FunctionType::new(vec![Type::I32], vec![])
            );
            assert_eq!(
                StaticFunction::new(func_i32__i32).ty(),
                FunctionType::new(vec![Type::I32], vec![Type::I32])
            );
            assert_eq!(
                StaticFunction::new(func_i32_i32__i32).ty(),
                FunctionType::new(vec![Type::I32, Type::I32], vec![Type::I32])
            );
            assert_eq!(
                StaticFunction::new(func_i32_i32__i32_i32).ty(),
                FunctionType::new(vec![Type::I32, Type::I32], vec![Type::I32, Type::I32])
            );
            assert_eq!(
                StaticFunction::new(func_f32_i32__i32_f32).ty(),
                FunctionType::new(vec![Type::F32, Type::I32], vec![Type::I32, Type::F32])
            );
        }

        #[test]
        fn test_function_pointer() {
            let f = StaticFunction::new(func_i32__i32);
            let function = unsafe { std::mem::transmute::<_, fn(usize, i32) -> i32>(f.address) };
            assert_eq!(function(0, 3), 6);
        }
    }
*/

#[cfg(test)]
mod test_native_type {
    use super::*;
    use wasmer_types::Type;

    #[test]
    fn test_wasm_types() {
        assert_eq!(i32::WASM_TYPE, Type::I32);
        assert_eq!(i64::WASM_TYPE, Type::I64);
        assert_eq!(f32::WASM_TYPE, Type::F32);
        assert_eq!(f64::WASM_TYPE, Type::F64);
        assert_eq!(u128::WASM_TYPE, Type::V128);
    }
    /*
    #[test]
    fn test_roundtrip() {
        unsafe {
            assert_eq!(i32::from_raw(42i32.into_raw()), 42i32);
            assert_eq!(i64::from_raw(42i64.into_raw()), 42i64);
            assert_eq!(f32::from_raw(42f32.into_raw()), 42f32);
            assert_eq!(f64::from_raw(42f64.into_raw()), 42f64);
            assert_eq!(u128::from_raw(42u128.into_raw()), 42u128);
        }
    }
    */
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
