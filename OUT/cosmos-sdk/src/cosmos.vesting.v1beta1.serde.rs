// @generated
impl serde::Serialize for BaseVestingAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_account.is_some() {
            len += 1;
        }
        if !self.original_vesting.is_empty() {
            len += 1;
        }
        if !self.delegated_free.is_empty() {
            len += 1;
        }
        if !self.delegated_vesting.is_empty() {
            len += 1;
        }
        if self.end_time != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.BaseVestingAccount", len)?;
        if let Some(v) = self.base_account.as_ref() {
            struct_ser.serialize_field("baseAccount", v)?;
        }
        if !self.original_vesting.is_empty() {
            struct_ser.serialize_field("originalVesting", &self.original_vesting)?;
        }
        if !self.delegated_free.is_empty() {
            struct_ser.serialize_field("delegatedFree", &self.delegated_free)?;
        }
        if !self.delegated_vesting.is_empty() {
            struct_ser.serialize_field("delegatedVesting", &self.delegated_vesting)?;
        }
        if self.end_time != 0 {
            struct_ser.serialize_field("endTime", ToString::to_string(&self.end_time).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BaseVestingAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_account",
            "baseAccount",
            "original_vesting",
            "originalVesting",
            "delegated_free",
            "delegatedFree",
            "delegated_vesting",
            "delegatedVesting",
            "end_time",
            "endTime",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseAccount,
            OriginalVesting,
            DelegatedFree,
            DelegatedVesting,
            EndTime,
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
                            "baseAccount" | "base_account" => Ok(GeneratedField::BaseAccount),
                            "originalVesting" | "original_vesting" => Ok(GeneratedField::OriginalVesting),
                            "delegatedFree" | "delegated_free" => Ok(GeneratedField::DelegatedFree),
                            "delegatedVesting" | "delegated_vesting" => Ok(GeneratedField::DelegatedVesting),
                            "endTime" | "end_time" => Ok(GeneratedField::EndTime),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BaseVestingAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.BaseVestingAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BaseVestingAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_account__ = None;
                let mut original_vesting__ = None;
                let mut delegated_free__ = None;
                let mut delegated_vesting__ = None;
                let mut end_time__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BaseAccount => {
                            if base_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseAccount"));
                            }
                            base_account__ = map.next_value()?;
                        }
                        GeneratedField::OriginalVesting => {
                            if original_vesting__.is_some() {
                                return Err(serde::de::Error::duplicate_field("originalVesting"));
                            }
                            original_vesting__ = Some(map.next_value()?);
                        }
                        GeneratedField::DelegatedFree => {
                            if delegated_free__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatedFree"));
                            }
                            delegated_free__ = Some(map.next_value()?);
                        }
                        GeneratedField::DelegatedVesting => {
                            if delegated_vesting__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatedVesting"));
                            }
                            delegated_vesting__ = Some(map.next_value()?);
                        }
                        GeneratedField::EndTime => {
                            if end_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endTime"));
                            }
                            end_time__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(BaseVestingAccount {
                    base_account: base_account__,
                    original_vesting: original_vesting__.unwrap_or_default(),
                    delegated_free: delegated_free__.unwrap_or_default(),
                    delegated_vesting: delegated_vesting__.unwrap_or_default(),
                    end_time: end_time__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.BaseVestingAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ContinuousVestingAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_vesting_account.is_some() {
            len += 1;
        }
        if self.start_time != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.ContinuousVestingAccount", len)?;
        if let Some(v) = self.base_vesting_account.as_ref() {
            struct_ser.serialize_field("baseVestingAccount", v)?;
        }
        if self.start_time != 0 {
            struct_ser.serialize_field("startTime", ToString::to_string(&self.start_time).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ContinuousVestingAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_vesting_account",
            "baseVestingAccount",
            "start_time",
            "startTime",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseVestingAccount,
            StartTime,
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
                            "baseVestingAccount" | "base_vesting_account" => Ok(GeneratedField::BaseVestingAccount),
                            "startTime" | "start_time" => Ok(GeneratedField::StartTime),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ContinuousVestingAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.ContinuousVestingAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ContinuousVestingAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_vesting_account__ = None;
                let mut start_time__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BaseVestingAccount => {
                            if base_vesting_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseVestingAccount"));
                            }
                            base_vesting_account__ = map.next_value()?;
                        }
                        GeneratedField::StartTime => {
                            if start_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTime"));
                            }
                            start_time__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ContinuousVestingAccount {
                    base_vesting_account: base_vesting_account__,
                    start_time: start_time__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.ContinuousVestingAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelayedVestingAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_vesting_account.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.DelayedVestingAccount", len)?;
        if let Some(v) = self.base_vesting_account.as_ref() {
            struct_ser.serialize_field("baseVestingAccount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelayedVestingAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_vesting_account",
            "baseVestingAccount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseVestingAccount,
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
                            "baseVestingAccount" | "base_vesting_account" => Ok(GeneratedField::BaseVestingAccount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelayedVestingAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.DelayedVestingAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelayedVestingAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_vesting_account__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BaseVestingAccount => {
                            if base_vesting_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseVestingAccount"));
                            }
                            base_vesting_account__ = map.next_value()?;
                        }
                    }
                }
                Ok(DelayedVestingAccount {
                    base_vesting_account: base_vesting_account__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.DelayedVestingAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreatePeriodicVestingAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.from_address.is_empty() {
            len += 1;
        }
        if !self.to_address.is_empty() {
            len += 1;
        }
        if self.start_time != 0 {
            len += 1;
        }
        if !self.vesting_periods.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.MsgCreatePeriodicVestingAccount", len)?;
        if !self.from_address.is_empty() {
            struct_ser.serialize_field("fromAddress", &self.from_address)?;
        }
        if !self.to_address.is_empty() {
            struct_ser.serialize_field("toAddress", &self.to_address)?;
        }
        if self.start_time != 0 {
            struct_ser.serialize_field("startTime", ToString::to_string(&self.start_time).as_str())?;
        }
        if !self.vesting_periods.is_empty() {
            struct_ser.serialize_field("vestingPeriods", &self.vesting_periods)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreatePeriodicVestingAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "from_address",
            "fromAddress",
            "to_address",
            "toAddress",
            "start_time",
            "startTime",
            "vesting_periods",
            "vestingPeriods",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FromAddress,
            ToAddress,
            StartTime,
            VestingPeriods,
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
                            "fromAddress" | "from_address" => Ok(GeneratedField::FromAddress),
                            "toAddress" | "to_address" => Ok(GeneratedField::ToAddress),
                            "startTime" | "start_time" => Ok(GeneratedField::StartTime),
                            "vestingPeriods" | "vesting_periods" => Ok(GeneratedField::VestingPeriods),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreatePeriodicVestingAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.MsgCreatePeriodicVestingAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreatePeriodicVestingAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut from_address__ = None;
                let mut to_address__ = None;
                let mut start_time__ = None;
                let mut vesting_periods__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FromAddress => {
                            if from_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fromAddress"));
                            }
                            from_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ToAddress => {
                            if to_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toAddress"));
                            }
                            to_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::StartTime => {
                            if start_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTime"));
                            }
                            start_time__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VestingPeriods => {
                            if vesting_periods__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vestingPeriods"));
                            }
                            vesting_periods__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgCreatePeriodicVestingAccount {
                    from_address: from_address__.unwrap_or_default(),
                    to_address: to_address__.unwrap_or_default(),
                    start_time: start_time__.unwrap_or_default(),
                    vesting_periods: vesting_periods__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.MsgCreatePeriodicVestingAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreatePeriodicVestingAccountResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.MsgCreatePeriodicVestingAccountResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreatePeriodicVestingAccountResponse {
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
            type Value = MsgCreatePeriodicVestingAccountResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.MsgCreatePeriodicVestingAccountResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreatePeriodicVestingAccountResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgCreatePeriodicVestingAccountResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.MsgCreatePeriodicVestingAccountResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreatePermanentLockedAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.from_address.is_empty() {
            len += 1;
        }
        if !self.to_address.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.MsgCreatePermanentLockedAccount", len)?;
        if !self.from_address.is_empty() {
            struct_ser.serialize_field("fromAddress", &self.from_address)?;
        }
        if !self.to_address.is_empty() {
            struct_ser.serialize_field("toAddress", &self.to_address)?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreatePermanentLockedAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "from_address",
            "fromAddress",
            "to_address",
            "toAddress",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FromAddress,
            ToAddress,
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
                            "fromAddress" | "from_address" => Ok(GeneratedField::FromAddress),
                            "toAddress" | "to_address" => Ok(GeneratedField::ToAddress),
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
            type Value = MsgCreatePermanentLockedAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.MsgCreatePermanentLockedAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreatePermanentLockedAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut from_address__ = None;
                let mut to_address__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FromAddress => {
                            if from_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fromAddress"));
                            }
                            from_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ToAddress => {
                            if to_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toAddress"));
                            }
                            to_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgCreatePermanentLockedAccount {
                    from_address: from_address__.unwrap_or_default(),
                    to_address: to_address__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.MsgCreatePermanentLockedAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreatePermanentLockedAccountResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.MsgCreatePermanentLockedAccountResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreatePermanentLockedAccountResponse {
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
            type Value = MsgCreatePermanentLockedAccountResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.MsgCreatePermanentLockedAccountResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreatePermanentLockedAccountResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgCreatePermanentLockedAccountResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.MsgCreatePermanentLockedAccountResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateVestingAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.from_address.is_empty() {
            len += 1;
        }
        if !self.to_address.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        if self.end_time != 0 {
            len += 1;
        }
        if self.delayed {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.MsgCreateVestingAccount", len)?;
        if !self.from_address.is_empty() {
            struct_ser.serialize_field("fromAddress", &self.from_address)?;
        }
        if !self.to_address.is_empty() {
            struct_ser.serialize_field("toAddress", &self.to_address)?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        if self.end_time != 0 {
            struct_ser.serialize_field("endTime", ToString::to_string(&self.end_time).as_str())?;
        }
        if self.delayed {
            struct_ser.serialize_field("delayed", &self.delayed)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateVestingAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "from_address",
            "fromAddress",
            "to_address",
            "toAddress",
            "amount",
            "end_time",
            "endTime",
            "delayed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FromAddress,
            ToAddress,
            Amount,
            EndTime,
            Delayed,
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
                            "fromAddress" | "from_address" => Ok(GeneratedField::FromAddress),
                            "toAddress" | "to_address" => Ok(GeneratedField::ToAddress),
                            "amount" => Ok(GeneratedField::Amount),
                            "endTime" | "end_time" => Ok(GeneratedField::EndTime),
                            "delayed" => Ok(GeneratedField::Delayed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreateVestingAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.MsgCreateVestingAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateVestingAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut from_address__ = None;
                let mut to_address__ = None;
                let mut amount__ = None;
                let mut end_time__ = None;
                let mut delayed__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FromAddress => {
                            if from_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fromAddress"));
                            }
                            from_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ToAddress => {
                            if to_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toAddress"));
                            }
                            to_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                        GeneratedField::EndTime => {
                            if end_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endTime"));
                            }
                            end_time__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Delayed => {
                            if delayed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delayed"));
                            }
                            delayed__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgCreateVestingAccount {
                    from_address: from_address__.unwrap_or_default(),
                    to_address: to_address__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                    end_time: end_time__.unwrap_or_default(),
                    delayed: delayed__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.MsgCreateVestingAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateVestingAccountResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.MsgCreateVestingAccountResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateVestingAccountResponse {
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
            type Value = MsgCreateVestingAccountResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.MsgCreateVestingAccountResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateVestingAccountResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgCreateVestingAccountResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.MsgCreateVestingAccountResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Period {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.length != 0 {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.Period", len)?;
        if self.length != 0 {
            struct_ser.serialize_field("length", ToString::to_string(&self.length).as_str())?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Period {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "length",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Length,
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
                            "length" => Ok(GeneratedField::Length),
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
            type Value = Period;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.Period")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Period, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut length__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Length => {
                            if length__.is_some() {
                                return Err(serde::de::Error::duplicate_field("length"));
                            }
                            length__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Period {
                    length: length__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.Period", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PeriodicVestingAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_vesting_account.is_some() {
            len += 1;
        }
        if self.start_time != 0 {
            len += 1;
        }
        if !self.vesting_periods.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.PeriodicVestingAccount", len)?;
        if let Some(v) = self.base_vesting_account.as_ref() {
            struct_ser.serialize_field("baseVestingAccount", v)?;
        }
        if self.start_time != 0 {
            struct_ser.serialize_field("startTime", ToString::to_string(&self.start_time).as_str())?;
        }
        if !self.vesting_periods.is_empty() {
            struct_ser.serialize_field("vestingPeriods", &self.vesting_periods)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PeriodicVestingAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_vesting_account",
            "baseVestingAccount",
            "start_time",
            "startTime",
            "vesting_periods",
            "vestingPeriods",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseVestingAccount,
            StartTime,
            VestingPeriods,
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
                            "baseVestingAccount" | "base_vesting_account" => Ok(GeneratedField::BaseVestingAccount),
                            "startTime" | "start_time" => Ok(GeneratedField::StartTime),
                            "vestingPeriods" | "vesting_periods" => Ok(GeneratedField::VestingPeriods),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PeriodicVestingAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.PeriodicVestingAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PeriodicVestingAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_vesting_account__ = None;
                let mut start_time__ = None;
                let mut vesting_periods__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BaseVestingAccount => {
                            if base_vesting_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseVestingAccount"));
                            }
                            base_vesting_account__ = map.next_value()?;
                        }
                        GeneratedField::StartTime => {
                            if start_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTime"));
                            }
                            start_time__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VestingPeriods => {
                            if vesting_periods__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vestingPeriods"));
                            }
                            vesting_periods__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PeriodicVestingAccount {
                    base_vesting_account: base_vesting_account__,
                    start_time: start_time__.unwrap_or_default(),
                    vesting_periods: vesting_periods__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.PeriodicVestingAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PermanentLockedAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_vesting_account.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.vesting.v1beta1.PermanentLockedAccount", len)?;
        if let Some(v) = self.base_vesting_account.as_ref() {
            struct_ser.serialize_field("baseVestingAccount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PermanentLockedAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_vesting_account",
            "baseVestingAccount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseVestingAccount,
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
                            "baseVestingAccount" | "base_vesting_account" => Ok(GeneratedField::BaseVestingAccount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PermanentLockedAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.vesting.v1beta1.PermanentLockedAccount")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PermanentLockedAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_vesting_account__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BaseVestingAccount => {
                            if base_vesting_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseVestingAccount"));
                            }
                            base_vesting_account__ = map.next_value()?;
                        }
                    }
                }
                Ok(PermanentLockedAccount {
                    base_vesting_account: base_vesting_account__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.vesting.v1beta1.PermanentLockedAccount", FIELDS, GeneratedVisitor)
    }
}
