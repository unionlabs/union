//! Bindings to JavaScript's standard, built-in objects, including their methods
//! and properties.
//!
//! This does *not* include any Web, Node, or any other JS environment
//! APIs. Only the things that are guaranteed to exist in the global scope by
//! the ECMAScript standard.
//!
//! <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects>
//!
//! ## A Note About `camelCase`, `snake_case`, and Naming Conventions
//!
//! JavaScript's global objects use `camelCase` naming conventions for functions
//! and methods, but Rust style is to use `snake_case`. These bindings expose
//! the Rust style `snake_case` name. Additionally, acronyms within a method
//! name are all lower case, where as in JavaScript they are all upper case. For
//! example, `decodeURI` in JavaScript is exposed as `decode_uri` in these
//! bindings.

#![doc(html_root_url = "https://docs.rs/js-sys/0.2")]

use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};
use std::cmp::Ordering;
use std::convert::{self, Infallible, TryFrom};
use std::f64;
use std::fmt;
use std::iter::{self, Product, Sum};
use std::mem;
use std::str;
use std::str::FromStr;

use wasm_bindgen::prelude::*;

// When adding new imports:
//
// * Keep imports in alphabetical order.
//
// * Rename imports with `js_name = ...` according to the note about `camelCase`
//   and `snake_case` in the module's documentation above.
//
// * Include the one sentence summary of the import from the MDN link in the
//   module's documentation above, and the MDN link itself.
//
// * If a function or method can throw an exception, make it catchable by adding
//   `#[wasm_bindgen(catch)]`.
//
// * Add a new `#[test]` into the appropriate file in the
//   `crates/js-sys/tests/wasm/` directory. If the imported function or method
//   can throw an exception, make sure to also add test coverage for that case.
//
// * Arguments that are `JsValue`s or imported JavaScript types should be taken
//   by reference.

macro_rules! forward_deref_unop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        impl $imp for $t {
            type Output = <&'static $t as $imp>::Output;

            #[inline]
            fn $method(self) -> Self::Output {
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
            fn $method(self, other: $t) -> Self::Output {
                $imp::$method(self, &other)
            }
        }

        impl $imp<&$t> for $t {
            type Output = <&'static $t as $imp<&'static $t>>::Output;

            #[inline]
            fn $method(self, other: &$t) -> Self::Output {
                $imp::$method(&self, other)
            }
        }

        impl $imp<$t> for $t {
            type Output = <&'static $t as $imp<&'static $t>>::Output;

            #[inline]
            fn $method(self, other: $t) -> Self::Output {
                $imp::$method(&self, &other)
            }
        }
    };
}

macro_rules! forward_js_unop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        impl $imp for &$t {
            type Output = $t;

            #[inline]
            fn $method(self) -> Self::Output {
                $imp::$method(JsValue::as_ref(self)).unchecked_into()
            }
        }

        forward_deref_unop!(impl $imp, $method for $t);
    };
}

macro_rules! forward_js_binop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        impl $imp<&$t> for &$t {
            type Output = $t;

            #[inline]
            fn $method(self, other: &$t) -> Self::Output {
                $imp::$method(JsValue::as_ref(self), JsValue::as_ref(other)).unchecked_into()
            }
        }

        forward_deref_binop!(impl $imp, $method for $t);
    };
}

macro_rules! sum_product {
    ($($a:ident)*) => ($(
        impl Sum for $a {
            #[inline]
            fn sum<I: iter::Iterator<Item=Self>>(iter: I) -> Self {
                iter.fold(
                    $a::from(0),
                    |a, b| a + b,
                )
            }
        }

        impl Product for $a {
            #[inline]
            fn product<I: iter::Iterator<Item=Self>>(iter: I) -> Self {
                iter.fold(
                    $a::from(1),
                    |a, b| a * b,
                )
            }
        }

        impl<'a> Sum<&'a $a> for $a {
            fn sum<I: iter::Iterator<Item=&'a Self>>(iter: I) -> Self {
                iter.fold(
                    $a::from(0),
                    |a, b| a + b,
                )
            }
        }

        impl<'a> Product<&'a $a> for $a {
            #[inline]
            fn product<I: iter::Iterator<Item=&'a Self>>(iter: I) -> Self {
                iter.fold(
                    $a::from(1),
                    |a, b| a * b,
                )
            }
        }
    )*)
}

macro_rules! partialord_ord {
    ($t:ident) => {
        impl PartialOrd for $t {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }

            #[inline]
            fn lt(&self, other: &Self) -> bool {
                JsValue::as_ref(self).lt(JsValue::as_ref(other))
            }

            #[inline]
            fn le(&self, other: &Self) -> bool {
                JsValue::as_ref(self).le(JsValue::as_ref(other))
            }

            #[inline]
            fn ge(&self, other: &Self) -> bool {
                JsValue::as_ref(self).ge(JsValue::as_ref(other))
            }

            #[inline]
            fn gt(&self, other: &Self) -> bool {
                JsValue::as_ref(self).gt(JsValue::as_ref(other))
            }
        }

        impl Ord for $t {
            #[inline]
            fn cmp(&self, other: &Self) -> Ordering {
                if self == other {
                    Ordering::Equal
                } else if self.lt(other) {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
        }
    };
}

#[wasm_bindgen]
extern "C" {
    /// The `decodeURI()` function decodes a Uniform Resource Identifier (URI)
    /// previously created by `encodeURI` or by a similar routine.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/decodeURI)
    #[wasm_bindgen(catch, js_name = decodeURI)]
    pub fn decode_uri(encoded: &str) -> Result<JsString, JsValue>;

    /// The `decodeURIComponent()` function decodes a Uniform Resource Identifier (URI) component
    /// previously created by `encodeURIComponent` or by a similar routine.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/decodeURIComponent)
    #[wasm_bindgen(catch, js_name = decodeURIComponent)]
    pub fn decode_uri_component(encoded: &str) -> Result<JsString, JsValue>;

    /// The `encodeURI()` function encodes a Uniform Resource Identifier (URI)
    /// by replacing each instance of certain characters by one, two, three, or
    /// four escape sequences representing the UTF-8 encoding of the character
    /// (will only be four escape sequences for characters composed of two
    /// "surrogate" characters).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURI)
    #[wasm_bindgen(js_name = encodeURI)]
    pub fn encode_uri(decoded: &str) -> JsString;

    /// The `encodeURIComponent()` function encodes a Uniform Resource Identifier (URI) component
    /// by replacing each instance of certain characters by one, two, three, or four escape sequences
    /// representing the UTF-8 encoding of the character
    /// (will only be four escape sequences for characters composed of two "surrogate" characters).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent)
    #[wasm_bindgen(js_name = encodeURIComponent)]
    pub fn encode_uri_component(decoded: &str) -> JsString;

    /// The `eval()` function evaluates JavaScript code represented as a string.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval)
    #[wasm_bindgen(catch)]
    pub fn eval(js_source_text: &str) -> Result<JsValue, JsValue>;

    /// The global `isFinite()` function determines whether the passed value is a finite number.
    /// If needed, the parameter is first converted to a number.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/isFinite)
    #[wasm_bindgen(js_name = isFinite)]
    pub fn is_finite(value: &JsValue) -> bool;

    /// The `parseInt()` function parses a string argument and returns an integer
    /// of the specified radix (the base in mathematical numeral systems), or NaN on error.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseInt)
    #[wasm_bindgen(js_name = parseInt)]
    pub fn parse_int(text: &str, radix: u8) -> f64;

    /// The `parseFloat()` function parses an argument and returns a floating point number,
    /// or NaN on error.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseFloat)
    #[wasm_bindgen(js_name = parseFloat)]
    pub fn parse_float(text: &str) -> f64;

    /// The `escape()` function computes a new string in which certain characters have been
    /// replaced by a hexadecimal escape sequence.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/escape)
    #[wasm_bindgen]
    pub fn escape(string: &str) -> JsString;

    /// The `unescape()` function computes a new string in which hexadecimal escape
    /// sequences are replaced with the character that it represents. The escape sequences might
    /// be introduced by a function like `escape`. Usually, `decodeURI` or `decodeURIComponent`
    /// are preferred over `unescape`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/unescape)
    #[wasm_bindgen]
    pub fn unescape(string: &str) -> JsString;
}

// Array
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, is_type_of = Array::is_array, typescript_type = "Array<any>")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Array;

    /// Creates a new empty array.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Array;

    /// Creates a new array with the specified length (elements are initialized to `undefined`).
    #[wasm_bindgen(constructor)]
    pub fn new_with_length(len: u32) -> Array;

    /// Retrieves the element at the index, counting from the end if negative
    /// (returns `undefined` if the index is out of range).
    #[wasm_bindgen(method)]
    pub fn at(this: &Array, index: i32) -> JsValue;

    /// Retrieves the element at the index (returns `undefined` if the index is out of range).
    #[wasm_bindgen(method, structural, indexing_getter)]
    pub fn get(this: &Array, index: u32) -> JsValue;

    /// Sets the element at the index (auto-enlarges the array if the index is out of range).
    #[wasm_bindgen(method, structural, indexing_setter)]
    pub fn set(this: &Array, index: u32, value: JsValue);

    /// Deletes the element at the index (does nothing if the index is out of range).
    ///
    /// The element at the index is set to `undefined`.
    ///
    /// This does not resize the array, the array will still be the same length.
    #[wasm_bindgen(method, structural, indexing_deleter)]
    pub fn delete(this: &Array, index: u32);

    /// The `Array.from()` method creates a new, shallow-copied `Array` instance
    /// from an array-like or iterable object.
    #[wasm_bindgen(static_method_of = Array)]
    pub fn from(val: &JsValue) -> Array;

    /// The `copyWithin()` method shallow copies part of an array to another
    /// location in the same array and returns it, without modifying its size.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/copyWithin)
    #[wasm_bindgen(method, js_name = copyWithin)]
    pub fn copy_within(this: &Array, target: i32, start: i32, end: i32) -> Array;

    /// The `concat()` method is used to merge two or more arrays. This method
    /// does not change the existing arrays, but instead returns a new array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/concat)
    #[wasm_bindgen(method)]
    pub fn concat(this: &Array, array: &Array) -> Array;

    /// The `every()` method tests whether all elements in the array pass the test
    /// implemented by the provided function.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/every)
    #[wasm_bindgen(method)]
    pub fn every(this: &Array, predicate: &mut dyn FnMut(JsValue, u32, Array) -> bool) -> bool;

    /// The `fill()` method fills all the elements of an array from a start index
    /// to an end index with a static value. The end index is not included.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/fill)
    #[wasm_bindgen(method)]
    pub fn fill(this: &Array, value: &JsValue, start: u32, end: u32) -> Array;

    /// The `filter()` method creates a new array with all elements that pass the
    /// test implemented by the provided function.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/filter)
    #[wasm_bindgen(method)]
    pub fn filter(this: &Array, predicate: &mut dyn FnMut(JsValue, u32, Array) -> bool) -> Array;

    /// The `find()` method returns the value of the first element in the array that satisfies
    ///  the provided testing function. Otherwise `undefined` is returned.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/find)
    #[wasm_bindgen(method)]
    pub fn find(this: &Array, predicate: &mut dyn FnMut(JsValue, u32, Array) -> bool) -> JsValue;

    /// The `findIndex()` method returns the index of the first element in the array that
    /// satisfies the provided testing function. Otherwise -1 is returned.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findIndex)
    #[wasm_bindgen(method, js_name = findIndex)]
    pub fn find_index(this: &Array, predicate: &mut dyn FnMut(JsValue, u32, Array) -> bool) -> i32;

    /// The `flat()` method creates a new array with all sub-array elements concatenated into it
    /// recursively up to the specified depth.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flat)
    #[wasm_bindgen(method)]
    pub fn flat(this: &Array, depth: i32) -> Array;

    /// The `flatMap()` method first maps each element using a mapping function, then flattens
    /// the result into a new array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flatMap)
    #[wasm_bindgen(method, js_name = flatMap)]
    pub fn flat_map(
        this: &Array,
        callback: &mut dyn FnMut(JsValue, u32, Array) -> Vec<JsValue>,
    ) -> Array;

    /// The `forEach()` method executes a provided function once for each array element.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/forEach)
    #[wasm_bindgen(method, js_name = forEach)]
    pub fn for_each(this: &Array, callback: &mut dyn FnMut(JsValue, u32, Array));

    /// The `includes()` method determines whether an array includes a certain
    /// element, returning true or false as appropriate.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/includes)
    #[wasm_bindgen(method)]
    pub fn includes(this: &Array, value: &JsValue, from_index: i32) -> bool;

    /// The `indexOf()` method returns the first index at which a given element
    /// can be found in the array, or -1 if it is not present.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/indexOf)
    #[wasm_bindgen(method, js_name = indexOf)]
    pub fn index_of(this: &Array, value: &JsValue, from_index: i32) -> i32;

    /// The `Array.isArray()` method determines whether the passed value is an Array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/isArray)
    #[wasm_bindgen(static_method_of = Array, js_name = isArray)]
    pub fn is_array(value: &JsValue) -> bool;

    /// The `join()` method joins all elements of an array (or an array-like object)
    /// into a string and returns this string.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/join)
    #[wasm_bindgen(method)]
    pub fn join(this: &Array, delimiter: &str) -> JsString;

    /// The `lastIndexOf()` method returns the last index at which a given element
    /// can be found in the array, or -1 if it is not present. The array is
    /// searched backwards, starting at fromIndex.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/lastIndexOf)
    #[wasm_bindgen(method, js_name = lastIndexOf)]
    pub fn last_index_of(this: &Array, value: &JsValue, from_index: i32) -> i32;

    /// The length property of an object which is an instance of type Array
    /// sets or returns the number of elements in that array. The value is an
    /// unsigned, 32-bit integer that is always numerically greater than the
    /// highest index in the array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/length)
    #[wasm_bindgen(method, getter, structural)]
    pub fn length(this: &Array) -> u32;

    /// Sets the length of the array.
    ///
    /// If it is set to less than the current length of the array, it will
    /// shrink the array.
    ///
    /// If it is set to more than the current length of the array, it will
    /// increase the length of the array, filling the new space with empty
    /// slots.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/length)
    #[wasm_bindgen(method, setter)]
    pub fn set_length(this: &Array, value: u32);

    /// `map()` calls a provided callback function once for each element in an array,
    /// in order, and constructs a new array from the results. callback is invoked
    /// only for indexes of the array which have assigned values, including undefined.
    /// It is not called for missing elements of the array (that is, indexes that have
    /// never been set, which have been deleted or which have never been assigned a value).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/map)
    #[wasm_bindgen(method)]
    pub fn map(this: &Array, predicate: &mut dyn FnMut(JsValue, u32, Array) -> JsValue) -> Array;

    /// The `Array.of()` method creates a new Array instance with a variable
    /// number of arguments, regardless of number or type of the arguments.
    ///
    /// The difference between `Array.of()` and the `Array` constructor is in the
    /// handling of integer arguments: `Array.of(7)` creates an array with a single
    /// element, `7`, whereas `Array(7)` creates an empty array with a `length`
    /// property of `7` (Note: this implies an array of 7 empty slots, not slots
    /// with actual undefined values).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/of)
    ///
    /// # Notes
    ///
    /// There are a few bindings to `of` in `js-sys`: `of1`, `of2`, etc...
    /// with different arities.
    #[wasm_bindgen(static_method_of = Array, js_name = of)]
    pub fn of1(a: &JsValue) -> Array;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/of)
    #[wasm_bindgen(static_method_of = Array, js_name = of)]
    pub fn of2(a: &JsValue, b: &JsValue) -> Array;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/of)
    #[wasm_bindgen(static_method_of = Array, js_name = of)]
    pub fn of3(a: &JsValue, b: &JsValue, c: &JsValue) -> Array;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/of)
    #[wasm_bindgen(static_method_of = Array, js_name = of)]
    pub fn of4(a: &JsValue, b: &JsValue, c: &JsValue, d: &JsValue) -> Array;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/of)
    #[wasm_bindgen(static_method_of = Array, js_name = of)]
    pub fn of5(a: &JsValue, b: &JsValue, c: &JsValue, d: &JsValue, e: &JsValue) -> Array;

    /// The `pop()` method removes the last element from an array and returns that
    /// element. This method changes the length of the array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/pop)
    #[wasm_bindgen(method)]
    pub fn pop(this: &Array) -> JsValue;

    /// The `push()` method adds one or more elements to the end of an array and
    /// returns the new length of the array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/push)
    #[wasm_bindgen(method)]
    pub fn push(this: &Array, value: &JsValue) -> u32;

    /// The `reduce()` method applies a function against an accumulator and each element in
    /// the array (from left to right) to reduce it to a single value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/Reduce)
    #[wasm_bindgen(method)]
    pub fn reduce(
        this: &Array,
        predicate: &mut dyn FnMut(JsValue, JsValue, u32, Array) -> JsValue,
        initial_value: &JsValue,
    ) -> JsValue;

    /// The `reduceRight()` method applies a function against an accumulator and each value
    /// of the array (from right-to-left) to reduce it to a single value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/ReduceRight)
    #[wasm_bindgen(method, js_name = reduceRight)]
    pub fn reduce_right(
        this: &Array,
        predicate: &mut dyn FnMut(JsValue, JsValue, u32, Array) -> JsValue,
        initial_value: &JsValue,
    ) -> JsValue;

    /// The `reverse()` method reverses an array in place. The first array
    /// element becomes the last, and the last array element becomes the first.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/reverse)
    #[wasm_bindgen(method)]
    pub fn reverse(this: &Array) -> Array;

    /// The `shift()` method removes the first element from an array and returns
    /// that removed element. This method changes the length of the array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/shift)
    #[wasm_bindgen(method)]
    pub fn shift(this: &Array) -> JsValue;

    /// The `slice()` method returns a shallow copy of a portion of an array into
    /// a new array object selected from begin to end (end not included).
    /// The original array will not be modified.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/slice)
    #[wasm_bindgen(method)]
    pub fn slice(this: &Array, start: u32, end: u32) -> Array;

    /// The `some()` method tests whether at least one element in the array passes the test implemented
    /// by the provided function.
    /// Note: This method returns false for any condition put on an empty array.
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/some)
    #[wasm_bindgen(method)]
    pub fn some(this: &Array, predicate: &mut dyn FnMut(JsValue) -> bool) -> bool;

    /// The `sort()` method sorts the elements of an array in place and returns
    /// the array. The sort is not necessarily stable. The default sort
    /// order is according to string Unicode code points.
    ///
    /// The time and space complexity of the sort cannot be guaranteed as it
    /// is implementation dependent.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/sort)
    #[wasm_bindgen(method)]
    pub fn sort(this: &Array) -> Array;

    /// The `splice()` method changes the contents of an array by removing existing elements and/or
    /// adding new elements.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/splice)
    #[wasm_bindgen(method)]
    pub fn splice(this: &Array, start: u32, delete_count: u32, item: &JsValue) -> Array;

    /// The `toLocaleString()` method returns a string representing the elements of the array.
    /// The elements are converted to Strings using their toLocaleString methods and these
    /// Strings are separated by a locale-specific String (such as a comma “,”).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/toLocaleString)
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &Array, locales: &JsValue, options: &JsValue) -> JsString;

    /// The `toString()` method returns a string representing the specified array
    /// and its elements.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/toString)
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Array) -> JsString;

    /// The `unshift()` method adds one or more elements to the beginning of an
    /// array and returns the new length of the array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/unshift)
    #[wasm_bindgen(method)]
    pub fn unshift(this: &Array, value: &JsValue) -> u32;
}

/// Iterator returned by `Array::into_iter`
#[derive(Debug, Clone)]
pub struct ArrayIntoIter {
    range: std::ops::Range<u32>,
    array: Array,
}

impl std::iter::Iterator for ArrayIntoIter {
    type Item = JsValue;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.range.next()?;
        Some(self.array.get(index))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.range.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { range, array } = self;
        range.last().map(|index| array.get(index))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth(n).map(|index| self.array.get(index))
    }
}

impl std::iter::DoubleEndedIterator for ArrayIntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.next_back()?;
        Some(self.array.get(index))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth_back(n).map(|index| self.array.get(index))
    }
}

impl std::iter::FusedIterator for ArrayIntoIter {}

impl std::iter::ExactSizeIterator for ArrayIntoIter {}

/// Iterator returned by `Array::iter`
#[derive(Debug, Clone)]
pub struct ArrayIter<'a> {
    range: std::ops::Range<u32>,
    array: &'a Array,
}

impl<'a> std::iter::Iterator for ArrayIter<'a> {
    type Item = JsValue;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.range.next()?;
        Some(self.array.get(index))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.range.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let Self { range, array } = self;
        range.last().map(|index| array.get(index))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth(n).map(|index| self.array.get(index))
    }
}

impl<'a> std::iter::DoubleEndedIterator for ArrayIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.next_back()?;
        Some(self.array.get(index))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth_back(n).map(|index| self.array.get(index))
    }
}

impl<'a> std::iter::FusedIterator for ArrayIter<'a> {}

impl<'a> std::iter::ExactSizeIterator for ArrayIter<'a> {}

impl Array {
    /// Returns an iterator over the values of the JS array.
    pub fn iter(&self) -> ArrayIter<'_> {
        ArrayIter {
            range: 0..self.length(),
            array: self,
        }
    }

    /// Converts the JS array into a new Vec.
    pub fn to_vec(&self) -> Vec<JsValue> {
        let len = self.length();

        let mut output = Vec::with_capacity(len as usize);

        for i in 0..len {
            output.push(self.get(i));
        }

        output
    }
}

impl std::iter::IntoIterator for Array {
    type Item = JsValue;
    type IntoIter = ArrayIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        ArrayIntoIter {
            range: 0..self.length(),
            array: self,
        }
    }
}

// TODO pre-initialize the Array with the correct length using TrustedLen
impl<A> std::iter::FromIterator<A> for Array
where
    A: AsRef<JsValue>,
{
    fn from_iter<T>(iter: T) -> Array
    where
        T: IntoIterator<Item = A>,
    {
        let mut out = Array::new();
        out.extend(iter);
        out
    }
}

impl<A> std::iter::Extend<A> for Array
where
    A: AsRef<JsValue>,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = A>,
    {
        for value in iter {
            self.push(value.as_ref());
        }
    }
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}

// ArrayBuffer
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "ArrayBuffer")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type ArrayBuffer;

    /// The `ArrayBuffer` object is used to represent a generic,
    /// fixed-length raw binary data buffer. You cannot directly
    /// manipulate the contents of an `ArrayBuffer`; instead, you
    /// create one of the typed array objects or a `DataView` object
    /// which represents the buffer in a specific format, and use that
    /// to read and write the contents of the buffer.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer)
    #[wasm_bindgen(constructor)]
    pub fn new(length: u32) -> ArrayBuffer;

    /// The byteLength property of an object which is an instance of type ArrayBuffer
    /// it's an accessor property whose set accessor function is undefined,
    /// meaning that you can only read this property.
    /// The value is established when the array is constructed and cannot be changed.
    /// This property returns 0 if this ArrayBuffer has been detached.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer/byteLength)
    #[wasm_bindgen(method, getter, js_name = byteLength)]
    pub fn byte_length(this: &ArrayBuffer) -> u32;

    /// The `isView()` method returns true if arg is one of the `ArrayBuffer`
    /// views, such as typed array objects or a DataView; false otherwise.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer/isView)
    #[wasm_bindgen(static_method_of = ArrayBuffer, js_name = isView)]
    pub fn is_view(value: &JsValue) -> bool;

    /// The `slice()` method returns a new `ArrayBuffer` whose contents
    /// are a copy of this `ArrayBuffer`'s bytes from begin, inclusive,
    /// up to end, exclusive.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer/slice)
    #[wasm_bindgen(method)]
    pub fn slice(this: &ArrayBuffer, begin: u32) -> ArrayBuffer;

    /// Like `slice()` but with the `end` argument.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer/slice)
    #[wasm_bindgen(method, js_name = slice)]
    pub fn slice_with_end(this: &ArrayBuffer, begin: u32, end: u32) -> ArrayBuffer;
}

