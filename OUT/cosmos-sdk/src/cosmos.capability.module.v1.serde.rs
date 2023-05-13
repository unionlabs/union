// @generated
impl serde::Serialize for Module {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.seal_keeper {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.capability.module.v1.Module", len)?;
        if self.seal_keeper {
            struct_ser.serialize_field("sealKeeper", &self.seal_keeper)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Module {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "seal_keeper",
            "sealKeeper",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SealKeeper,
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
                            "sealKeeper" | "seal_keeper" => Ok(GeneratedField::SealKeeper),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Module;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.capability.module.v1.Module")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Module, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut seal_keeper__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SealKeeper => {
                            if seal_keeper__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sealKeeper"));
                            }
                            seal_keeper__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Module {
                    seal_keeper: seal_keeper__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.capability.module.v1.Module", FIELDS, GeneratedVisitor)
    }
}
