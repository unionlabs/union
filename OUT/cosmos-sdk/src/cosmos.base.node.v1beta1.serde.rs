// @generated
impl serde::Serialize for ConfigRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.node.v1beta1.ConfigRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConfigRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConfigRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.node.v1beta1.ConfigRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConfigRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ConfigRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.node.v1beta1.ConfigRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConfigResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.minimum_gas_price.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.node.v1beta1.ConfigResponse", len)?;
        if !self.minimum_gas_price.is_empty() {
            struct_ser.serialize_field("minimumGasPrice", &self.minimum_gas_price)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConfigResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "minimum_gas_price",
            "minimumGasPrice",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MinimumGasPrice,
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
                            "minimumGasPrice" | "minimum_gas_price" => Ok(GeneratedField::MinimumGasPrice),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConfigResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.node.v1beta1.ConfigResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConfigResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut minimum_gas_price__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MinimumGasPrice => {
                            if minimum_gas_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minimumGasPrice"));
                            }
                            minimum_gas_price__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ConfigResponse {
                    minimum_gas_price: minimum_gas_price__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.node.v1beta1.ConfigResponse", FIELDS, GeneratedVisitor)
    }
}