// SharedArrayBuffer
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "SharedArrayBuffer")]
    #[derive(Clone, Debug)]
    pub type SharedArrayBuffer;

    /// The `SharedArrayBuffer` object is used to represent a generic,
    /// fixed-length raw binary data buffer, similar to the `ArrayBuffer`
    /// object, but in a way that they can be used to create views
    /// on shared memory. Unlike an `ArrayBuffer`, a `SharedArrayBuffer`
    /// cannot become detached.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer)
    #[wasm_bindgen(constructor)]
    pub fn new(length: u32) -> SharedArrayBuffer;

    /// The byteLength accessor property represents the length of
    /// an `SharedArrayBuffer` in bytes. This is established when
    /// the `SharedArrayBuffer` is constructed and cannot be changed.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer/byteLength)
    #[wasm_bindgen(method, getter, js_name = byteLength)]
    pub fn byte_length(this: &SharedArrayBuffer) -> u32;

    /// The `slice()` method returns a new `SharedArrayBuffer` whose contents
    /// are a copy of this `SharedArrayBuffer`'s bytes from begin, inclusive,
    /// up to end, exclusive.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer/slice)
    #[wasm_bindgen(method)]
    pub fn slice(this: &SharedArrayBuffer, begin: u32) -> SharedArrayBuffer;

    /// Like `slice()` but with the `end` argument.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer/slice)
    #[wasm_bindgen(method, js_name = slice)]
    pub fn slice_with_end(this: &SharedArrayBuffer, begin: u32, end: u32) -> SharedArrayBuffer;
}

// Array Iterator
#[wasm_bindgen]
extern "C" {
    /// The `keys()` method returns a new Array Iterator object that contains the
    /// keys for each index in the array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/keys)
    #[wasm_bindgen(method)]
    pub fn keys(this: &Array) -> Iterator;

    /// The `entries()` method returns a new Array Iterator object that contains
    /// the key/value pairs for each index in the array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/entries)
    #[wasm_bindgen(method)]
    pub fn entries(this: &Array) -> Iterator;

    /// The `values()` method returns a new Array Iterator object that
    /// contains the values for each index in the array.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/values)
    #[wasm_bindgen(method)]
    pub fn values(this: &Array) -> Iterator;
}

/// The `Atomics` object provides atomic operations as static methods.
/// They are used with `SharedArrayBuffer` objects.
///
/// The Atomic operations are installed on an `Atomics` module. Unlike
/// the other global objects, `Atomics` is not a constructor. You cannot
/// use it with a new operator or invoke the `Atomics` object as a
/// function. All properties and methods of `Atomics` are static
/// (as is the case with the Math object, for example).
/// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics)
#[allow(non_snake_case)]
pub mod Atomics {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        /// The static `Atomics.add()` method adds a given value at a given
        /// position in the array and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// You should use `add_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/add)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn add(typed_array: &JsValue, index: u32, value: i32) -> Result<i32, JsValue>;

        /// The static `Atomics.add()` method adds a given value at a given
        /// position in the array and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/add)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = add)]
        pub fn add_bigint(typed_array: &JsValue, index: u32, value: i64) -> Result<i64, JsValue>;

        /// The static `Atomics.and()` method computes a bitwise AND with a given
        /// value at a given position in the array, and returns the old value
        /// at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// You should use `and_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/and)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn and(typed_array: &JsValue, index: u32, value: i32) -> Result<i32, JsValue>;

        /// The static `Atomics.and()` method computes a bitwise AND with a given
        /// value at a given position in the array, and returns the old value
        /// at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/and)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = and)]
        pub fn and_bigint(typed_array: &JsValue, index: u32, value: i64) -> Result<i64, JsValue>;

        /// The static `Atomics.compareExchange()` method exchanges a given
        /// replacement value at a given position in the array, if a given expected
        /// value equals the old value. It returns the old value at that position
        /// whether it was equal to the expected value or not.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// You should use `compare_exchange_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/compareExchange)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = compareExchange)]
        pub fn compare_exchange(
            typed_array: &JsValue,
            index: u32,
            expected_value: i32,
            replacement_value: i32,
        ) -> Result<i32, JsValue>;

        /// The static `Atomics.compareExchange()` method exchanges a given
        /// replacement value at a given position in the array, if a given expected
        /// value equals the old value. It returns the old value at that position
        /// whether it was equal to the expected value or not.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/compareExchange)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = compareExchange)]
        pub fn compare_exchange_bigint(
            typed_array: &JsValue,
            index: u32,
            expected_value: i64,
            replacement_value: i64,
        ) -> Result<i64, JsValue>;

        /// The static `Atomics.exchange()` method stores a given value at a given
        /// position in the array and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// You should use `exchange_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/exchange)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn exchange(typed_array: &JsValue, index: u32, value: i32) -> Result<i32, JsValue>;

        /// The static `Atomics.exchange()` method stores a given value at a given
        /// position in the array and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/exchange)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = exchange)]
        pub fn exchange_bigint(
            typed_array: &JsValue,
            index: u32,
            value: i64,
        ) -> Result<i64, JsValue>;

        /// The static `Atomics.isLockFree()` method is used to determine
        /// whether to use locks or atomic operations. It returns true,
        /// if the given size is one of the `BYTES_PER_ELEMENT` property
        /// of integer `TypedArray` types.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/isLockFree)
        #[wasm_bindgen(js_namespace = Atomics, js_name = isLockFree)]
        pub fn is_lock_free(size: u32) -> bool;

        /// The static `Atomics.load()` method returns a value at a given
        /// position in the array.
        ///
        /// You should use `load_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/load)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn load(typed_array: &JsValue, index: u32) -> Result<i32, JsValue>;

        /// The static `Atomics.load()` method returns a value at a given
        /// position in the array.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/load)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = load)]
        pub fn load_bigint(typed_array: &JsValue, index: i64) -> Result<i64, JsValue>;

        /// The static `Atomics.notify()` method notifies up some agents that
        /// are sleeping in the wait queue.
        /// Note: This operation works with a shared `Int32Array` only.
        /// If `count` is not provided, notifies all the agents in the queue.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/notify)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn notify(typed_array: &Int32Array, index: u32) -> Result<u32, JsValue>;

        /// Notifies up to `count` agents in the wait queue.
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = notify)]
        pub fn notify_with_count(
            typed_array: &Int32Array,
            index: u32,
            count: u32,
        ) -> Result<u32, JsValue>;

        /// The static `Atomics.or()` method computes a bitwise OR with a given value
        /// at a given position in the array, and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// You should use `or_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/or)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn or(typed_array: &JsValue, index: u32, value: i32) -> Result<i32, JsValue>;

        /// The static `Atomics.or()` method computes a bitwise OR with a given value
        /// at a given position in the array, and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/or)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = or)]
        pub fn or_bigint(typed_array: &JsValue, index: u32, value: i64) -> Result<i64, JsValue>;

        /// The static `Atomics.store()` method stores a given value at the given
        /// position in the array and returns that value.
        ///
        /// You should use `store_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/store)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn store(typed_array: &JsValue, index: u32, value: i32) -> Result<i32, JsValue>;

        /// The static `Atomics.store()` method stores a given value at the given
        /// position in the array and returns that value.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/store)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = store)]
        pub fn store_bigint(typed_array: &JsValue, index: u32, value: i64) -> Result<i64, JsValue>;

        /// The static `Atomics.sub()` method subtracts a given value at a
        /// given position in the array and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// You should use `sub_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/sub)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn sub(typed_array: &JsValue, index: u32, value: i32) -> Result<i32, JsValue>;

        /// The static `Atomics.sub()` method subtracts a given value at a
        /// given position in the array and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/sub)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = sub)]
        pub fn sub_bigint(typed_array: &JsValue, index: u32, value: i64) -> Result<i64, JsValue>;

        /// The static `Atomics.wait()` method verifies that a given
        /// position in an `Int32Array` still contains a given value
        /// and if so sleeps, awaiting a wakeup or a timeout.
        /// It returns a string which is either "ok", "not-equal", or "timed-out".
        /// Note: This operation only works with a shared `Int32Array`
        /// and may not be allowed on the main thread.
        ///
        /// You should use `wait_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/wait)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn wait(typed_array: &Int32Array, index: u32, value: i32) -> Result<JsString, JsValue>;

        /// The static `Atomics.wait()` method verifies that a given
        /// position in an `Int32Array` still contains a given value
        /// and if so sleeps, awaiting a wakeup or a timeout.
        /// It returns a string which is either "ok", "not-equal", or "timed-out".
        /// Note: This operation only works with a shared `Int32Array`
        /// and may not be allowed on the main thread.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/wait)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = wait)]
        pub fn wait_bigint(
            typed_array: &Int32Array,
            index: u32,
            value: i64,
        ) -> Result<JsString, JsValue>;

        /// Like `wait()`, but with timeout
        ///
        /// You should use `wait_with_timeout_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/wait)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = wait)]
        pub fn wait_with_timeout(
            typed_array: &Int32Array,
            index: u32,
            value: i32,
            timeout: f64,
        ) -> Result<JsString, JsValue>;

        /// Like `wait()`, but with timeout
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/wait)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = wait)]
        pub fn wait_with_timeout_bigint(
            typed_array: &Int32Array,
            index: u32,
            value: i64,
            timeout: f64,
        ) -> Result<JsString, JsValue>;

        /// The static `Atomics.xor()` method computes a bitwise XOR
        /// with a given value at a given position in the array,
        /// and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// You should use `xor_bigint` to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/xor)
        #[wasm_bindgen(js_namespace = Atomics, catch)]
        pub fn xor(typed_array: &JsValue, index: u32, value: i32) -> Result<i32, JsValue>;

        /// The static `Atomics.xor()` method computes a bitwise XOR
        /// with a given value at a given position in the array,
        /// and returns the old value at that position.
        /// This atomic operation guarantees that no other write happens
        /// until the modified value is written back.
        ///
        /// This method is used to operate on a `BigInt64Array` or a `BigUint64Array`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/xor)
        #[wasm_bindgen(js_namespace = Atomics, catch, js_name = xor)]
        pub fn xor_bigint(typed_array: &JsValue, index: u32, value: i64) -> Result<i64, JsValue>;
    }
}

// BigInt
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, is_type_of = |v| v.is_bigint(), typescript_type = "bigint")]
    #[derive(Clone, PartialEq, Eq)]
    pub type BigInt;

    #[wasm_bindgen(catch, js_name = BigInt)]
    fn new_bigint(value: &JsValue) -> Result<BigInt, Error>;

    #[wasm_bindgen(js_name = BigInt)]
    fn new_bigint_unchecked(value: &JsValue) -> BigInt;

    /// Clamps a BigInt value to a signed integer value, and returns that value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/asIntN)
    #[wasm_bindgen(static_method_of = BigInt, js_name = asIntN)]
    pub fn as_int_n(bits: f64, bigint: &BigInt) -> BigInt;

    /// Clamps a BigInt value to an unsigned integer value, and returns that value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/asUintN)
    #[wasm_bindgen(static_method_of = BigInt, js_name = asUintN)]
    pub fn as_uint_n(bits: f64, bigint: &BigInt) -> BigInt;

    /// Returns a string with a language-sensitive representation of this BigInt value. Overrides the [`Object.prototype.toLocaleString()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/toLocaleString) method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/toLocaleString)
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &BigInt, locales: &JsValue, options: &JsValue) -> JsString;

    /// Returns a string representing this BigInt value in the specified radix (base). Overrides the [`Object.prototype.toString()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/toString) method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/toString)
    #[wasm_bindgen(catch, method, js_name = toString)]
    pub fn to_string(this: &BigInt, radix: u8) -> Result<JsString, RangeError>;

    #[wasm_bindgen(method, js_name = toString)]
    fn to_string_unchecked(this: &BigInt, radix: u8) -> String;

    /// Returns this BigInt value. Overrides the [`Object.prototype.valueOf()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/valueOf) method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/valueOf)
    #[wasm_bindgen(method, js_name = valueOf)]
    pub fn value_of(this: &BigInt, radix: u8) -> BigInt;
}

impl BigInt {
    /// Creates a new BigInt value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/BigInt)
    #[inline]
    pub fn new(value: &JsValue) -> Result<BigInt, Error> {
        new_bigint(value)
    }

    /// Applies the binary `/` JS operator on two `BigInt`s, catching and returning any `RangeError` thrown.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Division)
    pub fn checked_div(&self, rhs: &Self) -> Result<Self, RangeError> {
        let result = JsValue::as_ref(self).checked_div(JsValue::as_ref(rhs));

        if result.is_instance_of::<RangeError>() {
            Err(result.unchecked_into())
        } else {
            Ok(result.unchecked_into())
        }
    }

    /// Applies the binary `**` JS operator on the two `BigInt`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Exponentiation)
    #[inline]
    pub fn pow(&self, rhs: &Self) -> Self {
        JsValue::as_ref(self)
            .pow(JsValue::as_ref(rhs))
            .unchecked_into()
    }
}

macro_rules! bigint_from {
    ($($x:ident)*) => ($(
        impl From<$x> for BigInt {
            #[inline]
            fn from(x: $x) -> BigInt {
                new_bigint_unchecked(&JsValue::from(x))
            }
        }

        impl PartialEq<$x> for BigInt {
            #[inline]
            fn eq(&self, other: &$x) -> bool {
                JsValue::from(self) == JsValue::from(BigInt::from(*other))
            }
        }
    )*)
}
bigint_from!(i8 u8 i16 u16 i32 u32 isize usize);

macro_rules! bigint_from_big {
    ($($x:ident)*) => ($(
        impl From<$x> for BigInt {
            #[inline]
            fn from(x: $x) -> BigInt {
                JsValue::from(x).unchecked_into()
            }
        }

        impl PartialEq<$x> for BigInt {
            #[inline]
            fn eq(&self, other: &$x) -> bool {
                self == &BigInt::from(*other)
            }
        }

        impl TryFrom<BigInt> for $x {
            type Error = BigInt;

            #[inline]
            fn try_from(x: BigInt) -> Result<Self, BigInt> {
                Self::try_from(JsValue::from(x)).map_err(JsCast::unchecked_into)
            }
        }
    )*)
}
bigint_from_big!(i64 u64 i128 u128);

impl PartialEq<Number> for BigInt {
    #[inline]
    fn eq(&self, other: &Number) -> bool {
        JsValue::as_ref(self).loose_eq(JsValue::as_ref(other))
    }
}

impl Not for &BigInt {
    type Output = BigInt;

    #[inline]
    fn not(self) -> Self::Output {
        JsValue::as_ref(self).bit_not().unchecked_into()
    }
}

forward_deref_unop!(impl Not, not for BigInt);
forward_js_unop!(impl Neg, neg for BigInt);
forward_js_binop!(impl BitAnd, bitand for BigInt);
forward_js_binop!(impl BitOr, bitor for BigInt);
forward_js_binop!(impl BitXor, bitxor for BigInt);
forward_js_binop!(impl Shl, shl for BigInt);
forward_js_binop!(impl Shr, shr for BigInt);
forward_js_binop!(impl Add, add for BigInt);
forward_js_binop!(impl Sub, sub for BigInt);
forward_js_binop!(impl Div, div for BigInt);
forward_js_binop!(impl Mul, mul for BigInt);
forward_js_binop!(impl Rem, rem for BigInt);
sum_product!(BigInt);

partialord_ord!(BigInt);

impl Default for BigInt {
    fn default() -> Self {
        BigInt::from(i32::default())
    }
}

impl FromStr for BigInt {
    type Err = Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BigInt::new(&s.into())
    }
}

impl fmt::Debug for BigInt {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for BigInt {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad_integral(self >= &BigInt::from(0), "", &self.to_string_unchecked(10))
    }
}

impl fmt::Binary for BigInt {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad_integral(self >= &BigInt::from(0), "0b", &self.to_string_unchecked(2))
    }
}

impl fmt::Octal for BigInt {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad_integral(self >= &BigInt::from(0), "0o", &self.to_string_unchecked(8))
    }
}

impl fmt::LowerHex for BigInt {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad_integral(
            self >= &BigInt::from(0),
            "0x",
            &self.to_string_unchecked(16),
        )
    }
}

impl fmt::UpperHex for BigInt {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s: String = self.to_string_unchecked(16);
        s.make_ascii_uppercase();
        f.pad_integral(self >= &BigInt::from(0), "0x", &s)
    }
}

// Boolean
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, is_type_of = |v| v.as_bool().is_some(), typescript_type = "boolean")]
    #[derive(Clone, PartialEq, Eq)]
    pub type Boolean;

    /// The `Boolean()` constructor creates an object wrapper for a boolean value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean)
    #[wasm_bindgen(constructor)]
    #[deprecated(note = "recommended to use `Boolean::from` instead")]
    #[allow(deprecated)]
    pub fn new(value: &JsValue) -> Boolean;

    /// The `valueOf()` method returns the primitive value of a `Boolean` object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean/valueOf)
    #[wasm_bindgen(method, js_name = valueOf)]
    pub fn value_of(this: &Boolean) -> bool;
}

impl From<bool> for Boolean {
    #[inline]
    fn from(b: bool) -> Boolean {
        Boolean::unchecked_from_js(JsValue::from(b))
    }
}

impl From<Boolean> for bool {
    #[inline]
    fn from(b: Boolean) -> bool {
        b.value_of()
    }
}

impl PartialEq<bool> for Boolean {
    #[inline]
    fn eq(&self, other: &bool) -> bool {
        self.value_of() == *other
    }
}

impl fmt::Debug for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.value_of(), f)
    }
}

impl fmt::Display for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.value_of(), f)
    }
}

impl Default for Boolean {
    fn default() -> Self {
        Self::from(bool::default())
    }
}

impl Not for &Boolean {
    type Output = Boolean;

    #[inline]
    fn not(self) -> Self::Output {
        (!JsValue::as_ref(self)).into()
    }
}

forward_deref_unop!(impl Not, not for Boolean);

partialord_ord!(Boolean);

// DataView
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "DataView")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type DataView;

    /// The `DataView` view provides a low-level interface for reading and
    /// writing multiple number types in an `ArrayBuffer` irrespective of the
    /// platform's endianness.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView)
    #[wasm_bindgen(constructor)]
    pub fn new(buffer: &ArrayBuffer, byteOffset: usize, byteLength: usize) -> DataView;

    /// The ArrayBuffer referenced by this view. Fixed at construction time and thus read only.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/buffer)
    #[wasm_bindgen(method, getter, structural)]
    pub fn buffer(this: &DataView) -> ArrayBuffer;

    /// The length (in bytes) of this view from the start of its ArrayBuffer.
    /// Fixed at construction time and thus read only.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/byteLength)
    #[wasm_bindgen(method, getter, structural, js_name = byteLength)]
    pub fn byte_length(this: &DataView) -> usize;

    /// The offset (in bytes) of this view from the start of its ArrayBuffer.
    /// Fixed at construction time and thus read only.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/byteOffset)
    #[wasm_bindgen(method, getter, structural, js_name = byteOffset)]
    pub fn byte_offset(this: &DataView) -> usize;

    /// The `getInt8()` method gets a signed 8-bit integer (byte) at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getInt8)
    #[wasm_bindgen(method, js_name = getInt8)]
    pub fn get_int8(this: &DataView, byte_offset: usize) -> i8;

    /// The `getUint8()` method gets a unsigned 8-bit integer (byte) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getUint8)
    #[wasm_bindgen(method, js_name = getUint8)]
    pub fn get_uint8(this: &DataView, byte_offset: usize) -> u8;

    /// The `getInt16()` method gets a signed 16-bit integer (short) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getInt16)
    #[wasm_bindgen(method, js_name = getInt16)]
    pub fn get_int16(this: &DataView, byte_offset: usize) -> i16;

    /// The `getInt16()` method gets a signed 16-bit integer (short) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getInt16)
    #[wasm_bindgen(method, js_name = getInt16)]
    pub fn get_int16_endian(this: &DataView, byte_offset: usize, little_endian: bool) -> i16;

    /// The `getUint16()` method gets an unsigned 16-bit integer (unsigned short) at the specified
    /// byte offset from the start of the view.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getUint16)
    #[wasm_bindgen(method, js_name = getUint16)]
    pub fn get_uint16(this: &DataView, byte_offset: usize) -> u16;

    /// The `getUint16()` method gets an unsigned 16-bit integer (unsigned short) at the specified
    /// byte offset from the start of the view.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getUint16)
    #[wasm_bindgen(method, js_name = getUint16)]
    pub fn get_uint16_endian(this: &DataView, byte_offset: usize, little_endian: bool) -> u16;

    /// The `getInt32()` method gets a signed 32-bit integer (long) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getInt32)
    #[wasm_bindgen(method, js_name = getInt32)]
    pub fn get_int32(this: &DataView, byte_offset: usize) -> i32;

    /// The `getInt32()` method gets a signed 32-bit integer (long) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getInt32)
    #[wasm_bindgen(method, js_name = getInt32)]
    pub fn get_int32_endian(this: &DataView, byte_offset: usize, little_endian: bool) -> i32;

    /// The `getUint32()` method gets an unsigned 32-bit integer (unsigned long) at the specified
    /// byte offset from the start of the view.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getUint32)
    #[wasm_bindgen(method, js_name = getUint32)]
    pub fn get_uint32(this: &DataView, byte_offset: usize) -> u32;

    /// The `getUint32()` method gets an unsigned 32-bit integer (unsigned long) at the specified
    /// byte offset from the start of the view.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getUint32)
    #[wasm_bindgen(method, js_name = getUint32)]
    pub fn get_uint32_endian(this: &DataView, byte_offset: usize, little_endian: bool) -> u32;

    /// The `getFloat32()` method gets a signed 32-bit float (float) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getFloat32)
    #[wasm_bindgen(method, js_name = getFloat32)]
    pub fn get_float32(this: &DataView, byte_offset: usize) -> f32;

    /// The `getFloat32()` method gets a signed 32-bit float (float) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getFloat32)
    #[wasm_bindgen(method, js_name = getFloat32)]
    pub fn get_float32_endian(this: &DataView, byte_offset: usize, little_endian: bool) -> f32;

    /// The `getFloat64()` method gets a signed 64-bit float (double) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getFloat64)
    #[wasm_bindgen(method, js_name = getFloat64)]
    pub fn get_float64(this: &DataView, byte_offset: usize) -> f64;

    /// The `getFloat64()` method gets a signed 64-bit float (double) at the specified
    /// byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getFloat64)
    #[wasm_bindgen(method, js_name = getFloat64)]
    pub fn get_float64_endian(this: &DataView, byte_offset: usize, little_endian: bool) -> f64;

    /// The `setInt8()` method stores a signed 8-bit integer (byte) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setInt8)
    #[wasm_bindgen(method, js_name = setInt8)]
    pub fn set_int8(this: &DataView, byte_offset: usize, value: i8);

    /// The `setUint8()` method stores an unsigned 8-bit integer (byte) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setUint8)
    #[wasm_bindgen(method, js_name = setUint8)]
    pub fn set_uint8(this: &DataView, byte_offset: usize, value: u8);

    /// The `setInt16()` method stores a signed 16-bit integer (short) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setInt16)
    #[wasm_bindgen(method, js_name = setInt16)]
    pub fn set_int16(this: &DataView, byte_offset: usize, value: i16);

    /// The `setInt16()` method stores a signed 16-bit integer (short) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setInt16)
    #[wasm_bindgen(method, js_name = setInt16)]
    pub fn set_int16_endian(this: &DataView, byte_offset: usize, value: i16, little_endian: bool);

    /// The `setUint16()` method stores an unsigned 16-bit integer (unsigned short) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setUint16)
    #[wasm_bindgen(method, js_name = setUint16)]
    pub fn set_uint16(this: &DataView, byte_offset: usize, value: u16);

    /// The `setUint16()` method stores an unsigned 16-bit integer (unsigned short) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setUint16)
    #[wasm_bindgen(method, js_name = setUint16)]
    pub fn set_uint16_endian(this: &DataView, byte_offset: usize, value: u16, little_endian: bool);

    /// The `setInt32()` method stores a signed 32-bit integer (long) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setInt32)
    #[wasm_bindgen(method, js_name = setInt32)]
    pub fn set_int32(this: &DataView, byte_offset: usize, value: i32);

    /// The `setInt32()` method stores a signed 32-bit integer (long) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setInt32)
    #[wasm_bindgen(method, js_name = setInt32)]
    pub fn set_int32_endian(this: &DataView, byte_offset: usize, value: i32, little_endian: bool);

    /// The `setUint32()` method stores an unsigned 32-bit integer (unsigned long) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setUint32)
    #[wasm_bindgen(method, js_name = setUint32)]
    pub fn set_uint32(this: &DataView, byte_offset: usize, value: u32);

    /// The `setUint32()` method stores an unsigned 32-bit integer (unsigned long) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setUint32)
    #[wasm_bindgen(method, js_name = setUint32)]
    pub fn set_uint32_endian(this: &DataView, byte_offset: usize, value: u32, little_endian: bool);

    /// The `setFloat32()` method stores a signed 32-bit float (float) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setFloat32)
    #[wasm_bindgen(method, js_name = setFloat32)]
    pub fn set_float32(this: &DataView, byte_offset: usize, value: f32);

    /// The `setFloat32()` method stores a signed 32-bit float (float) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setFloat32)
    #[wasm_bindgen(method, js_name = setFloat32)]
    pub fn set_float32_endian(this: &DataView, byte_offset: usize, value: f32, little_endian: bool);

    /// The `setFloat64()` method stores a signed 64-bit float (double) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setFloat64)
    #[wasm_bindgen(method, js_name = setFloat64)]
    pub fn set_float64(this: &DataView, byte_offset: usize, value: f64);

    /// The `setFloat64()` method stores a signed 64-bit float (double) value at the
    /// specified byte offset from the start of the DataView.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setFloat64)
    #[wasm_bindgen(method, js_name = setFloat64)]
    pub fn set_float64_endian(this: &DataView, byte_offset: usize, value: f64, little_endian: bool);
}

