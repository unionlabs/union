// @generated
impl serde::Serialize for AbciQueryRequest {
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
        if !self.path.is_empty() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.prove {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.ABCIQueryRequest", len)?;
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.prove {
            struct_ser.serialize_field("prove", &self.prove)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AbciQueryRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
            "path",
            "height",
            "prove",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
            Path,
            Height,
            Prove,
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
                            "path" => Ok(GeneratedField::Path),
                            "height" => Ok(GeneratedField::Height),
                            "prove" => Ok(GeneratedField::Prove),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AbciQueryRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.ABCIQueryRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AbciQueryRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                let mut path__ = None;
                let mut height__ = None;
                let mut prove__ = None;
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
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map.next_value()?);
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Prove => {
                            if prove__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prove"));
                            }
                            prove__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AbciQueryRequest {
                    data: data__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    prove: prove__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.ABCIQueryRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AbciQueryResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.code != 0 {
            len += 1;
        }
        if !self.log.is_empty() {
            len += 1;
        }
        if !self.info.is_empty() {
            len += 1;
        }
        if self.index != 0 {
            len += 1;
        }
        if !self.key.is_empty() {
            len += 1;
        }
        if !self.value.is_empty() {
            len += 1;
        }
        if self.proof_ops.is_some() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if !self.codespace.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.ABCIQueryResponse", len)?;
        if self.code != 0 {
            struct_ser.serialize_field("code", &self.code)?;
        }
        if !self.log.is_empty() {
            struct_ser.serialize_field("log", &self.log)?;
        }
        if !self.info.is_empty() {
            struct_ser.serialize_field("info", &self.info)?;
        }
        if self.index != 0 {
            struct_ser.serialize_field("index", ToString::to_string(&self.index).as_str())?;
        }
        if !self.key.is_empty() {
            struct_ser.serialize_field("key", pbjson::private::base64::encode(&self.key).as_str())?;
        }
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", pbjson::private::base64::encode(&self.value).as_str())?;
        }
        if let Some(v) = self.proof_ops.as_ref() {
            struct_ser.serialize_field("proofOps", v)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if !self.codespace.is_empty() {
            struct_ser.serialize_field("codespace", &self.codespace)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AbciQueryResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "code",
            "log",
            "info",
            "index",
            "key",
            "value",
            "proof_ops",
            "proofOps",
            "height",
            "codespace",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Code,
            Log,
            Info,
            Index,
            Key,
            Value,
            ProofOps,
            Height,
            Codespace,
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
                            "code" => Ok(GeneratedField::Code),
                            "log" => Ok(GeneratedField::Log),
                            "info" => Ok(GeneratedField::Info),
                            "index" => Ok(GeneratedField::Index),
                            "key" => Ok(GeneratedField::Key),
                            "value" => Ok(GeneratedField::Value),
                            "proofOps" | "proof_ops" => Ok(GeneratedField::ProofOps),
                            "height" => Ok(GeneratedField::Height),
                            "codespace" => Ok(GeneratedField::Codespace),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AbciQueryResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.ABCIQueryResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AbciQueryResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut code__ = None;
                let mut log__ = None;
                let mut info__ = None;
                let mut index__ = None;
                let mut key__ = None;
                let mut value__ = None;
                let mut proof_ops__ = None;
                let mut height__ = None;
                let mut codespace__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Code => {
                            if code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("code"));
                            }
                            code__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Log => {
                            if log__.is_some() {
                                return Err(serde::de::Error::duplicate_field("log"));
                            }
                            log__ = Some(map.next_value()?);
                        }
                        GeneratedField::Info => {
                            if info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("info"));
                            }
                            info__ = Some(map.next_value()?);
                        }
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofOps => {
                            if proof_ops__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofOps"));
                            }
                            proof_ops__ = map.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Codespace => {
                            if codespace__.is_some() {
                                return Err(serde::de::Error::duplicate_field("codespace"));
                            }
                            codespace__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AbciQueryResponse {
                    code: code__.unwrap_or_default(),
                    log: log__.unwrap_or_default(),
                    info: info__.unwrap_or_default(),
                    index: index__.unwrap_or_default(),
                    key: key__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                    proof_ops: proof_ops__,
                    height: height__.unwrap_or_default(),
                    codespace: codespace__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.ABCIQueryResponse", FIELDS, GeneratedVisitor)
    }
}
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
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.Block", len)?;
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
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.Block")
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
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.Block", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetBlockByHeightRequest {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetBlockByHeightRequest", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetBlockByHeightRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetBlockByHeightRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetBlockByHeightRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetBlockByHeightRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
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
                    }
                }
                Ok(GetBlockByHeightRequest {
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetBlockByHeightRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetBlockByHeightResponse {
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
        if self.block.is_some() {
            len += 1;
        }
        if self.sdk_block.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetBlockByHeightResponse", len)?;
        if let Some(v) = self.block_id.as_ref() {
            struct_ser.serialize_field("blockId", v)?;
        }
        if let Some(v) = self.block.as_ref() {
            struct_ser.serialize_field("block", v)?;
        }
        if let Some(v) = self.sdk_block.as_ref() {
            struct_ser.serialize_field("sdkBlock", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetBlockByHeightResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_id",
            "blockId",
            "block",
            "sdk_block",
            "sdkBlock",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockId,
            Block,
            SdkBlock,
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
                            "block" => Ok(GeneratedField::Block),
                            "sdkBlock" | "sdk_block" => Ok(GeneratedField::SdkBlock),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetBlockByHeightResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetBlockByHeightResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetBlockByHeightResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_id__ = None;
                let mut block__ = None;
                let mut sdk_block__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockId => {
                            if block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockId"));
                            }
                            block_id__ = map.next_value()?;
                        }
                        GeneratedField::Block => {
                            if block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("block"));
                            }
                            block__ = map.next_value()?;
                        }
                        GeneratedField::SdkBlock => {
                            if sdk_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sdkBlock"));
                            }
                            sdk_block__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetBlockByHeightResponse {
                    block_id: block_id__,
                    block: block__,
                    sdk_block: sdk_block__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetBlockByHeightResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestBlockRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetLatestBlockRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestBlockRequest {
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
            type Value = GetLatestBlockRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetLatestBlockRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetLatestBlockRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetLatestBlockRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetLatestBlockRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestBlockResponse {
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
        if self.block.is_some() {
            len += 1;
        }
        if self.sdk_block.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetLatestBlockResponse", len)?;
        if let Some(v) = self.block_id.as_ref() {
            struct_ser.serialize_field("blockId", v)?;
        }
        if let Some(v) = self.block.as_ref() {
            struct_ser.serialize_field("block", v)?;
        }
        if let Some(v) = self.sdk_block.as_ref() {
            struct_ser.serialize_field("sdkBlock", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestBlockResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_id",
            "blockId",
            "block",
            "sdk_block",
            "sdkBlock",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockId,
            Block,
            SdkBlock,
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
                            "block" => Ok(GeneratedField::Block),
                            "sdkBlock" | "sdk_block" => Ok(GeneratedField::SdkBlock),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetLatestBlockResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetLatestBlockResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetLatestBlockResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_id__ = None;
                let mut block__ = None;
                let mut sdk_block__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockId => {
                            if block_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockId"));
                            }
                            block_id__ = map.next_value()?;
                        }
                        GeneratedField::Block => {
                            if block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("block"));
                            }
                            block__ = map.next_value()?;
                        }
                        GeneratedField::SdkBlock => {
                            if sdk_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sdkBlock"));
                            }
                            sdk_block__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetLatestBlockResponse {
                    block_id: block_id__,
                    block: block__,
                    sdk_block: sdk_block__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetLatestBlockResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestValidatorSetRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetLatestValidatorSetRequest", len)?;
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestValidatorSetRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = GetLatestValidatorSetRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetLatestValidatorSetRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetLatestValidatorSetRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetLatestValidatorSetRequest {
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetLatestValidatorSetRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetLatestValidatorSetResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_height != 0 {
            len += 1;
        }
        if !self.validators.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetLatestValidatorSetResponse", len)?;
        if self.block_height != 0 {
            struct_ser.serialize_field("blockHeight", ToString::to_string(&self.block_height).as_str())?;
        }
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetLatestValidatorSetResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_height",
            "blockHeight",
            "validators",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockHeight,
            Validators,
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
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            "validators" => Ok(GeneratedField::Validators),
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
            type Value = GetLatestValidatorSetResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetLatestValidatorSetResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetLatestValidatorSetResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_height__ = None;
                let mut validators__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetLatestValidatorSetResponse {
                    block_height: block_height__.unwrap_or_default(),
                    validators: validators__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetLatestValidatorSetResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetNodeInfoRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetNodeInfoRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetNodeInfoRequest {
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
            type Value = GetNodeInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetNodeInfoRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetNodeInfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetNodeInfoRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetNodeInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetNodeInfoResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.default_node_info.is_some() {
            len += 1;
        }
        if self.application_version.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetNodeInfoResponse", len)?;
        if let Some(v) = self.default_node_info.as_ref() {
            struct_ser.serialize_field("defaultNodeInfo", v)?;
        }
        if let Some(v) = self.application_version.as_ref() {
            struct_ser.serialize_field("applicationVersion", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetNodeInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "default_node_info",
            "defaultNodeInfo",
            "application_version",
            "applicationVersion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DefaultNodeInfo,
            ApplicationVersion,
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
                            "defaultNodeInfo" | "default_node_info" => Ok(GeneratedField::DefaultNodeInfo),
                            "applicationVersion" | "application_version" => Ok(GeneratedField::ApplicationVersion),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetNodeInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetNodeInfoResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetNodeInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut default_node_info__ = None;
                let mut application_version__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DefaultNodeInfo => {
                            if default_node_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("defaultNodeInfo"));
                            }
                            default_node_info__ = map.next_value()?;
                        }
                        GeneratedField::ApplicationVersion => {
                            if application_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("applicationVersion"));
                            }
                            application_version__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetNodeInfoResponse {
                    default_node_info: default_node_info__,
                    application_version: application_version__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetNodeInfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetSyncingRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetSyncingRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetSyncingRequest {
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
            type Value = GetSyncingRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetSyncingRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetSyncingRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetSyncingRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetSyncingRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetSyncingResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.syncing {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetSyncingResponse", len)?;
        if self.syncing {
            struct_ser.serialize_field("syncing", &self.syncing)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetSyncingResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "syncing",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Syncing,
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
                            "syncing" => Ok(GeneratedField::Syncing),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetSyncingResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetSyncingResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetSyncingResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut syncing__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Syncing => {
                            if syncing__.is_some() {
                                return Err(serde::de::Error::duplicate_field("syncing"));
                            }
                            syncing__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GetSyncingResponse {
                    syncing: syncing__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetSyncingResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetValidatorSetByHeightRequest {
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
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetValidatorSetByHeightRequest", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetValidatorSetByHeightRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
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
                            "height" => Ok(GeneratedField::Height),
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
            type Value = GetValidatorSetByHeightRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetValidatorSetByHeightRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetValidatorSetByHeightRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut pagination__ = None;
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
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetValidatorSetByHeightRequest {
                    height: height__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetValidatorSetByHeightRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetValidatorSetByHeightResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_height != 0 {
            len += 1;
        }
        if !self.validators.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.GetValidatorSetByHeightResponse", len)?;
        if self.block_height != 0 {
            struct_ser.serialize_field("blockHeight", ToString::to_string(&self.block_height).as_str())?;
        }
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetValidatorSetByHeightResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_height",
            "blockHeight",
            "validators",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockHeight,
            Validators,
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
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            "validators" => Ok(GeneratedField::Validators),
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
            type Value = GetValidatorSetByHeightResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.GetValidatorSetByHeightResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetValidatorSetByHeightResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_height__ = None;
                let mut validators__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetValidatorSetByHeightResponse {
                    block_height: block_height__.unwrap_or_default(),
                    validators: validators__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.GetValidatorSetByHeightResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.Header", len)?;
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
            struct_ser.serialize_field("proposerAddress", &self.proposer_address)?;
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
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.Header")
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
                            proposer_address__ = Some(map.next_value()?);
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
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.Header", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Module {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.path.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        if !self.sum.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.Module", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if !self.sum.is_empty() {
            struct_ser.serialize_field("sum", &self.sum)?;
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
            "path",
            "version",
            "sum",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            Version,
            Sum,
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
                            "version" => Ok(GeneratedField::Version),
                            "sum" => Ok(GeneratedField::Sum),
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
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.Module")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Module, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut version__ = None;
                let mut sum__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                        GeneratedField::Sum => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sum"));
                            }
                            sum__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Module {
                    path: path__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    sum: sum__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.Module", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProofOp {
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
        if !self.key.is_empty() {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.ProofOp", len)?;
        if !self.r#type.is_empty() {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if !self.key.is_empty() {
            struct_ser.serialize_field("key", pbjson::private::base64::encode(&self.key).as_str())?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProofOp {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "key",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Key,
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
                            "type" => Ok(GeneratedField::Type),
                            "key" => Ok(GeneratedField::Key),
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
            type Value = ProofOp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.ProofOp")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProofOp, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut key__ = None;
                let mut data__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value()?);
                        }
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = 
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
                    }
                }
                Ok(ProofOp {
                    r#type: r#type__.unwrap_or_default(),
                    key: key__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.ProofOp", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProofOps {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.ops.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.ProofOps", len)?;
        if !self.ops.is_empty() {
            struct_ser.serialize_field("ops", &self.ops)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProofOps {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ops",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Ops,
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
                            "ops" => Ok(GeneratedField::Ops),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProofOps;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.ProofOps")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProofOps, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ops__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Ops => {
                            if ops__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ops"));
                            }
                            ops__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ProofOps {
                    ops: ops__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.ProofOps", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.Validator", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
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
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.Validator")
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
                            address__ = Some(map.next_value()?);
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
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.Validator", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VersionInfo {
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
        if !self.app_name.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        if !self.git_commit.is_empty() {
            len += 1;
        }
        if !self.build_tags.is_empty() {
            len += 1;
        }
        if !self.go_version.is_empty() {
            len += 1;
        }
        if !self.build_deps.is_empty() {
            len += 1;
        }
        if !self.cosmos_sdk_version.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.tendermint.v1beta1.VersionInfo", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.app_name.is_empty() {
            struct_ser.serialize_field("appName", &self.app_name)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if !self.git_commit.is_empty() {
            struct_ser.serialize_field("gitCommit", &self.git_commit)?;
        }
        if !self.build_tags.is_empty() {
            struct_ser.serialize_field("buildTags", &self.build_tags)?;
        }
        if !self.go_version.is_empty() {
            struct_ser.serialize_field("goVersion", &self.go_version)?;
        }
        if !self.build_deps.is_empty() {
            struct_ser.serialize_field("buildDeps", &self.build_deps)?;
        }
        if !self.cosmos_sdk_version.is_empty() {
            struct_ser.serialize_field("cosmosSdkVersion", &self.cosmos_sdk_version)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VersionInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "app_name",
            "appName",
            "version",
            "git_commit",
            "gitCommit",
            "build_tags",
            "buildTags",
            "go_version",
            "goVersion",
            "build_deps",
            "buildDeps",
            "cosmos_sdk_version",
            "cosmosSdkVersion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            AppName,
            Version,
            GitCommit,
            BuildTags,
            GoVersion,
            BuildDeps,
            CosmosSdkVersion,
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
                            "appName" | "app_name" => Ok(GeneratedField::AppName),
                            "version" => Ok(GeneratedField::Version),
                            "gitCommit" | "git_commit" => Ok(GeneratedField::GitCommit),
                            "buildTags" | "build_tags" => Ok(GeneratedField::BuildTags),
                            "goVersion" | "go_version" => Ok(GeneratedField::GoVersion),
                            "buildDeps" | "build_deps" => Ok(GeneratedField::BuildDeps),
                            "cosmosSdkVersion" | "cosmos_sdk_version" => Ok(GeneratedField::CosmosSdkVersion),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VersionInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.tendermint.v1beta1.VersionInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<VersionInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut app_name__ = None;
                let mut version__ = None;
                let mut git_commit__ = None;
                let mut build_tags__ = None;
                let mut go_version__ = None;
                let mut build_deps__ = None;
                let mut cosmos_sdk_version__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppName => {
                            if app_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appName"));
                            }
                            app_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                        GeneratedField::GitCommit => {
                            if git_commit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gitCommit"));
                            }
                            git_commit__ = Some(map.next_value()?);
                        }
                        GeneratedField::BuildTags => {
                            if build_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildTags"));
                            }
                            build_tags__ = Some(map.next_value()?);
                        }
                        GeneratedField::GoVersion => {
                            if go_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("goVersion"));
                            }
                            go_version__ = Some(map.next_value()?);
                        }
                        GeneratedField::BuildDeps => {
                            if build_deps__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildDeps"));
                            }
                            build_deps__ = Some(map.next_value()?);
                        }
                        GeneratedField::CosmosSdkVersion => {
                            if cosmos_sdk_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cosmosSdkVersion"));
                            }
                            cosmos_sdk_version__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(VersionInfo {
                    name: name__.unwrap_or_default(),
                    app_name: app_name__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    git_commit: git_commit__.unwrap_or_default(),
                    build_tags: build_tags__.unwrap_or_default(),
                    go_version: go_version__.unwrap_or_default(),
                    build_deps: build_deps__.unwrap_or_default(),
                    cosmos_sdk_version: cosmos_sdk_version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.tendermint.v1beta1.VersionInfo", FIELDS, GeneratedVisitor)
    }
}
