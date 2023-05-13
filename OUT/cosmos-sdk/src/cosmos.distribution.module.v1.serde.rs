// @generated
impl serde::Serialize for Module {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fee_collector_name.is_empty() {
            len += 1;
        }
        if !self.authority.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.module.v1.Module", len)?;
        if !self.fee_collector_name.is_empty() {
            struct_ser.serialize_field("feeCollectorName", &self.fee_collector_name)?;
        }
        if !self.authority.is_empty() {
            struct_ser.serialize_field("authority", &self.authority)?;
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
            "fee_collector_name",
            "feeCollectorName",
            "authority",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FeeCollectorName,
            Authority,
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
                            "feeCollectorName" | "fee_collector_name" => Ok(GeneratedField::FeeCollectorName),
                            "authority" => Ok(GeneratedField::Authority),
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
                formatter.write_str("struct cosmos.distribution.module.v1.Module")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Module, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee_collector_name__ = None;
                let mut authority__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FeeCollectorName => {
                            if fee_collector_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeCollectorName"));
                            }
                            fee_collector_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Authority => {
                            if authority__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authority"));
                            }
                            authority__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Module {
                    fee_collector_name: fee_collector_name__.unwrap_or_default(),
                    authority: authority__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.module.v1.Module", FIELDS, GeneratedVisitor)
    }
}