// Error
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "Error")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Error;

    /// The Error constructor creates an error object.
    /// Instances of Error objects are thrown when runtime errors occur.
    /// The Error object can also be used as a base object for user-defined exceptions.
    /// See below for standard built-in error types.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error)
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Error;
    #[wasm_bindgen(constructor)]
    pub fn new_with_options(message: &str, options: &Object) -> Error;

    /// The cause property is the underlying cause of the error.
    /// Usually this is used to add context to re-thrown errors.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error#differentiate_between_similar_errors)
    #[wasm_bindgen(method, getter, structural)]
    pub fn cause(this: &Error) -> JsValue;
    #[wasm_bindgen(method, setter, structural)]
    pub fn set_cause(this: &Error, cause: &JsValue);

    /// The message property is a human-readable description of the error.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/message)
    #[wasm_bindgen(method, getter, structural)]
    pub fn message(this: &Error) -> JsString;
    #[wasm_bindgen(method, setter, structural)]
    pub fn set_message(this: &Error, message: &str);

    /// The name property represents a name for the type of error. The initial value is "Error".
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/name)
    #[wasm_bindgen(method, getter, structural)]
    pub fn name(this: &Error) -> JsString;
    #[wasm_bindgen(method, setter, structural)]
    pub fn set_name(this: &Error, name: &str);

    /// The `toString()` method returns a string representing the specified Error object
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/toString)
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Error) -> JsString;
}

partialord_ord!(JsString);

// EvalError
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, extends = Error, typescript_type = "EvalError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type EvalError;

    /// The EvalError object indicates an error regarding the global eval() function. This
    /// exception is not thrown by JavaScript anymore, however the EvalError object remains for
    /// compatibility.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/EvalError)
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> EvalError;
}

// Function
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, is_type_of = JsValue::is_function, typescript_type = "Function")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Function;

    /// The `Function` constructor creates a new `Function` object. Calling the
    /// constructor directly can create functions dynamically, but suffers from
    /// security and similar (but far less significant) performance issues
    /// similar to `eval`. However, unlike `eval`, the `Function` constructor
    /// allows executing code in the global scope, prompting better programming
    /// habits and allowing for more efficient code minification.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function)
    #[wasm_bindgen(constructor)]
    pub fn new_with_args(args: &str, body: &str) -> Function;

    /// The `Function` constructor creates a new `Function` object. Calling the
    /// constructor directly can create functions dynamically, but suffers from
    /// security and similar (but far less significant) performance issues
    /// similar to `eval`. However, unlike `eval`, the `Function` constructor
    /// allows executing code in the global scope, prompting better programming
    /// habits and allowing for more efficient code minification.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function)
    #[wasm_bindgen(constructor)]
    pub fn new_no_args(body: &str) -> Function;

    /// The `apply()` method calls a function with a given this value, and arguments provided as an array
    /// (or an array-like object).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/apply)
    #[wasm_bindgen(method, catch)]
    pub fn apply(this: &Function, context: &JsValue, args: &Array) -> Result<JsValue, JsValue>;

    /// The `call()` method calls a function with a given this value and
    /// arguments provided individually.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call)
    #[wasm_bindgen(method, catch, js_name = call)]
    pub fn call0(this: &Function, context: &JsValue) -> Result<JsValue, JsValue>;

    /// The `call()` method calls a function with a given this value and
    /// arguments provided individually.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call)
    #[wasm_bindgen(method, catch, js_name = call)]
    pub fn call1(this: &Function, context: &JsValue, arg1: &JsValue) -> Result<JsValue, JsValue>;

    /// The `call()` method calls a function with a given this value and
    /// arguments provided individually.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call)
    #[wasm_bindgen(method, catch, js_name = call)]
    pub fn call2(
        this: &Function,
        context: &JsValue,
        arg1: &JsValue,
        arg2: &JsValue,
    ) -> Result<JsValue, JsValue>;

    /// The `call()` method calls a function with a given this value and
    /// arguments provided individually.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call)
    #[wasm_bindgen(method, catch, js_name = call)]
    pub fn call3(
        this: &Function,
        context: &JsValue,
        arg1: &JsValue,
        arg2: &JsValue,
        arg3: &JsValue,
    ) -> Result<JsValue, JsValue>;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind(this: &Function, context: &JsValue) -> Function;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind0(this: &Function, context: &JsValue) -> Function;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind1(this: &Function, context: &JsValue, arg1: &JsValue) -> Function;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind2(this: &Function, context: &JsValue, arg1: &JsValue, arg2: &JsValue) -> Function;

    /// The `bind()` method creates a new function that, when called, has its this keyword set to the provided value,
    /// with a given sequence of arguments preceding any provided when the new function is called.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/bind)
    #[wasm_bindgen(method, js_name = bind)]
    pub fn bind3(
        this: &Function,
        context: &JsValue,
        arg1: &JsValue,
        arg2: &JsValue,
        arg3: &JsValue,
    ) -> Function;

    /// The length property indicates the number of arguments expected by the function.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/length)
    #[wasm_bindgen(method, getter, structural)]
    pub fn length(this: &Function) -> u32;

    /// A Function object's read-only name property indicates the function's
    /// name as specified when it was created or "anonymous" for functions
    /// created anonymously.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/name)
    #[wasm_bindgen(method, getter, structural)]
    pub fn name(this: &Function) -> JsString;

    /// The `toString()` method returns a string representing the source code of the function.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/toString)
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Function) -> JsString;
}

impl Function {
    /// Returns the `Function` value of this JS value if it's an instance of a
    /// function.
    ///
    /// If this JS value is not an instance of a function then this returns
    /// `None`.
    #[deprecated(note = "recommended to use dyn_ref instead which is now equivalent")]
    pub fn try_from(val: &JsValue) -> Option<&Function> {
        val.dyn_ref()
    }
}

impl Default for Function {
    fn default() -> Self {
        Self::new_no_args("")
    }
}

// Generator
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "Generator<any, any, any>")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Generator;

    /// The `next()` method returns an object with two properties done and value.
    /// You can also provide a parameter to the next method to send a value to the generator.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/next)
    #[wasm_bindgen(method, structural, catch)]
    pub fn next(this: &Generator, value: &JsValue) -> Result<JsValue, JsValue>;

    /// The `return()` method returns the given value and finishes the generator.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/return)
    #[wasm_bindgen(method, structural, js_name = return)]
    pub fn return_(this: &Generator, value: &JsValue) -> JsValue;

    /// The `throw()` method resumes the execution of a generator by throwing an error into it
    /// and returns an object with two properties done and value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Generator/throw)
    #[wasm_bindgen(method, structural, catch)]
    pub fn throw(this: &Generator, error: &Error) -> Result<JsValue, JsValue>;
}

// Map
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "Map<any, any>")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Map;

    /// The `clear()` method removes all elements from a Map object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/clear)
    #[wasm_bindgen(method)]
    pub fn clear(this: &Map);

    /// The `delete()` method removes the specified element from a Map object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/delete)
    #[wasm_bindgen(method)]
    pub fn delete(this: &Map, key: &JsValue) -> bool;

    /// The `forEach()` method executes a provided function once per each
    /// key/value pair in the Map object, in insertion order.
    /// Note that in Javascript land the `Key` and `Value` are reversed compared to normal expectations:
    /// # Examples
    /// ```
    /// let js_map = Map::new();
    /// js_map.for_each(&mut |value, key| {
    ///     // Do something here...
    /// })
    /// ```
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/forEach)
    #[wasm_bindgen(method, js_name = forEach)]
    pub fn for_each(this: &Map, callback: &mut dyn FnMut(JsValue, JsValue));

    /// The `get()` method returns a specified element from a Map object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/get)
    #[wasm_bindgen(method)]
    pub fn get(this: &Map, key: &JsValue) -> JsValue;

    /// The `has()` method returns a boolean indicating whether an element with
    /// the specified key exists or not.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/has)
    #[wasm_bindgen(method)]
    pub fn has(this: &Map, key: &JsValue) -> bool;

    /// The Map object holds key-value pairs. Any value (both objects and
    /// primitive values) maybe used as either a key or a value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map)
    #[wasm_bindgen(constructor)]
    pub fn new() -> Map;

    /// The `set()` method adds or updates an element with a specified key
    /// and value to a Map object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/set)
    #[wasm_bindgen(method)]
    pub fn set(this: &Map, key: &JsValue, value: &JsValue) -> Map;

    /// The value of size is an integer representing how many entries
    /// the Map object has. A set accessor function for size is undefined;
    /// you can not change this property.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/size)
    #[wasm_bindgen(method, getter, structural)]
    pub fn size(this: &Map) -> u32;
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

// Map Iterator
#[wasm_bindgen]
extern "C" {
    /// The `entries()` method returns a new Iterator object that contains
    /// the [key, value] pairs for each element in the Map object in
    /// insertion order.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/entries)
    #[wasm_bindgen(method)]
    pub fn entries(this: &Map) -> Iterator;

    /// The `keys()` method returns a new Iterator object that contains the
    /// keys for each element in the Map object in insertion order.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/keys)
    #[wasm_bindgen(method)]
    pub fn keys(this: &Map) -> Iterator;

    /// The `values()` method returns a new Iterator object that contains the
    /// values for each element in the Map object in insertion order.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/values)
    #[wasm_bindgen(method)]
    pub fn values(this: &Map) -> Iterator;
}

// Iterator
#[wasm_bindgen]
extern "C" {
    /// Any object that conforms to the JS iterator protocol. For example,
    /// something returned by `myArray[Symbol.iterator]()`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols)
    #[derive(Clone, Debug)]
    #[wasm_bindgen(is_type_of = Iterator::looks_like_iterator, typescript_type = "Iterator<any>")]
    pub type Iterator;

    /// The `next()` method always has to return an object with appropriate
    /// properties including done and value. If a non-object value gets returned
    /// (such as false or undefined), a TypeError ("iterator.next() returned a
    /// non-object value") will be thrown.
    #[wasm_bindgen(catch, method, structural)]
    pub fn next(this: &Iterator) -> Result<IteratorNext, JsValue>;
}

impl Iterator {
    fn looks_like_iterator(it: &JsValue) -> bool {
        #[wasm_bindgen]
        extern "C" {
            type MaybeIterator;

            #[wasm_bindgen(method, getter)]
            fn next(this: &MaybeIterator) -> JsValue;
        }

        if !it.is_object() {
            return false;
        }

        let it = it.unchecked_ref::<MaybeIterator>();

        it.next().is_function()
    }
}

// Async Iterator
#[wasm_bindgen]
extern "C" {
    /// Any object that conforms to the JS async iterator protocol. For example,
    /// something returned by `myObject[Symbol.asyncIterator]()`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for-await...of)
    #[derive(Clone, Debug)]
    #[wasm_bindgen(is_type_of = Iterator::looks_like_iterator, typescript_type = "AsyncIterator<any>")]
    pub type AsyncIterator;

    /// The `next()` method always has to return a Promise which resolves to an object
    /// with appropriate properties including done and value. If a non-object value
    /// gets returned (such as false or undefined), a TypeError ("iterator.next()
    /// returned a non-object value") will be thrown.
    #[wasm_bindgen(catch, method, structural)]
    pub fn next(this: &AsyncIterator) -> Result<Promise, JsValue>;
}

/// An iterator over the JS `Symbol.iterator` iteration protocol.
///
/// Use the `IntoIterator for &js_sys::Iterator` implementation to create this.
pub struct Iter<'a> {
    js: &'a Iterator,
    state: IterState,
}

/// An iterator over the JS `Symbol.iterator` iteration protocol.
///
/// Use the `IntoIterator for js_sys::Iterator` implementation to create this.
pub struct IntoIter {
    js: Iterator,
    state: IterState,
}

struct IterState {
    done: bool,
}

impl<'a> IntoIterator for &'a Iterator {
    type Item = Result<JsValue, JsValue>;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        Iter {
            js: self,
            state: IterState::new(),
        }
    }
}

impl<'a> std::iter::Iterator for Iter<'a> {
    type Item = Result<JsValue, JsValue>;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next(self.js)
    }
}

impl IntoIterator for Iterator {
    type Item = Result<JsValue, JsValue>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter {
            js: self,
            state: IterState::new(),
        }
    }
}

impl std::iter::Iterator for IntoIter {
    type Item = Result<JsValue, JsValue>;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next(&self.js)
    }
}

impl IterState {
    fn new() -> IterState {
        IterState { done: false }
    }

    fn next(&mut self, js: &Iterator) -> Option<Result<JsValue, JsValue>> {
        if self.done {
            return None;
        }
        let next = match js.next() {
            Ok(val) => val,
            Err(e) => {
                self.done = true;
                return Some(Err(e));
            }
        };
        if next.done() {
            self.done = true;
            None
        } else {
            Some(Ok(next.value()))
        }
    }
}

/// Create an iterator over `val` using the JS iteration protocol and
/// `Symbol.iterator`.
pub fn try_iter(val: &JsValue) -> Result<Option<IntoIter>, JsValue> {
    let iter_sym = Symbol::iterator();
    let iter_fn = Reflect::get(val, iter_sym.as_ref())?;

    let iter_fn: Function = match iter_fn.dyn_into() {
        Ok(iter_fn) => iter_fn,
        Err(_) => return Ok(None),
    };

    let it: Iterator = match iter_fn.call0(val)?.dyn_into() {
        Ok(it) => it,
        Err(_) => return Ok(None),
    };

    Ok(Some(it.into_iter()))
}

// IteratorNext
#[wasm_bindgen]
extern "C" {
    /// The result of calling `next()` on a JS iterator.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols)
    #[wasm_bindgen(extends = Object, typescript_type = "IteratorResult<any>")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type IteratorNext;

    /// Has the value `true` if the iterator is past the end of the iterated
    /// sequence. In this case value optionally specifies the return value of
    /// the iterator.
    ///
    /// Has the value `false` if the iterator was able to produce the next value
    /// in the sequence. This is equivalent of not specifying the done property
    /// altogether.
    #[wasm_bindgen(method, getter, structural)]
    pub fn done(this: &IteratorNext) -> bool;

    /// Any JavaScript value returned by the iterator. Can be omitted when done
    /// is true.
    #[wasm_bindgen(method, getter, structural)]
    pub fn value(this: &IteratorNext) -> JsValue;
}

#[allow(non_snake_case)]
pub mod Math {
    use super::*;

    // Math
    #[wasm_bindgen]
    extern "C" {
        /// The `Math.abs()` function returns the absolute value of a number, that is
        /// Math.abs(x) = |x|
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/abs)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn abs(x: f64) -> f64;

        /// The `Math.acos()` function returns the arccosine (in radians) of a
        /// number, that is ∀x∊[-1;1]
        /// Math.acos(x) = arccos(x) = the unique y∊[0;π] such that cos(y)=x
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/acos)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn acos(x: f64) -> f64;

        /// The `Math.acosh()` function returns the hyperbolic arc-cosine of a
        /// number, that is ∀x ≥ 1
        /// Math.acosh(x) = arcosh(x) = the unique y ≥ 0 such that cosh(y) = x
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/acosh)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn acosh(x: f64) -> f64;

        /// The `Math.asin()` function returns the arcsine (in radians) of a
        /// number, that is ∀x ∊ [-1;1]
        /// Math.asin(x) = arcsin(x) = the unique y∊[-π2;π2] such that sin(y) = x
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/asin)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn asin(x: f64) -> f64;

        /// The `Math.asinh()` function returns the hyperbolic arcsine of a
        /// number, that is Math.asinh(x) = arsinh(x) = the unique y such that sinh(y) = x
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/asinh)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn asinh(x: f64) -> f64;

        /// The `Math.atan()` function returns the arctangent (in radians) of a
        /// number, that is Math.atan(x) = arctan(x) = the unique y ∊ [-π2;π2]such that
        /// tan(y) = x
        #[wasm_bindgen(js_namespace = Math)]
        pub fn atan(x: f64) -> f64;

        /// The `Math.atan2()` function returns the arctangent of the quotient of
        /// its arguments.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/atan2)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn atan2(y: f64, x: f64) -> f64;

        /// The `Math.atanh()` function returns the hyperbolic arctangent of a number,
        /// that is ∀x ∊ (-1,1), Math.atanh(x) = arctanh(x) = the unique y such that
        /// tanh(y) = x
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/atanh)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn atanh(x: f64) -> f64;

        /// The `Math.cbrt() `function returns the cube root of a number, that is
        /// Math.cbrt(x) = ∛x = the unique y such that y^3 = x
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cbrt)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn cbrt(x: f64) -> f64;

        /// The `Math.ceil()` function returns the smallest integer greater than
        /// or equal to a given number.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/ceil)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn ceil(x: f64) -> f64;

        /// The `Math.clz32()` function returns the number of leading zero bits in
        /// the 32-bit binary representation of a number.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/clz32)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn clz32(x: i32) -> u32;

        /// The `Math.cos()` static function returns the cosine of the specified angle,
        /// which must be specified in radians. This value is length(adjacent)/length(hypotenuse).
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cos)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn cos(x: f64) -> f64;

        /// The `Math.cosh()` function returns the hyperbolic cosine of a number,
        /// that can be expressed using the constant e.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cosh)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn cosh(x: f64) -> f64;

        /// The `Math.exp()` function returns e^x, where x is the argument, and e is Euler's number
        /// (also known as Napier's constant), the base of the natural logarithms.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/exp)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn exp(x: f64) -> f64;

        /// The `Math.expm1()` function returns e^x - 1, where x is the argument, and e the base of the
        /// natural logarithms.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/expm1)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn expm1(x: f64) -> f64;

        /// The `Math.floor()` function returns the largest integer less than or
        /// equal to a given number.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/floor)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn floor(x: f64) -> f64;

        /// The `Math.fround()` function returns the nearest 32-bit single precision float representation
        /// of a Number.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/fround)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn fround(x: f64) -> f32;

        /// The `Math.hypot()` function returns the square root of the sum of squares of its arguments.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/hypot)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn hypot(x: f64, y: f64) -> f64;

        /// The `Math.imul()` function returns the result of the C-like 32-bit multiplication of the
        /// two parameters.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/imul)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn imul(x: i32, y: i32) -> i32;

        /// The `Math.log()` function returns the natural logarithm (base e) of a number.
        /// The JavaScript `Math.log()` function is equivalent to ln(x) in mathematics.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn log(x: f64) -> f64;

        /// The `Math.log10()` function returns the base 10 logarithm of a number.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log10)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn log10(x: f64) -> f64;

        /// The `Math.log1p()` function returns the natural logarithm (base e) of 1 + a number.
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log1p)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn log1p(x: f64) -> f64;

        /// The `Math.log2()` function returns the base 2 logarithm of a number.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log2)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn log2(x: f64) -> f64;

        /// The `Math.max()` function returns the largest of two numbers.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/max)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn max(x: f64, y: f64) -> f64;

        /// The static function `Math.min()` returns the lowest-valued number passed into it.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/min)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn min(x: f64, y: f64) -> f64;

        /// The `Math.pow()` function returns the base to the exponent power, that is, base^exponent.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/pow)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn pow(base: f64, exponent: f64) -> f64;

        /// The `Math.random()` function returns a floating-point, pseudo-random number
        /// in the range 0–1 (inclusive of 0, but not 1) with approximately uniform distribution
        /// over that range — which you can then scale to your desired range.
        /// The implementation selects the initial seed to the random number generation algorithm;
        /// it cannot be chosen or reset by the user.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/random)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn random() -> f64;

        /// The `Math.round()` function returns the value of a number rounded to the nearest integer.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/round)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn round(x: f64) -> f64;

        /// The `Math.sign()` function returns the sign of a number, indicating whether the number is
        /// positive, negative or zero.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sign)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn sign(x: f64) -> f64;

        /// The `Math.sin()` function returns the sine of a number.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sin)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn sin(x: f64) -> f64;

        /// The `Math.sinh()` function returns the hyperbolic sine of a number, that can be expressed
        /// using the constant e: Math.sinh(x) = (e^x - e^-x)/2
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sinh)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn sinh(x: f64) -> f64;

        /// The `Math.sqrt()` function returns the square root of a number, that is
        /// ∀x ≥ 0, Math.sqrt(x) = √x = the unique y ≥ 0 such that y^2 = x
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sqrt)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn sqrt(x: f64) -> f64;

        /// The `Math.tan()` function returns the tangent of a number.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/tan)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn tan(x: f64) -> f64;

        /// The `Math.tanh()` function returns the hyperbolic tangent of a number, that is
        /// tanh x = sinh x / cosh x = (e^x - e^-x)/(e^x + e^-x) = (e^2x - 1)/(e^2x + 1)
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/tanh)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn tanh(x: f64) -> f64;

        /// The `Math.trunc()` function returns the integer part of a number by removing any fractional
        /// digits.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/trunc)
        #[wasm_bindgen(js_namespace = Math)]
        pub fn trunc(x: f64) -> f64;
    }
}

