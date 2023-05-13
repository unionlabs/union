// @generated
impl serde::Serialize for Fee {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.recv_fee.is_empty() {
            len += 1;
        }
        if !self.ack_fee.is_empty() {
            len += 1;
        }
        if !self.timeout_fee.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.Fee", len)?;
        if !self.recv_fee.is_empty() {
            struct_ser.serialize_field("recvFee", &self.recv_fee)?;
        }
        if !self.ack_fee.is_empty() {
            struct_ser.serialize_field("ackFee", &self.ack_fee)?;
        }
        if !self.timeout_fee.is_empty() {
            struct_ser.serialize_field("timeoutFee", &self.timeout_fee)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Fee {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "recv_fee",
            "recvFee",
            "ack_fee",
            "ackFee",
            "timeout_fee",
            "timeoutFee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RecvFee,
            AckFee,
            TimeoutFee,
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
                            "recvFee" | "recv_fee" => Ok(GeneratedField::RecvFee),
                            "ackFee" | "ack_fee" => Ok(GeneratedField::AckFee),
                            "timeoutFee" | "timeout_fee" => Ok(GeneratedField::TimeoutFee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Fee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.Fee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Fee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut recv_fee__ = None;
                let mut ack_fee__ = None;
                let mut timeout_fee__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RecvFee => {
                            if recv_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recvFee"));
                            }
                            recv_fee__ = Some(map.next_value()?);
                        }
                        GeneratedField::AckFee => {
                            if ack_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ackFee"));
                            }
                            ack_fee__ = Some(map.next_value()?);
                        }
                        GeneratedField::TimeoutFee => {
                            if timeout_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutFee"));
                            }
                            timeout_fee__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Fee {
                    recv_fee: recv_fee__.unwrap_or_default(),
                    ack_fee: ack_fee__.unwrap_or_default(),
                    timeout_fee: timeout_fee__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.Fee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FeeEnabledChannel {
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
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.FeeEnabledChannel", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FeeEnabledChannel {
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
            type Value = FeeEnabledChannel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.FeeEnabledChannel")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FeeEnabledChannel, V::Error>
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
                Ok(FeeEnabledChannel {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.FeeEnabledChannel", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ForwardRelayerAddress {
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
        if self.packet_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.ForwardRelayerAddress", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if let Some(v) = self.packet_id.as_ref() {
            struct_ser.serialize_field("packetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ForwardRelayerAddress {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "packet_id",
            "packetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            PacketId,
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
                            "packetId" | "packet_id" => Ok(GeneratedField::PacketId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ForwardRelayerAddress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.ForwardRelayerAddress")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ForwardRelayerAddress, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut packet_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::PacketId => {
                            if packet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetId"));
                            }
                            packet_id__ = map.next_value()?;
                        }
                    }
                }
                Ok(ForwardRelayerAddress {
                    address: address__.unwrap_or_default(),
                    packet_id: packet_id__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.ForwardRelayerAddress", FIELDS, GeneratedVisitor)
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
        if !self.identified_fees.is_empty() {
            len += 1;
        }
        if !self.fee_enabled_channels.is_empty() {
            len += 1;
        }
        if !self.registered_payees.is_empty() {
            len += 1;
        }
        if !self.registered_counterparty_payees.is_empty() {
            len += 1;
        }
        if !self.forward_relayers.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.GenesisState", len)?;
        if !self.identified_fees.is_empty() {
            struct_ser.serialize_field("identifiedFees", &self.identified_fees)?;
        }
        if !self.fee_enabled_channels.is_empty() {
            struct_ser.serialize_field("feeEnabledChannels", &self.fee_enabled_channels)?;
        }
        if !self.registered_payees.is_empty() {
            struct_ser.serialize_field("registeredPayees", &self.registered_payees)?;
        }
        if !self.registered_counterparty_payees.is_empty() {
            struct_ser.serialize_field("registeredCounterpartyPayees", &self.registered_counterparty_payees)?;
        }
        if !self.forward_relayers.is_empty() {
            struct_ser.serialize_field("forwardRelayers", &self.forward_relayers)?;
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
            "identified_fees",
            "identifiedFees",
            "fee_enabled_channels",
            "feeEnabledChannels",
            "registered_payees",
            "registeredPayees",
            "registered_counterparty_payees",
            "registeredCounterpartyPayees",
            "forward_relayers",
            "forwardRelayers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IdentifiedFees,
            FeeEnabledChannels,
            RegisteredPayees,
            RegisteredCounterpartyPayees,
            ForwardRelayers,
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
                            "identifiedFees" | "identified_fees" => Ok(GeneratedField::IdentifiedFees),
                            "feeEnabledChannels" | "fee_enabled_channels" => Ok(GeneratedField::FeeEnabledChannels),
                            "registeredPayees" | "registered_payees" => Ok(GeneratedField::RegisteredPayees),
                            "registeredCounterpartyPayees" | "registered_counterparty_payees" => Ok(GeneratedField::RegisteredCounterpartyPayees),
                            "forwardRelayers" | "forward_relayers" => Ok(GeneratedField::ForwardRelayers),
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
                formatter.write_str("struct ibc.applications.fee.v1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identified_fees__ = None;
                let mut fee_enabled_channels__ = None;
                let mut registered_payees__ = None;
                let mut registered_counterparty_payees__ = None;
                let mut forward_relayers__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::IdentifiedFees => {
                            if identified_fees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identifiedFees"));
                            }
                            identified_fees__ = Some(map.next_value()?);
                        }
                        GeneratedField::FeeEnabledChannels => {
                            if fee_enabled_channels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeEnabledChannels"));
                            }
                            fee_enabled_channels__ = Some(map.next_value()?);
                        }
                        GeneratedField::RegisteredPayees => {
                            if registered_payees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("registeredPayees"));
                            }
                            registered_payees__ = Some(map.next_value()?);
                        }
                        GeneratedField::RegisteredCounterpartyPayees => {
                            if registered_counterparty_payees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("registeredCounterpartyPayees"));
                            }
                            registered_counterparty_payees__ = Some(map.next_value()?);
                        }
                        GeneratedField::ForwardRelayers => {
                            if forward_relayers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("forwardRelayers"));
                            }
                            forward_relayers__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenesisState {
                    identified_fees: identified_fees__.unwrap_or_default(),
                    fee_enabled_channels: fee_enabled_channels__.unwrap_or_default(),
                    registered_payees: registered_payees__.unwrap_or_default(),
                    registered_counterparty_payees: registered_counterparty_payees__.unwrap_or_default(),
                    forward_relayers: forward_relayers__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IdentifiedPacketFees {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet_id.is_some() {
            len += 1;
        }
        if !self.packet_fees.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.IdentifiedPacketFees", len)?;
        if let Some(v) = self.packet_id.as_ref() {
            struct_ser.serialize_field("packetId", v)?;
        }
        if !self.packet_fees.is_empty() {
            struct_ser.serialize_field("packetFees", &self.packet_fees)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IdentifiedPacketFees {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet_id",
            "packetId",
            "packet_fees",
            "packetFees",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PacketId,
            PacketFees,
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
                            "packetId" | "packet_id" => Ok(GeneratedField::PacketId),
                            "packetFees" | "packet_fees" => Ok(GeneratedField::PacketFees),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IdentifiedPacketFees;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.IdentifiedPacketFees")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<IdentifiedPacketFees, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet_id__ = None;
                let mut packet_fees__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PacketId => {
                            if packet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetId"));
                            }
                            packet_id__ = map.next_value()?;
                        }
                        GeneratedField::PacketFees => {
                            if packet_fees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetFees"));
                            }
                            packet_fees__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(IdentifiedPacketFees {
                    packet_id: packet_id__,
                    packet_fees: packet_fees__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.IdentifiedPacketFees", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IncentivizedAcknowledgement {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.app_acknowledgement.is_empty() {
            len += 1;
        }
        if !self.forward_relayer_address.is_empty() {
            len += 1;
        }
        if self.underlying_app_success {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.IncentivizedAcknowledgement", len)?;
        if !self.app_acknowledgement.is_empty() {
            struct_ser.serialize_field("appAcknowledgement", pbjson::private::base64::encode(&self.app_acknowledgement).as_str())?;
        }
        if !self.forward_relayer_address.is_empty() {
            struct_ser.serialize_field("forwardRelayerAddress", &self.forward_relayer_address)?;
        }
        if self.underlying_app_success {
            struct_ser.serialize_field("underlyingAppSuccess", &self.underlying_app_success)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IncentivizedAcknowledgement {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "app_acknowledgement",
            "appAcknowledgement",
            "forward_relayer_address",
            "forwardRelayerAddress",
            "underlying_app_success",
            "underlyingAppSuccess",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AppAcknowledgement,
            ForwardRelayerAddress,
            UnderlyingAppSuccess,
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
                            "appAcknowledgement" | "app_acknowledgement" => Ok(GeneratedField::AppAcknowledgement),
                            "forwardRelayerAddress" | "forward_relayer_address" => Ok(GeneratedField::ForwardRelayerAddress),
                            "underlyingAppSuccess" | "underlying_app_success" => Ok(GeneratedField::UnderlyingAppSuccess),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IncentivizedAcknowledgement;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.IncentivizedAcknowledgement")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<IncentivizedAcknowledgement, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut app_acknowledgement__ = None;
                let mut forward_relayer_address__ = None;
                let mut underlying_app_success__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AppAcknowledgement => {
                            if app_acknowledgement__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appAcknowledgement"));
                            }
                            app_acknowledgement__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ForwardRelayerAddress => {
                            if forward_relayer_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("forwardRelayerAddress"));
                            }
                            forward_relayer_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::UnderlyingAppSuccess => {
                            if underlying_app_success__.is_some() {
                                return Err(serde::de::Error::duplicate_field("underlyingAppSuccess"));
                            }
                            underlying_app_success__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(IncentivizedAcknowledgement {
                    app_acknowledgement: app_acknowledgement__.unwrap_or_default(),
                    forward_relayer_address: forward_relayer_address__.unwrap_or_default(),
                    underlying_app_success: underlying_app_success__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.IncentivizedAcknowledgement", FIELDS, GeneratedVisitor)
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
        if !self.fee_version.is_empty() {
            len += 1;
        }
        if !self.app_version.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.Metadata", len)?;
        if !self.fee_version.is_empty() {
            struct_ser.serialize_field("feeVersion", &self.fee_version)?;
        }
        if !self.app_version.is_empty() {
            struct_ser.serialize_field("appVersion", &self.app_version)?;
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
            "fee_version",
            "feeVersion",
            "app_version",
            "appVersion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FeeVersion,
            AppVersion,
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
                            "feeVersion" | "fee_version" => Ok(GeneratedField::FeeVersion),
                            "appVersion" | "app_version" => Ok(GeneratedField::AppVersion),
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
                formatter.write_str("struct ibc.applications.fee.v1.Metadata")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Metadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee_version__ = None;
                let mut app_version__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FeeVersion => {
                            if fee_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeVersion"));
                            }
                            fee_version__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppVersion => {
                            if app_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appVersion"));
                            }
                            app_version__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Metadata {
                    fee_version: fee_version__.unwrap_or_default(),
                    app_version: app_version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.Metadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgPayPacketFee {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fee.is_some() {
            len += 1;
        }
        if !self.source_port_id.is_empty() {
            len += 1;
        }
        if !self.source_channel_id.is_empty() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        if !self.relayers.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.MsgPayPacketFee", len)?;
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if !self.source_port_id.is_empty() {
            struct_ser.serialize_field("sourcePortId", &self.source_port_id)?;
        }
        if !self.source_channel_id.is_empty() {
            struct_ser.serialize_field("sourceChannelId", &self.source_channel_id)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        if !self.relayers.is_empty() {
            struct_ser.serialize_field("relayers", &self.relayers)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgPayPacketFee {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee",
            "source_port_id",
            "sourcePortId",
            "source_channel_id",
            "sourceChannelId",
            "signer",
            "relayers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fee,
            SourcePortId,
            SourceChannelId,
            Signer,
            Relayers,
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
                            "fee" => Ok(GeneratedField::Fee),
                            "sourcePortId" | "source_port_id" => Ok(GeneratedField::SourcePortId),
                            "sourceChannelId" | "source_channel_id" => Ok(GeneratedField::SourceChannelId),
                            "signer" => Ok(GeneratedField::Signer),
                            "relayers" => Ok(GeneratedField::Relayers),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgPayPacketFee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.MsgPayPacketFee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgPayPacketFee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee__ = None;
                let mut source_port_id__ = None;
                let mut source_channel_id__ = None;
                let mut signer__ = None;
                let mut relayers__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map.next_value()?;
                        }
                        GeneratedField::SourcePortId => {
                            if source_port_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourcePortId"));
                            }
                            source_port_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::SourceChannelId => {
                            if source_channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceChannelId"));
                            }
                            source_channel_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                        GeneratedField::Relayers => {
                            if relayers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("relayers"));
                            }
                            relayers__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgPayPacketFee {
                    fee: fee__,
                    source_port_id: source_port_id__.unwrap_or_default(),
                    source_channel_id: source_channel_id__.unwrap_or_default(),
                    signer: signer__.unwrap_or_default(),
                    relayers: relayers__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.MsgPayPacketFee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgPayPacketFeeAsync {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet_id.is_some() {
            len += 1;
        }
        if self.packet_fee.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.MsgPayPacketFeeAsync", len)?;
        if let Some(v) = self.packet_id.as_ref() {
            struct_ser.serialize_field("packetId", v)?;
        }
        if let Some(v) = self.packet_fee.as_ref() {
            struct_ser.serialize_field("packetFee", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgPayPacketFeeAsync {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet_id",
            "packetId",
            "packet_fee",
            "packetFee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PacketId,
            PacketFee,
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
                            "packetId" | "packet_id" => Ok(GeneratedField::PacketId),
                            "packetFee" | "packet_fee" => Ok(GeneratedField::PacketFee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgPayPacketFeeAsync;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.MsgPayPacketFeeAsync")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgPayPacketFeeAsync, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet_id__ = None;
                let mut packet_fee__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PacketId => {
                            if packet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetId"));
                            }
                            packet_id__ = map.next_value()?;
                        }
                        GeneratedField::PacketFee => {
                            if packet_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetFee"));
                            }
                            packet_fee__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgPayPacketFeeAsync {
                    packet_id: packet_id__,
                    packet_fee: packet_fee__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.MsgPayPacketFeeAsync", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgPayPacketFeeAsyncResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.MsgPayPacketFeeAsyncResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgPayPacketFeeAsyncResponse {
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
            type Value = MsgPayPacketFeeAsyncResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.MsgPayPacketFeeAsyncResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgPayPacketFeeAsyncResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgPayPacketFeeAsyncResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.MsgPayPacketFeeAsyncResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgPayPacketFeeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.MsgPayPacketFeeResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgPayPacketFeeResponse {
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
            type Value = MsgPayPacketFeeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.MsgPayPacketFeeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgPayPacketFeeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgPayPacketFeeResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.MsgPayPacketFeeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRegisterCounterpartyPayee {
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
        if !self.relayer.is_empty() {
            len += 1;
        }
        if !self.counterparty_payee.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.MsgRegisterCounterpartyPayee", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.relayer.is_empty() {
            struct_ser.serialize_field("relayer", &self.relayer)?;
        }
        if !self.counterparty_payee.is_empty() {
            struct_ser.serialize_field("counterpartyPayee", &self.counterparty_payee)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRegisterCounterpartyPayee {
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
            "relayer",
            "counterparty_payee",
            "counterpartyPayee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            Relayer,
            CounterpartyPayee,
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
                            "relayer" => Ok(GeneratedField::Relayer),
                            "counterpartyPayee" | "counterparty_payee" => Ok(GeneratedField::CounterpartyPayee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgRegisterCounterpartyPayee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.MsgRegisterCounterpartyPayee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgRegisterCounterpartyPayee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut relayer__ = None;
                let mut counterparty_payee__ = None;
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
                        GeneratedField::Relayer => {
                            if relayer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("relayer"));
                            }
                            relayer__ = Some(map.next_value()?);
                        }
                        GeneratedField::CounterpartyPayee => {
                            if counterparty_payee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterpartyPayee"));
                            }
                            counterparty_payee__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgRegisterCounterpartyPayee {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    relayer: relayer__.unwrap_or_default(),
                    counterparty_payee: counterparty_payee__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.MsgRegisterCounterpartyPayee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRegisterCounterpartyPayeeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.MsgRegisterCounterpartyPayeeResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRegisterCounterpartyPayeeResponse {
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
            type Value = MsgRegisterCounterpartyPayeeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.MsgRegisterCounterpartyPayeeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgRegisterCounterpartyPayeeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgRegisterCounterpartyPayeeResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.MsgRegisterCounterpartyPayeeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRegisterPayee {
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
        if !self.relayer.is_empty() {
            len += 1;
        }
        if !self.payee.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.MsgRegisterPayee", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.relayer.is_empty() {
            struct_ser.serialize_field("relayer", &self.relayer)?;
        }
        if !self.payee.is_empty() {
            struct_ser.serialize_field("payee", &self.payee)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRegisterPayee {
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
            "relayer",
            "payee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PortId,
            ChannelId,
            Relayer,
            Payee,
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
                            "relayer" => Ok(GeneratedField::Relayer),
                            "payee" => Ok(GeneratedField::Payee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgRegisterPayee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.MsgRegisterPayee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgRegisterPayee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut relayer__ = None;
                let mut payee__ = None;
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
                        GeneratedField::Relayer => {
                            if relayer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("relayer"));
                            }
                            relayer__ = Some(map.next_value()?);
                        }
                        GeneratedField::Payee => {
                            if payee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payee"));
                            }
                            payee__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgRegisterPayee {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    relayer: relayer__.unwrap_or_default(),
                    payee: payee__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.MsgRegisterPayee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRegisterPayeeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.MsgRegisterPayeeResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRegisterPayeeResponse {
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
            type Value = MsgRegisterPayeeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.MsgRegisterPayeeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgRegisterPayeeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgRegisterPayeeResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.MsgRegisterPayeeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketFee {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fee.is_some() {
            len += 1;
        }
        if !self.refund_address.is_empty() {
            len += 1;
        }
        if !self.relayers.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.PacketFee", len)?;
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if !self.refund_address.is_empty() {
            struct_ser.serialize_field("refundAddress", &self.refund_address)?;
        }
        if !self.relayers.is_empty() {
            struct_ser.serialize_field("relayers", &self.relayers)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketFee {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee",
            "refund_address",
            "refundAddress",
            "relayers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fee,
            RefundAddress,
            Relayers,
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
                            "fee" => Ok(GeneratedField::Fee),
                            "refundAddress" | "refund_address" => Ok(GeneratedField::RefundAddress),
                            "relayers" => Ok(GeneratedField::Relayers),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PacketFee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.PacketFee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketFee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee__ = None;
                let mut refund_address__ = None;
                let mut relayers__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map.next_value()?;
                        }
                        GeneratedField::RefundAddress => {
                            if refund_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("refundAddress"));
                            }
                            refund_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Relayers => {
                            if relayers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("relayers"));
                            }
                            relayers__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PacketFee {
                    fee: fee__,
                    refund_address: refund_address__.unwrap_or_default(),
                    relayers: relayers__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.PacketFee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketFees {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.packet_fees.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.PacketFees", len)?;
        if !self.packet_fees.is_empty() {
            struct_ser.serialize_field("packetFees", &self.packet_fees)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketFees {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet_fees",
            "packetFees",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PacketFees,
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
                            "packetFees" | "packet_fees" => Ok(GeneratedField::PacketFees),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PacketFees;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.PacketFees")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketFees, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet_fees__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PacketFees => {
                            if packet_fees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetFees"));
                            }
                            packet_fees__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PacketFees {
                    packet_fees: packet_fees__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.PacketFees", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryCounterpartyPayeeRequest {
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
        if !self.relayer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryCounterpartyPayeeRequest", len)?;
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.relayer.is_empty() {
            struct_ser.serialize_field("relayer", &self.relayer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryCounterpartyPayeeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channel_id",
            "channelId",
            "relayer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChannelId,
            Relayer,
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
                            "relayer" => Ok(GeneratedField::Relayer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryCounterpartyPayeeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryCounterpartyPayeeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryCounterpartyPayeeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channel_id__ = None;
                let mut relayer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChannelId => {
                            if channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelId"));
                            }
                            channel_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Relayer => {
                            if relayer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("relayer"));
                            }
                            relayer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryCounterpartyPayeeRequest {
                    channel_id: channel_id__.unwrap_or_default(),
                    relayer: relayer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryCounterpartyPayeeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryCounterpartyPayeeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.counterparty_payee.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryCounterpartyPayeeResponse", len)?;
        if !self.counterparty_payee.is_empty() {
            struct_ser.serialize_field("counterpartyPayee", &self.counterparty_payee)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryCounterpartyPayeeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "counterparty_payee",
            "counterpartyPayee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CounterpartyPayee,
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
                            "counterpartyPayee" | "counterparty_payee" => Ok(GeneratedField::CounterpartyPayee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryCounterpartyPayeeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryCounterpartyPayeeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryCounterpartyPayeeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut counterparty_payee__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CounterpartyPayee => {
                            if counterparty_payee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterpartyPayee"));
                            }
                            counterparty_payee__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryCounterpartyPayeeResponse {
                    counterparty_payee: counterparty_payee__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryCounterpartyPayeeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryFeeEnabledChannelRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryFeeEnabledChannelRequest", len)?;
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryFeeEnabledChannelRequest {
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
            type Value = QueryFeeEnabledChannelRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryFeeEnabledChannelRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryFeeEnabledChannelRequest, V::Error>
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
                Ok(QueryFeeEnabledChannelRequest {
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryFeeEnabledChannelRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryFeeEnabledChannelResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fee_enabled {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryFeeEnabledChannelResponse", len)?;
        if self.fee_enabled {
            struct_ser.serialize_field("feeEnabled", &self.fee_enabled)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryFeeEnabledChannelResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee_enabled",
            "feeEnabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FeeEnabled,
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
                            "feeEnabled" | "fee_enabled" => Ok(GeneratedField::FeeEnabled),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryFeeEnabledChannelResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryFeeEnabledChannelResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryFeeEnabledChannelResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee_enabled__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FeeEnabled => {
                            if fee_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeEnabled"));
                            }
                            fee_enabled__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryFeeEnabledChannelResponse {
                    fee_enabled: fee_enabled__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryFeeEnabledChannelResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryFeeEnabledChannelsRequest {
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
        if self.query_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryFeeEnabledChannelsRequest", len)?;
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if self.query_height != 0 {
            struct_ser.serialize_field("queryHeight", ToString::to_string(&self.query_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryFeeEnabledChannelsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pagination",
            "query_height",
            "queryHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pagination,
            QueryHeight,
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
                            "queryHeight" | "query_height" => Ok(GeneratedField::QueryHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryFeeEnabledChannelsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryFeeEnabledChannelsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryFeeEnabledChannelsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pagination__ = None;
                let mut query_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                        GeneratedField::QueryHeight => {
                            if query_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("queryHeight"));
                            }
                            query_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryFeeEnabledChannelsRequest {
                    pagination: pagination__,
                    query_height: query_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryFeeEnabledChannelsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryFeeEnabledChannelsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fee_enabled_channels.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryFeeEnabledChannelsResponse", len)?;
        if !self.fee_enabled_channels.is_empty() {
            struct_ser.serialize_field("feeEnabledChannels", &self.fee_enabled_channels)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryFeeEnabledChannelsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee_enabled_channels",
            "feeEnabledChannels",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FeeEnabledChannels,
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
                            "feeEnabledChannels" | "fee_enabled_channels" => Ok(GeneratedField::FeeEnabledChannels),
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
            type Value = QueryFeeEnabledChannelsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryFeeEnabledChannelsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryFeeEnabledChannelsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee_enabled_channels__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FeeEnabledChannels => {
                            if fee_enabled_channels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeEnabledChannels"));
                            }
                            fee_enabled_channels__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryFeeEnabledChannelsResponse {
                    fee_enabled_channels: fee_enabled_channels__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryFeeEnabledChannelsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryIncentivizedPacketRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet_id.is_some() {
            len += 1;
        }
        if self.query_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketRequest", len)?;
        if let Some(v) = self.packet_id.as_ref() {
            struct_ser.serialize_field("packetId", v)?;
        }
        if self.query_height != 0 {
            struct_ser.serialize_field("queryHeight", ToString::to_string(&self.query_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryIncentivizedPacketRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet_id",
            "packetId",
            "query_height",
            "queryHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PacketId,
            QueryHeight,
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
                            "packetId" | "packet_id" => Ok(GeneratedField::PacketId),
                            "queryHeight" | "query_height" => Ok(GeneratedField::QueryHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryIncentivizedPacketRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryIncentivizedPacketRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryIncentivizedPacketRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet_id__ = None;
                let mut query_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PacketId => {
                            if packet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetId"));
                            }
                            packet_id__ = map.next_value()?;
                        }
                        GeneratedField::QueryHeight => {
                            if query_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("queryHeight"));
                            }
                            query_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryIncentivizedPacketRequest {
                    packet_id: packet_id__,
                    query_height: query_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryIncentivizedPacketResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.incentivized_packet.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketResponse", len)?;
        if let Some(v) = self.incentivized_packet.as_ref() {
            struct_ser.serialize_field("incentivizedPacket", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryIncentivizedPacketResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "incentivized_packet",
            "incentivizedPacket",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IncentivizedPacket,
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
                            "incentivizedPacket" | "incentivized_packet" => Ok(GeneratedField::IncentivizedPacket),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryIncentivizedPacketResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryIncentivizedPacketResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryIncentivizedPacketResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut incentivized_packet__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::IncentivizedPacket => {
                            if incentivized_packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivizedPacket"));
                            }
                            incentivized_packet__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryIncentivizedPacketResponse {
                    incentivized_packet: incentivized_packet__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryIncentivizedPacketsForChannelRequest {
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
        if !self.port_id.is_empty() {
            len += 1;
        }
        if !self.channel_id.is_empty() {
            len += 1;
        }
        if self.query_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketsForChannelRequest", len)?;
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.query_height != 0 {
            struct_ser.serialize_field("queryHeight", ToString::to_string(&self.query_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryIncentivizedPacketsForChannelRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pagination",
            "port_id",
            "portId",
            "channel_id",
            "channelId",
            "query_height",
            "queryHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pagination,
            PortId,
            ChannelId,
            QueryHeight,
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
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
                            "queryHeight" | "query_height" => Ok(GeneratedField::QueryHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryIncentivizedPacketsForChannelRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryIncentivizedPacketsForChannelRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryIncentivizedPacketsForChannelRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pagination__ = None;
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut query_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
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
                        GeneratedField::QueryHeight => {
                            if query_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("queryHeight"));
                            }
                            query_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryIncentivizedPacketsForChannelRequest {
                    pagination: pagination__,
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    query_height: query_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketsForChannelRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryIncentivizedPacketsForChannelResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.incentivized_packets.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketsForChannelResponse", len)?;
        if !self.incentivized_packets.is_empty() {
            struct_ser.serialize_field("incentivizedPackets", &self.incentivized_packets)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryIncentivizedPacketsForChannelResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "incentivized_packets",
            "incentivizedPackets",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IncentivizedPackets,
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
                            "incentivizedPackets" | "incentivized_packets" => Ok(GeneratedField::IncentivizedPackets),
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
            type Value = QueryIncentivizedPacketsForChannelResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryIncentivizedPacketsForChannelResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryIncentivizedPacketsForChannelResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut incentivized_packets__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::IncentivizedPackets => {
                            if incentivized_packets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivizedPackets"));
                            }
                            incentivized_packets__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryIncentivizedPacketsForChannelResponse {
                    incentivized_packets: incentivized_packets__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketsForChannelResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryIncentivizedPacketsRequest {
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
        if self.query_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketsRequest", len)?;
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        if self.query_height != 0 {
            struct_ser.serialize_field("queryHeight", ToString::to_string(&self.query_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryIncentivizedPacketsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pagination",
            "query_height",
            "queryHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pagination,
            QueryHeight,
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
                            "queryHeight" | "query_height" => Ok(GeneratedField::QueryHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryIncentivizedPacketsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryIncentivizedPacketsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryIncentivizedPacketsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pagination__ = None;
                let mut query_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                        GeneratedField::QueryHeight => {
                            if query_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("queryHeight"));
                            }
                            query_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryIncentivizedPacketsRequest {
                    pagination: pagination__,
                    query_height: query_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryIncentivizedPacketsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.incentivized_packets.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketsResponse", len)?;
        if !self.incentivized_packets.is_empty() {
            struct_ser.serialize_field("incentivizedPackets", &self.incentivized_packets)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryIncentivizedPacketsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "incentivized_packets",
            "incentivizedPackets",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IncentivizedPackets,
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
                            "incentivizedPackets" | "incentivized_packets" => Ok(GeneratedField::IncentivizedPackets),
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
            type Value = QueryIncentivizedPacketsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryIncentivizedPacketsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryIncentivizedPacketsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut incentivized_packets__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::IncentivizedPackets => {
                            if incentivized_packets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivizedPackets"));
                            }
                            incentivized_packets__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryIncentivizedPacketsResponse {
                    incentivized_packets: incentivized_packets__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryIncentivizedPacketsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPayeeRequest {
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
        if !self.relayer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryPayeeRequest", len)?;
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.relayer.is_empty() {
            struct_ser.serialize_field("relayer", &self.relayer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPayeeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channel_id",
            "channelId",
            "relayer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChannelId,
            Relayer,
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
                            "relayer" => Ok(GeneratedField::Relayer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryPayeeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryPayeeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPayeeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channel_id__ = None;
                let mut relayer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChannelId => {
                            if channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelId"));
                            }
                            channel_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Relayer => {
                            if relayer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("relayer"));
                            }
                            relayer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryPayeeRequest {
                    channel_id: channel_id__.unwrap_or_default(),
                    relayer: relayer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryPayeeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPayeeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.payee_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryPayeeResponse", len)?;
        if !self.payee_address.is_empty() {
            struct_ser.serialize_field("payeeAddress", &self.payee_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPayeeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payee_address",
            "payeeAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PayeeAddress,
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
                            "payeeAddress" | "payee_address" => Ok(GeneratedField::PayeeAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryPayeeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryPayeeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPayeeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payee_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PayeeAddress => {
                            if payee_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payeeAddress"));
                            }
                            payee_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryPayeeResponse {
                    payee_address: payee_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryPayeeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTotalAckFeesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryTotalAckFeesRequest", len)?;
        if let Some(v) = self.packet_id.as_ref() {
            struct_ser.serialize_field("packetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTotalAckFeesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet_id",
            "packetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PacketId,
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
                            "packetId" | "packet_id" => Ok(GeneratedField::PacketId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTotalAckFeesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryTotalAckFeesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTotalAckFeesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PacketId => {
                            if packet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetId"));
                            }
                            packet_id__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryTotalAckFeesRequest {
                    packet_id: packet_id__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryTotalAckFeesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTotalAckFeesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.ack_fees.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryTotalAckFeesResponse", len)?;
        if !self.ack_fees.is_empty() {
            struct_ser.serialize_field("ackFees", &self.ack_fees)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTotalAckFeesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ack_fees",
            "ackFees",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AckFees,
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
                            "ackFees" | "ack_fees" => Ok(GeneratedField::AckFees),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTotalAckFeesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryTotalAckFeesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTotalAckFeesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ack_fees__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AckFees => {
                            if ack_fees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ackFees"));
                            }
                            ack_fees__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryTotalAckFeesResponse {
                    ack_fees: ack_fees__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryTotalAckFeesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTotalRecvFeesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryTotalRecvFeesRequest", len)?;
        if let Some(v) = self.packet_id.as_ref() {
            struct_ser.serialize_field("packetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTotalRecvFeesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet_id",
            "packetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PacketId,
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
                            "packetId" | "packet_id" => Ok(GeneratedField::PacketId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTotalRecvFeesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryTotalRecvFeesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTotalRecvFeesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PacketId => {
                            if packet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetId"));
                            }
                            packet_id__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryTotalRecvFeesRequest {
                    packet_id: packet_id__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryTotalRecvFeesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTotalRecvFeesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.recv_fees.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryTotalRecvFeesResponse", len)?;
        if !self.recv_fees.is_empty() {
            struct_ser.serialize_field("recvFees", &self.recv_fees)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTotalRecvFeesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "recv_fees",
            "recvFees",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RecvFees,
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
                            "recvFees" | "recv_fees" => Ok(GeneratedField::RecvFees),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTotalRecvFeesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryTotalRecvFeesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTotalRecvFeesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut recv_fees__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RecvFees => {
                            if recv_fees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recvFees"));
                            }
                            recv_fees__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryTotalRecvFeesResponse {
                    recv_fees: recv_fees__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryTotalRecvFeesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTotalTimeoutFeesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryTotalTimeoutFeesRequest", len)?;
        if let Some(v) = self.packet_id.as_ref() {
            struct_ser.serialize_field("packetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTotalTimeoutFeesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "packet_id",
            "packetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PacketId,
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
                            "packetId" | "packet_id" => Ok(GeneratedField::PacketId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTotalTimeoutFeesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryTotalTimeoutFeesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTotalTimeoutFeesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PacketId => {
                            if packet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetId"));
                            }
                            packet_id__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryTotalTimeoutFeesRequest {
                    packet_id: packet_id__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryTotalTimeoutFeesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTotalTimeoutFeesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.timeout_fees.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.QueryTotalTimeoutFeesResponse", len)?;
        if !self.timeout_fees.is_empty() {
            struct_ser.serialize_field("timeoutFees", &self.timeout_fees)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTotalTimeoutFeesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timeout_fees",
            "timeoutFees",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimeoutFees,
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
                            "timeoutFees" | "timeout_fees" => Ok(GeneratedField::TimeoutFees),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTotalTimeoutFeesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.QueryTotalTimeoutFeesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTotalTimeoutFeesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timeout_fees__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TimeoutFees => {
                            if timeout_fees__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutFees"));
                            }
                            timeout_fees__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryTotalTimeoutFeesResponse {
                    timeout_fees: timeout_fees__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.QueryTotalTimeoutFeesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RegisteredCounterpartyPayee {
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
        if !self.relayer.is_empty() {
            len += 1;
        }
        if !self.counterparty_payee.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.RegisteredCounterpartyPayee", len)?;
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.relayer.is_empty() {
            struct_ser.serialize_field("relayer", &self.relayer)?;
        }
        if !self.counterparty_payee.is_empty() {
            struct_ser.serialize_field("counterpartyPayee", &self.counterparty_payee)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RegisteredCounterpartyPayee {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channel_id",
            "channelId",
            "relayer",
            "counterparty_payee",
            "counterpartyPayee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChannelId,
            Relayer,
            CounterpartyPayee,
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
                            "relayer" => Ok(GeneratedField::Relayer),
                            "counterpartyPayee" | "counterparty_payee" => Ok(GeneratedField::CounterpartyPayee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RegisteredCounterpartyPayee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.RegisteredCounterpartyPayee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RegisteredCounterpartyPayee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channel_id__ = None;
                let mut relayer__ = None;
                let mut counterparty_payee__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChannelId => {
                            if channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelId"));
                            }
                            channel_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Relayer => {
                            if relayer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("relayer"));
                            }
                            relayer__ = Some(map.next_value()?);
                        }
                        GeneratedField::CounterpartyPayee => {
                            if counterparty_payee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterpartyPayee"));
                            }
                            counterparty_payee__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RegisteredCounterpartyPayee {
                    channel_id: channel_id__.unwrap_or_default(),
                    relayer: relayer__.unwrap_or_default(),
                    counterparty_payee: counterparty_payee__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.RegisteredCounterpartyPayee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RegisteredPayee {
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
        if !self.relayer.is_empty() {
            len += 1;
        }
        if !self.payee.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.fee.v1.RegisteredPayee", len)?;
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if !self.relayer.is_empty() {
            struct_ser.serialize_field("relayer", &self.relayer)?;
        }
        if !self.payee.is_empty() {
            struct_ser.serialize_field("payee", &self.payee)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RegisteredPayee {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channel_id",
            "channelId",
            "relayer",
            "payee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChannelId,
            Relayer,
            Payee,
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
                            "relayer" => Ok(GeneratedField::Relayer),
                            "payee" => Ok(GeneratedField::Payee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RegisteredPayee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.fee.v1.RegisteredPayee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RegisteredPayee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channel_id__ = None;
                let mut relayer__ = None;
                let mut payee__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChannelId => {
                            if channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelId"));
                            }
                            channel_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Relayer => {
                            if relayer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("relayer"));
                            }
                            relayer__ = Some(map.next_value()?);
                        }
                        GeneratedField::Payee => {
                            if payee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payee"));
                            }
                            payee__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RegisteredPayee {
                    channel_id: channel_id__.unwrap_or_default(),
                    relayer: relayer__.unwrap_or_default(),
                    payee: payee__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.fee.v1.RegisteredPayee", FIELDS, GeneratedVisitor)
    }
}
