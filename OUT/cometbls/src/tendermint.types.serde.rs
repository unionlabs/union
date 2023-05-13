// @generated
impl serde::Serialize for Block {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.header.is_some() {
            len += 1;
        }
        if self.data.is_some() {
            len += 1;
        }
        if self.evidence.is_some() {
            len += 1;
        }
        if self.last_commit.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Block", len)?;
        if let Some(v) = self.header.as_ref() {
            struct_ser.serialize_field("header", v)?;
        }
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        if let Some(v) = self.evidence.as_ref() {
            struct_ser.serialize_field("evidence", v)?;
        }
        if let Some(v) = self.last_commit.as_ref() {
            struct_ser.serialize_field("lastCommit", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Block {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "header",
            "data",
            "evidence",
            "last_commit",
            "lastCommit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Header,
            Data,
            Evidence,
            LastCommit,
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
                            "header" => Ok(GeneratedField::Header),
                            "data" => Ok(GeneratedField::Data),
                            "evidence" => Ok(GeneratedField::Evidence),
                            "lastCommit" | "last_commit" => Ok(GeneratedField::LastCommit),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Block;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Block")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Block, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut header__ = None;
                let mut data__ = None;
                let mut evidence__ = None;
                let mut last_commit__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Header => {
                            if header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("header"));
                            }
                            header__ = map.next_value()?;
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = map.next_value()?;
                        }
                        GeneratedField::Evidence => {
                            if evidence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("evidence"));
                            }
                            evidence__ = map.next_value()?;
                        }
                        GeneratedField::LastCommit => {
                            if last_commit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastCommit"));
                            }
                            last_commit__ = map.next_value()?;
                        }
                    }
                }
                Ok(Block {
                    header: header__,
                    data: data__,
                    evidence: evidence__,
                    last_commit: last_commit__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Block", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BlockId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.hash.is_empty() {
            len += 1;
        }
        if self.part_set_header.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.BlockID", len)?;
        if !self.hash.is_empty() {
            struct_ser.serialize_field("hash", pbjson::private::base64::encode(&self.hash).as_str())?;
        }
        if let Some(v) = self.part_set_header.as_ref() {
            struct_ser.serialize_field("partSetHeader", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BlockId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "hash",
            "part_set_header",
            "partSetHeader",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Hash,
            PartSetHeader,
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
                            "hash" => Ok(GeneratedField::Hash),
                            "partSetHeader" | "part_set_header" => Ok(GeneratedField::PartSetHeader),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BlockId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.BlockID")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BlockId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut hash__ = None;
                let mut part_set_header__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Hash => {
                            if hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hash"));
                            }
                            hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PartSetHeader => {
                            if part_set_header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partSetHeader"));
                            }
                            part_set_header__ = map.next_value()?;
                        }
                    }
                }
                Ok(BlockId {
                    hash: hash__.unwrap_or_default(),
                    part_set_header: part_set_header__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.BlockID", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BlockIdFlag {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "BLOCK_ID_FLAG_UNKNOWN",
            Self::Absent => "BLOCK_ID_FLAG_ABSENT",
            Self::Commit => "BLOCK_ID_FLAG_COMMIT",
            Self::Nil => "BLOCK_ID_FLAG_NIL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for BlockIdFlag {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "BLOCK_ID_FLAG_UNKNOWN",
            "BLOCK_ID_FLAG_ABSENT",
            "BLOCK_ID_FLAG_COMMIT",
            "BLOCK_ID_FLAG_NIL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BlockIdFlag;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(BlockIdFlag::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(BlockIdFlag::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "BLOCK_ID_FLAG_UNKNOWN" => Ok(BlockIdFlag::Unknown),
                    "BLOCK_ID_FLAG_ABSENT" => Ok(BlockIdFlag::Absent),
                    "BLOCK_ID_FLAG_COMMIT" => Ok(BlockIdFlag::Commit),
                    "BLOCK_ID_FLAG_NIL" => Ok(BlockIdFlag::Nil),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for BlockMeta {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_id.is_some() {
            len += 1;
        }
        if self.block_size != 0 {
            len += 1;
        }
        if self.header.is_some() {
            len += 1;
        }
        if self.num_txs != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.BlockMeta", len)?;
        if let Some(v) = self.block_id.as_ref() {
            struct_ser.serialize_field("blockId", v)?;
        }
        if self.block_size != 0 {
            struct_ser.serialize_field("blockSize", ToString::to_string(&self.block_size).as_str())?;
        }
        if let Some(v) = self.header.as_ref() {
            struct_ser.serialize_field("header", v)?;
        }
        if self.num_txs != 0 {
            struct_ser.serialize_field("numTxs", ToString::to_string(&self.num_txs).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BlockMeta {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_id",
            "blockId",
            "block_size",
            "blockSize",
            "header",
            "num_txs",
            "numTxs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockId,
            BlockSize,
            Header,
            NumTxs,
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
                            "blockId" | "block_id" => Ok(GeneratedField::BlockId),
                            "blockSize" | "block_size" => Ok(GeneratedField::BlockSize),
                            "header" => Ok(GeneratedField::Header),
                            "numTxs" | "num_txs" => Ok(GeneratedField::NumTxs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BlockMeta;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.BlockMeta")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BlockMeta, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_id__ = None;
                let mut block_size__ = None;
                let mut header__ = None;
                let mut num_txs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockId => {
                            if block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockId"));
                            }
                            block_id__ = map.next_value()?;
                        }
                        GeneratedField::BlockSize => {
                            if block_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockSize"));
                            }
                            block_size__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Header => {
                            if header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("header"));
                            }
                            header__ = map.next_value()?;
                        }
                        GeneratedField::NumTxs => {
                            if num_txs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("numTxs"));
                            }
                            num_txs__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(BlockMeta {
                    block_id: block_id__,
                    block_size: block_size__.unwrap_or_default(),
                    header: header__,
                    num_txs: num_txs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.BlockMeta", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BlockParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.max_bytes != 0 {
            len += 1;
        }
        if self.max_gas != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.BlockParams", len)?;
        if self.max_bytes != 0 {
            struct_ser.serialize_field("maxBytes", ToString::to_string(&self.max_bytes).as_str())?;
        }
        if self.max_gas != 0 {
            struct_ser.serialize_field("maxGas", ToString::to_string(&self.max_gas).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BlockParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "max_bytes",
            "maxBytes",
            "max_gas",
            "maxGas",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxBytes,
            MaxGas,
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
                            "maxBytes" | "max_bytes" => Ok(GeneratedField::MaxBytes),
                            "maxGas" | "max_gas" => Ok(GeneratedField::MaxGas),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BlockParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.BlockParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BlockParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_bytes__ = None;
                let mut max_gas__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MaxBytes => {
                            if max_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxBytes"));
                            }
                            max_bytes__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxGas => {
                            if max_gas__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxGas"));
                            }
                            max_gas__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(BlockParams {
                    max_bytes: max_bytes__.unwrap_or_default(),
                    max_gas: max_gas__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.BlockParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CanonicalBlockId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.hash.is_empty() {
            len += 1;
        }
        if self.part_set_header.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.CanonicalBlockID", len)?;
        if !self.hash.is_empty() {
            struct_ser.serialize_field("hash", pbjson::private::base64::encode(&self.hash).as_str())?;
        }
        if let Some(v) = self.part_set_header.as_ref() {
            struct_ser.serialize_field("partSetHeader", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CanonicalBlockId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "hash",
            "part_set_header",
            "partSetHeader",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Hash,
            PartSetHeader,
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
                            "hash" => Ok(GeneratedField::Hash),
                            "partSetHeader" | "part_set_header" => Ok(GeneratedField::PartSetHeader),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CanonicalBlockId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.CanonicalBlockID")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CanonicalBlockId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut hash__ = None;
                let mut part_set_header__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Hash => {
                            if hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hash"));
                            }
                            hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PartSetHeader => {
                            if part_set_header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partSetHeader"));
                            }
                            part_set_header__ = map.next_value()?;
                        }
                    }
                }
                Ok(CanonicalBlockId {
                    hash: hash__.unwrap_or_default(),
                    part_set_header: part_set_header__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.CanonicalBlockID", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CanonicalPartSetHeader {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.total != 0 {
            len += 1;
        }
        if !self.hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.CanonicalPartSetHeader", len)?;
        if self.total != 0 {
            struct_ser.serialize_field("total", &self.total)?;
        }
        if !self.hash.is_empty() {
            struct_ser.serialize_field("hash", pbjson::private::base64::encode(&self.hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CanonicalPartSetHeader {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "total",
            "hash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Total,
            Hash,
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
                            "total" => Ok(GeneratedField::Total),
                            "hash" => Ok(GeneratedField::Hash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CanonicalPartSetHeader;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.CanonicalPartSetHeader")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CanonicalPartSetHeader, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut total__ = None;
                let mut hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Total => {
                            if total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("total"));
                            }
                            total__ = 
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
                    }
                }
                Ok(CanonicalPartSetHeader {
                    total: total__.unwrap_or_default(),
                    hash: hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.CanonicalPartSetHeader", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CanonicalProposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type != 0 {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.round != 0 {
            len += 1;
        }
        if self.pol_round != 0 {
            len += 1;
        }
        if self.block_id.is_some() {
            len += 1;
        }
        if self.timestamp.is_some() {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.CanonicalProposal", len)?;
        if self.r#type != 0 {
            let v = SignedMsgType::from_i32(self.r#type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.round != 0 {
            struct_ser.serialize_field("round", ToString::to_string(&self.round).as_str())?;
        }
        if self.pol_round != 0 {
            struct_ser.serialize_field("polRound", ToString::to_string(&self.pol_round).as_str())?;
        }
        if let Some(v) = self.block_id.as_ref() {
            struct_ser.serialize_field("blockId", v)?;
        }
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CanonicalProposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "height",
            "round",
            "pol_round",
            "polRound",
            "block_id",
            "blockId",
            "timestamp",
            "chain_id",
            "chainId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Height,
            Round,
            PolRound,
            BlockId,
            Timestamp,
            ChainId,
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
                            "height" => Ok(GeneratedField::Height),
                            "round" => Ok(GeneratedField::Round),
                            "polRound" | "pol_round" => Ok(GeneratedField::PolRound),
                            "blockId" | "block_id" => Ok(GeneratedField::BlockId),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CanonicalProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.CanonicalProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CanonicalProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut height__ = None;
                let mut round__ = None;
                let mut pol_round__ = None;
                let mut block_id__ = None;
                let mut timestamp__ = None;
                let mut chain_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value::<SignedMsgType>()? as i32);
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Round => {
                            if round__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round"));
                            }
                            round__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PolRound => {
                            if pol_round__.is_some() {
                                return Err(serde::de::Error::duplicate_field("polRound"));
                            }
                            pol_round__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BlockId => {
                            if block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockId"));
                            }
                            block_id__ = map.next_value()?;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map.next_value()?;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CanonicalProposal {
                    r#type: r#type__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    round: round__.unwrap_or_default(),
                    pol_round: pol_round__.unwrap_or_default(),
                    block_id: block_id__,
                    timestamp: timestamp__,
                    chain_id: chain_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.CanonicalProposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CanonicalVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type != 0 {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.round != 0 {
            len += 1;
        }
        if self.block_id.is_some() {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.CanonicalVote", len)?;
        if self.r#type != 0 {
            let v = SignedMsgType::from_i32(self.r#type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.round != 0 {
            struct_ser.serialize_field("round", ToString::to_string(&self.round).as_str())?;
        }
        if let Some(v) = self.block_id.as_ref() {
            struct_ser.serialize_field("blockId", v)?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CanonicalVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "height",
            "round",
            "block_id",
            "blockId",
            "chain_id",
            "chainId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Height,
            Round,
            BlockId,
            ChainId,
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
                            "height" => Ok(GeneratedField::Height),
                            "round" => Ok(GeneratedField::Round),
                            "blockId" | "block_id" => Ok(GeneratedField::BlockId),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CanonicalVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.CanonicalVote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CanonicalVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut height__ = None;
                let mut round__ = None;
                let mut block_id__ = None;
                let mut chain_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value::<SignedMsgType>()? as i32);
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Round => {
                            if round__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round"));
                            }
                            round__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BlockId => {
                            if block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockId"));
                            }
                            block_id__ = map.next_value()?;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CanonicalVote {
                    r#type: r#type__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    round: round__.unwrap_or_default(),
                    block_id: block_id__,
                    chain_id: chain_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.CanonicalVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Commit {
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
        if self.round != 0 {
            len += 1;
        }
        if self.block_id.is_some() {
            len += 1;
        }
        if !self.signatures.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Commit", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.round != 0 {
            struct_ser.serialize_field("round", &self.round)?;
        }
        if let Some(v) = self.block_id.as_ref() {
            struct_ser.serialize_field("blockId", v)?;
        }
        if !self.signatures.is_empty() {
            struct_ser.serialize_field("signatures", &self.signatures)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Commit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "round",
            "block_id",
            "blockId",
            "signatures",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Round,
            BlockId,
            Signatures,
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
                            "round" => Ok(GeneratedField::Round),
                            "blockId" | "block_id" => Ok(GeneratedField::BlockId),
                            "signatures" => Ok(GeneratedField::Signatures),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Commit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Commit")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Commit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut round__ = None;
                let mut block_id__ = None;
                let mut signatures__ = None;
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
                        GeneratedField::Round => {
                            if round__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round"));
                            }
                            round__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BlockId => {
                            if block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockId"));
                            }
                            block_id__ = map.next_value()?;
                        }
                        GeneratedField::Signatures => {
                            if signatures__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signatures"));
                            }
                            signatures__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Commit {
                    height: height__.unwrap_or_default(),
                    round: round__.unwrap_or_default(),
                    block_id: block_id__,
                    signatures: signatures__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Commit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CommitSig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_id_flag != 0 {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.timestamp.is_some() {
            len += 1;
        }
        if !self.signature.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.CommitSig", len)?;
        if self.block_id_flag != 0 {
            let v = BlockIdFlag::from_i32(self.block_id_flag)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.block_id_flag)))?;
            struct_ser.serialize_field("blockIdFlag", &v)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", pbjson::private::base64::encode(&self.validator_address).as_str())?;
        }
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        if !self.signature.is_empty() {
            struct_ser.serialize_field("signature", pbjson::private::base64::encode(&self.signature).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommitSig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_id_flag",
            "blockIdFlag",
            "validator_address",
            "validatorAddress",
            "timestamp",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockIdFlag,
            ValidatorAddress,
            Timestamp,
            Signature,
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
                            "blockIdFlag" | "block_id_flag" => Ok(GeneratedField::BlockIdFlag),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CommitSig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.CommitSig")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CommitSig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_id_flag__ = None;
                let mut validator_address__ = None;
                let mut timestamp__ = None;
                let mut signature__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockIdFlag => {
                            if block_id_flag__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockIdFlag"));
                            }
                            block_id_flag__ = Some(map.next_value::<BlockIdFlag>()? as i32);
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map.next_value()?;
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CommitSig {
                    block_id_flag: block_id_flag__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                    timestamp: timestamp__,
                    signature: signature__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.CommitSig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConsensusParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block.is_some() {
            len += 1;
        }
        if self.evidence.is_some() {
            len += 1;
        }
        if self.validator.is_some() {
            len += 1;
        }
        if self.version.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.ConsensusParams", len)?;
        if let Some(v) = self.block.as_ref() {
            struct_ser.serialize_field("block", v)?;
        }
        if let Some(v) = self.evidence.as_ref() {
            struct_ser.serialize_field("evidence", v)?;
        }
        if let Some(v) = self.validator.as_ref() {
            struct_ser.serialize_field("validator", v)?;
        }
        if let Some(v) = self.version.as_ref() {
            struct_ser.serialize_field("version", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConsensusParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block",
            "evidence",
            "validator",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Block,
            Evidence,
            Validator,
            Version,
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
                            "evidence" => Ok(GeneratedField::Evidence),
                            "validator" => Ok(GeneratedField::Validator),
                            "version" => Ok(GeneratedField::Version),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConsensusParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.ConsensusParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConsensusParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block__ = None;
                let mut evidence__ = None;
                let mut validator__ = None;
                let mut version__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Block => {
                            if block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("block"));
                            }
                            block__ = map.next_value()?;
                        }
                        GeneratedField::Evidence => {
                            if evidence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("evidence"));
                            }
                            evidence__ = map.next_value()?;
                        }
                        GeneratedField::Validator => {
                            if validator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validator"));
                            }
                            validator__ = map.next_value()?;
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = map.next_value()?;
                        }
                    }
                }
                Ok(ConsensusParams {
                    block: block__,
                    evidence: evidence__,
                    validator: validator__,
                    version: version__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.ConsensusParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Data {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.txs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Data", len)?;
        if !self.txs.is_empty() {
            struct_ser.serialize_field("txs", &self.txs.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Data {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "txs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = Data;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Data")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Data, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut txs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Txs => {
                            if txs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txs"));
                            }
                            txs__ = 
                                Some(map.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(Data {
                    txs: txs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Data", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DuplicateVoteEvidence {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote_a.is_some() {
            len += 1;
        }
        if self.vote_b.is_some() {
            len += 1;
        }
        if self.total_voting_power != 0 {
            len += 1;
        }
        if self.validator_power != 0 {
            len += 1;
        }
        if self.timestamp.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.DuplicateVoteEvidence", len)?;
        if let Some(v) = self.vote_a.as_ref() {
            struct_ser.serialize_field("voteA", v)?;
        }
        if let Some(v) = self.vote_b.as_ref() {
            struct_ser.serialize_field("voteB", v)?;
        }
        if self.total_voting_power != 0 {
            struct_ser.serialize_field("totalVotingPower", ToString::to_string(&self.total_voting_power).as_str())?;
        }
        if self.validator_power != 0 {
            struct_ser.serialize_field("validatorPower", ToString::to_string(&self.validator_power).as_str())?;
        }
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DuplicateVoteEvidence {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote_a",
            "voteA",
            "vote_b",
            "voteB",
            "total_voting_power",
            "totalVotingPower",
            "validator_power",
            "validatorPower",
            "timestamp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VoteA,
            VoteB,
            TotalVotingPower,
            ValidatorPower,
            Timestamp,
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
                            "voteA" | "vote_a" => Ok(GeneratedField::VoteA),
                            "voteB" | "vote_b" => Ok(GeneratedField::VoteB),
                            "totalVotingPower" | "total_voting_power" => Ok(GeneratedField::TotalVotingPower),
                            "validatorPower" | "validator_power" => Ok(GeneratedField::ValidatorPower),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DuplicateVoteEvidence;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.DuplicateVoteEvidence")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DuplicateVoteEvidence, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote_a__ = None;
                let mut vote_b__ = None;
                let mut total_voting_power__ = None;
                let mut validator_power__ = None;
                let mut timestamp__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::VoteA => {
                            if vote_a__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voteA"));
                            }
                            vote_a__ = map.next_value()?;
                        }
                        GeneratedField::VoteB => {
                            if vote_b__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voteB"));
                            }
                            vote_b__ = map.next_value()?;
                        }
                        GeneratedField::TotalVotingPower => {
                            if total_voting_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalVotingPower"));
                            }
                            total_voting_power__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValidatorPower => {
                            if validator_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorPower"));
                            }
                            validator_power__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map.next_value()?;
                        }
                    }
                }
                Ok(DuplicateVoteEvidence {
                    vote_a: vote_a__,
                    vote_b: vote_b__,
                    total_voting_power: total_voting_power__.unwrap_or_default(),
                    validator_power: validator_power__.unwrap_or_default(),
                    timestamp: timestamp__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.DuplicateVoteEvidence", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventDataRoundState {
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
        if self.round != 0 {
            len += 1;
        }
        if !self.step.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.EventDataRoundState", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.round != 0 {
            struct_ser.serialize_field("round", &self.round)?;
        }
        if !self.step.is_empty() {
            struct_ser.serialize_field("step", &self.step)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventDataRoundState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "round",
            "step",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Round,
            Step,
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
                            "round" => Ok(GeneratedField::Round),
                            "step" => Ok(GeneratedField::Step),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventDataRoundState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.EventDataRoundState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventDataRoundState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut round__ = None;
                let mut step__ = None;
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
                        GeneratedField::Round => {
                            if round__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round"));
                            }
                            round__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Step => {
                            if step__.is_some() {
                                return Err(serde::de::Error::duplicate_field("step"));
                            }
                            step__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(EventDataRoundState {
                    height: height__.unwrap_or_default(),
                    round: round__.unwrap_or_default(),
                    step: step__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.EventDataRoundState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Evidence {
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
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Evidence", len)?;
        if let Some(v) = self.sum.as_ref() {
            match v {
                evidence::Sum::DuplicateVoteEvidence(v) => {
                    struct_ser.serialize_field("duplicateVoteEvidence", v)?;
                }
                evidence::Sum::LightClientAttackEvidence(v) => {
                    struct_ser.serialize_field("lightClientAttackEvidence", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Evidence {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "duplicate_vote_evidence",
            "duplicateVoteEvidence",
            "light_client_attack_evidence",
            "lightClientAttackEvidence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DuplicateVoteEvidence,
            LightClientAttackEvidence,
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
                            "duplicateVoteEvidence" | "duplicate_vote_evidence" => Ok(GeneratedField::DuplicateVoteEvidence),
                            "lightClientAttackEvidence" | "light_client_attack_evidence" => Ok(GeneratedField::LightClientAttackEvidence),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Evidence;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Evidence")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Evidence, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sum__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DuplicateVoteEvidence => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("duplicateVoteEvidence"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(evidence::Sum::DuplicateVoteEvidence)
;
                        }
                        GeneratedField::LightClientAttackEvidence => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lightClientAttackEvidence"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(evidence::Sum::LightClientAttackEvidence)
;
                        }
                    }
                }
                Ok(Evidence {
                    sum: sum__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Evidence", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EvidenceList {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.evidence.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.EvidenceList", len)?;
        if !self.evidence.is_empty() {
            struct_ser.serialize_field("evidence", &self.evidence)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EvidenceList {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "evidence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Evidence,
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
                            "evidence" => Ok(GeneratedField::Evidence),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EvidenceList;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.EvidenceList")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EvidenceList, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut evidence__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Evidence => {
                            if evidence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("evidence"));
                            }
                            evidence__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(EvidenceList {
                    evidence: evidence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.EvidenceList", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EvidenceParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.max_age_num_blocks != 0 {
            len += 1;
        }
        if self.max_age_duration.is_some() {
            len += 1;
        }
        if self.max_bytes != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.EvidenceParams", len)?;
        if self.max_age_num_blocks != 0 {
            struct_ser.serialize_field("maxAgeNumBlocks", ToString::to_string(&self.max_age_num_blocks).as_str())?;
        }
        if let Some(v) = self.max_age_duration.as_ref() {
            struct_ser.serialize_field("maxAgeDuration", v)?;
        }
        if self.max_bytes != 0 {
            struct_ser.serialize_field("maxBytes", ToString::to_string(&self.max_bytes).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EvidenceParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "max_age_num_blocks",
            "maxAgeNumBlocks",
            "max_age_duration",
            "maxAgeDuration",
            "max_bytes",
            "maxBytes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxAgeNumBlocks,
            MaxAgeDuration,
            MaxBytes,
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
                            "maxAgeNumBlocks" | "max_age_num_blocks" => Ok(GeneratedField::MaxAgeNumBlocks),
                            "maxAgeDuration" | "max_age_duration" => Ok(GeneratedField::MaxAgeDuration),
                            "maxBytes" | "max_bytes" => Ok(GeneratedField::MaxBytes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EvidenceParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.EvidenceParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EvidenceParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_age_num_blocks__ = None;
                let mut max_age_duration__ = None;
                let mut max_bytes__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MaxAgeNumBlocks => {
                            if max_age_num_blocks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxAgeNumBlocks"));
                            }
                            max_age_num_blocks__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxAgeDuration => {
                            if max_age_duration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxAgeDuration"));
                            }
                            max_age_duration__ = map.next_value()?;
                        }
                        GeneratedField::MaxBytes => {
                            if max_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxBytes"));
                            }
                            max_bytes__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EvidenceParams {
                    max_age_num_blocks: max_age_num_blocks__.unwrap_or_default(),
                    max_age_duration: max_age_duration__,
                    max_bytes: max_bytes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.EvidenceParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HashedParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_max_bytes != 0 {
            len += 1;
        }
        if self.block_max_gas != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.HashedParams", len)?;
        if self.block_max_bytes != 0 {
            struct_ser.serialize_field("blockMaxBytes", ToString::to_string(&self.block_max_bytes).as_str())?;
        }
        if self.block_max_gas != 0 {
            struct_ser.serialize_field("blockMaxGas", ToString::to_string(&self.block_max_gas).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HashedParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_max_bytes",
            "blockMaxBytes",
            "block_max_gas",
            "blockMaxGas",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockMaxBytes,
            BlockMaxGas,
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
                            "blockMaxBytes" | "block_max_bytes" => Ok(GeneratedField::BlockMaxBytes),
                            "blockMaxGas" | "block_max_gas" => Ok(GeneratedField::BlockMaxGas),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HashedParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.HashedParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HashedParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_max_bytes__ = None;
                let mut block_max_gas__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockMaxBytes => {
                            if block_max_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockMaxBytes"));
                            }
                            block_max_bytes__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BlockMaxGas => {
                            if block_max_gas__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockMaxGas"));
                            }
                            block_max_gas__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(HashedParams {
                    block_max_bytes: block_max_bytes__.unwrap_or_default(),
                    block_max_gas: block_max_gas__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.HashedParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Header {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.version.is_some() {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.time.is_some() {
            len += 1;
        }
        if self.last_block_id.is_some() {
            len += 1;
        }
        if !self.last_commit_hash.is_empty() {
            len += 1;
        }
        if !self.data_hash.is_empty() {
            len += 1;
        }
        if !self.validators_hash.is_empty() {
            len += 1;
        }
        if !self.next_validators_hash.is_empty() {
            len += 1;
        }
        if !self.consensus_hash.is_empty() {
            len += 1;
        }
        if !self.app_hash.is_empty() {
            len += 1;
        }
        if !self.last_results_hash.is_empty() {
            len += 1;
        }
        if !self.evidence_hash.is_empty() {
            len += 1;
        }
        if !self.proposer_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Header", len)?;
        if let Some(v) = self.version.as_ref() {
            struct_ser.serialize_field("version", v)?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.time.as_ref() {
            struct_ser.serialize_field("time", v)?;
        }
        if let Some(v) = self.last_block_id.as_ref() {
            struct_ser.serialize_field("lastBlockId", v)?;
        }
        if !self.last_commit_hash.is_empty() {
            struct_ser.serialize_field("lastCommitHash", pbjson::private::base64::encode(&self.last_commit_hash).as_str())?;
        }
        if !self.data_hash.is_empty() {
            struct_ser.serialize_field("dataHash", pbjson::private::base64::encode(&self.data_hash).as_str())?;
        }
        if !self.validators_hash.is_empty() {
            struct_ser.serialize_field("validatorsHash", pbjson::private::base64::encode(&self.validators_hash).as_str())?;
        }
        if !self.next_validators_hash.is_empty() {
            struct_ser.serialize_field("nextValidatorsHash", pbjson::private::base64::encode(&self.next_validators_hash).as_str())?;
        }
        if !self.consensus_hash.is_empty() {
            struct_ser.serialize_field("consensusHash", pbjson::private::base64::encode(&self.consensus_hash).as_str())?;
        }
        if !self.app_hash.is_empty() {
            struct_ser.serialize_field("appHash", pbjson::private::base64::encode(&self.app_hash).as_str())?;
        }
        if !self.last_results_hash.is_empty() {
            struct_ser.serialize_field("lastResultsHash", pbjson::private::base64::encode(&self.last_results_hash).as_str())?;
        }
        if !self.evidence_hash.is_empty() {
            struct_ser.serialize_field("evidenceHash", pbjson::private::base64::encode(&self.evidence_hash).as_str())?;
        }
        if !self.proposer_address.is_empty() {
            struct_ser.serialize_field("proposerAddress", pbjson::private::base64::encode(&self.proposer_address).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Header {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "version",
            "chain_id",
            "chainId",
            "height",
            "time",
            "last_block_id",
            "lastBlockId",
            "last_commit_hash",
            "lastCommitHash",
            "data_hash",
            "dataHash",
            "validators_hash",
            "validatorsHash",
            "next_validators_hash",
            "nextValidatorsHash",
            "consensus_hash",
            "consensusHash",
            "app_hash",
            "appHash",
            "last_results_hash",
            "lastResultsHash",
            "evidence_hash",
            "evidenceHash",
            "proposer_address",
            "proposerAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Version,
            ChainId,
            Height,
            Time,
            LastBlockId,
            LastCommitHash,
            DataHash,
            ValidatorsHash,
            NextValidatorsHash,
            ConsensusHash,
            AppHash,
            LastResultsHash,
            EvidenceHash,
            ProposerAddress,
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
                            "version" => Ok(GeneratedField::Version),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "height" => Ok(GeneratedField::Height),
                            "time" => Ok(GeneratedField::Time),
                            "lastBlockId" | "last_block_id" => Ok(GeneratedField::LastBlockId),
                            "lastCommitHash" | "last_commit_hash" => Ok(GeneratedField::LastCommitHash),
                            "dataHash" | "data_hash" => Ok(GeneratedField::DataHash),
                            "validatorsHash" | "validators_hash" => Ok(GeneratedField::ValidatorsHash),
                            "nextValidatorsHash" | "next_validators_hash" => Ok(GeneratedField::NextValidatorsHash),
                            "consensusHash" | "consensus_hash" => Ok(GeneratedField::ConsensusHash),
                            "appHash" | "app_hash" => Ok(GeneratedField::AppHash),
                            "lastResultsHash" | "last_results_hash" => Ok(GeneratedField::LastResultsHash),
                            "evidenceHash" | "evidence_hash" => Ok(GeneratedField::EvidenceHash),
                            "proposerAddress" | "proposer_address" => Ok(GeneratedField::ProposerAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Header;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Header")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Header, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut version__ = None;
                let mut chain_id__ = None;
                let mut height__ = None;
                let mut time__ = None;
                let mut last_block_id__ = None;
                let mut last_commit_hash__ = None;
                let mut data_hash__ = None;
                let mut validators_hash__ = None;
                let mut next_validators_hash__ = None;
                let mut consensus_hash__ = None;
                let mut app_hash__ = None;
                let mut last_results_hash__ = None;
                let mut evidence_hash__ = None;
                let mut proposer_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = map.next_value()?;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Time => {
                            if time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("time"));
                            }
                            time__ = map.next_value()?;
                        }
                        GeneratedField::LastBlockId => {
                            if last_block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastBlockId"));
                            }
                            last_block_id__ = map.next_value()?;
                        }
                        GeneratedField::LastCommitHash => {
                            if last_commit_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastCommitHash"));
                            }
                            last_commit_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DataHash => {
                            if data_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataHash"));
                            }
                            data_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValidatorsHash => {
                            if validators_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorsHash"));
                            }
                            validators_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NextValidatorsHash => {
                            if next_validators_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextValidatorsHash"));
                            }
                            next_validators_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ConsensusHash => {
                            if consensus_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusHash"));
                            }
                            consensus_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AppHash => {
                            if app_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appHash"));
                            }
                            app_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LastResultsHash => {
                            if last_results_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastResultsHash"));
                            }
                            last_results_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EvidenceHash => {
                            if evidence_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("evidenceHash"));
                            }
                            evidence_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProposerAddress => {
                            if proposer_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposerAddress"));
                            }
                            proposer_address__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Header {
                    version: version__,
                    chain_id: chain_id__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    time: time__,
                    last_block_id: last_block_id__,
                    last_commit_hash: last_commit_hash__.unwrap_or_default(),
                    data_hash: data_hash__.unwrap_or_default(),
                    validators_hash: validators_hash__.unwrap_or_default(),
                    next_validators_hash: next_validators_hash__.unwrap_or_default(),
                    consensus_hash: consensus_hash__.unwrap_or_default(),
                    app_hash: app_hash__.unwrap_or_default(),
                    last_results_hash: last_results_hash__.unwrap_or_default(),
                    evidence_hash: evidence_hash__.unwrap_or_default(),
                    proposer_address: proposer_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Header", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LightBlock {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.signed_header.is_some() {
            len += 1;
        }
        if self.validator_set.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.LightBlock", len)?;
        if let Some(v) = self.signed_header.as_ref() {
            struct_ser.serialize_field("signedHeader", v)?;
        }
        if let Some(v) = self.validator_set.as_ref() {
            struct_ser.serialize_field("validatorSet", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LightBlock {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signed_header",
            "signedHeader",
            "validator_set",
            "validatorSet",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignedHeader,
            ValidatorSet,
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
                            "signedHeader" | "signed_header" => Ok(GeneratedField::SignedHeader),
                            "validatorSet" | "validator_set" => Ok(GeneratedField::ValidatorSet),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LightBlock;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.LightBlock")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<LightBlock, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signed_header__ = None;
                let mut validator_set__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SignedHeader => {
                            if signed_header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signedHeader"));
                            }
                            signed_header__ = map.next_value()?;
                        }
                        GeneratedField::ValidatorSet => {
                            if validator_set__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSet"));
                            }
                            validator_set__ = map.next_value()?;
                        }
                    }
                }
                Ok(LightBlock {
                    signed_header: signed_header__,
                    validator_set: validator_set__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.LightBlock", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LightClientAttackEvidence {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.conflicting_block.is_some() {
            len += 1;
        }
        if self.common_height != 0 {
            len += 1;
        }
        if !self.byzantine_validators.is_empty() {
            len += 1;
        }
        if self.total_voting_power != 0 {
            len += 1;
        }
        if self.timestamp.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.LightClientAttackEvidence", len)?;
        if let Some(v) = self.conflicting_block.as_ref() {
            struct_ser.serialize_field("conflictingBlock", v)?;
        }
        if self.common_height != 0 {
            struct_ser.serialize_field("commonHeight", ToString::to_string(&self.common_height).as_str())?;
        }
        if !self.byzantine_validators.is_empty() {
            struct_ser.serialize_field("byzantineValidators", &self.byzantine_validators)?;
        }
        if self.total_voting_power != 0 {
            struct_ser.serialize_field("totalVotingPower", ToString::to_string(&self.total_voting_power).as_str())?;
        }
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LightClientAttackEvidence {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "conflicting_block",
            "conflictingBlock",
            "common_height",
            "commonHeight",
            "byzantine_validators",
            "byzantineValidators",
            "total_voting_power",
            "totalVotingPower",
            "timestamp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConflictingBlock,
            CommonHeight,
            ByzantineValidators,
            TotalVotingPower,
            Timestamp,
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
                            "conflictingBlock" | "conflicting_block" => Ok(GeneratedField::ConflictingBlock),
                            "commonHeight" | "common_height" => Ok(GeneratedField::CommonHeight),
                            "byzantineValidators" | "byzantine_validators" => Ok(GeneratedField::ByzantineValidators),
                            "totalVotingPower" | "total_voting_power" => Ok(GeneratedField::TotalVotingPower),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LightClientAttackEvidence;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.LightClientAttackEvidence")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<LightClientAttackEvidence, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut conflicting_block__ = None;
                let mut common_height__ = None;
                let mut byzantine_validators__ = None;
                let mut total_voting_power__ = None;
                let mut timestamp__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConflictingBlock => {
                            if conflicting_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("conflictingBlock"));
                            }
                            conflicting_block__ = map.next_value()?;
                        }
                        GeneratedField::CommonHeight => {
                            if common_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commonHeight"));
                            }
                            common_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ByzantineValidators => {
                            if byzantine_validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("byzantineValidators"));
                            }
                            byzantine_validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::TotalVotingPower => {
                            if total_voting_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalVotingPower"));
                            }
                            total_voting_power__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map.next_value()?;
                        }
                    }
                }
                Ok(LightClientAttackEvidence {
                    conflicting_block: conflicting_block__,
                    common_height: common_height__.unwrap_or_default(),
                    byzantine_validators: byzantine_validators__.unwrap_or_default(),
                    total_voting_power: total_voting_power__.unwrap_or_default(),
                    timestamp: timestamp__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.LightClientAttackEvidence", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Part {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.index != 0 {
            len += 1;
        }
        if !self.bytes.is_empty() {
            len += 1;
        }
        if self.proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Part", len)?;
        if self.index != 0 {
            struct_ser.serialize_field("index", &self.index)?;
        }
        if !self.bytes.is_empty() {
            struct_ser.serialize_field("bytes", pbjson::private::base64::encode(&self.bytes).as_str())?;
        }
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Part {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "index",
            "bytes",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Index,
            Bytes,
            Proof,
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
                            "index" => Ok(GeneratedField::Index),
                            "bytes" => Ok(GeneratedField::Bytes),
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Part;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Part")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Part, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut index__ = None;
                let mut bytes__ = None;
                let mut proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Bytes => {
                            if bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bytes"));
                            }
                            bytes__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map.next_value()?;
                        }
                    }
                }
                Ok(Part {
                    index: index__.unwrap_or_default(),
                    bytes: bytes__.unwrap_or_default(),
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Part", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PartSetHeader {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.total != 0 {
            len += 1;
        }
        if !self.hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.PartSetHeader", len)?;
        if self.total != 0 {
            struct_ser.serialize_field("total", &self.total)?;
        }
        if !self.hash.is_empty() {
            struct_ser.serialize_field("hash", pbjson::private::base64::encode(&self.hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PartSetHeader {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "total",
            "hash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Total,
            Hash,
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
                            "total" => Ok(GeneratedField::Total),
                            "hash" => Ok(GeneratedField::Hash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PartSetHeader;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.PartSetHeader")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PartSetHeader, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut total__ = None;
                let mut hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Total => {
                            if total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("total"));
                            }
                            total__ = 
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
                    }
                }
                Ok(PartSetHeader {
                    total: total__.unwrap_or_default(),
                    hash: hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.PartSetHeader", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Proposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type != 0 {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.round != 0 {
            len += 1;
        }
        if self.pol_round != 0 {
            len += 1;
        }
        if self.block_id.is_some() {
            len += 1;
        }
        if self.timestamp.is_some() {
            len += 1;
        }
        if !self.signature.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Proposal", len)?;
        if self.r#type != 0 {
            let v = SignedMsgType::from_i32(self.r#type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.round != 0 {
            struct_ser.serialize_field("round", &self.round)?;
        }
        if self.pol_round != 0 {
            struct_ser.serialize_field("polRound", &self.pol_round)?;
        }
        if let Some(v) = self.block_id.as_ref() {
            struct_ser.serialize_field("blockId", v)?;
        }
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        if !self.signature.is_empty() {
            struct_ser.serialize_field("signature", pbjson::private::base64::encode(&self.signature).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Proposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "height",
            "round",
            "pol_round",
            "polRound",
            "block_id",
            "blockId",
            "timestamp",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Height,
            Round,
            PolRound,
            BlockId,
            Timestamp,
            Signature,
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
                            "height" => Ok(GeneratedField::Height),
                            "round" => Ok(GeneratedField::Round),
                            "polRound" | "pol_round" => Ok(GeneratedField::PolRound),
                            "blockId" | "block_id" => Ok(GeneratedField::BlockId),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Proposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Proposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Proposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut height__ = None;
                let mut round__ = None;
                let mut pol_round__ = None;
                let mut block_id__ = None;
                let mut timestamp__ = None;
                let mut signature__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value::<SignedMsgType>()? as i32);
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Round => {
                            if round__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round"));
                            }
                            round__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PolRound => {
                            if pol_round__.is_some() {
                                return Err(serde::de::Error::duplicate_field("polRound"));
                            }
                            pol_round__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BlockId => {
                            if block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockId"));
                            }
                            block_id__ = map.next_value()?;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map.next_value()?;
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Proposal {
                    r#type: r#type__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    round: round__.unwrap_or_default(),
                    pol_round: pol_round__.unwrap_or_default(),
                    block_id: block_id__,
                    timestamp: timestamp__,
                    signature: signature__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Proposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignedHeader {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.header.is_some() {
            len += 1;
        }
        if self.commit.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.SignedHeader", len)?;
        if let Some(v) = self.header.as_ref() {
            struct_ser.serialize_field("header", v)?;
        }
        if let Some(v) = self.commit.as_ref() {
            struct_ser.serialize_field("commit", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignedHeader {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "header",
            "commit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Header,
            Commit,
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
                            "header" => Ok(GeneratedField::Header),
                            "commit" => Ok(GeneratedField::Commit),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignedHeader;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.SignedHeader")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SignedHeader, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut header__ = None;
                let mut commit__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Header => {
                            if header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("header"));
                            }
                            header__ = map.next_value()?;
                        }
                        GeneratedField::Commit => {
                            if commit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commit"));
                            }
                            commit__ = map.next_value()?;
                        }
                    }
                }
                Ok(SignedHeader {
                    header: header__,
                    commit: commit__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.SignedHeader", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignedMsgType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "SIGNED_MSG_TYPE_UNKNOWN",
            Self::Prevote => "SIGNED_MSG_TYPE_PREVOTE",
            Self::Precommit => "SIGNED_MSG_TYPE_PRECOMMIT",
            Self::Proposal => "SIGNED_MSG_TYPE_PROPOSAL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for SignedMsgType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SIGNED_MSG_TYPE_UNKNOWN",
            "SIGNED_MSG_TYPE_PREVOTE",
            "SIGNED_MSG_TYPE_PRECOMMIT",
            "SIGNED_MSG_TYPE_PROPOSAL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignedMsgType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(SignedMsgType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(SignedMsgType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SIGNED_MSG_TYPE_UNKNOWN" => Ok(SignedMsgType::Unknown),
                    "SIGNED_MSG_TYPE_PREVOTE" => Ok(SignedMsgType::Prevote),
                    "SIGNED_MSG_TYPE_PRECOMMIT" => Ok(SignedMsgType::Precommit),
                    "SIGNED_MSG_TYPE_PROPOSAL" => Ok(SignedMsgType::Proposal),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SimpleValidator {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.pub_key.is_some() {
            len += 1;
        }
        if self.voting_power != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.SimpleValidator", len)?;
        if let Some(v) = self.pub_key.as_ref() {
            struct_ser.serialize_field("pubKey", v)?;
        }
        if self.voting_power != 0 {
            struct_ser.serialize_field("votingPower", ToString::to_string(&self.voting_power).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SimpleValidator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pub_key",
            "pubKey",
            "voting_power",
            "votingPower",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PubKey,
            VotingPower,
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
                            "pubKey" | "pub_key" => Ok(GeneratedField::PubKey),
                            "votingPower" | "voting_power" => Ok(GeneratedField::VotingPower),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SimpleValidator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.SimpleValidator")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SimpleValidator, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pub_key__ = None;
                let mut voting_power__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PubKey => {
                            if pub_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubKey"));
                            }
                            pub_key__ = map.next_value()?;
                        }
                        GeneratedField::VotingPower => {
                            if voting_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPower"));
                            }
                            voting_power__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SimpleValidator {
                    pub_key: pub_key__,
                    voting_power: voting_power__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.SimpleValidator", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TxProof {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.root_hash.is_empty() {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        if self.proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.TxProof", len)?;
        if !self.root_hash.is_empty() {
            struct_ser.serialize_field("rootHash", pbjson::private::base64::encode(&self.root_hash).as_str())?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TxProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "root_hash",
            "rootHash",
            "data",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RootHash,
            Data,
            Proof,
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
                            "rootHash" | "root_hash" => Ok(GeneratedField::RootHash),
                            "data" => Ok(GeneratedField::Data),
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TxProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.TxProof")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TxProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut root_hash__ = None;
                let mut data__ = None;
                let mut proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RootHash => {
                            if root_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootHash"));
                            }
                            root_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map.next_value()?;
                        }
                    }
                }
                Ok(TxProof {
                    root_hash: root_hash__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.TxProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Validator {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        if self.pub_key.is_some() {
            len += 1;
        }
        if self.voting_power != 0 {
            len += 1;
        }
        if self.proposer_priority != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Validator", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", pbjson::private::base64::encode(&self.address).as_str())?;
        }
        if let Some(v) = self.pub_key.as_ref() {
            struct_ser.serialize_field("pubKey", v)?;
        }
        if self.voting_power != 0 {
            struct_ser.serialize_field("votingPower", ToString::to_string(&self.voting_power).as_str())?;
        }
        if self.proposer_priority != 0 {
            struct_ser.serialize_field("proposerPriority", ToString::to_string(&self.proposer_priority).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Validator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "pub_key",
            "pubKey",
            "voting_power",
            "votingPower",
            "proposer_priority",
            "proposerPriority",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            PubKey,
            VotingPower,
            ProposerPriority,
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
                            "address" => Ok(GeneratedField::Address),
                            "pubKey" | "pub_key" => Ok(GeneratedField::PubKey),
                            "votingPower" | "voting_power" => Ok(GeneratedField::VotingPower),
                            "proposerPriority" | "proposer_priority" => Ok(GeneratedField::ProposerPriority),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Validator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Validator")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Validator, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut pub_key__ = None;
                let mut voting_power__ = None;
                let mut proposer_priority__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PubKey => {
                            if pub_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubKey"));
                            }
                            pub_key__ = map.next_value()?;
                        }
                        GeneratedField::VotingPower => {
                            if voting_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPower"));
                            }
                            voting_power__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProposerPriority => {
                            if proposer_priority__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposerPriority"));
                            }
                            proposer_priority__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Validator {
                    address: address__.unwrap_or_default(),
                    pub_key: pub_key__,
                    voting_power: voting_power__.unwrap_or_default(),
                    proposer_priority: proposer_priority__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Validator", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.pub_key_types.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.ValidatorParams", len)?;
        if !self.pub_key_types.is_empty() {
            struct_ser.serialize_field("pubKeyTypes", &self.pub_key_types)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pub_key_types",
            "pubKeyTypes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PubKeyTypes,
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
                            "pubKeyTypes" | "pub_key_types" => Ok(GeneratedField::PubKeyTypes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.ValidatorParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pub_key_types__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PubKeyTypes => {
                            if pub_key_types__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubKeyTypes"));
                            }
                            pub_key_types__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValidatorParams {
                    pub_key_types: pub_key_types__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.ValidatorParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorSet {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validators.is_empty() {
            len += 1;
        }
        if self.proposer.is_some() {
            len += 1;
        }
        if self.total_voting_power != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.ValidatorSet", len)?;
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
        }
        if let Some(v) = self.proposer.as_ref() {
            struct_ser.serialize_field("proposer", v)?;
        }
        if self.total_voting_power != 0 {
            struct_ser.serialize_field("totalVotingPower", ToString::to_string(&self.total_voting_power).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorSet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validators",
            "proposer",
            "total_voting_power",
            "totalVotingPower",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Validators,
            Proposer,
            TotalVotingPower,
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
                            "validators" => Ok(GeneratedField::Validators),
                            "proposer" => Ok(GeneratedField::Proposer),
                            "totalVotingPower" | "total_voting_power" => Ok(GeneratedField::TotalVotingPower),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorSet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.ValidatorSet")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorSet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validators__ = None;
                let mut proposer__ = None;
                let mut total_voting_power__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proposer => {
                            if proposer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposer"));
                            }
                            proposer__ = map.next_value()?;
                        }
                        GeneratedField::TotalVotingPower => {
                            if total_voting_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalVotingPower"));
                            }
                            total_voting_power__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ValidatorSet {
                    validators: validators__.unwrap_or_default(),
                    proposer: proposer__,
                    total_voting_power: total_voting_power__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.ValidatorSet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VersionParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.app != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.VersionParams", len)?;
        if self.app != 0 {
            struct_ser.serialize_field("app", ToString::to_string(&self.app).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VersionParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "app",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = VersionParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.VersionParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<VersionParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut app__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
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
                Ok(VersionParams {
                    app: app__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.VersionParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Vote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type != 0 {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.round != 0 {
            len += 1;
        }
        if self.block_id.is_some() {
            len += 1;
        }
        if self.timestamp.is_some() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.validator_index != 0 {
            len += 1;
        }
        if !self.signature.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.types.Vote", len)?;
        if self.r#type != 0 {
            let v = SignedMsgType::from_i32(self.r#type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.round != 0 {
            struct_ser.serialize_field("round", &self.round)?;
        }
        if let Some(v) = self.block_id.as_ref() {
            struct_ser.serialize_field("blockId", v)?;
        }
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", pbjson::private::base64::encode(&self.validator_address).as_str())?;
        }
        if self.validator_index != 0 {
            struct_ser.serialize_field("validatorIndex", &self.validator_index)?;
        }
        if !self.signature.is_empty() {
            struct_ser.serialize_field("signature", pbjson::private::base64::encode(&self.signature).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Vote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "height",
            "round",
            "block_id",
            "blockId",
            "timestamp",
            "validator_address",
            "validatorAddress",
            "validator_index",
            "validatorIndex",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Height,
            Round,
            BlockId,
            Timestamp,
            ValidatorAddress,
            ValidatorIndex,
            Signature,
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
                            "height" => Ok(GeneratedField::Height),
                            "round" => Ok(GeneratedField::Round),
                            "blockId" | "block_id" => Ok(GeneratedField::BlockId),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "validatorIndex" | "validator_index" => Ok(GeneratedField::ValidatorIndex),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Vote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.types.Vote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Vote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut height__ = None;
                let mut round__ = None;
                let mut block_id__ = None;
                let mut timestamp__ = None;
                let mut validator_address__ = None;
                let mut validator_index__ = None;
                let mut signature__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value::<SignedMsgType>()? as i32);
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Round => {
                            if round__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round"));
                            }
                            round__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BlockId => {
                            if block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockId"));
                            }
                            block_id__ = map.next_value()?;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map.next_value()?;
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValidatorIndex => {
                            if validator_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorIndex"));
                            }
                            validator_index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Vote {
                    r#type: r#type__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    round: round__.unwrap_or_default(),
                    block_id: block_id__,
                    timestamp: timestamp__,
                    validator_address: validator_address__.unwrap_or_default(),
                    validator_index: validator_index__.unwrap_or_default(),
                    signature: signature__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.types.Vote", FIELDS, GeneratedVisitor)
    }
}