// Number.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, is_type_of = |v| v.as_f64().is_some(), typescript_type = "number")]
    #[derive(Clone, PartialEq)]
    pub type Number;

    /// The `Number.isFinite()` method determines whether the passed value is a finite number.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isFinite)
    #[wasm_bindgen(static_method_of = Number, js_name = isFinite)]
    pub fn is_finite(value: &JsValue) -> bool;

    /// The `Number.isInteger()` method determines whether the passed value is an integer.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isInteger)
    #[wasm_bindgen(static_method_of = Number, js_name = isInteger)]
    pub fn is_integer(value: &JsValue) -> bool;

    /// The `Number.isNaN()` method determines whether the passed value is `NaN` and its type is Number.
    /// It is a more robust version of the original, global isNaN().
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isNaN)
    #[wasm_bindgen(static_method_of = Number, js_name = isNaN)]
    pub fn is_nan(value: &JsValue) -> bool;

    /// The `Number.isSafeInteger()` method determines whether the provided value is a number
    /// that is a safe integer.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isSafeInteger)
    #[wasm_bindgen(static_method_of = Number, js_name = isSafeInteger)]
    pub fn is_safe_integer(value: &JsValue) -> bool;

    /// The `Number` JavaScript object is a wrapper object allowing
    /// you to work with numerical values. A `Number` object is
    /// created using the `Number()` constructor.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number)
    #[wasm_bindgen(constructor)]
    #[deprecated(note = "recommended to use `Number::from` instead")]
    #[allow(deprecated)]
    pub fn new(value: &JsValue) -> Number;

    #[wasm_bindgen(constructor)]
    fn new_from_str(value: &str) -> Number;

    /// The `Number.parseInt()` method parses a string argument and returns an
    /// integer of the specified radix or base.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/parseInt)
    #[wasm_bindgen(static_method_of = Number, js_name = parseInt)]
    pub fn parse_int(text: &str, radix: u8) -> f64;

    /// The `Number.parseFloat()` method parses a string argument and returns a
    /// floating point number.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/parseFloat)
    #[wasm_bindgen(static_method_of = Number, js_name = parseFloat)]
    pub fn parse_float(text: &str) -> f64;

    /// The `toLocaleString()` method returns a string with a language sensitive
    /// representation of this number.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toLocaleString)
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &Number, locale: &str) -> JsString;

    /// The `toPrecision()` method returns a string representing the Number
    /// object to the specified precision.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toPrecision)
    #[wasm_bindgen(catch, method, js_name = toPrecision)]
    pub fn to_precision(this: &Number, precision: u8) -> Result<JsString, JsValue>;

    /// The `toFixed()` method returns a string representing the Number
    /// object using fixed-point notation.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toFixed)
    #[wasm_bindgen(catch, method, js_name = toFixed)]
    pub fn to_fixed(this: &Number, digits: u8) -> Result<JsString, JsValue>;

    /// The `toExponential()` method returns a string representing the Number
    /// object in exponential notation.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toExponential)
    #[wasm_bindgen(catch, method, js_name = toExponential)]
    pub fn to_exponential(this: &Number, fraction_digits: u8) -> Result<JsString, JsValue>;

    /// The `toString()` method returns a string representing the
    /// specified Number object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toString)
    #[wasm_bindgen(catch, method, js_name = toString)]
    pub fn to_string(this: &Number, radix: u8) -> Result<JsString, JsValue>;

    /// The `valueOf()` method returns the wrapped primitive value of
    /// a Number object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/valueOf)
    #[wasm_bindgen(method, js_name = valueOf)]
    pub fn value_of(this: &Number) -> f64;
}

impl Number {
    /// The smallest interval between two representable numbers.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/EPSILON)
    pub const EPSILON: f64 = f64::EPSILON;
    /// The maximum safe integer in JavaScript (2^53 - 1).
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER)
    pub const MAX_SAFE_INTEGER: f64 = 9007199254740991.0;
    /// The largest positive representable number.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_VALUE)
    pub const MAX_VALUE: f64 = f64::MAX;
    /// The minimum safe integer in JavaScript (-(2^53 - 1)).
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MIN_SAFE_INTEGER)
    pub const MIN_SAFE_INTEGER: f64 = -9007199254740991.0;
    /// The smallest positive representable number—that is, the positive number closest to zero
    /// (without actually being zero).
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MIN_VALUE)
    // Cannot use f64::MIN_POSITIVE since that is the smallest **normal** postitive number.
    pub const MIN_VALUE: f64 = 5E-324;
    /// Special "Not a Number" value.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/NaN)
    pub const NAN: f64 = f64::NAN;
    /// Special value representing negative infinity. Returned on overflow.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/NEGATIVE_INFINITY)
    pub const NEGATIVE_INFINITY: f64 = f64::NEG_INFINITY;
    /// Special value representing infinity. Returned on overflow.
    ///
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/POSITIVE_INFINITY)
    pub const POSITIVE_INFINITY: f64 = f64::INFINITY;

    /// Applies the binary `**` JS operator on the two `Number`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Exponentiation)
    #[inline]
    pub fn pow(&self, rhs: &Self) -> Self {
        JsValue::as_ref(self)
            .pow(JsValue::as_ref(rhs))
            .unchecked_into()
    }

    /// Applies the binary `>>>` JS operator on the two `Number`s.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unsigned_right_shift)
    #[inline]
    pub fn unsigned_shr(&self, rhs: &Self) -> Self {
        Number::from(JsValue::as_ref(self).unsigned_shr(JsValue::as_ref(rhs)))
    }
}

macro_rules! number_from {
    ($($x:ident)*) => ($(
        impl From<$x> for Number {
            #[inline]
            fn from(x: $x) -> Number {
                Number::unchecked_from_js(JsValue::from(x))
            }
        }

        impl PartialEq<$x> for Number {
            #[inline]
            fn eq(&self, other: &$x) -> bool {
                self.value_of() == f64::from(*other)
            }
        }
    )*)
}
number_from!(i8 u8 i16 u16 i32 u32 f32 f64);

// TODO: add this on the next major version, when blanket impl is removed
/*
impl convert::TryFrom<JsValue> for Number {
    type Error = Error;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        return match f64::try_from(value) {
            Ok(num) => Ok(Number::from(num)),
            Err(jsval) => Err(jsval.unchecked_into())
        }
    }
}
*/

impl From<&Number> for f64 {
    #[inline]
    fn from(n: &Number) -> f64 {
        n.value_of()
    }
}

impl From<Number> for f64 {
    #[inline]
    fn from(n: Number) -> f64 {
        <f64 as From<&'_ Number>>::from(&n)
    }
}

impl fmt::Debug for Number {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.value_of(), f)
    }
}

impl fmt::Display for Number {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.value_of(), f)
    }
}

impl Default for Number {
    fn default() -> Self {
        Self::from(f64::default())
    }
}

impl PartialEq<BigInt> for Number {
    #[inline]
    fn eq(&self, other: &BigInt) -> bool {
        JsValue::as_ref(self).loose_eq(JsValue::as_ref(other))
    }
}

impl Not for &Number {
    type Output = BigInt;

    #[inline]
    fn not(self) -> Self::Output {
        JsValue::as_ref(self).bit_not().unchecked_into()
    }
}

forward_deref_unop!(impl Not, not for Number);
forward_js_unop!(impl Neg, neg for Number);
forward_js_binop!(impl BitAnd, bitand for Number);
forward_js_binop!(impl BitOr, bitor for Number);
forward_js_binop!(impl BitXor, bitxor for Number);
forward_js_binop!(impl Shl, shl for Number);
forward_js_binop!(impl Shr, shr for Number);
forward_js_binop!(impl Add, add for Number);
forward_js_binop!(impl Sub, sub for Number);
forward_js_binop!(impl Div, div for Number);
forward_js_binop!(impl Mul, mul for Number);
forward_js_binop!(impl Rem, rem for Number);

sum_product!(Number);

impl PartialOrd for Number {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if Number::is_nan(self) || Number::is_nan(other) {
            None
        } else if self == other {
            Some(Ordering::Equal)
        } else if self.lt(other) {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        JsValue::as_ref(self).lt(JsValue::as_ref(other))
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        JsValue::as_ref(self).le(JsValue::as_ref(other))
    }

    #[inline]
    fn ge(&self, other: &Self) -> bool {
        JsValue::as_ref(self).ge(JsValue::as_ref(other))
    }

    #[inline]
    fn gt(&self, other: &Self) -> bool {
        JsValue::as_ref(self).gt(JsValue::as_ref(other))
    }
}

impl FromStr for Number {
    type Err = Infallible;

    #[allow(deprecated)]
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Number::new_from_str(s))
    }
}

// Date.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "Date")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Date;

    /// The `getDate()` method returns the day of the month for the
    /// specified date according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getDate)
    #[wasm_bindgen(method, js_name = getDate)]
    pub fn get_date(this: &Date) -> u32;

    /// The `getDay()` method returns the day of the week for the specified date according to local time,
    /// where 0 represents Sunday. For the day of the month see getDate().
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getDay)
    #[wasm_bindgen(method, js_name = getDay)]
    pub fn get_day(this: &Date) -> u32;

    /// The `getFullYear()` method returns the year of the specified date according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getFullYear)
    #[wasm_bindgen(method, js_name = getFullYear)]
    pub fn get_full_year(this: &Date) -> u32;

    /// The `getHours()` method returns the hour for the specified date, according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getHours)
    #[wasm_bindgen(method, js_name = getHours)]
    pub fn get_hours(this: &Date) -> u32;

    /// The `getMilliseconds()` method returns the milliseconds in the specified date according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getMilliseconds)
    #[wasm_bindgen(method, js_name = getMilliseconds)]
    pub fn get_milliseconds(this: &Date) -> u32;

    /// The `getMinutes()` method returns the minutes in the specified date according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getMinutes)
    #[wasm_bindgen(method, js_name = getMinutes)]
    pub fn get_minutes(this: &Date) -> u32;

    /// The `getMonth()` method returns the month in the specified date according to local time,
    /// as a zero-based value (where zero indicates the first month of the year).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getMonth)
    #[wasm_bindgen(method, js_name = getMonth)]
    pub fn get_month(this: &Date) -> u32;

    /// The `getSeconds()` method returns the seconds in the specified date according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getSeconds)
    #[wasm_bindgen(method, js_name = getSeconds)]
    pub fn get_seconds(this: &Date) -> u32;

    /// The `getTime()` method returns the numeric value corresponding to the time for the specified date
    /// according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getTime)
    #[wasm_bindgen(method, js_name = getTime)]
    pub fn get_time(this: &Date) -> f64;

    /// The `getTimezoneOffset()` method returns the time zone difference, in minutes,
    /// from current locale (host system settings) to UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getTimezoneOffset)
    #[wasm_bindgen(method, js_name = getTimezoneOffset)]
    pub fn get_timezone_offset(this: &Date) -> f64;

    /// The `getUTCDate()` method returns the day (date) of the month in the specified date
    /// according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCDate)
    #[wasm_bindgen(method, js_name = getUTCDate)]
    pub fn get_utc_date(this: &Date) -> u32;

    /// The `getUTCDay()` method returns the day of the week in the specified date according to universal time,
    /// where 0 represents Sunday.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCDay)
    #[wasm_bindgen(method, js_name = getUTCDay)]
    pub fn get_utc_day(this: &Date) -> u32;

    /// The `getUTCFullYear()` method returns the year in the specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCFullYear)
    #[wasm_bindgen(method, js_name = getUTCFullYear)]
    pub fn get_utc_full_year(this: &Date) -> u32;

    /// The `getUTCHours()` method returns the hours in the specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCHours)
    #[wasm_bindgen(method, js_name = getUTCHours)]
    pub fn get_utc_hours(this: &Date) -> u32;

    /// The `getUTCMilliseconds()` method returns the milliseconds in the specified date
    /// according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCMilliseconds)
    #[wasm_bindgen(method, js_name = getUTCMilliseconds)]
    pub fn get_utc_milliseconds(this: &Date) -> u32;

    /// The `getUTCMinutes()` method returns the minutes in the specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCMinutes)
    #[wasm_bindgen(method, js_name = getUTCMinutes)]
    pub fn get_utc_minutes(this: &Date) -> u32;

    /// The `getUTCMonth()` returns the month of the specified date according to universal time,
    /// as a zero-based value (where zero indicates the first month of the year).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCMonth)
    #[wasm_bindgen(method, js_name = getUTCMonth)]
    pub fn get_utc_month(this: &Date) -> u32;

    /// The `getUTCSeconds()` method returns the seconds in the specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCSeconds)
    #[wasm_bindgen(method, js_name = getUTCSeconds)]
    pub fn get_utc_seconds(this: &Date) -> u32;

    /// Creates a JavaScript `Date` instance that represents
    /// a single moment in time. `Date` objects are based on a time value that is
    /// the number of milliseconds since 1 January 1970 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
    #[wasm_bindgen(constructor)]
    pub fn new(init: &JsValue) -> Date;

    /// Creates a JavaScript `Date` instance that represents the current moment in
    /// time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
    #[wasm_bindgen(constructor)]
    pub fn new_0() -> Date;

    /// Creates a JavaScript `Date` instance that represents
    /// a single moment in time. `Date` objects are based on a time value that is
    /// the number of milliseconds since 1 January 1970 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
    #[wasm_bindgen(constructor)]
    pub fn new_with_year_month(year: u32, month: i32) -> Date;

    /// Creates a JavaScript `Date` instance that represents
    /// a single moment in time. `Date` objects are based on a time value that is
    /// the number of milliseconds since 1 January 1970 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
    #[wasm_bindgen(constructor)]
    pub fn new_with_year_month_day(year: u32, month: i32, day: i32) -> Date;

    /// Creates a JavaScript `Date` instance that represents
    /// a single moment in time. `Date` objects are based on a time value that is
    /// the number of milliseconds since 1 January 1970 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
    #[wasm_bindgen(constructor)]
    pub fn new_with_year_month_day_hr(year: u32, month: i32, day: i32, hr: i32) -> Date;

    /// Creates a JavaScript `Date` instance that represents
    /// a single moment in time. `Date` objects are based on a time value that is
    /// the number of milliseconds since 1 January 1970 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
    #[wasm_bindgen(constructor)]
    pub fn new_with_year_month_day_hr_min(
        year: u32,
        month: i32,
        day: i32,
        hr: i32,
        min: i32,
    ) -> Date;

    /// Creates a JavaScript `Date` instance that represents
    /// a single moment in time. `Date` objects are based on a time value that is
    /// the number of milliseconds since 1 January 1970 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
    #[wasm_bindgen(constructor)]
    pub fn new_with_year_month_day_hr_min_sec(
        year: u32,
        month: i32,
        day: i32,
        hr: i32,
        min: i32,
        sec: i32,
    ) -> Date;

    /// Creates a JavaScript `Date` instance that represents
    /// a single moment in time. `Date` objects are based on a time value that is
    /// the number of milliseconds since 1 January 1970 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date)
    #[wasm_bindgen(constructor)]
    pub fn new_with_year_month_day_hr_min_sec_milli(
        year: u32,
        month: i32,
        day: i32,
        hr: i32,
        min: i32,
        sec: i32,
        milli: i32,
    ) -> Date;

    /// The `Date.now()` method returns the number of milliseconds
    /// elapsed since January 1, 1970 00:00:00 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/now)
    #[wasm_bindgen(static_method_of = Date)]
    pub fn now() -> f64;

    /// The `Date.parse()` method parses a string representation of a date, and returns the number of milliseconds
    /// since January 1, 1970, 00:00:00 UTC or `NaN` if the string is unrecognized or, in some cases,
    /// contains illegal date values (e.g. 2015-02-31).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/parse)
    #[wasm_bindgen(static_method_of = Date)]
    pub fn parse(date: &str) -> f64;

    /// The `setDate()` method sets the day of the Date object relative to the beginning of the currently set month.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setDate)
    #[wasm_bindgen(method, js_name = setDate)]
    pub fn set_date(this: &Date, day: u32) -> f64;

    /// The `setFullYear()` method sets the full year for a specified date according to local time.
    /// Returns new timestamp.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setFullYear)
    #[wasm_bindgen(method, js_name = setFullYear)]
    pub fn set_full_year(this: &Date, year: u32) -> f64;

    /// The `setFullYear()` method sets the full year for a specified date according to local time.
    /// Returns new timestamp.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setFullYear)
    #[wasm_bindgen(method, js_name = setFullYear)]
    pub fn set_full_year_with_month(this: &Date, year: u32, month: i32) -> f64;

    /// The `setFullYear()` method sets the full year for a specified date according to local time.
    /// Returns new timestamp.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setFullYear)
    #[wasm_bindgen(method, js_name = setFullYear)]
    pub fn set_full_year_with_month_date(this: &Date, year: u32, month: i32, date: i32) -> f64;

    /// The `setHours()` method sets the hours for a specified date according to local time,
    /// and returns the number of milliseconds since January 1, 1970 00:00:00 UTC until the time represented
    /// by the updated Date instance.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setHours)
    #[wasm_bindgen(method, js_name = setHours)]
    pub fn set_hours(this: &Date, hours: u32) -> f64;

    /// The `setMilliseconds()` method sets the milliseconds for a specified date according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setMilliseconds)
    #[wasm_bindgen(method, js_name = setMilliseconds)]
    pub fn set_milliseconds(this: &Date, milliseconds: u32) -> f64;

    /// The `setMinutes()` method sets the minutes for a specified date according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setMinutes)
    #[wasm_bindgen(method, js_name = setMinutes)]
    pub fn set_minutes(this: &Date, minutes: u32) -> f64;

    /// The `setMonth()` method sets the month for a specified date according to the currently set year.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setMonth)
    #[wasm_bindgen(method, js_name = setMonth)]
    pub fn set_month(this: &Date, month: u32) -> f64;

    /// The `setSeconds()` method sets the seconds for a specified date according to local time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setSeconds)
    #[wasm_bindgen(method, js_name = setSeconds)]
    pub fn set_seconds(this: &Date, seconds: u32) -> f64;

    /// The `setTime()` method sets the Date object to the time represented by a number of milliseconds
    /// since January 1, 1970, 00:00:00 UTC.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setTime)
    #[wasm_bindgen(method, js_name = setTime)]
    pub fn set_time(this: &Date, time: f64) -> f64;

    /// The `setUTCDate()` method sets the day of the month for a specified date
    /// according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCDate)
    #[wasm_bindgen(method, js_name = setUTCDate)]
    pub fn set_utc_date(this: &Date, day: u32) -> f64;

    /// The `setUTCFullYear()` method sets the full year for a specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCFullYear)
    #[wasm_bindgen(method, js_name = setUTCFullYear)]
    pub fn set_utc_full_year(this: &Date, year: u32) -> f64;

    /// The `setUTCFullYear()` method sets the full year for a specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCFullYear)
    #[wasm_bindgen(method, js_name = setUTCFullYear)]
    pub fn set_utc_full_year_with_month(this: &Date, year: u32, month: i32) -> f64;

    /// The `setUTCFullYear()` method sets the full year for a specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCFullYear)
    #[wasm_bindgen(method, js_name = setUTCFullYear)]
    pub fn set_utc_full_year_with_month_date(this: &Date, year: u32, month: i32, date: i32) -> f64;

    /// The `setUTCHours()` method sets the hour for a specified date according to universal time,
    /// and returns the number of milliseconds since  January 1, 1970 00:00:00 UTC until the time
    /// represented by the updated Date instance.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCHours)
    #[wasm_bindgen(method, js_name = setUTCHours)]
    pub fn set_utc_hours(this: &Date, hours: u32) -> f64;

    /// The `setUTCMilliseconds()` method sets the milliseconds for a specified date
    /// according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCMilliseconds)
    #[wasm_bindgen(method, js_name = setUTCMilliseconds)]
    pub fn set_utc_milliseconds(this: &Date, milliseconds: u32) -> f64;

    /// The `setUTCMinutes()` method sets the minutes for a specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCMinutes)
    #[wasm_bindgen(method, js_name = setUTCMinutes)]
    pub fn set_utc_minutes(this: &Date, minutes: u32) -> f64;

    /// The `setUTCMonth()` method sets the month for a specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCMonth)
    #[wasm_bindgen(method, js_name = setUTCMonth)]
    pub fn set_utc_month(this: &Date, month: u32) -> f64;

    /// The `setUTCSeconds()` method sets the seconds for a specified date according to universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCSeconds)
    #[wasm_bindgen(method, js_name = setUTCSeconds)]
    pub fn set_utc_seconds(this: &Date, seconds: u32) -> f64;

    /// The `toDateString()` method returns the date portion of a Date object
    /// in human readable form in American English.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toDateString)
    #[wasm_bindgen(method, js_name = toDateString)]
    pub fn to_date_string(this: &Date) -> JsString;

    /// The `toISOString()` method returns a string in simplified extended ISO format (ISO
    /// 8601), which is always 24 or 27 characters long (YYYY-MM-DDTHH:mm:ss.sssZ or
    /// ±YYYYYY-MM-DDTHH:mm:ss.sssZ, respectively). The timezone is always zero UTC offset,
    /// as denoted by the suffix "Z"
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toISOString)
    #[wasm_bindgen(method, js_name = toISOString)]
    pub fn to_iso_string(this: &Date) -> JsString;

    /// The `toJSON()` method returns a string representation of the Date object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toJSON)
    #[wasm_bindgen(method, js_name = toJSON)]
    pub fn to_json(this: &Date) -> JsString;

    /// The `toLocaleDateString()` method returns a string with a language sensitive
    /// representation of the date portion of this date. The new locales and options
    /// arguments let applications specify the language whose formatting conventions
    /// should be used and allow to customize the behavior of the function.
    /// In older implementations, which ignore the locales and options arguments,
    /// the locale used and the form of the string
    /// returned are entirely implementation dependent.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toLocaleDateString)
    #[wasm_bindgen(method, js_name = toLocaleDateString)]
    pub fn to_locale_date_string(this: &Date, locale: &str, options: &JsValue) -> JsString;

    /// The `toLocaleString()` method returns a string with a language sensitive
    /// representation of this date. The new locales and options arguments
    /// let applications specify the language whose formatting conventions
    /// should be used and customize the behavior of the function.
    /// In older implementations, which ignore the locales
    /// and options arguments, the locale used and the form of the string
    /// returned are entirely implementation dependent.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toLocaleString)
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &Date, locale: &str, options: &JsValue) -> JsString;

    /// The `toLocaleTimeString()` method returns a string with a language sensitive
    /// representation of the time portion of this date. The new locales and options
    /// arguments let applications specify the language whose formatting conventions should be
    /// used and customize the behavior of the function. In older implementations, which ignore
    /// the locales and options arguments, the locale used and the form of the string
    /// returned are entirely implementation dependent.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toLocaleTimeString)
    #[wasm_bindgen(method, js_name = toLocaleTimeString)]
    pub fn to_locale_time_string(this: &Date, locale: &str) -> JsString;

    /// The `toString()` method returns a string representing
    /// the specified Date object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toString)
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Date) -> JsString;

    /// The `toTimeString()` method returns the time portion of a Date object in human
    /// readable form in American English.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toTimeString)
    #[wasm_bindgen(method, js_name = toTimeString)]
    pub fn to_time_string(this: &Date) -> JsString;

    /// The `toUTCString()` method converts a date to a string,
    /// using the UTC time zone.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toUTCString)
    #[wasm_bindgen(method, js_name = toUTCString)]
    pub fn to_utc_string(this: &Date) -> JsString;

    /// The `Date.UTC()` method accepts the same parameters as the
    /// longest form of the constructor, and returns the number of
    /// milliseconds in a `Date` object since January 1, 1970,
    /// 00:00:00, universal time.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/UTC)
    #[wasm_bindgen(static_method_of = Date, js_name = UTC)]
    pub fn utc(year: f64, month: f64) -> f64;

    /// The `valueOf()` method  returns the primitive value of
    /// a Date object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/valueOf)
    #[wasm_bindgen(method, js_name = valueOf)]
    pub fn value_of(this: &Date) -> f64;
}

