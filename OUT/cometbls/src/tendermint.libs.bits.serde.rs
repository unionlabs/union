// @generated
impl serde::Serialize for BitArray {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.bits != 0 {
            len += 1;
        }
        if !self.elems.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.libs.bits.BitArray", len)?;
        if self.bits != 0 {
            struct_ser.serialize_field("bits", ToString::to_string(&self.bits).as_str())?;
        }
        if !self.elems.is_empty() {
            struct_ser.serialize_field("elems", &self.elems.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BitArray {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "bits",
            "elems",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Bits,
            Elems,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "bits" => Ok(GeneratedField::Bits),
                            "elems" => Ok(GeneratedField::Elems),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BitArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.libs.bits.BitArray")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BitArray, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut bits__ = None;
                let mut elems__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Bits => {
                            if bits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bits"));
                            }
                            bits__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Elems => {
                            if elems__.is_some() {
                                return Err(serde::de::Error::duplicate_field("elems"));
                            }
                            elems__ = 
                                Some(map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(BitArray {
                    bits: bits__.unwrap_or_default(),
                    elems: elems__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.libs.bits.BitArray", FIELDS, GeneratedVisitor)
    }
}
