// @generated
impl serde::Serialize for ChunkRequest {
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
        if self.format != 0 {
            len += 1;
        }
        if self.index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.statesync.ChunkRequest", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.format != 0 {
            struct_ser.serialize_field("format", &self.format)?;
        }
        if self.index != 0 {
            struct_ser.serialize_field("index", &self.index)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChunkRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "format",
            "index",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Format,
            Index,
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
                            "format" => Ok(GeneratedField::Format),
                            "index" => Ok(GeneratedField::Index),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChunkRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.statesync.ChunkRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChunkRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut format__ = None;
                let mut index__ = None;
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
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ChunkRequest {
                    height: height__.unwrap_or_default(),
                    format: format__.unwrap_or_default(),
                    index: index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.statesync.ChunkRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ChunkResponse {
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
        if self.format != 0 {
            len += 1;
        }
        if self.index != 0 {
            len += 1;
        }
        if !self.chunk.is_empty() {
            len += 1;
        }
        if self.missing {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.statesync.ChunkResponse", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.format != 0 {
            struct_ser.serialize_field("format", &self.format)?;
        }
        if self.index != 0 {
            struct_ser.serialize_field("index", &self.index)?;
        }
        if !self.chunk.is_empty() {
            struct_ser.serialize_field("chunk", pbjson::private::base64::encode(&self.chunk).as_str())?;
        }
        if self.missing {
            struct_ser.serialize_field("missing", &self.missing)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChunkResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "format",
            "index",
            "chunk",
            "missing",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Format,
            Index,
            Chunk,
            Missing,
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
                            "format" => Ok(GeneratedField::Format),
                            "index" => Ok(GeneratedField::Index),
                            "chunk" => Ok(GeneratedField::Chunk),
                            "missing" => Ok(GeneratedField::Missing),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChunkResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.statesync.ChunkResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChunkResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut format__ = None;
                let mut index__ = None;
                let mut chunk__ = None;
                let mut missing__ = None;
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
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Chunk => {
                            if chunk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunk"));
                            }
                            chunk__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Missing => {
                            if missing__.is_some() {
                                return Err(serde::de::Error::duplicate_field("missing"));
                            }
                            missing__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ChunkResponse {
                    height: height__.unwrap_or_default(),
                    format: format__.unwrap_or_default(),
                    index: index__.unwrap_or_default(),
                    chunk: chunk__.unwrap_or_default(),
                    missing: missing__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.statesync.ChunkResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Message {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.sum.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.statesync.Message", len)?;
        if let Some(v) = self.sum.as_ref() {
            match v {
                message::Sum::SnapshotsRequest(v) => {
                    struct_ser.serialize_field("snapshotsRequest", v)?;
                }
                message::Sum::SnapshotsResponse(v) => {
                    struct_ser.serialize_field("snapshotsResponse", v)?;
                }
                message::Sum::ChunkRequest(v) => {
                    struct_ser.serialize_field("chunkRequest", v)?;
                }
                message::Sum::ChunkResponse(v) => {
                    struct_ser.serialize_field("chunkResponse", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Message {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "snapshots_request",
            "snapshotsRequest",
            "snapshots_response",
            "snapshotsResponse",
            "chunk_request",
            "chunkRequest",
            "chunk_response",
            "chunkResponse",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SnapshotsRequest,
            SnapshotsResponse,
            ChunkRequest,
            ChunkResponse,
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
                            "snapshotsRequest" | "snapshots_request" => Ok(GeneratedField::SnapshotsRequest),
                            "snapshotsResponse" | "snapshots_response" => Ok(GeneratedField::SnapshotsResponse),
                            "chunkRequest" | "chunk_request" => Ok(GeneratedField::ChunkRequest),
                            "chunkResponse" | "chunk_response" => Ok(GeneratedField::ChunkResponse),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Message;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.statesync.Message")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Message, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sum__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SnapshotsRequest => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("snapshotsRequest"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::SnapshotsRequest)
;
                        }
                        GeneratedField::SnapshotsResponse => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("snapshotsResponse"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::SnapshotsResponse)
;
                        }
                        GeneratedField::ChunkRequest => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunkRequest"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::ChunkRequest)
;
                        }
                        GeneratedField::ChunkResponse => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunkResponse"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::ChunkResponse)
;
                        }
                    }
                }
                Ok(Message {
                    sum: sum__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.statesync.Message", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SnapshotsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("tendermint.statesync.SnapshotsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SnapshotsRequest {
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
            type Value = SnapshotsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.statesync.SnapshotsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SnapshotsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(SnapshotsRequest {
                })
            }
        }
        deserializer.deserialize_struct("tendermint.statesync.SnapshotsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SnapshotsResponse {
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
        if self.format != 0 {
            len += 1;
        }
        if self.chunks != 0 {
            len += 1;
        }
        if !self.hash.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.statesync.SnapshotsResponse", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.format != 0 {
            struct_ser.serialize_field("format", &self.format)?;
        }
        if self.chunks != 0 {
            struct_ser.serialize_field("chunks", &self.chunks)?;
        }
        if !self.hash.is_empty() {
            struct_ser.serialize_field("hash", pbjson::private::base64::encode(&self.hash).as_str())?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", pbjson::private::base64::encode(&self.metadata).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SnapshotsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "format",
            "chunks",
            "hash",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Format,
            Chunks,
            Hash,
            Metadata,
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
                            "format" => Ok(GeneratedField::Format),
                            "chunks" => Ok(GeneratedField::Chunks),
                            "hash" => Ok(GeneratedField::Hash),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SnapshotsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.statesync.SnapshotsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SnapshotsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut format__ = None;
                let mut chunks__ = None;
                let mut hash__ = None;
                let mut metadata__ = None;
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
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Chunks => {
                            if chunks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunks"));
                            }
                            chunks__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Hash => {
                            if hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hash"));
                            }
                            hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SnapshotsResponse {
                    height: height__.unwrap_or_default(),
                    format: format__.unwrap_or_default(),
                    chunks: chunks__.unwrap_or_default(),
                    hash: hash__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.statesync.SnapshotsResponse", FIELDS, GeneratedVisitor)
    }
}