// Object.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "object")]
    #[derive(Clone, Debug)]
    pub type Object;

    /// The `Object.assign()` method is used to copy the values of all enumerable
    /// own properties from one or more source objects to a target object. It
    /// will return the target object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/assign)
    #[wasm_bindgen(static_method_of = Object)]
    pub fn assign(target: &Object, source: &Object) -> Object;

    /// The `Object.assign()` method is used to copy the values of all enumerable
    /// own properties from one or more source objects to a target object. It
    /// will return the target object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/assign)
    #[wasm_bindgen(static_method_of = Object, js_name = assign)]
    pub fn assign2(target: &Object, source1: &Object, source2: &Object) -> Object;

    /// The `Object.assign()` method is used to copy the values of all enumerable
    /// own properties from one or more source objects to a target object. It
    /// will return the target object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/assign)
    #[wasm_bindgen(static_method_of = Object, js_name = assign)]
    pub fn assign3(target: &Object, source1: &Object, source2: &Object, source3: &Object)
        -> Object;

    /// The constructor property returns a reference to the `Object` constructor
    /// function that created the instance object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/constructor)
    #[wasm_bindgen(method, getter)]
    pub fn constructor(this: &Object) -> Function;

    /// The `Object.create()` method creates a new object, using an existing
    /// object to provide the newly created object's prototype.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/create)
    #[wasm_bindgen(static_method_of = Object)]
    pub fn create(prototype: &Object) -> Object;

    /// The static method `Object.defineProperty()` defines a new
    /// property directly on an object, or modifies an existing
    /// property on an object, and returns the object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperty)
    #[wasm_bindgen(static_method_of = Object, js_name = defineProperty)]
    pub fn define_property(obj: &Object, prop: &JsValue, descriptor: &Object) -> Object;

    /// The `Object.defineProperties()` method defines new or modifies
    /// existing properties directly on an object, returning the
    /// object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/defineProperties)
    #[wasm_bindgen(static_method_of = Object, js_name = defineProperties)]
    pub fn define_properties(obj: &Object, props: &Object) -> Object;

    /// The `Object.entries()` method returns an array of a given
    /// object's own enumerable property [key, value] pairs, in the
    /// same order as that provided by a for...in loop (the difference
    /// being that a for-in loop enumerates properties in the
    /// prototype chain as well).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/entries)
    #[wasm_bindgen(static_method_of = Object)]
    pub fn entries(object: &Object) -> Array;

    /// The `Object.freeze()` method freezes an object: that is, prevents new
    /// properties from being added to it; prevents existing properties from
    /// being removed; and prevents existing properties, or their enumerability,
    /// configurability, or writability, from being changed, it also prevents
    /// the prototype from being changed. The method returns the passed object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/freeze)
    #[wasm_bindgen(static_method_of = Object)]
    pub fn freeze(value: &Object) -> Object;

    /// The `Object.fromEntries()` method transforms a list of key-value pairs
    /// into an object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/fromEntries)
    #[wasm_bindgen(static_method_of = Object, catch, js_name = fromEntries)]
    pub fn from_entries(iterable: &JsValue) -> Result<Object, JsValue>;

    /// The `Object.getOwnPropertyDescriptor()` method returns a
    /// property descriptor for an own property (that is, one directly
    /// present on an object and not in the object's prototype chain)
    /// of a given object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertyDescriptor)
    #[wasm_bindgen(static_method_of = Object, js_name = getOwnPropertyDescriptor)]
    pub fn get_own_property_descriptor(obj: &Object, prop: &JsValue) -> JsValue;

    /// The `Object.getOwnPropertyDescriptors()` method returns all own
    /// property descriptors of a given object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertyDescriptors)
    #[wasm_bindgen(static_method_of = Object, js_name = getOwnPropertyDescriptors)]
    pub fn get_own_property_descriptors(obj: &Object) -> JsValue;

    /// The `Object.getOwnPropertyNames()` method returns an array of
    /// all properties (including non-enumerable properties except for
    /// those which use Symbol) found directly upon a given object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertyNames)
    #[wasm_bindgen(static_method_of = Object, js_name = getOwnPropertyNames)]
    pub fn get_own_property_names(obj: &Object) -> Array;

    /// The `Object.getOwnPropertySymbols()` method returns an array of
    /// all symbol properties found directly upon a given object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertySymbols)
    #[wasm_bindgen(static_method_of = Object, js_name = getOwnPropertySymbols)]
    pub fn get_own_property_symbols(obj: &Object) -> Array;

    /// The `Object.getPrototypeOf()` method returns the prototype
    /// (i.e. the value of the internal [[Prototype]] property) of the
    /// specified object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getPrototypeOf)
    #[wasm_bindgen(static_method_of = Object, js_name = getPrototypeOf)]
    pub fn get_prototype_of(obj: &JsValue) -> Object;

    /// The `hasOwnProperty()` method returns a boolean indicating whether the
    /// object has the specified property as its own property (as opposed to
    /// inheriting it).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwnProperty)
    #[wasm_bindgen(method, js_name = hasOwnProperty)]
    pub fn has_own_property(this: &Object, property: &JsValue) -> bool;

    /// The `Object.hasOwn()` method returns a boolean indicating whether the
    /// object passed in has the specified property as its own property (as
    /// opposed to inheriting it).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwn)
    #[wasm_bindgen(static_method_of = Object, js_name = hasOwn)]
    pub fn has_own(instance: &Object, property: &JsValue) -> bool;

    /// The `Object.is()` method determines whether two values are the same value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/is)
    #[wasm_bindgen(static_method_of = Object)]
    pub fn is(value_1: &JsValue, value_2: &JsValue) -> bool;

    /// The `Object.isExtensible()` method determines if an object is extensible
    /// (whether it can have new properties added to it).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/isExtensible)
    #[wasm_bindgen(static_method_of = Object, js_name = isExtensible)]
    pub fn is_extensible(object: &Object) -> bool;

    /// The `Object.isFrozen()` determines if an object is frozen.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/isFrozen)
    #[wasm_bindgen(static_method_of = Object, js_name = isFrozen)]
    pub fn is_frozen(object: &Object) -> bool;

    /// The `Object.isSealed()` method determines if an object is sealed.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/isSealed)
    #[wasm_bindgen(static_method_of = Object, js_name = isSealed)]
    pub fn is_sealed(object: &Object) -> bool;

    /// The `isPrototypeOf()` method checks if an object exists in another
    /// object's prototype chain.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/isPrototypeOf)
    #[wasm_bindgen(method, js_name = isPrototypeOf)]
    pub fn is_prototype_of(this: &Object, value: &JsValue) -> bool;

    /// The `Object.keys()` method returns an array of a given object's property
    /// names, in the same order as we get with a normal loop.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/keys)
    #[wasm_bindgen(static_method_of = Object)]
    pub fn keys(object: &Object) -> Array;

    /// The [`Object`] constructor creates an object wrapper.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object)
    #[wasm_bindgen(constructor)]
    pub fn new() -> Object;

    /// The `Object.preventExtensions()` method prevents new properties from
    /// ever being added to an object (i.e. prevents future extensions to the
    /// object).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/preventExtensions)
    #[wasm_bindgen(static_method_of = Object, js_name = preventExtensions)]
    pub fn prevent_extensions(object: &Object);

    /// The `propertyIsEnumerable()` method returns a Boolean indicating
    /// whether the specified property is enumerable.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/propertyIsEnumerable)
    #[wasm_bindgen(method, js_name = propertyIsEnumerable)]
    pub fn property_is_enumerable(this: &Object, property: &JsValue) -> bool;

    /// The `Object.seal()` method seals an object, preventing new properties
    /// from being added to it and marking all existing properties as
    /// non-configurable.  Values of present properties can still be changed as
    /// long as they are writable.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/seal)
    #[wasm_bindgen(static_method_of = Object)]
    pub fn seal(value: &Object) -> Object;

    /// The `Object.setPrototypeOf()` method sets the prototype (i.e., the
    /// internal `[[Prototype]]` property) of a specified object to another
    /// object or `null`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/setPrototypeOf)
    #[wasm_bindgen(static_method_of = Object, js_name = setPrototypeOf)]
    pub fn set_prototype_of(object: &Object, prototype: &Object) -> Object;

    /// The `toLocaleString()` method returns a string representing the object.
    /// This method is meant to be overridden by derived objects for
    /// locale-specific purposes.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/toLocaleString)
    #[wasm_bindgen(method, js_name = toLocaleString)]
    pub fn to_locale_string(this: &Object) -> JsString;

    /// The `toString()` method returns a string representing the object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/toString)
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Object) -> JsString;

    /// The `valueOf()` method returns the primitive value of the
    /// specified object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/valueOf)
    #[wasm_bindgen(method, js_name = valueOf)]
    pub fn value_of(this: &Object) -> Object;

    /// The `Object.values()` method returns an array of a given object's own
    /// enumerable property values, in the same order as that provided by a
    /// `for...in` loop (the difference being that a for-in loop enumerates
    /// properties in the prototype chain as well).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/values)
    #[wasm_bindgen(static_method_of = Object)]
    pub fn values(object: &Object) -> Array;
}

impl Object {
    /// Returns the `Object` value of this JS value if it's an instance of an
    /// object.
    ///
    /// If this JS value is not an instance of an object then this returns
    /// `None`.
    pub fn try_from(val: &JsValue) -> Option<&Object> {
        if val.is_object() {
            Some(val.unchecked_ref())
        } else {
            None
        }
    }
}

impl PartialEq for Object {
    #[inline]
    fn eq(&self, other: &Object) -> bool {
        Object::is(self.as_ref(), other.as_ref())
    }
}

impl Eq for Object {}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

// Proxy
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ProxyConstructor")]
    #[derive(Clone, Debug)]
    pub type Proxy;

    /// The [`Proxy`] object is used to define custom behavior for fundamental
    /// operations (e.g. property lookup, assignment, enumeration, function
    /// invocation, etc).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy)
    #[wasm_bindgen(constructor)]
    pub fn new(target: &JsValue, handler: &Object) -> Proxy;

    /// The `Proxy.revocable()` method is used to create a revocable [`Proxy`]
    /// object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/revocable)
    #[wasm_bindgen(static_method_of = Proxy)]
    pub fn revocable(target: &JsValue, handler: &Object) -> Object;
}

// RangeError
#[wasm_bindgen]
extern "C" {
    /// The `RangeError` object indicates an error when a value is not in the set
    /// or range of allowed values.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RangeError)
    #[wasm_bindgen(extends = Error, extends = Object, typescript_type = "RangeError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type RangeError;

    /// The `RangeError` object indicates an error when a value is not in the set
    /// or range of allowed values.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RangeError)
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> RangeError;
}

// ReferenceError
#[wasm_bindgen]
extern "C" {
    /// The `ReferenceError` object represents an error when a non-existent
    /// variable is referenced.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ReferenceError)
    #[wasm_bindgen(extends = Error, extends = Object, typescript_type = "ReferenceError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type ReferenceError;

    /// The `ReferenceError` object represents an error when a non-existent
    /// variable is referenced.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ReferenceError)
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> ReferenceError;
}

#[allow(non_snake_case)]
pub mod Reflect {
    use super::*;

    // Reflect
    #[wasm_bindgen]
    extern "C" {
        /// The static `Reflect.apply()` method calls a target function with
        /// arguments as specified.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/apply)
        #[wasm_bindgen(js_namespace = Reflect, catch)]
        pub fn apply(
            target: &Function,
            this_argument: &JsValue,
            arguments_list: &Array,
        ) -> Result<JsValue, JsValue>;

        /// The static `Reflect.construct()` method acts like the new operator, but
        /// as a function.  It is equivalent to calling `new target(...args)`. It
        /// gives also the added option to specify a different prototype.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/construct)
        #[wasm_bindgen(js_namespace = Reflect, catch)]
        pub fn construct(target: &Function, arguments_list: &Array) -> Result<JsValue, JsValue>;

        /// The static `Reflect.construct()` method acts like the new operator, but
        /// as a function.  It is equivalent to calling `new target(...args)`. It
        /// gives also the added option to specify a different prototype.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/construct)
        #[wasm_bindgen(js_namespace = Reflect, js_name = construct, catch)]
        pub fn construct_with_new_target(
            target: &Function,
            arguments_list: &Array,
            new_target: &Function,
        ) -> Result<JsValue, JsValue>;

        /// The static `Reflect.defineProperty()` method is like
        /// `Object.defineProperty()` but returns a `Boolean`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/defineProperty)
        #[wasm_bindgen(js_namespace = Reflect, js_name = defineProperty, catch)]
        pub fn define_property(
            target: &Object,
            property_key: &JsValue,
            attributes: &Object,
        ) -> Result<bool, JsValue>;

        /// The static `Reflect.deleteProperty()` method allows to delete
        /// properties.  It is like the `delete` operator as a function.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/deleteProperty)
        #[wasm_bindgen(js_namespace = Reflect, js_name = deleteProperty, catch)]
        pub fn delete_property(target: &Object, key: &JsValue) -> Result<bool, JsValue>;

        /// The static `Reflect.get()` method works like getting a property from
        /// an object (`target[propertyKey]`) as a function.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/get)
        #[wasm_bindgen(js_namespace = Reflect, catch)]
        pub fn get(target: &JsValue, key: &JsValue) -> Result<JsValue, JsValue>;

        /// The same as [`get`](fn.get.html)
        /// except the key is an `f64`, which is slightly faster.
        #[wasm_bindgen(js_namespace = Reflect, js_name = "get", catch)]
        pub fn get_f64(target: &JsValue, key: f64) -> Result<JsValue, JsValue>;

        /// The same as [`get`](fn.get.html)
        /// except the key is a `u32`, which is slightly faster.
        #[wasm_bindgen(js_namespace = Reflect, js_name = "get", catch)]
        pub fn get_u32(target: &JsValue, key: u32) -> Result<JsValue, JsValue>;

        /// The static `Reflect.getOwnPropertyDescriptor()` method is similar to
        /// `Object.getOwnPropertyDescriptor()`. It returns a property descriptor
        /// of the given property if it exists on the object, `undefined` otherwise.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/getOwnPropertyDescriptor)
        #[wasm_bindgen(js_namespace = Reflect, js_name = getOwnPropertyDescriptor, catch)]
        pub fn get_own_property_descriptor(
            target: &Object,
            property_key: &JsValue,
        ) -> Result<JsValue, JsValue>;

        /// The static `Reflect.getPrototypeOf()` method is almost the same
        /// method as `Object.getPrototypeOf()`. It returns the prototype
        /// (i.e. the value of the internal `[[Prototype]]` property) of
        /// the specified object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/getPrototypeOf)
        #[wasm_bindgen(js_namespace = Reflect, js_name = getPrototypeOf, catch)]
        pub fn get_prototype_of(target: &JsValue) -> Result<Object, JsValue>;

        /// The static `Reflect.has()` method works like the in operator as a
        /// function.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/has)
        #[wasm_bindgen(js_namespace = Reflect, catch)]
        pub fn has(target: &JsValue, property_key: &JsValue) -> Result<bool, JsValue>;

        /// The static `Reflect.isExtensible()` method determines if an object is
        /// extensible (whether it can have new properties added to it). It is
        /// similar to `Object.isExtensible()`, but with some differences.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/isExtensible)
        #[wasm_bindgen(js_namespace = Reflect, js_name = isExtensible, catch)]
        pub fn is_extensible(target: &Object) -> Result<bool, JsValue>;

        /// The static `Reflect.ownKeys()` method returns an array of the
        /// target object's own property keys.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/ownKeys)
        #[wasm_bindgen(js_namespace = Reflect, js_name = ownKeys, catch)]
        pub fn own_keys(target: &JsValue) -> Result<Array, JsValue>;

        /// The static `Reflect.preventExtensions()` method prevents new
        /// properties from ever being added to an object (i.e. prevents
        /// future extensions to the object). It is similar to
        /// `Object.preventExtensions()`, but with some differences.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/preventExtensions)
        #[wasm_bindgen(js_namespace = Reflect, js_name = preventExtensions, catch)]
        pub fn prevent_extensions(target: &Object) -> Result<bool, JsValue>;

        /// The static `Reflect.set()` method works like setting a
        /// property on an object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/set)
        #[wasm_bindgen(js_namespace = Reflect, catch)]
        pub fn set(
            target: &JsValue,
            property_key: &JsValue,
            value: &JsValue,
        ) -> Result<bool, JsValue>;

        /// The same as [`set`](fn.set.html)
        /// except the key is an `f64`, which is slightly faster.
        #[wasm_bindgen(js_namespace = Reflect, js_name = "set", catch)]
        pub fn set_f64(
            target: &JsValue,
            property_key: f64,
            value: &JsValue,
        ) -> Result<bool, JsValue>;

        /// The same as [`set`](fn.set.html)
        /// except the key is a `u32`, which is slightly faster.
        #[wasm_bindgen(js_namespace = Reflect, js_name = "set", catch)]
        pub fn set_u32(
            target: &JsValue,
            property_key: u32,
            value: &JsValue,
        ) -> Result<bool, JsValue>;

        /// The static `Reflect.set()` method works like setting a
        /// property on an object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/set)
        #[wasm_bindgen(js_namespace = Reflect, js_name = set, catch)]
        pub fn set_with_receiver(
            target: &JsValue,
            property_key: &JsValue,
            value: &JsValue,
            receiver: &JsValue,
        ) -> Result<bool, JsValue>;

        /// The static `Reflect.setPrototypeOf()` method is the same
        /// method as `Object.setPrototypeOf()`. It sets the prototype
        /// (i.e., the internal `[[Prototype]]` property) of a specified
        /// object to another object or to null.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect/setPrototypeOf)
        #[wasm_bindgen(js_namespace = Reflect, js_name = setPrototypeOf, catch)]
        pub fn set_prototype_of(target: &Object, prototype: &JsValue) -> Result<bool, JsValue>;
    }
}

// RegExp
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "RegExp")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type RegExp;

    /// The `exec()` method executes a search for a match in a specified
    /// string. Returns a result array, or null.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/exec)
    #[wasm_bindgen(method)]
    pub fn exec(this: &RegExp, text: &str) -> Option<Array>;

    /// The flags property returns a string consisting of the flags of
    /// the current regular expression object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/flags)
    #[wasm_bindgen(method, getter)]
    pub fn flags(this: &RegExp) -> JsString;

    /// The global property indicates whether or not the "g" flag is
    /// used with the regular expression. global is a read-only
    /// property of an individual regular expression instance.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/global)
    #[wasm_bindgen(method, getter)]
    pub fn global(this: &RegExp) -> bool;

    /// The ignoreCase property indicates whether or not the "i" flag
    /// is used with the regular expression. ignoreCase is a read-only
    /// property of an individual regular expression instance.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/ignoreCase)
    #[wasm_bindgen(method, getter, js_name = ignoreCase)]
    pub fn ignore_case(this: &RegExp) -> bool;

    /// The non-standard input property is a static property of
    /// regular expressions that contains the string against which a
    /// regular expression is matched. RegExp.$_ is an alias for this
    /// property.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/input)
    #[wasm_bindgen(static_method_of = RegExp, getter)]
    pub fn input() -> JsString;

    /// The lastIndex is a read/write integer property of regular expression
    /// instances that specifies the index at which to start the next match.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/lastIndex)
    #[wasm_bindgen(structural, getter = lastIndex, method)]
    pub fn last_index(this: &RegExp) -> u32;

    /// The lastIndex is a read/write integer property of regular expression
    /// instances that specifies the index at which to start the next match.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/lastIndex)
    #[wasm_bindgen(structural, setter = lastIndex, method)]
    pub fn set_last_index(this: &RegExp, index: u32);

    /// The non-standard lastMatch property is a static and read-only
    /// property of regular expressions that contains the last matched
    /// characters. `RegExp.$&` is an alias for this property.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/lastMatch)
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = lastMatch)]
    pub fn last_match() -> JsString;

    /// The non-standard lastParen property is a static and read-only
    /// property of regular expressions that contains the last
    /// parenthesized substring match, if any. `RegExp.$+` is an alias
    /// for this property.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/lastParen)
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = lastParen)]
    pub fn last_paren() -> JsString;

    /// The non-standard leftContext property is a static and
    /// read-only property of regular expressions that contains the
    /// substring preceding the most recent match. `RegExp.$`` is an
    /// alias for this property.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/leftContext)
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = leftContext)]
    pub fn left_context() -> JsString;

    /// The multiline property indicates whether or not the "m" flag
    /// is used with the regular expression. multiline is a read-only
    /// property of an individual regular expression instance.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/multiline)
    #[wasm_bindgen(method, getter)]
    pub fn multiline(this: &RegExp) -> bool;

    /// The non-standard $1, $2, $3, $4, $5, $6, $7, $8, $9 properties
    /// are static and read-only properties of regular expressions
    /// that contain parenthesized substring matches.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/n)
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$1")]
    pub fn n1() -> JsString;
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$2")]
    pub fn n2() -> JsString;
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$3")]
    pub fn n3() -> JsString;
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$4")]
    pub fn n4() -> JsString;
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$5")]
    pub fn n5() -> JsString;
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$6")]
    pub fn n6() -> JsString;
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$7")]
    pub fn n7() -> JsString;
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$8")]
    pub fn n8() -> JsString;
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = "$9")]
    pub fn n9() -> JsString;

    /// The `RegExp` constructor creates a regular expression object for matching text with a pattern.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp)
    #[wasm_bindgen(constructor)]
    pub fn new(pattern: &str, flags: &str) -> RegExp;
    #[wasm_bindgen(constructor)]
    pub fn new_regexp(pattern: &RegExp, flags: &str) -> RegExp;

    /// The non-standard rightContext property is a static and
    /// read-only property of regular expressions that contains the
    /// substring following the most recent match. `RegExp.$'` is an
    /// alias for this property.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/rightContext)
    #[wasm_bindgen(static_method_of = RegExp, getter, js_name = rightContext)]
    pub fn right_context() -> JsString;

    /// The source property returns a String containing the source
    /// text of the regexp object, and it doesn't contain the two
    /// forward slashes on both sides and any flags.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/source)
    #[wasm_bindgen(method, getter)]
    pub fn source(this: &RegExp) -> JsString;

    /// The sticky property reflects whether or not the search is
    /// sticky (searches in strings only from the index indicated by
    /// the lastIndex property of this regular expression). sticky is
    /// a read-only property of an individual regular expression
    /// object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/sticky)
    #[wasm_bindgen(method, getter)]
    pub fn sticky(this: &RegExp) -> bool;

    /// The `test()` method executes a search for a match between a
    /// regular expression and a specified string. Returns true or
    /// false.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/test)
    #[wasm_bindgen(method)]
    pub fn test(this: &RegExp, text: &str) -> bool;

    /// The `toString()` method returns a string representing the
    /// regular expression.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/toString)
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &RegExp) -> JsString;

    /// The unicode property indicates whether or not the "u" flag is
    /// used with a regular expression. unicode is a read-only
    /// property of an individual regular expression instance.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/unicode)
    #[wasm_bindgen(method, getter)]
    pub fn unicode(this: &RegExp) -> bool;
}

// Set
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "Set<any>")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Set;

    /// The `add()` method appends a new element with a specified value to the
    /// end of a [`Set`] object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/add)
    #[wasm_bindgen(method)]
    pub fn add(this: &Set, value: &JsValue) -> Set;

    /// The `clear()` method removes all elements from a [`Set`] object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/clear)
    #[wasm_bindgen(method)]
    pub fn clear(this: &Set);

    /// The `delete()` method removes the specified element from a [`Set`]
    /// object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/delete)
    #[wasm_bindgen(method)]
    pub fn delete(this: &Set, value: &JsValue) -> bool;

    /// The `forEach()` method executes a provided function once for each value
    /// in the Set object, in insertion order.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/forEach)
    #[wasm_bindgen(method, js_name = forEach)]
    pub fn for_each(this: &Set, callback: &mut dyn FnMut(JsValue, JsValue, Set));

    /// The `has()` method returns a boolean indicating whether an element with
    /// the specified value exists in a [`Set`] object or not.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/has)
    #[wasm_bindgen(method)]
    pub fn has(this: &Set, value: &JsValue) -> bool;

    /// The [`Set`] object lets you store unique values of any type, whether
    /// primitive values or object references.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set)
    #[wasm_bindgen(constructor)]
    pub fn new(init: &JsValue) -> Set;

    /// The size accessor property returns the number of elements in a [`Set`]
    /// object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/de/docs/Web/JavaScript/Reference/Global_Objects/Set/size)
    #[wasm_bindgen(method, getter, structural)]
    pub fn size(this: &Set) -> u32;
}

impl Default for Set {
    fn default() -> Self {
        Self::new(&JsValue::UNDEFINED)
    }
}

// SetIterator
#[wasm_bindgen]
extern "C" {
    /// The `entries()` method returns a new Iterator object that contains an
    /// array of [value, value] for each element in the Set object, in insertion
    /// order. For Set objects there is no key like in Map objects. However, to
    /// keep the API similar to the Map object, each entry has the same value
    /// for its key and value here, so that an array [value, value] is returned.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/entries)
    #[wasm_bindgen(method)]
    pub fn entries(set: &Set) -> Iterator;

    /// The `keys()` method is an alias for this method (for similarity with
    /// Map objects); it behaves exactly the same and returns values
    /// of Set elements.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/values)
    #[wasm_bindgen(method)]
    pub fn keys(set: &Set) -> Iterator;

    /// The `values()` method returns a new Iterator object that contains the
    /// values for each element in the Set object in insertion order.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/values)
    #[wasm_bindgen(method)]
    pub fn values(set: &Set) -> Iterator;
}

