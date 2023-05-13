// @generated
impl serde::Serialize for PrimaryKeyDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fields.is_empty() {
            len += 1;
        }
        if self.auto_increment {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.v1.PrimaryKeyDescriptor", len)?;
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        if self.auto_increment {
            struct_ser.serialize_field("autoIncrement", &self.auto_increment)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PrimaryKeyDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fields",
            "auto_increment",
            "autoIncrement",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fields,
            AutoIncrement,
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
                            "fields" => Ok(GeneratedField::Fields),
                            "autoIncrement" | "auto_increment" => Ok(GeneratedField::AutoIncrement),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PrimaryKeyDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.v1.PrimaryKeyDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PrimaryKeyDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fields__ = None;
                let mut auto_increment__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fields => {
                            if fields__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields__ = Some(map.next_value()?);
                        }
                        GeneratedField::AutoIncrement => {
                            if auto_increment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("autoIncrement"));
                            }
                            auto_increment__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PrimaryKeyDescriptor {
                    fields: fields__.unwrap_or_default(),
                    auto_increment: auto_increment__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.v1.PrimaryKeyDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SecondaryIndexDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fields.is_empty() {
            len += 1;
        }
        if self.id != 0 {
            len += 1;
        }
        if self.unique {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.v1.SecondaryIndexDescriptor", len)?;
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if self.unique {
            struct_ser.serialize_field("unique", &self.unique)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SecondaryIndexDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fields",
            "id",
            "unique",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fields,
            Id,
            Unique,
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
                            "fields" => Ok(GeneratedField::Fields),
                            "id" => Ok(GeneratedField::Id),
                            "unique" => Ok(GeneratedField::Unique),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SecondaryIndexDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.v1.SecondaryIndexDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SecondaryIndexDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fields__ = None;
                let mut id__ = None;
                let mut unique__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fields => {
                            if fields__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields__ = Some(map.next_value()?);
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Unique => {
                            if unique__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unique"));
                            }
                            unique__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SecondaryIndexDescriptor {
                    fields: fields__.unwrap_or_default(),
                    id: id__.unwrap_or_default(),
                    unique: unique__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.v1.SecondaryIndexDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SingletonDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.v1.SingletonDescriptor", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SingletonDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
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
                            "id" => Ok(GeneratedField::Id),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SingletonDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.v1.SingletonDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SingletonDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SingletonDescriptor {
                    id: id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.v1.SingletonDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TableDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.primary_key.is_some() {
            len += 1;
        }
        if !self.index.is_empty() {
            len += 1;
        }
        if self.id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.v1.TableDescriptor", len)?;
        if let Some(v) = self.primary_key.as_ref() {
            struct_ser.serialize_field("primaryKey", v)?;
        }
        if !self.index.is_empty() {
            struct_ser.serialize_field("index", &self.index)?;
        }
        if self.id != 0 {
            struct_ser.serialize_field("id", &self.id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TableDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "primary_key",
            "primaryKey",
            "index",
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrimaryKey,
            Index,
            Id,
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
                            "primaryKey" | "primary_key" => Ok(GeneratedField::PrimaryKey),
                            "index" => Ok(GeneratedField::Index),
                            "id" => Ok(GeneratedField::Id),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TableDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.v1.TableDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TableDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut primary_key__ = None;
                let mut index__ = None;
                let mut id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PrimaryKey => {
                            if primary_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("primaryKey"));
                            }
                            primary_key__ = map.next_value()?;
                        }
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = Some(map.next_value()?);
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TableDescriptor {
                    primary_key: primary_key__,
                    index: index__.unwrap_or_default(),
                    id: id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.v1.TableDescriptor", FIELDS, GeneratedVisitor)
    }
}
