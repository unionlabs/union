// @generated
impl serde::Serialize for CompactBitArray {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.extra_bits_stored != 0 {
            len += 1;
        }
        if !self.elems.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.crypto.multisig.v1beta1.CompactBitArray", len)?;
        if self.extra_bits_stored != 0 {
            struct_ser.serialize_field("extraBitsStored", &self.extra_bits_stored)?;
        }
        if !self.elems.is_empty() {
            struct_ser.serialize_field("elems", pbjson::private::base64::encode(&self.elems).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CompactBitArray {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "extra_bits_stored",
            "extraBitsStored",
            "elems",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExtraBitsStored,
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
                            "extraBitsStored" | "extra_bits_stored" => Ok(GeneratedField::ExtraBitsStored),
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
            type Value = CompactBitArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.multisig.v1beta1.CompactBitArray")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CompactBitArray, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut extra_bits_stored__ = None;
                let mut elems__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ExtraBitsStored => {
                            if extra_bits_stored__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extraBitsStored"));
                            }
                            extra_bits_stored__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Elems => {
                            if elems__.is_some() {
                                return Err(serde::de::Error::duplicate_field("elems"));
                            }
                            elems__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CompactBitArray {
                    extra_bits_stored: extra_bits_stored__.unwrap_or_default(),
                    elems: elems__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.multisig.v1beta1.CompactBitArray", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MultiSignature {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.signatures.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.crypto.multisig.v1beta1.MultiSignature", len)?;
        if !self.signatures.is_empty() {
            struct_ser.serialize_field("signatures", &self.signatures.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MultiSignature {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signatures",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Signatures,
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
                            "signatures" => Ok(GeneratedField::Signatures),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MultiSignature;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.multisig.v1beta1.MultiSignature")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MultiSignature, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signatures__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Signatures => {
                            if signatures__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signatures"));
                            }
                            signatures__ = 
                                Some(map.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(MultiSignature {
                    signatures: signatures__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.multisig.v1beta1.MultiSignature", FIELDS, GeneratedVisitor)
    }
}
