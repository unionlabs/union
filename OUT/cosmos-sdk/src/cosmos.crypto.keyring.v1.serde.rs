// @generated
impl serde::Serialize for Record {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.pub_key.is_some() {
            len += 1;
        }
        if self.item.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.crypto.keyring.v1.Record", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.pub_key.as_ref() {
            struct_ser.serialize_field("pubKey", v)?;
        }
        if let Some(v) = self.item.as_ref() {
            match v {
                record::Item::Local(v) => {
                    struct_ser.serialize_field("local", v)?;
                }
                record::Item::Ledger(v) => {
                    struct_ser.serialize_field("ledger", v)?;
                }
                record::Item::Multi(v) => {
                    struct_ser.serialize_field("multi", v)?;
                }
                record::Item::Offline(v) => {
                    struct_ser.serialize_field("offline", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Record {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "pub_key",
            "pubKey",
            "local",
            "ledger",
            "multi",
            "offline",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            PubKey,
            Local,
            Ledger,
            Multi,
            Offline,
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
                            "name" => Ok(GeneratedField::Name),
                            "pubKey" | "pub_key" => Ok(GeneratedField::PubKey),
                            "local" => Ok(GeneratedField::Local),
                            "ledger" => Ok(GeneratedField::Ledger),
                            "multi" => Ok(GeneratedField::Multi),
                            "offline" => Ok(GeneratedField::Offline),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Record;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.keyring.v1.Record")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Record, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut pub_key__ = None;
                let mut item__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::PubKey => {
                            if pub_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubKey"));
                            }
                            pub_key__ = map.next_value()?;
                        }
                        GeneratedField::Local => {
                            if item__.is_some() {
                                return Err(serde::de::Error::duplicate_field("local"));
                            }
                            item__ = map.next_value::<::std::option::Option<_>>()?.map(record::Item::Local)
;
                        }
                        GeneratedField::Ledger => {
                            if item__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ledger"));
                            }
                            item__ = map.next_value::<::std::option::Option<_>>()?.map(record::Item::Ledger)
;
                        }
                        GeneratedField::Multi => {
                            if item__.is_some() {
                                return Err(serde::de::Error::duplicate_field("multi"));
                            }
                            item__ = map.next_value::<::std::option::Option<_>>()?.map(record::Item::Multi)
;
                        }
                        GeneratedField::Offline => {
                            if item__.is_some() {
                                return Err(serde::de::Error::duplicate_field("offline"));
                            }
                            item__ = map.next_value::<::std::option::Option<_>>()?.map(record::Item::Offline)
;
                        }
                    }
                }
                Ok(Record {
                    name: name__.unwrap_or_default(),
                    pub_key: pub_key__,
                    item: item__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.keyring.v1.Record", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for record::Ledger {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.path.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.crypto.keyring.v1.Record.Ledger", len)?;
        if let Some(v) = self.path.as_ref() {
            struct_ser.serialize_field("path", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for record::Ledger {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
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
                            "path" => Ok(GeneratedField::Path),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = record::Ledger;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.keyring.v1.Record.Ledger")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<record::Ledger, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = map.next_value()?;
                        }
                    }
                }
                Ok(record::Ledger {
                    path: path__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.keyring.v1.Record.Ledger", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for record::Local {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.priv_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.crypto.keyring.v1.Record.Local", len)?;
        if let Some(v) = self.priv_key.as_ref() {
            struct_ser.serialize_field("privKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for record::Local {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "priv_key",
            "privKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivKey,
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
                            "privKey" | "priv_key" => Ok(GeneratedField::PrivKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = record::Local;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.keyring.v1.Record.Local")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<record::Local, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut priv_key__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PrivKey => {
                            if priv_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privKey"));
                            }
                            priv_key__ = map.next_value()?;
                        }
                    }
                }
                Ok(record::Local {
                    priv_key: priv_key__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.keyring.v1.Record.Local", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for record::Multi {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.crypto.keyring.v1.Record.Multi", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for record::Multi {
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
            type Value = record::Multi;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.keyring.v1.Record.Multi")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<record::Multi, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(record::Multi {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.keyring.v1.Record.Multi", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for record::Offline {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.crypto.keyring.v1.Record.Offline", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for record::Offline {
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
            type Value = record::Offline;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.keyring.v1.Record.Offline")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<record::Offline, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(record::Offline {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.keyring.v1.Record.Offline", FIELDS, GeneratedVisitor)
    }
}
