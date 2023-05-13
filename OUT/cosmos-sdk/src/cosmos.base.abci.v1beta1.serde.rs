// @generated
impl serde::Serialize for AbciMessageLog {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.msg_index != 0 {
            len += 1;
        }
        if !self.log.is_empty() {
            len += 1;
        }
        if !self.events.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.ABCIMessageLog", len)?;
        if self.msg_index != 0 {
            struct_ser.serialize_field("msgIndex", &self.msg_index)?;
        }
        if !self.log.is_empty() {
            struct_ser.serialize_field("log", &self.log)?;
        }
        if !self.events.is_empty() {
            struct_ser.serialize_field("events", &self.events)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AbciMessageLog {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "msg_index",
            "msgIndex",
            "log",
            "events",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MsgIndex,
            Log,
            Events,
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
                            "msgIndex" | "msg_index" => Ok(GeneratedField::MsgIndex),
                            "log" => Ok(GeneratedField::Log),
                            "events" => Ok(GeneratedField::Events),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AbciMessageLog;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.ABCIMessageLog")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AbciMessageLog, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut msg_index__ = None;
                let mut log__ = None;
                let mut events__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MsgIndex => {
                            if msg_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("msgIndex"));
                            }
                            msg_index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Log => {
                            if log__.is_some() {
                                return Err(serde::de::Error::duplicate_field("log"));
                            }
                            log__ = Some(map.next_value()?);
                        }
                        GeneratedField::Events => {
                            if events__.is_some() {
                                return Err(serde::de::Error::duplicate_field("events"));
                            }
                            events__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AbciMessageLog {
                    msg_index: msg_index__.unwrap_or_default(),
                    log: log__.unwrap_or_default(),
                    events: events__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.ABCIMessageLog", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Attribute {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.key.is_empty() {
            len += 1;
        }
        if !self.value.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.Attribute", len)?;
        if !self.key.is_empty() {
            struct_ser.serialize_field("key", &self.key)?;
        }
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", &self.value)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Attribute {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "key",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Key,
            Value,
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
                            "key" => Ok(GeneratedField::Key),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Attribute;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.Attribute")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Attribute, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut key__ = None;
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = Some(map.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Attribute {
                    key: key__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.Attribute", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GasInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.gas_wanted != 0 {
            len += 1;
        }
        if self.gas_used != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.GasInfo", len)?;
        if self.gas_wanted != 0 {
            struct_ser.serialize_field("gasWanted", ToString::to_string(&self.gas_wanted).as_str())?;
        }
        if self.gas_used != 0 {
            struct_ser.serialize_field("gasUsed", ToString::to_string(&self.gas_used).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GasInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "gas_wanted",
            "gasWanted",
            "gas_used",
            "gasUsed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GasWanted,
            GasUsed,
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
                            "gasWanted" | "gas_wanted" => Ok(GeneratedField::GasWanted),
                            "gasUsed" | "gas_used" => Ok(GeneratedField::GasUsed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GasInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.GasInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GasInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut gas_wanted__ = None;
                let mut gas_used__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GasWanted => {
                            if gas_wanted__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasWanted"));
                            }
                            gas_wanted__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GasUsed => {
                            if gas_used__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasUsed"));
                            }
                            gas_used__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GasInfo {
                    gas_wanted: gas_wanted__.unwrap_or_default(),
                    gas_used: gas_used__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.GasInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.msg_type.is_empty() {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.MsgData", len)?;
        if !self.msg_type.is_empty() {
            struct_ser.serialize_field("msgType", &self.msg_type)?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "msg_type",
            "msgType",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MsgType,
            Data,
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
                            "msgType" | "msg_type" => Ok(GeneratedField::MsgType),
                            "data" => Ok(GeneratedField::Data),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.MsgData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut msg_type__ = None;
                let mut data__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MsgType => {
                            if msg_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("msgType"));
                            }
                            msg_type__ = Some(map.next_value()?);
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgData {
                    msg_type: msg_type__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.MsgData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Result {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data.is_empty() {
            len += 1;
        }
        if !self.log.is_empty() {
            len += 1;
        }
        if !self.events.is_empty() {
            len += 1;
        }
        if !self.msg_responses.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.Result", len)?;
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        if !self.log.is_empty() {
            struct_ser.serialize_field("log", &self.log)?;
        }
        if !self.events.is_empty() {
            struct_ser.serialize_field("events", &self.events)?;
        }
        if !self.msg_responses.is_empty() {
            struct_ser.serialize_field("msgResponses", &self.msg_responses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Result {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
            "log",
            "events",
            "msg_responses",
            "msgResponses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
            Log,
            Events,
            MsgResponses,
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
                            "data" => Ok(GeneratedField::Data),
                            "log" => Ok(GeneratedField::Log),
                            "events" => Ok(GeneratedField::Events),
                            "msgResponses" | "msg_responses" => Ok(GeneratedField::MsgResponses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Result;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.Result")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Result, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                let mut log__ = None;
                let mut events__ = None;
                let mut msg_responses__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Log => {
                            if log__.is_some() {
                                return Err(serde::de::Error::duplicate_field("log"));
                            }
                            log__ = Some(map.next_value()?);
                        }
                        GeneratedField::Events => {
                            if events__.is_some() {
                                return Err(serde::de::Error::duplicate_field("events"));
                            }
                            events__ = Some(map.next_value()?);
                        }
                        GeneratedField::MsgResponses => {
                            if msg_responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("msgResponses"));
                            }
                            msg_responses__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Result {
                    data: data__.unwrap_or_default(),
                    log: log__.unwrap_or_default(),
                    events: events__.unwrap_or_default(),
                    msg_responses: msg_responses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.Result", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SearchTxsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.total_count != 0 {
            len += 1;
        }
        if self.count != 0 {
            len += 1;
        }
        if self.page_number != 0 {
            len += 1;
        }
        if self.page_total != 0 {
            len += 1;
        }
        if self.limit != 0 {
            len += 1;
        }
        if !self.txs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.SearchTxsResult", len)?;
        if self.total_count != 0 {
            struct_ser.serialize_field("totalCount", ToString::to_string(&self.total_count).as_str())?;
        }
        if self.count != 0 {
            struct_ser.serialize_field("count", ToString::to_string(&self.count).as_str())?;
        }
        if self.page_number != 0 {
            struct_ser.serialize_field("pageNumber", ToString::to_string(&self.page_number).as_str())?;
        }
        if self.page_total != 0 {
            struct_ser.serialize_field("pageTotal", ToString::to_string(&self.page_total).as_str())?;
        }
        if self.limit != 0 {
            struct_ser.serialize_field("limit", ToString::to_string(&self.limit).as_str())?;
        }
        if !self.txs.is_empty() {
            struct_ser.serialize_field("txs", &self.txs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SearchTxsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "total_count",
            "totalCount",
            "count",
            "page_number",
            "pageNumber",
            "page_total",
            "pageTotal",
            "limit",
            "txs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TotalCount,
            Count,
            PageNumber,
            PageTotal,
            Limit,
            Txs,
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
                            "totalCount" | "total_count" => Ok(GeneratedField::TotalCount),
                            "count" => Ok(GeneratedField::Count),
                            "pageNumber" | "page_number" => Ok(GeneratedField::PageNumber),
                            "pageTotal" | "page_total" => Ok(GeneratedField::PageTotal),
                            "limit" => Ok(GeneratedField::Limit),
                            "txs" => Ok(GeneratedField::Txs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SearchTxsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.SearchTxsResult")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SearchTxsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut total_count__ = None;
                let mut count__ = None;
                let mut page_number__ = None;
                let mut page_total__ = None;
                let mut limit__ = None;
                let mut txs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TotalCount => {
                            if total_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalCount"));
                            }
                            total_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Count => {
                            if count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("count"));
                            }
                            count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PageNumber => {
                            if page_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageNumber"));
                            }
                            page_number__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PageTotal => {
                            if page_total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageTotal"));
                            }
                            page_total__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Limit => {
                            if limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("limit"));
                            }
                            limit__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Txs => {
                            if txs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txs"));
                            }
                            txs__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SearchTxsResult {
                    total_count: total_count__.unwrap_or_default(),
                    count: count__.unwrap_or_default(),
                    page_number: page_number__.unwrap_or_default(),
                    page_total: page_total__.unwrap_or_default(),
                    limit: limit__.unwrap_or_default(),
                    txs: txs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.SearchTxsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SimulationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.gas_info.is_some() {
            len += 1;
        }
        if self.result.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.SimulationResponse", len)?;
        if let Some(v) = self.gas_info.as_ref() {
            struct_ser.serialize_field("gasInfo", v)?;
        }
        if let Some(v) = self.result.as_ref() {
            struct_ser.serialize_field("result", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SimulationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "gas_info",
            "gasInfo",
            "result",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GasInfo,
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
                            "gasInfo" | "gas_info" => Ok(GeneratedField::GasInfo),
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
            type Value = SimulationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.SimulationResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SimulationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut gas_info__ = None;
                let mut result__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GasInfo => {
                            if gas_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasInfo"));
                            }
                            gas_info__ = map.next_value()?;
                        }
                        GeneratedField::Result => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            result__ = map.next_value()?;
                        }
                    }
                }
                Ok(SimulationResponse {
                    gas_info: gas_info__,
                    result: result__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.SimulationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StringEvent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.r#type.is_empty() {
            len += 1;
        }
        if !self.attributes.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.StringEvent", len)?;
        if !self.r#type.is_empty() {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if !self.attributes.is_empty() {
            struct_ser.serialize_field("attributes", &self.attributes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StringEvent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "attributes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Attributes,
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
                            "type" => Ok(GeneratedField::Type),
                            "attributes" => Ok(GeneratedField::Attributes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StringEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.StringEvent")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StringEvent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut attributes__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value()?);
                        }
                        GeneratedField::Attributes => {
                            if attributes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attributes"));
                            }
                            attributes__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(StringEvent {
                    r#type: r#type__.unwrap_or_default(),
                    attributes: attributes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.StringEvent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TxMsgData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data.is_empty() {
            len += 1;
        }
        if !self.msg_responses.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.TxMsgData", len)?;
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", &self.data)?;
        }
        if !self.msg_responses.is_empty() {
            struct_ser.serialize_field("msgResponses", &self.msg_responses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TxMsgData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
            "msg_responses",
            "msgResponses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
            MsgResponses,
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
                            "data" => Ok(GeneratedField::Data),
                            "msgResponses" | "msg_responses" => Ok(GeneratedField::MsgResponses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TxMsgData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.TxMsgData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TxMsgData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                let mut msg_responses__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = Some(map.next_value()?);
                        }
                        GeneratedField::MsgResponses => {
                            if msg_responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("msgResponses"));
                            }
                            msg_responses__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TxMsgData {
                    data: data__.unwrap_or_default(),
                    msg_responses: msg_responses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.TxMsgData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TxResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.height != 0 {
            len += 1;
        }
        if !self.txhash.is_empty() {
            len += 1;
        }
        if !self.codespace.is_empty() {
            len += 1;
        }
        if self.code != 0 {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        if !self.raw_log.is_empty() {
            len += 1;
        }
        if !self.logs.is_empty() {
            len += 1;
        }
        if !self.info.is_empty() {
            len += 1;
        }
        if self.gas_wanted != 0 {
            len += 1;
        }
        if self.gas_used != 0 {
            len += 1;
        }
        if self.tx.is_some() {
            len += 1;
        }
        if !self.timestamp.is_empty() {
            len += 1;
        }
        if !self.events.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.abci.v1beta1.TxResponse", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if !self.txhash.is_empty() {
            struct_ser.serialize_field("txhash", &self.txhash)?;
        }
        if !self.codespace.is_empty() {
            struct_ser.serialize_field("codespace", &self.codespace)?;
        }
        if self.code != 0 {
            struct_ser.serialize_field("code", &self.code)?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", &self.data)?;
        }
        if !self.raw_log.is_empty() {
            struct_ser.serialize_field("rawLog", &self.raw_log)?;
        }
        if !self.logs.is_empty() {
            struct_ser.serialize_field("logs", &self.logs)?;
        }
        if !self.info.is_empty() {
            struct_ser.serialize_field("info", &self.info)?;
        }
        if self.gas_wanted != 0 {
            struct_ser.serialize_field("gasWanted", ToString::to_string(&self.gas_wanted).as_str())?;
        }
        if self.gas_used != 0 {
            struct_ser.serialize_field("gasUsed", ToString::to_string(&self.gas_used).as_str())?;
        }
        if let Some(v) = self.tx.as_ref() {
            struct_ser.serialize_field("tx", v)?;
        }
        if !self.timestamp.is_empty() {
            struct_ser.serialize_field("timestamp", &self.timestamp)?;
        }
        if !self.events.is_empty() {
            struct_ser.serialize_field("events", &self.events)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TxResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "txhash",
            "codespace",
            "code",
            "data",
            "raw_log",
            "rawLog",
            "logs",
            "info",
            "gas_wanted",
            "gasWanted",
            "gas_used",
            "gasUsed",
            "tx",
            "timestamp",
            "events",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Txhash,
            Codespace,
            Code,
            Data,
            RawLog,
            Logs,
            Info,
            GasWanted,
            GasUsed,
            Tx,
            Timestamp,
            Events,
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
                            "height" => Ok(GeneratedField::Height),
                            "txhash" => Ok(GeneratedField::Txhash),
                            "codespace" => Ok(GeneratedField::Codespace),
                            "code" => Ok(GeneratedField::Code),
                            "data" => Ok(GeneratedField::Data),
                            "rawLog" | "raw_log" => Ok(GeneratedField::RawLog),
                            "logs" => Ok(GeneratedField::Logs),
                            "info" => Ok(GeneratedField::Info),
                            "gasWanted" | "gas_wanted" => Ok(GeneratedField::GasWanted),
                            "gasUsed" | "gas_used" => Ok(GeneratedField::GasUsed),
                            "tx" => Ok(GeneratedField::Tx),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "events" => Ok(GeneratedField::Events),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TxResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.abci.v1beta1.TxResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TxResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut txhash__ = None;
                let mut codespace__ = None;
                let mut code__ = None;
                let mut data__ = None;
                let mut raw_log__ = None;
                let mut logs__ = None;
                let mut info__ = None;
                let mut gas_wanted__ = None;
                let mut gas_used__ = None;
                let mut tx__ = None;
                let mut timestamp__ = None;
                let mut events__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Txhash => {
                            if txhash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txhash"));
                            }
                            txhash__ = Some(map.next_value()?);
                        }
                        GeneratedField::Codespace => {
                            if codespace__.is_some() {
                                return Err(serde::de::Error::duplicate_field("codespace"));
                            }
                            codespace__ = Some(map.next_value()?);
                        }
                        GeneratedField::Code => {
                            if code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("code"));
                            }
                            code__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = Some(map.next_value()?);
                        }
                        GeneratedField::RawLog => {
                            if raw_log__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rawLog"));
                            }
                            raw_log__ = Some(map.next_value()?);
                        }
                        GeneratedField::Logs => {
                            if logs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("logs"));
                            }
                            logs__ = Some(map.next_value()?);
                        }
                        GeneratedField::Info => {
                            if info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("info"));
                            }
                            info__ = Some(map.next_value()?);
                        }
                        GeneratedField::GasWanted => {
                            if gas_wanted__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasWanted"));
                            }
                            gas_wanted__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GasUsed => {
                            if gas_used__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasUsed"));
                            }
                            gas_used__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Tx => {
                            if tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tx"));
                            }
                            tx__ = map.next_value()?;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = Some(map.next_value()?);
                        }
                        GeneratedField::Events => {
                            if events__.is_some() {
                                return Err(serde::de::Error::duplicate_field("events"));
                            }
                            events__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TxResponse {
                    height: height__.unwrap_or_default(),
                    txhash: txhash__.unwrap_or_default(),
                    codespace: codespace__.unwrap_or_default(),
                    code: code__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                    raw_log: raw_log__.unwrap_or_default(),
                    logs: logs__.unwrap_or_default(),
                    info: info__.unwrap_or_default(),
                    gas_wanted: gas_wanted__.unwrap_or_default(),
                    gas_used: gas_used__.unwrap_or_default(),
                    tx: tx__,
                    timestamp: timestamp__.unwrap_or_default(),
                    events: events__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.abci.v1beta1.TxResponse", FIELDS, GeneratedVisitor)
    }
}
