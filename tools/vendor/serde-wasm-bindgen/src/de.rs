use js_sys::{Array, ArrayBuffer, JsString, Number, Object, Symbol, Uint8Array};
use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{self, IntoDeserializer};
use std::convert::TryFrom;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};

use super::{static_str_to_js, Error, ObjectExt, Result};

/// Provides [`de::SeqAccess`] from any JS iterator.
struct SeqAccess {
    iter: js_sys::IntoIter,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        Ok(match self.iter.next().transpose()? {
            Some(value) => Some(seed.deserialize(Deserializer::from(value))?),
            None => None,
        })
    }
}

/// Provides [`serde::de::MapAccess`] from any JS iterator that returns `[key, value]` pairs.
struct MapAccess {
    iter: js_sys::IntoIter,
    next_value: Option<Deserializer>,
}

impl MapAccess {
    const fn new(iter: js_sys::IntoIter) -> Self {
        Self {
            iter,
            next_value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for MapAccess {
    type Error = Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        debug_assert!(self.next_value.is_none());

        Ok(match self.iter.next().transpose()? {
            Some(pair) => {
                let (key, value) = convert_pair(pair);
                self.next_value = Some(value);
                Some(seed.deserialize(key)?)
            }
            None => None,
        })
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        seed.deserialize(self.next_value.take().unwrap_throw())
    }
}

struct ObjectAccess {
    obj: ObjectExt,
    fields: std::slice::Iter<'static, &'static str>,
    next_value: Option<Deserializer>,
}

impl ObjectAccess {
    fn new(obj: ObjectExt, fields: &'static [&'static str]) -> Self {
        Self {
            obj,
            fields: fields.iter(),
            next_value: None,
        }
    }
}

fn str_deserializer(s: &str) -> de::value::StrDeserializer<Error> {
    de::IntoDeserializer::into_deserializer(s)
}

impl<'de> de::MapAccess<'de> for ObjectAccess {
    type Error = Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        debug_assert!(self.next_value.is_none());

        for field in &mut self.fields {
            let js_field = static_str_to_js(field);
            let next_value = self.obj.get_with_ref_key(&js_field);
            // If this value is `undefined`, it might be actually a missing field;
            // double-check with an `in` operator if so.
            let is_missing_field = next_value.is_undefined() && !js_field.js_in(&self.obj);
            if !is_missing_field {
                self.next_value = Some(Deserializer::from(next_value));
                return Ok(Some(seed.deserialize(str_deserializer(field))?));
            }
        }

        Ok(None)
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        seed.deserialize(self.next_value.take().unwrap_throw())
    }
}

/// Provides [`serde::de::EnumAccess`] from given JS values for the `tag` and the `payload`.
struct EnumAccess {
    tag: Deserializer,
    payload: Deserializer,
}

impl<'de> de::EnumAccess<'de> for EnumAccess {
    type Error = Error;
    type Variant = Deserializer;

    fn variant_seed<V: de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant)> {
        Ok((seed.deserialize(self.tag)?, self.payload))
    }
}

/// A newtype that allows using any [`JsValue`] as a [`serde::Deserializer`].
pub struct Deserializer {
    value: JsValue,
}

impl From<JsValue> for Deserializer {
    fn from(value: JsValue) -> Self {
        Self { value }
    }
}

// Ideally this would be implemented for `JsValue` instead, but we can't because
// of the orphan rule.
impl<'de> IntoDeserializer<'de, Error> for Deserializer {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

/// Destructures a JS `[key, value]` pair into a tuple of [`Deserializer`]s.
fn convert_pair(pair: JsValue) -> (Deserializer, Deserializer) {
    let pair = pair.unchecked_into::<Array>();
    (pair.get(0).into(), pair.get(1).into())
}

impl Deserializer {
    /// Casts the internal value into an object, including support for prototype-less objects.
    /// See https://github.com/rustwasm/wasm-bindgen/issues/1366 for why we don't use `dyn_ref`.
    fn as_object_entries(&self) -> Option<Array> {
        if self.value.is_object() {
            Some(Object::entries(self.value.unchecked_ref()))
        } else {
            None
        }
    }

    fn is_nullish(&self) -> bool {
        self.value.loose_eq(&JsValue::NULL)
    }

    fn as_bytes(&self) -> Option<Vec<u8>> {
        let temp;

        let v = if let Some(v) = self.value.dyn_ref::<Uint8Array>() {
            v
        } else if let Some(v) = self.value.dyn_ref::<ArrayBuffer>() {
            temp = Uint8Array::new(v);
            &temp
        } else {
            return None;
        };

        Some(v.to_vec())
    }

    #[cold]
    fn invalid_type_(&self, visitor: &dyn de::Expected) -> Error {
        let string;
        let bytes;

        let unexpected = if self.is_nullish() {
            de::Unexpected::Unit
        } else if let Some(v) = self.value.as_bool() {
            de::Unexpected::Bool(v)
        } else if let Some(v) = self.value.as_f64() {
            de::Unexpected::Float(v)
        } else if let Some(v) = self.value.as_string() {
            string = v;
            de::Unexpected::Str(&string)
        } else if let Some(v) = self.as_bytes() {
            bytes = v;
            de::Unexpected::Bytes(&bytes)
        } else {
            string = format!("{:?}", self.value);
            de::Unexpected::Other(&string)
        };

        de::Error::invalid_type(unexpected, visitor)
    }

