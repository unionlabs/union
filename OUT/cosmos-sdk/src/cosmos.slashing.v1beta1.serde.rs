// @generated
impl serde::Serialize for GenesisState {
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
        if !self.signing_infos.is_empty() {
            len += 1;
        }
        if !self.missed_blocks.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.GenesisState", len)?;
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        if !self.signing_infos.is_empty() {
            struct_ser.serialize_field("signingInfos", &self.signing_infos)?;
        }
        if !self.missed_blocks.is_empty() {
            struct_ser.serialize_field("missedBlocks", &self.missed_blocks)?;
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
            "params",
            "signing_infos",
            "signingInfos",
            "missed_blocks",
            "missedBlocks",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Params,
            SigningInfos,
            MissedBlocks,
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
                            "signingInfos" | "signing_infos" => Ok(GeneratedField::SigningInfos),
                            "missedBlocks" | "missed_blocks" => Ok(GeneratedField::MissedBlocks),
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
                formatter.write_str("struct cosmos.slashing.v1beta1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut params__ = None;
                let mut signing_infos__ = None;
                let mut missed_blocks__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                        GeneratedField::SigningInfos => {
                            if signing_infos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signingInfos"));
                            }
                            signing_infos__ = Some(map.next_value()?);
                        }
                        GeneratedField::MissedBlocks => {
                            if missed_blocks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("missedBlocks"));
                            }
                            missed_blocks__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenesisState {
                    params: params__,
                    signing_infos: signing_infos__.unwrap_or_default(),
                    missed_blocks: missed_blocks__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MissedBlock {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.index != 0 {
            len += 1;
        }
        if self.missed {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.MissedBlock", len)?;
        if self.index != 0 {
            struct_ser.serialize_field("index", ToString::to_string(&self.index).as_str())?;
        }
        if self.missed {
            struct_ser.serialize_field("missed", &self.missed)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MissedBlock {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "index",
            "missed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Index,
            Missed,
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
                            "index" => Ok(GeneratedField::Index),
                            "missed" => Ok(GeneratedField::Missed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MissedBlock;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.MissedBlock")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MissedBlock, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut index__ = None;
                let mut missed__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Missed => {
                            if missed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("missed"));
                            }
                            missed__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MissedBlock {
                    index: index__.unwrap_or_default(),
                    missed: missed__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.MissedBlock", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUnjail {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_addr.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.MsgUnjail", len)?;
        if !self.validator_addr.is_empty() {
            struct_ser.serialize_field("validatorAddr", &self.validator_addr)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUnjail {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_addr",
            "validatorAddr",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddr,
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
                            "validatorAddr" | "validator_addr" => Ok(GeneratedField::ValidatorAddr),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUnjail;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.MsgUnjail")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUnjail, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_addr__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddr => {
                            if validator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddr"));
                            }
                            validator_addr__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgUnjail {
                    validator_addr: validator_addr__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.MsgUnjail", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUnjailResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.MsgUnjailResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUnjailResponse {
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
            type Value = MsgUnjailResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.MsgUnjailResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUnjailResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgUnjailResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.MsgUnjailResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.authority.is_empty() {
            len += 1;
        }
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.MsgUpdateParams", len)?;
        if !self.authority.is_empty() {
            struct_ser.serialize_field("authority", &self.authority)?;
        }
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authority",
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Authority,
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
                            "authority" => Ok(GeneratedField::Authority),
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
            type Value = MsgUpdateParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.MsgUpdateParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authority__ = None;
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Authority => {
                            if authority__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authority"));
                            }
                            authority__ = Some(map.next_value()?);
                        }
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgUpdateParams {
                    authority: authority__.unwrap_or_default(),
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.MsgUpdateParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateParamsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.MsgUpdateParamsResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateParamsResponse {
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
            type Value = MsgUpdateParamsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.MsgUpdateParamsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateParamsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgUpdateParamsResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.MsgUpdateParamsResponse", FIELDS, GeneratedVisitor)
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
        if self.signed_blocks_window != 0 {
            len += 1;
        }
        if !self.min_signed_per_window.is_empty() {
            len += 1;
        }
        if self.downtime_jail_duration.is_some() {
            len += 1;
        }
        if !self.slash_fraction_double_sign.is_empty() {
            len += 1;
        }
        if !self.slash_fraction_downtime.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.Params", len)?;
        if self.signed_blocks_window != 0 {
            struct_ser.serialize_field("signedBlocksWindow", ToString::to_string(&self.signed_blocks_window).as_str())?;
        }
        if !self.min_signed_per_window.is_empty() {
            struct_ser.serialize_field("minSignedPerWindow", pbjson::private::base64::encode(&self.min_signed_per_window).as_str())?;
        }
        if let Some(v) = self.downtime_jail_duration.as_ref() {
            struct_ser.serialize_field("downtimeJailDuration", v)?;
        }
        if !self.slash_fraction_double_sign.is_empty() {
            struct_ser.serialize_field("slashFractionDoubleSign", pbjson::private::base64::encode(&self.slash_fraction_double_sign).as_str())?;
        }
        if !self.slash_fraction_downtime.is_empty() {
            struct_ser.serialize_field("slashFractionDowntime", pbjson::private::base64::encode(&self.slash_fraction_downtime).as_str())?;
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
            "signed_blocks_window",
            "signedBlocksWindow",
            "min_signed_per_window",
            "minSignedPerWindow",
            "downtime_jail_duration",
            "downtimeJailDuration",
            "slash_fraction_double_sign",
            "slashFractionDoubleSign",
            "slash_fraction_downtime",
            "slashFractionDowntime",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignedBlocksWindow,
            MinSignedPerWindow,
            DowntimeJailDuration,
            SlashFractionDoubleSign,
            SlashFractionDowntime,
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
                            "signedBlocksWindow" | "signed_blocks_window" => Ok(GeneratedField::SignedBlocksWindow),
                            "minSignedPerWindow" | "min_signed_per_window" => Ok(GeneratedField::MinSignedPerWindow),
                            "downtimeJailDuration" | "downtime_jail_duration" => Ok(GeneratedField::DowntimeJailDuration),
                            "slashFractionDoubleSign" | "slash_fraction_double_sign" => Ok(GeneratedField::SlashFractionDoubleSign),
                            "slashFractionDowntime" | "slash_fraction_downtime" => Ok(GeneratedField::SlashFractionDowntime),
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
                formatter.write_str("struct cosmos.slashing.v1beta1.Params")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Params, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signed_blocks_window__ = None;
                let mut min_signed_per_window__ = None;
                let mut downtime_jail_duration__ = None;
                let mut slash_fraction_double_sign__ = None;
                let mut slash_fraction_downtime__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SignedBlocksWindow => {
                            if signed_blocks_window__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signedBlocksWindow"));
                            }
                            signed_blocks_window__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MinSignedPerWindow => {
                            if min_signed_per_window__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minSignedPerWindow"));
                            }
                            min_signed_per_window__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DowntimeJailDuration => {
                            if downtime_jail_duration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("downtimeJailDuration"));
                            }
                            downtime_jail_duration__ = map.next_value()?;
                        }
                        GeneratedField::SlashFractionDoubleSign => {
                            if slash_fraction_double_sign__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashFractionDoubleSign"));
                            }
                            slash_fraction_double_sign__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SlashFractionDowntime => {
                            if slash_fraction_downtime__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashFractionDowntime"));
                            }
                            slash_fraction_downtime__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Params {
                    signed_blocks_window: signed_blocks_window__.unwrap_or_default(),
                    min_signed_per_window: min_signed_per_window__.unwrap_or_default(),
                    downtime_jail_duration: downtime_jail_duration__,
                    slash_fraction_double_sign: slash_fraction_double_sign__.unwrap_or_default(),
                    slash_fraction_downtime: slash_fraction_downtime__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.Params", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.QueryParamsRequest", len)?;
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
                formatter.write_str("struct cosmos.slashing.v1beta1.QueryParamsRequest")
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
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.QueryParamsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.QueryParamsResponse", len)?;
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
                formatter.write_str("struct cosmos.slashing.v1beta1.QueryParamsResponse")
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
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.QueryParamsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QuerySigningInfoRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.cons_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.QuerySigningInfoRequest", len)?;
        if !self.cons_address.is_empty() {
            struct_ser.serialize_field("consAddress", &self.cons_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QuerySigningInfoRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cons_address",
            "consAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConsAddress,
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
                            "consAddress" | "cons_address" => Ok(GeneratedField::ConsAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QuerySigningInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.QuerySigningInfoRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QuerySigningInfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cons_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConsAddress => {
                            if cons_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consAddress"));
                            }
                            cons_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QuerySigningInfoRequest {
                    cons_address: cons_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.QuerySigningInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QuerySigningInfoResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.val_signing_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.QuerySigningInfoResponse", len)?;
        if let Some(v) = self.val_signing_info.as_ref() {
            struct_ser.serialize_field("valSigningInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QuerySigningInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "val_signing_info",
            "valSigningInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValSigningInfo,
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
                            "valSigningInfo" | "val_signing_info" => Ok(GeneratedField::ValSigningInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QuerySigningInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.QuerySigningInfoResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QuerySigningInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut val_signing_info__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValSigningInfo => {
                            if val_signing_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valSigningInfo"));
                            }
                            val_signing_info__ = map.next_value()?;
                        }
                    }
                }
                Ok(QuerySigningInfoResponse {
                    val_signing_info: val_signing_info__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.QuerySigningInfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QuerySigningInfosRequest {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.QuerySigningInfosRequest", len)?;
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QuerySigningInfosRequest {
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
            type Value = QuerySigningInfosRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.QuerySigningInfosRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QuerySigningInfosRequest, V::Error>
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
                Ok(QuerySigningInfosRequest {
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.QuerySigningInfosRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QuerySigningInfosResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.info.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.QuerySigningInfosResponse", len)?;
        if !self.info.is_empty() {
            struct_ser.serialize_field("info", &self.info)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QuerySigningInfosResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "info",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Info,
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
                            "info" => Ok(GeneratedField::Info),
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
            type Value = QuerySigningInfosResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.QuerySigningInfosResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QuerySigningInfosResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut info__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Info => {
                            if info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("info"));
                            }
                            info__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QuerySigningInfosResponse {
                    info: info__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.QuerySigningInfosResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SigningInfo {
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
        if self.validator_signing_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.SigningInfo", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if let Some(v) = self.validator_signing_info.as_ref() {
            struct_ser.serialize_field("validatorSigningInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SigningInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "validator_signing_info",
            "validatorSigningInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            ValidatorSigningInfo,
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
                            "validatorSigningInfo" | "validator_signing_info" => Ok(GeneratedField::ValidatorSigningInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SigningInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.SigningInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SigningInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut validator_signing_info__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorSigningInfo => {
                            if validator_signing_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSigningInfo"));
                            }
                            validator_signing_info__ = map.next_value()?;
                        }
                    }
                }
                Ok(SigningInfo {
                    address: address__.unwrap_or_default(),
                    validator_signing_info: validator_signing_info__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.SigningInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorMissedBlocks {
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
        if !self.missed_blocks.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.ValidatorMissedBlocks", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if !self.missed_blocks.is_empty() {
            struct_ser.serialize_field("missedBlocks", &self.missed_blocks)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorMissedBlocks {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "missed_blocks",
            "missedBlocks",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            MissedBlocks,
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
                            "missedBlocks" | "missed_blocks" => Ok(GeneratedField::MissedBlocks),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorMissedBlocks;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.ValidatorMissedBlocks")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorMissedBlocks, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut missed_blocks__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::MissedBlocks => {
                            if missed_blocks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("missedBlocks"));
                            }
                            missed_blocks__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValidatorMissedBlocks {
                    address: address__.unwrap_or_default(),
                    missed_blocks: missed_blocks__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.ValidatorMissedBlocks", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorSigningInfo {
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
        if self.start_height != 0 {
            len += 1;
        }
        if self.index_offset != 0 {
            len += 1;
        }
        if self.jailed_until.is_some() {
            len += 1;
        }
        if self.tombstoned {
            len += 1;
        }
        if self.missed_blocks_counter != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.slashing.v1beta1.ValidatorSigningInfo", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if self.start_height != 0 {
            struct_ser.serialize_field("startHeight", ToString::to_string(&self.start_height).as_str())?;
        }
        if self.index_offset != 0 {
            struct_ser.serialize_field("indexOffset", ToString::to_string(&self.index_offset).as_str())?;
        }
        if let Some(v) = self.jailed_until.as_ref() {
            struct_ser.serialize_field("jailedUntil", v)?;
        }
        if self.tombstoned {
            struct_ser.serialize_field("tombstoned", &self.tombstoned)?;
        }
        if self.missed_blocks_counter != 0 {
            struct_ser.serialize_field("missedBlocksCounter", ToString::to_string(&self.missed_blocks_counter).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorSigningInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "start_height",
            "startHeight",
            "index_offset",
            "indexOffset",
            "jailed_until",
            "jailedUntil",
            "tombstoned",
            "missed_blocks_counter",
            "missedBlocksCounter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            StartHeight,
            IndexOffset,
            JailedUntil,
            Tombstoned,
            MissedBlocksCounter,
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
                            "startHeight" | "start_height" => Ok(GeneratedField::StartHeight),
                            "indexOffset" | "index_offset" => Ok(GeneratedField::IndexOffset),
                            "jailedUntil" | "jailed_until" => Ok(GeneratedField::JailedUntil),
                            "tombstoned" => Ok(GeneratedField::Tombstoned),
                            "missedBlocksCounter" | "missed_blocks_counter" => Ok(GeneratedField::MissedBlocksCounter),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorSigningInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.slashing.v1beta1.ValidatorSigningInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorSigningInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut start_height__ = None;
                let mut index_offset__ = None;
                let mut jailed_until__ = None;
                let mut tombstoned__ = None;
                let mut missed_blocks_counter__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::StartHeight => {
                            if start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startHeight"));
                            }
                            start_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::IndexOffset => {
                            if index_offset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("indexOffset"));
                            }
                            index_offset__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::JailedUntil => {
                            if jailed_until__.is_some() {
                                return Err(serde::de::Error::duplicate_field("jailedUntil"));
                            }
                            jailed_until__ = map.next_value()?;
                        }
                        GeneratedField::Tombstoned => {
                            if tombstoned__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tombstoned"));
                            }
                            tombstoned__ = Some(map.next_value()?);
                        }
                        GeneratedField::MissedBlocksCounter => {
                            if missed_blocks_counter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("missedBlocksCounter"));
                            }
                            missed_blocks_counter__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ValidatorSigningInfo {
                    address: address__.unwrap_or_default(),
                    start_height: start_height__.unwrap_or_default(),
                    index_offset: index_offset__.unwrap_or_default(),
                    jailed_until: jailed_until__,
                    tombstoned: tombstoned__.unwrap_or_default(),
                    missed_blocks_counter: missed_blocks_counter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.slashing.v1beta1.ValidatorSigningInfo", FIELDS, GeneratedVisitor)
    }
}
