// @generated
impl serde::Serialize for GetRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.message_name.is_empty() {
            len += 1;
        }
        if !self.index.is_empty() {
            len += 1;
        }
        if !self.values.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.query.v1alpha1.GetRequest", len)?;
        if !self.message_name.is_empty() {
            struct_ser.serialize_field("messageName", &self.message_name)?;
        }
        if !self.index.is_empty() {
            struct_ser.serialize_field("index", &self.index)?;
        }
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "message_name",
            "messageName",
            "index",
            "values",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MessageName,
            Index,
            Values,
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
                            "messageName" | "message_name" => Ok(GeneratedField::MessageName),
                            "index" => Ok(GeneratedField::Index),
                            "values" => Ok(GeneratedField::Values),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.query.v1alpha1.GetRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut message_name__ = None;
                let mut index__ = None;
                let mut values__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MessageName => {
                            if message_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messageName"));
                            }
                            message_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = Some(map.next_value()?);
                        }
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GetRequest {
                    message_name: message_name__.unwrap_or_default(),
                    index: index__.unwrap_or_default(),
                    values: values__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.query.v1alpha1.GetRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.result.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.query.v1alpha1.GetResponse", len)?;
        if let Some(v) = self.result.as_ref() {
            struct_ser.serialize_field("result", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "result",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Result,
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
                            "result" => Ok(GeneratedField::Result),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.query.v1alpha1.GetResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut result__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Result => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            result__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetResponse {
                    result: result__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.query.v1alpha1.GetResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IndexValue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.query.v1alpha1.IndexValue", len)?;
        if let Some(v) = self.value.as_ref() {
            match v {
                index_value::Value::Uint(v) => {
                    struct_ser.serialize_field("uint", ToString::to_string(&v).as_str())?;
                }
                index_value::Value::Int(v) => {
                    struct_ser.serialize_field("int", ToString::to_string(&v).as_str())?;
                }
                index_value::Value::Str(v) => {
                    struct_ser.serialize_field("str", v)?;
                }
                index_value::Value::Bytes(v) => {
                    struct_ser.serialize_field("bytes", pbjson::private::base64::encode(&v).as_str())?;
                }
                index_value::Value::Enum(v) => {
                    struct_ser.serialize_field("enum", v)?;
                }
                index_value::Value::Bool(v) => {
                    struct_ser.serialize_field("bool", v)?;
                }
                index_value::Value::Timestamp(v) => {
                    struct_ser.serialize_field("timestamp", v)?;
                }
                index_value::Value::Duration(v) => {
                    struct_ser.serialize_field("duration", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IndexValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "uint",
            "int",
            "str",
            "bytes",
            "enum",
            "bool",
            "timestamp",
            "duration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Uint,
            Int,
            Str,
            Bytes,
            Enum,
            Bool,
            Timestamp,
            Duration,
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
                            "uint" => Ok(GeneratedField::Uint),
                            "int" => Ok(GeneratedField::Int),
                            "str" => Ok(GeneratedField::Str),
                            "bytes" => Ok(GeneratedField::Bytes),
                            "enum" => Ok(GeneratedField::Enum),
                            "bool" => Ok(GeneratedField::Bool),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "duration" => Ok(GeneratedField::Duration),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IndexValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.query.v1alpha1.IndexValue")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<IndexValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Uint => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uint"));
                            }
                            value__ = map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| index_value::Value::Uint(x.0));
                        }
                        GeneratedField::Int => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("int"));
                            }
                            value__ = map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| index_value::Value::Int(x.0));
                        }
                        GeneratedField::Str => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("str"));
                            }
                            value__ = map.next_value::<::std::option::Option<_>>()?.map(index_value::Value::Str);
                        }
                        GeneratedField::Bytes => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bytes"));
                            }
                            value__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| index_value::Value::Bytes(x.0));
                        }
                        GeneratedField::Enum => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enum"));
                            }
                            value__ = map.next_value::<::std::option::Option<_>>()?.map(index_value::Value::Enum);
                        }
                        GeneratedField::Bool => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bool"));
                            }
                            value__ = map.next_value::<::std::option::Option<_>>()?.map(index_value::Value::Bool);
                        }
                        GeneratedField::Timestamp => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            value__ = map.next_value::<::std::option::Option<_>>()?.map(index_value::Value::Timestamp)
