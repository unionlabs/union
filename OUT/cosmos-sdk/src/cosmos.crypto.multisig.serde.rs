// @generated
impl serde::Serialize for LegacyAminoPubKey {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.threshold != 0 {
            len += 1;
        }
        if !self.public_keys.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.crypto.multisig.LegacyAminoPubKey", len)?;
        if self.threshold != 0 {
            struct_ser.serialize_field("threshold", &self.threshold)?;
        }
        if !self.public_keys.is_empty() {
            struct_ser.serialize_field("publicKeys", &self.public_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LegacyAminoPubKey {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "threshold",
            "public_keys",
            "publicKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Threshold,
            PublicKeys,
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
                            "threshold" => Ok(GeneratedField::Threshold),
                            "publicKeys" | "public_keys" => Ok(GeneratedField::PublicKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LegacyAminoPubKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.multisig.LegacyAminoPubKey")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<LegacyAminoPubKey, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut threshold__ = None;
                let mut public_keys__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Threshold => {
                            if threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("threshold"));
                            }
                            threshold__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PublicKeys => {
                            if public_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("publicKeys"));
                            }
                            public_keys__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(LegacyAminoPubKey {
                    threshold: threshold__.unwrap_or_default(),
                    public_keys: public_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.multisig.LegacyAminoPubKey", FIELDS, GeneratedVisitor)
    }
}
