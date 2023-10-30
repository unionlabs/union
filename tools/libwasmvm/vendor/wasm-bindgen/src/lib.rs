//! Runtime support for the `wasm-bindgen` tool
//!
//! This crate contains the runtime support necessary for `wasm-bindgen` the
//! attribute and tool. Crates pull in the `#[wasm_bindgen]` attribute through
//! this crate and this crate also provides JS bindings through the `JsValue`
//! interface.

#![no_std]
#![allow(coherence_leak_check)]
#![doc(html_root_url = "https://docs.rs/wasm-bindgen/0.2")]

use core::convert::TryFrom;
use core::fmt;
use core::marker;
use core::mem;
use core::ops::{
    Add, BitAnd, BitOr, BitXor, Deref, DerefMut, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub,
};
use core::u32;

use crate::convert::{FromWasmAbi, WasmSlice};

macro_rules! if_std {
    ($($i:item)*) => ($(
        #[cfg(feature = "std")] $i
    )*)
}

macro_rules! externs {
    ($(#[$attr:meta])* extern "C" { $(fn $name:ident($($args:tt)*) -> $ret:ty;)* }) => (
        #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
        $(#[$attr])*
        extern "C" {
            $(fn $name($($args)*) -> $ret;)*
        }

        $(
            #[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
            #[allow(unused_variables)]
            unsafe extern fn $name($($args)*) -> $ret {
                panic!("function not implemented on non-wasm32 targets")
            }
        )*
    )
}

/// A module which is typically glob imported.
///
/// ```
/// use wasm_bindgen::prelude::*;
/// ```
pub mod prelude {
    pub use crate::JsCast;
    pub use crate::JsValue;
    pub use crate::UnwrapThrowExt;
    #[doc(hidden)]
    pub use wasm_bindgen_macro::__wasm_bindgen_class_marker;
    pub use wasm_bindgen_macro::wasm_bindgen;

    if_std! {
        pub use crate::closure::Closure;
    }

    pub use crate::JsError;
}

pub use wasm_bindgen_macro::link_to;

pub mod convert;
pub mod describe;

mod cast;
pub use crate::cast::{JsCast, JsObject};
use convert::WasmOption;

if_std! {
    extern crate std;
    use std::prelude::v1::*;
    pub mod closure;
    mod externref;

    mod cache;
    pub use cache::intern::{intern, unintern};
}

/// Representation of an object owned by JS.
///
/// A `JsValue` doesn't actually live in Rust right now but actually in a table
/// owned by the `wasm-bindgen` generated JS glue code. Eventually the ownership
/// will transfer into wasm directly and this will likely become more efficient,
/// but for now it may be slightly slow.
pub struct JsValue {
    idx: u32,
    _marker: marker::PhantomData<*mut u8>, // not at all threadsafe
}

const JSIDX_OFFSET: u32 = 128; // keep in sync with js/mod.rs
const JSIDX_UNDEFINED: u32 = JSIDX_OFFSET;
const JSIDX_NULL: u32 = JSIDX_OFFSET + 1;
const JSIDX_TRUE: u32 = JSIDX_OFFSET + 2;
const JSIDX_FALSE: u32 = JSIDX_OFFSET + 3;
const JSIDX_RESERVED: u32 = JSIDX_OFFSET + 4;

impl JsValue {
    /// The `null` JS value constant.
    pub const NULL: JsValue = JsValue::_new(JSIDX_NULL);

    /// The `undefined` JS value constant.
    pub const UNDEFINED: JsValue = JsValue::_new(JSIDX_UNDEFINED);

    /// The `true` JS value constant.
    pub const TRUE: JsValue = JsValue::_new(JSIDX_TRUE);

    /// The `false` JS value constant.
    pub const FALSE: JsValue = JsValue::_new(JSIDX_FALSE);

    #[inline]
    const fn _new(idx: u32) -> JsValue {
        JsValue {
            idx,
            _marker: marker::PhantomData,
        }
    }

    /// Creates a new JS value which is a string.
    ///
    /// The utf-8 string provided is copied to the JS heap and the string will
    /// be owned by the JS garbage collector.
    #[allow(clippy::should_implement_trait)] // cannot fix without breaking change
    #[inline]
    pub fn from_str(s: &str) -> JsValue {
        unsafe { JsValue::_new(__wbindgen_string_new(s.as_ptr(), s.len())) }
    }

    /// Creates a new JS value which is a number.
    ///
    /// This function creates a JS value representing a number (a heap
    /// allocated number) and returns a handle to the JS version of it.
    #[inline]
    pub fn from_f64(n: f64) -> JsValue {
        unsafe { JsValue::_new(__wbindgen_number_new(n)) }
    }

    /// Creates a new JS value which is a bigint from a string representing a number.
    ///
    /// This function creates a JS value representing a bigint (a heap
    /// allocated large integer) and returns a handle to the JS version of it.
    #[inline]
    pub fn bigint_from_str(s: &str) -> JsValue {
        unsafe { JsValue::_new(__wbindgen_bigint_from_str(s.as_ptr(), s.len())) }
    }

    /// Creates a new JS value which is a boolean.
    ///
    /// This function creates a JS object representing a boolean (a heap
    /// allocated boolean) and returns a handle to the JS version of it.
    #[inline]
    pub const fn from_bool(b: bool) -> JsValue {
        if b {
            JsValue::TRUE
        } else {
            JsValue::FALSE
        }
    }

    /// Creates a new JS value representing `undefined`.
    #[inline]
    pub const fn undefined() -> JsValue {
        JsValue::UNDEFINED
    }

    /// Creates a new JS value representing `null`.
    #[inline]
    pub const fn null() -> JsValue {
        JsValue::NULL
    }

    /// Creates a new JS symbol with the optional description specified.
    ///
    /// This function will invoke the `Symbol` constructor in JS and return the
    /// JS object corresponding to the symbol created.
    pub fn symbol(description: Option<&str>) -> JsValue {
        unsafe {
            match description {
                Some(description) => JsValue::_new(__wbindgen_symbol_named_new(
                    description.as_ptr(),
                    description.len(),
                )),
                None => JsValue::_new(__wbindgen_symbol_anonymous_new()),
            }
        }
    }

    /// Creates a new `JsValue` from the JSON serialization of the object `t`
    /// provided.
    ///
    /// **This function is deprecated**, due to [creating a dependency cycle in
    /// some circumstances][dep-cycle-issue]. Use [`serde-wasm-bindgen`] or
    /// [`gloo_utils::format::JsValueSerdeExt`] instead.
    ///
    /// [dep-cycle-issue]: https://github.com/rustwasm/wasm-bindgen/issues/2770
    /// [`serde-wasm-bindgen`]: https://docs.rs/serde-wasm-bindgen
    /// [`gloo_utils::format::JsValueSerdeExt`]: https://docs.rs/gloo-utils/latest/gloo_utils/format/trait.JsValueSerdeExt.html
    ///
    /// This function will serialize the provided value `t` to a JSON string,
    /// send the JSON string to JS, parse it into a JS object, and then return
    /// a handle to the JS object. This is unlikely to be super speedy so it's
    /// not recommended for large payloads, but it's a nice to have in some
    /// situations!
    ///
    /// Usage of this API requires activating the `serde-serialize` feature of
    /// the `wasm-bindgen` crate.
    ///
    /// # Errors
    ///
    /// Returns any error encountered when serializing `T` into JSON.
    #[cfg(feature = "serde-serialize")]
    #[deprecated = "causes dependency cycles, use `serde-wasm-bindgen` or `gloo_utils::format::JsValueSerdeExt` instead"]
    pub fn from_serde<T>(t: &T) -> serde_json::Result<JsValue>
    where
        T: serde::ser::Serialize + ?Sized,
    {
        let s = serde_json::to_string(t)?;
        unsafe { Ok(JsValue::_new(__wbindgen_json_parse(s.as_ptr(), s.len()))) }
    }

    /// Invokes `JSON.stringify` on this value and then parses the resulting
    /// JSON into an arbitrary Rust value.
    ///
    /// **This function is deprecated**, due to [creating a dependency cycle in
    /// some circumstances][dep-cycle-issue]. Use [`serde-wasm-bindgen`] or
    /// [`gloo_utils::format::JsValueSerdeExt`] instead.
    ///
    /// [dep-cycle-issue]: https://github.com/rustwasm/wasm-bindgen/issues/2770
    /// [`serde-wasm-bindgen`]: https://docs.rs/serde-wasm-bindgen
    /// [`gloo_utils::format::JsValueSerdeExt`]: https://docs.rs/gloo-utils/latest/gloo_utils/format/trait.JsValueSerdeExt.html
    ///
    /// This function will first call `JSON.stringify` on the `JsValue` itself.
    /// The resulting string is then passed into Rust which then parses it as
    /// JSON into the resulting value.
    ///
    /// Usage of this API requires activating the `serde-serialize` feature of
    /// the `wasm-bindgen` crate.
    ///
    /// # Errors
    ///
    /// Returns any error encountered when parsing the JSON into a `T`.
    #[cfg(feature = "serde-serialize")]
    #[deprecated = "causes dependency cycles, use `serde-wasm-bindgen` or `gloo_utils::format::JsValueSerdeExt` instead"]
    pub fn into_serde<T>(&self) -> serde_json::Result<T>
    where
        T: for<'a> serde::de::Deserialize<'a>,
    {
        unsafe {
            let ret = __wbindgen_json_serialize(self.idx);
            let s = String::from_abi(ret);
            serde_json::from_str(&s)
        }
    }

    /// Returns the `f64` value of this JS value if it's an instance of a
    /// number.
    ///
    /// If this JS value is not an instance of a number then this returns
    /// `None`.
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        unsafe { FromWasmAbi::from_abi(__wbindgen_number_get(self.idx)) }
    }

    /// Tests whether this JS value is a JS string.
    #[inline]
    pub fn is_string(&self) -> bool {
        unsafe { __wbindgen_is_string(self.idx) == 1 }
    }

    /// If this JS value is a string value, this function copies the JS string
    /// value into wasm linear memory, encoded as UTF-8, and returns it as a
    /// Rust `String`.
    ///
    /// To avoid the copying and re-encoding, consider the
    /// `JsString::try_from()` function from [js-sys](https://docs.rs/js-sys)
    /// instead.
    ///
    /// If this JS value is not an instance of a string or if it's not valid
    /// utf-8 then this returns `None`.
    ///
    /// # UTF-16 vs UTF-8
    ///
    /// JavaScript strings in general are encoded as UTF-16, but Rust strings
    /// are encoded as UTF-8. This can cause the Rust string to look a bit
    /// different than the JS string sometimes. For more details see the
    /// [documentation about the `str` type][caveats] which contains a few
    /// caveats about the encodings.
    ///
    /// [caveats]: https://rustwasm.github.io/docs/wasm-bindgen/reference/types/str.html
    #[cfg(feature = "std")]
    #[inline]
    pub fn as_string(&self) -> Option<String> {
        unsafe { FromWasmAbi::from_abi(__wbindgen_string_get(self.idx)) }
    }

    /// Returns the `bool` value of this JS value if it's an instance of a
    /// boolean.
    ///
    /// If this JS value is not an instance of a boolean then this returns
    /// `None`.
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        unsafe {
            match __wbindgen_boolean_get(self.idx) {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            }
        }
    }

    /// Tests whether this JS value is `null`
    #[inline]
    pub fn is_null(&self) -> bool {
        unsafe { __wbindgen_is_null(self.idx) == 1 }
    }

    /// Tests whether this JS value is `undefined`
    #[inline]
    pub fn is_undefined(&self) -> bool {
        unsafe { __wbindgen_is_undefined(self.idx) == 1 }
    }

    /// Tests whether the type of this JS value is `symbol`
    #[inline]
    pub fn is_symbol(&self) -> bool {
        unsafe { __wbindgen_is_symbol(self.idx) == 1 }
    }

    /// Tests whether `typeof self == "object" && self !== null`.
    #[inline]
    pub fn is_object(&self) -> bool {
        unsafe { __wbindgen_is_object(self.idx) == 1 }
    }

    /// Tests whether this JS value is an instance of Array.
    #[inline]
    pub fn is_array(&self) -> bool {
        unsafe { __wbindgen_is_array(self.idx) == 1 }
    }

    /// Tests whether the type of this JS value is `function`.
    #[inline]
    pub fn is_function(&self) -> bool {
        unsafe { __wbindgen_is_function(self.idx) == 1 }
    }

    /// Tests whether the type of this JS value is `bigint`.
    #[inline]
    pub fn is_bigint(&self) -> bool {
        unsafe { __wbindgen_is_bigint(self.idx) == 1 }
    }

    /// Applies the unary `typeof` JS operator on a `JsValue`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof)
    #[inline]
    pub fn js_typeof(&self) -> JsValue {
        unsafe { JsValue::_new(__wbindgen_typeof(self.idx)) }
    }

    /// Applies the binary `in` JS operator on the two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/in)
    #[inline]
    pub fn js_in(&self, obj: &JsValue) -> bool {
        unsafe { __wbindgen_in(self.idx, obj.idx) == 1 }
    }

    /// Tests whether the value is ["truthy"].
    ///
    /// ["truthy"]: https://developer.mozilla.org/en-US/docs/Glossary/Truthy
    #[inline]
    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }

    /// Tests whether the value is ["falsy"].
    ///
    /// ["falsy"]: https://developer.mozilla.org/en-US/docs/Glossary/Falsy
    #[inline]
    pub fn is_falsy(&self) -> bool {
        unsafe { __wbindgen_is_falsy(self.idx) == 1 }
    }

    /// Get a string representation of the JavaScript object for debugging.
    #[cfg(feature = "std")]
    fn as_debug_string(&self) -> String {
        unsafe {
            let mut ret = [0; 2];
            __wbindgen_debug_string(&mut ret, self.idx);
            let data = Vec::from_raw_parts(ret[0] as *mut u8, ret[1], ret[1]);
            String::from_utf8_unchecked(data)
        }
    }

    /// Compare two `JsValue`s for equality, using the `==` operator in JS.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Equality)
    #[inline]
    pub fn loose_eq(&self, other: &Self) -> bool {
        unsafe { __wbindgen_jsval_loose_eq(self.idx, other.idx) != 0 }
    }

    /// Applies the unary `~` JS operator on a `JsValue`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_NOT)
    #[inline]
    pub fn bit_not(&self) -> JsValue {
        unsafe { JsValue::_new(__wbindgen_bit_not(self.idx)) }
    }

    /// Applies the binary `>>>` JS operator on the two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unsigned_right_shift)
    #[inline]
    pub fn unsigned_shr(&self, rhs: &Self) -> u32 {
        unsafe { __wbindgen_unsigned_shr(self.idx, rhs.idx) }
    }

    /// Applies the binary `/` JS operator on two `JsValue`s, catching and returning any `RangeError` thrown.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Division)
    #[inline]
    pub fn checked_div(&self, rhs: &Self) -> Self {
        unsafe { JsValue::_new(__wbindgen_checked_div(self.idx, rhs.idx)) }
    }

    /// Applies the binary `**` JS operator on the two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Exponentiation)
    #[inline]
    pub fn pow(&self, rhs: &Self) -> Self {
        unsafe { JsValue::_new(__wbindgen_pow(self.idx, rhs.idx)) }
    }

    /// Applies the binary `<` JS operator on the two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Less_than)
    #[inline]
    pub fn lt(&self, other: &Self) -> bool {
        unsafe { __wbindgen_lt(self.idx, other.idx) == 1 }
    }

    /// Applies the binary `<=` JS operator on the two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Less_than_or_equal)
    #[inline]
    pub fn le(&self, other: &Self) -> bool {
        unsafe { __wbindgen_le(self.idx, other.idx) == 1 }
    }

    /// Applies the binary `>=` JS operator on the two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Greater_than_or_equal)
    #[inline]
    pub fn ge(&self, other: &Self) -> bool {
        unsafe { __wbindgen_ge(self.idx, other.idx) == 1 }
    }

    /// Applies the binary `>` JS operator on the two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Greater_than)
    #[inline]
    pub fn gt(&self, other: &Self) -> bool {
        unsafe { __wbindgen_gt(self.idx, other.idx) == 1 }
    }

    /// Applies the unary `+` JS operator on a `JsValue`. Can throw.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unary_plus)
    #[inline]
    pub fn unchecked_into_f64(&self) -> f64 {
        unsafe { __wbindgen_as_number(self.idx) }
    }
}

