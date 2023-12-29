use crate::extern_ref::ExternRef;
use crate::lib::std::convert::TryFrom;
use crate::lib::std::fmt;
use crate::lib::std::ptr;
use crate::lib::std::string::{String, ToString};
use crate::types::Type;

/// Possible runtime values that a WebAssembly module can either consume or
/// produce.
#[derive(Clone, PartialEq)]
pub enum Value<T> {
    /// A 32-bit integer.
    ///
    /// In Wasm integers are sign-agnostic, i.e. this can either be signed or unsigned.
    I32(i32),

    /// A 64-bit integer.
    ///
    /// In Wasm integers are sign-agnostic, i.e. this can either be signed or unsigned.
    I64(i64),

    /// A 32-bit float.
    F32(f32),

    /// A 64-bit float.
    F64(f64),

    /// An `externref` value which can hold opaque data to the wasm instance itself.
    ///
    /// Note that this is a nullable value as well.
    ExternRef(ExternRef),

    /// A first-class reference to a WebAssembly function.
    FuncRef(Option<T>),

    /// A 128-bit number
    V128(u128),
}

macro_rules! accessors {
    ($bind:ident $(($variant:ident($ty:ty) $get:ident $unwrap:ident $cvt:expr))*) => ($(
        /// Attempt to access the underlying value of this `Value`, returning
        /// `None` if it is not the correct type.
        pub fn $get(&self) -> Option<$ty> {
            if let Self::$variant($bind) = self {
                Some($cvt)
            } else {
                None
            }
        }

        /// Returns the underlying value of this `Value`, panicking if it's the
        /// wrong type.
        ///
        /// # Panics
        ///
        /// Panics if `self` is not of the right type.
        pub fn $unwrap(&self) -> $ty {
            self.$get().expect(concat!("expected ", stringify!($ty)))
        }
    )*)
}

/// Trait for reading and writing Wasm values into binary for use on the layer
/// between the API and the VM internals, specifically with `wasmer_types::Value`.
pub trait WasmValueType: std::fmt::Debug + 'static {
    /// Write the value
    unsafe fn write_value_to(&self, p: *mut i128);

    /// read the value
    // TODO(reftypes): passing the store as `dyn Any` is a hack to work around the
    // structure of our crates. We need to talk about the store in the rest of the
    // VM (for example where this method is used) but cannot do so. Fixing this
    // may be non-trivial.
    unsafe fn read_value_from(store: &dyn std::any::Any, p: *const i128) -> Self;
}

impl WasmValueType for () {
    unsafe fn write_value_to(&self, _p: *mut i128) {}

    unsafe fn read_value_from(_store: &dyn std::any::Any, _p: *const i128) -> Self {}
}

impl<T> Value<T>
where
    T: WasmValueType,
{
    /// Returns a null `externref` value.
    pub fn null() -> Self {
        Self::ExternRef(ExternRef::null())
    }

    /// Returns the corresponding [`Type`] for this `Value`.
    pub fn ty(&self) -> Type {
        match self {
            Self::I32(_) => Type::I32,
            Self::I64(_) => Type::I64,
            Self::F32(_) => Type::F32,
            Self::F64(_) => Type::F64,
            Self::ExternRef(_) => Type::ExternRef,
            Self::FuncRef(_) => Type::FuncRef,
            Self::V128(_) => Type::V128,
        }
    }

    /// Writes it's value to a given pointer
    ///
    /// # Safety
    /// `p` must be:
    /// - Sufficiently aligned for the Rust equivalent of the type in `self`
    /// - Non-null and pointing to valid, mutable memory
    pub unsafe fn write_value_to(&self, p: *mut i128) {
        match self {
            Self::I32(i) => ptr::write(p as *mut i32, *i),
            Self::I64(i) => ptr::write(p as *mut i64, *i),
            Self::F32(u) => ptr::write(p as *mut f32, *u),
            Self::F64(u) => ptr::write(p as *mut f64, *u),
            Self::V128(b) => ptr::write(p as *mut u128, *b),
            Self::FuncRef(Some(b)) => T::write_value_to(b, p),
            Self::FuncRef(None) => ptr::write(p as *mut usize, 0),
            // TODO(reftypes): review clone here
            Self::ExternRef(extern_ref) => ptr::write(p as *mut ExternRef, extern_ref.clone()),
        }
    }

    /// Gets a `Value` given a pointer and a `Type`
    ///
    /// # Safety
    /// `p` must be:
    /// - Properly aligned to the specified `ty`'s Rust equivalent
    /// - Non-null and pointing to valid memory
    pub unsafe fn read_value_from(store: &dyn std::any::Any, p: *const i128, ty: Type) -> Self {
        match ty {
            Type::I32 => Self::I32(ptr::read(p as *const i32)),
            Type::I64 => Self::I64(ptr::read(p as *const i64)),
            Type::F32 => Self::F32(ptr::read(p as *const f32)),
            Type::F64 => Self::F64(ptr::read(p as *const f64)),
            Type::V128 => Self::V128(ptr::read(p as *const u128)),
            Type::FuncRef => {
                // We do the null check ourselves
                if (*(p as *const usize)) == 0 {
                    Self::FuncRef(None)
                } else {
                    Self::FuncRef(Some(T::read_value_from(store, p)))
                }
            }
            Type::ExternRef => {
                let extern_ref = (&*(p as *const ExternRef)).clone();
                Self::ExternRef(extern_ref)
            }
        }
    }

    accessors! {
        e
        (I32(i32) i32 unwrap_i32 *e)
        (I64(i64) i64 unwrap_i64 *e)
        (F32(f32) f32 unwrap_f32 *e)
        (F64(f64) f64 unwrap_f64 *e)
        (ExternRef(ExternRef) externref unwrap_externref e.clone())
        (FuncRef(&Option<T>) funcref unwrap_funcref e)
        (V128(u128) v128 unwrap_v128 *e)
    }
}

