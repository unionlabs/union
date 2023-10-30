use core::char;
use core::mem::{self, ManuallyDrop};

use crate::convert::traits::WasmAbi;
use crate::convert::{FromWasmAbi, IntoWasmAbi, LongRefFromWasmAbi, RefFromWasmAbi};
use crate::convert::{OptionFromWasmAbi, OptionIntoWasmAbi, ReturnWasmAbi};
use crate::{Clamped, JsError, JsValue};

unsafe impl WasmAbi for () {}

#[repr(C, u32)]
pub enum WasmOption<T: WasmAbi> {
    None,
    Some(T),
}

unsafe impl<T: WasmAbi> WasmAbi for WasmOption<T> {}

impl<Abi: WasmAbi> WasmOption<Abi> {
    pub fn from_option<T: IntoWasmAbi<Abi = Abi>>(option: Option<T>) -> Self {
        match option {
            Some(v) => WasmOption::Some(v.into_abi()),
            None => WasmOption::None,
        }
    }

    pub unsafe fn into_option<T: FromWasmAbi<Abi = Abi>>(v: Self) -> Option<T> {
        match v {
            WasmOption::Some(v) => Some(T::from_abi(v)),
            WasmOption::None => None,
        }
    }
}

macro_rules! type_wasm_native {
    ($($t:tt as $c:tt)*) => ($(
        impl IntoWasmAbi for $t {
            type Abi = $c;

            #[inline]
            fn into_abi(self) -> $c { self as $c }
        }

        impl FromWasmAbi for $t {
            type Abi = $c;

            #[inline]
            unsafe fn from_abi(js: $c) -> Self { js as $t }
        }

        impl IntoWasmAbi for Option<$t> {
            type Abi = WasmOption<$c>;

            #[inline]
            fn into_abi(self) -> Self::Abi {
                WasmOption::from_option(self.map(|v| v as $c))
            }
        }

        impl FromWasmAbi for Option<$t> {
            type Abi = WasmOption<$c>;

            #[inline]
            unsafe fn from_abi(js: Self::Abi) -> Self {
                WasmOption::into_option(js).map(|v: $c| v as $t)
            }
        }
    )*)
}

type_wasm_native!(
    i32 as i32
    isize as i32
    u32 as u32
    usize as u32
    i64 as i64
    u64 as u64
    f32 as f32
    f64 as f64
);

macro_rules! type_abi_as_u32 {
    ($($t:tt)*) => ($(
        impl IntoWasmAbi for $t {
            type Abi = u32;

            #[inline]
            fn into_abi(self) -> u32 { self as u32 }
        }

        impl FromWasmAbi for $t {
            type Abi = u32;

            #[inline]
            unsafe fn from_abi(js: u32) -> Self { js as $t }
        }

        impl OptionIntoWasmAbi for $t {
            #[inline]
            fn none() -> u32 { 0x00FF_FFFFu32 }
        }

        impl OptionFromWasmAbi for $t {
            #[inline]
            fn is_none(js: &u32) -> bool { *js == 0x00FF_FFFFu32 }
        }
    )*)
}

type_abi_as_u32!(i8 u8 i16 u16);

impl IntoWasmAbi for bool {
    type Abi = u32;

    #[inline]
    fn into_abi(self) -> u32 {
        self as u32
    }
}

impl FromWasmAbi for bool {
    type Abi = u32;

    #[inline]
    unsafe fn from_abi(js: u32) -> bool {
        js != 0
    }
}

impl OptionIntoWasmAbi for bool {
    #[inline]
    fn none() -> u32 {
        0x00FF_FFFFu32
    }
}

impl OptionFromWasmAbi for bool {
    #[inline]
    fn is_none(js: &u32) -> bool {
        *js == 0x00FF_FFFFu32
    }
}

impl IntoWasmAbi for char {
    type Abi = u32;

    #[inline]
    fn into_abi(self) -> u32 {
        self as u32
    }
}

impl FromWasmAbi for char {
    type Abi = u32;

    #[inline]
    unsafe fn from_abi(js: u32) -> char {
        char::from_u32_unchecked(js)
    }
}

impl OptionIntoWasmAbi for char {
    #[inline]
    fn none() -> u32 {
        0x00FF_FFFFu32
    }
}

impl OptionFromWasmAbi for char {
    #[inline]
    fn is_none(js: &u32) -> bool {
        *js == 0x00FF_FFFFu32
    }
}

impl<T> IntoWasmAbi for *const T {
    type Abi = u32;

    #[inline]
    fn into_abi(self) -> u32 {
        self as u32
    }
}

impl<T> FromWasmAbi for *const T {
    type Abi = u32;

    #[inline]
    unsafe fn from_abi(js: u32) -> *const T {
        js as *const T
    }
}

impl<T> IntoWasmAbi for *mut T {
    type Abi = u32;

    #[inline]
    fn into_abi(self) -> u32 {
        self as u32
    }
}

impl<T> FromWasmAbi for *mut T {
    type Abi = u32;

    #[inline]
    unsafe fn from_abi(js: u32) -> *mut T {
        js as *mut T
    }
}

impl IntoWasmAbi for JsValue {
    type Abi = u32;

