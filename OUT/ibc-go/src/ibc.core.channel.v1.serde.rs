// @generated
impl serde::Serialize for Acknowledgement {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.response.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.Acknowledgement", len)?;
        if let Some(v) = self.response.as_ref() {
            match v {
                acknowledgement::Response::Result(v) => {
                    struct_ser.serialize_field("result", pbjson::private::base64::encode(&v).as_str())?;
                }
                acknowledgement::Response::Error(v) => {
                    struct_ser.serialize_field("error", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Acknowledgement {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "result",
            "error",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Result,
            Error,
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
                            "error" => Ok(GeneratedField::Error),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Acknowledgement;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.Acknowledgement")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Acknowledgement, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut response__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Result => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            response__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| acknowledgement::Response::Result(x.0));
                        }
                        GeneratedField::Error => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            response__ = map.next_value::<::std::option::Option<_>>()?.map(acknowledgement::Response::Error);
                        }
                    }
                }
                Ok(Acknowledgement {
                    response: response__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.Acknowledgement", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Channel {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state != 0 {
            len += 1;
        }
        if self.ordering != 0 {
            len += 1;
        }
        if self.counterparty.is_some() {
            len += 1;
        }
        if !self.connection_hops.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.Channel", len)?;
        if self.state != 0 {
            let v = State::from_i32(self.state)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        if self.ordering != 0 {
            let v = Order::from_i32(self.ordering)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.ordering)))?;
            struct_ser.serialize_field("ordering", &v)?;
        }
        if let Some(v) = self.counterparty.as_ref() {
            struct_ser.serialize_field("counterparty", v)?;
        }
        if !self.connection_hops.is_empty() {
            struct_ser.serialize_field("connectionHops", &self.connection_hops)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Channel {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "state",
            "ordering",
            "counterparty",
            "connection_hops",
            "connectionHops",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            State,
            Ordering,
            Counterparty,
            ConnectionHops,
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
                            "state" => Ok(GeneratedField::State),
                            "ordering" => Ok(GeneratedField::Ordering),
                            "counterparty" => Ok(GeneratedField::Counterparty),
                            "connectionHops" | "connection_hops" => Ok(GeneratedField::ConnectionHops),
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
            type Value = Channel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.Channel")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Channel, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                let mut ordering__ = None;
                let mut counterparty__ = None;
                let mut connection_hops__ = None;
                let mut version__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map.next_value::<State>()? as i32);
                        }
                        GeneratedField::Ordering => {
                            if ordering__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ordering"));
                            }
                            ordering__ = Some(map.next_value::<Order>()? as i32);
                        }
                        GeneratedField::Counterparty => {
                            if counterparty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterparty"));
                            }
                            counterparty__ = map.next_value()?;
                        }
                        GeneratedField::ConnectionHops => {
                            if connection_hops__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionHops"));
                            }
                            connection_hops__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Channel {
                    state: state__.unwrap_or_default(),
                    ordering: ordering__.unwrap_or_default(),
                    counterparty: counterparty__,
                    connection_hops: connection_hops__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.Channel", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Counterparty {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.Counterparty", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Counterparty {
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
            type Value = Counterparty;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.Counterparty")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Counterparty, V::Error>
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
                Ok(Counterparty {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.Counterparty", FIELDS, GeneratedVisitor)
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
        if !self.channels.is_empty() {
            len += 1;
        }
        if !self.acknowledgements.is_empty() {
            len += 1;
        }
        if !self.commitments.is_empty() {
            len += 1;
        }
        if !self.receipts.is_empty() {
            len += 1;
        }
        if !self.send_sequences.is_empty() {
            len += 1;
        }
        if !self.recv_sequences.is_empty() {
            len += 1;
        }
        if !self.ack_sequences.is_empty() {
            len += 1;
        }
        if self.next_channel_sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.GenesisState", len)?;
        if !self.channels.is_empty() {
            struct_ser.serialize_field("channels", &self.channels)?;
        }
        if !self.acknowledgements.is_empty() {
            struct_ser.serialize_field("acknowledgements", &self.acknowledgements)?;
        }
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        if !self.receipts.is_empty() {
            struct_ser.serialize_field("receipts", &self.receipts)?;
        }
        if !self.send_sequences.is_empty() {
            struct_ser.serialize_field("sendSequences", &self.send_sequences)?;
        }
        if !self.recv_sequences.is_empty() {
            struct_ser.serialize_field("recvSequences", &self.recv_sequences)?;
        }
        if !self.ack_sequences.is_empty() {
            struct_ser.serialize_field("ackSequences", &self.ack_sequences)?;
        }
        if self.next_channel_sequence != 0 {
            struct_ser.serialize_field("nextChannelSequence", ToString::to_string(&self.next_channel_sequence).as_str())?;
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
            "channels",
            "acknowledgements",
            "commitments",
            "receipts",
            "send_sequences",
            "sendSequences",
            "recv_sequences",
            "recvSequences",
            "ack_sequences",
            "ackSequences",
            "next_channel_sequence",
            "nextChannelSequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Channels,
            Acknowledgements,
            Commitments,
            Receipts,
            SendSequences,
            RecvSequences,
            AckSequences,
            NextChannelSequence,
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
                            "channels" => Ok(GeneratedField::Channels),
                            "acknowledgements" => Ok(GeneratedField::Acknowledgements),
                            "commitments" => Ok(GeneratedField::Commitments),
                            "receipts" => Ok(GeneratedField::Receipts),
                            "sendSequences" | "send_sequences" => Ok(GeneratedField::SendSequences),
                            "recvSequences" | "recv_sequences" => Ok(GeneratedField::RecvSequences),
                            "ackSequences" | "ack_sequences" => Ok(GeneratedField::AckSequences),
                            "nextChannelSequence" | "next_channel_sequence" => Ok(GeneratedField::NextChannelSequence),
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
                formatter.write_str("struct ibc.core.channel.v1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channels__ = None;
                let mut acknowledgements__ = None;
                let mut commitments__ = None;
                let mut receipts__ = None;
                let mut send_sequences__ = None;
                let mut recv_sequences__ = None;
                let mut ack_sequences__ = None;
                let mut next_channel_sequence__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Channels => {
                            if channels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channels"));
                            }
                            channels__ = Some(map.next_value()?);
                        }
                        GeneratedField::Acknowledgements => {
                            if acknowledgements__.is_some() {
                                return Err(serde::de::Error::duplicate_field("acknowledgements"));
                            }
                            acknowledgements__ = Some(map.next_value()?);
                        }
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map.next_value()?);
                        }
                        GeneratedField::Receipts => {
                            if receipts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receipts"));
                            }
                            receipts__ = Some(map.next_value()?);
                        }
                        GeneratedField::SendSequences => {
                            if send_sequences__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sendSequences"));
                            }
                            send_sequences__ = Some(map.next_value()?);
                        }
                        GeneratedField::RecvSequences => {
                            if recv_sequences__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recvSequences"));
                            }
                            recv_sequences__ = Some(map.next_value()?);
                        }
                        GeneratedField::AckSequences => {
                            if ack_sequences__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ackSequences"));
                            }
                            ack_sequences__ = Some(map.next_value()?);
                        }
                        GeneratedField::NextChannelSequence => {
                            if next_channel_sequence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextChannelSequence"));
                            }
                            next_channel_sequence__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GenesisState {
                    channels: channels__.unwrap_or_default(),
                    acknowledgements: acknowledgements__.unwrap_or_default(),
                    commitments: commitments__.unwrap_or_default(),
                    receipts: receipts__.unwrap_or_default(),
                    send_sequences: send_sequences__.unwrap_or_default(),
                    recv_sequences: recv_sequences__.unwrap_or_default(),
                    ack_sequences: ack_sequences__.unwrap_or_default(),
                    next_channel_sequence: next_channel_sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IdentifiedChannel {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state != 0 {
            len += 1;
        }
        if self.ordering != 0 {
            len += 1;
        }
        if self.counterparty.is_some() {
            len += 1;
        }
        if !self.connection_hops.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        if !self.port_id.is_empty() {
            len += 1;
        }
        if !self.channel_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.IdentifiedChannel", len)?;
        if self.state != 0 {
            let v = State::from_i32(self.state)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        if self.ordering != 0 {
            let v = Order::from_i32(self.ordering)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.ordering)))?;
            struct_ser.serialize_field("ordering", &v)?;
        }
        if let Some(v) = self.counterparty.as_ref() {
            struct_ser.serialize_field("counterparty", v)?;
        }
        if !self.connection_hops.is_empty() {
            struct_ser.serialize_field("connectionHops", &self.connection_hops)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IdentifiedChannel {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "state",
            "ordering",
            "counterparty",
            "connection_hops",
            "connectionHops",
            "version",
            "port_id",
            "portId",
            "channel_id",
            "channelId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            State,
            Ordering,
            Counterparty,
            ConnectionHops,
            Version,
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
                            "state" => Ok(GeneratedField::State),
                            "ordering" => Ok(GeneratedField::Ordering),
                            "counterparty" => Ok(GeneratedField::Counterparty),
                            "connectionHops" | "connection_hops" => Ok(GeneratedField::ConnectionHops),
                            "version" => Ok(GeneratedField::Version),
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
            type Value = IdentifiedChannel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.IdentifiedChannel")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<IdentifiedChannel, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                let mut ordering__ = None;
                let mut counterparty__ = None;
                let mut connection_hops__ = None;
                let mut version__ = None;
                let mut port_id__ = None;
                let mut channel_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map.next_value::<State>()? as i32);
                        }
                        GeneratedField::Ordering => {
                            if ordering__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ordering"));
                            }
                            ordering__ = Some(map.next_value::<Order>()? as i32);
                        }
                        GeneratedField::Counterparty => {
                            if counterparty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterparty"));
                            }
                            counterparty__ = map.next_value()?;
                        }
                        GeneratedField::ConnectionHops => {
                            if connection_hops__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionHops"));
                            }
                            connection_hops__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
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
                Ok(IdentifiedChannel {
                    state: state__.unwrap_or_default(),
                    ordering: ordering__.unwrap_or_default(),
                    counterparty: counterparty__,
                    connection_hops: connection_hops__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.IdentifiedChannel", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgAcknowledgement {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet.is_some() {
            len += 1;
        }
        if !self.acknowledgement.is_empty() {
            len += 1;
        }
        if !self.proof_acked.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgAcknowledgement", len)?;
        if let Some(v) = self.packet.as_ref() {
            struct_ser.serialize_field("packet", v)?;
        }
        if !self.acknowledgement.is_empty() {
            struct_ser.serialize_field("acknowledgement", pbjson::private::base64::encode(&self.acknowledgement).as_str())?;
        }
        if !self.proof_acked.is_empty() {
            struct_ser.serialize_field("proofAcked", pbjson::private::base64::encode(&self.proof_acked).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgAcknowledgement {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet",
            "acknowledgement",
            "proof_acked",
            "proofAcked",
            "proof_height",
            "proofHeight",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Packet,
            Acknowledgement,
            ProofAcked,
            ProofHeight,
            Signer,
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
                            "packet" => Ok(GeneratedField::Packet),
                            "acknowledgement" => Ok(GeneratedField::Acknowledgement),
                            "proofAcked" | "proof_acked" => Ok(GeneratedField::ProofAcked),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgAcknowledgement;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgAcknowledgement")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgAcknowledgement, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet__ = None;
                let mut acknowledgement__ = None;
                let mut proof_acked__ = None;
                let mut proof_height__ = None;
                let mut signer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Packet => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packet"));
                            }
                            packet__ = map.next_value()?;
                        }
                        GeneratedField::Acknowledgement => {
                            if acknowledgement__.is_some() {
                                return Err(serde::de::Error::duplicate_field("acknowledgement"));
                            }
                            acknowledgement__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofAcked => {
                            if proof_acked__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofAcked"));
                            }
                            proof_acked__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgAcknowledgement {
                    packet: packet__,
                    acknowledgement: acknowledgement__.unwrap_or_default(),
                    proof_acked: proof_acked__.unwrap_or_default(),
                    proof_height: proof_height__,
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgAcknowledgement", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgAcknowledgementResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.result != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgAcknowledgementResponse", len)?;
        if self.result != 0 {
            let v = ResponseResultType::from_i32(self.result)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.result)))?;
            struct_ser.serialize_field("result", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgAcknowledgementResponse {
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
            type Value = MsgAcknowledgementResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgAcknowledgementResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgAcknowledgementResponse, V::Error>
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
                            result__ = Some(map.next_value::<ResponseResultType>()? as i32);
                        }
                    }
                }
                Ok(MsgAcknowledgementResponse {
                    result: result__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgAcknowledgementResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelCloseConfirm {
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
        if !self.proof_init.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelCloseConfirm", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.proof_init.is_empty() {
            struct_ser.serialize_field("proofInit", pbjson::private::base64::encode(&self.proof_init).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelCloseConfirm {
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
            "proof_init",
            "proofInit",
            "proof_height",
            "proofHeight",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            ProofInit,
            ProofHeight,
            Signer,
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
                            "proofInit" | "proof_init" => Ok(GeneratedField::ProofInit),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgChannelCloseConfirm;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelCloseConfirm")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelCloseConfirm, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut proof_init__ = None;
                let mut proof_height__ = None;
                let mut signer__ = None;
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
                        GeneratedField::ProofInit => {
                            if proof_init__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofInit"));
                            }
                            proof_init__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgChannelCloseConfirm {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    proof_init: proof_init__.unwrap_or_default(),
                    proof_height: proof_height__,
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelCloseConfirm", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelCloseConfirmResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelCloseConfirmResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelCloseConfirmResponse {
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
            type Value = MsgChannelCloseConfirmResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelCloseConfirmResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelCloseConfirmResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgChannelCloseConfirmResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelCloseConfirmResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelCloseInit {
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
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelCloseInit", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelCloseInit {
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
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            Signer,
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
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgChannelCloseInit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelCloseInit")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelCloseInit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut signer__ = None;
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
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgChannelCloseInit {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelCloseInit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelCloseInitResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelCloseInitResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelCloseInitResponse {
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
            type Value = MsgChannelCloseInitResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelCloseInitResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelCloseInitResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgChannelCloseInitResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelCloseInitResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelOpenAck {
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
        if !self.counterparty_channel_id.is_empty() {
            len += 1;
        }
        if !self.counterparty_version.is_empty() {
            len += 1;
        }
        if !self.proof_try.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelOpenAck", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.counterparty_channel_id.is_empty() {
            struct_ser.serialize_field("counterpartyChannelId", &self.counterparty_channel_id)?;
        }
        if !self.counterparty_version.is_empty() {
            struct_ser.serialize_field("counterpartyVersion", &self.counterparty_version)?;
        }
        if !self.proof_try.is_empty() {
            struct_ser.serialize_field("proofTry", pbjson::private::base64::encode(&self.proof_try).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelOpenAck {
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
            "counterparty_channel_id",
            "counterpartyChannelId",
            "counterparty_version",
            "counterpartyVersion",
            "proof_try",
            "proofTry",
            "proof_height",
            "proofHeight",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            CounterpartyChannelId,
            CounterpartyVersion,
            ProofTry,
            ProofHeight,
            Signer,
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
                            "counterpartyChannelId" | "counterparty_channel_id" => Ok(GeneratedField::CounterpartyChannelId),
                            "counterpartyVersion" | "counterparty_version" => Ok(GeneratedField::CounterpartyVersion),
                            "proofTry" | "proof_try" => Ok(GeneratedField::ProofTry),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgChannelOpenAck;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelOpenAck")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelOpenAck, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut counterparty_channel_id__ = None;
                let mut counterparty_version__ = None;
                let mut proof_try__ = None;
                let mut proof_height__ = None;
                let mut signer__ = None;
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
                        GeneratedField::CounterpartyChannelId => {
                            if counterparty_channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterpartyChannelId"));
                            }
                            counterparty_channel_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::CounterpartyVersion => {
                            if counterparty_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterpartyVersion"));
                            }
                            counterparty_version__ = Some(map.next_value()?);
                        }
                        GeneratedField::ProofTry => {
                            if proof_try__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofTry"));
                            }
                            proof_try__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgChannelOpenAck {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    counterparty_channel_id: counterparty_channel_id__.unwrap_or_default(),
                    counterparty_version: counterparty_version__.unwrap_or_default(),
                    proof_try: proof_try__.unwrap_or_default(),
                    proof_height: proof_height__,
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelOpenAck", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelOpenAckResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelOpenAckResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelOpenAckResponse {
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
            type Value = MsgChannelOpenAckResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelOpenAckResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelOpenAckResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgChannelOpenAckResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelOpenAckResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelOpenConfirm {
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
        if !self.proof_ack.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelOpenConfirm", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.proof_ack.is_empty() {
            struct_ser.serialize_field("proofAck", pbjson::private::base64::encode(&self.proof_ack).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelOpenConfirm {
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
            "proof_ack",
            "proofAck",
            "proof_height",
            "proofHeight",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            ProofAck,
            ProofHeight,
            Signer,
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
                            "proofAck" | "proof_ack" => Ok(GeneratedField::ProofAck),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgChannelOpenConfirm;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelOpenConfirm")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelOpenConfirm, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut proof_ack__ = None;
                let mut proof_height__ = None;
                let mut signer__ = None;
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
                        GeneratedField::ProofAck => {
                            if proof_ack__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofAck"));
                            }
                            proof_ack__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgChannelOpenConfirm {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    proof_ack: proof_ack__.unwrap_or_default(),
                    proof_height: proof_height__,
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelOpenConfirm", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelOpenConfirmResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelOpenConfirmResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelOpenConfirmResponse {
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
            type Value = MsgChannelOpenConfirmResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelOpenConfirmResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelOpenConfirmResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgChannelOpenConfirmResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelOpenConfirmResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelOpenInit {
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
        if self.channel.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelOpenInit", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if let Some(v) = self.channel.as_ref() {
            struct_ser.serialize_field("channel", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelOpenInit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "port_id",
            "portId",
            "channel",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            Channel,
            Signer,
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
                            "channel" => Ok(GeneratedField::Channel),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgChannelOpenInit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelOpenInit")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelOpenInit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel__ = None;
                let mut signer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PortId => {
                            if port_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("portId"));
                            }
                            port_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Channel => {
                            if channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channel"));
                            }
                            channel__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgChannelOpenInit {
                    port_id: port_id__.unwrap_or_default(),
                    channel: channel__,
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelOpenInit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelOpenInitResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.channel_id.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelOpenInitResponse", len)?;
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelOpenInitResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channel_id",
            "channelId",
            "version",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChannelId,
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
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
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
            type Value = MsgChannelOpenInitResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelOpenInitResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelOpenInitResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channel_id__ = None;
                let mut version__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChannelId => {
                            if channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelId"));
                            }
                            channel_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgChannelOpenInitResponse {
                    channel_id: channel_id__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelOpenInitResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelOpenTry {
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
        if !self.previous_channel_id.is_empty() {
            len += 1;
        }
        if self.channel.is_some() {
            len += 1;
        }
        if !self.counterparty_version.is_empty() {
            len += 1;
        }
        if !self.proof_init.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelOpenTry", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.previous_channel_id.is_empty() {
            struct_ser.serialize_field("previousChannelId", &self.previous_channel_id)?;
        }
        if let Some(v) = self.channel.as_ref() {
            struct_ser.serialize_field("channel", v)?;
        }
        if !self.counterparty_version.is_empty() {
            struct_ser.serialize_field("counterpartyVersion", &self.counterparty_version)?;
        }
        if !self.proof_init.is_empty() {
            struct_ser.serialize_field("proofInit", pbjson::private::base64::encode(&self.proof_init).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelOpenTry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "port_id",
            "portId",
            "previous_channel_id",
            "previousChannelId",
            "channel",
            "counterparty_version",
            "counterpartyVersion",
            "proof_init",
            "proofInit",
            "proof_height",
            "proofHeight",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            PreviousChannelId,
            Channel,
            CounterpartyVersion,
            ProofInit,
            ProofHeight,
            Signer,
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
                            "previousChannelId" | "previous_channel_id" => Ok(GeneratedField::PreviousChannelId),
                            "channel" => Ok(GeneratedField::Channel),
                            "counterpartyVersion" | "counterparty_version" => Ok(GeneratedField::CounterpartyVersion),
                            "proofInit" | "proof_init" => Ok(GeneratedField::ProofInit),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgChannelOpenTry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelOpenTry")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelOpenTry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut previous_channel_id__ = None;
                let mut channel__ = None;
                let mut counterparty_version__ = None;
                let mut proof_init__ = None;
                let mut proof_height__ = None;
                let mut signer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PortId => {
                            if port_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("portId"));
                            }
                            port_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::PreviousChannelId => {
                            if previous_channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("previousChannelId"));
                            }
                            previous_channel_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Channel => {
                            if channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channel"));
                            }
                            channel__ = map.next_value()?;
                        }
                        GeneratedField::CounterpartyVersion => {
                            if counterparty_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterpartyVersion"));
                            }
                            counterparty_version__ = Some(map.next_value()?);
                        }
                        GeneratedField::ProofInit => {
                            if proof_init__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofInit"));
                            }
                            proof_init__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgChannelOpenTry {
                    port_id: port_id__.unwrap_or_default(),
                    previous_channel_id: previous_channel_id__.unwrap_or_default(),
                    channel: channel__,
                    counterparty_version: counterparty_version__.unwrap_or_default(),
                    proof_init: proof_init__.unwrap_or_default(),
                    proof_height: proof_height__,
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelOpenTry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgChannelOpenTryResponse {
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
        if !self.channel_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgChannelOpenTryResponse", len)?;
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgChannelOpenTryResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "version",
            "channel_id",
            "channelId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Version,
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
                            "version" => Ok(GeneratedField::Version),
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
            type Value = MsgChannelOpenTryResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgChannelOpenTryResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgChannelOpenTryResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut version__ = None;
                let mut channel_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                        GeneratedField::ChannelId => {
                            if channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelId"));
                            }
                            channel_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgChannelOpenTryResponse {
                    version: version__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgChannelOpenTryResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRecvPacket {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet.is_some() {
            len += 1;
        }
        if !self.proof_commitment.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgRecvPacket", len)?;
        if let Some(v) = self.packet.as_ref() {
            struct_ser.serialize_field("packet", v)?;
        }
        if !self.proof_commitment.is_empty() {
            struct_ser.serialize_field("proofCommitment", pbjson::private::base64::encode(&self.proof_commitment).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRecvPacket {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet",
            "proof_commitment",
            "proofCommitment",
            "proof_height",
            "proofHeight",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Packet,
            ProofCommitment,
            ProofHeight,
            Signer,
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
                            "packet" => Ok(GeneratedField::Packet),
                            "proofCommitment" | "proof_commitment" => Ok(GeneratedField::ProofCommitment),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgRecvPacket;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgRecvPacket")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgRecvPacket, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet__ = None;
                let mut proof_commitment__ = None;
                let mut proof_height__ = None;
                let mut signer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Packet => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packet"));
                            }
                            packet__ = map.next_value()?;
                        }
                        GeneratedField::ProofCommitment => {
                            if proof_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofCommitment"));
                            }
                            proof_commitment__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgRecvPacket {
                    packet: packet__,
                    proof_commitment: proof_commitment__.unwrap_or_default(),
                    proof_height: proof_height__,
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgRecvPacket", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRecvPacketResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.result != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgRecvPacketResponse", len)?;
        if self.result != 0 {
            let v = ResponseResultType::from_i32(self.result)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.result)))?;
            struct_ser.serialize_field("result", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRecvPacketResponse {
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
            type Value = MsgRecvPacketResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgRecvPacketResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgRecvPacketResponse, V::Error>
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
                            result__ = Some(map.next_value::<ResponseResultType>()? as i32);
                        }
                    }
                }
                Ok(MsgRecvPacketResponse {
                    result: result__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgRecvPacketResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgTimeout {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet.is_some() {
            len += 1;
        }
        if !self.proof_unreceived.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if self.next_sequence_recv != 0 {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgTimeout", len)?;
        if let Some(v) = self.packet.as_ref() {
            struct_ser.serialize_field("packet", v)?;
        }
        if !self.proof_unreceived.is_empty() {
            struct_ser.serialize_field("proofUnreceived", pbjson::private::base64::encode(&self.proof_unreceived).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if self.next_sequence_recv != 0 {
            struct_ser.serialize_field("nextSequenceRecv", ToString::to_string(&self.next_sequence_recv).as_str())?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgTimeout {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet",
            "proof_unreceived",
            "proofUnreceived",
            "proof_height",
            "proofHeight",
            "next_sequence_recv",
            "nextSequenceRecv",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Packet,
            ProofUnreceived,
            ProofHeight,
            NextSequenceRecv,
            Signer,
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
                            "packet" => Ok(GeneratedField::Packet),
                            "proofUnreceived" | "proof_unreceived" => Ok(GeneratedField::ProofUnreceived),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "nextSequenceRecv" | "next_sequence_recv" => Ok(GeneratedField::NextSequenceRecv),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgTimeout;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgTimeout")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgTimeout, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet__ = None;
                let mut proof_unreceived__ = None;
                let mut proof_height__ = None;
                let mut next_sequence_recv__ = None;
                let mut signer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Packet => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packet"));
                            }
                            packet__ = map.next_value()?;
                        }
                        GeneratedField::ProofUnreceived => {
                            if proof_unreceived__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofUnreceived"));
                            }
                            proof_unreceived__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::NextSequenceRecv => {
                            if next_sequence_recv__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextSequenceRecv"));
                            }
                            next_sequence_recv__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgTimeout {
                    packet: packet__,
                    proof_unreceived: proof_unreceived__.unwrap_or_default(),
                    proof_height: proof_height__,
                    next_sequence_recv: next_sequence_recv__.unwrap_or_default(),
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgTimeout", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgTimeoutOnClose {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet.is_some() {
            len += 1;
        }
        if !self.proof_unreceived.is_empty() {
            len += 1;
        }
        if !self.proof_close.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if self.next_sequence_recv != 0 {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgTimeoutOnClose", len)?;
        if let Some(v) = self.packet.as_ref() {
            struct_ser.serialize_field("packet", v)?;
        }
        if !self.proof_unreceived.is_empty() {
            struct_ser.serialize_field("proofUnreceived", pbjson::private::base64::encode(&self.proof_unreceived).as_str())?;
        }
        if !self.proof_close.is_empty() {
            struct_ser.serialize_field("proofClose", pbjson::private::base64::encode(&self.proof_close).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if self.next_sequence_recv != 0 {
            struct_ser.serialize_field("nextSequenceRecv", ToString::to_string(&self.next_sequence_recv).as_str())?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgTimeoutOnClose {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet",
            "proof_unreceived",
            "proofUnreceived",
            "proof_close",
            "proofClose",
            "proof_height",
            "proofHeight",
            "next_sequence_recv",
            "nextSequenceRecv",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Packet,
            ProofUnreceived,
            ProofClose,
            ProofHeight,
            NextSequenceRecv,
            Signer,
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
                            "packet" => Ok(GeneratedField::Packet),
                            "proofUnreceived" | "proof_unreceived" => Ok(GeneratedField::ProofUnreceived),
                            "proofClose" | "proof_close" => Ok(GeneratedField::ProofClose),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "nextSequenceRecv" | "next_sequence_recv" => Ok(GeneratedField::NextSequenceRecv),
                            "signer" => Ok(GeneratedField::Signer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgTimeoutOnClose;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgTimeoutOnClose")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgTimeoutOnClose, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet__ = None;
                let mut proof_unreceived__ = None;
                let mut proof_close__ = None;
                let mut proof_height__ = None;
                let mut next_sequence_recv__ = None;
                let mut signer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Packet => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packet"));
                            }
                            packet__ = map.next_value()?;
                        }
                        GeneratedField::ProofUnreceived => {
                            if proof_unreceived__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofUnreceived"));
                            }
                            proof_unreceived__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofClose => {
                            if proof_close__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofClose"));
                            }
                            proof_close__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::NextSequenceRecv => {
                            if next_sequence_recv__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextSequenceRecv"));
                            }
                            next_sequence_recv__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgTimeoutOnClose {
                    packet: packet__,
                    proof_unreceived: proof_unreceived__.unwrap_or_default(),
                    proof_close: proof_close__.unwrap_or_default(),
                    proof_height: proof_height__,
                    next_sequence_recv: next_sequence_recv__.unwrap_or_default(),
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgTimeoutOnClose", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgTimeoutOnCloseResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.result != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgTimeoutOnCloseResponse", len)?;
        if self.result != 0 {
            let v = ResponseResultType::from_i32(self.result)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.result)))?;
            struct_ser.serialize_field("result", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgTimeoutOnCloseResponse {
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
            type Value = MsgTimeoutOnCloseResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgTimeoutOnCloseResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgTimeoutOnCloseResponse, V::Error>
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
                            result__ = Some(map.next_value::<ResponseResultType>()? as i32);
                        }
                    }
                }
                Ok(MsgTimeoutOnCloseResponse {
                    result: result__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgTimeoutOnCloseResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgTimeoutResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.result != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.MsgTimeoutResponse", len)?;
        if self.result != 0 {
            let v = ResponseResultType::from_i32(self.result)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.result)))?;
            struct_ser.serialize_field("result", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgTimeoutResponse {
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
            type Value = MsgTimeoutResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.MsgTimeoutResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgTimeoutResponse, V::Error>
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
                            result__ = Some(map.next_value::<ResponseResultType>()? as i32);
                        }
                    }
                }
                Ok(MsgTimeoutResponse {
                    result: result__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.MsgTimeoutResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Order {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::NoneUnspecified => "ORDER_NONE_UNSPECIFIED",
            Self::Unordered => "ORDER_UNORDERED",
            Self::Ordered => "ORDER_ORDERED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Order {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ORDER_NONE_UNSPECIFIED",
            "ORDER_UNORDERED",
            "ORDER_ORDERED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Order;

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
                    .and_then(Order::from_i32)
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
                    .and_then(Order::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ORDER_NONE_UNSPECIFIED" => Ok(Order::NoneUnspecified),
                    "ORDER_UNORDERED" => Ok(Order::Unordered),
                    "ORDER_ORDERED" => Ok(Order::Ordered),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Packet {
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
        if !self.source_port.is_empty() {
            len += 1;
        }
        if !self.source_channel.is_empty() {
            len += 1;
        }
        if !self.destination_port.is_empty() {
            len += 1;
        }
        if !self.destination_channel.is_empty() {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        if self.timeout_height.is_some() {
            len += 1;
        }
        if self.timeout_timestamp != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.Packet", len)?;
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        if !self.source_port.is_empty() {
            struct_ser.serialize_field("sourcePort", &self.source_port)?;
        }
        if !self.source_channel.is_empty() {
            struct_ser.serialize_field("sourceChannel", &self.source_channel)?;
        }
        if !self.destination_port.is_empty() {
            struct_ser.serialize_field("destinationPort", &self.destination_port)?;
        }
        if !self.destination_channel.is_empty() {
            struct_ser.serialize_field("destinationChannel", &self.destination_channel)?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        if let Some(v) = self.timeout_height.as_ref() {
            struct_ser.serialize_field("timeoutHeight", v)?;
        }
        if self.timeout_timestamp != 0 {
            struct_ser.serialize_field("timeoutTimestamp", ToString::to_string(&self.timeout_timestamp).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Packet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sequence",
            "source_port",
            "sourcePort",
            "source_channel",
            "sourceChannel",
            "destination_port",
            "destinationPort",
            "destination_channel",
            "destinationChannel",
            "data",
            "timeout_height",
            "timeoutHeight",
            "timeout_timestamp",
            "timeoutTimestamp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sequence,
            SourcePort,
            SourceChannel,
            DestinationPort,
            DestinationChannel,
            Data,
            TimeoutHeight,
            TimeoutTimestamp,
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
                            "sourcePort" | "source_port" => Ok(GeneratedField::SourcePort),
                            "sourceChannel" | "source_channel" => Ok(GeneratedField::SourceChannel),
                            "destinationPort" | "destination_port" => Ok(GeneratedField::DestinationPort),
                            "destinationChannel" | "destination_channel" => Ok(GeneratedField::DestinationChannel),
                            "data" => Ok(GeneratedField::Data),
                            "timeoutHeight" | "timeout_height" => Ok(GeneratedField::TimeoutHeight),
                            "timeoutTimestamp" | "timeout_timestamp" => Ok(GeneratedField::TimeoutTimestamp),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Packet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.Packet")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Packet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sequence__ = None;
                let mut source_port__ = None;
                let mut source_channel__ = None;
                let mut destination_port__ = None;
                let mut destination_channel__ = None;
                let mut data__ = None;
                let mut timeout_height__ = None;
                let mut timeout_timestamp__ = None;
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
                        GeneratedField::DestinationPort => {
                            if destination_port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationPort"));
                            }
                            destination_port__ = Some(map.next_value()?);
                        }
                        GeneratedField::DestinationChannel => {
                            if destination_channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationChannel"));
                            }
                            destination_channel__ = Some(map.next_value()?);
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
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
                    }
                }
                Ok(Packet {
                    sequence: sequence__.unwrap_or_default(),
                    source_port: source_port__.unwrap_or_default(),
                    source_channel: source_channel__.unwrap_or_default(),
                    destination_port: destination_port__.unwrap_or_default(),
                    destination_channel: destination_channel__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                    timeout_height: timeout_height__,
                    timeout_timestamp: timeout_timestamp__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.Packet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketId {
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
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.PacketId", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketId {
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
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
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
            type Value = PacketId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.PacketId")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut sequence__ = None;
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
                Ok(PacketId {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.PacketId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketSequence {
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
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.PacketSequence", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketSequence {
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
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
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
            type Value = PacketSequence;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.PacketSequence")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketSequence, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut sequence__ = None;
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
                Ok(PacketSequence {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.PacketSequence", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketState {
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
        if self.sequence != 0 {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.PacketState", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketState {
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
            "sequence",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            Sequence,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
                            "sequence" => Ok(GeneratedField::Sequence),
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
            type Value = PacketState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.PacketState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut sequence__ = None;
                let mut data__ = None;
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
                        GeneratedField::Sequence => {
                            if sequence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sequence"));
                            }
                            sequence__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
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
                Ok(PacketState {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.PacketState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryChannelClientStateRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryChannelClientStateRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryChannelClientStateRequest {
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
            type Value = QueryChannelClientStateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryChannelClientStateRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryChannelClientStateRequest, V::Error>
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
                Ok(QueryChannelClientStateRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryChannelClientStateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryChannelClientStateResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.identified_client_state.is_some() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryChannelClientStateResponse", len)?;
        if let Some(v) = self.identified_client_state.as_ref() {
            struct_ser.serialize_field("identifiedClientState", v)?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryChannelClientStateResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identified_client_state",
            "identifiedClientState",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IdentifiedClientState,
            Proof,
            ProofHeight,
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
                            "identifiedClientState" | "identified_client_state" => Ok(GeneratedField::IdentifiedClientState),
                            "proof" => Ok(GeneratedField::Proof),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryChannelClientStateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryChannelClientStateResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryChannelClientStateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identified_client_state__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::IdentifiedClientState => {
                            if identified_client_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identifiedClientState"));
                            }
                            identified_client_state__ = map.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryChannelClientStateResponse {
                    identified_client_state: identified_client_state__,
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryChannelClientStateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryChannelConsensusStateRequest {
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
        if self.revision_number != 0 {
            len += 1;
        }
        if self.revision_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryChannelConsensusStateRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.revision_number != 0 {
            struct_ser.serialize_field("revisionNumber", ToString::to_string(&self.revision_number).as_str())?;
        }
        if self.revision_height != 0 {
            struct_ser.serialize_field("revisionHeight", ToString::to_string(&self.revision_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryChannelConsensusStateRequest {
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
            "revision_number",
            "revisionNumber",
            "revision_height",
            "revisionHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            RevisionNumber,
            RevisionHeight,
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
                            "revisionNumber" | "revision_number" => Ok(GeneratedField::RevisionNumber),
                            "revisionHeight" | "revision_height" => Ok(GeneratedField::RevisionHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryChannelConsensusStateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryChannelConsensusStateRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryChannelConsensusStateRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut revision_number__ = None;
                let mut revision_height__ = None;
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
                        GeneratedField::RevisionNumber => {
                            if revision_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("revisionNumber"));
                            }
                            revision_number__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RevisionHeight => {
                            if revision_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("revisionHeight"));
                            }
                            revision_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryChannelConsensusStateRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    revision_number: revision_number__.unwrap_or_default(),
                    revision_height: revision_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryChannelConsensusStateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryChannelConsensusStateResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.consensus_state.is_some() {
            len += 1;
        }
        if !self.client_id.is_empty() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryChannelConsensusStateResponse", len)?;
        if let Some(v) = self.consensus_state.as_ref() {
            struct_ser.serialize_field("consensusState", v)?;
        }
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryChannelConsensusStateResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "consensus_state",
            "consensusState",
            "client_id",
            "clientId",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConsensusState,
            ClientId,
            Proof,
            ProofHeight,
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
                            "consensusState" | "consensus_state" => Ok(GeneratedField::ConsensusState),
                            "clientId" | "client_id" => Ok(GeneratedField::ClientId),
                            "proof" => Ok(GeneratedField::Proof),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryChannelConsensusStateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryChannelConsensusStateResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryChannelConsensusStateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut consensus_state__ = None;
                let mut client_id__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConsensusState => {
                            if consensus_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusState"));
                            }
                            consensus_state__ = map.next_value()?;
                        }
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryChannelConsensusStateResponse {
                    consensus_state: consensus_state__,
                    client_id: client_id__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryChannelConsensusStateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryChannelRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryChannelRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryChannelRequest {
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
            type Value = QueryChannelRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryChannelRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryChannelRequest, V::Error>
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
                Ok(QueryChannelRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryChannelRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryChannelResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.channel.is_some() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryChannelResponse", len)?;
        if let Some(v) = self.channel.as_ref() {
            struct_ser.serialize_field("channel", v)?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryChannelResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channel",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Channel,
            Proof,
            ProofHeight,
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
                            "channel" => Ok(GeneratedField::Channel),
                            "proof" => Ok(GeneratedField::Proof),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryChannelResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryChannelResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryChannelResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channel__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Channel => {
                            if channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channel"));
                            }
                            channel__ = map.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryChannelResponse {
                    channel: channel__,
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryChannelResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryChannelsRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryChannelsRequest", len)?;
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryChannelsRequest {
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
            type Value = QueryChannelsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryChannelsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryChannelsRequest, V::Error>
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
                Ok(QueryChannelsRequest {
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryChannelsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryChannelsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.channels.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        if self.height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryChannelsResponse", len)?;
        if !self.channels.is_empty() {
            struct_ser.serialize_field("channels", &self.channels)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if let Some(v) = self.height.as_ref() {
            struct_ser.serialize_field("height", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryChannelsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channels",
            "pagination",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Channels,
            Pagination,
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
                            "channels" => Ok(GeneratedField::Channels),
                            "pagination" => Ok(GeneratedField::Pagination),
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
            type Value = QueryChannelsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryChannelsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryChannelsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channels__ = None;
                let mut pagination__ = None;
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Channels => {
                            if channels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channels"));
                            }
                            channels__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryChannelsResponse {
                    channels: channels__.unwrap_or_default(),
                    pagination: pagination__,
                    height: height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryChannelsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionChannelsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.connection.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryConnectionChannelsRequest", len)?;
        if !self.connection.is_empty() {
            struct_ser.serialize_field("connection", &self.connection)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConnectionChannelsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Connection,
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
                            "connection" => Ok(GeneratedField::Connection),
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
            type Value = QueryConnectionChannelsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryConnectionChannelsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionChannelsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Connection => {
                            if connection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connection"));
                            }
                            connection__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryConnectionChannelsRequest {
                    connection: connection__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryConnectionChannelsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionChannelsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.channels.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        if self.height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryConnectionChannelsResponse", len)?;
        if !self.channels.is_empty() {
            struct_ser.serialize_field("channels", &self.channels)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if let Some(v) = self.height.as_ref() {
            struct_ser.serialize_field("height", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConnectionChannelsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channels",
            "pagination",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Channels,
            Pagination,
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
                            "channels" => Ok(GeneratedField::Channels),
                            "pagination" => Ok(GeneratedField::Pagination),
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
            type Value = QueryConnectionChannelsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryConnectionChannelsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionChannelsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channels__ = None;
                let mut pagination__ = None;
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Channels => {
                            if channels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channels"));
                            }
                            channels__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryConnectionChannelsResponse {
                    channels: channels__.unwrap_or_default(),
                    pagination: pagination__,
                    height: height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryConnectionChannelsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryNextSequenceReceiveRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryNextSequenceReceiveRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryNextSequenceReceiveRequest {
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
            type Value = QueryNextSequenceReceiveRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryNextSequenceReceiveRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryNextSequenceReceiveRequest, V::Error>
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
                Ok(QueryNextSequenceReceiveRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryNextSequenceReceiveRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryNextSequenceReceiveResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.next_sequence_receive != 0 {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryNextSequenceReceiveResponse", len)?;
        if self.next_sequence_receive != 0 {
            struct_ser.serialize_field("nextSequenceReceive", ToString::to_string(&self.next_sequence_receive).as_str())?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryNextSequenceReceiveResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "next_sequence_receive",
            "nextSequenceReceive",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NextSequenceReceive,
            Proof,
            ProofHeight,
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
                            "nextSequenceReceive" | "next_sequence_receive" => Ok(GeneratedField::NextSequenceReceive),
                            "proof" => Ok(GeneratedField::Proof),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryNextSequenceReceiveResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryNextSequenceReceiveResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryNextSequenceReceiveResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut next_sequence_receive__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NextSequenceReceive => {
                            if next_sequence_receive__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextSequenceReceive"));
                            }
                            next_sequence_receive__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryNextSequenceReceiveResponse {
                    next_sequence_receive: next_sequence_receive__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryNextSequenceReceiveResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryNextSequenceSendRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryNextSequenceSendRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryNextSequenceSendRequest {
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
            type Value = QueryNextSequenceSendRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryNextSequenceSendRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryNextSequenceSendRequest, V::Error>
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
                Ok(QueryNextSequenceSendRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryNextSequenceSendRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryNextSequenceSendResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.next_sequence_send != 0 {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryNextSequenceSendResponse", len)?;
        if self.next_sequence_send != 0 {
            struct_ser.serialize_field("nextSequenceSend", ToString::to_string(&self.next_sequence_send).as_str())?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryNextSequenceSendResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "next_sequence_send",
            "nextSequenceSend",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NextSequenceSend,
            Proof,
            ProofHeight,
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
                            "nextSequenceSend" | "next_sequence_send" => Ok(GeneratedField::NextSequenceSend),
                            "proof" => Ok(GeneratedField::Proof),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryNextSequenceSendResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryNextSequenceSendResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryNextSequenceSendResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut next_sequence_send__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NextSequenceSend => {
                            if next_sequence_send__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextSequenceSend"));
                            }
                            next_sequence_send__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryNextSequenceSendResponse {
                    next_sequence_send: next_sequence_send__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryNextSequenceSendResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketAcknowledgementRequest {
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
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketAcknowledgementRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketAcknowledgementRequest {
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
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
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
            type Value = QueryPacketAcknowledgementRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketAcknowledgementRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketAcknowledgementRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut sequence__ = None;
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
                Ok(QueryPacketAcknowledgementRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketAcknowledgementRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketAcknowledgementResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.acknowledgement.is_empty() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketAcknowledgementResponse", len)?;
        if !self.acknowledgement.is_empty() {
            struct_ser.serialize_field("acknowledgement", pbjson::private::base64::encode(&self.acknowledgement).as_str())?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketAcknowledgementResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "acknowledgement",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Acknowledgement,
            Proof,
            ProofHeight,
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
                            "acknowledgement" => Ok(GeneratedField::Acknowledgement),
                            "proof" => Ok(GeneratedField::Proof),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryPacketAcknowledgementResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketAcknowledgementResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketAcknowledgementResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut acknowledgement__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Acknowledgement => {
                            if acknowledgement__.is_some() {
                                return Err(serde::de::Error::duplicate_field("acknowledgement"));
                            }
                            acknowledgement__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryPacketAcknowledgementResponse {
                    acknowledgement: acknowledgement__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketAcknowledgementResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketAcknowledgementsRequest {
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
        if self.pagination.is_some() {
            len += 1;
        }
        if !self.packet_commitment_sequences.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketAcknowledgementsRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if !self.packet_commitment_sequences.is_empty() {
            struct_ser.serialize_field("packetCommitmentSequences", &self.packet_commitment_sequences.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketAcknowledgementsRequest {
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
            "pagination",
            "packet_commitment_sequences",
            "packetCommitmentSequences",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            Pagination,
            PacketCommitmentSequences,
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
                            "pagination" => Ok(GeneratedField::Pagination),
                            "packetCommitmentSequences" | "packet_commitment_sequences" => Ok(GeneratedField::PacketCommitmentSequences),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryPacketAcknowledgementsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketAcknowledgementsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketAcknowledgementsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut pagination__ = None;
                let mut packet_commitment_sequences__ = None;
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
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                        GeneratedField::PacketCommitmentSequences => {
                            if packet_commitment_sequences__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetCommitmentSequences"));
                            }
                            packet_commitment_sequences__ = 
                                Some(map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(QueryPacketAcknowledgementsRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    pagination: pagination__,
                    packet_commitment_sequences: packet_commitment_sequences__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketAcknowledgementsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketAcknowledgementsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.acknowledgements.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        if self.height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketAcknowledgementsResponse", len)?;
        if !self.acknowledgements.is_empty() {
            struct_ser.serialize_field("acknowledgements", &self.acknowledgements)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if let Some(v) = self.height.as_ref() {
            struct_ser.serialize_field("height", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketAcknowledgementsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "acknowledgements",
            "pagination",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Acknowledgements,
            Pagination,
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
                            "acknowledgements" => Ok(GeneratedField::Acknowledgements),
                            "pagination" => Ok(GeneratedField::Pagination),
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
            type Value = QueryPacketAcknowledgementsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketAcknowledgementsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketAcknowledgementsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut acknowledgements__ = None;
                let mut pagination__ = None;
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Acknowledgements => {
                            if acknowledgements__.is_some() {
                                return Err(serde::de::Error::duplicate_field("acknowledgements"));
                            }
                            acknowledgements__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryPacketAcknowledgementsResponse {
                    acknowledgements: acknowledgements__.unwrap_or_default(),
                    pagination: pagination__,
                    height: height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketAcknowledgementsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketCommitmentRequest {
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
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketCommitmentRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketCommitmentRequest {
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
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
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
            type Value = QueryPacketCommitmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketCommitmentRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketCommitmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut sequence__ = None;
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
                Ok(QueryPacketCommitmentRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketCommitmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketCommitmentResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.commitment.is_empty() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketCommitmentResponse", len)?;
        if !self.commitment.is_empty() {
            struct_ser.serialize_field("commitment", pbjson::private::base64::encode(&self.commitment).as_str())?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketCommitmentResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commitment",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commitment,
            Proof,
            ProofHeight,
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
                            "commitment" => Ok(GeneratedField::Commitment),
                            "proof" => Ok(GeneratedField::Proof),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryPacketCommitmentResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketCommitmentResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketCommitmentResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitment__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryPacketCommitmentResponse {
                    commitment: commitment__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketCommitmentResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketCommitmentsRequest {
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
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketCommitmentsRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketCommitmentsRequest {
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
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
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
            type Value = QueryPacketCommitmentsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketCommitmentsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketCommitmentsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut pagination__ = None;
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
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryPacketCommitmentsRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketCommitmentsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketCommitmentsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.commitments.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        if self.height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketCommitmentsResponse", len)?;
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if let Some(v) = self.height.as_ref() {
            struct_ser.serialize_field("height", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketCommitmentsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commitments",
            "pagination",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commitments,
            Pagination,
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
                            "commitments" => Ok(GeneratedField::Commitments),
                            "pagination" => Ok(GeneratedField::Pagination),
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
            type Value = QueryPacketCommitmentsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketCommitmentsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketCommitmentsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitments__ = None;
                let mut pagination__ = None;
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryPacketCommitmentsResponse {
                    commitments: commitments__.unwrap_or_default(),
                    pagination: pagination__,
                    height: height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketCommitmentsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketReceiptRequest {
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
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketReceiptRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.sequence != 0 {
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketReceiptRequest {
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
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
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
            type Value = QueryPacketReceiptRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketReceiptRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketReceiptRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut sequence__ = None;
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
                Ok(QueryPacketReceiptRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketReceiptRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPacketReceiptResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.received {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryPacketReceiptResponse", len)?;
        if self.received {
            struct_ser.serialize_field("received", &self.received)?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPacketReceiptResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "received",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Received,
            Proof,
            ProofHeight,
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
                            "received" => Ok(GeneratedField::Received),
                            "proof" => Ok(GeneratedField::Proof),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryPacketReceiptResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryPacketReceiptResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPacketReceiptResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut received__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Received => {
                            if received__.is_some() {
                                return Err(serde::de::Error::duplicate_field("received"));
                            }
                            received__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryPacketReceiptResponse {
                    received: received__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryPacketReceiptResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryUnreceivedAcksRequest {
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
        if !self.packet_ack_sequences.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryUnreceivedAcksRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.packet_ack_sequences.is_empty() {
            struct_ser.serialize_field("packetAckSequences", &self.packet_ack_sequences.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryUnreceivedAcksRequest {
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
            "packet_ack_sequences",
            "packetAckSequences",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            PacketAckSequences,
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
                            "packetAckSequences" | "packet_ack_sequences" => Ok(GeneratedField::PacketAckSequences),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryUnreceivedAcksRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryUnreceivedAcksRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryUnreceivedAcksRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut packet_ack_sequences__ = None;
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
                        GeneratedField::PacketAckSequences => {
                            if packet_ack_sequences__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetAckSequences"));
                            }
                            packet_ack_sequences__ = 
                                Some(map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(QueryUnreceivedAcksRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    packet_ack_sequences: packet_ack_sequences__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryUnreceivedAcksRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryUnreceivedAcksResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.sequences.is_empty() {
            len += 1;
        }
        if self.height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryUnreceivedAcksResponse", len)?;
        if !self.sequences.is_empty() {
            struct_ser.serialize_field("sequences", &self.sequences.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if let Some(v) = self.height.as_ref() {
            struct_ser.serialize_field("height", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryUnreceivedAcksResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sequences",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sequences,
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
                            "sequences" => Ok(GeneratedField::Sequences),
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
            type Value = QueryUnreceivedAcksResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryUnreceivedAcksResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryUnreceivedAcksResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sequences__ = None;
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Sequences => {
                            if sequences__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sequences"));
                            }
                            sequences__ = 
                                Some(map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryUnreceivedAcksResponse {
                    sequences: sequences__.unwrap_or_default(),
                    height: height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryUnreceivedAcksResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryUnreceivedPacketsRequest {
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
        if !self.packet_commitment_sequences.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryUnreceivedPacketsRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.packet_commitment_sequences.is_empty() {
            struct_ser.serialize_field("packetCommitmentSequences", &self.packet_commitment_sequences.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryUnreceivedPacketsRequest {
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
            "packet_commitment_sequences",
            "packetCommitmentSequences",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            PacketCommitmentSequences,
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
                            "packetCommitmentSequences" | "packet_commitment_sequences" => Ok(GeneratedField::PacketCommitmentSequences),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryUnreceivedPacketsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryUnreceivedPacketsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryUnreceivedPacketsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut packet_commitment_sequences__ = None;
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
                        GeneratedField::PacketCommitmentSequences => {
                            if packet_commitment_sequences__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetCommitmentSequences"));
                            }
                            packet_commitment_sequences__ = 
                                Some(map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(QueryUnreceivedPacketsRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    packet_commitment_sequences: packet_commitment_sequences__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryUnreceivedPacketsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryUnreceivedPacketsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.sequences.is_empty() {
            len += 1;
        }
        if self.height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.channel.v1.QueryUnreceivedPacketsResponse", len)?;
        if !self.sequences.is_empty() {
            struct_ser.serialize_field("sequences", &self.sequences.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if let Some(v) = self.height.as_ref() {
            struct_ser.serialize_field("height", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryUnreceivedPacketsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sequences",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sequences,
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
                            "sequences" => Ok(GeneratedField::Sequences),
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
            type Value = QueryUnreceivedPacketsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.channel.v1.QueryUnreceivedPacketsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryUnreceivedPacketsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sequences__ = None;
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Sequences => {
                            if sequences__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sequences"));
                            }
                            sequences__ = 
                                Some(map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryUnreceivedPacketsResponse {
                    sequences: sequences__.unwrap_or_default(),
                    height: height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.channel.v1.QueryUnreceivedPacketsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ResponseResultType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "RESPONSE_RESULT_TYPE_UNSPECIFIED",
            Self::Noop => "RESPONSE_RESULT_TYPE_NOOP",
            Self::Success => "RESPONSE_RESULT_TYPE_SUCCESS",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ResponseResultType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "RESPONSE_RESULT_TYPE_UNSPECIFIED",
            "RESPONSE_RESULT_TYPE_NOOP",
            "RESPONSE_RESULT_TYPE_SUCCESS",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ResponseResultType;

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
                    .and_then(ResponseResultType::from_i32)
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
                    .and_then(ResponseResultType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "RESPONSE_RESULT_TYPE_UNSPECIFIED" => Ok(ResponseResultType::Unspecified),
                    "RESPONSE_RESULT_TYPE_NOOP" => Ok(ResponseResultType::Noop),
                    "RESPONSE_RESULT_TYPE_SUCCESS" => Ok(ResponseResultType::Success),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for State {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::UninitializedUnspecified => "STATE_UNINITIALIZED_UNSPECIFIED",
            Self::Init => "STATE_INIT",
            Self::Tryopen => "STATE_TRYOPEN",
            Self::Open => "STATE_OPEN",
            Self::Closed => "STATE_CLOSED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for State {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "STATE_UNINITIALIZED_UNSPECIFIED",
            "STATE_INIT",
            "STATE_TRYOPEN",
            "STATE_OPEN",
            "STATE_CLOSED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = State;

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
                    .and_then(State::from_i32)
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
                    .and_then(State::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "STATE_UNINITIALIZED_UNSPECIFIED" => Ok(State::UninitializedUnspecified),
                    "STATE_INIT" => Ok(State::Init),
                    "STATE_TRYOPEN" => Ok(State::Tryopen),
                    "STATE_OPEN" => Ok(State::Open),
                    "STATE_CLOSED" => Ok(State::Closed),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
