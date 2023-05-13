// @generated
impl serde::Serialize for CosmosTx {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.messages.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.v1.CosmosTx", len)?;
        if !self.messages.is_empty() {
            struct_ser.serialize_field("messages", &self.messages)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CosmosTx {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "messages",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Messages,
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
                            "messages" => Ok(GeneratedField::Messages),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CosmosTx;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.interchain_accounts.v1.CosmosTx")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CosmosTx, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut messages__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Messages => {
                            if messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messages"));
                            }
                            messages__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CosmosTx {
                    messages: messages__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.v1.CosmosTx", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InterchainAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_account.is_some() {
            len += 1;
        }
        if !self.account_owner.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.v1.InterchainAccount", len)?;
        if let Some(v) = self.base_account.as_ref() {
            struct_ser.serialize_field("baseAccount", v)?;
        }
        if !self.account_owner.is_empty() {
            struct_ser.serialize_field("accountOwner", &self.account_owner)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InterchainAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_account",
            "baseAccount",
            "account_owner",
            "accountOwner",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseAccount,
            AccountOwner,
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
                            "baseAccount" | "base_account" => Ok(GeneratedField::BaseAccount),
                            "accountOwner" | "account_owner" => Ok(GeneratedField::AccountOwner),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InterchainAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.interchain_accounts.v1.InterchainAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<InterchainAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_account__ = None;
                let mut account_owner__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BaseAccount => {
                            if base_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseAccount"));
                            }
                            base_account__ = map.next_value()?;
                        }
                        GeneratedField::AccountOwner => {
                            if account_owner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountOwner"));
                            }
                            account_owner__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(InterchainAccount {
                    base_account: base_account__,
                    account_owner: account_owner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.v1.InterchainAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InterchainAccountPacketData {
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
        if !self.data.is_empty() {
            len += 1;
        }
        if !self.memo.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.v1.InterchainAccountPacketData", len)?;
        if self.r#type != 0 {
            let v = Type::from_i32(self.r#type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        if !self.memo.is_empty() {
            struct_ser.serialize_field("memo", &self.memo)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InterchainAccountPacketData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "data",
            "memo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Data,
            Memo,
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
                            "data" => Ok(GeneratedField::Data),
                            "memo" => Ok(GeneratedField::Memo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InterchainAccountPacketData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.interchain_accounts.v1.InterchainAccountPacketData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<InterchainAccountPacketData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut data__ = None;
                let mut memo__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value::<Type>()? as i32);
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Memo => {
                            if memo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memo"));
                            }
                            memo__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(InterchainAccountPacketData {
                    r#type: r#type__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                    memo: memo__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.v1.InterchainAccountPacketData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Metadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.version.is_empty() {
            len += 1;
        }
        if !self.controller_connection_id.is_empty() {
            len += 1;
        }
        if !self.host_connection_id.is_empty() {
            len += 1;
        }
        if !self.address.is_empty() {
            len += 1;
        }
        if !self.encoding.is_empty() {
            len += 1;
        }
        if !self.tx_type.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.v1.Metadata", len)?;
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if !self.controller_connection_id.is_empty() {
            struct_ser.serialize_field("controllerConnectionId", &self.controller_connection_id)?;
        }
        if !self.host_connection_id.is_empty() {
            struct_ser.serialize_field("hostConnectionId", &self.host_connection_id)?;
        }
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if !self.encoding.is_empty() {
            struct_ser.serialize_field("encoding", &self.encoding)?;
        }
        if !self.tx_type.is_empty() {
            struct_ser.serialize_field("txType", &self.tx_type)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Metadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "version",
            "controller_connection_id",
            "controllerConnectionId",
            "host_connection_id",
            "hostConnectionId",
            "address",
            "encoding",
            "tx_type",
            "txType",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Version,
            ControllerConnectionId,
            HostConnectionId,
            Address,
            Encoding,
            TxType,
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
                            "controllerConnectionId" | "controller_connection_id" => Ok(GeneratedField::ControllerConnectionId),
                            "hostConnectionId" | "host_connection_id" => Ok(GeneratedField::HostConnectionId),
                            "address" => Ok(GeneratedField::Address),
                            "encoding" => Ok(GeneratedField::Encoding),
                            "txType" | "tx_type" => Ok(GeneratedField::TxType),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Metadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.interchain_accounts.v1.Metadata")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Metadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut version__ = None;
                let mut controller_connection_id__ = None;
                let mut host_connection_id__ = None;
                let mut address__ = None;
                let mut encoding__ = None;
                let mut tx_type__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                        GeneratedField::ControllerConnectionId => {
                            if controller_connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("controllerConnectionId"));
                            }
                            controller_connection_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::HostConnectionId => {
                            if host_connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hostConnectionId"));
                            }
                            host_connection_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Encoding => {
                            if encoding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encoding"));
                            }
                            encoding__ = Some(map.next_value()?);
                        }
                        GeneratedField::TxType => {
                            if tx_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txType"));
                            }
                            tx_type__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Metadata {
                    version: version__.unwrap_or_default(),
                    controller_connection_id: controller_connection_id__.unwrap_or_default(),
                    host_connection_id: host_connection_id__.unwrap_or_default(),
                    address: address__.unwrap_or_default(),
                    encoding: encoding__.unwrap_or_default(),
                    tx_type: tx_type__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.v1.Metadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Type {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "TYPE_UNSPECIFIED",
            Self::ExecuteTx => "TYPE_EXECUTE_TX",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Type {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TYPE_UNSPECIFIED",
            "TYPE_EXECUTE_TX",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Type;

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
                    .and_then(Type::from_i32)
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
                    .and_then(Type::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "TYPE_UNSPECIFIED" => Ok(Type::Unspecified),
                    "TYPE_EXECUTE_TX" => Ok(Type::ExecuteTx),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
