// @generated
impl serde::Serialize for AuthSigMessage {
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
        if !self.sig.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.AuthSigMessage", len)?;
        if let Some(v) = self.pub_key.as_ref() {
            struct_ser.serialize_field("pubKey", v)?;
        }
        if !self.sig.is_empty() {
            struct_ser.serialize_field("sig", pbjson::private::base64::encode(&self.sig).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthSigMessage {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pub_key",
            "pubKey",
            "sig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PubKey,
            Sig,
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
                            "sig" => Ok(GeneratedField::Sig),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthSigMessage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.AuthSigMessage")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AuthSigMessage, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pub_key__ = None;
                let mut sig__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PubKey => {
                            if pub_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubKey"));
                            }
                            pub_key__ = map.next_value()?;
                        }
                        GeneratedField::Sig => {
                            if sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sig"));
                            }
                            sig__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(AuthSigMessage {
                    pub_key: pub_key__,
                    sig: sig__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.AuthSigMessage", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DefaultNodeInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.protocol_version.is_some() {
            len += 1;
        }
        if !self.default_node_id.is_empty() {
            len += 1;
        }
        if !self.listen_addr.is_empty() {
            len += 1;
        }
        if !self.network.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        if !self.channels.is_empty() {
            len += 1;
        }
        if !self.moniker.is_empty() {
            len += 1;
        }
        if self.other.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.DefaultNodeInfo", len)?;
        if let Some(v) = self.protocol_version.as_ref() {
            struct_ser.serialize_field("protocolVersion", v)?;
        }
        if !self.default_node_id.is_empty() {
            struct_ser.serialize_field("defaultNodeId", &self.default_node_id)?;
        }
        if !self.listen_addr.is_empty() {
            struct_ser.serialize_field("listenAddr", &self.listen_addr)?;
        }
        if !self.network.is_empty() {
            struct_ser.serialize_field("network", &self.network)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if !self.channels.is_empty() {
            struct_ser.serialize_field("channels", pbjson::private::base64::encode(&self.channels).as_str())?;
        }
        if !self.moniker.is_empty() {
            struct_ser.serialize_field("moniker", &self.moniker)?;
        }
        if let Some(v) = self.other.as_ref() {
            struct_ser.serialize_field("other", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DefaultNodeInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "protocol_version",
            "protocolVersion",
            "default_node_id",
            "defaultNodeId",
            "listen_addr",
            "listenAddr",
            "network",
            "version",
            "channels",
            "moniker",
            "other",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProtocolVersion,
            DefaultNodeId,
            ListenAddr,
            Network,
            Version,
            Channels,
            Moniker,
            Other,
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
                            "protocolVersion" | "protocol_version" => Ok(GeneratedField::ProtocolVersion),
                            "defaultNodeId" | "default_node_id" => Ok(GeneratedField::DefaultNodeId),
                            "listenAddr" | "listen_addr" => Ok(GeneratedField::ListenAddr),
                            "network" => Ok(GeneratedField::Network),
                            "version" => Ok(GeneratedField::Version),
                            "channels" => Ok(GeneratedField::Channels),
                            "moniker" => Ok(GeneratedField::Moniker),
                            "other" => Ok(GeneratedField::Other),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DefaultNodeInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.DefaultNodeInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DefaultNodeInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut protocol_version__ = None;
                let mut default_node_id__ = None;
                let mut listen_addr__ = None;
                let mut network__ = None;
                let mut version__ = None;
                let mut channels__ = None;
                let mut moniker__ = None;
                let mut other__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProtocolVersion => {
                            if protocol_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("protocolVersion"));
                            }
                            protocol_version__ = map.next_value()?;
                        }
                        GeneratedField::DefaultNodeId => {
                            if default_node_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("defaultNodeId"));
                            }
                            default_node_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ListenAddr => {
                            if listen_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("listenAddr"));
                            }
                            listen_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::Network => {
                            if network__.is_some() {
                                return Err(serde::de::Error::duplicate_field("network"));
                            }
                            network__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                        GeneratedField::Channels => {
                            if channels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channels"));
                            }
                            channels__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Moniker => {
                            if moniker__.is_some() {
                                return Err(serde::de::Error::duplicate_field("moniker"));
                            }
                            moniker__ = Some(map.next_value()?);
                        }
                        GeneratedField::Other => {
                            if other__.is_some() {
                                return Err(serde::de::Error::duplicate_field("other"));
                            }
                            other__ = map.next_value()?;
                        }
                    }
                }
                Ok(DefaultNodeInfo {
                    protocol_version: protocol_version__,
                    default_node_id: default_node_id__.unwrap_or_default(),
                    listen_addr: listen_addr__.unwrap_or_default(),
                    network: network__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    channels: channels__.unwrap_or_default(),
                    moniker: moniker__.unwrap_or_default(),
                    other: other__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.DefaultNodeInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DefaultNodeInfoOther {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tx_index.is_empty() {
            len += 1;
        }
        if !self.rpc_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.DefaultNodeInfoOther", len)?;
        if !self.tx_index.is_empty() {
            struct_ser.serialize_field("txIndex", &self.tx_index)?;
        }
        if !self.rpc_address.is_empty() {
            struct_ser.serialize_field("rpcAddress", &self.rpc_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DefaultNodeInfoOther {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tx_index",
            "txIndex",
            "rpc_address",
            "rpcAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TxIndex,
            RpcAddress,
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
                            "txIndex" | "tx_index" => Ok(GeneratedField::TxIndex),
                            "rpcAddress" | "rpc_address" => Ok(GeneratedField::RpcAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DefaultNodeInfoOther;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.DefaultNodeInfoOther")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DefaultNodeInfoOther, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tx_index__ = None;
                let mut rpc_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TxIndex => {
                            if tx_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txIndex"));
                            }
                            tx_index__ = Some(map.next_value()?);
                        }
                        GeneratedField::RpcAddress => {
                            if rpc_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rpcAddress"));
                            }
                            rpc_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DefaultNodeInfoOther {
                    tx_index: tx_index__.unwrap_or_default(),
                    rpc_address: rpc_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.DefaultNodeInfoOther", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.Message", len)?;
        if let Some(v) = self.sum.as_ref() {
            match v {
                message::Sum::PexRequest(v) => {
                    struct_ser.serialize_field("pexRequest", v)?;
                }
                message::Sum::PexAddrs(v) => {
                    struct_ser.serialize_field("pexAddrs", v)?;
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
            "pex_request",
            "pexRequest",
            "pex_addrs",
            "pexAddrs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PexRequest,
            PexAddrs,
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
                            "pexRequest" | "pex_request" => Ok(GeneratedField::PexRequest),
                            "pexAddrs" | "pex_addrs" => Ok(GeneratedField::PexAddrs),
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
                formatter.write_str("struct tendermint.p2p.Message")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Message, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sum__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PexRequest => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pexRequest"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::PexRequest)
;
                        }
                        GeneratedField::PexAddrs => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pexAddrs"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::PexAddrs)
;
                        }
                    }
                }
                Ok(Message {
                    sum: sum__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.Message", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NetAddress {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        if !self.ip.is_empty() {
            len += 1;
        }
        if self.port != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.NetAddress", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.ip.is_empty() {
            struct_ser.serialize_field("ip", &self.ip)?;
        }
        if self.port != 0 {
            struct_ser.serialize_field("port", &self.port)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NetAddress {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "ip",
            "port",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Ip,
            Port,
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
                            "id" => Ok(GeneratedField::Id),
                            "ip" => Ok(GeneratedField::Ip),
                            "port" => Ok(GeneratedField::Port),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NetAddress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.NetAddress")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NetAddress, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut ip__ = None;
                let mut port__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Ip => {
                            if ip__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ip"));
                            }
                            ip__ = Some(map.next_value()?);
                        }
                        GeneratedField::Port => {
                            if port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("port"));
                            }
                            port__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(NetAddress {
                    id: id__.unwrap_or_default(),
                    ip: ip__.unwrap_or_default(),
                    port: port__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.NetAddress", FIELDS, GeneratedVisitor)
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
        if self.sum.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.Packet", len)?;
        if let Some(v) = self.sum.as_ref() {
            match v {
                packet::Sum::PacketPing(v) => {
                    struct_ser.serialize_field("packetPing", v)?;
                }
                packet::Sum::PacketPong(v) => {
                    struct_ser.serialize_field("packetPong", v)?;
                }
                packet::Sum::PacketMsg(v) => {
                    struct_ser.serialize_field("packetMsg", v)?;
                }
            }
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
            "packet_ping",
            "packetPing",
            "packet_pong",
            "packetPong",
            "packet_msg",
            "packetMsg",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PacketPing,
            PacketPong,
            PacketMsg,
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
                            "packetPing" | "packet_ping" => Ok(GeneratedField::PacketPing),
                            "packetPong" | "packet_pong" => Ok(GeneratedField::PacketPong),
                            "packetMsg" | "packet_msg" => Ok(GeneratedField::PacketMsg),
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
                formatter.write_str("struct tendermint.p2p.Packet")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Packet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sum__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PacketPing => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetPing"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(packet::Sum::PacketPing)
;
                        }
                        GeneratedField::PacketPong => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetPong"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(packet::Sum::PacketPong)
;
                        }
                        GeneratedField::PacketMsg => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("packetMsg"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(packet::Sum::PacketMsg)
;
                        }
                    }
                }
                Ok(Packet {
                    sum: sum__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.Packet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketMsg {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.channel_id != 0 {
            len += 1;
        }
        if self.eof {
            len += 1;
        }
        if !self.data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.PacketMsg", len)?;
        if self.channel_id != 0 {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.eof {
            struct_ser.serialize_field("eof", &self.eof)?;
        }
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketMsg {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channel_id",
            "channelId",
            "eof",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChannelId,
            Eof,
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
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
                            "eof" => Ok(GeneratedField::Eof),
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
            type Value = PacketMsg;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.PacketMsg")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketMsg, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channel_id__ = None;
                let mut eof__ = None;
                let mut data__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChannelId => {
                            if channel_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelId"));
                            }
                            channel_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Eof => {
                            if eof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("eof"));
                            }
                            eof__ = Some(map.next_value()?);
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
                Ok(PacketMsg {
                    channel_id: channel_id__.unwrap_or_default(),
                    eof: eof__.unwrap_or_default(),
                    data: data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.PacketMsg", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketPing {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("tendermint.p2p.PacketPing", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketPing {
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
            type Value = PacketPing;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.PacketPing")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketPing, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PacketPing {
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.PacketPing", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PacketPong {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("tendermint.p2p.PacketPong", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PacketPong {
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
            type Value = PacketPong;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.PacketPong")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PacketPong, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PacketPong {
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.PacketPong", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PexAddrs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.addrs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.PexAddrs", len)?;
        if !self.addrs.is_empty() {
            struct_ser.serialize_field("addrs", &self.addrs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PexAddrs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "addrs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Addrs,
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
                            "addrs" => Ok(GeneratedField::Addrs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PexAddrs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.PexAddrs")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PexAddrs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut addrs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Addrs => {
                            if addrs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addrs"));
                            }
                            addrs__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PexAddrs {
                    addrs: addrs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.PexAddrs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PexRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("tendermint.p2p.PexRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PexRequest {
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
            type Value = PexRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.PexRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PexRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PexRequest {
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.PexRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProtocolVersion {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.p2p != 0 {
            len += 1;
        }
        if self.block != 0 {
            len += 1;
        }
        if self.app != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.p2p.ProtocolVersion", len)?;
        if self.p2p != 0 {
            struct_ser.serialize_field("p2p", ToString::to_string(&self.p2p).as_str())?;
        }
        if self.block != 0 {
            struct_ser.serialize_field("block", ToString::to_string(&self.block).as_str())?;
        }
        if self.app != 0 {
            struct_ser.serialize_field("app", ToString::to_string(&self.app).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProtocolVersion {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "p2p",
            "block",
            "app",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            P2p,
            Block,
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
                            "p2p" => Ok(GeneratedField::P2p),
                            "block" => Ok(GeneratedField::Block),
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
            type Value = ProtocolVersion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.p2p.ProtocolVersion")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProtocolVersion, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut p2p__ = None;
                let mut block__ = None;
                let mut app__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::P2p => {
                            if p2p__.is_some() {
                                return Err(serde::de::Error::duplicate_field("p2p"));
                            }
                            p2p__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Block => {
                            if block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("block"));
                            }
                            block__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
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
                Ok(ProtocolVersion {
                    p2p: p2p__.unwrap_or_default(),
                    block: block__.unwrap_or_default(),
                    app: app__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.p2p.ProtocolVersion", FIELDS, GeneratedVisitor)
    }
}