// SyntaxError
#[wasm_bindgen]
extern "C" {
    /// A `SyntaxError` is thrown when the JavaScript engine encounters tokens or
    /// token order that does not conform to the syntax of the language when
    /// parsing code.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SyntaxError)
    #[wasm_bindgen(extends = Error, extends = Object, typescript_type = "SyntaxError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type SyntaxError;

    /// A `SyntaxError` is thrown when the JavaScript engine encounters tokens or
    /// token order that does not conform to the syntax of the language when
    /// parsing code.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SyntaxError)
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> SyntaxError;
}

// TypeError
#[wasm_bindgen]
extern "C" {
    /// The `TypeError` object represents an error when a value is not of the
    /// expected type.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError)
    #[wasm_bindgen(extends = Error, extends = Object, typescript_type = "TypeError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type TypeError;

    /// The `TypeError` object represents an error when a value is not of the
    /// expected type.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError)
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> TypeError;
}

// URIError
#[wasm_bindgen]
extern "C" {
    /// The `URIError` object represents an error when a global URI handling
    /// function was used in a wrong way.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/URIError)
    #[wasm_bindgen(extends = Error, extends = Object, js_name = URIError, typescript_type = "URIError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type UriError;

    /// The `URIError` object represents an error when a global URI handling
    /// function was used in a wrong way.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/URIError)
    #[wasm_bindgen(constructor, js_class = "URIError")]
    pub fn new(message: &str) -> UriError;
}

// WeakMap
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "WeakMap<object, any>")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type WeakMap;

    /// The [`WeakMap`] object is a collection of key/value pairs in which the
    /// keys are weakly referenced.  The keys must be objects and the values can
    /// be arbitrary values.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap)
    #[wasm_bindgen(constructor)]
    pub fn new() -> WeakMap;

    /// The `set()` method sets the value for the key in the [`WeakMap`] object.
    /// Returns the [`WeakMap`] object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/set)
    #[wasm_bindgen(method, js_class = "WeakMap")]
    pub fn set(this: &WeakMap, key: &Object, value: &JsValue) -> WeakMap;

    /// The `get()` method returns a specified by key element
    /// from a [`WeakMap`] object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/get)
    #[wasm_bindgen(method)]
    pub fn get(this: &WeakMap, key: &Object) -> JsValue;

    /// The `has()` method returns a boolean indicating whether an element with
    /// the specified key exists in the [`WeakMap`] object or not.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/has)
    #[wasm_bindgen(method)]
    pub fn has(this: &WeakMap, key: &Object) -> bool;

    /// The `delete()` method removes the specified element from a [`WeakMap`]
    /// object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap/delete)
    #[wasm_bindgen(method)]
    pub fn delete(this: &WeakMap, key: &Object) -> bool;
}

impl Default for WeakMap {
    fn default() -> Self {
        Self::new()
    }
}

// WeakSet
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, typescript_type = "WeakSet<object>")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type WeakSet;

    /// The `WeakSet` object lets you store weakly held objects in a collection.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet)
    #[wasm_bindgen(constructor)]
    pub fn new() -> WeakSet;

    /// The `has()` method returns a boolean indicating whether an object exists
    /// in a WeakSet or not.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet/has)
    #[wasm_bindgen(method)]
    pub fn has(this: &WeakSet, value: &Object) -> bool;

    /// The `add()` method appends a new object to the end of a WeakSet object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet/add)
    #[wasm_bindgen(method)]
    pub fn add(this: &WeakSet, value: &Object) -> WeakSet;

    /// The `delete()` method removes the specified element from a WeakSet
    /// object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakSet/delete)
    #[wasm_bindgen(method)]
    pub fn delete(this: &WeakSet, value: &Object) -> bool;
}

impl Default for WeakSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(js_sys_unstable_apis)]
#[allow(non_snake_case)]
pub mod Temporal;

#[allow(non_snake_case)]
pub mod WebAssembly {
    use super::*;

    // WebAssembly
    #[wasm_bindgen]
    extern "C" {
        /// The `WebAssembly.compile()` function compiles a `WebAssembly.Module`
        /// from WebAssembly binary code.  This function is useful if it is
        /// necessary to a compile a module before it can be instantiated
        /// (otherwise, the `WebAssembly.instantiate()` function should be used).
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/compile)
        #[wasm_bindgen(js_namespace = WebAssembly)]
        pub fn compile(buffer_source: &JsValue) -> Promise;

        /// The `WebAssembly.compileStreaming()` function compiles a
        /// `WebAssembly.Module` module directly from a streamed underlying
        /// source. This function is useful if it is necessary to a compile a
        /// module before it can be instantiated (otherwise, the
        /// `WebAssembly.instantiateStreaming()` function should be used).
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/compileStreaming)
        #[wasm_bindgen(js_namespace = WebAssembly, js_name = compileStreaming)]
        pub fn compile_streaming(response: &Promise) -> Promise;

        /// The `WebAssembly.instantiate()` function allows you to compile and
        /// instantiate WebAssembly code.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/instantiate)
        #[wasm_bindgen(js_namespace = WebAssembly, js_name = instantiate)]
        pub fn instantiate_buffer(buffer: &[u8], imports: &Object) -> Promise;

        /// The `WebAssembly.instantiate()` function allows you to compile and
        /// instantiate WebAssembly code.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/instantiate)
        #[wasm_bindgen(js_namespace = WebAssembly, js_name = instantiate)]
        pub fn instantiate_module(module: &Module, imports: &Object) -> Promise;

        /// The `WebAssembly.instantiateStreaming()` function compiles and
        /// instantiates a WebAssembly module directly from a streamed
        /// underlying source. This is the most efficient, optimized way to load
        /// wasm code.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/instantiateStreaming)
        #[wasm_bindgen(js_namespace = WebAssembly, js_name = instantiateStreaming)]
        pub fn instantiate_streaming(response: &Promise, imports: &Object) -> Promise;

        /// The `WebAssembly.validate()` function validates a given typed
        /// array of WebAssembly binary code, returning whether the bytes
        /// form a valid wasm module (`true`) or not (`false`).
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/validate)
        #[wasm_bindgen(js_namespace = WebAssembly, catch)]
        pub fn validate(buffer_source: &JsValue) -> Result<bool, JsValue>;
    }

    // WebAssembly.CompileError
    #[wasm_bindgen]
    extern "C" {
        /// The `WebAssembly.CompileError()` constructor creates a new
        /// WebAssembly `CompileError` object, which indicates an error during
        /// WebAssembly decoding or validation.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/CompileError)
        #[wasm_bindgen(extends = Error, js_namespace = WebAssembly, typescript_type = "WebAssembly.CompileError")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub type CompileError;

        /// The `WebAssembly.CompileError()` constructor creates a new
        /// WebAssembly `CompileError` object, which indicates an error during
        /// WebAssembly decoding or validation.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/CompileError)
        #[wasm_bindgen(constructor, js_namespace = WebAssembly)]
        pub fn new(message: &str) -> CompileError;
    }

    // WebAssembly.Instance
    #[wasm_bindgen]
    extern "C" {
        /// A `WebAssembly.Instance` object is a stateful, executable instance
        /// of a `WebAssembly.Module`. Instance objects contain all the exported
        /// WebAssembly functions that allow calling into WebAssembly code from
        /// JavaScript.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Instance)
        #[wasm_bindgen(extends = Object, js_namespace = WebAssembly, typescript_type = "WebAssembly.Instance")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub type Instance;

        /// The `WebAssembly.Instance()` constructor function can be called to
        /// synchronously instantiate a given `WebAssembly.Module`
        /// object. However, the primary way to get an `Instance` is through the
        /// asynchronous `WebAssembly.instantiateStreaming()` function.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Instance)
        #[wasm_bindgen(catch, constructor, js_namespace = WebAssembly)]
        pub fn new(module: &Module, imports: &Object) -> Result<Instance, JsValue>;

        /// The `exports` readonly property of the `WebAssembly.Instance` object
        /// prototype returns an object containing as its members all the
        /// functions exported from the WebAssembly module instance, to allow
        /// them to be accessed and used by JavaScript.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Instance/exports)
        #[wasm_bindgen(getter, method, js_namespace = WebAssembly)]
        pub fn exports(this: &Instance) -> Object;
    }

    // WebAssembly.LinkError
    #[wasm_bindgen]
    extern "C" {
        /// The `WebAssembly.LinkError()` constructor creates a new WebAssembly
        /// LinkError object, which indicates an error during module
        /// instantiation (besides traps from the start function).
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/LinkError)
        #[wasm_bindgen(extends = Error, js_namespace = WebAssembly, typescript_type = "WebAssembly.LinkError")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub type LinkError;

        /// The `WebAssembly.LinkError()` constructor creates a new WebAssembly
        /// LinkError object, which indicates an error during module
        /// instantiation (besides traps from the start function).
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/LinkError)
        #[wasm_bindgen(constructor, js_namespace = WebAssembly)]
        pub fn new(message: &str) -> LinkError;
    }

    // WebAssembly.RuntimeError
    #[wasm_bindgen]
    extern "C" {
        /// The `WebAssembly.RuntimeError()` constructor creates a new WebAssembly
        /// `RuntimeError` object — the type that is thrown whenever WebAssembly
        /// specifies a trap.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/RuntimeError)
        #[wasm_bindgen(extends = Error, js_namespace = WebAssembly, typescript_type = "WebAssembly.RuntimeError")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub type RuntimeError;

        /// The `WebAssembly.RuntimeError()` constructor creates a new WebAssembly
        /// `RuntimeError` object — the type that is thrown whenever WebAssembly
        /// specifies a trap.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/RuntimeError)
        #[wasm_bindgen(constructor, js_namespace = WebAssembly)]
        pub fn new(message: &str) -> RuntimeError;
    }

    // WebAssembly.Module
    #[wasm_bindgen]
    extern "C" {
        /// A `WebAssembly.Module` object contains stateless WebAssembly code
        /// that has already been compiled by the browser and can be
        /// efficiently shared with Workers, and instantiated multiple times.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Module)
        #[wasm_bindgen(js_namespace = WebAssembly, extends = Object, typescript_type = "WebAssembly.Module")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub type Module;

        /// A `WebAssembly.Module` object contains stateless WebAssembly code
        /// that has already been compiled by the browser and can be
        /// efficiently shared with Workers, and instantiated multiple times.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Module)
        #[wasm_bindgen(constructor, js_namespace = WebAssembly, catch)]
        pub fn new(buffer_source: &JsValue) -> Result<Module, JsValue>;

        /// The `WebAssembly.customSections()` function returns a copy of the
        /// contents of all custom sections in the given module with the given
        /// string name.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Module/customSections)
        #[wasm_bindgen(static_method_of = Module, js_namespace = WebAssembly, js_name = customSections)]
        pub fn custom_sections(module: &Module, sectionName: &str) -> Array;

        /// The `WebAssembly.exports()` function returns an array containing
        /// descriptions of all the declared exports of the given `Module`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Module/exports)
        #[wasm_bindgen(static_method_of = Module, js_namespace = WebAssembly)]
        pub fn exports(module: &Module) -> Array;

        /// The `WebAssembly.imports()` function returns an array containing
        /// descriptions of all the declared imports of the given `Module`.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Module/imports)
        #[wasm_bindgen(static_method_of = Module, js_namespace = WebAssembly)]
        pub fn imports(module: &Module) -> Array;
    }

    // WebAssembly.Table
    #[wasm_bindgen]
    extern "C" {
        /// The `WebAssembly.Table()` constructor creates a new `Table` object
        /// of the given size and element type.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Table)
        #[wasm_bindgen(js_namespace = WebAssembly, extends = Object, typescript_type = "WebAssembly.Table")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub type Table;

        /// The `WebAssembly.Table()` constructor creates a new `Table` object
        /// of the given size and element type.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Table)
        #[wasm_bindgen(constructor, js_namespace = WebAssembly, catch)]
        pub fn new(table_descriptor: &Object) -> Result<Table, JsValue>;

        /// The length prototype property of the `WebAssembly.Table` object
        /// returns the length of the table, i.e. the number of elements in the
        /// table.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Table/length)
        #[wasm_bindgen(method, getter, js_namespace = WebAssembly)]
        pub fn length(this: &Table) -> u32;

        /// The `get()` prototype method of the `WebAssembly.Table()` object
        /// retrieves a function reference stored at a given index.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Table/get)
        #[wasm_bindgen(method, catch, js_namespace = WebAssembly)]
        pub fn get(this: &Table, index: u32) -> Result<Function, JsValue>;

        /// The `grow()` prototype method of the `WebAssembly.Table` object
        /// increases the size of the `Table` instance by a specified number of
        /// elements.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Table/grow)
        #[wasm_bindgen(method, catch, js_namespace = WebAssembly)]
        pub fn grow(this: &Table, additional_capacity: u32) -> Result<u32, JsValue>;

        /// The `set()` prototype method of the `WebAssembly.Table` object mutates a
        /// reference stored at a given index to a different value.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Table/set)
        #[wasm_bindgen(method, catch, js_namespace = WebAssembly)]
        pub fn set(this: &Table, index: u32, function: &Function) -> Result<(), JsValue>;
    }

    // WebAssembly.Global
    #[wasm_bindgen]
    extern "C" {
        /// The `WebAssembly.Global()` constructor creates a new `Global` object
        /// of the given type and value.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Global)
        #[wasm_bindgen(js_namespace = WebAssembly, extends = Object, typescript_type = "WebAssembly.Global")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub type Global;

        /// The `WebAssembly.Global()` constructor creates a new `Global` object
        /// of the given type and value.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Global)
        #[wasm_bindgen(constructor, js_namespace = WebAssembly, catch)]
        pub fn new(global_descriptor: &Object, value: &JsValue) -> Result<Global, JsValue>;

        /// The value prototype property of the `WebAssembly.Global` object
        /// returns the value of the global.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Global)
        #[wasm_bindgen(method, getter, structural, js_namespace = WebAssembly)]
        pub fn value(this: &Global) -> JsValue;
        #[wasm_bindgen(method, setter = value, structural, js_namespace = WebAssembly)]
        pub fn set_value(this: &Global, value: &JsValue);
    }

    // WebAssembly.Memory
    #[wasm_bindgen]
    extern "C" {
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Memory)
        #[wasm_bindgen(js_namespace = WebAssembly, extends = Object, typescript_type = "WebAssembly.Memory")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub type Memory;

        /// The `WebAssembly.Memory()` constructor creates a new `Memory` object
        /// which is a resizable `ArrayBuffer` that holds the raw bytes of
        /// memory accessed by a WebAssembly `Instance`.
        ///
        /// A memory created by JavaScript or in WebAssembly code will be
        /// accessible and mutable from both JavaScript and WebAssembly.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Memory)
        #[wasm_bindgen(constructor, js_namespace = WebAssembly, catch)]
        pub fn new(descriptor: &Object) -> Result<Memory, JsValue>;

        /// An accessor property that returns the buffer contained in the
        /// memory.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Memory/buffer)
        #[wasm_bindgen(method, getter, js_namespace = WebAssembly)]
        pub fn buffer(this: &Memory) -> JsValue;

        /// The `grow()` prototype method of the `Memory` object increases the
        /// size of the memory instance by a specified number of WebAssembly
        /// pages.
        ///
        /// Takes the number of pages to grow (64KiB in size) and returns the
        /// previous size of memory, in pages.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Memory/grow)
        #[wasm_bindgen(method, js_namespace = WebAssembly)]
        pub fn grow(this: &Memory, pages: u32) -> u32;
    }
}

/// The `JSON` object contains methods for parsing [JavaScript Object
/// Notation (JSON)](https://json.org/) and converting values to JSON. It
/// can't be called or constructed, and aside from its two method
/// properties, it has no interesting functionality of its own.
#[allow(non_snake_case)]
pub mod JSON {
    use super::*;

    // JSON
    #[wasm_bindgen]
    extern "C" {
        /// The `JSON.parse()` method parses a JSON string, constructing the
        /// JavaScript value or object described by the string.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse)
        #[wasm_bindgen(catch, js_namespace = JSON)]
        pub fn parse(text: &str) -> Result<JsValue, JsValue>;

        /// The `JSON.stringify()` method converts a JavaScript value to a JSON string.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify)
        #[wasm_bindgen(catch, js_namespace = JSON)]
        pub fn stringify(obj: &JsValue) -> Result<JsString, JsValue>;

        /// The `JSON.stringify()` method converts a JavaScript value to a JSON string.
        ///
        /// The `replacer` argument is a function that alters the behavior of the stringification
        /// process, or an array of String and Number objects that serve as a whitelist
        /// for selecting/filtering the properties of the value object to be included
        /// in the JSON string. If this value is null or not provided, all properties
        /// of the object are included in the resulting JSON string.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify)
        #[wasm_bindgen(catch, js_namespace = JSON, js_name = stringify)]
        pub fn stringify_with_replacer(
            obj: &JsValue,
            replacer: &JsValue,
        ) -> Result<JsString, JsValue>;

        /// The `JSON.stringify()` method converts a JavaScript value to a JSON string.
        ///
        /// The `replacer` argument is a function that alters the behavior of the stringification
        /// process, or an array of String and Number objects that serve as a whitelist
        /// for selecting/filtering the properties of the value object to be included
        /// in the JSON string. If this value is null or not provided, all properties
        /// of the object are included in the resulting JSON string.
        ///
        /// The `space` argument is a String or Number object that's used to insert white space into
        /// the output JSON string for readability purposes. If this is a Number, it
        /// indicates the number of space characters to use as white space; this number
        /// is capped at 10 (if it is greater, the value is just 10). Values less than
        /// 1 indicate that no space should be used. If this is a String, the string
        /// (or the first 10 characters of the string, if it's longer than that) is
        /// used as white space. If this parameter is not provided (or is null), no
        /// white space is used.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify)
        #[wasm_bindgen(catch, js_namespace = JSON, js_name = stringify)]
        pub fn stringify_with_replacer_and_space(
            obj: &JsValue,
            replacer: &JsValue,
            space: &JsValue,
        ) -> Result<JsString, JsValue>;

    }
}