impl PartialEq for JsValue {
    /// Compares two `JsValue`s for equality, using the `===` operator in JS.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Strict_equality)
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        unsafe { __wbindgen_jsval_eq(self.idx, other.idx) != 0 }
    }
}

impl PartialEq<bool> for JsValue {
    #[inline]
    fn eq(&self, other: &bool) -> bool {
        self.as_bool() == Some(*other)
    }
}

impl PartialEq<str> for JsValue {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        *self == JsValue::from_str(other)
    }
}

impl<'a> PartialEq<&'a str> for JsValue {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        <JsValue as PartialEq<str>>::eq(self, other)
    }
}

if_std! {
    impl PartialEq<String> for JsValue {
        #[inline]
        fn eq(&self, other: &String) -> bool {
            <JsValue as PartialEq<str>>::eq(self, other)
        }
    }
    impl<'a> PartialEq<&'a String> for JsValue {
        #[inline]
        fn eq(&self, other: &&'a String) -> bool {
            <JsValue as PartialEq<str>>::eq(self, other)
        }
    }
}

macro_rules! forward_deref_unop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        impl $imp for $t {
            type Output = <&'static $t as $imp>::Output;

            #[inline]
            fn $method(self) -> <&'static $t as $imp>::Output {
                $imp::$method(&self)
            }
        }
    };
}

