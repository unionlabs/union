// @generated
impl serde::Serialize for Allocation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.source_port.is_empty() {
            len += 1;
        }
        if !self.source_channel.is_empty() {
            len += 1;
        }
        if !self.spend_limit.is_empty() {
            len += 1;
        }
        if !self.allow_list.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.Allocation", len)?;
        if !self.source_port.is_empty() {
            struct_ser.serialize_field("sourcePort", &self.source_port)?;
        }
        if !self.source_channel.is_empty() {
            struct_ser.serialize_field("sourceChannel", &self.source_channel)?;
        }
        if !self.spend_limit.is_empty() {
            struct_ser.serialize_field("spendLimit", &self.spend_limit)?;
        }
        if !self.allow_list.is_empty() {
            struct_ser.serialize_field("allowList", &self.allow_list)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Allocation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "source_port",
            "sourcePort",
            "source_channel",
            "sourceChannel",
            "spend_limit",
            "spendLimit",
            "allow_list",
            "allowList",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SourcePort,
            SourceChannel,
            SpendLimit,
            AllowList,
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
                            "sourcePort" | "source_port" => Ok(GeneratedField::SourcePort),
                            "sourceChannel" | "source_channel" => Ok(GeneratedField::SourceChannel),
                            "spendLimit" | "spend_limit" => Ok(GeneratedField::SpendLimit),
                            "allowList" | "allow_list" => Ok(GeneratedField::AllowList),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Allocation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.Allocation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Allocation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut source_port__ = None;
                let mut source_channel__ = None;
                let mut spend_limit__ = None;
                let mut allow_list__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SourcePort => {
                            if source_port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourcePort"));
                            }
                            source_port__ = Some(map.next_value()?);
                        }
                        GeneratedField::SourceChannel => {
                            if source_channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceChannel"));
                            }
                            source_channel__ = Some(map.next_value()?);
                        }
                        GeneratedField::SpendLimit => {
                            if spend_limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendLimit"));
                            }
                            spend_limit__ = Some(map.next_value()?);
                        }
                        GeneratedField::AllowList => {
                            if allow_list__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowList"));
                            }
                            allow_list__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Allocation {
                    source_port: source_port__.unwrap_or_default(),
                    source_channel: source_channel__.unwrap_or_default(),
                    spend_limit: spend_limit__.unwrap_or_default(),
                    allow_list: allow_list__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.Allocation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DenomTrace {
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
        if !self.base_denom.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.DenomTrace", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if !self.base_denom.is_empty() {
            struct_ser.serialize_field("baseDenom", &self.base_denom)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DenomTrace {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "base_denom",
            "baseDenom",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            BaseDenom,
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
                            "baseDenom" | "base_denom" => Ok(GeneratedField::BaseDenom),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DenomTrace;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.DenomTrace")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DenomTrace, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut base_denom__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map.next_value()?);
                        }
                        GeneratedField::BaseDenom => {
                            if base_denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseDenom"));
                            }
                            base_denom__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DenomTrace {
                    path: path__.unwrap_or_default(),
                    base_denom: base_denom__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.DenomTrace", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenesisState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.port_id.is_empty() {
            len += 1;
        }
        if !self.denom_traces.is_empty() {
            len += 1;
        }
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.GenesisState", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.denom_traces.is_empty() {
            struct_ser.serialize_field("denomTraces", &self.denom_traces)?;
        }
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenesisState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "port_id",
            "portId",
            "denom_traces",
            "denomTraces",
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            DenomTraces,
            Params,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "denomTraces" | "denom_traces" => Ok(GeneratedField::DenomTraces),
                            "params" => Ok(GeneratedField::Params),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenesisState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut denom_traces__ = None;
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PortId => {
                            if port_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("portId"));
                            }
                            port_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::DenomTraces => {
                            if denom_traces__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denomTraces"));
                            }
                            denom_traces__ = Some(map.next_value()?);
                        }
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                    }
                }
                Ok(GenesisState {
                    port_id: port_id__.unwrap_or_default(),
                    denom_traces: denom_traces__.unwrap_or_default(),
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgTransfer {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.source_port.is_empty() {
            len += 1;
        }
        if !self.source_channel.is_empty() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        if !self.sender.is_empty() {
            len += 1;
        }
        if !self.receiver.is_empty() {
            len += 1;
        }
        if self.timeout_height.is_some() {
            len += 1;
        }
        if self.timeout_timestamp != 0 {
            len += 1;
        }
        if !self.memo.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.MsgTransfer", len)?;
        if !self.source_port.is_empty() {
            struct_ser.serialize_field("sourcePort", &self.source_port)?;
        }
        if !self.source_channel.is_empty() {
            struct_ser.serialize_field("sourceChannel", &self.source_channel)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        if !self.sender.is_empty() {
            struct_ser.serialize_field("sender", &self.sender)?;
        }
        if !self.receiver.is_empty() {
            struct_ser.serialize_field("receiver", &self.receiver)?;
        }
        if let Some(v) = self.timeout_height.as_ref() {
            struct_ser.serialize_field("timeoutHeight", v)?;
        }
        if self.timeout_timestamp != 0 {
            struct_ser.serialize_field("timeoutTimestamp", ToString::to_string(&self.timeout_timestamp).as_str())?;
        }
        if !self.memo.is_empty() {
            struct_ser.serialize_field("memo", &self.memo)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgTransfer {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "source_port",
            "sourcePort",
            "source_channel",
            "sourceChannel",
            "token",
            "sender",
            "receiver",
            "timeout_height",
            "timeoutHeight",
            "timeout_timestamp",
            "timeoutTimestamp",
            "memo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SourcePort,
            SourceChannel,
            Token,
            Sender,
            Receiver,
            TimeoutHeight,
            TimeoutTimestamp,
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
                            "sourcePort" | "source_port" => Ok(GeneratedField::SourcePort),
                            "sourceChannel" | "source_channel" => Ok(GeneratedField::SourceChannel),
                            "token" => Ok(GeneratedField::Token),
                            "sender" => Ok(GeneratedField::Sender),
                            "receiver" => Ok(GeneratedField::Receiver),
                            "timeoutHeight" | "timeout_height" => Ok(GeneratedField::TimeoutHeight),
                            "timeoutTimestamp" | "timeout_timestamp" => Ok(GeneratedField::TimeoutTimestamp),
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
            type Value = MsgTransfer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.MsgTransfer")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgTransfer, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut source_port__ = None;
                let mut source_channel__ = None;
                let mut token__ = None;
                let mut sender__ = None;
                let mut receiver__ = None;
                let mut timeout_height__ = None;
                let mut timeout_timestamp__ = None;
                let mut memo__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SourcePort => {
                            if source_port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourcePort"));
                            }
                            source_port__ = Some(map.next_value()?);
                        }
                        GeneratedField::SourceChannel => {
                            if source_channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceChannel"));
                            }
                            source_channel__ = Some(map.next_value()?);
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                        GeneratedField::Sender => {
                            if sender__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sender"));
                            }
                            sender__ = Some(map.next_value()?);
                        }
                        GeneratedField::Receiver => {
                            if receiver__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiver"));
                            }
                            receiver__ = Some(map.next_value()?);
                        }
                        GeneratedField::TimeoutHeight => {
                            if timeout_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutHeight"));
                            }
                            timeout_height__ = map.next_value()?;
                        }
                        GeneratedField::TimeoutTimestamp => {
                            if timeout_timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutTimestamp"));
                            }
                            timeout_timestamp__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
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
                Ok(MsgTransfer {
                    source_port: source_port__.unwrap_or_default(),
                    source_channel: source_channel__.unwrap_or_default(),
                    token: token__,
                    sender: sender__.unwrap_or_default(),
                    receiver: receiver__.unwrap_or_default(),
                    timeout_height: timeout_height__,
                    timeout_timestamp: timeout_timestamp__.unwrap_or_default(),
                    memo: memo__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.MsgTransfer", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgTransferResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.MsgTransferResponse", len)?;
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgTransferResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sequence,
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
                            "sequence" => Ok(GeneratedField::Sequence),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgTransferResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.MsgTransferResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgTransferResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sequence__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Sequence => {
                            if sequence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sequence"));
                            }
                            sequence__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgTransferResponse {
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.MsgTransferResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Params {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.send_enabled {
            len += 1;
        }
        if self.receive_enabled {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.Params", len)?;
        if self.send_enabled {
            struct_ser.serialize_field("sendEnabled", &self.send_enabled)?;
        }
        if self.receive_enabled {
            struct_ser.serialize_field("receiveEnabled", &self.receive_enabled)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Params {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "send_enabled",
            "sendEnabled",
            "receive_enabled",
            "receiveEnabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SendEnabled,
            ReceiveEnabled,
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
                            "sendEnabled" | "send_enabled" => Ok(GeneratedField::SendEnabled),
                            "receiveEnabled" | "receive_enabled" => Ok(GeneratedField::ReceiveEnabled),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Params;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.Params")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Params, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut send_enabled__ = None;
                let mut receive_enabled__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SendEnabled => {
                            if send_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sendEnabled"));
                            }
                            send_enabled__ = Some(map.next_value()?);
                        }
                        GeneratedField::ReceiveEnabled => {
                            if receive_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiveEnabled"));
                            }
                            receive_enabled__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Params {
                    send_enabled: send_enabled__.unwrap_or_default(),
                    receive_enabled: receive_enabled__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.Params", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDenomHashRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.trace.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryDenomHashRequest", len)?;
        if !self.trace.is_empty() {
            struct_ser.serialize_field("trace", &self.trace)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDenomHashRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trace",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Trace,
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
                            "trace" => Ok(GeneratedField::Trace),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDenomHashRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryDenomHashRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDenomHashRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trace__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Trace => {
                            if trace__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trace"));
                            }
                            trace__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDenomHashRequest {
                    trace: trace__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryDenomHashRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDenomHashResponse {
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
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryDenomHashResponse", len)?;
        if !self.hash.is_empty() {
            struct_ser.serialize_field("hash", &self.hash)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDenomHashResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "hash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = QueryDenomHashResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryDenomHashResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDenomHashResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Hash => {
                            if hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hash"));
                            }
                            hash__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDenomHashResponse {
                    hash: hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryDenomHashResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDenomTraceRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryDenomTraceRequest", len)?;
        if !self.hash.is_empty() {
            struct_ser.serialize_field("hash", &self.hash)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDenomTraceRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "hash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = QueryDenomTraceRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryDenomTraceRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDenomTraceRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Hash => {
                            if hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hash"));
                            }
                            hash__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDenomTraceRequest {
                    hash: hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryDenomTraceRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDenomTraceResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.denom_trace.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryDenomTraceResponse", len)?;
        if let Some(v) = self.denom_trace.as_ref() {
            struct_ser.serialize_field("denomTrace", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDenomTraceResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "denom_trace",
            "denomTrace",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DenomTrace,
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
                            "denomTrace" | "denom_trace" => Ok(GeneratedField::DenomTrace),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDenomTraceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryDenomTraceResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDenomTraceResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut denom_trace__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DenomTrace => {
                            if denom_trace__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denomTrace"));
                            }
                            denom_trace__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDenomTraceResponse {
                    denom_trace: denom_trace__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryDenomTraceResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDenomTracesRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryDenomTracesRequest", len)?;
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDenomTracesRequest {
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
            type Value = QueryDenomTracesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryDenomTracesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDenomTracesRequest, V::Error>
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
                Ok(QueryDenomTracesRequest {
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryDenomTracesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDenomTracesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.denom_traces.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryDenomTracesResponse", len)?;
        if !self.denom_traces.is_empty() {
            struct_ser.serialize_field("denomTraces", &self.denom_traces)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDenomTracesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "denom_traces",
            "denomTraces",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DenomTraces,
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
                            "denomTraces" | "denom_traces" => Ok(GeneratedField::DenomTraces),
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
            type Value = QueryDenomTracesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryDenomTracesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDenomTracesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut denom_traces__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DenomTraces => {
                            if denom_traces__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denomTraces"));
                            }
                            denom_traces__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDenomTracesResponse {
                    denom_traces: denom_traces__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryDenomTracesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryEscrowAddressRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.port_id.is_empty() {
            len += 1;
        }
        if !self.channel_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryEscrowAddressRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryEscrowAddressRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "port_id",
            "portId",
            "channel_id",
            "channelId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryEscrowAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryEscrowAddressRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryEscrowAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PortId => {
                            if port_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("portId"));
                            }
                            port_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ChannelId => {
                            if channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelId"));
                            }
                            channel_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryEscrowAddressRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryEscrowAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryEscrowAddressResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.escrow_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryEscrowAddressResponse", len)?;
        if !self.escrow_address.is_empty() {
            struct_ser.serialize_field("escrowAddress", &self.escrow_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryEscrowAddressResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "escrow_address",
            "escrowAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EscrowAddress,
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
                            "escrowAddress" | "escrow_address" => Ok(GeneratedField::EscrowAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryEscrowAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryEscrowAddressResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryEscrowAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut escrow_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::EscrowAddress => {
                            if escrow_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("escrowAddress"));
                            }
                            escrow_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryEscrowAddressResponse {
                    escrow_address: escrow_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryEscrowAddressResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryParamsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryParamsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryParamsRequest {
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
            type Value = QueryParamsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryParamsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryParamsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(QueryParamsRequest {
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryParamsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryParamsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.QueryParamsResponse", len)?;
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryParamsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Params,
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
                            "params" => Ok(GeneratedField::Params),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryParamsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.QueryParamsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryParamsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryParamsResponse {
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.QueryParamsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransferAuthorization {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.allocations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.transfer.v1.TransferAuthorization", len)?;
        if !self.allocations.is_empty() {
            struct_ser.serialize_field("allocations", &self.allocations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransferAuthorization {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "allocations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Allocations,
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
                            "allocations" => Ok(GeneratedField::Allocations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransferAuthorization;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.transfer.v1.TransferAuthorization")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransferAuthorization, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut allocations__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Allocations => {
                            if allocations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allocations"));
                            }
                            allocations__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TransferAuthorization {
                    allocations: allocations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.transfer.v1.TransferAuthorization", FIELDS, GeneratedVisitor)
    }
}