// JsString
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = String, extends = Object, is_type_of = JsValue::is_string, typescript_type = "string")]
    #[derive(Clone, PartialEq, Eq)]
    pub type JsString;

    /// The length property of a String object indicates the length of a string,
    /// in UTF-16 code units.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/length)
    #[wasm_bindgen(method, getter, structural)]
    pub fn length(this: &JsString) -> u32;

    /// The 'at()' method returns a new string consisting of the single UTF-16
    /// code unit located at the specified offset into the string, counting from
    /// the end if it's negative.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/at)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn at(this: &JsString, index: i32) -> Option<JsString>;

    /// The String object's `charAt()` method returns a new string consisting of
    /// the single UTF-16 code unit located at the specified offset into the
    /// string.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/charAt)
    #[wasm_bindgen(method, js_class = "String", js_name = charAt)]
    pub fn char_at(this: &JsString, index: u32) -> JsString;

    /// The `charCodeAt()` method returns an integer between 0 and 65535
    /// representing the UTF-16 code unit at the given index (the UTF-16 code
    /// unit matches the Unicode code point for code points representable in a
    /// single UTF-16 code unit, but might also be the first code unit of a
    /// surrogate pair for code points not representable in a single UTF-16 code
    /// unit, e.g. Unicode code points > 0x10000).  If you want the entire code
    /// point value, use `codePointAt()`.
    ///
    /// Returns `NaN` if index is out of range.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/charCodeAt)
    #[wasm_bindgen(method, js_class = "String", js_name = charCodeAt)]
    pub fn char_code_at(this: &JsString, index: u32) -> f64;

    /// The `codePointAt()` method returns a non-negative integer that is the
    /// Unicode code point value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/codePointAt)
    #[wasm_bindgen(method, js_class = "String", js_name = codePointAt)]
    pub fn code_point_at(this: &JsString, pos: u32) -> JsValue;

    /// The `concat()` method concatenates the string arguments to the calling
    /// string and returns a new string.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/concat)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn concat(this: &JsString, string_2: &JsValue) -> JsString;

    /// The `endsWith()` method determines whether a string ends with the characters of a
    /// specified string, returning true or false as appropriate.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/endsWith)
    #[wasm_bindgen(method, js_class = "String", js_name = endsWith)]
    pub fn ends_with(this: &JsString, search_string: &str, length: i32) -> bool;

    /// The static `String.fromCharCode()` method returns a string created from
    /// the specified sequence of UTF-16 code units.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCharCode)
    ///
    /// # Notes
    ///
    /// There are a few bindings to `from_char_code` in `js-sys`: `from_char_code1`, `from_char_code2`, etc...
    /// with different arities.
    ///
    /// Additionally, this function accepts `u16` for character codes, but
    /// fixing others requires a breaking change release
    /// (see https://github.com/rustwasm/wasm-bindgen/issues/1460 for details).
    #[wasm_bindgen(static_method_of = JsString, js_class = "String", js_name = fromCharCode, variadic)]
    pub fn from_char_code(char_codes: &[u16]) -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCharCode)
    #[wasm_bindgen(static_method_of = JsString, js_class = "String", js_name = fromCharCode)]
    pub fn from_char_code1(a: u32) -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCharCode)
    #[wasm_bindgen(static_method_of = JsString, js_class = "String", js_name = fromCharCode)]
    pub fn from_char_code2(a: u32, b: u32) -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCharCode)
    #[wasm_bindgen(static_method_of = JsString, js_class = "String", js_name = fromCharCode)]
    pub fn from_char_code3(a: u32, b: u32, c: u32) -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCharCode)
    #[wasm_bindgen(static_method_of = JsString, js_class = "String", js_name = fromCharCode)]
    pub fn from_char_code4(a: u32, b: u32, c: u32, d: u32) -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCharCode)
    #[wasm_bindgen(static_method_of = JsString, js_class = "String", js_name = fromCharCode)]
    pub fn from_char_code5(a: u32, b: u32, c: u32, d: u32, e: u32) -> JsString;

    /// The static `String.fromCodePoint()` method returns a string created by
    /// using the specified sequence of code points.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint)
    ///
    /// # Exceptions
    ///
    /// A RangeError is thrown if an invalid Unicode code point is given
    ///
    /// # Notes
    ///
    /// There are a few bindings to `from_code_point` in `js-sys`: `from_code_point1`, `from_code_point2`, etc...
    /// with different arities.
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = fromCodePoint, variadic)]
    pub fn from_code_point(code_points: &[u32]) -> Result<JsString, JsValue>;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = fromCodePoint)]
    pub fn from_code_point1(a: u32) -> Result<JsString, JsValue>;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = fromCodePoint)]
    pub fn from_code_point2(a: u32, b: u32) -> Result<JsString, JsValue>;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = fromCodePoint)]
    pub fn from_code_point3(a: u32, b: u32, c: u32) -> Result<JsString, JsValue>;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = fromCodePoint)]
    pub fn from_code_point4(a: u32, b: u32, c: u32, d: u32) -> Result<JsString, JsValue>;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = fromCodePoint)]
    pub fn from_code_point5(a: u32, b: u32, c: u32, d: u32, e: u32) -> Result<JsString, JsValue>;

    /// The `includes()` method determines whether one string may be found
    /// within another string, returning true or false as appropriate.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/includes)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn includes(this: &JsString, search_string: &str, position: i32) -> bool;

    /// The `indexOf()` method returns the index within the calling String
    /// object of the first occurrence of the specified value, starting the
    /// search at fromIndex.  Returns -1 if the value is not found.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/indexOf)
    #[wasm_bindgen(method, js_class = "String", js_name = indexOf)]
    pub fn index_of(this: &JsString, search_value: &str, from_index: i32) -> i32;

    /// The `lastIndexOf()` method returns the index within the calling String
    /// object of the last occurrence of the specified value, searching
    /// backwards from fromIndex.  Returns -1 if the value is not found.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/lastIndexOf)
    #[wasm_bindgen(method, js_class = "String", js_name = lastIndexOf)]
    pub fn last_index_of(this: &JsString, search_value: &str, from_index: i32) -> i32;

    /// The `localeCompare()` method returns a number indicating whether
    /// a reference string comes before or after or is the same as
    /// the given string in sort order.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/localeCompare)
    #[wasm_bindgen(method, js_class = "String", js_name = localeCompare)]
    pub fn locale_compare(
        this: &JsString,
        compare_string: &str,
        locales: &Array,
        options: &Object,
    ) -> i32;

    /// The `match()` method retrieves the matches when matching a string against a regular expression.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/match)
    #[wasm_bindgen(method, js_class = "String", js_name = match)]
    pub fn match_(this: &JsString, pattern: &RegExp) -> Option<Object>;

    /// The `match_all()` method is similar to `match()`, but gives an iterator of `exec()` arrays, which preserve capture groups.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/matchAll)
    #[wasm_bindgen(method, js_class = "String", js_name = matchAll)]
    pub fn match_all(this: &JsString, pattern: &RegExp) -> Iterator;

    /// The `normalize()` method returns the Unicode Normalization Form
    /// of a given string (if the value isn't a string, it will be converted to one first).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/normalize)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn normalize(this: &JsString, form: &str) -> JsString;

    /// The `padEnd()` method pads the current string with a given string
    /// (repeated, if needed) so that the resulting string reaches a given
    /// length. The padding is applied from the end (right) of the current
    /// string.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/padEnd)
    #[wasm_bindgen(method, js_class = "String", js_name = padEnd)]
    pub fn pad_end(this: &JsString, target_length: u32, pad_string: &str) -> JsString;

    /// The `padStart()` method pads the current string with another string
    /// (repeated, if needed) so that the resulting string reaches the given
    /// length. The padding is applied from the start (left) of the current
    /// string.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/padStart)
    #[wasm_bindgen(method, js_class = "String", js_name = padStart)]
    pub fn pad_start(this: &JsString, target_length: u32, pad_string: &str) -> JsString;

    /// The `repeat()` method constructs and returns a new string which contains the specified
    /// number of copies of the string on which it was called, concatenated together.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/repeat)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn repeat(this: &JsString, count: i32) -> JsString;

    /// The `replace()` method returns a new string with some or all matches of a pattern
    /// replaced by a replacement. The pattern can be a string or a RegExp, and
    /// the replacement can be a string or a function to be called for each match.
    ///
    /// Note: The original string will remain unchanged.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replace)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn replace(this: &JsString, pattern: &str, replacement: &str) -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replace)
    #[wasm_bindgen(method, js_class = "String", js_name = replace)]
    pub fn replace_with_function(
        this: &JsString,
        pattern: &str,
        replacement: &Function,
    ) -> JsString;

    #[wasm_bindgen(method, js_class = "String", js_name = replace)]
    pub fn replace_by_pattern(this: &JsString, pattern: &RegExp, replacement: &str) -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replace)
    #[wasm_bindgen(method, js_class = "String", js_name = replace)]
    pub fn replace_by_pattern_with_function(
        this: &JsString,
        pattern: &RegExp,
        replacement: &Function,
    ) -> JsString;

    /// The `replace_all()` method returns a new string with all matches of a pattern
    /// replaced by a replacement. The pattern can be a string or a global RegExp, and
    /// the replacement can be a string or a function to be called for each match.
    ///
    /// Note: The original string will remain unchanged.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replaceAll)
    #[wasm_bindgen(method, js_class = "String", js_name = replaceAll)]
    pub fn replace_all(this: &JsString, pattern: &str, replacement: &str) -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replaceAll)
    #[wasm_bindgen(method, js_class = "String", js_name = replaceAll)]
    pub fn replace_all_with_function(
        this: &JsString,
        pattern: &str,
        replacement: &Function,
    ) -> JsString;

    #[wasm_bindgen(method, js_class = "String", js_name = replaceAll)]
    pub fn replace_all_by_pattern(this: &JsString, pattern: &RegExp, replacement: &str)
        -> JsString;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replaceAll)
    #[wasm_bindgen(method, js_class = "String", js_name = replaceAll)]
    pub fn replace_all_by_pattern_with_function(
        this: &JsString,
        pattern: &RegExp,
        replacement: &Function,
    ) -> JsString;

    /// The `search()` method executes a search for a match between
    /// a regular expression and this String object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/search)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn search(this: &JsString, pattern: &RegExp) -> i32;

    /// The `slice()` method extracts a section of a string and returns it as a
    /// new string, without modifying the original string.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/slice)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn slice(this: &JsString, start: u32, end: u32) -> JsString;

    /// The `split()` method splits a String object into an array of strings by separating the string
    /// into substrings, using a specified separator string to determine where to make each split.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/split)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn split(this: &JsString, separator: &str) -> Array;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/split)
    #[wasm_bindgen(method, js_class = "String", js_name = split)]
    pub fn split_limit(this: &JsString, separator: &str, limit: u32) -> Array;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/split)
    #[wasm_bindgen(method, js_class = "String", js_name = split)]
    pub fn split_by_pattern(this: &JsString, pattern: &RegExp) -> Array;

    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/split)
    #[wasm_bindgen(method, js_class = "String", js_name = split)]
    pub fn split_by_pattern_limit(this: &JsString, pattern: &RegExp, limit: u32) -> Array;

    /// The `startsWith()` method determines whether a string begins with the
    /// characters of a specified string, returning true or false as
    /// appropriate.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/startsWith)
    #[wasm_bindgen(method, js_class = "String", js_name = startsWith)]
    pub fn starts_with(this: &JsString, search_string: &str, position: u32) -> bool;

    /// The `substring()` method returns the part of the string between the
    /// start and end indexes, or to the end of the string.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substring)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn substring(this: &JsString, index_start: u32, index_end: u32) -> JsString;

    /// The `substr()` method returns the part of a string between
    /// the start index and a number of characters after it.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substr)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn substr(this: &JsString, start: i32, length: i32) -> JsString;

    /// The `toLocaleLowerCase()` method returns the calling string value converted to lower case,
    /// according to any locale-specific case mappings.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/toLocaleLowerCase)
    #[wasm_bindgen(method, js_class = "String", js_name = toLocaleLowerCase)]
    pub fn to_locale_lower_case(this: &JsString, locale: Option<&str>) -> JsString;

    /// The `toLocaleUpperCase()` method returns the calling string value converted to upper case,
    /// according to any locale-specific case mappings.
    ///
    /// [MDN documentation](https://developer.mozilla.org/ja/docs/Web/JavaScript/Reference/Global_Objects/String/toLocaleUpperCase)
    #[wasm_bindgen(method, js_class = "String", js_name = toLocaleUpperCase)]
    pub fn to_locale_upper_case(this: &JsString, locale: Option<&str>) -> JsString;

    /// The `toLowerCase()` method returns the calling string value
    /// converted to lower case.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/toLowerCase)
    #[wasm_bindgen(method, js_class = "String", js_name = toLowerCase)]
    pub fn to_lower_case(this: &JsString) -> JsString;

    /// The `toString()` method returns a string representing the specified
    /// object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/toString)
    #[wasm_bindgen(method, js_class = "String", js_name = toString)]
    pub fn to_string(this: &JsString) -> JsString;

    /// The `toUpperCase()` method returns the calling string value converted to
    /// uppercase (the value will be converted to a string if it isn't one).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/toUpperCase)
    #[wasm_bindgen(method, js_class = "String", js_name = toUpperCase)]
    pub fn to_upper_case(this: &JsString) -> JsString;

    /// The `trim()` method removes whitespace from both ends of a string.
    /// Whitespace in this context is all the whitespace characters (space, tab,
    /// no-break space, etc.) and all the line terminator characters (LF, CR,
    /// etc.).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trim)
    #[wasm_bindgen(method, js_class = "String")]
    pub fn trim(this: &JsString) -> JsString;

    /// The `trimEnd()` method removes whitespace from the end of a string.
    /// `trimRight()` is an alias of this method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimEnd)
    #[wasm_bindgen(method, js_class = "String", js_name = trimEnd)]
    pub fn trim_end(this: &JsString) -> JsString;

    /// The `trimEnd()` method removes whitespace from the end of a string.
    /// `trimRight()` is an alias of this method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimEnd)
    #[wasm_bindgen(method, js_class = "String", js_name = trimRight)]
    pub fn trim_right(this: &JsString) -> JsString;

    /// The `trimStart()` method removes whitespace from the beginning of a
    /// string. `trimLeft()` is an alias of this method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimStart)
    #[wasm_bindgen(method, js_class = "String", js_name = trimStart)]
    pub fn trim_start(this: &JsString) -> JsString;

    /// The `trimStart()` method removes whitespace from the beginning of a
    /// string. `trimLeft()` is an alias of this method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimStart)
    #[wasm_bindgen(method, js_class = "String", js_name = trimLeft)]
    pub fn trim_left(this: &JsString) -> JsString;

    /// The `valueOf()` method returns the primitive value of a `String` object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/valueOf)
    #[wasm_bindgen(method, js_class = "String", js_name = valueOf)]
    pub fn value_of(this: &JsString) -> JsString;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, variadic, static_method_of = JsString, js_class = "String")]
    pub fn raw(call_site: &Object, substitutions: &Array) -> Result<JsString, JsValue>;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = raw)]
    pub fn raw_0(call_site: &Object) -> Result<JsString, JsValue>;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = raw)]
    pub fn raw_1(call_site: &Object, substitutions_1: &str) -> Result<JsString, JsValue>;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = raw)]
    pub fn raw_2(
        call_site: &Object,
        substitutions_1: &str,
        substitutions_2: &str,
    ) -> Result<JsString, JsValue>;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = raw)]
    pub fn raw_3(
        call_site: &Object,
        substitutions_1: &str,
        substitutions_2: &str,
        substitutions_3: &str,
    ) -> Result<JsString, JsValue>;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = raw)]
    pub fn raw_4(
        call_site: &Object,
        substitutions_1: &str,
        substitutions_2: &str,
        substitutions_3: &str,
        substitutions_4: &str,
    ) -> Result<JsString, JsValue>;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = raw)]
    pub fn raw_5(
        call_site: &Object,
        substitutions_1: &str,
        substitutions_2: &str,
        substitutions_3: &str,
        substitutions_4: &str,
        substitutions_5: &str,
    ) -> Result<JsString, JsValue>;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = raw)]
    pub fn raw_6(
        call_site: &Object,
        substitutions_1: &str,
        substitutions_2: &str,
        substitutions_3: &str,
        substitutions_4: &str,
        substitutions_5: &str,
        substitutions_6: &str,
    ) -> Result<JsString, JsValue>;

    /// The static `raw()` method is a tag function of template literals,
    /// similar to the `r` prefix in Python or the `@` prefix in C# for string literals.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw)
    #[wasm_bindgen(catch, static_method_of = JsString, js_class = "String", js_name = raw)]
    pub fn raw_7(
        call_site: &Object,
        substitutions_1: &str,
        substitutions_2: &str,
        substitutions_3: &str,
        substitutions_4: &str,
        substitutions_5: &str,
        substitutions_6: &str,
        substitutions_7: &str,
    ) -> Result<JsString, JsValue>;
}

impl JsString {
    /// Returns the `JsString` value of this JS value if it's an instance of a
    /// string.
    ///
    /// If this JS value is not an instance of a string then this returns
    /// `None`.
    #[deprecated(note = "recommended to use dyn_ref instead which is now equivalent")]
    pub fn try_from(val: &JsValue) -> Option<&JsString> {
        val.dyn_ref()
    }

    /// Returns whether this string is a valid UTF-16 string.
    ///
    /// This is useful for learning whether `String::from(..)` will return a
    /// lossless representation of the JS string. If this string contains
    /// unpaired surrogates then `String::from` will succeed but it will be a
    /// lossy representation of the JS string because unpaired surrogates will
    /// become replacement characters.
    ///
    /// If this function returns `false` then to get a lossless representation
    /// of the string you'll need to manually use the `iter` method (or the
    /// `char_code_at` accessor) to view the raw character codes.
    ///
    /// For more information, see the documentation on [JS strings vs Rust
    /// strings][docs]
    ///
    /// [docs]: https://rustwasm.github.io/docs/wasm-bindgen/reference/types/str.html
    pub fn is_valid_utf16(&self) -> bool {
        std::char::decode_utf16(self.iter()).all(|i| i.is_ok())
    }

    /// Returns an iterator over the `u16` character codes that make up this JS
    /// string.
    ///
    /// This method will call `char_code_at` for each code in this JS string,
    /// returning an iterator of the codes in sequence.
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = u16> + DoubleEndedIterator<Item = u16> + '_ {
        (0..self.length()).map(move |i| self.char_code_at(i) as u16)
    }

    /// If this string consists of a single Unicode code point, then this method
    /// converts it into a Rust `char` without doing any allocations.
    ///
    /// If this JS value is not a valid UTF-8 or consists of more than a single
    /// codepoint, then this returns `None`.
    ///
    /// Note that a single Unicode code point might be represented as more than
    /// one code unit on the JavaScript side. For example, a JavaScript string
    /// `"\uD801\uDC37"` is actually a single Unicode code point U+10437 which
    /// corresponds to a character '𐐷'.
    pub fn as_char(&self) -> Option<char> {
        let len = self.length();

        if len == 0 || len > 2 {
            return None;
        }

        // This will be simplified when definitions are fixed:
        // https://github.com/rustwasm/wasm-bindgen/issues/1362
        let cp = self.code_point_at(0).as_f64().unwrap_throw() as u32;

        let c = std::char::from_u32(cp)?;

        if c.len_utf16() as u32 == len {
            Some(c)
        } else {
            None
        }
    }
}

impl PartialEq<str> for JsString {
    #[allow(clippy::cmp_owned)] // prevent infinite recursion
    fn eq(&self, other: &str) -> bool {
        String::from(self) == other
    }
}

impl<'a> PartialEq<&'a str> for JsString {
    fn eq(&self, other: &&'a str) -> bool {
        <JsString as PartialEq<str>>::eq(self, other)
    }
}

impl PartialEq<String> for JsString {
    fn eq(&self, other: &String) -> bool {
        <JsString as PartialEq<str>>::eq(self, other)
    }
}

impl<'a> PartialEq<&'a String> for JsString {
    fn eq(&self, other: &&'a String) -> bool {
        <JsString as PartialEq<str>>::eq(self, other)
    }
}

impl<'a> From<&'a str> for JsString {
    fn from(s: &'a str) -> Self {
        JsString::unchecked_from_js(JsValue::from_str(s))
    }
}

impl From<String> for JsString {
    fn from(s: String) -> Self {
        From::from(&*s)
    }
}

impl From<char> for JsString {
    #[inline]
    fn from(c: char) -> Self {
        JsString::from_code_point1(c as u32).unwrap_throw()
    }
}

impl<'a> From<&'a JsString> for String {
    fn from(s: &'a JsString) -> Self {
        s.obj.as_string().unwrap_throw()
    }
}

impl From<JsString> for String {
    fn from(s: JsString) -> Self {
        From::from(&s)
    }
}

impl fmt::Debug for JsString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&String::from(self), f)
    }
}

impl fmt::Display for JsString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&String::from(self), f)
    }
}

impl str::FromStr for JsString {
    type Err = convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(JsString::from(s))
    }
}

// Symbol
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(is_type_of = JsValue::is_symbol, typescript_type = "Symbol")]
    #[derive(Clone, Debug)]
    pub type Symbol;

    /// The `Symbol.hasInstance` well-known symbol is used to determine
    /// if a constructor object recognizes an object as its instance.
    /// The `instanceof` operator's behavior can be customized by this symbol.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/hasInstance)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural, js_name = hasInstance)]
    pub fn has_instance() -> Symbol;

    /// The `Symbol.isConcatSpreadable` well-known symbol is used to configure
    /// if an object should be flattened to its array elements when using the
    /// `Array.prototype.concat()` method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/isConcatSpreadable)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural, js_name = isConcatSpreadable)]
    pub fn is_concat_spreadable() -> Symbol;

    /// The `Symbol.asyncIterator` well-known symbol specifies the default AsyncIterator for an object.
    /// If this property is set on an object, it is an async iterable and can be used in a `for await...of` loop.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/asyncIterator)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural, js_name = asyncIterator)]
    pub fn async_iterator() -> Symbol;

    /// The `Symbol.iterator` well-known symbol specifies the default iterator
    /// for an object.  Used by `for...of`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/iterator)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural)]
    pub fn iterator() -> Symbol;

    /// The `Symbol.match` well-known symbol specifies the matching of a regular
    /// expression against a string. This function is called by the
    /// `String.prototype.match()` method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/match)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural, js_name = match)]
    pub fn match_() -> Symbol;

    /// The `Symbol.replace` well-known symbol specifies the method that
    /// replaces matched substrings of a string.  This function is called by the
    /// `String.prototype.replace()` method.
    ///
    /// For more information, see `RegExp.prototype[@@replace]()` and
    /// `String.prototype.replace()`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/replace)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural)]
    pub fn replace() -> Symbol;

    /// The `Symbol.search` well-known symbol specifies the method that returns
    /// the index within a string that matches the regular expression.  This
    /// function is called by the `String.prototype.search()` method.
    ///
    /// For more information, see `RegExp.prototype[@@search]()` and
    /// `String.prototype.search()`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/search)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural)]
    pub fn search() -> Symbol;

    /// The well-known symbol `Symbol.species` specifies a function-valued
    /// property that the constructor function uses to create derived objects.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/species)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural)]
    pub fn species() -> Symbol;

    /// The `Symbol.split` well-known symbol specifies the method that splits a
    /// string at the indices that match a regular expression.  This function is
    /// called by the `String.prototype.split()` method.
    ///
    /// For more information, see `RegExp.prototype[@@split]()` and
    /// `String.prototype.split()`.
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/split)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural)]
    pub fn split() -> Symbol;

    /// The `Symbol.toPrimitive` is a symbol that specifies a function valued
    /// property that is called to convert an object to a corresponding
    /// primitive value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/toPrimitive)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural, js_name = toPrimitive)]
    pub fn to_primitive() -> Symbol;

    /// The `Symbol.toStringTag` well-known symbol is a string valued property
    /// that is used in the creation of the default string description of an
    /// object.  It is accessed internally by the `Object.prototype.toString()`
    /// method.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/toString)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural, js_name = toStringTag)]
    pub fn to_string_tag() -> Symbol;

    /// The `Symbol.for(key)` method searches for existing symbols in a runtime-wide symbol registry with
    /// the given key and returns it if found.
    /// Otherwise a new symbol gets created in the global symbol registry with this key.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/for)
    #[wasm_bindgen(static_method_of = Symbol, js_name = for)]
    pub fn for_(key: &str) -> Symbol;

    /// The `Symbol.keyFor(sym)` method retrieves a shared symbol key from the global symbol registry for the given symbol.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/keyFor)
    #[wasm_bindgen(static_method_of = Symbol, js_name = keyFor)]
    pub fn key_for(sym: &Symbol) -> JsValue;

    /// The `toString()` method returns a string representing the specified Symbol object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/toString)
    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Symbol) -> JsString;

    /// The `Symbol.unscopables` well-known symbol is used to specify an object
    /// value of whose own and inherited property names are excluded from the
    /// with environment bindings of the associated object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/unscopables)
    #[wasm_bindgen(static_method_of = Symbol, getter, structural)]
    pub fn unscopables() -> Symbol;

    /// The `valueOf()` method returns the primitive value of a Symbol object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/valueOf)
    #[wasm_bindgen(method, js_name = valueOf)]
    pub fn value_of(this: &Symbol) -> Symbol;
}

#[allow(non_snake_case)]
pub mod Intl {
    use super::*;

    // Intl
    #[wasm_bindgen]
    extern "C" {
        /// The `Intl.getCanonicalLocales()` method returns an array containing
        /// the canonical locale names. Duplicates will be omitted and elements
        /// will be validated as structurally valid language tags.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/getCanonicalLocales)
        #[wasm_bindgen(js_name = getCanonicalLocales, js_namespace = Intl)]
        pub fn get_canonical_locales(s: &JsValue) -> Array;
    }

    // Intl.Collator
    #[wasm_bindgen]
    extern "C" {
        /// The `Intl.Collator` object is a constructor for collators, objects
        /// that enable language sensitive string comparison.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Collator)
        #[wasm_bindgen(extends = Object, js_namespace = Intl, typescript_type = "Intl.Collator")]
        #[derive(Clone, Debug)]
        pub type Collator;

        /// The `Intl.Collator` object is a constructor for collators, objects
        /// that enable language sensitive string comparison.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Collator)
        #[wasm_bindgen(constructor, js_namespace = Intl)]
        pub fn new(locales: &Array, options: &Object) -> Collator;

        /// The Intl.Collator.prototype.compare property returns a function that
        /// compares two strings according to the sort order of this Collator
        /// object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Collator/compare)
        #[wasm_bindgen(method, getter, js_class = "Intl.Collator")]
        pub fn compare(this: &Collator) -> Function;

        /// The `Intl.Collator.prototype.resolvedOptions()` method returns a new
        /// object with properties reflecting the locale and collation options
        /// computed during initialization of this Collator object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Collator/resolvedOptions)
        #[wasm_bindgen(method, js_namespace = Intl, js_name = resolvedOptions)]
        pub fn resolved_options(this: &Collator) -> Object;

        /// The `Intl.Collator.supportedLocalesOf()` method returns an array
        /// containing those of the provided locales that are supported in
        /// collation without having to fall back to the runtime's default
        /// locale.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Collator/supportedLocalesOf)
        #[wasm_bindgen(static_method_of = Collator, js_namespace = Intl, js_name = supportedLocalesOf)]
        pub fn supported_locales_of(locales: &Array, options: &Object) -> Array;
    }

    impl Default for Collator {
        fn default() -> Self {
            Self::new(
                &JsValue::UNDEFINED.unchecked_into(),
                &JsValue::UNDEFINED.unchecked_into(),
            )
        }
    }

    // Intl.DateTimeFormat
    #[wasm_bindgen]
    extern "C" {
        /// The `Intl.DateTimeFormat` object is a constructor for objects
        /// that enable language-sensitive date and time formatting.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DateTimeFormat)
        #[wasm_bindgen(extends = Object, js_namespace = Intl, typescript_type = "Intl.DateTimeFormat")]
        #[derive(Clone, Debug)]
        pub type DateTimeFormat;

        /// The `Intl.DateTimeFormat` object is a constructor for objects
        /// that enable language-sensitive date and time formatting.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DateTimeFormat)
        #[wasm_bindgen(constructor, js_namespace = Intl)]
        pub fn new(locales: &Array, options: &Object) -> DateTimeFormat;

        /// The Intl.DateTimeFormat.prototype.format property returns a getter function that
        /// formats a date according to the locale and formatting options of this
        /// Intl.DateTimeFormat object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DateTimeFormat/format)
        #[wasm_bindgen(method, getter, js_class = "Intl.DateTimeFormat")]
        pub fn format(this: &DateTimeFormat) -> Function;

        /// The `Intl.DateTimeFormat.prototype.formatToParts()` method allows locale-aware
        /// formatting of strings produced by DateTimeFormat formatters.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DateTimeFormat/formatToParts)
        #[wasm_bindgen(method, js_class = "Intl.DateTimeFormat", js_name = formatToParts)]
        pub fn format_to_parts(this: &DateTimeFormat, date: &Date) -> Array;

        /// The `Intl.DateTimeFormat.prototype.resolvedOptions()` method returns a new
        /// object with properties reflecting the locale and date and time formatting
        /// options computed during initialization of this DateTimeFormat object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DateTimeFormat/resolvedOptions)
        #[wasm_bindgen(method, js_namespace = Intl, js_name = resolvedOptions)]
        pub fn resolved_options(this: &DateTimeFormat) -> Object;

        /// The `Intl.DateTimeFormat.supportedLocalesOf()` method returns an array
        /// containing those of the provided locales that are supported in date
        /// and time formatting without having to fall back to the runtime's default
        /// locale.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DateTimeFormat/supportedLocalesOf)
        #[wasm_bindgen(static_method_of = DateTimeFormat, js_namespace = Intl, js_name = supportedLocalesOf)]
        pub fn supported_locales_of(locales: &Array, options: &Object) -> Array;
    }

    impl Default for DateTimeFormat {
        fn default() -> Self {
            Self::new(
                &JsValue::UNDEFINED.unchecked_into(),
                &JsValue::UNDEFINED.unchecked_into(),
            )
        }
    }