macro_rules! forward_deref_binop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        impl<'a> $imp<$t> for &'a $t {
            type Output = <&'static $t as $imp<&'static $t>>::Output;

            #[inline]
            fn $method(self, other: $t) -> <&'static $t as $imp<&'static $t>>::Output {
                $imp::$method(self, &other)
            }
        }

        impl $imp<&$t> for $t {
            type Output = <&'static $t as $imp<&'static $t>>::Output;

            #[inline]
            fn $method(self, other: &$t) -> <&'static $t as $imp<&'static $t>>::Output {
                $imp::$method(&self, other)
            }
        }

        impl $imp<$t> for $t {
            type Output = <&'static $t as $imp<&'static $t>>::Output;

            #[inline]
            fn $method(self, other: $t) -> <&'static $t as $imp<&'static $t>>::Output {
                $imp::$method(&self, &other)
            }
        }
    };
}

impl Not for &JsValue {
    type Output = bool;

    /// Applies the `!` JS operator on a `JsValue`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_NOT)
    #[inline]
    fn not(self) -> Self::Output {
        JsValue::is_falsy(self)
    }
}

forward_deref_unop!(impl Not, not for JsValue);

impl TryFrom<JsValue> for f64 {
    type Error = JsValue;

    /// Applies the unary `+` JS operator on a `JsValue`.
    /// Returns the numeric result on success, or the JS error value on error.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unary_plus)
    #[inline]
    fn try_from(val: JsValue) -> Result<Self, Self::Error> {
        f64::try_from(&val)
    }
}

impl TryFrom<&JsValue> for f64 {
    type Error = JsValue;

    /// Applies the unary `+` JS operator on a `JsValue`.
    /// Returns the numeric result on success, or the JS error value on error.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unary_plus)
    #[inline]
    fn try_from(val: &JsValue) -> Result<Self, Self::Error> {
        let jsval = unsafe { JsValue::_new(__wbindgen_try_into_number(val.idx)) };
        match jsval.as_f64() {
            Some(num) => Ok(num),
            None => Err(jsval),
        }
    }
}

impl Neg for &JsValue {
    type Output = JsValue;

    /// Applies the unary `-` JS operator on a `JsValue`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unary_negation)
    #[inline]
    fn neg(self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_neg(self.idx)) }
    }
}

forward_deref_unop!(impl Neg, neg for JsValue);

impl BitAnd for &JsValue {
    type Output = JsValue;

    /// Applies the binary `&` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_AND)
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_bit_and(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl BitAnd, bitand for JsValue);

impl BitOr for &JsValue {
    type Output = JsValue;

    /// Applies the binary `|` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_OR)
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_bit_or(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl BitOr, bitor for JsValue);

impl BitXor for &JsValue {
    type Output = JsValue;

    /// Applies the binary `^` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_XOR)
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_bit_xor(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl BitXor, bitxor for JsValue);

impl Shl for &JsValue {
    type Output = JsValue;