    #[inline]
    fn into_abi(self) -> u32 {
        let ret = self.idx;
        mem::forget(self);
        ret
    }
}

impl FromWasmAbi for JsValue {
    type Abi = u32;

    #[inline]
    unsafe fn from_abi(js: u32) -> JsValue {
        JsValue::_new(js)
    }
}

impl<'a> IntoWasmAbi for &'a JsValue {
    type Abi = u32;

    #[inline]
    fn into_abi(self) -> u32 {
        self.idx
    }
}

impl RefFromWasmAbi for JsValue {
    type Abi = u32;
    type Anchor = ManuallyDrop<JsValue>;

    #[inline]
    unsafe fn ref_from_abi(js: u32) -> Self::Anchor {
        ManuallyDrop::new(JsValue::_new(js))
    }
}

impl LongRefFromWasmAbi for JsValue {
    type Abi = u32;
    type Anchor = JsValue;

    #[inline]
    unsafe fn long_ref_from_abi(js: u32) -> Self::Anchor {
        Self::from_abi(js)
    }
}

impl<T: OptionIntoWasmAbi> IntoWasmAbi for Option<T> {
    type Abi = T::Abi;

    #[inline]
    fn into_abi(self) -> T::Abi {
        match self {
            None => T::none(),
            Some(me) => me.into_abi(),
        }
    }
}

impl<T: OptionFromWasmAbi> FromWasmAbi for Option<T> {
    type Abi = T::Abi;

    #[inline]
    unsafe fn from_abi(js: T::Abi) -> Self {
        if T::is_none(&js) {
            None
        } else {
            Some(T::from_abi(js))
        }
    }
}

impl<T: IntoWasmAbi> IntoWasmAbi for Clamped<T> {
    type Abi = T::Abi;

    #[inline]
    fn into_abi(self) -> Self::Abi {
        self.0.into_abi()
    }
}

impl<T: FromWasmAbi> FromWasmAbi for Clamped<T> {
    type Abi = T::Abi;

    #[inline]
    unsafe fn from_abi(js: T::Abi) -> Self {
        Clamped(T::from_abi(js))
    }
}

impl IntoWasmAbi for () {
    type Abi = ();

    #[inline]
    fn into_abi(self) {
        self
    }
}

/// This is an encoding of a Result. It can only store things that can be decoded by the JS
/// bindings.
///
/// At the moment, we do not write the exact struct packing layout of everything into the
/// glue/descriptions of datatypes, so T cannot be arbitrary. The current requirements of the
/// struct unpacker (StructUnpacker), which apply to ResultAbi<T> as a whole, are as follows:
///
/// - repr(C), of course
/// - u32/i32/f32/f64 fields at the "leaf fields" of the "field tree"
/// - layout equivalent to a completely flattened repr(C) struct, constructed by an in order
///   traversal of all the leaf fields in it.
///  
/// This means that you can't embed struct A(u32, f64) as struct B(u32, A); because the "completely
/// flattened" struct AB(u32, u32, f64) would miss the 4 byte padding that is actually present
/// within B and then as a consequence also miss the 4 byte padding within A that repr(C) inserts.
///
/// The enemy is padding. Padding is only required when there is an `f64` field. So the enemy is
/// `f64` after anything else, particularly anything arbitrary. There is no smaller sized type, so
/// we don't need to worry about 1-byte integers, etc. It's best, therefore, to place your f64s
/// first in your structs, that's why we have `abi` first, although here it doesn't matter as the
/// other two fields total 8 bytes anyway.
///
#[repr(C)]
pub struct ResultAbi<T> {
    /// This field is the same size/align as `T`.
    abi: ResultAbiUnion<T>,
    /// Order of args here is such that we can pop() the possible error first, deal with it and
    /// move on. Later fields are popped off the stack first.
    err: u32,
    is_err: u32,
}

#[repr(C)]
pub union ResultAbiUnion<T> {
    // ManuallyDrop is #[repr(transparent)]
    ok: std::mem::ManuallyDrop<T>,
    err: (),
}

unsafe impl<T: WasmAbi> WasmAbi for ResultAbi<T> {}
unsafe impl<T: WasmAbi> WasmAbi for ResultAbiUnion<T> {}

impl<T: IntoWasmAbi, E: Into<JsValue>> ReturnWasmAbi for Result<T, E> {
    type Abi = ResultAbi<T::Abi>;
    #[inline]
    fn return_abi(self) -> Self::Abi {
        match self {
            Ok(v) => {
                let abi = ResultAbiUnion {
                    ok: std::mem::ManuallyDrop::new(v.into_abi()),
                };
                ResultAbi {
                    abi,
                    is_err: 0,
                    err: 0,
                }
            }
            Err(e) => {
                let jsval = e.into();
                ResultAbi {
                    abi: ResultAbiUnion { err: () },
                    is_err: 1,
                    err: jsval.into_abi(),
                }
            }
        }
    }
}

impl IntoWasmAbi for JsError {
    type Abi = <JsValue as IntoWasmAbi>::Abi;

    fn into_abi(self) -> Self::Abi {
        self.value.into_abi()
    }
}
