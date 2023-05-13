// @generated
impl serde::Serialize for ClientPaths {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.paths.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.ClientPaths", len)?;
        if !self.paths.is_empty() {
            struct_ser.serialize_field("paths", &self.paths)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientPaths {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "paths",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Paths,
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
                            "paths" => Ok(GeneratedField::Paths),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientPaths;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.ClientPaths")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ClientPaths, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut paths__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Paths => {
                            if paths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("paths"));
                            }
                            paths__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ClientPaths {
                    paths: paths__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.ClientPaths", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConnectionEnd {
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
        if !self.versions.is_empty() {
            len += 1;
        }
        if self.state != 0 {
            len += 1;
        }
        if self.counterparty.is_some() {
            len += 1;
        }
        if self.delay_period != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.ConnectionEnd", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if !self.versions.is_empty() {
            struct_ser.serialize_field("versions", &self.versions)?;
        }
        if self.state != 0 {
            let v = State::from_i32(self.state)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        if let Some(v) = self.counterparty.as_ref() {
            struct_ser.serialize_field("counterparty", v)?;
        }
        if self.delay_period != 0 {
            struct_ser.serialize_field("delayPeriod", ToString::to_string(&self.delay_period).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConnectionEnd {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
            "versions",
            "state",
            "counterparty",
            "delay_period",
            "delayPeriod",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            Versions,
            State,
            Counterparty,
            DelayPeriod,
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
                            "versions" => Ok(GeneratedField::Versions),
                            "state" => Ok(GeneratedField::State),
                            "counterparty" => Ok(GeneratedField::Counterparty),
                            "delayPeriod" | "delay_period" => Ok(GeneratedField::DelayPeriod),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConnectionEnd;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.ConnectionEnd")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConnectionEnd, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut versions__ = None;
                let mut state__ = None;
                let mut counterparty__ = None;
                let mut delay_period__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Versions => {
                            if versions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("versions"));
                            }
                            versions__ = Some(map.next_value()?);
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map.next_value::<State>()? as i32);
                        }
                        GeneratedField::Counterparty => {
                            if counterparty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterparty"));
                            }
                            counterparty__ = map.next_value()?;
                        }
                        GeneratedField::DelayPeriod => {
                            if delay_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delayPeriod"));
                            }
                            delay_period__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ConnectionEnd {
                    client_id: client_id__.unwrap_or_default(),
                    versions: versions__.unwrap_or_default(),
                    state: state__.unwrap_or_default(),
                    counterparty: counterparty__,
                    delay_period: delay_period__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.ConnectionEnd", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConnectionPaths {
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
        if !self.paths.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.ConnectionPaths", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if !self.paths.is_empty() {
            struct_ser.serialize_field("paths", &self.paths)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConnectionPaths {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
            "paths",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            Paths,
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
                            "paths" => Ok(GeneratedField::Paths),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConnectionPaths;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.ConnectionPaths")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConnectionPaths, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut paths__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Paths => {
                            if paths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("paths"));
                            }
                            paths__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ConnectionPaths {
                    client_id: client_id__.unwrap_or_default(),
                    paths: paths__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.ConnectionPaths", FIELDS, GeneratedVisitor)
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
        if !self.client_id.is_empty() {
            len += 1;
        }
        if !self.connection_id.is_empty() {
            len += 1;
        }
        if self.prefix.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.Counterparty", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if !self.connection_id.is_empty() {
            struct_ser.serialize_field("connectionId", &self.connection_id)?;
        }
        if let Some(v) = self.prefix.as_ref() {
            struct_ser.serialize_field("prefix", v)?;
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
            "client_id",
            "clientId",
            "connection_id",
            "connectionId",
            "prefix",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            ConnectionId,
            Prefix,
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
                            "connectionId" | "connection_id" => Ok(GeneratedField::ConnectionId),
                            "prefix" => Ok(GeneratedField::Prefix),
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
                formatter.write_str("struct ibc.core.connection.v1.Counterparty")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Counterparty, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut connection_id__ = None;
                let mut prefix__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ConnectionId => {
                            if connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionId"));
                            }
                            connection_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Prefix => {
                            if prefix__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prefix"));
                            }
                            prefix__ = map.next_value()?;
                        }
                    }
                }
                Ok(Counterparty {
                    client_id: client_id__.unwrap_or_default(),
                    connection_id: connection_id__.unwrap_or_default(),
                    prefix: prefix__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.Counterparty", FIELDS, GeneratedVisitor)
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
        if !self.connections.is_empty() {
            len += 1;
        }
        if !self.client_connection_paths.is_empty() {
            len += 1;
        }
        if self.next_connection_sequence != 0 {
            len += 1;
        }
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.GenesisState", len)?;
        if !self.connections.is_empty() {
            struct_ser.serialize_field("connections", &self.connections)?;
        }
        if !self.client_connection_paths.is_empty() {
            struct_ser.serialize_field("clientConnectionPaths", &self.client_connection_paths)?;
        }
        if self.next_connection_sequence != 0 {
            struct_ser.serialize_field("nextConnectionSequence", ToString::to_string(&self.next_connection_sequence).as_str())?;
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
            "connections",
            "client_connection_paths",
            "clientConnectionPaths",
            "next_connection_sequence",
            "nextConnectionSequence",
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Connections,
            ClientConnectionPaths,
            NextConnectionSequence,
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
                            "connections" => Ok(GeneratedField::Connections),
                            "clientConnectionPaths" | "client_connection_paths" => Ok(GeneratedField::ClientConnectionPaths),
                            "nextConnectionSequence" | "next_connection_sequence" => Ok(GeneratedField::NextConnectionSequence),
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
                formatter.write_str("struct ibc.core.connection.v1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connections__ = None;
                let mut client_connection_paths__ = None;
                let mut next_connection_sequence__ = None;
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Connections => {
                            if connections__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connections"));
                            }
                            connections__ = Some(map.next_value()?);
                        }
                        GeneratedField::ClientConnectionPaths => {
                            if client_connection_paths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientConnectionPaths"));
                            }
                            client_connection_paths__ = Some(map.next_value()?);
                        }
                        GeneratedField::NextConnectionSequence => {
                            if next_connection_sequence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextConnectionSequence"));
                            }
                            next_connection_sequence__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
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
                    connections: connections__.unwrap_or_default(),
                    client_connection_paths: client_connection_paths__.unwrap_or_default(),
                    next_connection_sequence: next_connection_sequence__.unwrap_or_default(),
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IdentifiedConnection {
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
        if !self.client_id.is_empty() {
            len += 1;
        }
        if !self.versions.is_empty() {
            len += 1;
        }
        if self.state != 0 {
            len += 1;
        }
        if self.counterparty.is_some() {
            len += 1;
        }
        if self.delay_period != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.IdentifiedConnection", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if !self.versions.is_empty() {
            struct_ser.serialize_field("versions", &self.versions)?;
        }
        if self.state != 0 {
            let v = State::from_i32(self.state)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        if let Some(v) = self.counterparty.as_ref() {
            struct_ser.serialize_field("counterparty", v)?;
        }
        if self.delay_period != 0 {
            struct_ser.serialize_field("delayPeriod", ToString::to_string(&self.delay_period).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IdentifiedConnection {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "client_id",
            "clientId",
            "versions",
            "state",
            "counterparty",
            "delay_period",
            "delayPeriod",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            ClientId,
            Versions,
            State,
            Counterparty,
            DelayPeriod,
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
                            "clientId" | "client_id" => Ok(GeneratedField::ClientId),
                            "versions" => Ok(GeneratedField::Versions),
                            "state" => Ok(GeneratedField::State),
                            "counterparty" => Ok(GeneratedField::Counterparty),
                            "delayPeriod" | "delay_period" => Ok(GeneratedField::DelayPeriod),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IdentifiedConnection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.IdentifiedConnection")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<IdentifiedConnection, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut client_id__ = None;
                let mut versions__ = None;
                let mut state__ = None;
                let mut counterparty__ = None;
                let mut delay_period__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Versions => {
                            if versions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("versions"));
                            }
                            versions__ = Some(map.next_value()?);
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map.next_value::<State>()? as i32);
                        }
                        GeneratedField::Counterparty => {
                            if counterparty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterparty"));
                            }
                            counterparty__ = map.next_value()?;
                        }
                        GeneratedField::DelayPeriod => {
                            if delay_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delayPeriod"));
                            }
                            delay_period__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(IdentifiedConnection {
                    id: id__.unwrap_or_default(),
                    client_id: client_id__.unwrap_or_default(),
                    versions: versions__.unwrap_or_default(),
                    state: state__.unwrap_or_default(),
                    counterparty: counterparty__,
                    delay_period: delay_period__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.IdentifiedConnection", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgConnectionOpenAck {
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
        if !self.counterparty_connection_id.is_empty() {
            len += 1;
        }
        if self.version.is_some() {
            len += 1;
        }
        if self.client_state.is_some() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.proof_try.is_empty() {
            len += 1;
        }
        if !self.proof_client.is_empty() {
            len += 1;
        }
        if !self.proof_consensus.is_empty() {
            len += 1;
        }
        if self.consensus_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        if !self.host_consensus_state_proof.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.MsgConnectionOpenAck", len)?;
        if !self.connection_id.is_empty() {
            struct_ser.serialize_field("connectionId", &self.connection_id)?;
        }
        if !self.counterparty_connection_id.is_empty() {
            struct_ser.serialize_field("counterpartyConnectionId", &self.counterparty_connection_id)?;
        }
        if let Some(v) = self.version.as_ref() {
            struct_ser.serialize_field("version", v)?;
        }
        if let Some(v) = self.client_state.as_ref() {
            struct_ser.serialize_field("clientState", v)?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if !self.proof_try.is_empty() {
            struct_ser.serialize_field("proofTry", pbjson::private::base64::encode(&self.proof_try).as_str())?;
        }
        if !self.proof_client.is_empty() {
            struct_ser.serialize_field("proofClient", pbjson::private::base64::encode(&self.proof_client).as_str())?;
        }
        if !self.proof_consensus.is_empty() {
            struct_ser.serialize_field("proofConsensus", pbjson::private::base64::encode(&self.proof_consensus).as_str())?;
        }
        if let Some(v) = self.consensus_height.as_ref() {
            struct_ser.serialize_field("consensusHeight", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        if !self.host_consensus_state_proof.is_empty() {
            struct_ser.serialize_field("hostConsensusStateProof", pbjson::private::base64::encode(&self.host_consensus_state_proof).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgConnectionOpenAck {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_id",
            "connectionId",
            "counterparty_connection_id",
            "counterpartyConnectionId",
            "version",
            "client_state",
            "clientState",
            "proof_height",
            "proofHeight",
            "proof_try",
            "proofTry",
            "proof_client",
            "proofClient",
            "proof_consensus",
            "proofConsensus",
            "consensus_height",
            "consensusHeight",
            "signer",
            "host_consensus_state_proof",
            "hostConsensusStateProof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionId,
            CounterpartyConnectionId,
            Version,
            ClientState,
            ProofHeight,
            ProofTry,
            ProofClient,
            ProofConsensus,
            ConsensusHeight,
            Signer,
            HostConsensusStateProof,
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
                            "counterpartyConnectionId" | "counterparty_connection_id" => Ok(GeneratedField::CounterpartyConnectionId),
                            "version" => Ok(GeneratedField::Version),
                            "clientState" | "client_state" => Ok(GeneratedField::ClientState),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "proofTry" | "proof_try" => Ok(GeneratedField::ProofTry),
                            "proofClient" | "proof_client" => Ok(GeneratedField::ProofClient),
                            "proofConsensus" | "proof_consensus" => Ok(GeneratedField::ProofConsensus),
                            "consensusHeight" | "consensus_height" => Ok(GeneratedField::ConsensusHeight),
                            "signer" => Ok(GeneratedField::Signer),
                            "hostConsensusStateProof" | "host_consensus_state_proof" => Ok(GeneratedField::HostConsensusStateProof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgConnectionOpenAck;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.MsgConnectionOpenAck")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgConnectionOpenAck, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_id__ = None;
                let mut counterparty_connection_id__ = None;
                let mut version__ = None;
                let mut client_state__ = None;
                let mut proof_height__ = None;
                let mut proof_try__ = None;
                let mut proof_client__ = None;
                let mut proof_consensus__ = None;
                let mut consensus_height__ = None;
                let mut signer__ = None;
                let mut host_consensus_state_proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConnectionId => {
                            if connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionId"));
                            }
                            connection_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::CounterpartyConnectionId => {
                            if counterparty_connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterpartyConnectionId"));
                            }
                            counterparty_connection_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = map.next_value()?;
                        }
                        GeneratedField::ClientState => {
                            if client_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientState"));
                            }
                            client_state__ = map.next_value()?;
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::ProofTry => {
                            if proof_try__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofTry"));
                            }
                            proof_try__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofClient => {
                            if proof_client__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofClient"));
                            }
                            proof_client__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofConsensus => {
                            if proof_consensus__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofConsensus"));
                            }
                            proof_consensus__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ConsensusHeight => {
                            if consensus_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusHeight"));
                            }
                            consensus_height__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                        GeneratedField::HostConsensusStateProof => {
                            if host_consensus_state_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hostConsensusStateProof"));
                            }
                            host_consensus_state_proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgConnectionOpenAck {
                    connection_id: connection_id__.unwrap_or_default(),
                    counterparty_connection_id: counterparty_connection_id__.unwrap_or_default(),
                    version: version__,
                    client_state: client_state__,
                    proof_height: proof_height__,
                    proof_try: proof_try__.unwrap_or_default(),
                    proof_client: proof_client__.unwrap_or_default(),
                    proof_consensus: proof_consensus__.unwrap_or_default(),
                    consensus_height: consensus_height__,
                    signer: signer__.unwrap_or_default(),
                    host_consensus_state_proof: host_consensus_state_proof__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.MsgConnectionOpenAck", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgConnectionOpenAckResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.connection.v1.MsgConnectionOpenAckResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgConnectionOpenAckResponse {
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
            type Value = MsgConnectionOpenAckResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.MsgConnectionOpenAckResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgConnectionOpenAckResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgConnectionOpenAckResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.MsgConnectionOpenAckResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgConnectionOpenConfirm {
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
        if !self.proof_ack.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.MsgConnectionOpenConfirm", len)?;
        if !self.connection_id.is_empty() {
            struct_ser.serialize_field("connectionId", &self.connection_id)?;
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
impl<'de> serde::Deserialize<'de> for MsgConnectionOpenConfirm {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_id",
            "connectionId",
            "proof_ack",
            "proofAck",
            "proof_height",
            "proofHeight",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionId,
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
                            "connectionId" | "connection_id" => Ok(GeneratedField::ConnectionId),
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
            type Value = MsgConnectionOpenConfirm;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.MsgConnectionOpenConfirm")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgConnectionOpenConfirm, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_id__ = None;
                let mut proof_ack__ = None;
                let mut proof_height__ = None;
                let mut signer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConnectionId => {
                            if connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionId"));
                            }
                            connection_id__ = Some(map.next_value()?);
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
                Ok(MsgConnectionOpenConfirm {
                    connection_id: connection_id__.unwrap_or_default(),
                    proof_ack: proof_ack__.unwrap_or_default(),
                    proof_height: proof_height__,
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.MsgConnectionOpenConfirm", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgConnectionOpenConfirmResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.connection.v1.MsgConnectionOpenConfirmResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgConnectionOpenConfirmResponse {
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
            type Value = MsgConnectionOpenConfirmResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.MsgConnectionOpenConfirmResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgConnectionOpenConfirmResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgConnectionOpenConfirmResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.MsgConnectionOpenConfirmResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgConnectionOpenInit {
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
        if self.counterparty.is_some() {
            len += 1;
        }
        if self.version.is_some() {
            len += 1;
        }
        if self.delay_period != 0 {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.MsgConnectionOpenInit", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if let Some(v) = self.counterparty.as_ref() {
            struct_ser.serialize_field("counterparty", v)?;
        }
        if let Some(v) = self.version.as_ref() {
            struct_ser.serialize_field("version", v)?;
        }
        if self.delay_period != 0 {
            struct_ser.serialize_field("delayPeriod", ToString::to_string(&self.delay_period).as_str())?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgConnectionOpenInit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
            "counterparty",
            "version",
            "delay_period",
            "delayPeriod",
            "signer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            Counterparty,
            Version,
            DelayPeriod,
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
                            "clientId" | "client_id" => Ok(GeneratedField::ClientId),
                            "counterparty" => Ok(GeneratedField::Counterparty),
                            "version" => Ok(GeneratedField::Version),
                            "delayPeriod" | "delay_period" => Ok(GeneratedField::DelayPeriod),
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
            type Value = MsgConnectionOpenInit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.MsgConnectionOpenInit")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgConnectionOpenInit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut counterparty__ = None;
                let mut version__ = None;
                let mut delay_period__ = None;
                let mut signer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Counterparty => {
                            if counterparty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterparty"));
                            }
                            counterparty__ = map.next_value()?;
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = map.next_value()?;
                        }
                        GeneratedField::DelayPeriod => {
                            if delay_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delayPeriod"));
                            }
                            delay_period__ = 
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
                Ok(MsgConnectionOpenInit {
                    client_id: client_id__.unwrap_or_default(),
                    counterparty: counterparty__,
                    version: version__,
                    delay_period: delay_period__.unwrap_or_default(),
                    signer: signer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.MsgConnectionOpenInit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgConnectionOpenInitResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.connection.v1.MsgConnectionOpenInitResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgConnectionOpenInitResponse {
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
            type Value = MsgConnectionOpenInitResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.MsgConnectionOpenInitResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgConnectionOpenInitResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgConnectionOpenInitResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.MsgConnectionOpenInitResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgConnectionOpenTry {
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
        if !self.previous_connection_id.is_empty() {
            len += 1;
        }
        if self.client_state.is_some() {
            len += 1;
        }
        if self.counterparty.is_some() {
            len += 1;
        }
        if self.delay_period != 0 {
            len += 1;
        }
        if !self.counterparty_versions.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        if !self.proof_init.is_empty() {
            len += 1;
        }
        if !self.proof_client.is_empty() {
            len += 1;
        }
        if !self.proof_consensus.is_empty() {
            len += 1;
        }
        if self.consensus_height.is_some() {
            len += 1;
        }
        if !self.signer.is_empty() {
            len += 1;
        }
        if !self.host_consensus_state_proof.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.MsgConnectionOpenTry", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if !self.previous_connection_id.is_empty() {
            struct_ser.serialize_field("previousConnectionId", &self.previous_connection_id)?;
        }
        if let Some(v) = self.client_state.as_ref() {
            struct_ser.serialize_field("clientState", v)?;
        }
        if let Some(v) = self.counterparty.as_ref() {
            struct_ser.serialize_field("counterparty", v)?;
        }
        if self.delay_period != 0 {
            struct_ser.serialize_field("delayPeriod", ToString::to_string(&self.delay_period).as_str())?;
        }
        if !self.counterparty_versions.is_empty() {
            struct_ser.serialize_field("counterpartyVersions", &self.counterparty_versions)?;
        }
        if let Some(v) = self.proof_height.as_ref() {
            struct_ser.serialize_field("proofHeight", v)?;
        }
        if !self.proof_init.is_empty() {
            struct_ser.serialize_field("proofInit", pbjson::private::base64::encode(&self.proof_init).as_str())?;
        }
        if !self.proof_client.is_empty() {
            struct_ser.serialize_field("proofClient", pbjson::private::base64::encode(&self.proof_client).as_str())?;
        }
        if !self.proof_consensus.is_empty() {
            struct_ser.serialize_field("proofConsensus", pbjson::private::base64::encode(&self.proof_consensus).as_str())?;
        }
        if let Some(v) = self.consensus_height.as_ref() {
            struct_ser.serialize_field("consensusHeight", v)?;
        }
        if !self.signer.is_empty() {
            struct_ser.serialize_field("signer", &self.signer)?;
        }
        if !self.host_consensus_state_proof.is_empty() {
            struct_ser.serialize_field("hostConsensusStateProof", pbjson::private::base64::encode(&self.host_consensus_state_proof).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgConnectionOpenTry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
            "previous_connection_id",
            "previousConnectionId",
            "client_state",
            "clientState",
            "counterparty",
            "delay_period",
            "delayPeriod",
            "counterparty_versions",
            "counterpartyVersions",
            "proof_height",
            "proofHeight",
            "proof_init",
            "proofInit",
            "proof_client",
            "proofClient",
            "proof_consensus",
            "proofConsensus",
            "consensus_height",
            "consensusHeight",
            "signer",
            "host_consensus_state_proof",
            "hostConsensusStateProof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            PreviousConnectionId,
            ClientState,
            Counterparty,
            DelayPeriod,
            CounterpartyVersions,
            ProofHeight,
            ProofInit,
            ProofClient,
            ProofConsensus,
            ConsensusHeight,
            Signer,
            HostConsensusStateProof,
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
                            "previousConnectionId" | "previous_connection_id" => Ok(GeneratedField::PreviousConnectionId),
                            "clientState" | "client_state" => Ok(GeneratedField::ClientState),
                            "counterparty" => Ok(GeneratedField::Counterparty),
                            "delayPeriod" | "delay_period" => Ok(GeneratedField::DelayPeriod),
                            "counterpartyVersions" | "counterparty_versions" => Ok(GeneratedField::CounterpartyVersions),
                            "proofHeight" | "proof_height" => Ok(GeneratedField::ProofHeight),
                            "proofInit" | "proof_init" => Ok(GeneratedField::ProofInit),
                            "proofClient" | "proof_client" => Ok(GeneratedField::ProofClient),
                            "proofConsensus" | "proof_consensus" => Ok(GeneratedField::ProofConsensus),
                            "consensusHeight" | "consensus_height" => Ok(GeneratedField::ConsensusHeight),
                            "signer" => Ok(GeneratedField::Signer),
                            "hostConsensusStateProof" | "host_consensus_state_proof" => Ok(GeneratedField::HostConsensusStateProof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgConnectionOpenTry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.MsgConnectionOpenTry")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgConnectionOpenTry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut previous_connection_id__ = None;
                let mut client_state__ = None;
                let mut counterparty__ = None;
                let mut delay_period__ = None;
                let mut counterparty_versions__ = None;
                let mut proof_height__ = None;
                let mut proof_init__ = None;
                let mut proof_client__ = None;
                let mut proof_consensus__ = None;
                let mut consensus_height__ = None;
                let mut signer__ = None;
                let mut host_consensus_state_proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::PreviousConnectionId => {
                            if previous_connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("previousConnectionId"));
                            }
                            previous_connection_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ClientState => {
                            if client_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientState"));
                            }
                            client_state__ = map.next_value()?;
                        }
                        GeneratedField::Counterparty => {
                            if counterparty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterparty"));
                            }
                            counterparty__ = map.next_value()?;
                        }
                        GeneratedField::DelayPeriod => {
                            if delay_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delayPeriod"));
                            }
                            delay_period__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CounterpartyVersions => {
                            if counterparty_versions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counterpartyVersions"));
                            }
                            counterparty_versions__ = Some(map.next_value()?);
                        }
                        GeneratedField::ProofHeight => {
                            if proof_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofHeight"));
                            }
                            proof_height__ = map.next_value()?;
                        }
                        GeneratedField::ProofInit => {
                            if proof_init__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofInit"));
                            }
                            proof_init__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofClient => {
                            if proof_client__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofClient"));
                            }
                            proof_client__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofConsensus => {
                            if proof_consensus__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofConsensus"));
                            }
                            proof_consensus__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ConsensusHeight => {
                            if consensus_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusHeight"));
                            }
                            consensus_height__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = Some(map.next_value()?);
                        }
                        GeneratedField::HostConsensusStateProof => {
                            if host_consensus_state_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hostConsensusStateProof"));
                            }
                            host_consensus_state_proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgConnectionOpenTry {
                    client_id: client_id__.unwrap_or_default(),
                    previous_connection_id: previous_connection_id__.unwrap_or_default(),
                    client_state: client_state__,
                    counterparty: counterparty__,
                    delay_period: delay_period__.unwrap_or_default(),
                    counterparty_versions: counterparty_versions__.unwrap_or_default(),
                    proof_height: proof_height__,
                    proof_init: proof_init__.unwrap_or_default(),
                    proof_client: proof_client__.unwrap_or_default(),
                    proof_consensus: proof_consensus__.unwrap_or_default(),
                    consensus_height: consensus_height__,
                    signer: signer__.unwrap_or_default(),
                    host_consensus_state_proof: host_consensus_state_proof__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.MsgConnectionOpenTry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgConnectionOpenTryResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.connection.v1.MsgConnectionOpenTryResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgConnectionOpenTryResponse {
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
            type Value = MsgConnectionOpenTryResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.MsgConnectionOpenTryResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgConnectionOpenTryResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgConnectionOpenTryResponse {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.MsgConnectionOpenTryResponse", FIELDS, GeneratedVisitor)
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
        if self.max_expected_time_per_block != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.Params", len)?;
        if self.max_expected_time_per_block != 0 {
            struct_ser.serialize_field("maxExpectedTimePerBlock", ToString::to_string(&self.max_expected_time_per_block).as_str())?;
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
            "max_expected_time_per_block",
            "maxExpectedTimePerBlock",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxExpectedTimePerBlock,
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
                            "maxExpectedTimePerBlock" | "max_expected_time_per_block" => Ok(GeneratedField::MaxExpectedTimePerBlock),
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
                formatter.write_str("struct ibc.core.connection.v1.Params")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Params, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_expected_time_per_block__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MaxExpectedTimePerBlock => {
                            if max_expected_time_per_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxExpectedTimePerBlock"));
                            }
                            max_expected_time_per_block__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Params {
                    max_expected_time_per_block: max_expected_time_per_block__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.Params", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryClientConnectionsRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryClientConnectionsRequest", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryClientConnectionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryClientConnectionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryClientConnectionsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryClientConnectionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryClientConnectionsRequest {
                    client_id: client_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryClientConnectionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryClientConnectionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.connection_paths.is_empty() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryClientConnectionsResponse", len)?;
        if !self.connection_paths.is_empty() {
            struct_ser.serialize_field("connectionPaths", &self.connection_paths)?;
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
impl<'de> serde::Deserialize<'de> for QueryClientConnectionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_paths",
            "connectionPaths",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionPaths,
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
                            "connectionPaths" | "connection_paths" => Ok(GeneratedField::ConnectionPaths),
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
            type Value = QueryClientConnectionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryClientConnectionsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryClientConnectionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_paths__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConnectionPaths => {
                            if connection_paths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionPaths"));
                            }
                            connection_paths__ = Some(map.next_value()?);
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
                Ok(QueryClientConnectionsResponse {
                    connection_paths: connection_paths__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryClientConnectionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionClientStateRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionClientStateRequest", len)?;
        if !self.connection_id.is_empty() {
            struct_ser.serialize_field("connectionId", &self.connection_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConnectionClientStateRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_id",
            "connectionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryConnectionClientStateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionClientStateRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionClientStateRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConnectionId => {
                            if connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionId"));
                            }
                            connection_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryConnectionClientStateRequest {
                    connection_id: connection_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionClientStateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionClientStateResponse {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionClientStateResponse", len)?;
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
impl<'de> serde::Deserialize<'de> for QueryConnectionClientStateResponse {
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
            type Value = QueryConnectionClientStateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionClientStateResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionClientStateResponse, V::Error>
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
                Ok(QueryConnectionClientStateResponse {
                    identified_client_state: identified_client_state__,
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionClientStateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionConsensusStateRequest {
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
        if self.revision_number != 0 {
            len += 1;
        }
        if self.revision_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionConsensusStateRequest", len)?;
        if !self.connection_id.is_empty() {
            struct_ser.serialize_field("connectionId", &self.connection_id)?;
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
impl<'de> serde::Deserialize<'de> for QueryConnectionConsensusStateRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_id",
            "connectionId",
            "revision_number",
            "revisionNumber",
            "revision_height",
            "revisionHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionId,
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
                            "connectionId" | "connection_id" => Ok(GeneratedField::ConnectionId),
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
            type Value = QueryConnectionConsensusStateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionConsensusStateRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionConsensusStateRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_id__ = None;
                let mut revision_number__ = None;
                let mut revision_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConnectionId => {
                            if connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionId"));
                            }
                            connection_id__ = Some(map.next_value()?);
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
                Ok(QueryConnectionConsensusStateRequest {
                    connection_id: connection_id__.unwrap_or_default(),
                    revision_number: revision_number__.unwrap_or_default(),
                    revision_height: revision_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionConsensusStateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionConsensusStateResponse {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionConsensusStateResponse", len)?;
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
impl<'de> serde::Deserialize<'de> for QueryConnectionConsensusStateResponse {
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
            type Value = QueryConnectionConsensusStateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionConsensusStateResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionConsensusStateResponse, V::Error>
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
                Ok(QueryConnectionConsensusStateResponse {
                    consensus_state: consensus_state__,
                    client_id: client_id__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionConsensusStateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionParamsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionParamsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConnectionParamsRequest {
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
            type Value = QueryConnectionParamsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionParamsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionParamsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(QueryConnectionParamsRequest {
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionParamsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionParamsResponse {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionParamsResponse", len)?;
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConnectionParamsResponse {
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
            type Value = QueryConnectionParamsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionParamsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionParamsResponse, V::Error>
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
                Ok(QueryConnectionParamsResponse {
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionParamsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionRequest", len)?;
        if !self.connection_id.is_empty() {
            struct_ser.serialize_field("connectionId", &self.connection_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConnectionRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection_id",
            "connectionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConnectionId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryConnectionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConnectionId => {
                            if connection_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectionId"));
                            }
                            connection_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryConnectionRequest {
                    connection_id: connection_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.connection.is_some() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.proof_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionResponse", len)?;
        if let Some(v) = self.connection.as_ref() {
            struct_ser.serialize_field("connection", v)?;
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
impl<'de> serde::Deserialize<'de> for QueryConnectionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connection",
            "proof",
            "proof_height",
            "proofHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Connection,
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
                            "connection" => Ok(GeneratedField::Connection),
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
            type Value = QueryConnectionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connection__ = None;
                let mut proof__ = None;
                let mut proof_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Connection => {
                            if connection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connection"));
                            }
                            connection__ = map.next_value()?;
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
                Ok(QueryConnectionResponse {
                    connection: connection__,
                    proof: proof__.unwrap_or_default(),
                    proof_height: proof_height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionsRequest {
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
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionsRequest", len)?;
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConnectionsRequest {
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
            type Value = QueryConnectionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionsRequest, V::Error>
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
                Ok(QueryConnectionsRequest {
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConnectionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.connections.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        if self.height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.QueryConnectionsResponse", len)?;
        if !self.connections.is_empty() {
            struct_ser.serialize_field("connections", &self.connections)?;
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
impl<'de> serde::Deserialize<'de> for QueryConnectionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connections",
            "pagination",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Connections,
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
                            "connections" => Ok(GeneratedField::Connections),
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
            type Value = QueryConnectionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.QueryConnectionsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConnectionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connections__ = None;
                let mut pagination__ = None;
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Connections => {
                            if connections__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connections"));
                            }
                            connections__ = Some(map.next_value()?);
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
                Ok(QueryConnectionsResponse {
                    connections: connections__.unwrap_or_default(),
                    pagination: pagination__,
                    height: height__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.QueryConnectionsResponse", FIELDS, GeneratedVisitor)
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
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Version {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.identifier.is_empty() {
            len += 1;
        }
        if !self.features.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.core.connection.v1.Version", len)?;
        if !self.identifier.is_empty() {
            struct_ser.serialize_field("identifier", &self.identifier)?;
        }
        if !self.features.is_empty() {
            struct_ser.serialize_field("features", &self.features)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Version {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identifier",
            "features",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Identifier,
            Features,
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
                            "identifier" => Ok(GeneratedField::Identifier),
                            "features" => Ok(GeneratedField::Features),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.core.connection.v1.Version")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Version, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identifier__ = None;
                let mut features__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Identifier => {
                            if identifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identifier"));
                            }
                            identifier__ = Some(map.next_value()?);
                        }
                        GeneratedField::Features => {
                            if features__.is_some() {
                                return Err(serde::de::Error::duplicate_field("features"));
                            }
                            features__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Version {
                    identifier: identifier__.unwrap_or_default(),
                    features: features__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.core.connection.v1.Version", FIELDS, GeneratedVisitor)
    }
}
