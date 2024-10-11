// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::Builder;
use serde::ser;
use serde::ser::*;
use std::fmt::Display;

// This struct internally tracks the nested vectors representing
// nested structs and such.
// TODO: Add an option field names in a map.
/// Flexbuffer Serializer. This should be used to serialize structs.
#[derive(Debug, Default)]
pub struct FlexbufferSerializer {
    builder: Builder,
    nesting: Vec<Option<usize>>,
}
impl FlexbufferSerializer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn view(&self) -> &[u8] {
        self.builder.view()
    }
    pub fn take_buffer(&mut self) -> Vec<u8> {
        self.builder.take_buffer()
    }
    pub fn reset(&mut self) {
        self.builder.reset();
        self.nesting.clear();
    }
    fn finish_if_not_nested(&mut self) -> Result<(), Error> {
        if self.nesting.is_empty() {
            assert_eq!(self.builder.values.len(), 1);
            let root = self.builder.values.pop().unwrap();
            super::store_root(&mut self.builder.buffer, root);
        }
        Ok(())
    }
    fn start_vector(&mut self) {
        let previous_end = if self.nesting.is_empty() {
            None
        } else {
            Some(self.builder.values.len())
        };
        self.nesting.push(previous_end);
    }
    fn start_map(&mut self) {
        let previous_end = if self.nesting.is_empty() {
            None
        } else {
            Some(self.builder.values.len())
        };
        self.nesting.push(previous_end);
    }
    fn end_vector(&mut self) -> Result<(), Error> {
        let previous_end = self.nesting.pop().unwrap();
        self.builder.end_map_or_vector(false, previous_end);
        Ok(())
    }
    fn end_map(&mut self) -> Result<(), Error> {
        let previous_end = self.nesting.pop().unwrap();
        self.builder.end_map_or_vector(true, previous_end);
        Ok(())
    }
}

#[derive(Debug)]
/// Errors that may happen with Serde.
pub enum Error {
    /// Only `str` and `String` can be serialized as keys in serde maps.
    KeyMustBeString,
    Serde(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}
impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Serde(format!("{}", msg))
    }
}
impl ser::SerializeSeq for &mut FlexbufferSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_vector()
    }
}
// This is unlike a flexbuffers map which requires CString like keys.
// Its implemented as alternating keys and values (hopefully).
impl ser::SerializeMap for &mut FlexbufferSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(MapKeySerializer(self))
    }
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_map()
    }
}
impl ser::SerializeTuple for &mut FlexbufferSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_vector()
    }
}
impl ser::SerializeTupleStruct for &mut FlexbufferSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_vector()
    }
}
impl ser::SerializeStruct for &mut FlexbufferSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.builder.push_key(key);
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_map()
    }
}
impl ser::SerializeTupleVariant for &mut FlexbufferSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_vector()?;
        self.end_map()
    }
}
impl ser::SerializeStructVariant for &mut FlexbufferSerializer {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.builder.push_key(key);
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_map()?;
        self.end_map()
    }
    // TODO: skip field?
}

impl<'a> ser::Serializer for &'a mut FlexbufferSerializer {
    type SerializeSeq = &'a mut FlexbufferSerializer;
    type SerializeTuple = &'a mut FlexbufferSerializer;
    type SerializeTupleStruct = &'a mut FlexbufferSerializer;
    type SerializeTupleVariant = &'a mut FlexbufferSerializer;
    type SerializeMap = &'a mut FlexbufferSerializer;
    type SerializeStruct = &'a mut FlexbufferSerializer;
    type SerializeStructVariant = &'a mut FlexbufferSerializer;
    type Ok = ();
    type Error = Error;
    fn is_human_readable(&self) -> bool {
        cfg!(serialize_human_readable)
    }
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v as u8);
        self.finish_if_not_nested()
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.builder.push(v);
        self.finish_if_not_nested()
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.builder.push(crate::Blob(v));
        self.finish_if_not_nested()
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.builder.push(());
        self.finish_if_not_nested()
    }
    fn serialize_some<T: ?Sized>(self, t: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        t.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.builder.push(());
        self.finish_if_not_nested()
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.builder.push(());
        self.finish_if_not_nested()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.builder.push(variant);
        self.finish_if_not_nested()
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.start_map();
        self.builder.push_key(variant);
        value.serialize(&mut *self)?;
        self.end_map()
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.start_vector();
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.start_vector();
        Ok(self)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.start_map();
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.start_map();
        self.builder.push_key(variant);
        self.start_vector();
        Ok(self)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.start_map();
        Ok(self)
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.start_map();
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.start_map();
        self.builder.push_key(variant);
        self.start_map();
        Ok(self)
    }
}

fn key_must_be_a_string<T>() -> Result<T, Error> {
    Err(Error::KeyMustBeString)
}
struct MapKeySerializer<'a>(&'a mut FlexbufferSerializer);
impl<'a> Serializer for MapKeySerializer<'a> {
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_str(self, value: &str) -> Result<(), Error> {
        self.0.builder.push_key(value);
        Ok(())
    }
    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.0.builder.push_key(variant);
        Ok(())
    }
    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _value: bool) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_i8(self, _value: i8) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_i16(self, _value: i16) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_i32(self, _value: i32) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_i64(self, _value: i64) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_u8(self, _value: u8) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_u16(self, _value: u16) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_u32(self, _value: u32) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_u64(self, _value: u64) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_f32(self, _value: f32) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_f64(self, _value: f64) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_char(self, _value: char) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_unit(self) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), Error>
    where
        T: Serialize,
    {
        key_must_be_a_string()
    }
    fn serialize_none(self) -> Result<(), Error> {
        key_must_be_a_string()
    }
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        key_must_be_a_string()
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        key_must_be_a_string()
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        key_must_be_a_string()
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        key_must_be_a_string()
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        key_must_be_a_string()
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        key_must_be_a_string()
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        key_must_be_a_string()
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        key_must_be_a_string()
    }
}