    /// Applies the binary `<<` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Left_shift)
    #[inline]
    fn shl(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_shl(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl Shl, shl for JsValue);

impl Shr for &JsValue {
    type Output = JsValue;

    /// Applies the binary `>>` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Right_shift)
    #[inline]
    fn shr(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_shr(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl Shr, shr for JsValue);

impl Add for &JsValue {
    type Output = JsValue;

    /// Applies the binary `+` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Addition)
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_add(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl Add, add for JsValue);

impl Sub for &JsValue {
    type Output = JsValue;

    /// Applies the binary `-` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Subtraction)
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_sub(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl Sub, sub for JsValue);

impl Div for &JsValue {
    type Output = JsValue;

    /// Applies the binary `/` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Division)
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_div(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl Div, div for JsValue);

impl Mul for &JsValue {
    type Output = JsValue;

    /// Applies the binary `*` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Multiplication)
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_mul(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl Mul, mul for JsValue);

impl Rem for &JsValue {
    type Output = JsValue;

    /// Applies the binary `%` JS operator on two `JsValue`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Remainder)
    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        unsafe { JsValue::_new(__wbindgen_rem(self.idx, rhs.idx)) }
    }
}

forward_deref_binop!(impl Rem, rem for JsValue);

impl<'a> From<&'a str> for JsValue {
    #[inline]
    fn from(s: &'a str) -> JsValue {
        JsValue::from_str(s)
    }
}

impl<T> From<*mut T> for JsValue {
    #[inline]
    fn from(s: *mut T) -> JsValue {
        JsValue::from(s as usize)
    }
}

impl<T> From<*const T> for JsValue {
    #[inline]
    fn from(s: *const T) -> JsValue {
        JsValue::from(s as usize)
    }
}

if_std! {
    impl<'a> From<&'a String> for JsValue {
        #[inline]
        fn from(s: &'a String) -> JsValue {
            JsValue::from_str(s)
        }
    }

    impl From<String> for JsValue {
        #[inline]
        fn from(s: String) -> JsValue {
            JsValue::from_str(&s)
        }
    }
}

impl From<bool> for JsValue {
    #[inline]
    fn from(s: bool) -> JsValue {
        JsValue::from_bool(s)
    }
}

impl<'a, T> From<&'a T> for JsValue
where
    T: JsCast,
{
    #[inline]
    fn from(s: &'a T) -> JsValue {
        s.as_ref().clone()
    }
}

impl<T> From<Option<T>> for JsValue
where
    JsValue: From<T>,
{
    #[inline]
    fn from(s: Option<T>) -> JsValue {
        match s {
            Some(s) => s.into(),
            None => JsValue::undefined(),
        }
    }
}

impl JsCast for JsValue {
    // everything is a `JsValue`!
    #[inline]
    fn instanceof(_val: &JsValue) -> bool {
        true
    }
    #[inline]
    fn unchecked_from_js(val: JsValue) -> Self {
        val
    }
    #[inline]
    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        val
    }
}

impl AsRef<JsValue> for JsValue {
    #[inline]
    fn as_ref(&self) -> &JsValue {
        self
    }
}

macro_rules! numbers {
    ($($n:ident)*) => ($(
        impl PartialEq<$n> for JsValue {
            #[inline]
            fn eq(&self, other: &$n) -> bool {
                self.as_f64() == Some(f64::from(*other))
            }
        }

        impl From<$n> for JsValue {
            #[inline]
            fn from(n: $n) -> JsValue {
                JsValue::from_f64(n.into())
            }
        }
    )*)
}

numbers! { i8 u8 i16 u16 i32 u32 f32 f64 }

macro_rules! big_numbers {
    (|$arg:ident|, $($n:ident = $handle:expr,)*) => ($(
        impl PartialEq<$n> for JsValue {
            #[inline]
            fn eq(&self, other: &$n) -> bool {
                self == &JsValue::from(*other)
            }
        }

        impl From<$n> for JsValue {
            #[inline]
            fn from($arg: $n) -> JsValue {
                unsafe { JsValue::_new($handle) }
            }
        }
    )*)
}

fn bigint_get_as_i64(v: &JsValue) -> Option<i64> {
    unsafe { Option::from_abi(__wbindgen_bigint_get_as_i64(v.idx)) }
}

macro_rules! try_from_for_num64 {
    ($ty:ty) => {
        impl TryFrom<JsValue> for $ty {
            type Error = JsValue;

            #[inline]
            fn try_from(v: JsValue) -> Result<Self, JsValue> {
                bigint_get_as_i64(&v)
                    // Reinterpret bits; ABI-wise this is safe to do and allows us to avoid
                    // having separate intrinsics per signed/unsigned types.
                    .map(|as_i64| as_i64 as Self)
                    // Double-check that we didn't truncate the bigint to 64 bits.
                    .filter(|as_self| v == *as_self)
                    // Not a bigint or not in range.
                    .ok_or(v)
            }
        }
    };
}

try_from_for_num64!(i64);
try_from_for_num64!(u64);

macro_rules! try_from_for_num128 {
    ($ty:ty, $hi_ty:ty) => {
        impl TryFrom<JsValue> for $ty {
            type Error = JsValue;

            #[inline]
            fn try_from(v: JsValue) -> Result<Self, JsValue> {
                // Truncate the bigint to 64 bits, this will give us the lower part.
                let lo = match bigint_get_as_i64(&v) {
                    // The lower part must be interpreted as unsigned in both i128 and u128.
                    Some(lo) => lo as u64,
                    // Not a bigint.
                    None => return Err(v),
                };
                // Now we know it's a bigint, so we can safely use `>> 64n` without
                // worrying about a JS exception on type mismatch.
                let hi = v >> JsValue::from(64_u64);
                // The high part is the one we want checked against a 64-bit range.
                // If it fits, then our original number is in the 128-bit range.
                let hi = <$hi_ty>::try_from(hi)?;
                Ok(Self::from(hi) << 64 | Self::from(lo))
            }
        }
    };
}

try_from_for_num128!(i128, i64);
try_from_for_num128!(u128, u64);

big_numbers! {
    |n|,
    i64 = __wbindgen_bigint_from_i64(n),
    u64 = __wbindgen_bigint_from_u64(n),
    i128 = __wbindgen_bigint_from_i128((n >> 64) as i64, n as u64),
    u128 = __wbindgen_bigint_from_u128((n >> 64) as u64, n as u64),
}

// `usize` and `isize` have to be treated a bit specially, because we know that
// they're 32-bit but the compiler conservatively assumes they might be bigger.
// So, we have to manually forward to the `u32`/`i32` versions.
impl PartialEq<usize> for JsValue {
    #[inline]
    fn eq(&self, other: &usize) -> bool {
        *self == (*other as u32)
    }
}

impl From<usize> for JsValue {
    #[inline]
    fn from(n: usize) -> Self {
        Self::from(n as u32)
    }
}

impl PartialEq<isize> for JsValue {
    #[inline]
    fn eq(&self, other: &isize) -> bool {
        *self == (*other as i32)
    }
}

impl From<isize> for JsValue {
    #[inline]
    fn from(n: isize) -> Self {
        Self::from(n as i32)
    }
}

externs! {
    #[link(wasm_import_module = "__wbindgen_placeholder__")]
    extern "C" {
        fn __wbindgen_object_clone_ref(idx: u32) -> u32;
        fn __wbindgen_object_drop_ref(idx: u32) -> ();

        fn __wbindgen_string_new(ptr: *const u8, len: usize) -> u32;
        fn __wbindgen_number_new(f: f64) -> u32;
        fn __wbindgen_bigint_from_str(ptr: *const u8, len: usize) -> u32;
        fn __wbindgen_bigint_from_i64(n: i64) -> u32;
        fn __wbindgen_bigint_from_u64(n: u64) -> u32;
        fn __wbindgen_bigint_from_i128(hi: i64, lo: u64) -> u32;
        fn __wbindgen_bigint_from_u128(hi: u64, lo: u64) -> u32;
        fn __wbindgen_symbol_named_new(ptr: *const u8, len: usize) -> u32;
        fn __wbindgen_symbol_anonymous_new() -> u32;

        fn __wbindgen_externref_heap_live_count() -> u32;

        fn __wbindgen_is_null(idx: u32) -> u32;
        fn __wbindgen_is_undefined(idx: u32) -> u32;
        fn __wbindgen_is_symbol(idx: u32) -> u32;
        fn __wbindgen_is_object(idx: u32) -> u32;
        fn __wbindgen_is_array(idx: u32) -> u32;
        fn __wbindgen_is_function(idx: u32) -> u32;
        fn __wbindgen_is_string(idx: u32) -> u32;
        fn __wbindgen_is_bigint(idx: u32) -> u32;
        fn __wbindgen_typeof(idx: u32) -> u32;

        fn __wbindgen_in(prop: u32, obj: u32) -> u32;

        fn __wbindgen_is_falsy(idx: u32) -> u32;
        fn __wbindgen_as_number(idx: u32) -> f64;
        fn __wbindgen_try_into_number(idx: u32) -> u32;
        fn __wbindgen_neg(idx: u32) -> u32;
        fn __wbindgen_bit_and(a: u32, b: u32) -> u32;
        fn __wbindgen_bit_or(a: u32, b: u32) -> u32;
        fn __wbindgen_bit_xor(a: u32, b: u32) -> u32;
        fn __wbindgen_bit_not(idx: u32) -> u32;
        fn __wbindgen_shl(a: u32, b: u32) -> u32;
        fn __wbindgen_shr(a: u32, b: u32) -> u32;
        fn __wbindgen_unsigned_shr(a: u32, b: u32) -> u32;
        fn __wbindgen_add(a: u32, b: u32) -> u32;
        fn __wbindgen_sub(a: u32, b: u32) -> u32;
        fn __wbindgen_div(a: u32, b: u32) -> u32;
        fn __wbindgen_checked_div(a: u32, b: u32) -> u32;
        fn __wbindgen_mul(a: u32, b: u32) -> u32;
        fn __wbindgen_rem(a: u32, b: u32) -> u32;
        fn __wbindgen_pow(a: u32, b: u32) -> u32;
        fn __wbindgen_lt(a: u32, b: u32) -> u32;
        fn __wbindgen_le(a: u32, b: u32) -> u32;
        fn __wbindgen_ge(a: u32, b: u32) -> u32;
        fn __wbindgen_gt(a: u32, b: u32) -> u32;

        fn __wbindgen_number_get(idx: u32) -> WasmOption<f64>;
        fn __wbindgen_boolean_get(idx: u32) -> u32;
        fn __wbindgen_string_get(idx: u32) -> WasmSlice;
        fn __wbindgen_bigint_get_as_i64(idx: u32) -> WasmOption<i64>;

        fn __wbindgen_debug_string(ret: *mut [usize; 2], idx: u32) -> ();

        fn __wbindgen_throw(a: *const u8, b: usize) -> !;
        fn __wbindgen_rethrow(a: u32) -> !;
        fn __wbindgen_error_new(a: *const u8, b: usize) -> u32;

        fn __wbindgen_cb_drop(idx: u32) -> u32;

        fn __wbindgen_describe(v: u32) -> ();
        fn __wbindgen_describe_closure(a: u32, b: u32, c: u32) -> u32;

        fn __wbindgen_json_parse(ptr: *const u8, len: usize) -> u32;
        fn __wbindgen_json_serialize(idx: u32) -> WasmSlice;
        fn __wbindgen_jsval_eq(a: u32, b: u32) -> u32;
        fn __wbindgen_jsval_loose_eq(a: u32, b: u32) -> u32;

        fn __wbindgen_copy_to_typed_array(ptr: *const u8, len: usize, idx: u32) -> ();

        fn __wbindgen_not(idx: u32) -> u32;

        fn __wbindgen_exports() -> u32;
        fn __wbindgen_memory() -> u32;
        fn __wbindgen_module() -> u32;
        fn __wbindgen_function_table() -> u32;
    }
}

impl Clone for JsValue {
    #[inline]
    fn clone(&self) -> JsValue {
        unsafe {
            let idx = __wbindgen_object_clone_ref(self.idx);
            JsValue::_new(idx)
        }
    }
}

#[cfg(feature = "std")]
impl fmt::Debug for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JsValue({})", self.as_debug_string())
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Debug for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("JsValue")
    }
}

impl Drop for JsValue {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            // We definitely should never drop anything in the stack area
            debug_assert!(self.idx >= JSIDX_OFFSET, "free of stack slot {}", self.idx);

            // Otherwise if we're not dropping one of our reserved values,
            // actually call the intrinsic. See #1054 for eventually removing
            // this branch.
            if self.idx >= JSIDX_RESERVED {
                __wbindgen_object_drop_ref(self.idx);
            }
        }
    }
}

impl Default for JsValue {
    fn default() -> Self {
        Self::UNDEFINED
    }
}

/// Wrapper type for imported statics.
///
/// This type is used whenever a `static` is imported from a JS module, for
/// example this import:
///
/// ```ignore
/// #[wasm_bindgen]
/// extern "C" {
///     static console: JsValue;
/// }
/// ```
///
/// will generate in Rust a value that looks like:
///
/// ```ignore
/// static console: JsStatic<JsValue> = ...;
/// ```
///
/// This type implements `Deref` to the inner type so it's typically used as if
/// it were `&T`.
#[cfg(feature = "std")]
pub struct JsStatic<T: 'static> {
    #[doc(hidden)]
    pub __inner: &'static std::thread::LocalKey<T>,
}

#[cfg(feature = "std")]
impl<T: FromWasmAbi + 'static> Deref for JsStatic<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // We know that our tls key is never overwritten after initialization,
        // so it should be safe (on that axis at least) to hand out a reference
        // that lives longer than the closure below.
        //
        // FIXME: this is not sound if we ever implement thread exit hooks on
        // wasm, as the pointer will eventually be invalidated but you can get
        // `&'static T` from this interface. We... probably need to deprecate
        // and/or remove this interface nowadays.
        unsafe { self.__inner.with(|ptr| &*(ptr as *const T)) }
    }
}

#[cold]
#[inline(never)]
#[deprecated(note = "renamed to `throw_str`")]
#[doc(hidden)]
pub fn throw(s: &str) -> ! {
    throw_str(s)
}

/// Throws a JS exception.
///
/// This function will throw a JS exception with the message provided. The
/// function will not return as the wasm stack will be popped when the exception
/// is thrown.
///
/// Note that it is very easy to leak memory with this function because this
/// function, unlike `panic!` on other platforms, **will not run destructors**.
/// It's recommended to return a `Result` where possible to avoid the worry of
/// leaks.
#[cold]
#[inline(never)]
pub fn throw_str(s: &str) -> ! {
    unsafe {
        __wbindgen_throw(s.as_ptr(), s.len());
    }
}

/// Rethrow a JS exception
///
/// This function will throw a JS exception with the JS value provided. This
/// function will not return and the wasm stack will be popped until the point
/// of entry of wasm itself.
///
/// Note that it is very easy to leak memory with this function because this
/// function, unlike `panic!` on other platforms, **will not run destructors**.
/// It's recommended to return a `Result` where possible to avoid the worry of
/// leaks.
#[cold]
#[inline(never)]
pub fn throw_val(s: JsValue) -> ! {
    unsafe {
        let idx = s.idx;
        mem::forget(s);
        __wbindgen_rethrow(idx);
    }
}

/// Get the count of live `externref`s / `JsValue`s in `wasm-bindgen`'s heap.
///
/// ## Usage
///
/// This is intended for debugging and writing tests.
///
/// To write a test that asserts against unnecessarily keeping `anref`s /
/// `JsValue`s alive:
///
/// * get an initial live count,
///
/// * perform some series of operations or function calls that should clean up
///   after themselves, and should not keep holding onto `externref`s / `JsValue`s
///   after completion,
///
/// * get the final live count,
///
/// * and assert that the initial and final counts are the same.
///
/// ## What is Counted
///
/// Note that this only counts the *owned* `externref`s / `JsValue`s that end up in
/// `wasm-bindgen`'s heap. It does not count borrowed `externref`s / `JsValue`s
/// that are on its stack.
///
/// For example, these `JsValue`s are accounted for:
///
/// ```ignore
/// #[wasm_bindgen]
/// pub fn my_function(this_is_counted: JsValue) {
///     let also_counted = JsValue::from_str("hi");
///     assert!(wasm_bindgen::externref_heap_live_count() >= 2);
/// }
/// ```
///
/// While this borrowed `JsValue` ends up on the stack, not the heap, and
/// therefore is not accounted for:
///
/// ```ignore
/// #[wasm_bindgen]
/// pub fn my_other_function(this_is_not_counted: &JsValue) {
///     // ...
/// }
/// ```
pub fn externref_heap_live_count() -> u32 {
    unsafe { __wbindgen_externref_heap_live_count() }
}

#[doc(hidden)]
pub fn anyref_heap_live_count() -> u32 {
    externref_heap_live_count()
}

/// An extension trait for `Option<T>` and `Result<T, E>` for unwrapping the `T`
/// value, or throwing a JS error if it is not available.
///
/// These methods should have a smaller code size footprint than the normal
/// `Option::unwrap` and `Option::expect` methods, but they are specific to
/// working with wasm and JS.
///
/// On non-wasm32 targets, defaults to the normal unwrap/expect calls.
///
/// # Example
///
/// ```
/// use wasm_bindgen::prelude::*;
///
/// // If the value is `Option::Some` or `Result::Ok`, then we just get the
/// // contained `T` value.
/// let x = Some(42);
/// assert_eq!(x.unwrap_throw(), 42);
///
/// let y: Option<i32> = None;
///
/// // This call would throw an error to JS!
/// //
/// //     y.unwrap_throw()
/// //
/// // And this call would throw an error to JS with a custom error message!
/// //
/// //     y.expect_throw("woopsie daisy!")
/// ```
pub trait UnwrapThrowExt<T>: Sized {
    /// Unwrap this `Option` or `Result`, but instead of panicking on failure,
    /// throw an exception to JavaScript.
    #[cfg_attr(debug_assertions, track_caller)]
    fn unwrap_throw(self) -> T {
        if cfg!(all(debug_assertions, feature = "std")) {
            let loc = core::panic::Location::caller();
            let msg = std::format!(
                "`unwrap_throw` failed ({}:{}:{})",
                loc.file(),
                loc.line(),
                loc.column()
            );
            self.expect_throw(&msg)
        } else {
            self.expect_throw("`unwrap_throw` failed")
        }
    }

