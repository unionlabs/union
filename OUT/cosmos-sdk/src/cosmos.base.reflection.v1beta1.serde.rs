// @generated
impl serde::Serialize for ListAllInterfacesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.reflection.v1beta1.ListAllInterfacesRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListAllInterfacesRequest {
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
            type Value = ListAllInterfacesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v1beta1.ListAllInterfacesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ListAllInterfacesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ListAllInterfacesRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v1beta1.ListAllInterfacesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListAllInterfacesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.interface_names.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v1beta1.ListAllInterfacesResponse", len)?;
        if !self.interface_names.is_empty() {
            struct_ser.serialize_field("interfaceNames", &self.interface_names)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListAllInterfacesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "interface_names",
            "interfaceNames",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InterfaceNames,
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
                            "interfaceNames" | "interface_names" => Ok(GeneratedField::InterfaceNames),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListAllInterfacesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v1beta1.ListAllInterfacesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ListAllInterfacesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut interface_names__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::InterfaceNames => {
                            if interface_names__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interfaceNames"));
                            }
                            interface_names__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ListAllInterfacesResponse {
                    interface_names: interface_names__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v1beta1.ListAllInterfacesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListImplementationsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.interface_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v1beta1.ListImplementationsRequest", len)?;
        if !self.interface_name.is_empty() {
            struct_ser.serialize_field("interfaceName", &self.interface_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListImplementationsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "interface_name",
            "interfaceName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InterfaceName,
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
                            "interfaceName" | "interface_name" => Ok(GeneratedField::InterfaceName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListImplementationsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v1beta1.ListImplementationsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ListImplementationsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut interface_name__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::InterfaceName => {
                            if interface_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interfaceName"));
                            }
                            interface_name__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ListImplementationsRequest {
                    interface_name: interface_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v1beta1.ListImplementationsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListImplementationsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.implementation_message_names.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v1beta1.ListImplementationsResponse", len)?;
        if !self.implementation_message_names.is_empty() {
            struct_ser.serialize_field("implementationMessageNames", &self.implementation_message_names)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListImplementationsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "implementation_message_names",
            "implementationMessageNames",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ImplementationMessageNames,
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
                            "implementationMessageNames" | "implementation_message_names" => Ok(GeneratedField::ImplementationMessageNames),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListImplementationsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v1beta1.ListImplementationsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ListImplementationsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut implementation_message_names__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ImplementationMessageNames => {
                            if implementation_message_names__.is_some() {
                                return Err(serde::de::Error::duplicate_field("implementationMessageNames"));
                            }
                            implementation_message_names__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ListImplementationsResponse {
                    implementation_message_names: implementation_message_names__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v1beta1.ListImplementationsResponse", FIELDS, GeneratedVisitor)
    }
}