    fn invalid_type<'de, V: de::Visitor<'de>>(&self, visitor: V) -> Result<V::Value> {
        Err(self.invalid_type_(&visitor))
    }

    fn as_safe_integer(&self) -> Option<i64> {
        if let Some(v) = self.value.as_f64() {
            if Number::is_safe_integer(&self.value) {
                return Some(v as i64);
            }
        }
        None
    }

    fn deserialize_from_js_number_signed<'de, V: de::Visitor<'de>>(
        &self,
        visitor: V,
    ) -> Result<V::Value> {
        match self.as_safe_integer() {
            Some(v) => visitor.visit_i64(v),
            _ => self.invalid_type(visitor),
        }
    }

    fn deserialize_from_js_number_unsigned<'de, V: de::Visitor<'de>>(
        &self,
        visitor: V,
    ) -> Result<V::Value> {
        match self.as_safe_integer() {
            Some(v) if v >= 0 => visitor.visit_u64(v as _),
            _ => self.invalid_type(visitor),
        }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.is_nullish() {
            // Ideally we would only treat `undefined` as `()` / `None` which would be semantically closer
            // to JS definitions, but, unfortunately, WebIDL generates missing values as `null`
            // and we probably want to support these as well.
            visitor.visit_unit()
        } else if let Some(v) = self.value.as_bool() {
            visitor.visit_bool(v)
        } else if self.value.is_bigint() {
            match i64::try_from(self.value) {
                Ok(v) => visitor.visit_i64(v),
                Err(value) => match u64::try_from(value) {
                    Ok(v) => visitor.visit_u64(v),
                    Err(_) => Err(de::Error::custom("Couldn't deserialize i64 or u64 from a BigInt outside i64::MIN..u64::MAX bounds"))
                }
            }
        } else if let Some(v) = self.value.as_f64() {
            if Number::is_safe_integer(&self.value) {
                visitor.visit_i64(v as i64)
            } else {
                visitor.visit_f64(v)
            }
        } else if let Some(v) = self.value.as_string() {
            visitor.visit_string(v)
        } else if Array::is_array(&self.value) {
            self.deserialize_seq(visitor)
        } else if self.value.is_object() &&
            // The only reason we want to support objects here is because serde uses
            // `deserialize_any` for internally tagged enums
            // (see https://github.com/cloudflare/serde-wasm-bindgen/pull/4#discussion_r352245020).
            //
            // We expect such enums to be represented via plain JS objects, so let's explicitly
            // exclude Sets, Maps and any other iterables. These should be deserialized via concrete
            // `deserialize_*` methods instead of us trying to guess the right target type.
            //
            // Hopefully we can rid of these hacks altogether once
            // https://github.com/serde-rs/serde/issues/1183 is implemented / fixed on serde side.
            !Symbol::iterator().js_in(&self.value)
        {
            self.deserialize_map(visitor)
        } else {
            self.invalid_type(visitor)
        }
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.is_nullish() {
            visitor.visit_unit()
        } else {
            self.invalid_type(visitor)
        }
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(v) = self.value.as_bool() {
            visitor.visit_bool(v)
        } else {
            self.invalid_type(visitor)
        }
    }

    // Serde happily converts `f64` to `f32` (with checks), so we can forward.
    fn deserialize_f32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(v) = self.value.as_f64() {
            visitor.visit_f64(v)
        } else {
            self.invalid_type(visitor)
        }
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(v) = self.value.as_string() {
            visitor.visit_string(v)
        } else {
            self.invalid_type(visitor)
        }
    }