    /// Unwrap this container's `T` value, or throw an error to JS with the
    /// given message if the `T` value is unavailable (e.g. an `Option<T>` is
    /// `None`).
    #[cfg_attr(debug_assertions, track_caller)]
    fn expect_throw(self, message: &str) -> T;
}

impl<T> UnwrapThrowExt<T> for Option<T> {
    #[cfg_attr(debug_assertions, track_caller)]
    fn expect_throw(self, message: &str) -> T {
        if cfg!(all(target_arch = "wasm32", not(target_os = "emscripten"))) {
            match self {
                Some(val) => val,
                None => throw_str(message),
            }
        } else {
            self.expect(message)
        }
    }
}

impl<T, E> UnwrapThrowExt<T> for Result<T, E>
where
    E: core::fmt::Debug,
{
    #[cfg_attr(debug_assertions, track_caller)]
    fn expect_throw(self, message: &str) -> T {
        if cfg!(all(target_arch = "wasm32", not(target_os = "emscripten"))) {
            match self {
                Ok(val) => val,
                Err(_) => throw_str(message),
            }
        } else {
            self.expect(message)
        }
    }
}

/// Returns a handle to this wasm instance's `WebAssembly.Module`
///
/// Note that this is only available when the final wasm app is built with
/// `--target no-modules`, it's not recommended to rely on this API yet! This is
/// largely just an experimental addition to enable threading demos. Using this
/// may prevent your wasm module from building down the road.
#[doc(hidden)]
pub fn module() -> JsValue {
    unsafe { JsValue::_new(__wbindgen_module()) }
}

/// Returns a handle to this wasm instance's `WebAssembly.Instance.prototype.exports`
pub fn exports() -> JsValue {
    unsafe { JsValue::_new(__wbindgen_exports()) }
}

/// Returns a handle to this wasm instance's `WebAssembly.Memory`
pub fn memory() -> JsValue {
    unsafe { JsValue::_new(__wbindgen_memory()) }
}

/// Returns a handle to this wasm instance's `WebAssembly.Table` which is the
/// indirect function table used by Rust
pub fn function_table() -> JsValue {
    unsafe { JsValue::_new(__wbindgen_function_table()) }
}

#[doc(hidden)]
pub mod __rt {
    use crate::JsValue;
    use core::borrow::{Borrow, BorrowMut};
    use core::cell::{Cell, UnsafeCell};
    use core::convert::Infallible;
    use core::ops::{Deref, DerefMut};