;
                        }
                        GeneratedField::Duration => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("duration"));
                            }
                            value__ = map.next_value::<::std::option::Option<_>>()?.map(index_value::Value::Duration)
;
                        }
                    }
                }
                Ok(IndexValue {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.query.v1alpha1.IndexValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.message_name.is_empty() {
            len += 1;
        }
        if !self.index.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        if self.query.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.query.v1alpha1.ListRequest", len)?;
        if !self.message_name.is_empty() {
            struct_ser.serialize_field("messageName", &self.message_name)?;
        }
        if !self.index.is_empty() {
            struct_ser.serialize_field("index", &self.index)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if let Some(v) = self.query.as_ref() {
            match v {
                list_request::Query::Prefix(v) => {
                    struct_ser.serialize_field("prefix", v)?;
                }
                list_request::Query::Range(v) => {
                    struct_ser.serialize_field("range", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "message_name",
            "messageName",
            "index",
            "pagination",
            "prefix",
            "range",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MessageName,
            Index,
            Pagination,
            Prefix,
            Range,
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
                            "messageName" | "message_name" => Ok(GeneratedField::MessageName),
                            "index" => Ok(GeneratedField::Index),
                            "pagination" => Ok(GeneratedField::Pagination),
                            "prefix" => Ok(GeneratedField::Prefix),
                            "range" => Ok(GeneratedField::Range),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.query.v1alpha1.ListRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ListRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut message_name__ = None;
                let mut index__ = None;
                let mut pagination__ = None;
                let mut query__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MessageName => {
                            if message_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messageName"));
                            }
                            message_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                        GeneratedField::Prefix => {
                            if query__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prefix"));
                            }
                            query__ = map.next_value::<::std::option::Option<_>>()?.map(list_request::Query::Prefix)
;
                        }
                        GeneratedField::Range => {
                            if query__.is_some() {
                                return Err(serde::de::Error::duplicate_field("range"));
                            }
                            query__ = map.next_value::<::std::option::Option<_>>()?.map(list_request::Query::Range)
;
                        }
                    }
                }
                Ok(ListRequest {
                    message_name: message_name__.unwrap_or_default(),
                    index: index__.unwrap_or_default(),
                    pagination: pagination__,
                    query: query__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.query.v1alpha1.ListRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for list_request::Prefix {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.values.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.query.v1alpha1.ListRequest.Prefix", len)?;
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for list_request::Prefix {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "values",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Values,
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
                            "values" => Ok(GeneratedField::Values),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = list_request::Prefix;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.query.v1alpha1.ListRequest.Prefix")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<list_request::Prefix, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut values__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(list_request::Prefix {
                    values: values__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.query.v1alpha1.ListRequest.Prefix", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for list_request::Range {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.start.is_empty() {
            len += 1;
        }
        if !self.end.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.query.v1alpha1.ListRequest.Range", len)?;
        if !self.start.is_empty() {
            struct_ser.serialize_field("start", &self.start)?;
        }
        if !self.end.is_empty() {
            struct_ser.serialize_field("end", &self.end)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for list_request::Range {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start",
            "end",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Start,
            End,
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
                            "start" => Ok(GeneratedField::Start),
                            "end" => Ok(GeneratedField::End),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = list_request::Range;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.query.v1alpha1.ListRequest.Range")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<list_request::Range, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start__ = None;
                let mut end__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Start => {
                            if start__.is_some() {
                                return Err(serde::de::Error::duplicate_field("start"));
                            }
                            start__ = Some(map.next_value()?);
                        }
                        GeneratedField::End => {
                            if end__.is_some() {
                                return Err(serde::de::Error::duplicate_field("end"));
                            }
                            end__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(list_request::Range {
                    start: start__.unwrap_or_default(),
                    end: end__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.query.v1alpha1.ListRequest.Range", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.results.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.orm.query.v1alpha1.ListResponse", len)?;
        if !self.results.is_empty() {
            struct_ser.serialize_field("results", &self.results)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "results",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Results,
            Pagination,
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
                            "results" => Ok(GeneratedField::Results),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.orm.query.v1alpha1.ListResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ListResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut results__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Results => {
                            if results__.is_some() {
                                return Err(serde::de::Error::duplicate_field("results"));
                            }
                            results__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(ListResponse {
                    results: results__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.orm.query.v1alpha1.ListResponse", FIELDS, GeneratedVisitor)
    }
}
