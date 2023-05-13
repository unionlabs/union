// @generated
impl serde::Serialize for ChannelStateData {
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
        if self.channel.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.ChannelStateData", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", pbjson::private::base64::encode(&self.path).as_str())?;
        }
        if let Some(v) = self.channel.as_ref() {
            struct_ser.serialize_field("channel", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChannelStateData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "channel",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            Channel,
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
                            "channel" => Ok(GeneratedField::Channel),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChannelStateData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.ChannelStateData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChannelStateData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut channel__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Channel => {
                            if channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channel"));
                            }
                            channel__ = map.next_value()?;
                        }
                    }
                }
                Ok(ChannelStateData {
                    path: path__.unwrap_or_default(),
                    channel: channel__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.ChannelStateData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ClientState {
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
        if self.is_frozen {
            len += 1;
        }
        if self.consensus_state.is_some() {
            len += 1;
        }
        if self.allow_update_after_proposal {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.ClientState", len)?;
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        if self.is_frozen {
            struct_ser.serialize_field("isFrozen", &self.is_frozen)?;
        }
        if let Some(v) = self.consensus_state.as_ref() {
            struct_ser.serialize_field("consensusState", v)?;
        }
        if self.allow_update_after_proposal {
            struct_ser.serialize_field("allowUpdateAfterProposal", &self.allow_update_after_proposal)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sequence",
            "is_frozen",
            "isFrozen",
            "consensus_state",
            "consensusState",
            "allow_update_after_proposal",
            "allowUpdateAfterProposal",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sequence,
            IsFrozen,
            ConsensusState,
            AllowUpdateAfterProposal,
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
                            "isFrozen" | "is_frozen" => Ok(GeneratedField::IsFrozen),
                            "consensusState" | "consensus_state" => Ok(GeneratedField::ConsensusState),
                            "allowUpdateAfterProposal" | "allow_update_after_proposal" => Ok(GeneratedField::AllowUpdateAfterProposal),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.ClientState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ClientState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sequence__ = None;
                let mut is_frozen__ = None;
                let mut consensus_state__ = None;
                let mut allow_update_after_proposal__ = None;
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
                        GeneratedField::IsFrozen => {
                            if is_frozen__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isFrozen"));
                            }
                            is_frozen__ = Some(map.next_value()?);
                        }
                        GeneratedField::ConsensusState => {
                            if consensus_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusState"));
                            }
                            consensus_state__ = map.next_value()?;
                        }
                        GeneratedField::AllowUpdateAfterProposal => {
                            if allow_update_after_proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowUpdateAfterProposal"));
                            }
                            allow_update_after_proposal__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ClientState {
                    sequence: sequence__.unwrap_or_default(),
                    is_frozen: is_frozen__.unwrap_or_default(),
                    consensus_state: consensus_state__,
                    allow_update_after_proposal: allow_update_after_proposal__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.ClientState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ClientStateData {
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
        if self.client_state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.ClientStateData", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", pbjson::private::base64::encode(&self.path).as_str())?;
        }
        if let Some(v) = self.client_state.as_ref() {
            struct_ser.serialize_field("clientState", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientStateData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "client_state",
            "clientState",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            ClientState,
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
                            "clientState" | "client_state" => Ok(GeneratedField::ClientState),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientStateData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.ClientStateData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ClientStateData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut client_state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ClientState => {
                            if client_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientState"));
                            }
                            client_state__ = map.next_value()?;
                        }
                    }
                }
                Ok(ClientStateData {
                    path: path__.unwrap_or_default(),
                    client_state: client_state__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.ClientStateData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConnectionStateData {
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
        if self.connection.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.ConnectionStateData", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", pbjson::private::base64::encode(&self.path).as_str())?;
        }
        if let Some(v) = self.connection.as_ref() {
            struct_ser.serialize_field("connection", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConnectionStateData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "connection",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            Connection,
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
                            "connection" => Ok(GeneratedField::Connection),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConnectionStateData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.ConnectionStateData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConnectionStateData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut connection__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Connection => {
                            if connection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connection"));
                            }
                            connection__ = map.next_value()?;
                        }
                    }
                }
                Ok(ConnectionStateData {
                    path: path__.unwrap_or_default(),
                    connection: connection__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.ConnectionStateData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConsensusState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.public_key.is_some() {
            len += 1;
        }
        if !self.diversifier.is_empty() {
            len += 1;
        }
        if self.timestamp != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.ConsensusState", len)?;
        if let Some(v) = self.public_key.as_ref() {
            struct_ser.serialize_field("publicKey", v)?;
        }
        if !self.diversifier.is_empty() {
            struct_ser.serialize_field("diversifier", &self.diversifier)?;
        }
        if self.timestamp != 0 {
            struct_ser.serialize_field("timestamp", ToString::to_string(&self.timestamp).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConsensusState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "public_key",
            "publicKey",
            "diversifier",
            "timestamp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PublicKey,
            Diversifier,
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
                            "publicKey" | "public_key" => Ok(GeneratedField::PublicKey),
                            "diversifier" => Ok(GeneratedField::Diversifier),
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
            type Value = ConsensusState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.ConsensusState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConsensusState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut public_key__ = None;
                let mut diversifier__ = None;
                let mut timestamp__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PublicKey => {
                            if public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("publicKey"));
                            }
                            public_key__ = map.next_value()?;
                        }
                        GeneratedField::Diversifier => {
                            if diversifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("diversifier"));
                            }
                            diversifier__ = Some(map.next_value()?);
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ConsensusState {
                    public_key: public_key__,
                    diversifier: diversifier__.unwrap_or_default(),
                    timestamp: timestamp__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.ConsensusState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConsensusStateData {
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
        if self.consensus_state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.ConsensusStateData", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", pbjson::private::base64::encode(&self.path).as_str())?;
        }
        if let Some(v) = self.consensus_state.as_ref() {
            struct_ser.serialize_field("consensusState", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConsensusStateData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "consensus_state",
            "consensusState",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            ConsensusState,
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
                            "consensusState" | "consensus_state" => Ok(GeneratedField::ConsensusState),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConsensusStateData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.ConsensusStateData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConsensusStateData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut consensus_state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ConsensusState => {
                            if consensus_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusState"));
                            }
                            consensus_state__ = map.next_value()?;
                        }
                    }
                }
                Ok(ConsensusStateData {
                    path: path__.unwrap_or_default(),
                    consensus_state: consensus_state__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.ConsensusStateData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DataType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::UninitializedUnspecified => "DATA_TYPE_UNINITIALIZED_UNSPECIFIED",
            Self::ClientState => "DATA_TYPE_CLIENT_STATE",
            Self::ConsensusState => "DATA_TYPE_CONSENSUS_STATE",
            Self::ConnectionState => "DATA_TYPE_CONNECTION_STATE",
            Self::ChannelState => "DATA_TYPE_CHANNEL_STATE",
            Self::PacketCommitment => "DATA_TYPE_PACKET_COMMITMENT",
            Self::PacketAcknowledgement => "DATA_TYPE_PACKET_ACKNOWLEDGEMENT",
            Self::PacketReceiptAbsence => "DATA_TYPE_PACKET_RECEIPT_ABSENCE",
            Self::NextSequenceRecv => "DATA_TYPE_NEXT_SEQUENCE_RECV",
            Self::Header => "DATA_TYPE_HEADER",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for DataType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "DATA_TYPE_UNINITIALIZED_UNSPECIFIED",
            "DATA_TYPE_CLIENT_STATE",
            "DATA_TYPE_CONSENSUS_STATE",
            "DATA_TYPE_CONNECTION_STATE",
            "DATA_TYPE_CHANNEL_STATE",
            "DATA_TYPE_PACKET_COMMITMENT",
            "DATA_TYPE_PACKET_ACKNOWLEDGEMENT",
            "DATA_TYPE_PACKET_RECEIPT_ABSENCE",
            "DATA_TYPE_NEXT_SEQUENCE_RECV",
            "DATA_TYPE_HEADER",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DataType;

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
                    .and_then(DataType::from_i32)
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
                    .and_then(DataType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "DATA_TYPE_UNINITIALIZED_UNSPECIFIED" => Ok(DataType::UninitializedUnspecified),
                    "DATA_TYPE_CLIENT_STATE" => Ok(DataType::ClientState),
                    "DATA_TYPE_CONSENSUS_STATE" => Ok(DataType::ConsensusState),
                    "DATA_TYPE_CONNECTION_STATE" => Ok(DataType::ConnectionState),
                    "DATA_TYPE_CHANNEL_STATE" => Ok(DataType::ChannelState),
                    "DATA_TYPE_PACKET_COMMITMENT" => Ok(DataType::PacketCommitment),
                    "DATA_TYPE_PACKET_ACKNOWLEDGEMENT" => Ok(DataType::PacketAcknowledgement),
                    "DATA_TYPE_PACKET_RECEIPT_ABSENCE" => Ok(DataType::PacketReceiptAbsence),
                    "DATA_TYPE_NEXT_SEQUENCE_RECV" => Ok(DataType::NextSequenceRecv),
                    "DATA_TYPE_HEADER" => Ok(DataType::Header),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
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
        if self.sequence != 0 {
            len += 1;
        }
        if self.timestamp != 0 {
            len += 1;
        }
        if !self.signature.is_empty() {
            len += 1;
        }
        if self.new_public_key.is_some() {
            len += 1;
        }
        if !self.new_diversifier.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.Header", len)?;
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        if self.timestamp != 0 {
            struct_ser.serialize_field("timestamp", ToString::to_string(&self.timestamp).as_str())?;
        }
        if !self.signature.is_empty() {
            struct_ser.serialize_field("signature", pbjson::private::base64::encode(&self.signature).as_str())?;
        }
        if let Some(v) = self.new_public_key.as_ref() {
            struct_ser.serialize_field("newPublicKey", v)?;
        }
        if !self.new_diversifier.is_empty() {
            struct_ser.serialize_field("newDiversifier", &self.new_diversifier)?;
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
            "sequence",
            "timestamp",
            "signature",
            "new_public_key",
            "newPublicKey",
            "new_diversifier",
            "newDiversifier",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sequence,
            Timestamp,
            Signature,
            NewPublicKey,
            NewDiversifier,
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
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "signature" => Ok(GeneratedField::Signature),
                            "newPublicKey" | "new_public_key" => Ok(GeneratedField::NewPublicKey),
                            "newDiversifier" | "new_diversifier" => Ok(GeneratedField::NewDiversifier),
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
                formatter.write_str("struct ibc.lightclients.solomachine.v2.Header")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Header, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sequence__ = None;
                let mut timestamp__ = None;
                let mut signature__ = None;
                let mut new_public_key__ = None;
                let mut new_diversifier__ = None;
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
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = 
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
                        GeneratedField::NewPublicKey => {
                            if new_public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newPublicKey"));
                            }
                            new_public_key__ = map.next_value()?;
                        }
                        GeneratedField::NewDiversifier => {
                            if new_diversifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newDiversifier"));
                            }
                            new_diversifier__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Header {
                    sequence: sequence__.unwrap_or_default(),
                    timestamp: timestamp__.unwrap_or_default(),
                    signature: signature__.unwrap_or_default(),
                    new_public_key: new_public_key__,
                    new_diversifier: new_diversifier__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.Header", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HeaderData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.new_pub_key.is_some() {
            len += 1;
        }
        if !self.new_diversifier.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.HeaderData", len)?;
        if let Some(v) = self.new_pub_key.as_ref() {
            struct_ser.serialize_field("newPubKey", v)?;
        }
        if !self.new_diversifier.is_empty() {
            struct_ser.serialize_field("newDiversifier", &self.new_diversifier)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HeaderData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "new_pub_key",
            "newPubKey",
            "new_diversifier",
            "newDiversifier",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NewPubKey,
            NewDiversifier,
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
                            "newPubKey" | "new_pub_key" => Ok(GeneratedField::NewPubKey),
                            "newDiversifier" | "new_diversifier" => Ok(GeneratedField::NewDiversifier),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HeaderData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.HeaderData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HeaderData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut new_pub_key__ = None;
                let mut new_diversifier__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NewPubKey => {
                            if new_pub_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newPubKey"));
                            }
                            new_pub_key__ = map.next_value()?;
                        }
                        GeneratedField::NewDiversifier => {
                            if new_diversifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newDiversifier"));
                            }
                            new_diversifier__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(HeaderData {
                    new_pub_key: new_pub_key__,
                    new_diversifier: new_diversifier__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.HeaderData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Misbehaviour {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.client_id.is_empty() {
            len += 1;
        }
        if self.sequence != 0 {
            len += 1;
        }
        if self.signature_one.is_some() {
            len += 1;
        }
        if self.signature_two.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.Misbehaviour", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        if let Some(v) = self.signature_one.as_ref() {
            struct_ser.serialize_field("signatureOne", v)?;
        }
        if let Some(v) = self.signature_two.as_ref() {
            struct_ser.serialize_field("signatureTwo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Misbehaviour {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
            "sequence",
            "signature_one",
            "signatureOne",
            "signature_two",
            "signatureTwo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            Sequence,
            SignatureOne,
            SignatureTwo,
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
                            "clientId" | "client_id" => Ok(GeneratedField::ClientId),
                            "sequence" => Ok(GeneratedField::Sequence),
                            "signatureOne" | "signature_one" => Ok(GeneratedField::SignatureOne),
                            "signatureTwo" | "signature_two" => Ok(GeneratedField::SignatureTwo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Misbehaviour;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.Misbehaviour")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Misbehaviour, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut sequence__ = None;
                let mut signature_one__ = None;
                let mut signature_two__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Sequence => {
                            if sequence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sequence"));
                            }
                            sequence__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SignatureOne => {
                            if signature_one__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signatureOne"));
                            }
                            signature_one__ = map.next_value()?;
                        }
                        GeneratedField::SignatureTwo => {
                            if signature_two__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signatureTwo"));
                            }
                            signature_two__ = map.next_value()?;
                        }
                    }
                }
                Ok(Misbehaviour {
                    client_id: client_id__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                    signature_one: signature_one__,
                    signature_two: signature_two__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.Misbehaviour", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NextSequenceRecvData {
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
        if self.next_seq_recv != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.NextSequenceRecvData", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", pbjson::private::base64::encode(&self.path).as_str())?;
        }
        if self.next_seq_recv != 0 {
            struct_ser.serialize_field("nextSeqRecv", ToString::to_string(&self.next_seq_recv).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NextSequenceRecvData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "next_seq_recv",
            "nextSeqRecv",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            NextSeqRecv,
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
                            "nextSeqRecv" | "next_seq_recv" => Ok(GeneratedField::NextSeqRecv),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NextSequenceRecvData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.NextSequenceRecvData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NextSequenceRecvData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut next_seq_recv__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NextSeqRecv => {
                            if next_seq_recv__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextSeqRecv"));
                            }
                            next_seq_recv__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(NextSequenceRecvData {
                    path: path__.unwrap_or_default(),
                    next_seq_recv: next_seq_recv__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.NextSequenceRecvData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketAcknowledgementData {
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
        if !self.acknowledgement.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.PacketAcknowledgementData", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", pbjson::private::base64::encode(&self.path).as_str())?;
        }
        if !self.acknowledgement.is_empty() {
            struct_ser.serialize_field("acknowledgement", pbjson::private::base64::encode(&self.acknowledgement).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketAcknowledgementData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "acknowledgement",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            Acknowledgement,
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
                            "acknowledgement" => Ok(GeneratedField::Acknowledgement),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PacketAcknowledgementData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.PacketAcknowledgementData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketAcknowledgementData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut acknowledgement__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Acknowledgement => {
                            if acknowledgement__.is_some() {
                                return Err(serde::de::Error::duplicate_field("acknowledgement"));
                            }
                            acknowledgement__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(PacketAcknowledgementData {
                    path: path__.unwrap_or_default(),
                    acknowledgement: acknowledgement__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.PacketAcknowledgementData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketCommitmentData {
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
        if !self.commitment.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.PacketCommitmentData", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", pbjson::private::base64::encode(&self.path).as_str())?;
        }
        if !self.commitment.is_empty() {
            struct_ser.serialize_field("commitment", pbjson::private::base64::encode(&self.commitment).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketCommitmentData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "commitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            Commitment,
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
                            "commitment" => Ok(GeneratedField::Commitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PacketCommitmentData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.PacketCommitmentData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketCommitmentData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut commitment__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(PacketCommitmentData {
                    path: path__.unwrap_or_default(),
                    commitment: commitment__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.PacketCommitmentData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketReceiptAbsenceData {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.PacketReceiptAbsenceData", len)?;
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", pbjson::private::base64::encode(&self.path).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketReceiptAbsenceData {
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
            type Value = PacketReceiptAbsenceData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.PacketReceiptAbsenceData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketReceiptAbsenceData, V::Error>
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
                            path__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(PacketReceiptAbsenceData {
                    path: path__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.PacketReceiptAbsenceData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignBytes {
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
        if self.timestamp != 0 {
            len += 1;
        }
        if !self.diversifier.is_empty() {
            len += 1;
        }
        if self.data_type != 0 {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.SignBytes", len)?;
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        if self.timestamp != 0 {
            struct_ser.serialize_field("timestamp", ToString::to_string(&self.timestamp).as_str())?;
        }
        if !self.diversifier.is_empty() {
            struct_ser.serialize_field("diversifier", &self.diversifier)?;
        }
        if self.data_type != 0 {
            let v = DataType::from_i32(self.data_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.data_type)))?;
            struct_ser.serialize_field("dataType", &v)?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignBytes {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sequence",
            "timestamp",
            "diversifier",
            "data_type",
            "dataType",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sequence,
            Timestamp,
            Diversifier,
            DataType,
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
                            "sequence" => Ok(GeneratedField::Sequence),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "diversifier" => Ok(GeneratedField::Diversifier),
                            "dataType" | "data_type" => Ok(GeneratedField::DataType),
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
            type Value = SignBytes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.SignBytes")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SignBytes, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sequence__ = None;
                let mut timestamp__ = None;
                let mut diversifier__ = None;
                let mut data_type__ = None;
                let mut data__ = None;
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
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Diversifier => {
                            if diversifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("diversifier"));
                            }
                            diversifier__ = Some(map.next_value()?);
                        }
                        GeneratedField::DataType => {
                            if data_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataType"));
                            }
                            data_type__ = Some(map.next_value::<DataType>()? as i32);
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
                Ok(SignBytes {
                    sequence: sequence__.unwrap_or_default(),
                    timestamp: timestamp__.unwrap_or_default(),
                    diversifier: diversifier__.unwrap_or_default(),
                    data_type: data_type__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.SignBytes", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignatureAndData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.signature.is_empty() {
            len += 1;
        }
        if self.data_type != 0 {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        if self.timestamp != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.SignatureAndData", len)?;
        if !self.signature.is_empty() {
            struct_ser.serialize_field("signature", pbjson::private::base64::encode(&self.signature).as_str())?;
        }
        if self.data_type != 0 {
            let v = DataType::from_i32(self.data_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.data_type)))?;
            struct_ser.serialize_field("dataType", &v)?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        if self.timestamp != 0 {
            struct_ser.serialize_field("timestamp", ToString::to_string(&self.timestamp).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignatureAndData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signature",
            "data_type",
            "dataType",
            "data",
            "timestamp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Signature,
            DataType,
            Data,
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
                            "signature" => Ok(GeneratedField::Signature),
                            "dataType" | "data_type" => Ok(GeneratedField::DataType),
                            "data" => Ok(GeneratedField::Data),
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
            type Value = SignatureAndData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.SignatureAndData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SignatureAndData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signature__ = None;
                let mut data_type__ = None;
                let mut data__ = None;
                let mut timestamp__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DataType => {
                            if data_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataType"));
                            }
                            data_type__ = Some(map.next_value::<DataType>()? as i32);
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SignatureAndData {
                    signature: signature__.unwrap_or_default(),
                    data_type: data_type__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                    timestamp: timestamp__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.SignatureAndData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TimestampedSignatureData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.signature_data.is_empty() {
            len += 1;
        }
        if self.timestamp != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.solomachine.v2.TimestampedSignatureData", len)?;
        if !self.signature_data.is_empty() {
            struct_ser.serialize_field("signatureData", pbjson::private::base64::encode(&self.signature_data).as_str())?;
        }
        if self.timestamp != 0 {
            struct_ser.serialize_field("timestamp", ToString::to_string(&self.timestamp).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TimestampedSignatureData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signature_data",
            "signatureData",
            "timestamp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignatureData,
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
                            "signatureData" | "signature_data" => Ok(GeneratedField::SignatureData),
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
            type Value = TimestampedSignatureData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.solomachine.v2.TimestampedSignatureData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TimestampedSignatureData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signature_data__ = None;
                let mut timestamp__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SignatureData => {
                            if signature_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signatureData"));
                            }
                            signature_data__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TimestampedSignatureData {
                    signature_data: signature_data__.unwrap_or_default(),
                    timestamp: timestamp__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.solomachine.v2.TimestampedSignatureData", FIELDS, GeneratedVisitor)
    }
}