    pub extern crate core;
    #[cfg(feature = "std")]
    pub extern crate std;

    #[macro_export]
    #[doc(hidden)]
    #[cfg(feature = "std")]
    macro_rules! __wbindgen_if_not_std {
        ($($i:item)*) => {};
    }

    #[macro_export]
    #[doc(hidden)]
    #[cfg(not(feature = "std"))]
    macro_rules! __wbindgen_if_not_std {
        ($($i:item)*) => ($($i)*)
    }

    #[inline]
    pub fn assert_not_null<T>(s: *mut T) {
        if s.is_null() {
            throw_null();
        }
    }

    #[cold]
    #[inline(never)]
    fn throw_null() -> ! {
        super::throw_str("null pointer passed to rust");
    }

    /// A vendored version of `RefCell` from the standard library.
    ///
    /// Now why, you may ask, would we do that? Surely `RefCell` in libstd is
    /// quite good. And you're right, it is indeed quite good! Functionally
    /// nothing more is needed from `RefCell` in the standard library but for
    /// now this crate is also sort of optimizing for compiled code size.
    ///
    /// One major factor to larger binaries in Rust is when a panic happens.
    /// Panicking in the standard library involves a fair bit of machinery
    /// (formatting, panic hooks, synchronization, etc). It's all worthwhile if
    /// you need it but for something like `WasmRefCell` here we don't actually
    /// need all that!
    ///
    /// This is just a wrapper around all Rust objects passed to JS intended to
    /// guard accidental reentrancy, so this vendored version is intended solely
    /// to not panic in libstd. Instead when it "panics" it calls our `throw`
    /// function in this crate which raises an error in JS.
    pub struct WasmRefCell<T: ?Sized> {
        borrow: Cell<usize>,
        value: UnsafeCell<T>,
    }

