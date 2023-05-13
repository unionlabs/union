// @generated
impl serde::Serialize for AuthorizationType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "AUTHORIZATION_TYPE_UNSPECIFIED",
            Self::Delegate => "AUTHORIZATION_TYPE_DELEGATE",
            Self::Undelegate => "AUTHORIZATION_TYPE_UNDELEGATE",
            Self::Redelegate => "AUTHORIZATION_TYPE_REDELEGATE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizationType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "AUTHORIZATION_TYPE_UNSPECIFIED",
            "AUTHORIZATION_TYPE_DELEGATE",
            "AUTHORIZATION_TYPE_UNDELEGATE",
            "AUTHORIZATION_TYPE_REDELEGATE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizationType;

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
                    .and_then(AuthorizationType::from_i32)
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
                    .and_then(AuthorizationType::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "AUTHORIZATION_TYPE_UNSPECIFIED" => Ok(AuthorizationType::Unspecified),
                    "AUTHORIZATION_TYPE_DELEGATE" => Ok(AuthorizationType::Delegate),
                    "AUTHORIZATION_TYPE_UNDELEGATE" => Ok(AuthorizationType::Undelegate),
                    "AUTHORIZATION_TYPE_REDELEGATE" => Ok(AuthorizationType::Redelegate),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for BondStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "BOND_STATUS_UNSPECIFIED",
            Self::Unbonded => "BOND_STATUS_UNBONDED",
            Self::Unbonding => "BOND_STATUS_UNBONDING",
            Self::Bonded => "BOND_STATUS_BONDED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for BondStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "BOND_STATUS_UNSPECIFIED",
            "BOND_STATUS_UNBONDED",
            "BOND_STATUS_UNBONDING",
            "BOND_STATUS_BONDED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BondStatus;

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
                    .and_then(BondStatus::from_i32)
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
                    .and_then(BondStatus::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "BOND_STATUS_UNSPECIFIED" => Ok(BondStatus::Unspecified),
                    "BOND_STATUS_UNBONDED" => Ok(BondStatus::Unbonded),
                    "BOND_STATUS_UNBONDING" => Ok(BondStatus::Unbonding),
                    "BOND_STATUS_BONDED" => Ok(BondStatus::Bonded),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Commission {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.commission_rates.is_some() {
            len += 1;
        }
        if self.update_time.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.Commission", len)?;
        if let Some(v) = self.commission_rates.as_ref() {
            struct_ser.serialize_field("commissionRates", v)?;
        }
        if let Some(v) = self.update_time.as_ref() {
            struct_ser.serialize_field("updateTime", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Commission {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commission_rates",
            "commissionRates",
            "update_time",
            "updateTime",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CommissionRates,
            UpdateTime,
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
                            "commissionRates" | "commission_rates" => Ok(GeneratedField::CommissionRates),
                            "updateTime" | "update_time" => Ok(GeneratedField::UpdateTime),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Commission;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.Commission")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Commission, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commission_rates__ = None;
                let mut update_time__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CommissionRates => {
                            if commission_rates__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commissionRates"));
                            }
                            commission_rates__ = map.next_value()?;
                        }
                        GeneratedField::UpdateTime => {
                            if update_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateTime"));
                            }
                            update_time__ = map.next_value()?;
                        }
                    }
                }
                Ok(Commission {
                    commission_rates: commission_rates__,
                    update_time: update_time__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.Commission", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CommissionRates {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.rate.is_empty() {
            len += 1;
        }
        if !self.max_rate.is_empty() {
            len += 1;
        }
        if !self.max_change_rate.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.CommissionRates", len)?;
        if !self.rate.is_empty() {
            struct_ser.serialize_field("rate", &self.rate)?;
        }
        if !self.max_rate.is_empty() {
            struct_ser.serialize_field("maxRate", &self.max_rate)?;
        }
        if !self.max_change_rate.is_empty() {
            struct_ser.serialize_field("maxChangeRate", &self.max_change_rate)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommissionRates {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rate",
            "max_rate",
            "maxRate",
            "max_change_rate",
            "maxChangeRate",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rate,
            MaxRate,
            MaxChangeRate,
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
                            "rate" => Ok(GeneratedField::Rate),
                            "maxRate" | "max_rate" => Ok(GeneratedField::MaxRate),
                            "maxChangeRate" | "max_change_rate" => Ok(GeneratedField::MaxChangeRate),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CommissionRates;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.CommissionRates")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CommissionRates, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rate__ = None;
                let mut max_rate__ = None;
                let mut max_change_rate__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Rate => {
                            if rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rate"));
                            }
                            rate__ = Some(map.next_value()?);
                        }
                        GeneratedField::MaxRate => {
                            if max_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxRate"));
                            }
                            max_rate__ = Some(map.next_value()?);
                        }
                        GeneratedField::MaxChangeRate => {
                            if max_change_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxChangeRate"));
                            }
                            max_change_rate__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CommissionRates {
                    rate: rate__.unwrap_or_default(),
                    max_rate: max_rate__.unwrap_or_default(),
                    max_change_rate: max_change_rate__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.CommissionRates", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DvPair {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.DVPair", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DvPair {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_address",
            "validatorAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorAddress,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DvPair;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.DVPair")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DvPair, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DvPair {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.DVPair", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DvPairs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.pairs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.DVPairs", len)?;
        if !self.pairs.is_empty() {
            struct_ser.serialize_field("pairs", &self.pairs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DvPairs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pairs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pairs,
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
                            "pairs" => Ok(GeneratedField::Pairs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DvPairs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.DVPairs")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DvPairs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pairs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Pairs => {
                            if pairs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pairs"));
                            }
                            pairs__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DvPairs {
                    pairs: pairs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.DVPairs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DvvTriplet {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_src_address.is_empty() {
            len += 1;
        }
        if !self.validator_dst_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.DVVTriplet", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_src_address.is_empty() {
            struct_ser.serialize_field("validatorSrcAddress", &self.validator_src_address)?;
        }
        if !self.validator_dst_address.is_empty() {
            struct_ser.serialize_field("validatorDstAddress", &self.validator_dst_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DvvTriplet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_src_address",
            "validatorSrcAddress",
            "validator_dst_address",
            "validatorDstAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorSrcAddress,
            ValidatorDstAddress,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorSrcAddress" | "validator_src_address" => Ok(GeneratedField::ValidatorSrcAddress),
                            "validatorDstAddress" | "validator_dst_address" => Ok(GeneratedField::ValidatorDstAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DvvTriplet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.DVVTriplet")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DvvTriplet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_src_address__ = None;
                let mut validator_dst_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorSrcAddress => {
                            if validator_src_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSrcAddress"));
                            }
                            validator_src_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorDstAddress => {
                            if validator_dst_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDstAddress"));
                            }
                            validator_dst_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DvvTriplet {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_src_address: validator_src_address__.unwrap_or_default(),
                    validator_dst_address: validator_dst_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.DVVTriplet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DvvTriplets {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.triplets.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.DVVTriplets", len)?;
        if !self.triplets.is_empty() {
            struct_ser.serialize_field("triplets", &self.triplets)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DvvTriplets {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "triplets",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Triplets,
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
                            "triplets" => Ok(GeneratedField::Triplets),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DvvTriplets;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.DVVTriplets")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DvvTriplets, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut triplets__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Triplets => {
                            if triplets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("triplets"));
                            }
                            triplets__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DvvTriplets {
                    triplets: triplets__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.DVVTriplets", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Delegation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if !self.shares.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.Delegation", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if !self.shares.is_empty() {
            struct_ser.serialize_field("shares", &self.shares)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Delegation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_address",
            "validatorAddress",
            "shares",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorAddress,
            Shares,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "shares" => Ok(GeneratedField::Shares),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Delegation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.Delegation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Delegation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_address__ = None;
                let mut shares__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Shares => {
                            if shares__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shares"));
                            }
                            shares__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Delegation {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                    shares: shares__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.Delegation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delegation.is_some() {
            len += 1;
        }
        if self.balance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.DelegationResponse", len)?;
        if let Some(v) = self.delegation.as_ref() {
            struct_ser.serialize_field("delegation", v)?;
        }
        if let Some(v) = self.balance.as_ref() {
            struct_ser.serialize_field("balance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegation",
            "balance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Delegation,
            Balance,
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
                            "delegation" => Ok(GeneratedField::Delegation),
                            "balance" => Ok(GeneratedField::Balance),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.DelegationResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegation__ = None;
                let mut balance__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Delegation => {
                            if delegation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegation"));
                            }
                            delegation__ = map.next_value()?;
                        }
                        GeneratedField::Balance => {
                            if balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balance"));
                            }
                            balance__ = map.next_value()?;
                        }
                    }
                }
                Ok(DelegationResponse {
                    delegation: delegation__,
                    balance: balance__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.DelegationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Description {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.moniker.is_empty() {
            len += 1;
        }
        if !self.identity.is_empty() {
            len += 1;
        }
        if !self.website.is_empty() {
            len += 1;
        }
        if !self.security_contact.is_empty() {
            len += 1;
        }
        if !self.details.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.Description", len)?;
        if !self.moniker.is_empty() {
            struct_ser.serialize_field("moniker", &self.moniker)?;
        }
        if !self.identity.is_empty() {
            struct_ser.serialize_field("identity", &self.identity)?;
        }
        if !self.website.is_empty() {
            struct_ser.serialize_field("website", &self.website)?;
        }
        if !self.security_contact.is_empty() {
            struct_ser.serialize_field("securityContact", &self.security_contact)?;
        }
        if !self.details.is_empty() {
            struct_ser.serialize_field("details", &self.details)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Description {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "moniker",
            "identity",
            "website",
            "security_contact",
            "securityContact",
            "details",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Moniker,
            Identity,
            Website,
            SecurityContact,
            Details,
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
                            "moniker" => Ok(GeneratedField::Moniker),
                            "identity" => Ok(GeneratedField::Identity),
                            "website" => Ok(GeneratedField::Website),
                            "securityContact" | "security_contact" => Ok(GeneratedField::SecurityContact),
                            "details" => Ok(GeneratedField::Details),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Description;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.Description")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Description, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut moniker__ = None;
                let mut identity__ = None;
                let mut website__ = None;
                let mut security_contact__ = None;
                let mut details__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Moniker => {
                            if moniker__.is_some() {
                                return Err(serde::de::Error::duplicate_field("moniker"));
                            }
                            moniker__ = Some(map.next_value()?);
                        }
                        GeneratedField::Identity => {
                            if identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identity"));
                            }
                            identity__ = Some(map.next_value()?);
                        }
                        GeneratedField::Website => {
                            if website__.is_some() {
                                return Err(serde::de::Error::duplicate_field("website"));
                            }
                            website__ = Some(map.next_value()?);
                        }
                        GeneratedField::SecurityContact => {
                            if security_contact__.is_some() {
                                return Err(serde::de::Error::duplicate_field("securityContact"));
                            }
                            security_contact__ = Some(map.next_value()?);
                        }
                        GeneratedField::Details => {
                            if details__.is_some() {
                                return Err(serde::de::Error::duplicate_field("details"));
                            }
                            details__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Description {
                    moniker: moniker__.unwrap_or_default(),
                    identity: identity__.unwrap_or_default(),
                    website: website__.unwrap_or_default(),
                    security_contact: security_contact__.unwrap_or_default(),
                    details: details__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.Description", FIELDS, GeneratedVisitor)
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
        if self.params.is_some() {
            len += 1;
        }
        if !self.last_total_power.is_empty() {
            len += 1;
        }
        if !self.last_validator_powers.is_empty() {
            len += 1;
        }
        if !self.validators.is_empty() {
            len += 1;
        }
        if !self.delegations.is_empty() {
            len += 1;
        }
        if !self.unbonding_delegations.is_empty() {
            len += 1;
        }
        if !self.redelegations.is_empty() {
            len += 1;
        }
        if self.exported {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.GenesisState", len)?;
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        if !self.last_total_power.is_empty() {
            struct_ser.serialize_field("lastTotalPower", pbjson::private::base64::encode(&self.last_total_power).as_str())?;
        }
        if !self.last_validator_powers.is_empty() {
            struct_ser.serialize_field("lastValidatorPowers", &self.last_validator_powers)?;
        }
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
        }
        if !self.delegations.is_empty() {
            struct_ser.serialize_field("delegations", &self.delegations)?;
        }
        if !self.unbonding_delegations.is_empty() {
            struct_ser.serialize_field("unbondingDelegations", &self.unbonding_delegations)?;
        }
        if !self.redelegations.is_empty() {
            struct_ser.serialize_field("redelegations", &self.redelegations)?;
        }
        if self.exported {
            struct_ser.serialize_field("exported", &self.exported)?;
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
            "last_total_power",
            "lastTotalPower",
            "last_validator_powers",
            "lastValidatorPowers",
            "validators",
            "delegations",
            "unbonding_delegations",
            "unbondingDelegations",
            "redelegations",
            "exported",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Params,
            LastTotalPower,
            LastValidatorPowers,
            Validators,
            Delegations,
            UnbondingDelegations,
            Redelegations,
            Exported,
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
                            "lastTotalPower" | "last_total_power" => Ok(GeneratedField::LastTotalPower),
                            "lastValidatorPowers" | "last_validator_powers" => Ok(GeneratedField::LastValidatorPowers),
                            "validators" => Ok(GeneratedField::Validators),
                            "delegations" => Ok(GeneratedField::Delegations),
                            "unbondingDelegations" | "unbonding_delegations" => Ok(GeneratedField::UnbondingDelegations),
                            "redelegations" => Ok(GeneratedField::Redelegations),
                            "exported" => Ok(GeneratedField::Exported),
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
                formatter.write_str("struct cosmos.staking.v1beta1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut params__ = None;
                let mut last_total_power__ = None;
                let mut last_validator_powers__ = None;
                let mut validators__ = None;
                let mut delegations__ = None;
                let mut unbonding_delegations__ = None;
                let mut redelegations__ = None;
                let mut exported__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                        GeneratedField::LastTotalPower => {
                            if last_total_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastTotalPower"));
                            }
                            last_total_power__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LastValidatorPowers => {
                            if last_validator_powers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastValidatorPowers"));
                            }
                            last_validator_powers__ = Some(map.next_value()?);
                        }
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Delegations => {
                            if delegations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegations"));
                            }
                            delegations__ = Some(map.next_value()?);
                        }
                        GeneratedField::UnbondingDelegations => {
                            if unbonding_delegations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingDelegations"));
                            }
                            unbonding_delegations__ = Some(map.next_value()?);
                        }
                        GeneratedField::Redelegations => {
                            if redelegations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("redelegations"));
                            }
                            redelegations__ = Some(map.next_value()?);
                        }
                        GeneratedField::Exported => {
                            if exported__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exported"));
                            }
                            exported__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenesisState {
                    params: params__,
                    last_total_power: last_total_power__.unwrap_or_default(),
                    last_validator_powers: last_validator_powers__.unwrap_or_default(),
                    validators: validators__.unwrap_or_default(),
                    delegations: delegations__.unwrap_or_default(),
                    unbonding_delegations: unbonding_delegations__.unwrap_or_default(),
                    redelegations: redelegations__.unwrap_or_default(),
                    exported: exported__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HistoricalInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.header.is_some() {
            len += 1;
        }
        if !self.valset.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.HistoricalInfo", len)?;
        if let Some(v) = self.header.as_ref() {
            struct_ser.serialize_field("header", v)?;
        }
        if !self.valset.is_empty() {
            struct_ser.serialize_field("valset", &self.valset)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HistoricalInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "header",
            "valset",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Header,
            Valset,
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
                            "header" => Ok(GeneratedField::Header),
                            "valset" => Ok(GeneratedField::Valset),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HistoricalInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.HistoricalInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<HistoricalInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut header__ = None;
                let mut valset__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Header => {
                            if header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("header"));
                            }
                            header__ = map.next_value()?;
                        }
                        GeneratedField::Valset => {
                            if valset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valset"));
                            }
                            valset__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(HistoricalInfo {
                    header: header__,
                    valset: valset__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.HistoricalInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Infraction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "INFRACTION_UNSPECIFIED",
            Self::DoubleSign => "INFRACTION_DOUBLE_SIGN",
            Self::Downtime => "INFRACTION_DOWNTIME",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Infraction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "INFRACTION_UNSPECIFIED",
            "INFRACTION_DOUBLE_SIGN",
            "INFRACTION_DOWNTIME",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Infraction;

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
                    .and_then(Infraction::from_i32)
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
                    .and_then(Infraction::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "INFRACTION_UNSPECIFIED" => Ok(Infraction::Unspecified),
                    "INFRACTION_DOUBLE_SIGN" => Ok(Infraction::DoubleSign),
                    "INFRACTION_DOWNTIME" => Ok(Infraction::Downtime),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for LastValidatorPower {
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
        if self.power != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.LastValidatorPower", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if self.power != 0 {
            struct_ser.serialize_field("power", ToString::to_string(&self.power).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LastValidatorPower {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "power",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Power,
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
                            "power" => Ok(GeneratedField::Power),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LastValidatorPower;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.LastValidatorPower")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<LastValidatorPower, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut power__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Power => {
                            if power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("power"));
                            }
                            power__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(LastValidatorPower {
                    address: address__.unwrap_or_default(),
                    power: power__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.LastValidatorPower", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgBeginRedelegate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_src_address.is_empty() {
            len += 1;
        }
        if !self.validator_dst_address.is_empty() {
            len += 1;
        }
        if self.amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgBeginRedelegate", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_src_address.is_empty() {
            struct_ser.serialize_field("validatorSrcAddress", &self.validator_src_address)?;
        }
        if !self.validator_dst_address.is_empty() {
            struct_ser.serialize_field("validatorDstAddress", &self.validator_dst_address)?;
        }
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgBeginRedelegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_src_address",
            "validatorSrcAddress",
            "validator_dst_address",
            "validatorDstAddress",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorSrcAddress,
            ValidatorDstAddress,
            Amount,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorSrcAddress" | "validator_src_address" => Ok(GeneratedField::ValidatorSrcAddress),
                            "validatorDstAddress" | "validator_dst_address" => Ok(GeneratedField::ValidatorDstAddress),
                            "amount" => Ok(GeneratedField::Amount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgBeginRedelegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgBeginRedelegate")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgBeginRedelegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_src_address__ = None;
                let mut validator_dst_address__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorSrcAddress => {
                            if validator_src_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSrcAddress"));
                            }
                            validator_src_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorDstAddress => {
                            if validator_dst_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDstAddress"));
                            }
                            validator_dst_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgBeginRedelegate {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_src_address: validator_src_address__.unwrap_or_default(),
                    validator_dst_address: validator_dst_address__.unwrap_or_default(),
                    amount: amount__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgBeginRedelegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgBeginRedelegateResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.completion_time.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgBeginRedelegateResponse", len)?;
        if let Some(v) = self.completion_time.as_ref() {
            struct_ser.serialize_field("completionTime", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgBeginRedelegateResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "completion_time",
            "completionTime",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CompletionTime,
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
                            "completionTime" | "completion_time" => Ok(GeneratedField::CompletionTime),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgBeginRedelegateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgBeginRedelegateResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgBeginRedelegateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut completion_time__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CompletionTime => {
                            if completion_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("completionTime"));
                            }
                            completion_time__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgBeginRedelegateResponse {
                    completion_time: completion_time__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgBeginRedelegateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCancelUnbondingDelegation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.amount.is_some() {
            len += 1;
        }
        if self.creation_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgCancelUnbondingDelegation", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if self.creation_height != 0 {
            struct_ser.serialize_field("creationHeight", ToString::to_string(&self.creation_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCancelUnbondingDelegation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_address",
            "validatorAddress",
            "amount",
            "creation_height",
            "creationHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorAddress,
            Amount,
            CreationHeight,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "amount" => Ok(GeneratedField::Amount),
                            "creationHeight" | "creation_height" => Ok(GeneratedField::CreationHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCancelUnbondingDelegation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgCancelUnbondingDelegation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCancelUnbondingDelegation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_address__ = None;
                let mut amount__ = None;
                let mut creation_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                        GeneratedField::CreationHeight => {
                            if creation_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("creationHeight"));
                            }
                            creation_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgCancelUnbondingDelegation {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                    amount: amount__,
                    creation_height: creation_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgCancelUnbondingDelegation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCancelUnbondingDelegationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgCancelUnbondingDelegationResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCancelUnbondingDelegationResponse {
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
            type Value = MsgCancelUnbondingDelegationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgCancelUnbondingDelegationResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCancelUnbondingDelegationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgCancelUnbondingDelegationResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgCancelUnbondingDelegationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateValidator {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.description.is_some() {
            len += 1;
        }
        if self.commission.is_some() {
            len += 1;
        }
        if !self.min_self_delegation.is_empty() {
            len += 1;
        }
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.pubkey.is_some() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgCreateValidator", len)?;
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if let Some(v) = self.commission.as_ref() {
            struct_ser.serialize_field("commission", v)?;
        }
        if !self.min_self_delegation.is_empty() {
            struct_ser.serialize_field("minSelfDelegation", &self.min_self_delegation)?;
        }
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if let Some(v) = self.pubkey.as_ref() {
            struct_ser.serialize_field("pubkey", v)?;
        }
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateValidator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
            "commission",
            "min_self_delegation",
            "minSelfDelegation",
            "delegator_address",
            "delegatorAddress",
            "validator_address",
            "validatorAddress",
            "pubkey",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
            Commission,
            MinSelfDelegation,
            DelegatorAddress,
            ValidatorAddress,
            Pubkey,
            Value,
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
                            "description" => Ok(GeneratedField::Description),
                            "commission" => Ok(GeneratedField::Commission),
                            "minSelfDelegation" | "min_self_delegation" => Ok(GeneratedField::MinSelfDelegation),
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "pubkey" => Ok(GeneratedField::Pubkey),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreateValidator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgCreateValidator")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateValidator, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                let mut commission__ = None;
                let mut min_self_delegation__ = None;
                let mut delegator_address__ = None;
                let mut validator_address__ = None;
                let mut pubkey__ = None;
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map.next_value()?;
                        }
                        GeneratedField::Commission => {
                            if commission__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commission"));
                            }
                            commission__ = map.next_value()?;
                        }
                        GeneratedField::MinSelfDelegation => {
                            if min_self_delegation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minSelfDelegation"));
                            }
                            min_self_delegation__ = Some(map.next_value()?);
                        }
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pubkey => {
                            if pubkey__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubkey"));
                            }
                            pubkey__ = map.next_value()?;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgCreateValidator {
                    description: description__,
                    commission: commission__,
                    min_self_delegation: min_self_delegation__.unwrap_or_default(),
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                    pubkey: pubkey__,
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgCreateValidator", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateValidatorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgCreateValidatorResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateValidatorResponse {
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
            type Value = MsgCreateValidatorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgCreateValidatorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateValidatorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgCreateValidatorResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgCreateValidatorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgDelegate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgDelegate", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgDelegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_address",
            "validatorAddress",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorAddress,
            Amount,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "amount" => Ok(GeneratedField::Amount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgDelegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgDelegate")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgDelegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_address__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgDelegate {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                    amount: amount__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgDelegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgDelegateResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgDelegateResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgDelegateResponse {
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
            type Value = MsgDelegateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgDelegateResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgDelegateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgDelegateResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgDelegateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgEditValidator {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.description.is_some() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if !self.commission_rate.is_empty() {
            len += 1;
        }
        if !self.min_self_delegation.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgEditValidator", len)?;
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if !self.commission_rate.is_empty() {
            struct_ser.serialize_field("commissionRate", &self.commission_rate)?;
        }
        if !self.min_self_delegation.is_empty() {
            struct_ser.serialize_field("minSelfDelegation", &self.min_self_delegation)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgEditValidator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
            "validator_address",
            "validatorAddress",
            "commission_rate",
            "commissionRate",
            "min_self_delegation",
            "minSelfDelegation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
            ValidatorAddress,
            CommissionRate,
            MinSelfDelegation,
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
                            "description" => Ok(GeneratedField::Description),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "commissionRate" | "commission_rate" => Ok(GeneratedField::CommissionRate),
                            "minSelfDelegation" | "min_self_delegation" => Ok(GeneratedField::MinSelfDelegation),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgEditValidator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgEditValidator")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgEditValidator, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                let mut validator_address__ = None;
                let mut commission_rate__ = None;
                let mut min_self_delegation__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map.next_value()?;
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::CommissionRate => {
                            if commission_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commissionRate"));
                            }
                            commission_rate__ = Some(map.next_value()?);
                        }
                        GeneratedField::MinSelfDelegation => {
                            if min_self_delegation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minSelfDelegation"));
                            }
                            min_self_delegation__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgEditValidator {
                    description: description__,
                    validator_address: validator_address__.unwrap_or_default(),
                    commission_rate: commission_rate__.unwrap_or_default(),
                    min_self_delegation: min_self_delegation__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgEditValidator", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgEditValidatorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgEditValidatorResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgEditValidatorResponse {
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
            type Value = MsgEditValidatorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgEditValidatorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgEditValidatorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgEditValidatorResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgEditValidatorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUndelegate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgUndelegate", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUndelegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_address",
            "validatorAddress",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorAddress,
            Amount,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "amount" => Ok(GeneratedField::Amount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUndelegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgUndelegate")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUndelegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_address__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgUndelegate {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                    amount: amount__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgUndelegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUndelegateResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.completion_time.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgUndelegateResponse", len)?;
        if let Some(v) = self.completion_time.as_ref() {
            struct_ser.serialize_field("completionTime", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUndelegateResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "completion_time",
            "completionTime",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CompletionTime,
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
                            "completionTime" | "completion_time" => Ok(GeneratedField::CompletionTime),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUndelegateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.MsgUndelegateResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUndelegateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut completion_time__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CompletionTime => {
                            if completion_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("completionTime"));
                            }
                            completion_time__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgUndelegateResponse {
                    completion_time: completion_time__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgUndelegateResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgUpdateParams", len)?;
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
                formatter.write_str("struct cosmos.staking.v1beta1.MsgUpdateParams")
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
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgUpdateParams", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.MsgUpdateParamsResponse", len)?;
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
                formatter.write_str("struct cosmos.staking.v1beta1.MsgUpdateParamsResponse")
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
        deserializer.deserialize_struct("cosmos.staking.v1beta1.MsgUpdateParamsResponse", FIELDS, GeneratedVisitor)
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
        if self.unbonding_time.is_some() {
            len += 1;
        }
        if self.max_validators != 0 {
            len += 1;
        }
        if self.max_entries != 0 {
            len += 1;
        }
        if self.historical_entries != 0 {
            len += 1;
        }
        if !self.bond_denom.is_empty() {
            len += 1;
        }
        if !self.min_commission_rate.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.Params", len)?;
        if let Some(v) = self.unbonding_time.as_ref() {
            struct_ser.serialize_field("unbondingTime", v)?;
        }
        if self.max_validators != 0 {
            struct_ser.serialize_field("maxValidators", &self.max_validators)?;
        }
        if self.max_entries != 0 {
            struct_ser.serialize_field("maxEntries", &self.max_entries)?;
        }
        if self.historical_entries != 0 {
            struct_ser.serialize_field("historicalEntries", &self.historical_entries)?;
        }
        if !self.bond_denom.is_empty() {
            struct_ser.serialize_field("bondDenom", &self.bond_denom)?;
        }
        if !self.min_commission_rate.is_empty() {
            struct_ser.serialize_field("minCommissionRate", &self.min_commission_rate)?;
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
            "unbonding_time",
            "unbondingTime",
            "max_validators",
            "maxValidators",
            "max_entries",
            "maxEntries",
            "historical_entries",
            "historicalEntries",
            "bond_denom",
            "bondDenom",
            "min_commission_rate",
            "minCommissionRate",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UnbondingTime,
            MaxValidators,
            MaxEntries,
            HistoricalEntries,
            BondDenom,
            MinCommissionRate,
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
                            "unbondingTime" | "unbonding_time" => Ok(GeneratedField::UnbondingTime),
                            "maxValidators" | "max_validators" => Ok(GeneratedField::MaxValidators),
                            "maxEntries" | "max_entries" => Ok(GeneratedField::MaxEntries),
                            "historicalEntries" | "historical_entries" => Ok(GeneratedField::HistoricalEntries),
                            "bondDenom" | "bond_denom" => Ok(GeneratedField::BondDenom),
                            "minCommissionRate" | "min_commission_rate" => Ok(GeneratedField::MinCommissionRate),
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
                formatter.write_str("struct cosmos.staking.v1beta1.Params")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Params, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut unbonding_time__ = None;
                let mut max_validators__ = None;
                let mut max_entries__ = None;
                let mut historical_entries__ = None;
                let mut bond_denom__ = None;
                let mut min_commission_rate__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::UnbondingTime => {
                            if unbonding_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingTime"));
                            }
                            unbonding_time__ = map.next_value()?;
                        }
                        GeneratedField::MaxValidators => {
                            if max_validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxValidators"));
                            }
                            max_validators__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxEntries => {
                            if max_entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxEntries"));
                            }
                            max_entries__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::HistoricalEntries => {
                            if historical_entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("historicalEntries"));
                            }
                            historical_entries__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BondDenom => {
                            if bond_denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bondDenom"));
                            }
                            bond_denom__ = Some(map.next_value()?);
                        }
                        GeneratedField::MinCommissionRate => {
                            if min_commission_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minCommissionRate"));
                            }
                            min_commission_rate__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Params {
                    unbonding_time: unbonding_time__,
                    max_validators: max_validators__.unwrap_or_default(),
                    max_entries: max_entries__.unwrap_or_default(),
                    historical_entries: historical_entries__.unwrap_or_default(),
                    bond_denom: bond_denom__.unwrap_or_default(),
                    min_commission_rate: min_commission_rate__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.Params", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Pool {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.not_bonded_tokens.is_empty() {
            len += 1;
        }
        if !self.bonded_tokens.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.Pool", len)?;
        if !self.not_bonded_tokens.is_empty() {
            struct_ser.serialize_field("notBondedTokens", &self.not_bonded_tokens)?;
        }
        if !self.bonded_tokens.is_empty() {
            struct_ser.serialize_field("bondedTokens", &self.bonded_tokens)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Pool {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "not_bonded_tokens",
            "notBondedTokens",
            "bonded_tokens",
            "bondedTokens",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NotBondedTokens,
            BondedTokens,
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
                            "notBondedTokens" | "not_bonded_tokens" => Ok(GeneratedField::NotBondedTokens),
                            "bondedTokens" | "bonded_tokens" => Ok(GeneratedField::BondedTokens),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Pool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.Pool")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Pool, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut not_bonded_tokens__ = None;
                let mut bonded_tokens__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NotBondedTokens => {
                            if not_bonded_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("notBondedTokens"));
                            }
                            not_bonded_tokens__ = Some(map.next_value()?);
                        }
                        GeneratedField::BondedTokens => {
                            if bonded_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bondedTokens"));
                            }
                            bonded_tokens__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Pool {
                    not_bonded_tokens: not_bonded_tokens__.unwrap_or_default(),
                    bonded_tokens: bonded_tokens__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.Pool", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegationRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_addr.is_empty() {
            len += 1;
        }
        if !self.validator_addr.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegationRequest", len)?;
        if !self.delegator_addr.is_empty() {
            struct_ser.serialize_field("delegatorAddr", &self.delegator_addr)?;
        }
        if !self.validator_addr.is_empty() {
            struct_ser.serialize_field("validatorAddr", &self.validator_addr)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegationRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_addr",
            "delegatorAddr",
            "validator_addr",
            "validatorAddr",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddr,
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
                            "delegatorAddr" | "delegator_addr" => Ok(GeneratedField::DelegatorAddr),
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
            type Value = QueryDelegationRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegationRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegationRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_addr__ = None;
                let mut validator_addr__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddr => {
                            if delegator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddr"));
                            }
                            delegator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddr => {
                            if validator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddr"));
                            }
                            validator_addr__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegationRequest {
                    delegator_addr: delegator_addr__.unwrap_or_default(),
                    validator_addr: validator_addr__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegationRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delegation_response.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegationResponse", len)?;
        if let Some(v) = self.delegation_response.as_ref() {
            struct_ser.serialize_field("delegationResponse", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegation_response",
            "delegationResponse",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegationResponse,
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
                            "delegationResponse" | "delegation_response" => Ok(GeneratedField::DelegationResponse),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDelegationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegationResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegation_response__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegationResponse => {
                            if delegation_response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegationResponse"));
                            }
                            delegation_response__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDelegationResponse {
                    delegation_response: delegation_response__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorDelegationsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_addr.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegatorDelegationsRequest", len)?;
        if !self.delegator_addr.is_empty() {
            struct_ser.serialize_field("delegatorAddr", &self.delegator_addr)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorDelegationsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_addr",
            "delegatorAddr",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddr,
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
                            "delegatorAddr" | "delegator_addr" => Ok(GeneratedField::DelegatorAddr),
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
            type Value = QueryDelegatorDelegationsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegatorDelegationsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorDelegationsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_addr__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddr => {
                            if delegator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddr"));
                            }
                            delegator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDelegatorDelegationsRequest {
                    delegator_addr: delegator_addr__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegatorDelegationsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorDelegationsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegation_responses.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegatorDelegationsResponse", len)?;
        if !self.delegation_responses.is_empty() {
            struct_ser.serialize_field("delegationResponses", &self.delegation_responses)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorDelegationsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegation_responses",
            "delegationResponses",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegationResponses,
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
                            "delegationResponses" | "delegation_responses" => Ok(GeneratedField::DelegationResponses),
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
            type Value = QueryDelegatorDelegationsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegatorDelegationsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorDelegationsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegation_responses__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegationResponses => {
                            if delegation_responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegationResponses"));
                            }
                            delegation_responses__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDelegatorDelegationsResponse {
                    delegation_responses: delegation_responses__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegatorDelegationsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorUnbondingDelegationsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_addr.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegatorUnbondingDelegationsRequest", len)?;
        if !self.delegator_addr.is_empty() {
            struct_ser.serialize_field("delegatorAddr", &self.delegator_addr)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorUnbondingDelegationsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_addr",
            "delegatorAddr",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddr,
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
                            "delegatorAddr" | "delegator_addr" => Ok(GeneratedField::DelegatorAddr),
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
            type Value = QueryDelegatorUnbondingDelegationsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegatorUnbondingDelegationsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorUnbondingDelegationsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_addr__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddr => {
                            if delegator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddr"));
                            }
                            delegator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDelegatorUnbondingDelegationsRequest {
                    delegator_addr: delegator_addr__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegatorUnbondingDelegationsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorUnbondingDelegationsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.unbonding_responses.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegatorUnbondingDelegationsResponse", len)?;
        if !self.unbonding_responses.is_empty() {
            struct_ser.serialize_field("unbondingResponses", &self.unbonding_responses)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorUnbondingDelegationsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "unbonding_responses",
            "unbondingResponses",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UnbondingResponses,
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
                            "unbondingResponses" | "unbonding_responses" => Ok(GeneratedField::UnbondingResponses),
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
            type Value = QueryDelegatorUnbondingDelegationsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegatorUnbondingDelegationsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorUnbondingDelegationsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut unbonding_responses__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::UnbondingResponses => {
                            if unbonding_responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingResponses"));
                            }
                            unbonding_responses__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDelegatorUnbondingDelegationsResponse {
                    unbonding_responses: unbonding_responses__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegatorUnbondingDelegationsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorValidatorRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_addr.is_empty() {
            len += 1;
        }
        if !self.validator_addr.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegatorValidatorRequest", len)?;
        if !self.delegator_addr.is_empty() {
            struct_ser.serialize_field("delegatorAddr", &self.delegator_addr)?;
        }
        if !self.validator_addr.is_empty() {
            struct_ser.serialize_field("validatorAddr", &self.validator_addr)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorValidatorRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_addr",
            "delegatorAddr",
            "validator_addr",
            "validatorAddr",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddr,
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
                            "delegatorAddr" | "delegator_addr" => Ok(GeneratedField::DelegatorAddr),
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
            type Value = QueryDelegatorValidatorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegatorValidatorRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorValidatorRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_addr__ = None;
                let mut validator_addr__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddr => {
                            if delegator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddr"));
                            }
                            delegator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddr => {
                            if validator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddr"));
                            }
                            validator_addr__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegatorValidatorRequest {
                    delegator_addr: delegator_addr__.unwrap_or_default(),
                    validator_addr: validator_addr__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegatorValidatorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorValidatorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegatorValidatorResponse", len)?;
        if let Some(v) = self.validator.as_ref() {
            struct_ser.serialize_field("validator", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorValidatorResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Validator,
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
                            "validator" => Ok(GeneratedField::Validator),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDelegatorValidatorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegatorValidatorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorValidatorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Validator => {
                            if validator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validator"));
                            }
                            validator__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDelegatorValidatorResponse {
                    validator: validator__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegatorValidatorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorValidatorsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_addr.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegatorValidatorsRequest", len)?;
        if !self.delegator_addr.is_empty() {
            struct_ser.serialize_field("delegatorAddr", &self.delegator_addr)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorValidatorsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_addr",
            "delegatorAddr",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddr,
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
                            "delegatorAddr" | "delegator_addr" => Ok(GeneratedField::DelegatorAddr),
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
            type Value = QueryDelegatorValidatorsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegatorValidatorsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorValidatorsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_addr__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddr => {
                            if delegator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddr"));
                            }
                            delegator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDelegatorValidatorsRequest {
                    delegator_addr: delegator_addr__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegatorValidatorsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorValidatorsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validators.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryDelegatorValidatorsResponse", len)?;
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorValidatorsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validators",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Validators,
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
                            "validators" => Ok(GeneratedField::Validators),
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
            type Value = QueryDelegatorValidatorsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryDelegatorValidatorsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorValidatorsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validators__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDelegatorValidatorsResponse {
                    validators: validators__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryDelegatorValidatorsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryHistoricalInfoRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryHistoricalInfoRequest", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryHistoricalInfoRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = QueryHistoricalInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryHistoricalInfoRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryHistoricalInfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryHistoricalInfoRequest {
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryHistoricalInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryHistoricalInfoResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.hist.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryHistoricalInfoResponse", len)?;
        if let Some(v) = self.hist.as_ref() {
            struct_ser.serialize_field("hist", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryHistoricalInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "hist",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Hist,
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
                            "hist" => Ok(GeneratedField::Hist),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryHistoricalInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryHistoricalInfoResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryHistoricalInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut hist__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Hist => {
                            if hist__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hist"));
                            }
                            hist__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryHistoricalInfoResponse {
                    hist: hist__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryHistoricalInfoResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryParamsRequest", len)?;
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
                formatter.write_str("struct cosmos.staking.v1beta1.QueryParamsRequest")
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
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryParamsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryParamsResponse", len)?;
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
                formatter.write_str("struct cosmos.staking.v1beta1.QueryParamsResponse")
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
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryParamsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPoolRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryPoolRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPoolRequest {
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
            type Value = QueryPoolRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryPoolRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPoolRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(QueryPoolRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryPoolRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryPoolResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.pool.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryPoolResponse", len)?;
        if let Some(v) = self.pool.as_ref() {
            struct_ser.serialize_field("pool", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryPoolResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pool",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pool,
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
                            "pool" => Ok(GeneratedField::Pool),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryPoolResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryPoolResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryPoolResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pool__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Pool => {
                            if pool__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pool"));
                            }
                            pool__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryPoolResponse {
                    pool: pool__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryPoolResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryRedelegationsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_addr.is_empty() {
            len += 1;
        }
        if !self.src_validator_addr.is_empty() {
            len += 1;
        }
        if !self.dst_validator_addr.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryRedelegationsRequest", len)?;
        if !self.delegator_addr.is_empty() {
            struct_ser.serialize_field("delegatorAddr", &self.delegator_addr)?;
        }
        if !self.src_validator_addr.is_empty() {
            struct_ser.serialize_field("srcValidatorAddr", &self.src_validator_addr)?;
        }
        if !self.dst_validator_addr.is_empty() {
            struct_ser.serialize_field("dstValidatorAddr", &self.dst_validator_addr)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryRedelegationsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_addr",
            "delegatorAddr",
            "src_validator_addr",
            "srcValidatorAddr",
            "dst_validator_addr",
            "dstValidatorAddr",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddr,
            SrcValidatorAddr,
            DstValidatorAddr,
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
                            "delegatorAddr" | "delegator_addr" => Ok(GeneratedField::DelegatorAddr),
                            "srcValidatorAddr" | "src_validator_addr" => Ok(GeneratedField::SrcValidatorAddr),
                            "dstValidatorAddr" | "dst_validator_addr" => Ok(GeneratedField::DstValidatorAddr),
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
            type Value = QueryRedelegationsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryRedelegationsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryRedelegationsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_addr__ = None;
                let mut src_validator_addr__ = None;
                let mut dst_validator_addr__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddr => {
                            if delegator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddr"));
                            }
                            delegator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::SrcValidatorAddr => {
                            if src_validator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("srcValidatorAddr"));
                            }
                            src_validator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::DstValidatorAddr => {
                            if dst_validator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dstValidatorAddr"));
                            }
                            dst_validator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryRedelegationsRequest {
                    delegator_addr: delegator_addr__.unwrap_or_default(),
                    src_validator_addr: src_validator_addr__.unwrap_or_default(),
                    dst_validator_addr: dst_validator_addr__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryRedelegationsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryRedelegationsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.redelegation_responses.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryRedelegationsResponse", len)?;
        if !self.redelegation_responses.is_empty() {
            struct_ser.serialize_field("redelegationResponses", &self.redelegation_responses)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryRedelegationsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "redelegation_responses",
            "redelegationResponses",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RedelegationResponses,
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
                            "redelegationResponses" | "redelegation_responses" => Ok(GeneratedField::RedelegationResponses),
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
            type Value = QueryRedelegationsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryRedelegationsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryRedelegationsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut redelegation_responses__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RedelegationResponses => {
                            if redelegation_responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("redelegationResponses"));
                            }
                            redelegation_responses__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryRedelegationsResponse {
                    redelegation_responses: redelegation_responses__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryRedelegationsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryUnbondingDelegationRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_addr.is_empty() {
            len += 1;
        }
        if !self.validator_addr.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryUnbondingDelegationRequest", len)?;
        if !self.delegator_addr.is_empty() {
            struct_ser.serialize_field("delegatorAddr", &self.delegator_addr)?;
        }
        if !self.validator_addr.is_empty() {
            struct_ser.serialize_field("validatorAddr", &self.validator_addr)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryUnbondingDelegationRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_addr",
            "delegatorAddr",
            "validator_addr",
            "validatorAddr",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddr,
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
                            "delegatorAddr" | "delegator_addr" => Ok(GeneratedField::DelegatorAddr),
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
            type Value = QueryUnbondingDelegationRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryUnbondingDelegationRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryUnbondingDelegationRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_addr__ = None;
                let mut validator_addr__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddr => {
                            if delegator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddr"));
                            }
                            delegator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddr => {
                            if validator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddr"));
                            }
                            validator_addr__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryUnbondingDelegationRequest {
                    delegator_addr: delegator_addr__.unwrap_or_default(),
                    validator_addr: validator_addr__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryUnbondingDelegationRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryUnbondingDelegationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.unbond.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryUnbondingDelegationResponse", len)?;
        if let Some(v) = self.unbond.as_ref() {
            struct_ser.serialize_field("unbond", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryUnbondingDelegationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "unbond",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Unbond,
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
                            "unbond" => Ok(GeneratedField::Unbond),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryUnbondingDelegationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryUnbondingDelegationResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryUnbondingDelegationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut unbond__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Unbond => {
                            if unbond__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbond"));
                            }
                            unbond__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryUnbondingDelegationResponse {
                    unbond: unbond__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryUnbondingDelegationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorDelegationsRequest {
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
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryValidatorDelegationsRequest", len)?;
        if !self.validator_addr.is_empty() {
            struct_ser.serialize_field("validatorAddr", &self.validator_addr)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorDelegationsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_addr",
            "validatorAddr",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddr,
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
                            "validatorAddr" | "validator_addr" => Ok(GeneratedField::ValidatorAddr),
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
            type Value = QueryValidatorDelegationsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryValidatorDelegationsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorDelegationsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_addr__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddr => {
                            if validator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddr"));
                            }
                            validator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorDelegationsRequest {
                    validator_addr: validator_addr__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryValidatorDelegationsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorDelegationsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegation_responses.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryValidatorDelegationsResponse", len)?;
        if !self.delegation_responses.is_empty() {
            struct_ser.serialize_field("delegationResponses", &self.delegation_responses)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorDelegationsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegation_responses",
            "delegationResponses",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegationResponses,
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
                            "delegationResponses" | "delegation_responses" => Ok(GeneratedField::DelegationResponses),
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
            type Value = QueryValidatorDelegationsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryValidatorDelegationsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorDelegationsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegation_responses__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegationResponses => {
                            if delegation_responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegationResponses"));
                            }
                            delegation_responses__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorDelegationsResponse {
                    delegation_responses: delegation_responses__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryValidatorDelegationsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorRequest {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryValidatorRequest", len)?;
        if !self.validator_addr.is_empty() {
            struct_ser.serialize_field("validatorAddr", &self.validator_addr)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorRequest {
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
            type Value = QueryValidatorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryValidatorRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorRequest, V::Error>
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
                Ok(QueryValidatorRequest {
                    validator_addr: validator_addr__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryValidatorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryValidatorResponse", len)?;
        if let Some(v) = self.validator.as_ref() {
            struct_ser.serialize_field("validator", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Validator,
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
                            "validator" => Ok(GeneratedField::Validator),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryValidatorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryValidatorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Validator => {
                            if validator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validator"));
                            }
                            validator__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorResponse {
                    validator: validator__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryValidatorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorUnbondingDelegationsRequest {
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
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryValidatorUnbondingDelegationsRequest", len)?;
        if !self.validator_addr.is_empty() {
            struct_ser.serialize_field("validatorAddr", &self.validator_addr)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorUnbondingDelegationsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_addr",
            "validatorAddr",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddr,
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
                            "validatorAddr" | "validator_addr" => Ok(GeneratedField::ValidatorAddr),
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
            type Value = QueryValidatorUnbondingDelegationsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryValidatorUnbondingDelegationsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorUnbondingDelegationsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_addr__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddr => {
                            if validator_addr__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddr"));
                            }
                            validator_addr__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorUnbondingDelegationsRequest {
                    validator_addr: validator_addr__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryValidatorUnbondingDelegationsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorUnbondingDelegationsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.unbonding_responses.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryValidatorUnbondingDelegationsResponse", len)?;
        if !self.unbonding_responses.is_empty() {
            struct_ser.serialize_field("unbondingResponses", &self.unbonding_responses)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorUnbondingDelegationsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "unbonding_responses",
            "unbondingResponses",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UnbondingResponses,
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
                            "unbondingResponses" | "unbonding_responses" => Ok(GeneratedField::UnbondingResponses),
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
            type Value = QueryValidatorUnbondingDelegationsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryValidatorUnbondingDelegationsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorUnbondingDelegationsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut unbonding_responses__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::UnbondingResponses => {
                            if unbonding_responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingResponses"));
                            }
                            unbonding_responses__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorUnbondingDelegationsResponse {
                    unbonding_responses: unbonding_responses__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryValidatorUnbondingDelegationsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.status.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryValidatorsRequest", len)?;
        if !self.status.is_empty() {
            struct_ser.serialize_field("status", &self.status)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
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
                            "status" => Ok(GeneratedField::Status),
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
            type Value = QueryValidatorsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryValidatorsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorsRequest {
                    status: status__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryValidatorsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validators.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.QueryValidatorsResponse", len)?;
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validators",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Validators,
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
                            "validators" => Ok(GeneratedField::Validators),
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
            type Value = QueryValidatorsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.QueryValidatorsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validators__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorsResponse {
                    validators: validators__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.QueryValidatorsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Redelegation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_src_address.is_empty() {
            len += 1;
        }
        if !self.validator_dst_address.is_empty() {
            len += 1;
        }
        if !self.entries.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.Redelegation", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_src_address.is_empty() {
            struct_ser.serialize_field("validatorSrcAddress", &self.validator_src_address)?;
        }
        if !self.validator_dst_address.is_empty() {
            struct_ser.serialize_field("validatorDstAddress", &self.validator_dst_address)?;
        }
        if !self.entries.is_empty() {
            struct_ser.serialize_field("entries", &self.entries)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Redelegation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_src_address",
            "validatorSrcAddress",
            "validator_dst_address",
            "validatorDstAddress",
            "entries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorSrcAddress,
            ValidatorDstAddress,
            Entries,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorSrcAddress" | "validator_src_address" => Ok(GeneratedField::ValidatorSrcAddress),
                            "validatorDstAddress" | "validator_dst_address" => Ok(GeneratedField::ValidatorDstAddress),
                            "entries" => Ok(GeneratedField::Entries),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Redelegation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.Redelegation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Redelegation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_src_address__ = None;
                let mut validator_dst_address__ = None;
                let mut entries__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorSrcAddress => {
                            if validator_src_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSrcAddress"));
                            }
                            validator_src_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorDstAddress => {
                            if validator_dst_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDstAddress"));
                            }
                            validator_dst_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Entries => {
                            if entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Redelegation {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_src_address: validator_src_address__.unwrap_or_default(),
                    validator_dst_address: validator_dst_address__.unwrap_or_default(),
                    entries: entries__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.Redelegation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RedelegationEntry {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.creation_height != 0 {
            len += 1;
        }
        if self.completion_time.is_some() {
            len += 1;
        }
        if !self.initial_balance.is_empty() {
            len += 1;
        }
        if !self.shares_dst.is_empty() {
            len += 1;
        }
        if self.unbonding_id != 0 {
            len += 1;
        }
        if self.unbonding_on_hold_ref_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.RedelegationEntry", len)?;
        if self.creation_height != 0 {
            struct_ser.serialize_field("creationHeight", ToString::to_string(&self.creation_height).as_str())?;
        }
        if let Some(v) = self.completion_time.as_ref() {
            struct_ser.serialize_field("completionTime", v)?;
        }
        if !self.initial_balance.is_empty() {
            struct_ser.serialize_field("initialBalance", &self.initial_balance)?;
        }
        if !self.shares_dst.is_empty() {
            struct_ser.serialize_field("sharesDst", &self.shares_dst)?;
        }
        if self.unbonding_id != 0 {
            struct_ser.serialize_field("unbondingId", ToString::to_string(&self.unbonding_id).as_str())?;
        }
        if self.unbonding_on_hold_ref_count != 0 {
            struct_ser.serialize_field("unbondingOnHoldRefCount", ToString::to_string(&self.unbonding_on_hold_ref_count).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RedelegationEntry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "creation_height",
            "creationHeight",
            "completion_time",
            "completionTime",
            "initial_balance",
            "initialBalance",
            "shares_dst",
            "sharesDst",
            "unbonding_id",
            "unbondingId",
            "unbonding_on_hold_ref_count",
            "unbondingOnHoldRefCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CreationHeight,
            CompletionTime,
            InitialBalance,
            SharesDst,
            UnbondingId,
            UnbondingOnHoldRefCount,
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
                            "creationHeight" | "creation_height" => Ok(GeneratedField::CreationHeight),
                            "completionTime" | "completion_time" => Ok(GeneratedField::CompletionTime),
                            "initialBalance" | "initial_balance" => Ok(GeneratedField::InitialBalance),
                            "sharesDst" | "shares_dst" => Ok(GeneratedField::SharesDst),
                            "unbondingId" | "unbonding_id" => Ok(GeneratedField::UnbondingId),
                            "unbondingOnHoldRefCount" | "unbonding_on_hold_ref_count" => Ok(GeneratedField::UnbondingOnHoldRefCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RedelegationEntry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.RedelegationEntry")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RedelegationEntry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut creation_height__ = None;
                let mut completion_time__ = None;
                let mut initial_balance__ = None;
                let mut shares_dst__ = None;
                let mut unbonding_id__ = None;
                let mut unbonding_on_hold_ref_count__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CreationHeight => {
                            if creation_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("creationHeight"));
                            }
                            creation_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CompletionTime => {
                            if completion_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("completionTime"));
                            }
                            completion_time__ = map.next_value()?;
                        }
                        GeneratedField::InitialBalance => {
                            if initial_balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialBalance"));
                            }
                            initial_balance__ = Some(map.next_value()?);
                        }
                        GeneratedField::SharesDst => {
                            if shares_dst__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sharesDst"));
                            }
                            shares_dst__ = Some(map.next_value()?);
                        }
                        GeneratedField::UnbondingId => {
                            if unbonding_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingId"));
                            }
                            unbonding_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UnbondingOnHoldRefCount => {
                            if unbonding_on_hold_ref_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingOnHoldRefCount"));
                            }
                            unbonding_on_hold_ref_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(RedelegationEntry {
                    creation_height: creation_height__.unwrap_or_default(),
                    completion_time: completion_time__,
                    initial_balance: initial_balance__.unwrap_or_default(),
                    shares_dst: shares_dst__.unwrap_or_default(),
                    unbonding_id: unbonding_id__.unwrap_or_default(),
                    unbonding_on_hold_ref_count: unbonding_on_hold_ref_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.RedelegationEntry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RedelegationEntryResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.redelegation_entry.is_some() {
            len += 1;
        }
        if !self.balance.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.RedelegationEntryResponse", len)?;
        if let Some(v) = self.redelegation_entry.as_ref() {
            struct_ser.serialize_field("redelegationEntry", v)?;
        }
        if !self.balance.is_empty() {
            struct_ser.serialize_field("balance", &self.balance)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RedelegationEntryResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "redelegation_entry",
            "redelegationEntry",
            "balance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RedelegationEntry,
            Balance,
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
                            "redelegationEntry" | "redelegation_entry" => Ok(GeneratedField::RedelegationEntry),
                            "balance" => Ok(GeneratedField::Balance),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RedelegationEntryResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.RedelegationEntryResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RedelegationEntryResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut redelegation_entry__ = None;
                let mut balance__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RedelegationEntry => {
                            if redelegation_entry__.is_some() {
                                return Err(serde::de::Error::duplicate_field("redelegationEntry"));
                            }
                            redelegation_entry__ = map.next_value()?;
                        }
                        GeneratedField::Balance => {
                            if balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balance"));
                            }
                            balance__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RedelegationEntryResponse {
                    redelegation_entry: redelegation_entry__,
                    balance: balance__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.RedelegationEntryResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RedelegationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.redelegation.is_some() {
            len += 1;
        }
        if !self.entries.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.RedelegationResponse", len)?;
        if let Some(v) = self.redelegation.as_ref() {
            struct_ser.serialize_field("redelegation", v)?;
        }
        if !self.entries.is_empty() {
            struct_ser.serialize_field("entries", &self.entries)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RedelegationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "redelegation",
            "entries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Redelegation,
            Entries,
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
                            "redelegation" => Ok(GeneratedField::Redelegation),
                            "entries" => Ok(GeneratedField::Entries),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RedelegationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.RedelegationResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RedelegationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut redelegation__ = None;
                let mut entries__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Redelegation => {
                            if redelegation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("redelegation"));
                            }
                            redelegation__ = map.next_value()?;
                        }
                        GeneratedField::Entries => {
                            if entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RedelegationResponse {
                    redelegation: redelegation__,
                    entries: entries__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.RedelegationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StakeAuthorization {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.max_tokens.is_some() {
            len += 1;
        }
        if self.authorization_type != 0 {
            len += 1;
        }
        if self.validators.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.StakeAuthorization", len)?;
        if let Some(v) = self.max_tokens.as_ref() {
            struct_ser.serialize_field("maxTokens", v)?;
        }
        if self.authorization_type != 0 {
            let v = AuthorizationType::from_i32(self.authorization_type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.authorization_type)))?;
            struct_ser.serialize_field("authorizationType", &v)?;
        }
        if let Some(v) = self.validators.as_ref() {
            match v {
                stake_authorization::Validators::AllowList(v) => {
                    struct_ser.serialize_field("allowList", v)?;
                }
                stake_authorization::Validators::DenyList(v) => {
                    struct_ser.serialize_field("denyList", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StakeAuthorization {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "max_tokens",
            "maxTokens",
            "authorization_type",
            "authorizationType",
            "allow_list",
            "allowList",
            "deny_list",
            "denyList",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxTokens,
            AuthorizationType,
            AllowList,
            DenyList,
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
                            "maxTokens" | "max_tokens" => Ok(GeneratedField::MaxTokens),
                            "authorizationType" | "authorization_type" => Ok(GeneratedField::AuthorizationType),
                            "allowList" | "allow_list" => Ok(GeneratedField::AllowList),
                            "denyList" | "deny_list" => Ok(GeneratedField::DenyList),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StakeAuthorization;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.StakeAuthorization")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StakeAuthorization, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_tokens__ = None;
                let mut authorization_type__ = None;
                let mut validators__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MaxTokens => {
                            if max_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxTokens"));
                            }
                            max_tokens__ = map.next_value()?;
                        }
                        GeneratedField::AuthorizationType => {
                            if authorization_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorizationType"));
                            }
                            authorization_type__ = Some(map.next_value::<AuthorizationType>()? as i32);
                        }
                        GeneratedField::AllowList => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowList"));
                            }
                            validators__ = map.next_value::<::std::option::Option<_>>()?.map(stake_authorization::Validators::AllowList)
;
                        }
                        GeneratedField::DenyList => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denyList"));
                            }
                            validators__ = map.next_value::<::std::option::Option<_>>()?.map(stake_authorization::Validators::DenyList)
;
                        }
                    }
                }
                Ok(StakeAuthorization {
                    max_tokens: max_tokens__,
                    authorization_type: authorization_type__.unwrap_or_default(),
                    validators: validators__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.StakeAuthorization", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for stake_authorization::Validators {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.StakeAuthorization.Validators", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for stake_authorization::Validators {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = stake_authorization::Validators;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.StakeAuthorization.Validators")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<stake_authorization::Validators, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(stake_authorization::Validators {
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.StakeAuthorization.Validators", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UnbondingDelegation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if !self.entries.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.UnbondingDelegation", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if !self.entries.is_empty() {
            struct_ser.serialize_field("entries", &self.entries)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UnbondingDelegation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "validator_address",
            "validatorAddress",
            "entries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorAddress,
            Entries,
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
                            "delegatorAddress" | "delegator_address" => Ok(GeneratedField::DelegatorAddress),
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "entries" => Ok(GeneratedField::Entries),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UnbondingDelegation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.UnbondingDelegation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<UnbondingDelegation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_address__ = None;
                let mut entries__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Entries => {
                            if entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(UnbondingDelegation {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                    entries: entries__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.UnbondingDelegation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UnbondingDelegationEntry {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.creation_height != 0 {
            len += 1;
        }
        if self.completion_time.is_some() {
            len += 1;
        }
        if !self.initial_balance.is_empty() {
            len += 1;
        }
        if !self.balance.is_empty() {
            len += 1;
        }
        if self.unbonding_id != 0 {
            len += 1;
        }
        if self.unbonding_on_hold_ref_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.UnbondingDelegationEntry", len)?;
        if self.creation_height != 0 {
            struct_ser.serialize_field("creationHeight", ToString::to_string(&self.creation_height).as_str())?;
        }
        if let Some(v) = self.completion_time.as_ref() {
            struct_ser.serialize_field("completionTime", v)?;
        }
        if !self.initial_balance.is_empty() {
            struct_ser.serialize_field("initialBalance", &self.initial_balance)?;
        }
        if !self.balance.is_empty() {
            struct_ser.serialize_field("balance", &self.balance)?;
        }
        if self.unbonding_id != 0 {
            struct_ser.serialize_field("unbondingId", ToString::to_string(&self.unbonding_id).as_str())?;
        }
        if self.unbonding_on_hold_ref_count != 0 {
            struct_ser.serialize_field("unbondingOnHoldRefCount", ToString::to_string(&self.unbonding_on_hold_ref_count).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UnbondingDelegationEntry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "creation_height",
            "creationHeight",
            "completion_time",
            "completionTime",
            "initial_balance",
            "initialBalance",
            "balance",
            "unbonding_id",
            "unbondingId",
            "unbonding_on_hold_ref_count",
            "unbondingOnHoldRefCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CreationHeight,
            CompletionTime,
            InitialBalance,
            Balance,
            UnbondingId,
            UnbondingOnHoldRefCount,
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
                            "creationHeight" | "creation_height" => Ok(GeneratedField::CreationHeight),
                            "completionTime" | "completion_time" => Ok(GeneratedField::CompletionTime),
                            "initialBalance" | "initial_balance" => Ok(GeneratedField::InitialBalance),
                            "balance" => Ok(GeneratedField::Balance),
                            "unbondingId" | "unbonding_id" => Ok(GeneratedField::UnbondingId),
                            "unbondingOnHoldRefCount" | "unbonding_on_hold_ref_count" => Ok(GeneratedField::UnbondingOnHoldRefCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UnbondingDelegationEntry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.UnbondingDelegationEntry")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<UnbondingDelegationEntry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut creation_height__ = None;
                let mut completion_time__ = None;
                let mut initial_balance__ = None;
                let mut balance__ = None;
                let mut unbonding_id__ = None;
                let mut unbonding_on_hold_ref_count__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CreationHeight => {
                            if creation_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("creationHeight"));
                            }
                            creation_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CompletionTime => {
                            if completion_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("completionTime"));
                            }
                            completion_time__ = map.next_value()?;
                        }
                        GeneratedField::InitialBalance => {
                            if initial_balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialBalance"));
                            }
                            initial_balance__ = Some(map.next_value()?);
                        }
                        GeneratedField::Balance => {
                            if balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balance"));
                            }
                            balance__ = Some(map.next_value()?);
                        }
                        GeneratedField::UnbondingId => {
                            if unbonding_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingId"));
                            }
                            unbonding_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UnbondingOnHoldRefCount => {
                            if unbonding_on_hold_ref_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingOnHoldRefCount"));
                            }
                            unbonding_on_hold_ref_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(UnbondingDelegationEntry {
                    creation_height: creation_height__.unwrap_or_default(),
                    completion_time: completion_time__,
                    initial_balance: initial_balance__.unwrap_or_default(),
                    balance: balance__.unwrap_or_default(),
                    unbonding_id: unbonding_id__.unwrap_or_default(),
                    unbonding_on_hold_ref_count: unbonding_on_hold_ref_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.UnbondingDelegationEntry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValAddresses {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.addresses.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.ValAddresses", len)?;
        if !self.addresses.is_empty() {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValAddresses {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "addresses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Addresses,
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
                            "addresses" => Ok(GeneratedField::Addresses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValAddresses;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.ValAddresses")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValAddresses, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut addresses__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValAddresses {
                    addresses: addresses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.ValAddresses", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Validator {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.operator_address.is_empty() {
            len += 1;
        }
        if self.consensus_pubkey.is_some() {
            len += 1;
        }
        if self.jailed {
            len += 1;
        }
        if self.status != 0 {
            len += 1;
        }
        if !self.tokens.is_empty() {
            len += 1;
        }
        if !self.delegator_shares.is_empty() {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if self.unbonding_height != 0 {
            len += 1;
        }
        if self.unbonding_time.is_some() {
            len += 1;
        }
        if self.commission.is_some() {
            len += 1;
        }
        if !self.min_self_delegation.is_empty() {
            len += 1;
        }
        if self.unbonding_on_hold_ref_count != 0 {
            len += 1;
        }
        if !self.unbonding_ids.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.Validator", len)?;
        if !self.operator_address.is_empty() {
            struct_ser.serialize_field("operatorAddress", &self.operator_address)?;
        }
        if let Some(v) = self.consensus_pubkey.as_ref() {
            struct_ser.serialize_field("consensusPubkey", v)?;
        }
        if self.jailed {
            struct_ser.serialize_field("jailed", &self.jailed)?;
        }
        if self.status != 0 {
            let v = BondStatus::from_i32(self.status)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.status)))?;
            struct_ser.serialize_field("status", &v)?;
        }
        if !self.tokens.is_empty() {
            struct_ser.serialize_field("tokens", &self.tokens)?;
        }
        if !self.delegator_shares.is_empty() {
            struct_ser.serialize_field("delegatorShares", &self.delegator_shares)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if self.unbonding_height != 0 {
            struct_ser.serialize_field("unbondingHeight", ToString::to_string(&self.unbonding_height).as_str())?;
        }
        if let Some(v) = self.unbonding_time.as_ref() {
            struct_ser.serialize_field("unbondingTime", v)?;
        }
        if let Some(v) = self.commission.as_ref() {
            struct_ser.serialize_field("commission", v)?;
        }
        if !self.min_self_delegation.is_empty() {
            struct_ser.serialize_field("minSelfDelegation", &self.min_self_delegation)?;
        }
        if self.unbonding_on_hold_ref_count != 0 {
            struct_ser.serialize_field("unbondingOnHoldRefCount", ToString::to_string(&self.unbonding_on_hold_ref_count).as_str())?;
        }
        if !self.unbonding_ids.is_empty() {
            struct_ser.serialize_field("unbondingIds", &self.unbonding_ids.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Validator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "operator_address",
            "operatorAddress",
            "consensus_pubkey",
            "consensusPubkey",
            "jailed",
            "status",
            "tokens",
            "delegator_shares",
            "delegatorShares",
            "description",
            "unbonding_height",
            "unbondingHeight",
            "unbonding_time",
            "unbondingTime",
            "commission",
            "min_self_delegation",
            "minSelfDelegation",
            "unbonding_on_hold_ref_count",
            "unbondingOnHoldRefCount",
            "unbonding_ids",
            "unbondingIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OperatorAddress,
            ConsensusPubkey,
            Jailed,
            Status,
            Tokens,
            DelegatorShares,
            Description,
            UnbondingHeight,
            UnbondingTime,
            Commission,
            MinSelfDelegation,
            UnbondingOnHoldRefCount,
            UnbondingIds,
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
                            "operatorAddress" | "operator_address" => Ok(GeneratedField::OperatorAddress),
                            "consensusPubkey" | "consensus_pubkey" => Ok(GeneratedField::ConsensusPubkey),
                            "jailed" => Ok(GeneratedField::Jailed),
                            "status" => Ok(GeneratedField::Status),
                            "tokens" => Ok(GeneratedField::Tokens),
                            "delegatorShares" | "delegator_shares" => Ok(GeneratedField::DelegatorShares),
                            "description" => Ok(GeneratedField::Description),
                            "unbondingHeight" | "unbonding_height" => Ok(GeneratedField::UnbondingHeight),
                            "unbondingTime" | "unbonding_time" => Ok(GeneratedField::UnbondingTime),
                            "commission" => Ok(GeneratedField::Commission),
                            "minSelfDelegation" | "min_self_delegation" => Ok(GeneratedField::MinSelfDelegation),
                            "unbondingOnHoldRefCount" | "unbonding_on_hold_ref_count" => Ok(GeneratedField::UnbondingOnHoldRefCount),
                            "unbondingIds" | "unbonding_ids" => Ok(GeneratedField::UnbondingIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Validator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.Validator")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Validator, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut operator_address__ = None;
                let mut consensus_pubkey__ = None;
                let mut jailed__ = None;
                let mut status__ = None;
                let mut tokens__ = None;
                let mut delegator_shares__ = None;
                let mut description__ = None;
                let mut unbonding_height__ = None;
                let mut unbonding_time__ = None;
                let mut commission__ = None;
                let mut min_self_delegation__ = None;
                let mut unbonding_on_hold_ref_count__ = None;
                let mut unbonding_ids__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::OperatorAddress => {
                            if operator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operatorAddress"));
                            }
                            operator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ConsensusPubkey => {
                            if consensus_pubkey__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusPubkey"));
                            }
                            consensus_pubkey__ = map.next_value()?;
                        }
                        GeneratedField::Jailed => {
                            if jailed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("jailed"));
                            }
                            jailed__ = Some(map.next_value()?);
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map.next_value::<BondStatus>()? as i32);
                        }
                        GeneratedField::Tokens => {
                            if tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tokens"));
                            }
                            tokens__ = Some(map.next_value()?);
                        }
                        GeneratedField::DelegatorShares => {
                            if delegator_shares__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorShares"));
                            }
                            delegator_shares__ = Some(map.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map.next_value()?;
                        }
                        GeneratedField::UnbondingHeight => {
                            if unbonding_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingHeight"));
                            }
                            unbonding_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UnbondingTime => {
                            if unbonding_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingTime"));
                            }
                            unbonding_time__ = map.next_value()?;
                        }
                        GeneratedField::Commission => {
                            if commission__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commission"));
                            }
                            commission__ = map.next_value()?;
                        }
                        GeneratedField::MinSelfDelegation => {
                            if min_self_delegation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minSelfDelegation"));
                            }
                            min_self_delegation__ = Some(map.next_value()?);
                        }
                        GeneratedField::UnbondingOnHoldRefCount => {
                            if unbonding_on_hold_ref_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingOnHoldRefCount"));
                            }
                            unbonding_on_hold_ref_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UnbondingIds => {
                            if unbonding_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingIds"));
                            }
                            unbonding_ids__ = 
                                Some(map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(Validator {
                    operator_address: operator_address__.unwrap_or_default(),
                    consensus_pubkey: consensus_pubkey__,
                    jailed: jailed__.unwrap_or_default(),
                    status: status__.unwrap_or_default(),
                    tokens: tokens__.unwrap_or_default(),
                    delegator_shares: delegator_shares__.unwrap_or_default(),
                    description: description__,
                    unbonding_height: unbonding_height__.unwrap_or_default(),
                    unbonding_time: unbonding_time__,
                    commission: commission__,
                    min_self_delegation: min_self_delegation__.unwrap_or_default(),
                    unbonding_on_hold_ref_count: unbonding_on_hold_ref_count__.unwrap_or_default(),
                    unbonding_ids: unbonding_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.Validator", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorUpdates {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.updates.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.staking.v1beta1.ValidatorUpdates", len)?;
        if !self.updates.is_empty() {
            struct_ser.serialize_field("updates", &self.updates)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorUpdates {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "updates",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Updates,
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
                            "updates" => Ok(GeneratedField::Updates),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorUpdates;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.staking.v1beta1.ValidatorUpdates")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorUpdates, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut updates__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Updates => {
                            if updates__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updates"));
                            }
                            updates__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValidatorUpdates {
                    updates: updates__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.staking.v1beta1.ValidatorUpdates", FIELDS, GeneratedVisitor)
    }
}