impl<T> fmt::Debug for Value<T>
where
    T: WasmValueType,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32(v) => write!(f, "I32({:?})", v),
            Self::I64(v) => write!(f, "I64({:?})", v),
            Self::F32(v) => write!(f, "F32({:?})", v),
            Self::F64(v) => write!(f, "F64({:?})", v),
            Self::ExternRef(v) => write!(f, "ExternRef({:?})", v),
            Self::FuncRef(None) => write!(f, "Null FuncRef"),
            Self::FuncRef(Some(v)) => write!(f, "FuncRef({:?})", v),
            Self::V128(v) => write!(f, "V128({:?})", v),
        }
    }
}

impl<T> ToString for Value<T>
where
    T: WasmValueType,
{
    fn to_string(&self) -> String {
        match self {
            Self::I32(v) => v.to_string(),
            Self::I64(v) => v.to_string(),
            Self::F32(v) => v.to_string(),
            Self::F64(v) => v.to_string(),
            Self::ExternRef(_) => "externref".to_string(),
            Self::FuncRef(_) => "funcref".to_string(),
            Self::V128(v) => v.to_string(),
        }
    }
}

impl<T> From<i32> for Value<T>
where
    T: WasmValueType,
{
    fn from(val: i32) -> Self {
        Self::I32(val)
    }
}

impl<T> From<u32> for Value<T>
where
    T: WasmValueType,
{
    fn from(val: u32) -> Self {
        // In Wasm integers are sign-agnostic, so i32 is basically a 4 byte storage we can use for signed or unsigned 32-bit integers.
        Self::I32(val as i32)
    }
}

impl<T> From<i64> for Value<T>
where
    T: WasmValueType,
{
    fn from(val: i64) -> Self {
        Self::I64(val)
    }
}

impl<T> From<u64> for Value<T>
where
    T: WasmValueType,
{
    fn from(val: u64) -> Self {
        // In Wasm integers are sign-agnostic, so i64 is basically an 8 byte storage we can use for signed or unsigned 64-bit integers.
        Self::I64(val as i64)
    }
}

impl<T> From<f32> for Value<T>
where
    T: WasmValueType,
{
    fn from(val: f32) -> Self {
        Self::F32(val)
    }
}

impl<T> From<f64> for Value<T>
where
    T: WasmValueType,
{
    fn from(val: f64) -> Self {
        Self::F64(val)
    }
}

impl<T> From<ExternRef> for Value<T>
where
    T: WasmValueType,
{
    fn from(val: ExternRef) -> Self {
        Self::ExternRef(val)
    }
}

// impl<T> From<T> for Value<T> {
//     fn from(val: T) -> Self {
//         Self::FuncRef(val)
//     }
// }

const NOT_I32: &str = "Value is not of Wasm type i32";
const NOT_I64: &str = "Value is not of Wasm type i64";
const NOT_F32: &str = "Value is not of Wasm type f32";
const NOT_F64: &str = "Value is not of Wasm type f64";

impl<T> TryFrom<Value<T>> for i32
where
    T: WasmValueType,
{
    type Error = &'static str;

    fn try_from(value: Value<T>) -> Result<Self, Self::Error> {
        value.i32().ok_or(NOT_I32)
    }
}

impl<T> TryFrom<Value<T>> for u32
where
    T: WasmValueType,
{
    type Error = &'static str;

    fn try_from(value: Value<T>) -> Result<Self, Self::Error> {
        value.i32().ok_or(NOT_I32).map(|int| int as Self)
    }
}

impl<T> TryFrom<Value<T>> for i64
where
    T: WasmValueType,
{
    type Error = &'static str;

    fn try_from(value: Value<T>) -> Result<Self, Self::Error> {
        value.i64().ok_or(NOT_I64)
    }
}

impl<T> TryFrom<Value<T>> for u64
where
    T: WasmValueType,
{
    type Error = &'static str;

    fn try_from(value: Value<T>) -> Result<Self, Self::Error> {
        value.i64().ok_or(NOT_I64).map(|int| int as Self)
    }
}

impl<T> TryFrom<Value<T>> for f32
where
    T: WasmValueType,
{
    type Error = &'static str;

    fn try_from(value: Value<T>) -> Result<Self, Self::Error> {
        value.f32().ok_or(NOT_F32)
    }
}

