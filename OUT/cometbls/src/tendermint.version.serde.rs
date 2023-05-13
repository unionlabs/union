// @generated
impl serde::Serialize for App {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.protocol != 0 {
            len += 1;
        }
        if !self.software.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.version.App", len)?;
        if self.protocol != 0 {
            struct_ser.serialize_field("protocol", ToString::to_string(&self.protocol).as_str())?;
        }
        if !self.software.is_empty() {
            struct_ser.serialize_field("software", &self.software)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for App {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "protocol",
            "software",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Protocol,
            Software,
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
                            "protocol" => Ok(GeneratedField::Protocol),
                            "software" => Ok(GeneratedField::Software),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = App;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.version.App")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<App, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut protocol__ = None;
                let mut software__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Protocol => {
                            if protocol__.is_some() {
                                return Err(serde::de::Error::duplicate_field("protocol"));
                            }
                            protocol__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Software => {
                            if software__.is_some() {
                                return Err(serde::de::Error::duplicate_field("software"));
                            }
                            software__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(App {
                    protocol: protocol__.unwrap_or_default(),
                    software: software__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.version.App", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Consensus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block != 0 {
            len += 1;
        }
        if self.app != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.version.Consensus", len)?;
        if self.block != 0 {
            struct_ser.serialize_field("block", ToString::to_string(&self.block).as_str())?;
        }
        if self.app != 0 {
            struct_ser.serialize_field("app", ToString::to_string(&self.app).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Consensus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block",
            "app",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Block,
            App,
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
                            "block" => Ok(GeneratedField::Block),
                            "app" => Ok(GeneratedField::App),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Consensus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.version.Consensus")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Consensus, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block__ = None;
                let mut app__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Block => {
                            if block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("block"));
                            }
                            block__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::App => {
                            if app__.is_some() {
                                return Err(serde::de::Error::duplicate_field("app"));
                            }
                            app__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Consensus {
                    block: block__.unwrap_or_default(),
                    app: app__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.version.Consensus", FIELDS, GeneratedVisitor)
    }
}
