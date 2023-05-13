// @generated
impl serde::Serialize for GenesisState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.minter.is_some() {
            len += 1;
        }
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.GenesisState", len)?;
        if let Some(v) = self.minter.as_ref() {
            struct_ser.serialize_field("minter", v)?;
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
            "minter",
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Minter,
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
                            "minter" => Ok(GeneratedField::Minter),
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
                formatter.write_str("struct cosmos.mint.v1beta1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut minter__ = None;
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Minter => {
                            if minter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minter"));
                            }
                            minter__ = map.next_value()?;
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
                    minter: minter__,
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.mint.v1beta1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Minter {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.inflation.is_empty() {
            len += 1;
        }
        if !self.annual_provisions.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.Minter", len)?;
        if !self.inflation.is_empty() {
            struct_ser.serialize_field("inflation", &self.inflation)?;
        }
        if !self.annual_provisions.is_empty() {
            struct_ser.serialize_field("annualProvisions", &self.annual_provisions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Minter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inflation",
            "annual_provisions",
            "annualProvisions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inflation,
            AnnualProvisions,
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
                            "inflation" => Ok(GeneratedField::Inflation),
                            "annualProvisions" | "annual_provisions" => Ok(GeneratedField::AnnualProvisions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Minter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.mint.v1beta1.Minter")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Minter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inflation__ = None;
                let mut annual_provisions__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Inflation => {
                            if inflation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inflation"));
                            }
                            inflation__ = Some(map.next_value()?);
                        }
                        GeneratedField::AnnualProvisions => {
                            if annual_provisions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annualProvisions"));
                            }
                            annual_provisions__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Minter {
                    inflation: inflation__.unwrap_or_default(),
                    annual_provisions: annual_provisions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.mint.v1beta1.Minter", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.MsgUpdateParams", len)?;
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
                formatter.write_str("struct cosmos.mint.v1beta1.MsgUpdateParams")
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
        deserializer.deserialize_struct("cosmos.mint.v1beta1.MsgUpdateParams", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.MsgUpdateParamsResponse", len)?;
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
                formatter.write_str("struct cosmos.mint.v1beta1.MsgUpdateParamsResponse")
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
        deserializer.deserialize_struct("cosmos.mint.v1beta1.MsgUpdateParamsResponse", FIELDS, GeneratedVisitor)
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
        if !self.mint_denom.is_empty() {
            len += 1;
        }
        if !self.inflation_rate_change.is_empty() {
            len += 1;
        }
        if !self.inflation_max.is_empty() {
            len += 1;
        }
        if !self.inflation_min.is_empty() {
            len += 1;
        }
        if !self.goal_bonded.is_empty() {
            len += 1;
        }
        if self.blocks_per_year != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.Params", len)?;
        if !self.mint_denom.is_empty() {
            struct_ser.serialize_field("mintDenom", &self.mint_denom)?;
        }
        if !self.inflation_rate_change.is_empty() {
            struct_ser.serialize_field("inflationRateChange", &self.inflation_rate_change)?;
        }
        if !self.inflation_max.is_empty() {
            struct_ser.serialize_field("inflationMax", &self.inflation_max)?;
        }
        if !self.inflation_min.is_empty() {
            struct_ser.serialize_field("inflationMin", &self.inflation_min)?;
        }
        if !self.goal_bonded.is_empty() {
            struct_ser.serialize_field("goalBonded", &self.goal_bonded)?;
        }
        if self.blocks_per_year != 0 {
            struct_ser.serialize_field("blocksPerYear", ToString::to_string(&self.blocks_per_year).as_str())?;
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
            "mint_denom",
            "mintDenom",
            "inflation_rate_change",
            "inflationRateChange",
            "inflation_max",
            "inflationMax",
            "inflation_min",
            "inflationMin",
            "goal_bonded",
            "goalBonded",
            "blocks_per_year",
            "blocksPerYear",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MintDenom,
            InflationRateChange,
            InflationMax,
            InflationMin,
            GoalBonded,
            BlocksPerYear,
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
                            "mintDenom" | "mint_denom" => Ok(GeneratedField::MintDenom),
                            "inflationRateChange" | "inflation_rate_change" => Ok(GeneratedField::InflationRateChange),
                            "inflationMax" | "inflation_max" => Ok(GeneratedField::InflationMax),
                            "inflationMin" | "inflation_min" => Ok(GeneratedField::InflationMin),
                            "goalBonded" | "goal_bonded" => Ok(GeneratedField::GoalBonded),
                            "blocksPerYear" | "blocks_per_year" => Ok(GeneratedField::BlocksPerYear),
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
                formatter.write_str("struct cosmos.mint.v1beta1.Params")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Params, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mint_denom__ = None;
                let mut inflation_rate_change__ = None;
                let mut inflation_max__ = None;
                let mut inflation_min__ = None;
                let mut goal_bonded__ = None;
                let mut blocks_per_year__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MintDenom => {
                            if mint_denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mintDenom"));
                            }
                            mint_denom__ = Some(map.next_value()?);
                        }
                        GeneratedField::InflationRateChange => {
                            if inflation_rate_change__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inflationRateChange"));
                            }
                            inflation_rate_change__ = Some(map.next_value()?);
                        }
                        GeneratedField::InflationMax => {
                            if inflation_max__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inflationMax"));
                            }
                            inflation_max__ = Some(map.next_value()?);
                        }
                        GeneratedField::InflationMin => {
                            if inflation_min__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inflationMin"));
                            }
                            inflation_min__ = Some(map.next_value()?);
                        }
                        GeneratedField::GoalBonded => {
                            if goal_bonded__.is_some() {
                                return Err(serde::de::Error::duplicate_field("goalBonded"));
                            }
                            goal_bonded__ = Some(map.next_value()?);
                        }
                        GeneratedField::BlocksPerYear => {
                            if blocks_per_year__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blocksPerYear"));
                            }
                            blocks_per_year__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Params {
                    mint_denom: mint_denom__.unwrap_or_default(),
                    inflation_rate_change: inflation_rate_change__.unwrap_or_default(),
                    inflation_max: inflation_max__.unwrap_or_default(),
                    inflation_min: inflation_min__.unwrap_or_default(),
                    goal_bonded: goal_bonded__.unwrap_or_default(),
                    blocks_per_year: blocks_per_year__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.mint.v1beta1.Params", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryAnnualProvisionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.QueryAnnualProvisionsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryAnnualProvisionsRequest {
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
            type Value = QueryAnnualProvisionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.mint.v1beta1.QueryAnnualProvisionsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryAnnualProvisionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(QueryAnnualProvisionsRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.mint.v1beta1.QueryAnnualProvisionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryAnnualProvisionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.annual_provisions.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.QueryAnnualProvisionsResponse", len)?;
        if !self.annual_provisions.is_empty() {
            struct_ser.serialize_field("annualProvisions", pbjson::private::base64::encode(&self.annual_provisions).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryAnnualProvisionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "annual_provisions",
            "annualProvisions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AnnualProvisions,
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
                            "annualProvisions" | "annual_provisions" => Ok(GeneratedField::AnnualProvisions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryAnnualProvisionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.mint.v1beta1.QueryAnnualProvisionsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryAnnualProvisionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut annual_provisions__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AnnualProvisions => {
                            if annual_provisions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annualProvisions"));
                            }
                            annual_provisions__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryAnnualProvisionsResponse {
                    annual_provisions: annual_provisions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.mint.v1beta1.QueryAnnualProvisionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryInflationRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.QueryInflationRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryInflationRequest {
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
            type Value = QueryInflationRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.mint.v1beta1.QueryInflationRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryInflationRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(QueryInflationRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.mint.v1beta1.QueryInflationRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryInflationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.inflation.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.QueryInflationResponse", len)?;
        if !self.inflation.is_empty() {
            struct_ser.serialize_field("inflation", pbjson::private::base64::encode(&self.inflation).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryInflationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inflation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inflation,
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
                            "inflation" => Ok(GeneratedField::Inflation),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryInflationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.mint.v1beta1.QueryInflationResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryInflationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inflation__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Inflation => {
                            if inflation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inflation"));
                            }
                            inflation__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryInflationResponse {
                    inflation: inflation__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.mint.v1beta1.QueryInflationResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.QueryParamsRequest", len)?;
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
                formatter.write_str("struct cosmos.mint.v1beta1.QueryParamsRequest")
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
        deserializer.deserialize_struct("cosmos.mint.v1beta1.QueryParamsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.mint.v1beta1.QueryParamsResponse", len)?;
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
                formatter.write_str("struct cosmos.mint.v1beta1.QueryParamsResponse")
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
        deserializer.deserialize_struct("cosmos.mint.v1beta1.QueryParamsResponse", FIELDS, GeneratedVisitor)
    }
}
