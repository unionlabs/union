use crate::de::{Deserializer, Error};
use serde::de::{self, Visitor};

pub struct MapAccess<'a, 'b> {
    de: &'a mut Deserializer<'b>,
    first: bool,
}

impl<'a, 'b> MapAccess<'a, 'b> {
    pub(crate) fn new(de: &'a mut Deserializer<'b>) -> Self {
        MapAccess { de, first: true }
    }
}

macro_rules! deserialize_signed_key {
    ($self:ident, $visitor:ident, $ixx:ident, $visit_ixx:ident) => {{
        let de = $self.de;
        match de.parse_whitespace().ok_or(Error::EofWhileParsingValue)? {
            b'"' => de.eat_char(),
            _ => return Err(Error::InvalidType),
        };

        let result = match de.peek() {
            // after rust merged or-patterns feature, these two clause can be merged.
            // error[E0658]: or-patterns syntax is experimental
            Some(b'0'..=b'9') => super::deserialize_signed!(de, $visitor, $ixx, $visit_ixx),
            Some(b'-') => super::deserialize_signed!(de, $visitor, $ixx, $visit_ixx),
            _ => return Err(Error::InvalidType),
        };
        match de.peek() {
            Some(b'"') => {
                de.eat_char();
                result
            }
            _ => Err(Error::InvalidType),
        }
    }};
}

macro_rules! deserialize_unsigned_key {
    ($self:ident, $visitor:ident, $ixx:ident, $visit_ixx:ident) => {{
        let de = $self.de;
        match de.parse_whitespace().ok_or(Error::EofWhileParsingValue)? {
            b'"' => de.eat_char(),
            _ => return Err(Error::InvalidType),
        };

        let result = match de.peek() {
            // after rust merged or-patterns feature, these two clause can be merged.
            // error[E0658]: or-patterns syntax is experimental
            Some(b'0'..=b'9') => super::deserialize_unsigned!(de, $visitor, $ixx, $visit_ixx),
            Some(b'-') => super::deserialize_unsigned!(de, $visitor, $ixx, $visit_ixx),
            _ => return Err(Error::InvalidType),
        };
        match de.peek() {
            Some(b'"') => {
                de.eat_char();
                result
            }
            _ => Err(Error::InvalidType),
        }
    }};
}

impl<'a, 'de> de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let peek = match self
            .de
            .parse_whitespace()
            .ok_or(Error::EofWhileParsingObject)?
        {
            b'}' => return Ok(None),
            b',' if !self.first => {
                self.de.eat_char();
                self.de.parse_whitespace()
            }
            b => {
                if self.first {
                    self.first = false;
                    Some(b)
                } else {
                    return Err(Error::ExpectedObjectCommaOrEnd);
                }
            }
        };

        match peek.ok_or(Error::EofWhileParsingValue)? {
            b'"' => seed.deserialize(MapKey { de: &mut *self.de }).map(Some),
            b'}' => Err(Error::TrailingComma),
            _ => Err(Error::KeyMustBeAString),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.parse_object_colon()?;

        seed.deserialize(&mut *self.de)
    }
}

struct MapKey<'a, 'b> {
    de: &'a mut Deserializer<'b>,
}

impl<'de, 'a> de::Deserializer<'de> for MapKey<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        // Only identifiers are proper json object keys
        self.deserialize_identifier(visitor)
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_signed_key!(self, visitor, i8, visit_i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_signed_key!(self, visitor, i16, visit_i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_signed_key!(self, visitor, i32, visit_i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_signed_key!(self, visitor, i64, visit_i64)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // default implementation includes string unparsing
        self.de.deserialize_i128(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_unsigned_key!(self, visitor, u8, visit_u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_unsigned_key!(self, visitor, u16, visit_u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_unsigned_key!(self, visitor, u32, visit_u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        deserialize_unsigned_key!(self, visitor, u64, visit_u64)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.deserialize_u128(visitor)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.deserialize_str(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.deserialize_string(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Even if we’re ignoring the contents of the map, we still need to
        // deserialize the string here in order to chomp the key’s characters.
        self.deserialize_str(visitor)
    }
}