    impl<T: ?Sized> WasmRefCell<T> {
        pub fn new(value: T) -> WasmRefCell<T>
        where
            T: Sized,
        {
            WasmRefCell {
                value: UnsafeCell::new(value),
                borrow: Cell::new(0),
            }
        }

        pub fn get_mut(&mut self) -> &mut T {
            unsafe { &mut *self.value.get() }
        }

        pub fn borrow(&self) -> Ref<T> {
            unsafe {
                if self.borrow.get() == usize::max_value() {
                    borrow_fail();
                }
                self.borrow.set(self.borrow.get() + 1);
                Ref {
                    value: &*self.value.get(),
                    borrow: &self.borrow,
                }
            }
        }

        pub fn borrow_mut(&self) -> RefMut<T> {
            unsafe {
                if self.borrow.get() != 0 {
                    borrow_fail();
                }
                self.borrow.set(usize::max_value());
                RefMut {
                    value: &mut *self.value.get(),
                    borrow: &self.borrow,
                }
            }
        }

        pub fn into_inner(self) -> T
        where
            T: Sized,
        {
            self.value.into_inner()
        }
    }

    pub struct Ref<'b, T: ?Sized + 'b> {
        value: &'b T,
        borrow: &'b Cell<usize>,
    }

    impl<'b, T: ?Sized> Deref for Ref<'b, T> {
        type Target = T;

        #[inline]
        fn deref(&self) -> &T {
            self.value
        }
    }

    impl<'b, T: ?Sized> Borrow<T> for Ref<'b, T> {
        #[inline]
        fn borrow(&self) -> &T {
            self.value
        }
    }

    impl<'b, T: ?Sized> Drop for Ref<'b, T> {
        fn drop(&mut self) {
            self.borrow.set(self.borrow.get() - 1);
        }
    }

    pub struct RefMut<'b, T: ?Sized + 'b> {
        value: &'b mut T,
        borrow: &'b Cell<usize>,
    }

    impl<'b, T: ?Sized> Deref for RefMut<'b, T> {
        type Target = T;

        #[inline]
        fn deref(&self) -> &T {
            self.value
        }
    }

    impl<'b, T: ?Sized> DerefMut for RefMut<'b, T> {
        #[inline]
        fn deref_mut(&mut self) -> &mut T {
            self.value
        }
    }

    impl<'b, T: ?Sized> Borrow<T> for RefMut<'b, T> {
        #[inline]
        fn borrow(&self) -> &T {
            self.value
        }
    }

    impl<'b, T: ?Sized> BorrowMut<T> for RefMut<'b, T> {
        #[inline]
        fn borrow_mut(&mut self) -> &mut T {
            self.value
        }
    }

    impl<'b, T: ?Sized> Drop for RefMut<'b, T> {
        fn drop(&mut self) {
            self.borrow.set(0);
        }
    }

    fn borrow_fail() -> ! {
        super::throw_str(
            "recursive use of an object detected which would lead to \
             unsafe aliasing in rust",
        );
    }

