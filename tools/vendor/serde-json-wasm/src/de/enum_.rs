use serde::de;

use crate::de::{Deserializer, Error, Result};

pub(crate) struct UnitVariantAccess<'a, 'b> {
    de: &'a mut Deserializer<'b>,
}

impl<'a, 'b> UnitVariantAccess<'a, 'b> {
    pub(crate) fn new(de: &'a mut Deserializer<'b>) -> Self {
        UnitVariantAccess { de }
    }
}

impl<'a, 'de> de::EnumAccess<'de> for UnitVariantAccess<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for UnitVariantAccess<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::InvalidType)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::InvalidType)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::InvalidType)
    }
}

pub(crate) struct StructVariantAccess<'a, 'b> {
    de: &'a mut Deserializer<'b>,
}

impl<'a, 'b> StructVariantAccess<'a, 'b> {
    pub fn new(de: &'a mut Deserializer<'b>) -> Self {
        StructVariantAccess { de }
    }
}

impl<'a, 'de> de::EnumAccess<'de> for StructVariantAccess<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        self.de.parse_object_colon()?;
        Ok((val, self))
    }
}

impl<'a, 'de> de::VariantAccess<'de> for StructVariantAccess<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::InvalidType)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.de)?;
        // we remove trailing '}' to be consistent with struct_variant algorithm
        match self
            .de
            .parse_whitespace()
            .ok_or(Error::EofWhileParsingValue)?
        {
            b'}' => {
                self.de.eat_char();
                Ok(value)
            }
            _ => Err(Error::ExpectedSomeValue),
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let value = de::Deserializer::deserialize_tuple(&mut *self.de, len, visitor)?;
        match self
            .de
            .parse_whitespace()
            .ok_or(Error::EofWhileParsingValue)?
        {
            b'}' => {
                self.de.eat_char();
                Ok(value)
            }
            _ => Err(Error::ExpectedSomeValue),
        }
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let value = de::Deserializer::deserialize_struct(&mut *self.de, "", fields, visitor)?;
        match self
            .de
            .parse_whitespace()
            .ok_or(Error::EofWhileParsingValue)?
        {
            b'}' => {
                self.de.eat_char();
                Ok(value)
            }
            _ => Err(Error::ExpectedSomeValue),
        }
    }
}