    // Serde happily converts any integer to any integer (with checks), so let's forward all of
    // these to 64-bit methods to save some space in the generated WASM.

    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_from_js_number_signed(visitor)
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_from_js_number_signed(visitor)
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_from_js_number_signed(visitor)
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_from_js_number_unsigned(visitor)
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_from_js_number_unsigned(visitor)
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_from_js_number_unsigned(visitor)
    }

    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.value.is_bigint() {
            match i64::try_from(self.value) {
                Ok(v) => visitor.visit_i64(v),
                Err(_) => Err(de::Error::custom(
                    "Couldn't deserialize i64 from a BigInt outside i64::MIN..i64::MAX bounds",
                )),
            }
        } else {
            self.deserialize_from_js_number_signed(visitor)
        }
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.value.is_bigint() {
            match u64::try_from(self.value) {
                Ok(v) => visitor.visit_u64(v),
                Err(_) => Err(de::Error::custom(
                    "Couldn't deserialize u64 from a BigInt outside u64::MIN..u64::MAX bounds",
                )),
            }
        } else {
            self.deserialize_from_js_number_unsigned(visitor)
        }
    }

    fn deserialize_i128<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.value.is_bigint() {
            match i128::try_from(self.value) {
                Ok(v) => visitor.visit_i128(v),
                Err(_) => Err(de::Error::custom(
                    "Couldn't deserialize i128 from a BigInt outside i128::MIN..i128::MAX bounds",
                )),
            }
        } else {
            self.invalid_type(visitor)
        }
    }

    fn deserialize_u128<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.value.is_bigint() {
            match u128::try_from(self.value) {
                Ok(v) => visitor.visit_u128(v),
                Err(_) => Err(de::Error::custom(
                    "Couldn't deserialize u128 from a BigInt outside u128::MIN..u128::MAX bounds",
                )),
            }
        } else {
            self.invalid_type(visitor)
        }
    }

    /// Converts a JavaScript string to a Rust char.
    ///
    /// By default we don't perform detection of single chars because it's pretty complicated,
    /// but if we get a hint that they're expected, this methods allows to avoid heap allocations
    /// of an intermediate `String` by directly converting numeric codepoints instead.
    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(s) = self.value.dyn_ref::<JsString>() {
            if let Some(c) = s.as_char() {
                return visitor.visit_char(c);
            }
        }
        self.invalid_type(visitor)
    }

    // Serde can deserialize `visit_unit` into `None`, but can't deserialize arbitrary value
    // as `Some`, so we need to provide own simple implementation.
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if !self.is_nullish() {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    /// Simply calls `visit_newtype_struct`.
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    /// Supported inputs:
    ///  - JS iterable (an object with [`[Symbol.iterator]`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/iterator)).
    /// Supported outputs:
    ///  - Any Rust sequence from Serde point of view ([`Vec`], [`HashSet`](std::collections::HashSet), etc.)
    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(arr) = self.value.dyn_ref::<Array>() {
            visitor.visit_seq(SeqDeserializer::new(arr.iter().map(Deserializer::from)))
        } else if let Some(iter) = js_sys::try_iter(&self.value)? {
            visitor.visit_seq(SeqAccess { iter })
        } else {
            self.invalid_type(visitor)
        }
    }

    /// Forwards to [`Self::deserialize_seq`](#method.deserialize_seq).
    fn deserialize_tuple<V: de::Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    /// Forwards to [`Self::deserialize_tuple`](#method.deserialize_tuple).
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_tuple(len, visitor)
    }

    /// Supported inputs:
    ///  - A JS iterable that is expected to return `[key, value]` pairs.
    ///  - A JS object, which will be iterated using [`Object.entries`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/entries).
    /// Supported outputs:
    ///  - A Rust key-value map ([`HashMap`](std::collections::HashMap), [`BTreeMap`](std::collections::BTreeMap), etc.).
    ///  - A typed Rust structure with `#[derive(Deserialize)]`.
    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match js_sys::try_iter(&self.value)? {
            Some(iter) => visitor.visit_map(MapAccess::new(iter)),
            None => match self.as_object_entries() {
                Some(arr) => visitor.visit_map(MapDeserializer::new(arr.iter().map(convert_pair))),
                None => self.invalid_type(visitor),
            },
        }
    }

    /// Supported inputs:
    ///  - A plain JS object.
    /// Supported outputs:
    ///  - A typed Rust structure with `#[derive(Deserialize)]`.
    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let obj = if self.value.is_object() {
            self.value.unchecked_into::<ObjectExt>()
        } else {
            return self.invalid_type(visitor);
        };
        visitor.visit_map(ObjectAccess::new(obj, fields))
    }

    /// Here we try to be compatible with `serde-json`, which means supporting:
    ///  - `"Variant"` - gets converted to a unit variant `MyEnum::Variant`
    ///  - `{ Variant: ...payload... }` - gets converted to a `MyEnum::Variant { ...payload... }`.
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let access = if self.value.is_string() {
            EnumAccess {
                tag: self.value.into(),
                payload: JsValue::UNDEFINED.into(),
            }
        } else if let Some(entries) = self.as_object_entries() {
            if entries.length() != 1 {
                return Err(de::Error::invalid_length(entries.length() as _, &"1"));
            }
            let entry = entries.get(0);
            let (tag, payload) = convert_pair(entry);
            EnumAccess { tag, payload }
        } else {
            return self.invalid_type(visitor);
        };
        visitor.visit_enum(access)
    }

    /// Ignores any value without calling to the JS side even to check its type.
    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    /// We can't take references to JS memory, so forwards to an owned [`Self::deserialize_byte_buf`](#method.deserialize_byte_buf).
    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_byte_buf(visitor)
    }

    /// Serde expects `visit_byte_buf` to be called only in response to an explicit `deserialize_bytes`,
    /// so we provide conversions here.
    ///
    /// Supported inputs:
    ///  - `ArrayBuffer` - converted to an `Uint8Array` view first.
    ///  - `Uint8Array` - copied to a newly created `Vec<u8>` on the Rust side.
    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(bytes) = self.as_bytes() {
            visitor.visit_byte_buf(bytes)
        } else {
            self.invalid_type(visitor)
        }
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<'de> de::VariantAccess<'de> for Deserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V: de::Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        de::Deserializer::deserialize_struct(self, "", fields, visitor)
    }
}
