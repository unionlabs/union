// @generated
impl serde::Serialize for GenesisState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.client_genesis.is_some() {
            len += 1;
        }
        if self.connection_genesis.is_some() {
            len += 1;
        }
        if self.channel_genesis.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.types.v1.GenesisState", len)?;
        if let Some(v) = self.client_genesis.as_ref() {
            struct_ser.serialize_field("clientGenesis", v)?;
        }
        if let Some(v) = self.connection_genesis.as_ref() {
            struct_ser.serialize_field("connectionGenesis", v)?;
        }
        if let Some(v) = self.channel_genesis.as_ref() {
            struct_ser.serialize_field("channelGenesis", v)?;
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
            "client_genesis",
            "clientGenesis",
            "connection_genesis",
            "connectionGenesis",
            "channel_genesis",
            "channelGenesis",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientGenesis,
            ConnectionGenesis,
            ChannelGenesis,
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
                            "clientGenesis" | "client_genesis" => Ok(GeneratedField::ClientGenesis),
                            "connectionGenesis" | "connection_genesis" => Ok(GeneratedField::ConnectionGenesis),
                            "channelGenesis" | "channel_genesis" => Ok(GeneratedField::ChannelGenesis),
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
                formatter.write_str("struct ibc.core.types.v1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_genesis__ = None;
                let mut connection_genesis__ = None;
                let mut channel_genesis__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientGenesis => {
                            if client_genesis__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientGenesis"));
                            }
                            client_genesis__ = map.next_value()?;
                        }
                        GeneratedField::ConnectionGenesis => {
                            if connection_genesis__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionGenesis"));
                            }
                            connection_genesis__ = map.next_value()?;
                        }
                        GeneratedField::ChannelGenesis => {
                            if channel_genesis__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channelGenesis"));
                            }
                            channel_genesis__ = map.next_value()?;
                        }
                    }
                }
                Ok(GenesisState {
                    client_genesis: client_genesis__,
                    connection_genesis: connection_genesis__,
                    channel_genesis: channel_genesis__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.types.v1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
