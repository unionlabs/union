// @generated
impl serde::Serialize for ActiveChannel {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.connection_id.is_empty() {
            len += 1;
        }
        if !self.port_id.is_empty() {
            len += 1;
        }
        if !self.channel_id.is_empty() {
            len += 1;
        }
        if self.is_middleware_enabled {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.genesis.v1.ActiveChannel", len)?;
        if !self.connection_id.is_empty() {
            struct_ser.serialize_field("connectionId", &self.connection_id)?;
        }
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.channel_id.is_empty() {
            struct_ser.serialize_field("channelId", &self.channel_id)?;
        }
        if self.is_middleware_enabled {
            struct_ser.serialize_field("isMiddlewareEnabled", &self.is_middleware_enabled)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActiveChannel {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_id",
            "connectionId",
            "port_id",
            "portId",
            "channel_id",
            "channelId",
            "is_middleware_enabled",
            "isMiddlewareEnabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionId,
            PortId,
            ChannelId,
            IsMiddlewareEnabled,
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
                            "connectionId" | "connection_id" => Ok(GeneratedField::ConnectionId),
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "channelId" | "channel_id" => Ok(GeneratedField::ChannelId),
                            "isMiddlewareEnabled" | "is_middleware_enabled" => Ok(GeneratedField::IsMiddlewareEnabled),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActiveChannel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.interchain_accounts.genesis.v1.ActiveChannel")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ActiveChannel, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_id__ = None;
                let mut port_id__ = None;
                let mut channel_id__ = None;
                let mut is_middleware_enabled__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConnectionId => {
                            if connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionId"));
                            }
                            connection_id__ = Some(map.next_value()?);
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
                        GeneratedField::IsMiddlewareEnabled => {
                            if is_middleware_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isMiddlewareEnabled"));
                            }
                            is_middleware_enabled__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ActiveChannel {
                    connection_id: connection_id__.unwrap_or_default(),
                    port_id: port_id__.unwrap_or_default(),
                    channel_id: channel_id__.unwrap_or_default(),
                    is_middleware_enabled: is_middleware_enabled__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.genesis.v1.ActiveChannel", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ControllerGenesisState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.active_channels.is_empty() {
            len += 1;
        }
        if !self.interchain_accounts.is_empty() {
            len += 1;
        }
        if !self.ports.is_empty() {
            len += 1;
        }
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.genesis.v1.ControllerGenesisState", len)?;
        if !self.active_channels.is_empty() {
            struct_ser.serialize_field("activeChannels", &self.active_channels)?;
        }
        if !self.interchain_accounts.is_empty() {
            struct_ser.serialize_field("interchainAccounts", &self.interchain_accounts)?;
        }
        if !self.ports.is_empty() {
            struct_ser.serialize_field("ports", &self.ports)?;
        }
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ControllerGenesisState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "active_channels",
            "activeChannels",
            "interchain_accounts",
            "interchainAccounts",
            "ports",
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ActiveChannels,
            InterchainAccounts,
            Ports,
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
                            "activeChannels" | "active_channels" => Ok(GeneratedField::ActiveChannels),
                            "interchainAccounts" | "interchain_accounts" => Ok(GeneratedField::InterchainAccounts),
                            "ports" => Ok(GeneratedField::Ports),
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
            type Value = ControllerGenesisState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.interchain_accounts.genesis.v1.ControllerGenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ControllerGenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut active_channels__ = None;
                let mut interchain_accounts__ = None;
                let mut ports__ = None;
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ActiveChannels => {
                            if active_channels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("activeChannels"));
                            }
                            active_channels__ = Some(map.next_value()?);
                        }
                        GeneratedField::InterchainAccounts => {
                            if interchain_accounts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interchainAccounts"));
                            }
                            interchain_accounts__ = Some(map.next_value()?);
                        }
                        GeneratedField::Ports => {
                            if ports__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ports"));
                            }
                            ports__ = Some(map.next_value()?);
                        }
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                    }
                }
                Ok(ControllerGenesisState {
                    active_channels: active_channels__.unwrap_or_default(),
                    interchain_accounts: interchain_accounts__.unwrap_or_default(),
                    ports: ports__.unwrap_or_default(),
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.genesis.v1.ControllerGenesisState", FIELDS, GeneratedVisitor)
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
        if self.controller_genesis_state.is_some() {
            len += 1;
        }
        if self.host_genesis_state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.genesis.v1.GenesisState", len)?;
        if let Some(v) = self.controller_genesis_state.as_ref() {
            struct_ser.serialize_field("controllerGenesisState", v)?;
        }
        if let Some(v) = self.host_genesis_state.as_ref() {
            struct_ser.serialize_field("hostGenesisState", v)?;
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
            "controller_genesis_state",
            "controllerGenesisState",
            "host_genesis_state",
            "hostGenesisState",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ControllerGenesisState,
            HostGenesisState,
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
                            "controllerGenesisState" | "controller_genesis_state" => Ok(GeneratedField::ControllerGenesisState),
                            "hostGenesisState" | "host_genesis_state" => Ok(GeneratedField::HostGenesisState),
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
                formatter.write_str("struct ibc.applications.interchain_accounts.genesis.v1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut controller_genesis_state__ = None;
                let mut host_genesis_state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ControllerGenesisState => {
                            if controller_genesis_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("controllerGenesisState"));
                            }
                            controller_genesis_state__ = map.next_value()?;
                        }
                        GeneratedField::HostGenesisState => {
                            if host_genesis_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hostGenesisState"));
                            }
                            host_genesis_state__ = map.next_value()?;
                        }
                    }
                }
                Ok(GenesisState {
                    controller_genesis_state: controller_genesis_state__,
                    host_genesis_state: host_genesis_state__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.genesis.v1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HostGenesisState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.active_channels.is_empty() {
            len += 1;
        }
        if !self.interchain_accounts.is_empty() {
            len += 1;
        }
        if !self.port.is_empty() {
            len += 1;
        }
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.genesis.v1.HostGenesisState", len)?;
        if !self.active_channels.is_empty() {
            struct_ser.serialize_field("activeChannels", &self.active_channels)?;
        }
        if !self.interchain_accounts.is_empty() {
            struct_ser.serialize_field("interchainAccounts", &self.interchain_accounts)?;
        }
        if !self.port.is_empty() {
            struct_ser.serialize_field("port", &self.port)?;
        }
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HostGenesisState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "active_channels",
            "activeChannels",
            "interchain_accounts",
            "interchainAccounts",
            "port",
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ActiveChannels,
            InterchainAccounts,
            Port,
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
                            "activeChannels" | "active_channels" => Ok(GeneratedField::ActiveChannels),
                            "interchainAccounts" | "interchain_accounts" => Ok(GeneratedField::InterchainAccounts),
                            "port" => Ok(GeneratedField::Port),
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
            type Value = HostGenesisState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.interchain_accounts.genesis.v1.HostGenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HostGenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut active_channels__ = None;
                let mut interchain_accounts__ = None;
                let mut port__ = None;
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ActiveChannels => {
                            if active_channels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("activeChannels"));
                            }
                            active_channels__ = Some(map.next_value()?);
                        }
                        GeneratedField::InterchainAccounts => {
                            if interchain_accounts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interchainAccounts"));
                            }
                            interchain_accounts__ = Some(map.next_value()?);
                        }
                        GeneratedField::Port => {
                            if port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("port"));
                            }
                            port__ = Some(map.next_value()?);
                        }
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                    }
                }
                Ok(HostGenesisState {
                    active_channels: active_channels__.unwrap_or_default(),
                    interchain_accounts: interchain_accounts__.unwrap_or_default(),
                    port: port__.unwrap_or_default(),
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.genesis.v1.HostGenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RegisteredInterchainAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.connection_id.is_empty() {
            len += 1;
        }
        if !self.port_id.is_empty() {
            len += 1;
        }
        if !self.account_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.applications.interchain_accounts.genesis.v1.RegisteredInterchainAccount", len)?;
        if !self.connection_id.is_empty() {
            struct_ser.serialize_field("connectionId", &self.connection_id)?;
        }
        if !self.port_id.is_empty() {
            struct_ser.serialize_field("portId", &self.port_id)?;
        }
        if !self.account_address.is_empty() {
            struct_ser.serialize_field("accountAddress", &self.account_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RegisteredInterchainAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_id",
            "connectionId",
            "port_id",
            "portId",
            "account_address",
            "accountAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionId,
            PortId,
            AccountAddress,
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
                            "connectionId" | "connection_id" => Ok(GeneratedField::ConnectionId),
                            "portId" | "port_id" => Ok(GeneratedField::PortId),
                            "accountAddress" | "account_address" => Ok(GeneratedField::AccountAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RegisteredInterchainAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.applications.interchain_accounts.genesis.v1.RegisteredInterchainAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RegisteredInterchainAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_id__ = None;
                let mut port_id__ = None;
                let mut account_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConnectionId => {
                            if connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionId"));
                            }
                            connection_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::PortId => {
                            if port_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("portId"));
                            }
                            port_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::AccountAddress => {
                            if account_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountAddress"));
                            }
                            account_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RegisteredInterchainAccount {
                    connection_id: connection_id__.unwrap_or_default(),
                    port_id: port_id__.unwrap_or_default(),
                    account_address: account_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.applications.interchain_accounts.genesis.v1.RegisteredInterchainAccount", FIELDS, GeneratedVisitor)
    }
}