    // Intl.NumberFormat
    #[wasm_bindgen]
    extern "C" {
        /// The `Intl.NumberFormat` object is a constructor for objects
        /// that enable language sensitive number formatting.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/NumberFormat)
        #[wasm_bindgen(extends = Object, js_namespace = Intl, typescript_type = "Intl.NumberFormat")]
        #[derive(Clone, Debug)]
        pub type NumberFormat;

        /// The `Intl.NumberFormat` object is a constructor for objects
        /// that enable language sensitive number formatting.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/NumberFormat)
        #[wasm_bindgen(constructor, js_namespace = Intl)]
        pub fn new(locales: &Array, options: &Object) -> NumberFormat;

        /// The Intl.NumberFormat.prototype.format property returns a getter function that
        /// formats a number according to the locale and formatting options of this
        /// NumberFormat object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/NumberFormat/format)
        #[wasm_bindgen(method, getter, js_class = "Intl.NumberFormat")]
        pub fn format(this: &NumberFormat) -> Function;

        /// The `Intl.Numberformat.prototype.formatToParts()` method allows locale-aware
        /// formatting of strings produced by NumberTimeFormat formatters.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/NumberFormat/formatToParts)
        #[wasm_bindgen(method, js_class = "Intl.NumberFormat", js_name = formatToParts)]
        pub fn format_to_parts(this: &NumberFormat, number: f64) -> Array;

        /// The `Intl.NumberFormat.prototype.resolvedOptions()` method returns a new
        /// object with properties reflecting the locale and number formatting
        /// options computed during initialization of this NumberFormat object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/NumberFormat/resolvedOptions)
        #[wasm_bindgen(method, js_namespace = Intl, js_name = resolvedOptions)]
        pub fn resolved_options(this: &NumberFormat) -> Object;

        /// The `Intl.NumberFormat.supportedLocalesOf()` method returns an array
        /// containing those of the provided locales that are supported in number
        /// formatting without having to fall back to the runtime's default locale.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/NumberFormat/supportedLocalesOf)
        #[wasm_bindgen(static_method_of = NumberFormat, js_namespace = Intl, js_name = supportedLocalesOf)]
        pub fn supported_locales_of(locales: &Array, options: &Object) -> Array;
    }

    impl Default for NumberFormat {
        fn default() -> Self {
            Self::new(
                &JsValue::UNDEFINED.unchecked_into(),
                &JsValue::UNDEFINED.unchecked_into(),
            )
        }
    }

    // Intl.PluralRules
    #[wasm_bindgen]
    extern "C" {
        /// The `Intl.PluralRules` object is a constructor for objects
        /// that enable plural sensitive formatting and plural language rules.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/PluralRules)
        #[wasm_bindgen(extends = Object, js_namespace = Intl, typescript_type = "Intl.PluralRules")]
        #[derive(Clone, Debug)]
        pub type PluralRules;

        /// The `Intl.PluralRules` object is a constructor for objects
        /// that enable plural sensitive formatting and plural language rules.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/PluralRules)
        #[wasm_bindgen(constructor, js_namespace = Intl)]
        pub fn new(locales: &Array, options: &Object) -> PluralRules;

        /// The `Intl.PluralRules.prototype.resolvedOptions()` method returns a new
        /// object with properties reflecting the locale and plural formatting
        /// options computed during initialization of this PluralRules object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/PluralRules/resolvedOptions)
        #[wasm_bindgen(method, js_namespace = Intl, js_name = resolvedOptions)]
        pub fn resolved_options(this: &PluralRules) -> Object;

        /// The `Intl.PluralRules.prototype.select()` method returns a String indicating
        /// which plural rule to use for locale-aware formatting.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/PluralRules/select)
        #[wasm_bindgen(method, js_namespace = Intl)]
        pub fn select(this: &PluralRules, number: f64) -> JsString;

        /// The `Intl.PluralRules.supportedLocalesOf()` method returns an array
        /// containing those of the provided locales that are supported in plural
        /// formatting without having to fall back to the runtime's default locale.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/PluralRules/supportedLocalesOf)
        #[wasm_bindgen(static_method_of = PluralRules, js_namespace = Intl, js_name = supportedLocalesOf)]
        pub fn supported_locales_of(locales: &Array, options: &Object) -> Array;
    }

    impl Default for PluralRules {
        fn default() -> Self {
            Self::new(
                &JsValue::UNDEFINED.unchecked_into(),
                &JsValue::UNDEFINED.unchecked_into(),
            )
        }
    }

    // Intl.RelativeTimeFormat
    #[wasm_bindgen]
    extern "C" {
        /// The `Intl.RelativeTimeFormat` object is a constructor for objects
        /// that enable language-sensitive relative time formatting.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/RelativeTimeFormat)
        #[wasm_bindgen(extends = Object, js_namespace = Intl, typescript_type = "Intl.RelativeTimeFormat")]
        #[derive(Clone, Debug)]
        pub type RelativeTimeFormat;

        /// The `Intl.RelativeTimeFormat` object is a constructor for objects
        /// that enable language-sensitive relative time formatting.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/RelativeTimeFormat)
        #[wasm_bindgen(constructor, js_namespace = Intl)]
        pub fn new(locales: &Array, options: &Object) -> RelativeTimeFormat;

        /// The `Intl.RelativeTimeFormat.prototype.format` method formats a `value` and `unit`
        /// according to the locale and formatting options of this Intl.RelativeTimeFormat object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/RelativeTimeFormat/format)
        #[wasm_bindgen(method, js_class = "Intl.RelativeTimeFormat")]
        pub fn format(this: &RelativeTimeFormat, value: f64, unit: &str) -> JsString;

        /// The `Intl.RelativeTimeFormat.prototype.formatToParts()` method returns an array of
        /// objects representing the relative time format in parts that can be used for custom locale-aware formatting.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/RelativeTimeFormat/formatToParts)
        #[wasm_bindgen(method, js_class = "Intl.RelativeTimeFormat", js_name = formatToParts)]
        pub fn format_to_parts(this: &RelativeTimeFormat, value: f64, unit: &str) -> Array;

        /// The `Intl.RelativeTimeFormat.prototype.resolvedOptions()` method returns a new
        /// object with properties reflecting the locale and relative time formatting
        /// options computed during initialization of this RelativeTimeFormat object.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/RelativeTimeFormat/resolvedOptions)
        #[wasm_bindgen(method, js_namespace = Intl, js_name = resolvedOptions)]
        pub fn resolved_options(this: &RelativeTimeFormat) -> Object;

        /// The `Intl.RelativeTimeFormat.supportedLocalesOf()` method returns an array
        /// containing those of the provided locales that are supported in date and time
        /// formatting without having to fall back to the runtime's default locale.
        ///
        /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RelativeTimeFormat/supportedLocalesOf)
        #[wasm_bindgen(static_method_of = RelativeTimeFormat, js_namespace = Intl, js_name = supportedLocalesOf)]
        pub fn supported_locales_of(locales: &Array, options: &Object) -> Array;
    }

    impl Default for RelativeTimeFormat {
        fn default() -> Self {
            Self::new(
                &JsValue::UNDEFINED.unchecked_into(),
                &JsValue::UNDEFINED.unchecked_into(),
            )
        }
    }
}

// Promise
#[wasm_bindgen]
extern "C" {
    /// The `Promise` object represents the eventual completion (or failure) of
    /// an asynchronous operation, and its resulting value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
    #[must_use]
    #[wasm_bindgen(extends = Object, typescript_type = "Promise<any>")]
    #[derive(Clone, Debug)]
    pub type Promise;

    /// Creates a new `Promise` with the provided executor `cb`
    ///
    /// The `cb` is a function that is passed with the arguments `resolve` and
    /// `reject`. The `cb` function is executed immediately by the `Promise`
    /// implementation, passing `resolve` and `reject` functions (the executor
    /// is called before the `Promise` constructor even returns the created
    /// object). The `resolve` and `reject` functions, when called, resolve or
    /// reject the promise, respectively. The executor normally initiates
    /// some asynchronous work, and then, once that completes, either calls
    /// the `resolve` function to resolve the promise or else rejects it if an
    /// error occurred.
    ///
    /// If an error is thrown in the executor function, the promise is rejected.
    /// The return value of the executor is ignored.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
    #[wasm_bindgen(constructor)]
    pub fn new(cb: &mut dyn FnMut(Function, Function)) -> Promise;

    /// The `Promise.all(iterable)` method returns a single `Promise` that
    /// resolves when all of the promises in the iterable argument have resolved
    /// or when the iterable argument contains no promises. It rejects with the
    /// reason of the first promise that rejects.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/all)
    #[wasm_bindgen(static_method_of = Promise)]
    pub fn all(obj: &JsValue) -> Promise;

    /// The `Promise.allSettled(iterable)` method returns a single `Promise` that
    /// resolves when all of the promises in the iterable argument have either
    /// fulfilled or rejected or when the iterable argument contains no promises.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/allSettled)
    #[wasm_bindgen(static_method_of = Promise, js_name = allSettled)]
    pub fn all_settled(obj: &JsValue) -> Promise;

    /// The `Promise.any(iterable)` method returns a single `Promise` that
    /// resolves when any of the promises in the iterable argument have resolved
    /// or when the iterable argument contains no promises. It rejects with an
    /// `AggregateError` if all promises in the iterable rejected.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/any)
    #[wasm_bindgen(static_method_of = Promise)]
    pub fn any(obj: &JsValue) -> Promise;

    /// The `Promise.race(iterable)` method returns a promise that resolves or
    /// rejects as soon as one of the promises in the iterable resolves or
    /// rejects, with the value or reason from that promise.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/race)
    #[wasm_bindgen(static_method_of = Promise)]
    pub fn race(obj: &JsValue) -> Promise;

    /// The `Promise.reject(reason)` method returns a `Promise` object that is
    /// rejected with the given reason.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/reject)
    #[wasm_bindgen(static_method_of = Promise)]
    pub fn reject(obj: &JsValue) -> Promise;

    /// The `Promise.resolve(value)` method returns a `Promise` object that is
    /// resolved with the given value. If the value is a promise, that promise
    /// is returned; if the value is a thenable (i.e. has a "then" method), the
    /// returned promise will "follow" that thenable, adopting its eventual
    /// state; otherwise the returned promise will be fulfilled with the value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/resolve)
    #[wasm_bindgen(static_method_of = Promise)]
    pub fn resolve(obj: &JsValue) -> Promise;

    /// The `catch()` method returns a `Promise` and deals with rejected cases
    /// only.  It behaves the same as calling `Promise.prototype.then(undefined,
    /// onRejected)` (in fact, calling `obj.catch(onRejected)` internally calls
    /// `obj.then(undefined, onRejected)`).
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/catch)
    #[wasm_bindgen(method)]
    pub fn catch(this: &Promise, cb: &Closure<dyn FnMut(JsValue)>) -> Promise;

    /// The `then()` method returns a `Promise`. It takes up to two arguments:
    /// callback functions for the success and failure cases of the `Promise`.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/then)
    #[wasm_bindgen(method)]
    pub fn then(this: &Promise, cb: &Closure<dyn FnMut(JsValue)>) -> Promise;

    /// Same as `then`, only with both arguments provided.
    #[wasm_bindgen(method, js_name = then)]
    pub fn then2(
        this: &Promise,
        resolve: &Closure<dyn FnMut(JsValue)>,
        reject: &Closure<dyn FnMut(JsValue)>,
    ) -> Promise;

    /// The `finally()` method returns a `Promise`. When the promise is settled,
    /// whether fulfilled or rejected, the specified callback function is
    /// executed. This provides a way for code that must be executed once the
    /// `Promise` has been dealt with to be run whether the promise was
    /// fulfilled successfully or rejected.
    ///
    /// This lets you avoid duplicating code in both the promise's `then()` and
    /// `catch()` handlers.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/finally)
    #[wasm_bindgen(method)]
    pub fn finally(this: &Promise, cb: &Closure<dyn FnMut()>) -> Promise;
}

/// Returns a handle to the global scope object.
///
/// This allows access to the global properties and global names by accessing
/// the `Object` returned.
pub fn global() -> Object {
    thread_local!(static GLOBAL: Object = get_global_object());

    return GLOBAL.with(|g| g.clone());

    fn get_global_object() -> Object {
        // This is a bit wonky, but we're basically using `#[wasm_bindgen]`
        // attributes to synthesize imports so we can access properties of the
        // form:
        //
        // * `globalThis.globalThis`
        // * `self.self`
        // * ... (etc)
        //
        // Accessing the global object is not an easy thing to do, and what we
        // basically want is `globalThis` but we can't rely on that existing
        // everywhere. In the meantime we've got the fallbacks mentioned in:
        //
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/globalThis
        //
        // Note that this is pretty heavy code-size wise but it at least gets
        // the job largely done for now and avoids the `Function` constructor at
        // the end which triggers CSP errors.
        #[wasm_bindgen]
        extern "C" {
            type Global;

            #[wasm_bindgen(getter, catch, static_method_of = Global, js_class = globalThis, js_name = globalThis)]
            fn get_global_this() -> Result<Object, JsValue>;

            #[wasm_bindgen(getter, catch, static_method_of = Global, js_class = self, js_name = self)]
            fn get_self() -> Result<Object, JsValue>;

            #[wasm_bindgen(getter, catch, static_method_of = Global, js_class = window, js_name = window)]
            fn get_window() -> Result<Object, JsValue>;

            #[wasm_bindgen(getter, catch, static_method_of = Global, js_class = global, js_name = global)]
            fn get_global() -> Result<Object, JsValue>;
        }

        // The order is important: in Firefox Extension Content Scripts `globalThis`
        // is a Sandbox (not Window), so `globalThis` must be checked after `window`.
        let static_object = Global::get_self()
            .or_else(|_| Global::get_window())
            .or_else(|_| Global::get_global_this())
            .or_else(|_| Global::get_global());
        if let Ok(obj) = static_object {
            if !obj.is_undefined() {
                return obj;
            }
        }

        // According to StackOverflow you can access the global object via:
        //
        //      const global = Function('return this')();
        //
        // I think that's because the manufactured function isn't in "strict" mode.
        // It also turns out that non-strict functions will ignore `undefined`
        // values for `this` when using the `apply` function.
        //
        // As a result we use the equivalent of this snippet to get a handle to the
        // global object in a sort of roundabout way that should hopefully work in
        // all contexts like ESM, node, browsers, etc.
        let this = Function::new_no_args("return this")
            .call0(&JsValue::undefined())
            .ok();

        // Note that we avoid `unwrap()` on `call0` to avoid code size bloat, we
        // just handle the `Err` case as returning a different object.
        debug_assert!(this.is_some());
        match this {
            Some(this) => this.unchecked_into(),
            None => JsValue::undefined().unchecked_into(),
        }
    }
}

macro_rules! arrays {
    ($(#[doc = $ctor:literal] #[doc = $mdn:literal] $name:ident: $ty:ident,)*) => ($(
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(extends = Object, typescript_type = $name)]
            #[derive(Clone, Debug)]
            pub type $name;

            /// The
            #[doc = $ctor]
            /// constructor creates a new array.
            ///
            /// [MDN documentation](
            #[doc = $mdn]
            /// )
            #[wasm_bindgen(constructor)]
            pub fn new(constructor_arg: &JsValue) -> $name;

            /// An
            #[doc = $ctor]
            /// which creates an array with an internal buffer large
            /// enough for `length` elements.
            ///
            /// [MDN documentation](
            #[doc = $mdn]
            /// )
            #[wasm_bindgen(constructor)]
            pub fn new_with_length(length: u32) -> $name;

            /// An
            #[doc = $ctor]
            /// which creates an array with the given buffer but is a
            /// view starting at `byte_offset`.
            ///
            /// [MDN documentation](
            #[doc = $mdn]
            /// )
            #[wasm_bindgen(constructor)]
            pub fn new_with_byte_offset(buffer: &JsValue, byte_offset: u32) -> $name;

            /// An
            #[doc = $ctor]
            /// which creates an array with the given buffer but is a
            /// view starting at `byte_offset` for `length` elements.
            ///
            /// [MDN documentation](
            #[doc = $mdn]
            /// )
            #[wasm_bindgen(constructor)]
            pub fn new_with_byte_offset_and_length(
                buffer: &JsValue,
                byte_offset: u32,
                length: u32,
            ) -> $name;

            /// The `fill()` method fills all the elements of an array from a start index
            /// to an end index with a static value. The end index is not included.
            ///
            /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray/fill)
            #[wasm_bindgen(method)]
            pub fn fill(this: &$name, value: $ty, start: u32, end: u32) -> $name;

            /// The buffer accessor property represents the `ArrayBuffer` referenced
            /// by a `TypedArray` at construction time.
            #[wasm_bindgen(getter, method)]
            pub fn buffer(this: &$name) -> ArrayBuffer;

            /// The `subarray()` method returns a new `TypedArray` on the same
            /// `ArrayBuffer` store and with the same element types as for this
            /// `TypedArray` object.
            #[wasm_bindgen(method)]
            pub fn subarray(this: &$name, begin: u32, end: u32) -> $name;

            /// The `slice()` method returns a shallow copy of a portion of a typed
            /// array into a new typed array object. This method has the same algorithm
            /// as `Array.prototype.slice()`.
            #[wasm_bindgen(method)]
            pub fn slice(this: &$name, begin: u32, end: u32) -> $name;

            /// The `forEach()` method executes a provided function once per array
            /// element. This method has the same algorithm as
            /// `Array.prototype.forEach()`. `TypedArray` is one of the typed array
            /// types here.
            #[wasm_bindgen(method, js_name = forEach)]
            pub fn for_each(this: &$name, callback: &mut dyn FnMut($ty, u32, $name));

            /// The length accessor property represents the length (in elements) of a
            /// typed array.
            #[wasm_bindgen(method, getter)]
            pub fn length(this: &$name) -> u32;

            /// The byteLength accessor property represents the length (in bytes) of a
            /// typed array.
            #[wasm_bindgen(method, getter, js_name = byteLength)]
            pub fn byte_length(this: &$name) -> u32;

            /// The byteOffset accessor property represents the offset (in bytes) of a
            /// typed array from the start of its `ArrayBuffer`.
            #[wasm_bindgen(method, getter, js_name = byteOffset)]
            pub fn byte_offset(this: &$name) -> u32;

            /// The `set()` method stores multiple values in the typed array, reading
            /// input values from a specified array.
            #[wasm_bindgen(method)]
            pub fn set(this: &$name, src: &JsValue, offset: u32);

            /// Gets the value at `idx`, counting from the end if negative.
            #[wasm_bindgen(method)]
            pub fn at(this: &$name, idx: i32) -> Option<$ty>;

            /// Gets the value at `idx`, equivalent to the javascript `my_var = arr[idx]`.
            #[wasm_bindgen(method, structural, indexing_getter)]
            pub fn get_index(this: &$name, idx: u32) -> $ty;

            /// Sets the value at `idx`, equivalent to the javascript `arr[idx] = value`.
            #[wasm_bindgen(method, structural, indexing_setter)]
            pub fn set_index(this: &$name, idx: u32, value: $ty);
        }

        impl $name {
            /// Creates a JS typed array which is a view into wasm's linear
            /// memory at the slice specified.
            ///
            /// This function returns a new typed array which is a view into
            /// wasm's memory. This view does not copy the underlying data.
            ///
            /// # Unsafety
            ///
            /// Views into WebAssembly memory are only valid so long as the
            /// backing buffer isn't resized in JS. Once this function is called
            /// any future calls to `Box::new` (or malloc of any form) may cause
            /// the returned value here to be invalidated. Use with caution!
            ///
            /// Additionally the returned object can be safely mutated but the
            /// input slice isn't guaranteed to be mutable.
            ///
            /// Finally, the returned object is disconnected from the input
            /// slice's lifetime, so there's no guarantee that the data is read
            /// at the right time.
            pub unsafe fn view(rust: &[$ty]) -> $name {
                let buf = wasm_bindgen::memory();
                let mem = buf.unchecked_ref::<WebAssembly::Memory>();
                $name::new_with_byte_offset_and_length(
                    &mem.buffer(),
                    rust.as_ptr() as u32,
                    rust.len() as u32,
                )
            }

            /// Creates a JS typed array which is a view into wasm's linear
            /// memory at the specified pointer with specified length.
            ///
            /// This function returns a new typed array which is a view into
            /// wasm's memory. This view does not copy the underlying data.
            ///
            /// # Unsafety
            ///
            /// Views into WebAssembly memory are only valid so long as the
            /// backing buffer isn't resized in JS. Once this function is called
            /// any future calls to `Box::new` (or malloc of any form) may cause
            /// the returned value here to be invalidated. Use with caution!
            ///
            /// Additionally the returned object can be safely mutated,
            /// the changes are guaranteed to be reflected in the input array.
            pub unsafe fn view_mut_raw(ptr: *mut $ty, length: usize) -> $name {
                let buf = wasm_bindgen::memory();
                let mem = buf.unchecked_ref::<WebAssembly::Memory>();
                $name::new_with_byte_offset_and_length(
                    &mem.buffer(),
                    ptr as u32,
                    length as u32
                )
            }


            /// Copy the contents of this JS typed array into the destination
            /// Rust pointer.
            ///
            /// This function will efficiently copy the memory from a typed
            /// array into this wasm module's own linear memory, initializing
            /// the memory destination provided.
            ///
            /// # Unsafety
            ///
            /// This function requires `dst` to point to a buffer
            /// large enough to fit this array's contents.
            pub unsafe fn raw_copy_to_ptr(&self, dst: *mut $ty) {
                let buf = wasm_bindgen::memory();
                let mem = buf.unchecked_ref::<WebAssembly::Memory>();
                let all_wasm_memory = $name::new(&mem.buffer());
                let offset = dst as usize / mem::size_of::<$ty>();
                all_wasm_memory.set(self, offset as u32);
            }

            /// Copy the contents of this JS typed array into the destination
            /// Rust slice.
            ///
            /// This function will efficiently copy the memory from a typed
            /// array into this wasm module's own linear memory, initializing
            /// the memory destination provided.
            ///
            /// # Panics
            ///
            /// This function will panic if this typed array's length is
            /// different than the length of the provided `dst` array.
            pub fn copy_to(&self, dst: &mut [$ty]) {
                assert_eq!(self.length() as usize, dst.len());
                unsafe { self.raw_copy_to_ptr(dst.as_mut_ptr()); }
            }

            /// Copy the contents of the source Rust slice into this
            /// JS typed array.
            ///
            /// This function will efficiently copy the memory from within
            /// the wasm module's own linear memory to this typed array.
            ///
            /// # Panics
            ///
            /// This function will panic if this typed array's length is
            /// different than the length of the provided `src` array.
            pub fn copy_from(&self, src: &[$ty]) {
                assert_eq!(self.length() as usize, src.len());
                // This is safe because the `set` function copies from its TypedArray argument
                unsafe { self.set(&$name::view(src), 0) }
            }

            /// Efficiently copies the contents of this JS typed array into a new Vec.
            pub fn to_vec(&self) -> Vec<$ty> {
                let mut output = Vec::with_capacity(self.length() as usize);
                unsafe {
                    self.raw_copy_to_ptr(output.as_mut_ptr());
                    output.set_len(self.length() as usize);
                }
                output
            }
        }

        impl<'a> From<&'a [$ty]> for $name {
            #[inline]
            fn from(slice: &'a [$ty]) -> $name {
                // This is safe because the `new` function makes a copy if its argument is a TypedArray
                unsafe { $name::new(&$name::view(slice)) }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(&JsValue::UNDEFINED.unchecked_into())
            }
        }
    )*);
}

arrays! {
    /// `Int8Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Int8Array
    Int8Array: i8,

    /// `Int16Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Int16Array
    Int16Array: i16,

    /// `Int32Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Int32Array
    Int32Array: i32,

    /// `Uint8Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array
    Uint8Array: u8,

    /// `Uint8ClampedArray()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8ClampedArray
    Uint8ClampedArray: u8,

    /// `Uint16Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint16Array
    Uint16Array: u16,

    /// `Uint32Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint32Array
    Uint32Array: u32,

    /// `Float32Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Float32Array
    Float32Array: f32,

    /// `Float64Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Float64Array
    Float64Array: f64,

    /// `BigInt64Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt64Array
    BigInt64Array: i64,

    /// `BigUint64Array()`
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigUint64Array
    BigUint64Array: u64,
}
