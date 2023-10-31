//! This is an internal module, no stability guarantees are provided. Use at
//! your own risk.

#![doc(hidden)]

use crate::{Clamped, JsError, JsValue};
use cfg_if::cfg_if;

macro_rules! tys {
    ($($a:ident)*) => (tys! { @ ($($a)*) 0 });
    (@ () $v:expr) => {};
    (@ ($a:ident $($b:ident)*) $v:expr) => {
        pub const $a: u32 = $v;
        tys!(@ ($($b)*) $v+1);
    }
}

// NB: this list must be kept in sync with `crates/cli-support/src/descriptor.rs`
tys! {
    I8
    U8
    I16
    U16
    I32
    U32
    I64
    U64
    F32
    F64
    BOOLEAN
    FUNCTION
    CLOSURE
    CACHED_STRING
    STRING
    REF
    REFMUT
    SLICE
    VECTOR
    EXTERNREF
    NAMED_EXTERNREF
    ENUM
    RUST_STRUCT
    CHAR
    OPTIONAL
    RESULT
    UNIT
    CLAMPED
}

#[inline(always)] // see `interpret.rs` in the the cli-support crate
pub fn inform(a: u32) {
    unsafe { super::__wbindgen_describe(a) }
}

pub trait WasmDescribe {
    fn describe();
}

macro_rules! simple {
    ($($t:ident => $d:ident)*) => ($(
        impl WasmDescribe for $t {
            fn describe() { inform($d) }
        }
    )*)
}

simple! {
    i8 => I8
    u8 => U8
    i16 => I16
    u16 => U16
    i32 => I32
    u32 => U32
    i64 => I64
    u64 => U64
    isize => I32
    usize => U32
    f32 => F32
    f64 => F64
    bool => BOOLEAN
    char => CHAR
    JsValue => EXTERNREF
}

cfg_if! {
    if #[cfg(feature = "enable-interning")] {
        simple! {
            str => CACHED_STRING
        }

    } else {
        simple! {
            str => STRING
        }
    }
}

impl<T> WasmDescribe for *const T {
    fn describe() {
        inform(I32)
    }
}

impl<T> WasmDescribe for *mut T {
    fn describe() {
        inform(I32)
    }
}

impl<T: WasmDescribe> WasmDescribe for [T] {
    fn describe() {
        inform(SLICE);
        T::describe();
    }
}

impl<'a, T: WasmDescribe + ?Sized> WasmDescribe for &'a T {
    fn describe() {
        inform(REF);
        T::describe();
    }
}

impl<'a, T: WasmDescribe + ?Sized> WasmDescribe for &'a mut T {
    fn describe() {
        inform(REFMUT);
        T::describe();
    }
}

if_std! {
    use std::prelude::v1::*;

    cfg_if! {
        if #[cfg(feature = "enable-interning")] {
            simple! {
                String => CACHED_STRING
            }

        } else {
            simple! {
                String => STRING
            }
        }
    }

    impl<T: WasmDescribe> WasmDescribe for Box<[T]> {
        fn describe() {
            inform(VECTOR);
            T::describe();
        }
    }

    impl<T> WasmDescribe for Vec<T> where Box<[T]>: WasmDescribe {
        fn describe() {
            <Box<[T]>>::describe();
        }
    }
}

impl<T: WasmDescribe> WasmDescribe for Option<T> {
    fn describe() {
        inform(OPTIONAL);
        T::describe();
    }
}

impl WasmDescribe for () {
    fn describe() {
        inform(UNIT)
    }
}

impl<T: WasmDescribe, E: Into<JsValue>> WasmDescribe for Result<T, E> {
    fn describe() {
        inform(RESULT);
        T::describe();
    }
}

impl<T: WasmDescribe> WasmDescribe for Clamped<T> {
    fn describe() {
        inform(CLAMPED);
        T::describe();
    }
}

impl WasmDescribe for JsError {
    fn describe() {
        JsValue::describe();
    }
}