impl<T> TryFrom<Value<T>> for f64
where
    T: WasmValueType,
{
    type Error = &'static str;

    fn try_from(value: Value<T>) -> Result<Self, Self::Error> {
        value.f64().ok_or(NOT_F64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_i32_from_u32() {
        let bytes = [0x00, 0x00, 0x00, 0x00];
        let v = Value::<()>::from(u32::from_be_bytes(bytes));
        assert_eq!(v, Value::I32(i32::from_be_bytes(bytes)));

        let bytes = [0x00, 0x00, 0x00, 0x01];
        let v = Value::<()>::from(u32::from_be_bytes(bytes));
        assert_eq!(v, Value::I32(i32::from_be_bytes(bytes)));

        let bytes = [0xAA, 0xBB, 0xCC, 0xDD];
        let v = Value::<()>::from(u32::from_be_bytes(bytes));
        assert_eq!(v, Value::I32(i32::from_be_bytes(bytes)));

        let bytes = [0xFF, 0xFF, 0xFF, 0xFF];
        let v = Value::<()>::from(u32::from_be_bytes(bytes));
        assert_eq!(v, Value::I32(i32::from_be_bytes(bytes)));
    }

    #[test]
    fn test_value_i64_from_u64() {
        let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let v = Value::<()>::from(u64::from_be_bytes(bytes));
        assert_eq!(v, Value::I64(i64::from_be_bytes(bytes)));

        let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
        let v = Value::<()>::from(u64::from_be_bytes(bytes));
        assert_eq!(v, Value::I64(i64::from_be_bytes(bytes)));

        let bytes = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11];
        let v = Value::<()>::from(u64::from_be_bytes(bytes));
        assert_eq!(v, Value::I64(i64::from_be_bytes(bytes)));

        let bytes = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let v = Value::<()>::from(u64::from_be_bytes(bytes));
        assert_eq!(v, Value::I64(i64::from_be_bytes(bytes)));
    }

    #[test]
    fn convert_value_to_i32() {
        let value = Value::<()>::I32(5678);
        let result = i32::try_from(value);
        assert_eq!(result.unwrap(), 5678);

        let value = Value::<()>::from(u32::MAX);
        let result = i32::try_from(value);
        assert_eq!(result.unwrap(), -1);

        let value = Value::<()>::V128(42);
        let result = i32::try_from(value);
        assert_eq!(result.unwrap_err(), "Value is not of Wasm type i32");
    }

    #[test]
    fn convert_value_to_u32() {
        let value = Value::<()>::from(u32::MAX);
        let result = u32::try_from(value);
        assert_eq!(result.unwrap(), u32::MAX);

        let value = Value::<()>::I32(-1);
        let result = u32::try_from(value);
        assert_eq!(result.unwrap(), u32::MAX);

        let value = Value::<()>::V128(42);
        let result = u32::try_from(value);
        assert_eq!(result.unwrap_err(), "Value is not of Wasm type i32");
    }

    #[test]
    fn convert_value_to_i64() {
        let value = Value::<()>::I64(5678);
        let result = i64::try_from(value);
        assert_eq!(result.unwrap(), 5678);

        let value = Value::<()>::from(u64::MAX);
        let result = i64::try_from(value);
        assert_eq!(result.unwrap(), -1);

        let value = Value::<()>::V128(42);
        let result = i64::try_from(value);
        assert_eq!(result.unwrap_err(), "Value is not of Wasm type i64");
    }

    #[test]
    fn convert_value_to_u64() {
        let value = Value::<()>::from(u64::MAX);
        let result = u64::try_from(value);
        assert_eq!(result.unwrap(), u64::MAX);

        let value = Value::<()>::I64(-1);
        let result = u64::try_from(value);
        assert_eq!(result.unwrap(), u64::MAX);

        let value = Value::<()>::V128(42);
        let result = u64::try_from(value);
        assert_eq!(result.unwrap_err(), "Value is not of Wasm type i64");
    }

    #[test]
    fn convert_value_to_f32() {
        let value = Value::<()>::F32(1.234);
        let result = f32::try_from(value);
        assert_eq!(result.unwrap(), 1.234);

        let value = Value::<()>::V128(42);
        let result = f32::try_from(value);
        assert_eq!(result.unwrap_err(), "Value is not of Wasm type f32");

        let value = Value::<()>::F64(1.234);
        let result = f32::try_from(value);
        assert_eq!(result.unwrap_err(), "Value is not of Wasm type f32");
    }

    #[test]
    fn convert_value_to_f64() {
        let value = Value::<()>::F64(1.234);
        let result = f64::try_from(value);
        assert_eq!(result.unwrap(), 1.234);

        let value = Value::<()>::V128(42);
        let result = f64::try_from(value);
        assert_eq!(result.unwrap_err(), "Value is not of Wasm type f64");

        let value = Value::<()>::F32(1.234);
        let result = f64::try_from(value);
        assert_eq!(result.unwrap_err(), "Value is not of Wasm type f64");
    }
}
