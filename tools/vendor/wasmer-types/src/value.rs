use core::{fmt, marker::PhantomData, mem::MaybeUninit};

/// Raw representation of a WebAssembly value.
///
/// In most cases you will want to use the type-safe `Value` wrapper instead.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union RawValue {
    pub i32: i32,
    pub i64: i64,
    pub u32: u32,
    pub u64: u64,
    pub f32: f32,
    pub f64: f64,
    pub i128: i128,
    pub u128: u128,
    pub funcref: usize,
    pub externref: usize,
    pub bytes: [u8; 16],
}

impl From<i32> for RawValue {
    fn from(value: i32) -> Self {
        Self { i32: value }
    }
}

impl From<i64> for RawValue {
    fn from(value: i64) -> Self {
        Self { i64: value }
    }
}

impl From<f32> for RawValue {
    fn from(value: f32) -> Self {
        Self { f32: value }
    }
}

impl From<f64> for RawValue {
    fn from(value: f64) -> Self {
        Self { f64: value }
    }
}

impl Default for RawValue {
    fn default() -> Self {
        Self { bytes: [0; 16] }
    }
}

impl fmt::Debug for RawValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawValue")
            .field("bytes", unsafe { &self.bytes })
            .finish()
    }
}

macro_rules! partial_eq {
    ($($t:ty => $f:tt),*) => ($(
        impl PartialEq<$t> for RawValue {
            fn eq(&self, o: &$t) -> bool {
                unsafe { self.$f == *o }
            }
        }
    )*)
}

partial_eq! {
    i32 => i32,
    u32 => u32,
    i64 => i64,
    u64 => u64,
    f32 => f32,
    f64 => f64,
    i128 => i128,
    u128 => u128
}

impl PartialEq for RawValue {
    fn eq(&self, o: &Self) -> bool {
        unsafe { self.u128 == o.u128 }
    }
}

/// Trait for a Value type. A Value type is a type that is always valid and may
/// be safely copied.
///
/// # Safety
///
/// To maintain safety, types which implement this trait must be valid for all
/// bit patterns. This means that it cannot contain enums, `bool`, references,
/// etc.
///
/// Concretely a `u32` is a Value type because every combination of 32 bits is
/// a valid `u32`. However a `bool` is _not_ a Value type because any bit patterns
/// other than `0` and `1` are invalid in Rust and may cause undefined behavior if
/// a `bool` is constructed from those bytes.
///
/// Additionally this trait has a method which zeros out any uninitializes bytes
/// prior to writing them to Wasm memory, which prevents information leaks into
/// the sandbox.
pub unsafe trait ValueType: Copy {
    /// This method is passed a byte slice which contains the byte
    /// representation of `self`. It must zero out any bytes which are
    /// uninitialized (e.g. padding bytes).
    fn zero_padding_bytes(&self, bytes: &mut [MaybeUninit<u8>]);
}

// Trivial implementations for primitive types and arrays of them.
macro_rules! primitives {
    ($($t:ident)*) => ($(
        unsafe impl ValueType for $t {
            #[inline]
            fn zero_padding_bytes(&self, _bytes: &mut [MaybeUninit<u8>]) {}
        }
        unsafe impl<const N: usize> ValueType for [$t; N] {
            #[inline]
            fn zero_padding_bytes(&self, _bytes: &mut [MaybeUninit<u8>]) {}
        }
    )*)
}
primitives! {
    bool
    i8 u8
    i16 u16
    i32 u32
    i64 u64
    i128 u128
    isize usize
    f32 f64
}

// This impl for PhantomData allows #[derive(ValueType)] to work with types
// that contain a PhantomData.
unsafe impl<T: ?Sized> ValueType for PhantomData<T> {
    #[inline]
    fn zero_padding_bytes(&self, _bytes: &mut [MaybeUninit<u8>]) {}
}
