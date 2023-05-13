// @generated
impl serde::Serialize for AllowedMsgAllowance {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.allowance.is_some() {
            len += 1;
        }
        if !self.allowed_messages.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.AllowedMsgAllowance", len)?;
        if let Some(v) = self.allowance.as_ref() {
            struct_ser.serialize_field("allowance", v)?;
        }
        if !self.allowed_messages.is_empty() {
            struct_ser.serialize_field("allowedMessages", &self.allowed_messages)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AllowedMsgAllowance {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "allowance",
            "allowed_messages",
            "allowedMessages",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Allowance,
            AllowedMessages,
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
                            "allowance" => Ok(GeneratedField::Allowance),
                            "allowedMessages" | "allowed_messages" => Ok(GeneratedField::AllowedMessages),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AllowedMsgAllowance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.AllowedMsgAllowance")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AllowedMsgAllowance, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut allowance__ = None;
                let mut allowed_messages__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Allowance => {
                            if allowance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowance"));
                            }
                            allowance__ = map.next_value()?;
                        }
                        GeneratedField::AllowedMessages => {
                            if allowed_messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowedMessages"));
                            }
                            allowed_messages__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AllowedMsgAllowance {
                    allowance: allowance__,
                    allowed_messages: allowed_messages__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.AllowedMsgAllowance", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BasicAllowance {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.spend_limit.is_empty() {
            len += 1;
        }
        if self.expiration.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.BasicAllowance", len)?;
        if !self.spend_limit.is_empty() {
            struct_ser.serialize_field("spendLimit", &self.spend_limit)?;
        }
        if let Some(v) = self.expiration.as_ref() {
            struct_ser.serialize_field("expiration", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BasicAllowance {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend_limit",
            "spendLimit",
            "expiration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SpendLimit,
            Expiration,
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
                            "spendLimit" | "spend_limit" => Ok(GeneratedField::SpendLimit),
                            "expiration" => Ok(GeneratedField::Expiration),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BasicAllowance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.BasicAllowance")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BasicAllowance, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend_limit__ = None;
                let mut expiration__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SpendLimit => {
                            if spend_limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendLimit"));
                            }
                            spend_limit__ = Some(map.next_value()?);
                        }
                        GeneratedField::Expiration => {
                            if expiration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiration"));
                            }
                            expiration__ = map.next_value()?;
                        }
                    }
                }
                Ok(BasicAllowance {
                    spend_limit: spend_limit__.unwrap_or_default(),
                    expiration: expiration__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.BasicAllowance", FIELDS, GeneratedVisitor)
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
        if !self.allowances.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.GenesisState", len)?;
        if !self.allowances.is_empty() {
            struct_ser.serialize_field("allowances", &self.allowances)?;
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
            "allowances",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Allowances,
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
                            "allowances" => Ok(GeneratedField::Allowances),
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
                formatter.write_str("struct cosmos.feegrant.v1beta1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut allowances__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Allowances => {
                            if allowances__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowances"));
                            }
                            allowances__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenesisState {
                    allowances: allowances__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Grant {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.granter.is_empty() {
            len += 1;
        }
        if !self.grantee.is_empty() {
            len += 1;
        }
        if self.allowance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.Grant", len)?;
        if !self.granter.is_empty() {
            struct_ser.serialize_field("granter", &self.granter)?;
        }
        if !self.grantee.is_empty() {
            struct_ser.serialize_field("grantee", &self.grantee)?;
        }
        if let Some(v) = self.allowance.as_ref() {
            struct_ser.serialize_field("allowance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Grant {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "granter",
            "grantee",
            "allowance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Granter,
            Grantee,
            Allowance,
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
                            "granter" => Ok(GeneratedField::Granter),
                            "grantee" => Ok(GeneratedField::Grantee),
                            "allowance" => Ok(GeneratedField::Allowance),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Grant;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.Grant")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Grant, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut granter__ = None;
                let mut grantee__ = None;
                let mut allowance__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Granter => {
                            if granter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("granter"));
                            }
                            granter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Grantee => {
                            if grantee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantee"));
                            }
                            grantee__ = Some(map.next_value()?);
                        }
                        GeneratedField::Allowance => {
                            if allowance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowance"));
                            }
                            allowance__ = map.next_value()?;
                        }
                    }
                }
                Ok(Grant {
                    granter: granter__.unwrap_or_default(),
                    grantee: grantee__.unwrap_or_default(),
                    allowance: allowance__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.Grant", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgGrantAllowance {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.granter.is_empty() {
            len += 1;
        }
        if !self.grantee.is_empty() {
            len += 1;
        }
        if self.allowance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.MsgGrantAllowance", len)?;
        if !self.granter.is_empty() {
            struct_ser.serialize_field("granter", &self.granter)?;
        }
        if !self.grantee.is_empty() {
            struct_ser.serialize_field("grantee", &self.grantee)?;
        }
        if let Some(v) = self.allowance.as_ref() {
            struct_ser.serialize_field("allowance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgGrantAllowance {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "granter",
            "grantee",
            "allowance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Granter,
            Grantee,
            Allowance,
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
                            "granter" => Ok(GeneratedField::Granter),
                            "grantee" => Ok(GeneratedField::Grantee),
                            "allowance" => Ok(GeneratedField::Allowance),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgGrantAllowance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.MsgGrantAllowance")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgGrantAllowance, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut granter__ = None;
                let mut grantee__ = None;
                let mut allowance__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Granter => {
                            if granter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("granter"));
                            }
                            granter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Grantee => {
                            if grantee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantee"));
                            }
                            grantee__ = Some(map.next_value()?);
                        }
                        GeneratedField::Allowance => {
                            if allowance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowance"));
                            }
                            allowance__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgGrantAllowance {
                    granter: granter__.unwrap_or_default(),
                    grantee: grantee__.unwrap_or_default(),
                    allowance: allowance__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.MsgGrantAllowance", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgGrantAllowanceResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.MsgGrantAllowanceResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgGrantAllowanceResponse {
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
            type Value = MsgGrantAllowanceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.MsgGrantAllowanceResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgGrantAllowanceResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgGrantAllowanceResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.MsgGrantAllowanceResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRevokeAllowance {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.granter.is_empty() {
            len += 1;
        }
        if !self.grantee.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.MsgRevokeAllowance", len)?;
        if !self.granter.is_empty() {
            struct_ser.serialize_field("granter", &self.granter)?;
        }
        if !self.grantee.is_empty() {
            struct_ser.serialize_field("grantee", &self.grantee)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRevokeAllowance {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "granter",
            "grantee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Granter,
            Grantee,
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
                            "granter" => Ok(GeneratedField::Granter),
                            "grantee" => Ok(GeneratedField::Grantee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgRevokeAllowance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.MsgRevokeAllowance")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgRevokeAllowance, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut granter__ = None;
                let mut grantee__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Granter => {
                            if granter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("granter"));
                            }
                            granter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Grantee => {
                            if grantee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantee"));
                            }
                            grantee__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgRevokeAllowance {
                    granter: granter__.unwrap_or_default(),
                    grantee: grantee__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.MsgRevokeAllowance", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRevokeAllowanceResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.MsgRevokeAllowanceResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRevokeAllowanceResponse {
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
            type Value = MsgRevokeAllowanceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.MsgRevokeAllowanceResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgRevokeAllowanceResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgRevokeAllowanceResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.MsgRevokeAllowanceResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PeriodicAllowance {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.basic.is_some() {
            len += 1;
        }
        if self.period.is_some() {
            len += 1;
        }
        if !self.period_spend_limit.is_empty() {
            len += 1;
        }
        if !self.period_can_spend.is_empty() {
            len += 1;
        }
        if self.period_reset.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.PeriodicAllowance", len)?;
        if let Some(v) = self.basic.as_ref() {
            struct_ser.serialize_field("basic", v)?;
        }
        if let Some(v) = self.period.as_ref() {
            struct_ser.serialize_field("period", v)?;
        }
        if !self.period_spend_limit.is_empty() {
            struct_ser.serialize_field("periodSpendLimit", &self.period_spend_limit)?;
        }
        if !self.period_can_spend.is_empty() {
            struct_ser.serialize_field("periodCanSpend", &self.period_can_spend)?;
        }
        if let Some(v) = self.period_reset.as_ref() {
            struct_ser.serialize_field("periodReset", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PeriodicAllowance {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "basic",
            "period",
            "period_spend_limit",
            "periodSpendLimit",
            "period_can_spend",
            "periodCanSpend",
            "period_reset",
            "periodReset",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Basic,
            Period,
            PeriodSpendLimit,
            PeriodCanSpend,
            PeriodReset,
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
                            "basic" => Ok(GeneratedField::Basic),
                            "period" => Ok(GeneratedField::Period),
                            "periodSpendLimit" | "period_spend_limit" => Ok(GeneratedField::PeriodSpendLimit),
                            "periodCanSpend" | "period_can_spend" => Ok(GeneratedField::PeriodCanSpend),
                            "periodReset" | "period_reset" => Ok(GeneratedField::PeriodReset),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PeriodicAllowance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.PeriodicAllowance")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PeriodicAllowance, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut basic__ = None;
                let mut period__ = None;
                let mut period_spend_limit__ = None;
                let mut period_can_spend__ = None;
                let mut period_reset__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Basic => {
                            if basic__.is_some() {
                                return Err(serde::de::Error::duplicate_field("basic"));
                            }
                            basic__ = map.next_value()?;
                        }
                        GeneratedField::Period => {
                            if period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("period"));
                            }
                            period__ = map.next_value()?;
                        }
                        GeneratedField::PeriodSpendLimit => {
                            if period_spend_limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("periodSpendLimit"));
                            }
                            period_spend_limit__ = Some(map.next_value()?);
                        }
                        GeneratedField::PeriodCanSpend => {
                            if period_can_spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("periodCanSpend"));
                            }
                            period_can_spend__ = Some(map.next_value()?);
                        }
                        GeneratedField::PeriodReset => {
                            if period_reset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("periodReset"));
                            }
                            period_reset__ = map.next_value()?;
                        }
                    }
                }
                Ok(PeriodicAllowance {
                    basic: basic__,
                    period: period__,
                    period_spend_limit: period_spend_limit__.unwrap_or_default(),
                    period_can_spend: period_can_spend__.unwrap_or_default(),
                    period_reset: period_reset__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.PeriodicAllowance", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryAllowanceRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.granter.is_empty() {
            len += 1;
        }
        if !self.grantee.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.QueryAllowanceRequest", len)?;
        if !self.granter.is_empty() {
            struct_ser.serialize_field("granter", &self.granter)?;
        }
        if !self.grantee.is_empty() {
            struct_ser.serialize_field("grantee", &self.grantee)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryAllowanceRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "granter",
            "grantee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Granter,
            Grantee,
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
                            "granter" => Ok(GeneratedField::Granter),
                            "grantee" => Ok(GeneratedField::Grantee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryAllowanceRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.QueryAllowanceRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryAllowanceRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut granter__ = None;
                let mut grantee__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Granter => {
                            if granter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("granter"));
                            }
                            granter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Grantee => {
                            if grantee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantee"));
                            }
                            grantee__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryAllowanceRequest {
                    granter: granter__.unwrap_or_default(),
                    grantee: grantee__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.QueryAllowanceRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryAllowanceResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.allowance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.QueryAllowanceResponse", len)?;
        if let Some(v) = self.allowance.as_ref() {
            struct_ser.serialize_field("allowance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryAllowanceResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "allowance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Allowance,
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
                            "allowance" => Ok(GeneratedField::Allowance),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryAllowanceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.QueryAllowanceResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryAllowanceResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut allowance__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Allowance => {
                            if allowance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowance"));
                            }
                            allowance__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryAllowanceResponse {
                    allowance: allowance__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.QueryAllowanceResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryAllowancesByGranterRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.granter.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.QueryAllowancesByGranterRequest", len)?;
        if !self.granter.is_empty() {
            struct_ser.serialize_field("granter", &self.granter)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryAllowancesByGranterRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "granter",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Granter,
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
                            "granter" => Ok(GeneratedField::Granter),
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
            type Value = QueryAllowancesByGranterRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.QueryAllowancesByGranterRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryAllowancesByGranterRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut granter__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Granter => {
                            if granter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("granter"));
                            }
                            granter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryAllowancesByGranterRequest {
                    granter: granter__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.QueryAllowancesByGranterRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryAllowancesByGranterResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.allowances.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.QueryAllowancesByGranterResponse", len)?;
        if !self.allowances.is_empty() {
            struct_ser.serialize_field("allowances", &self.allowances)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryAllowancesByGranterResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "allowances",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Allowances,
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
                            "allowances" => Ok(GeneratedField::Allowances),
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
            type Value = QueryAllowancesByGranterResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.QueryAllowancesByGranterResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryAllowancesByGranterResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut allowances__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Allowances => {
                            if allowances__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowances"));
                            }
                            allowances__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryAllowancesByGranterResponse {
                    allowances: allowances__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.QueryAllowancesByGranterResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryAllowancesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.grantee.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.QueryAllowancesRequest", len)?;
        if !self.grantee.is_empty() {
            struct_ser.serialize_field("grantee", &self.grantee)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryAllowancesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "grantee",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Grantee,
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
                            "grantee" => Ok(GeneratedField::Grantee),
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
            type Value = QueryAllowancesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.QueryAllowancesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryAllowancesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut grantee__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Grantee => {
                            if grantee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("grantee"));
                            }
                            grantee__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryAllowancesRequest {
                    grantee: grantee__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.QueryAllowancesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryAllowancesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.allowances.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.feegrant.v1beta1.QueryAllowancesResponse", len)?;
        if !self.allowances.is_empty() {
            struct_ser.serialize_field("allowances", &self.allowances)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryAllowancesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "allowances",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Allowances,
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
                            "allowances" => Ok(GeneratedField::Allowances),
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
            type Value = QueryAllowancesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.feegrant.v1beta1.QueryAllowancesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryAllowancesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut allowances__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Allowances => {
                            if allowances__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowances"));
                            }
                            allowances__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryAllowancesResponse {
                    allowances: allowances__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.feegrant.v1beta1.QueryAllowancesResponse", FIELDS, GeneratedVisitor)
    }
}
