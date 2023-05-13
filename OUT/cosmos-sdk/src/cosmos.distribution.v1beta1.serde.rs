// @generated
impl serde::Serialize for CommunityPoolSpendProposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        if !self.recipient.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.CommunityPoolSpendProposal", len)?;
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if !self.recipient.is_empty() {
            struct_ser.serialize_field("recipient", &self.recipient)?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommunityPoolSpendProposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "title",
            "description",
            "recipient",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Title,
            Description,
            Recipient,
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
                            "title" => Ok(GeneratedField::Title),
                            "description" => Ok(GeneratedField::Description),
                            "recipient" => Ok(GeneratedField::Recipient),
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
            type Value = CommunityPoolSpendProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.CommunityPoolSpendProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CommunityPoolSpendProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut title__ = None;
                let mut description__ = None;
                let mut recipient__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map.next_value()?);
                        }
                        GeneratedField::Recipient => {
                            if recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recipient"));
                            }
                            recipient__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CommunityPoolSpendProposal {
                    title: title__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    recipient: recipient__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.CommunityPoolSpendProposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CommunityPoolSpendProposalWithDeposit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        if !self.recipient.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        if !self.deposit.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.CommunityPoolSpendProposalWithDeposit", len)?;
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if !self.recipient.is_empty() {
            struct_ser.serialize_field("recipient", &self.recipient)?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        if !self.deposit.is_empty() {
            struct_ser.serialize_field("deposit", &self.deposit)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommunityPoolSpendProposalWithDeposit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "title",
            "description",
            "recipient",
            "amount",
            "deposit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Title,
            Description,
            Recipient,
            Amount,
            Deposit,
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
                            "title" => Ok(GeneratedField::Title),
                            "description" => Ok(GeneratedField::Description),
                            "recipient" => Ok(GeneratedField::Recipient),
                            "amount" => Ok(GeneratedField::Amount),
                            "deposit" => Ok(GeneratedField::Deposit),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CommunityPoolSpendProposalWithDeposit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.CommunityPoolSpendProposalWithDeposit")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CommunityPoolSpendProposalWithDeposit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut title__ = None;
                let mut description__ = None;
                let mut recipient__ = None;
                let mut amount__ = None;
                let mut deposit__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map.next_value()?);
                        }
                        GeneratedField::Recipient => {
                            if recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recipient"));
                            }
                            recipient__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                        GeneratedField::Deposit => {
                            if deposit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deposit"));
                            }
                            deposit__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CommunityPoolSpendProposalWithDeposit {
                    title: title__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    recipient: recipient__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                    deposit: deposit__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.CommunityPoolSpendProposalWithDeposit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegationDelegatorReward {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if !self.reward.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.DelegationDelegatorReward", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if !self.reward.is_empty() {
            struct_ser.serialize_field("reward", &self.reward)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegationDelegatorReward {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
            "reward",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddress,
            Reward,
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
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "reward" => Ok(GeneratedField::Reward),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegationDelegatorReward;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.DelegationDelegatorReward")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegationDelegatorReward, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                let mut reward__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Reward => {
                            if reward__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reward"));
                            }
                            reward__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DelegationDelegatorReward {
                    validator_address: validator_address__.unwrap_or_default(),
                    reward: reward__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.DelegationDelegatorReward", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorStartingInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.previous_period != 0 {
            len += 1;
        }
        if !self.stake.is_empty() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.DelegatorStartingInfo", len)?;
        if self.previous_period != 0 {
            struct_ser.serialize_field("previousPeriod", ToString::to_string(&self.previous_period).as_str())?;
        }
        if !self.stake.is_empty() {
            struct_ser.serialize_field("stake", &self.stake)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorStartingInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "previous_period",
            "previousPeriod",
            "stake",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PreviousPeriod,
            Stake,
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
                            "previousPeriod" | "previous_period" => Ok(GeneratedField::PreviousPeriod),
                            "stake" => Ok(GeneratedField::Stake),
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
            type Value = DelegatorStartingInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.DelegatorStartingInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegatorStartingInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut previous_period__ = None;
                let mut stake__ = None;
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PreviousPeriod => {
                            if previous_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("previousPeriod"));
                            }
                            previous_period__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Stake => {
                            if stake__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stake"));
                            }
                            stake__ = Some(map.next_value()?);
                        }
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
                Ok(DelegatorStartingInfo {
                    previous_period: previous_period__.unwrap_or_default(),
                    stake: stake__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.DelegatorStartingInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorStartingInfoRecord {
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
        if self.starting_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.DelegatorStartingInfoRecord", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if let Some(v) = self.starting_info.as_ref() {
            struct_ser.serialize_field("startingInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorStartingInfoRecord {
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
            "starting_info",
            "startingInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            ValidatorAddress,
            StartingInfo,
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
                            "startingInfo" | "starting_info" => Ok(GeneratedField::StartingInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorStartingInfoRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.DelegatorStartingInfoRecord")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegatorStartingInfoRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut validator_address__ = None;
                let mut starting_info__ = None;
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
                        GeneratedField::StartingInfo => {
                            if starting_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startingInfo"));
                            }
                            starting_info__ = map.next_value()?;
                        }
                    }
                }
                Ok(DelegatorStartingInfoRecord {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                    starting_info: starting_info__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.DelegatorStartingInfoRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorWithdrawInfo {
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
        if !self.withdraw_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.DelegatorWithdrawInfo", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.withdraw_address.is_empty() {
            struct_ser.serialize_field("withdrawAddress", &self.withdraw_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorWithdrawInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "withdraw_address",
            "withdrawAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            WithdrawAddress,
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
                            "withdrawAddress" | "withdraw_address" => Ok(GeneratedField::WithdrawAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorWithdrawInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.DelegatorWithdrawInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegatorWithdrawInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut withdraw_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::WithdrawAddress => {
                            if withdraw_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawAddress"));
                            }
                            withdraw_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(DelegatorWithdrawInfo {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    withdraw_address: withdraw_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.DelegatorWithdrawInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FeePool {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.community_pool.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.FeePool", len)?;
        if !self.community_pool.is_empty() {
            struct_ser.serialize_field("communityPool", &self.community_pool)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FeePool {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "community_pool",
            "communityPool",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CommunityPool,
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
                            "communityPool" | "community_pool" => Ok(GeneratedField::CommunityPool),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FeePool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.FeePool")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FeePool, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut community_pool__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CommunityPool => {
                            if community_pool__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPool"));
                            }
                            community_pool__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(FeePool {
                    community_pool: community_pool__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.FeePool", FIELDS, GeneratedVisitor)
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
        if self.fee_pool.is_some() {
            len += 1;
        }
        if !self.delegator_withdraw_infos.is_empty() {
            len += 1;
        }
        if !self.previous_proposer.is_empty() {
            len += 1;
        }
        if !self.outstanding_rewards.is_empty() {
            len += 1;
        }
        if !self.validator_accumulated_commissions.is_empty() {
            len += 1;
        }
        if !self.validator_historical_rewards.is_empty() {
            len += 1;
        }
        if !self.validator_current_rewards.is_empty() {
            len += 1;
        }
        if !self.delegator_starting_infos.is_empty() {
            len += 1;
        }
        if !self.validator_slash_events.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.GenesisState", len)?;
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        if let Some(v) = self.fee_pool.as_ref() {
            struct_ser.serialize_field("feePool", v)?;
        }
        if !self.delegator_withdraw_infos.is_empty() {
            struct_ser.serialize_field("delegatorWithdrawInfos", &self.delegator_withdraw_infos)?;
        }
        if !self.previous_proposer.is_empty() {
            struct_ser.serialize_field("previousProposer", &self.previous_proposer)?;
        }
        if !self.outstanding_rewards.is_empty() {
            struct_ser.serialize_field("outstandingRewards", &self.outstanding_rewards)?;
        }
        if !self.validator_accumulated_commissions.is_empty() {
            struct_ser.serialize_field("validatorAccumulatedCommissions", &self.validator_accumulated_commissions)?;
        }
        if !self.validator_historical_rewards.is_empty() {
            struct_ser.serialize_field("validatorHistoricalRewards", &self.validator_historical_rewards)?;
        }
        if !self.validator_current_rewards.is_empty() {
            struct_ser.serialize_field("validatorCurrentRewards", &self.validator_current_rewards)?;
        }
        if !self.delegator_starting_infos.is_empty() {
            struct_ser.serialize_field("delegatorStartingInfos", &self.delegator_starting_infos)?;
        }
        if !self.validator_slash_events.is_empty() {
            struct_ser.serialize_field("validatorSlashEvents", &self.validator_slash_events)?;
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
            "fee_pool",
            "feePool",
            "delegator_withdraw_infos",
            "delegatorWithdrawInfos",
            "previous_proposer",
            "previousProposer",
            "outstanding_rewards",
            "outstandingRewards",
            "validator_accumulated_commissions",
            "validatorAccumulatedCommissions",
            "validator_historical_rewards",
            "validatorHistoricalRewards",
            "validator_current_rewards",
            "validatorCurrentRewards",
            "delegator_starting_infos",
            "delegatorStartingInfos",
            "validator_slash_events",
            "validatorSlashEvents",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Params,
            FeePool,
            DelegatorWithdrawInfos,
            PreviousProposer,
            OutstandingRewards,
            ValidatorAccumulatedCommissions,
            ValidatorHistoricalRewards,
            ValidatorCurrentRewards,
            DelegatorStartingInfos,
            ValidatorSlashEvents,
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
                            "feePool" | "fee_pool" => Ok(GeneratedField::FeePool),
                            "delegatorWithdrawInfos" | "delegator_withdraw_infos" => Ok(GeneratedField::DelegatorWithdrawInfos),
                            "previousProposer" | "previous_proposer" => Ok(GeneratedField::PreviousProposer),
                            "outstandingRewards" | "outstanding_rewards" => Ok(GeneratedField::OutstandingRewards),
                            "validatorAccumulatedCommissions" | "validator_accumulated_commissions" => Ok(GeneratedField::ValidatorAccumulatedCommissions),
                            "validatorHistoricalRewards" | "validator_historical_rewards" => Ok(GeneratedField::ValidatorHistoricalRewards),
                            "validatorCurrentRewards" | "validator_current_rewards" => Ok(GeneratedField::ValidatorCurrentRewards),
                            "delegatorStartingInfos" | "delegator_starting_infos" => Ok(GeneratedField::DelegatorStartingInfos),
                            "validatorSlashEvents" | "validator_slash_events" => Ok(GeneratedField::ValidatorSlashEvents),
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
                formatter.write_str("struct cosmos.distribution.v1beta1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut params__ = None;
                let mut fee_pool__ = None;
                let mut delegator_withdraw_infos__ = None;
                let mut previous_proposer__ = None;
                let mut outstanding_rewards__ = None;
                let mut validator_accumulated_commissions__ = None;
                let mut validator_historical_rewards__ = None;
                let mut validator_current_rewards__ = None;
                let mut delegator_starting_infos__ = None;
                let mut validator_slash_events__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                        GeneratedField::FeePool => {
                            if fee_pool__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feePool"));
                            }
                            fee_pool__ = map.next_value()?;
                        }
                        GeneratedField::DelegatorWithdrawInfos => {
                            if delegator_withdraw_infos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorWithdrawInfos"));
                            }
                            delegator_withdraw_infos__ = Some(map.next_value()?);
                        }
                        GeneratedField::PreviousProposer => {
                            if previous_proposer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("previousProposer"));
                            }
                            previous_proposer__ = Some(map.next_value()?);
                        }
                        GeneratedField::OutstandingRewards => {
                            if outstanding_rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outstandingRewards"));
                            }
                            outstanding_rewards__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorAccumulatedCommissions => {
                            if validator_accumulated_commissions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAccumulatedCommissions"));
                            }
                            validator_accumulated_commissions__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorHistoricalRewards => {
                            if validator_historical_rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorHistoricalRewards"));
                            }
                            validator_historical_rewards__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorCurrentRewards => {
                            if validator_current_rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorCurrentRewards"));
                            }
                            validator_current_rewards__ = Some(map.next_value()?);
                        }
                        GeneratedField::DelegatorStartingInfos => {
                            if delegator_starting_infos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorStartingInfos"));
                            }
                            delegator_starting_infos__ = Some(map.next_value()?);
                        }
                        GeneratedField::ValidatorSlashEvents => {
                            if validator_slash_events__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSlashEvents"));
                            }
                            validator_slash_events__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenesisState {
                    params: params__,
                    fee_pool: fee_pool__,
                    delegator_withdraw_infos: delegator_withdraw_infos__.unwrap_or_default(),
                    previous_proposer: previous_proposer__.unwrap_or_default(),
                    outstanding_rewards: outstanding_rewards__.unwrap_or_default(),
                    validator_accumulated_commissions: validator_accumulated_commissions__.unwrap_or_default(),
                    validator_historical_rewards: validator_historical_rewards__.unwrap_or_default(),
                    validator_current_rewards: validator_current_rewards__.unwrap_or_default(),
                    delegator_starting_infos: delegator_starting_infos__.unwrap_or_default(),
                    validator_slash_events: validator_slash_events__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCommunityPoolSpend {
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
        if !self.recipient.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgCommunityPoolSpend", len)?;
        if !self.authority.is_empty() {
            struct_ser.serialize_field("authority", &self.authority)?;
        }
        if !self.recipient.is_empty() {
            struct_ser.serialize_field("recipient", &self.recipient)?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCommunityPoolSpend {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authority",
            "recipient",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Authority,
            Recipient,
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
                            "authority" => Ok(GeneratedField::Authority),
                            "recipient" => Ok(GeneratedField::Recipient),
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
            type Value = MsgCommunityPoolSpend;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgCommunityPoolSpend")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCommunityPoolSpend, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authority__ = None;
                let mut recipient__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Authority => {
                            if authority__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authority"));
                            }
                            authority__ = Some(map.next_value()?);
                        }
                        GeneratedField::Recipient => {
                            if recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recipient"));
                            }
                            recipient__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgCommunityPoolSpend {
                    authority: authority__.unwrap_or_default(),
                    recipient: recipient__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgCommunityPoolSpend", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCommunityPoolSpendResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgCommunityPoolSpendResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCommunityPoolSpendResponse {
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
            type Value = MsgCommunityPoolSpendResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgCommunityPoolSpendResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCommunityPoolSpendResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgCommunityPoolSpendResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgCommunityPoolSpendResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgFundCommunityPool {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.amount.is_empty() {
            len += 1;
        }
        if !self.depositor.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgFundCommunityPool", len)?;
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        if !self.depositor.is_empty() {
            struct_ser.serialize_field("depositor", &self.depositor)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgFundCommunityPool {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "depositor",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
            Depositor,
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
                            "amount" => Ok(GeneratedField::Amount),
                            "depositor" => Ok(GeneratedField::Depositor),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgFundCommunityPool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgFundCommunityPool")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgFundCommunityPool, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut depositor__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                        GeneratedField::Depositor => {
                            if depositor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositor"));
                            }
                            depositor__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgFundCommunityPool {
                    amount: amount__.unwrap_or_default(),
                    depositor: depositor__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgFundCommunityPool", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgFundCommunityPoolResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgFundCommunityPoolResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgFundCommunityPoolResponse {
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
            type Value = MsgFundCommunityPoolResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgFundCommunityPoolResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgFundCommunityPoolResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgFundCommunityPoolResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgFundCommunityPoolResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgSetWithdrawAddress {
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
        if !self.withdraw_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgSetWithdrawAddress", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.withdraw_address.is_empty() {
            struct_ser.serialize_field("withdrawAddress", &self.withdraw_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgSetWithdrawAddress {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
            "withdraw_address",
            "withdrawAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
            WithdrawAddress,
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
                            "withdrawAddress" | "withdraw_address" => Ok(GeneratedField::WithdrawAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgSetWithdrawAddress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgSetWithdrawAddress")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgSetWithdrawAddress, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                let mut withdraw_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::WithdrawAddress => {
                            if withdraw_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawAddress"));
                            }
                            withdraw_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgSetWithdrawAddress {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    withdraw_address: withdraw_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgSetWithdrawAddress", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgSetWithdrawAddressResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgSetWithdrawAddressResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgSetWithdrawAddressResponse {
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
            type Value = MsgSetWithdrawAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgSetWithdrawAddressResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgSetWithdrawAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgSetWithdrawAddressResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgSetWithdrawAddressResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgUpdateParams", len)?;
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
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgUpdateParams")
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
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgUpdateParams", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgUpdateParamsResponse", len)?;
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
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgUpdateParamsResponse")
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
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgUpdateParamsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgWithdrawDelegatorReward {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgWithdrawDelegatorReward {
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
            type Value = MsgWithdrawDelegatorReward;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgWithdrawDelegatorReward, V::Error>
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
                Ok(MsgWithdrawDelegatorReward {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgWithdrawDelegatorRewardResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.amount.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgWithdrawDelegatorRewardResponse", len)?;
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgWithdrawDelegatorRewardResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = MsgWithdrawDelegatorRewardResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgWithdrawDelegatorRewardResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgWithdrawDelegatorRewardResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgWithdrawDelegatorRewardResponse {
                    amount: amount__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgWithdrawDelegatorRewardResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgWithdrawValidatorCommission {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgWithdrawValidatorCommission {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = MsgWithdrawValidatorCommission;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgWithdrawValidatorCommission, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgWithdrawValidatorCommission {
                    validator_address: validator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgWithdrawValidatorCommission", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgWithdrawValidatorCommissionResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.amount.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.MsgWithdrawValidatorCommissionResponse", len)?;
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgWithdrawValidatorCommissionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = MsgWithdrawValidatorCommissionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.MsgWithdrawValidatorCommissionResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgWithdrawValidatorCommissionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgWithdrawValidatorCommissionResponse {
                    amount: amount__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.MsgWithdrawValidatorCommissionResponse", FIELDS, GeneratedVisitor)
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
        if !self.community_tax.is_empty() {
            len += 1;
        }
        if !self.base_proposer_reward.is_empty() {
            len += 1;
        }
        if !self.bonus_proposer_reward.is_empty() {
            len += 1;
        }
        if self.withdraw_addr_enabled {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.Params", len)?;
        if !self.community_tax.is_empty() {
            struct_ser.serialize_field("communityTax", &self.community_tax)?;
        }
        if !self.base_proposer_reward.is_empty() {
            struct_ser.serialize_field("baseProposerReward", &self.base_proposer_reward)?;
        }
        if !self.bonus_proposer_reward.is_empty() {
            struct_ser.serialize_field("bonusProposerReward", &self.bonus_proposer_reward)?;
        }
        if self.withdraw_addr_enabled {
            struct_ser.serialize_field("withdrawAddrEnabled", &self.withdraw_addr_enabled)?;
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
            "community_tax",
            "communityTax",
            "base_proposer_reward",
            "baseProposerReward",
            "bonus_proposer_reward",
            "bonusProposerReward",
            "withdraw_addr_enabled",
            "withdrawAddrEnabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CommunityTax,
            BaseProposerReward,
            BonusProposerReward,
            WithdrawAddrEnabled,
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
                            "communityTax" | "community_tax" => Ok(GeneratedField::CommunityTax),
                            "baseProposerReward" | "base_proposer_reward" => Ok(GeneratedField::BaseProposerReward),
                            "bonusProposerReward" | "bonus_proposer_reward" => Ok(GeneratedField::BonusProposerReward),
                            "withdrawAddrEnabled" | "withdraw_addr_enabled" => Ok(GeneratedField::WithdrawAddrEnabled),
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
                formatter.write_str("struct cosmos.distribution.v1beta1.Params")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Params, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut community_tax__ = None;
                let mut base_proposer_reward__ = None;
                let mut bonus_proposer_reward__ = None;
                let mut withdraw_addr_enabled__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CommunityTax => {
                            if community_tax__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityTax"));
                            }
                            community_tax__ = Some(map.next_value()?);
                        }
                        GeneratedField::BaseProposerReward => {
                            if base_proposer_reward__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseProposerReward"));
                            }
                            base_proposer_reward__ = Some(map.next_value()?);
                        }
                        GeneratedField::BonusProposerReward => {
                            if bonus_proposer_reward__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bonusProposerReward"));
                            }
                            bonus_proposer_reward__ = Some(map.next_value()?);
                        }
                        GeneratedField::WithdrawAddrEnabled => {
                            if withdraw_addr_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawAddrEnabled"));
                            }
                            withdraw_addr_enabled__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Params {
                    community_tax: community_tax__.unwrap_or_default(),
                    base_proposer_reward: base_proposer_reward__.unwrap_or_default(),
                    bonus_proposer_reward: bonus_proposer_reward__.unwrap_or_default(),
                    withdraw_addr_enabled: withdraw_addr_enabled__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.Params", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryCommunityPoolRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryCommunityPoolRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryCommunityPoolRequest {
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
            type Value = QueryCommunityPoolRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryCommunityPoolRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryCommunityPoolRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(QueryCommunityPoolRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryCommunityPoolRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryCommunityPoolResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.pool.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryCommunityPoolResponse", len)?;
        if !self.pool.is_empty() {
            struct_ser.serialize_field("pool", &self.pool)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryCommunityPoolResponse {
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
            type Value = QueryCommunityPoolResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryCommunityPoolResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryCommunityPoolResponse, V::Error>
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
                            pool__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryCommunityPoolResponse {
                    pool: pool__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryCommunityPoolResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegationRewardsRequest {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryDelegationRewardsRequest", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegationRewardsRequest {
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
            type Value = QueryDelegationRewardsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryDelegationRewardsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegationRewardsRequest, V::Error>
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
                Ok(QueryDelegationRewardsRequest {
                    delegator_address: delegator_address__.unwrap_or_default(),
                    validator_address: validator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryDelegationRewardsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegationRewardsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.rewards.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryDelegationRewardsResponse", len)?;
        if !self.rewards.is_empty() {
            struct_ser.serialize_field("rewards", &self.rewards)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegationRewardsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rewards",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rewards,
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
                            "rewards" => Ok(GeneratedField::Rewards),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDelegationRewardsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryDelegationRewardsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegationRewardsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rewards__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Rewards => {
                            if rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewards"));
                            }
                            rewards__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegationRewardsResponse {
                    rewards: rewards__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryDelegationRewardsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegationTotalRewardsRequest {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryDelegationTotalRewardsRequest", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegationTotalRewardsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDelegationTotalRewardsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryDelegationTotalRewardsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegationTotalRewardsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegationTotalRewardsRequest {
                    delegator_address: delegator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryDelegationTotalRewardsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegationTotalRewardsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.rewards.is_empty() {
            len += 1;
        }
        if !self.total.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryDelegationTotalRewardsResponse", len)?;
        if !self.rewards.is_empty() {
            struct_ser.serialize_field("rewards", &self.rewards)?;
        }
        if !self.total.is_empty() {
            struct_ser.serialize_field("total", &self.total)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegationTotalRewardsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rewards",
            "total",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rewards,
            Total,
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
                            "rewards" => Ok(GeneratedField::Rewards),
                            "total" => Ok(GeneratedField::Total),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDelegationTotalRewardsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryDelegationTotalRewardsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegationTotalRewardsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rewards__ = None;
                let mut total__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Rewards => {
                            if rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewards"));
                            }
                            rewards__ = Some(map.next_value()?);
                        }
                        GeneratedField::Total => {
                            if total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("total"));
                            }
                            total__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegationTotalRewardsResponse {
                    rewards: rewards__.unwrap_or_default(),
                    total: total__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryDelegationTotalRewardsResponse", FIELDS, GeneratedVisitor)
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
        if !self.delegator_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryDelegatorValidatorsRequest", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
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
            "delegator_address",
            "delegatorAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
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
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryDelegatorValidatorsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorValidatorsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegatorValidatorsRequest {
                    delegator_address: delegator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryDelegatorValidatorsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryDelegatorValidatorsResponse", len)?;
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Validators,
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
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryDelegatorValidatorsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorValidatorsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validators__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegatorValidatorsResponse {
                    validators: validators__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryDelegatorValidatorsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorWithdrawAddressRequest {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryDelegatorWithdrawAddressRequest", len)?;
        if !self.delegator_address.is_empty() {
            struct_ser.serialize_field("delegatorAddress", &self.delegator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorWithdrawAddressRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_address",
            "delegatorAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorAddress,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDelegatorWithdrawAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryDelegatorWithdrawAddressRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorWithdrawAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorAddress => {
                            if delegator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorAddress"));
                            }
                            delegator_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegatorWithdrawAddressRequest {
                    delegator_address: delegator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryDelegatorWithdrawAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDelegatorWithdrawAddressResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.withdraw_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryDelegatorWithdrawAddressResponse", len)?;
        if !self.withdraw_address.is_empty() {
            struct_ser.serialize_field("withdrawAddress", &self.withdraw_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDelegatorWithdrawAddressResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "withdraw_address",
            "withdrawAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WithdrawAddress,
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
                            "withdrawAddress" | "withdraw_address" => Ok(GeneratedField::WithdrawAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryDelegatorWithdrawAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryDelegatorWithdrawAddressResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDelegatorWithdrawAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut withdraw_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::WithdrawAddress => {
                            if withdraw_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawAddress"));
                            }
                            withdraw_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDelegatorWithdrawAddressResponse {
                    withdraw_address: withdraw_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryDelegatorWithdrawAddressResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryParamsRequest", len)?;
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
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryParamsRequest")
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
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryParamsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryParamsResponse", len)?;
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
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryParamsResponse")
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
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryParamsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorCommissionRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryValidatorCommissionRequest", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorCommissionRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = QueryValidatorCommissionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryValidatorCommissionRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorCommissionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryValidatorCommissionRequest {
                    validator_address: validator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryValidatorCommissionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorCommissionResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.commission.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryValidatorCommissionResponse", len)?;
        if let Some(v) = self.commission.as_ref() {
            struct_ser.serialize_field("commission", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorCommissionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commission",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commission,
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
                            "commission" => Ok(GeneratedField::Commission),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryValidatorCommissionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryValidatorCommissionResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorCommissionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commission__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Commission => {
                            if commission__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commission"));
                            }
                            commission__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorCommissionResponse {
                    commission: commission__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryValidatorCommissionResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorDistributionInfoRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryValidatorDistributionInfoRequest", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorDistributionInfoRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = QueryValidatorDistributionInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryValidatorDistributionInfoRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorDistributionInfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryValidatorDistributionInfoRequest {
                    validator_address: validator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryValidatorDistributionInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorDistributionInfoResponse {
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
        if !self.self_bond_rewards.is_empty() {
            len += 1;
        }
        if !self.commission.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryValidatorDistributionInfoResponse", len)?;
        if !self.operator_address.is_empty() {
            struct_ser.serialize_field("operatorAddress", &self.operator_address)?;
        }
        if !self.self_bond_rewards.is_empty() {
            struct_ser.serialize_field("selfBondRewards", &self.self_bond_rewards)?;
        }
        if !self.commission.is_empty() {
            struct_ser.serialize_field("commission", &self.commission)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorDistributionInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "operator_address",
            "operatorAddress",
            "self_bond_rewards",
            "selfBondRewards",
            "commission",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OperatorAddress,
            SelfBondRewards,
            Commission,
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
                            "selfBondRewards" | "self_bond_rewards" => Ok(GeneratedField::SelfBondRewards),
                            "commission" => Ok(GeneratedField::Commission),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryValidatorDistributionInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryValidatorDistributionInfoResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorDistributionInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut operator_address__ = None;
                let mut self_bond_rewards__ = None;
                let mut commission__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::OperatorAddress => {
                            if operator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operatorAddress"));
                            }
                            operator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::SelfBondRewards => {
                            if self_bond_rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("selfBondRewards"));
                            }
                            self_bond_rewards__ = Some(map.next_value()?);
                        }
                        GeneratedField::Commission => {
                            if commission__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commission"));
                            }
                            commission__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryValidatorDistributionInfoResponse {
                    operator_address: operator_address__.unwrap_or_default(),
                    self_bond_rewards: self_bond_rewards__.unwrap_or_default(),
                    commission: commission__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryValidatorDistributionInfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorOutstandingRewardsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryValidatorOutstandingRewardsRequest", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorOutstandingRewardsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = QueryValidatorOutstandingRewardsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryValidatorOutstandingRewardsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorOutstandingRewardsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryValidatorOutstandingRewardsRequest {
                    validator_address: validator_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryValidatorOutstandingRewardsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorOutstandingRewardsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.rewards.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryValidatorOutstandingRewardsResponse", len)?;
        if let Some(v) = self.rewards.as_ref() {
            struct_ser.serialize_field("rewards", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorOutstandingRewardsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rewards",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rewards,
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
                            "rewards" => Ok(GeneratedField::Rewards),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryValidatorOutstandingRewardsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryValidatorOutstandingRewardsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorOutstandingRewardsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rewards__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Rewards => {
                            if rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewards"));
                            }
                            rewards__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorOutstandingRewardsResponse {
                    rewards: rewards__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryValidatorOutstandingRewardsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorSlashesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.starting_height != 0 {
            len += 1;
        }
        if self.ending_height != 0 {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryValidatorSlashesRequest", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if self.starting_height != 0 {
            struct_ser.serialize_field("startingHeight", ToString::to_string(&self.starting_height).as_str())?;
        }
        if self.ending_height != 0 {
            struct_ser.serialize_field("endingHeight", ToString::to_string(&self.ending_height).as_str())?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorSlashesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
            "starting_height",
            "startingHeight",
            "ending_height",
            "endingHeight",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddress,
            StartingHeight,
            EndingHeight,
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
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "startingHeight" | "starting_height" => Ok(GeneratedField::StartingHeight),
                            "endingHeight" | "ending_height" => Ok(GeneratedField::EndingHeight),
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
            type Value = QueryValidatorSlashesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryValidatorSlashesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorSlashesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                let mut starting_height__ = None;
                let mut ending_height__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::StartingHeight => {
                            if starting_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startingHeight"));
                            }
                            starting_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndingHeight => {
                            if ending_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endingHeight"));
                            }
                            ending_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorSlashesRequest {
                    validator_address: validator_address__.unwrap_or_default(),
                    starting_height: starting_height__.unwrap_or_default(),
                    ending_height: ending_height__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryValidatorSlashesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryValidatorSlashesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.slashes.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.QueryValidatorSlashesResponse", len)?;
        if !self.slashes.is_empty() {
            struct_ser.serialize_field("slashes", &self.slashes)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryValidatorSlashesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "slashes",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Slashes,
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
                            "slashes" => Ok(GeneratedField::Slashes),
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
            type Value = QueryValidatorSlashesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.QueryValidatorSlashesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryValidatorSlashesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut slashes__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Slashes => {
                            if slashes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashes"));
                            }
                            slashes__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryValidatorSlashesResponse {
                    slashes: slashes__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.QueryValidatorSlashesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorAccumulatedCommission {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.commission.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorAccumulatedCommission", len)?;
        if !self.commission.is_empty() {
            struct_ser.serialize_field("commission", &self.commission)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorAccumulatedCommission {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commission",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commission,
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
                            "commission" => Ok(GeneratedField::Commission),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorAccumulatedCommission;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorAccumulatedCommission")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorAccumulatedCommission, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commission__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Commission => {
                            if commission__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commission"));
                            }
                            commission__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValidatorAccumulatedCommission {
                    commission: commission__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorAccumulatedCommission", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorAccumulatedCommissionRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.accumulated.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorAccumulatedCommissionRecord", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if let Some(v) = self.accumulated.as_ref() {
            struct_ser.serialize_field("accumulated", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorAccumulatedCommissionRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
            "accumulated",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddress,
            Accumulated,
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
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "accumulated" => Ok(GeneratedField::Accumulated),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorAccumulatedCommissionRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorAccumulatedCommissionRecord")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorAccumulatedCommissionRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                let mut accumulated__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Accumulated => {
                            if accumulated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accumulated"));
                            }
                            accumulated__ = map.next_value()?;
                        }
                    }
                }
                Ok(ValidatorAccumulatedCommissionRecord {
                    validator_address: validator_address__.unwrap_or_default(),
                    accumulated: accumulated__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorAccumulatedCommissionRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorCurrentRewards {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.rewards.is_empty() {
            len += 1;
        }
        if self.period != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorCurrentRewards", len)?;
        if !self.rewards.is_empty() {
            struct_ser.serialize_field("rewards", &self.rewards)?;
        }
        if self.period != 0 {
            struct_ser.serialize_field("period", ToString::to_string(&self.period).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorCurrentRewards {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rewards",
            "period",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rewards,
            Period,
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
                            "rewards" => Ok(GeneratedField::Rewards),
                            "period" => Ok(GeneratedField::Period),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorCurrentRewards;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorCurrentRewards")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorCurrentRewards, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rewards__ = None;
                let mut period__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Rewards => {
                            if rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewards"));
                            }
                            rewards__ = Some(map.next_value()?);
                        }
                        GeneratedField::Period => {
                            if period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("period"));
                            }
                            period__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ValidatorCurrentRewards {
                    rewards: rewards__.unwrap_or_default(),
                    period: period__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorCurrentRewards", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorCurrentRewardsRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.rewards.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorCurrentRewardsRecord", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if let Some(v) = self.rewards.as_ref() {
            struct_ser.serialize_field("rewards", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorCurrentRewardsRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
            "rewards",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddress,
            Rewards,
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
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "rewards" => Ok(GeneratedField::Rewards),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorCurrentRewardsRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorCurrentRewardsRecord")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorCurrentRewardsRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                let mut rewards__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Rewards => {
                            if rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewards"));
                            }
                            rewards__ = map.next_value()?;
                        }
                    }
                }
                Ok(ValidatorCurrentRewardsRecord {
                    validator_address: validator_address__.unwrap_or_default(),
                    rewards: rewards__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorCurrentRewardsRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorHistoricalRewards {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.cumulative_reward_ratio.is_empty() {
            len += 1;
        }
        if self.reference_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorHistoricalRewards", len)?;
        if !self.cumulative_reward_ratio.is_empty() {
            struct_ser.serialize_field("cumulativeRewardRatio", &self.cumulative_reward_ratio)?;
        }
        if self.reference_count != 0 {
            struct_ser.serialize_field("referenceCount", &self.reference_count)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorHistoricalRewards {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cumulative_reward_ratio",
            "cumulativeRewardRatio",
            "reference_count",
            "referenceCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CumulativeRewardRatio,
            ReferenceCount,
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
                            "cumulativeRewardRatio" | "cumulative_reward_ratio" => Ok(GeneratedField::CumulativeRewardRatio),
                            "referenceCount" | "reference_count" => Ok(GeneratedField::ReferenceCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorHistoricalRewards;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorHistoricalRewards")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorHistoricalRewards, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cumulative_reward_ratio__ = None;
                let mut reference_count__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CumulativeRewardRatio => {
                            if cumulative_reward_ratio__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cumulativeRewardRatio"));
                            }
                            cumulative_reward_ratio__ = Some(map.next_value()?);
                        }
                        GeneratedField::ReferenceCount => {
                            if reference_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("referenceCount"));
                            }
                            reference_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ValidatorHistoricalRewards {
                    cumulative_reward_ratio: cumulative_reward_ratio__.unwrap_or_default(),
                    reference_count: reference_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorHistoricalRewards", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorHistoricalRewardsRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.period != 0 {
            len += 1;
        }
        if self.rewards.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorHistoricalRewardsRecord", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if self.period != 0 {
            struct_ser.serialize_field("period", ToString::to_string(&self.period).as_str())?;
        }
        if let Some(v) = self.rewards.as_ref() {
            struct_ser.serialize_field("rewards", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorHistoricalRewardsRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
            "period",
            "rewards",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddress,
            Period,
            Rewards,
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
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "period" => Ok(GeneratedField::Period),
                            "rewards" => Ok(GeneratedField::Rewards),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorHistoricalRewardsRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorHistoricalRewardsRecord")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorHistoricalRewardsRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                let mut period__ = None;
                let mut rewards__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Period => {
                            if period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("period"));
                            }
                            period__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Rewards => {
                            if rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewards"));
                            }
                            rewards__ = map.next_value()?;
                        }
                    }
                }
                Ok(ValidatorHistoricalRewardsRecord {
                    validator_address: validator_address__.unwrap_or_default(),
                    period: period__.unwrap_or_default(),
                    rewards: rewards__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorHistoricalRewardsRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorOutstandingRewards {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.rewards.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorOutstandingRewards", len)?;
        if !self.rewards.is_empty() {
            struct_ser.serialize_field("rewards", &self.rewards)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorOutstandingRewards {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rewards",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rewards,
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
                            "rewards" => Ok(GeneratedField::Rewards),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorOutstandingRewards;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorOutstandingRewards")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorOutstandingRewards, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rewards__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Rewards => {
                            if rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewards"));
                            }
                            rewards__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValidatorOutstandingRewards {
                    rewards: rewards__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorOutstandingRewards", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorOutstandingRewardsRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if !self.outstanding_rewards.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorOutstandingRewardsRecord", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if !self.outstanding_rewards.is_empty() {
            struct_ser.serialize_field("outstandingRewards", &self.outstanding_rewards)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorOutstandingRewardsRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
            "outstanding_rewards",
            "outstandingRewards",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddress,
            OutstandingRewards,
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
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "outstandingRewards" | "outstanding_rewards" => Ok(GeneratedField::OutstandingRewards),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorOutstandingRewardsRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorOutstandingRewardsRecord")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorOutstandingRewardsRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                let mut outstanding_rewards__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::OutstandingRewards => {
                            if outstanding_rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outstandingRewards"));
                            }
                            outstanding_rewards__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValidatorOutstandingRewardsRecord {
                    validator_address: validator_address__.unwrap_or_default(),
                    outstanding_rewards: outstanding_rewards__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorOutstandingRewardsRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorSlashEvent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_period != 0 {
            len += 1;
        }
        if !self.fraction.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorSlashEvent", len)?;
        if self.validator_period != 0 {
            struct_ser.serialize_field("validatorPeriod", ToString::to_string(&self.validator_period).as_str())?;
        }
        if !self.fraction.is_empty() {
            struct_ser.serialize_field("fraction", &self.fraction)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorSlashEvent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_period",
            "validatorPeriod",
            "fraction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorPeriod,
            Fraction,
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
                            "validatorPeriod" | "validator_period" => Ok(GeneratedField::ValidatorPeriod),
                            "fraction" => Ok(GeneratedField::Fraction),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorSlashEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorSlashEvent")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorSlashEvent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_period__ = None;
                let mut fraction__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorPeriod => {
                            if validator_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorPeriod"));
                            }
                            validator_period__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Fraction => {
                            if fraction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fraction"));
                            }
                            fraction__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValidatorSlashEvent {
                    validator_period: validator_period__.unwrap_or_default(),
                    fraction: fraction__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorSlashEvent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorSlashEventRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_address.is_empty() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.period != 0 {
            len += 1;
        }
        if self.validator_slash_event.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorSlashEventRecord", len)?;
        if !self.validator_address.is_empty() {
            struct_ser.serialize_field("validatorAddress", &self.validator_address)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.period != 0 {
            struct_ser.serialize_field("period", ToString::to_string(&self.period).as_str())?;
        }
        if let Some(v) = self.validator_slash_event.as_ref() {
            struct_ser.serialize_field("validatorSlashEvent", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorSlashEventRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_address",
            "validatorAddress",
            "height",
            "period",
            "validator_slash_event",
            "validatorSlashEvent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorAddress,
            Height,
            Period,
            ValidatorSlashEvent,
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
                            "validatorAddress" | "validator_address" => Ok(GeneratedField::ValidatorAddress),
                            "height" => Ok(GeneratedField::Height),
                            "period" => Ok(GeneratedField::Period),
                            "validatorSlashEvent" | "validator_slash_event" => Ok(GeneratedField::ValidatorSlashEvent),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorSlashEventRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorSlashEventRecord")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorSlashEventRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_address__ = None;
                let mut height__ = None;
                let mut period__ = None;
                let mut validator_slash_event__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorAddress => {
                            if validator_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorAddress"));
                            }
                            validator_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Period => {
                            if period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("period"));
                            }
                            period__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValidatorSlashEvent => {
                            if validator_slash_event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSlashEvent"));
                            }
                            validator_slash_event__ = map.next_value()?;
                        }
                    }
                }
                Ok(ValidatorSlashEventRecord {
                    validator_address: validator_address__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    period: period__.unwrap_or_default(),
                    validator_slash_event: validator_slash_event__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorSlashEventRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorSlashEvents {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_slash_events.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.distribution.v1beta1.ValidatorSlashEvents", len)?;
        if !self.validator_slash_events.is_empty() {
            struct_ser.serialize_field("validatorSlashEvents", &self.validator_slash_events)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorSlashEvents {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_slash_events",
            "validatorSlashEvents",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorSlashEvents,
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
                            "validatorSlashEvents" | "validator_slash_events" => Ok(GeneratedField::ValidatorSlashEvents),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorSlashEvents;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.distribution.v1beta1.ValidatorSlashEvents")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorSlashEvents, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_slash_events__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ValidatorSlashEvents => {
                            if validator_slash_events__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSlashEvents"));
                            }
                            validator_slash_events__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ValidatorSlashEvents {
                    validator_slash_events: validator_slash_events__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.distribution.v1beta1.ValidatorSlashEvents", FIELDS, GeneratedVisitor)
    }
}