    if_std! {
        use std::alloc::{alloc, dealloc, realloc, Layout};

        #[no_mangle]
        pub extern "C" fn __wbindgen_malloc(size: usize, align: usize) -> *mut u8 {
            if let Ok(layout) = Layout::from_size_align(size, align) {
                unsafe {
                    if layout.size() > 0 {
                        let ptr = alloc(layout);
                        if !ptr.is_null() {
                            return ptr
                        }
                    } else {
                        return align as *mut u8
                    }
                }
            }

            malloc_failure();
        }

        #[no_mangle]
        pub unsafe extern "C" fn __wbindgen_realloc(ptr: *mut u8, old_size: usize, new_size: usize, align: usize) -> *mut u8 {
            debug_assert!(old_size > 0);
            debug_assert!(new_size > 0);
            if let Ok(layout) = Layout::from_size_align(old_size, align) {
                let ptr = realloc(ptr, layout, new_size);
                if !ptr.is_null() {
                    return ptr
                }
            }
            malloc_failure();
        }

        #[cold]
        fn malloc_failure() -> ! {
            if cfg!(debug_assertions) {
                super::throw_str("invalid malloc request")
            } else {
                std::process::abort();
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn __wbindgen_free(ptr: *mut u8, size: usize, align: usize) {
            // This happens for zero-length slices, and in that case `ptr` is
            // likely bogus so don't actually send this to the system allocator
            if size == 0 {
                return
            }
            let layout = Layout::from_size_align_unchecked(size, align);
            dealloc(ptr, layout);
        }
    }

    /// This is a curious function necessary to get wasm-bindgen working today,
    /// and it's a bit of an unfortunate hack.
    ///
    /// The general problem is that somehow we need the above two symbols to
    /// exist in the final output binary (__wbindgen_malloc and
    /// __wbindgen_free). These symbols may be called by JS for various
    /// bindings, so we for sure need to make sure they're exported.
    ///
    /// The problem arises, though, when what if no Rust code uses the symbols?
    /// For all intents and purposes it looks to LLVM and the linker like the
    /// above two symbols are dead code, so they're completely discarded!
    ///
    /// Specifically what happens is this:
    ///
    /// * The above two symbols are generated into some object file inside of
    ///   libwasm_bindgen.rlib
    /// * The linker, LLD, will not load this object file unless *some* symbol
    ///   is loaded from the object. In this case, if the Rust code never calls
    ///   __wbindgen_malloc or __wbindgen_free then the symbols never get linked
    ///   in.
    /// * Later when `wasm-bindgen` attempts to use the symbols they don't
    ///   exist, causing an error.
    ///
    /// This function is a weird hack for this problem. We inject a call to this
    /// function in all generated code. Usage of this function should then
    /// ensure that the above two intrinsics are translated.
    ///
    /// Due to how rustc creates object files this function (and anything inside
    /// it) will be placed into the same object file as the two intrinsics
    /// above. That means if this function is called and referenced we'll pull
    /// in the object file and link the intrinsics.
    ///
    /// Ideas for how to improve this are most welcome!
    pub fn link_mem_intrinsics() {
        crate::externref::link_intrinsics();
    }

    static mut GLOBAL_EXNDATA: [u32; 2] = [0; 2];

    #[no_mangle]
    pub unsafe extern "C" fn __wbindgen_exn_store(idx: u32) {
        debug_assert_eq!(GLOBAL_EXNDATA[0], 0);
        GLOBAL_EXNDATA[0] = 1;
        GLOBAL_EXNDATA[1] = idx;
    }

    pub fn take_last_exception() -> Result<(), super::JsValue> {
        unsafe {
            let ret = if GLOBAL_EXNDATA[0] == 1 {
                Err(super::JsValue::_new(GLOBAL_EXNDATA[1]))
            } else {
                Ok(())
            };
            GLOBAL_EXNDATA[0] = 0;
            GLOBAL_EXNDATA[1] = 0;
            ret
        }
    }

    /// An internal helper trait for usage in `#[wasm_bindgen]` on `async`
    /// functions to convert the return value of the function to
    /// `Result<JsValue, JsValue>` which is what we'll return to JS (where an
    /// error is a failed future).
    pub trait IntoJsResult {
        fn into_js_result(self) -> Result<JsValue, JsValue>;
    }

    impl IntoJsResult for () {
        fn into_js_result(self) -> Result<JsValue, JsValue> {
            Ok(JsValue::undefined())
        }
    }

    impl<T: Into<JsValue>> IntoJsResult for T {
        fn into_js_result(self) -> Result<JsValue, JsValue> {
            Ok(self.into())
        }
    }

    impl<T: Into<JsValue>, E: Into<JsValue>> IntoJsResult for Result<T, E> {
        fn into_js_result(self) -> Result<JsValue, JsValue> {
            match self {
                Ok(e) => Ok(e.into()),
                Err(e) => Err(e.into()),
            }
        }
    }

    impl<E: Into<JsValue>> IntoJsResult for Result<(), E> {
        fn into_js_result(self) -> Result<JsValue, JsValue> {
            match self {
                Ok(()) => Ok(JsValue::undefined()),
                Err(e) => Err(e.into()),
            }
        }
    }

    /// An internal helper trait for usage in `#[wasm_bindgen(start)]`
    /// functions to throw the error (if it is `Err`).
    pub trait Start {
        fn start(self);
    }

    impl Start for () {
        #[inline]
        fn start(self) {}
    }

    impl<E: Into<JsValue>> Start for Result<(), E> {
        #[inline]
        fn start(self) {
            if let Err(e) = self {
                crate::throw_val(e.into());
            }
        }
    }

    /// An internal helper struct for usage in `#[wasm_bindgen(main)]`
    /// functions to throw the error (if it is `Err`).
    pub struct MainWrapper<T>(pub Option<T>);

    pub trait Main {
        fn __wasm_bindgen_main(&mut self);
    }

    impl Main for &mut &mut MainWrapper<()> {
        #[inline]
        fn __wasm_bindgen_main(&mut self) {}
    }

    impl Main for &mut &mut MainWrapper<Infallible> {
        #[inline]
        fn __wasm_bindgen_main(&mut self) {}
    }

    impl<E: Into<JsValue>> Main for &mut &mut MainWrapper<Result<(), E>> {
        #[inline]
        fn __wasm_bindgen_main(&mut self) {
            if let Err(e) = self.0.take().unwrap() {
                crate::throw_val(e.into());
            }
        }
    }

    impl<E: std::fmt::Debug> Main for &mut MainWrapper<Result<(), E>> {
        #[inline]
        fn __wasm_bindgen_main(&mut self) {
            if let Err(e) = self.0.take().unwrap() {
                crate::throw_str(&std::format!("{:?}", e));
            }
        }
    }
}

/// A wrapper type around slices and vectors for binding the `Uint8ClampedArray`
/// array in JS.
///
/// If you need to invoke a JS API which must take `Uint8ClampedArray` array,
/// then you can define it as taking one of these types:
///
/// * `Clamped<&[u8]>`
/// * `Clamped<&mut [u8]>`
/// * `Clamped<Vec<u8>>`
///
/// All of these types will show up as `Uint8ClampedArray` in JS and will have
/// different forms of ownership in Rust.
#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub struct Clamped<T>(pub T);

impl<T> Deref for Clamped<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Clamped<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// Convenience type for use on exported `fn() -> Result<T, JsError>` functions, where you wish to
/// throw a JavaScript `Error` object.
///
/// You can get wasm_bindgen to throw basic errors by simply returning
/// `Err(JsError::new("message"))` from such a function.
///
/// For more complex error handling, `JsError` implements `From<T> where T: std::error::Error` by
/// converting it to a string, so you can use it with `?`. Many Rust error types already do this,
/// and you can use [`thiserror`](https://crates.io/crates/thiserror) to derive Display
/// implementations easily or use any number of boxed error types that implement it already.
///
///
/// To allow JavaScript code to catch only your errors, you may wish to add a subclass of `Error`
/// in a JS module, and then implement `Into<JsValue>` directly on a type and instantiate that
/// subclass. In that case, you would not need `JsError` at all.
///
/// ### Basic example
///
/// ```rust,no_run
/// use wasm_bindgen::prelude::*;
///
/// #[wasm_bindgen]
/// pub fn throwing_function() -> Result<(), JsError> {
///     Err(JsError::new("message"))
/// }
/// ```
///
/// ### Complex Example
///
/// ```rust,no_run
/// use wasm_bindgen::prelude::*;
///
/// #[derive(Debug, Clone)]
/// enum MyErrorType {
///     SomeError,
/// }
///
/// use core::fmt;
/// impl std::error::Error for MyErrorType {}
/// impl fmt::Display for MyErrorType {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "display implementation becomes the error message")
///     }
/// }
///
/// fn internal_api() -> Result<(), MyErrorType> {
///     Err(MyErrorType::SomeError)
/// }
///
/// #[wasm_bindgen]
/// pub fn throwing_function() -> Result<(), JsError> {
///     internal_api()?;
///     Ok(())
/// }
///
/// ```
#[derive(Clone)]
pub struct JsError {
    value: JsValue,
}

impl JsError {
    /// Construct a JavaScript `Error` object with a string message
    #[inline]
    pub fn new(s: &str) -> JsError {
        Self {
            value: unsafe { JsValue::_new(crate::__wbindgen_error_new(s.as_ptr(), s.len())) },
        }
    }
}

if_std! {
    impl<E> From<E> for JsError
    where
        E: std::error::Error,
    {
        fn from(error: E) -> Self {
            JsError::new(&error.to_string())
        }
    }
}

impl From<JsError> for JsValue {
    fn from(error: JsError) -> Self {
        error.value
    }
}
