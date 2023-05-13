// @generated
impl serde::Serialize for Deposit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.depositor.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.Deposit", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.depositor.is_empty() {
            struct_ser.serialize_field("depositor", &self.depositor)?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Deposit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "depositor",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Depositor,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "depositor" => Ok(GeneratedField::Depositor),
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
            type Value = Deposit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.Deposit")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Deposit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut depositor__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Depositor => {
                            if depositor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositor"));
                            }
                            depositor__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Deposit {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    depositor: depositor__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.Deposit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DepositParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.min_deposit.is_empty() {
            len += 1;
        }
        if self.max_deposit_period.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.DepositParams", len)?;
        if !self.min_deposit.is_empty() {
            struct_ser.serialize_field("minDeposit", &self.min_deposit)?;
        }
        if let Some(v) = self.max_deposit_period.as_ref() {
            struct_ser.serialize_field("maxDepositPeriod", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DepositParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "min_deposit",
            "minDeposit",
            "max_deposit_period",
            "maxDepositPeriod",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MinDeposit,
            MaxDepositPeriod,
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
                            "minDeposit" | "min_deposit" => Ok(GeneratedField::MinDeposit),
                            "maxDepositPeriod" | "max_deposit_period" => Ok(GeneratedField::MaxDepositPeriod),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DepositParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.DepositParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DepositParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut min_deposit__ = None;
                let mut max_deposit_period__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MinDeposit => {
                            if min_deposit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minDeposit"));
                            }
                            min_deposit__ = Some(map.next_value()?);
                        }
                        GeneratedField::MaxDepositPeriod => {
                            if max_deposit_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxDepositPeriod"));
                            }
                            max_deposit_period__ = map.next_value()?;
                        }
                    }
                }
                Ok(DepositParams {
                    min_deposit: min_deposit__.unwrap_or_default(),
                    max_deposit_period: max_deposit_period__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.DepositParams", FIELDS, GeneratedVisitor)
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
        if self.starting_proposal_id != 0 {
            len += 1;
        }
        if !self.deposits.is_empty() {
            len += 1;
        }
        if !self.votes.is_empty() {
            len += 1;
        }
        if !self.proposals.is_empty() {
            len += 1;
        }
        if self.deposit_params.is_some() {
            len += 1;
        }
        if self.voting_params.is_some() {
            len += 1;
        }
        if self.tally_params.is_some() {
            len += 1;
        }
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.GenesisState", len)?;
        if self.starting_proposal_id != 0 {
            struct_ser.serialize_field("startingProposalId", ToString::to_string(&self.starting_proposal_id).as_str())?;
        }
        if !self.deposits.is_empty() {
            struct_ser.serialize_field("deposits", &self.deposits)?;
        }
        if !self.votes.is_empty() {
            struct_ser.serialize_field("votes", &self.votes)?;
        }
        if !self.proposals.is_empty() {
            struct_ser.serialize_field("proposals", &self.proposals)?;
        }
        if let Some(v) = self.deposit_params.as_ref() {
            struct_ser.serialize_field("depositParams", v)?;
        }
        if let Some(v) = self.voting_params.as_ref() {
            struct_ser.serialize_field("votingParams", v)?;
        }
        if let Some(v) = self.tally_params.as_ref() {
            struct_ser.serialize_field("tallyParams", v)?;
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
            "starting_proposal_id",
            "startingProposalId",
            "deposits",
            "votes",
            "proposals",
            "deposit_params",
            "depositParams",
            "voting_params",
            "votingParams",
            "tally_params",
            "tallyParams",
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartingProposalId,
            Deposits,
            Votes,
            Proposals,
            DepositParams,
            VotingParams,
            TallyParams,
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
                            "startingProposalId" | "starting_proposal_id" => Ok(GeneratedField::StartingProposalId),
                            "deposits" => Ok(GeneratedField::Deposits),
                            "votes" => Ok(GeneratedField::Votes),
                            "proposals" => Ok(GeneratedField::Proposals),
                            "depositParams" | "deposit_params" => Ok(GeneratedField::DepositParams),
                            "votingParams" | "voting_params" => Ok(GeneratedField::VotingParams),
                            "tallyParams" | "tally_params" => Ok(GeneratedField::TallyParams),
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
                formatter.write_str("struct cosmos.gov.v1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut starting_proposal_id__ = None;
                let mut deposits__ = None;
                let mut votes__ = None;
                let mut proposals__ = None;
                let mut deposit_params__ = None;
                let mut voting_params__ = None;
                let mut tally_params__ = None;
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::StartingProposalId => {
                            if starting_proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startingProposalId"));
                            }
                            starting_proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Deposits => {
                            if deposits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deposits"));
                            }
                            deposits__ = Some(map.next_value()?);
                        }
                        GeneratedField::Votes => {
                            if votes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votes"));
                            }
                            votes__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proposals => {
                            if proposals__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposals"));
                            }
                            proposals__ = Some(map.next_value()?);
                        }
                        GeneratedField::DepositParams => {
                            if deposit_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositParams"));
                            }
                            deposit_params__ = map.next_value()?;
                        }
                        GeneratedField::VotingParams => {
                            if voting_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingParams"));
                            }
                            voting_params__ = map.next_value()?;
                        }
                        GeneratedField::TallyParams => {
                            if tally_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tallyParams"));
                            }
                            tally_params__ = map.next_value()?;
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
                    starting_proposal_id: starting_proposal_id__.unwrap_or_default(),
                    deposits: deposits__.unwrap_or_default(),
                    votes: votes__.unwrap_or_default(),
                    proposals: proposals__.unwrap_or_default(),
                    deposit_params: deposit_params__,
                    voting_params: voting_params__,
                    tally_params: tally_params__,
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgDeposit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.depositor.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgDeposit", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.depositor.is_empty() {
            struct_ser.serialize_field("depositor", &self.depositor)?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgDeposit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "depositor",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Depositor,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "depositor" => Ok(GeneratedField::Depositor),
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
            type Value = MsgDeposit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgDeposit")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgDeposit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut depositor__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Depositor => {
                            if depositor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositor"));
                            }
                            depositor__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgDeposit {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    depositor: depositor__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgDeposit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgDepositResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgDepositResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgDepositResponse {
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
            type Value = MsgDepositResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgDepositResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgDepositResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgDepositResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgDepositResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgExecLegacyContent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.content.is_some() {
            len += 1;
        }
        if !self.authority.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgExecLegacyContent", len)?;
        if let Some(v) = self.content.as_ref() {
            struct_ser.serialize_field("content", v)?;
        }
        if !self.authority.is_empty() {
            struct_ser.serialize_field("authority", &self.authority)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgExecLegacyContent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "content",
            "authority",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Content,
            Authority,
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
                            "content" => Ok(GeneratedField::Content),
                            "authority" => Ok(GeneratedField::Authority),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgExecLegacyContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgExecLegacyContent")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgExecLegacyContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut content__ = None;
                let mut authority__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Content => {
                            if content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("content"));
                            }
                            content__ = map.next_value()?;
                        }
                        GeneratedField::Authority => {
                            if authority__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authority"));
                            }
                            authority__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgExecLegacyContent {
                    content: content__,
                    authority: authority__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgExecLegacyContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgExecLegacyContentResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgExecLegacyContentResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgExecLegacyContentResponse {
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
            type Value = MsgExecLegacyContentResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgExecLegacyContentResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgExecLegacyContentResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgExecLegacyContentResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgExecLegacyContentResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgSubmitProposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.messages.is_empty() {
            len += 1;
        }
        if !self.initial_deposit.is_empty() {
            len += 1;
        }
        if !self.proposer.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.summary.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgSubmitProposal", len)?;
        if !self.messages.is_empty() {
            struct_ser.serialize_field("messages", &self.messages)?;
        }
        if !self.initial_deposit.is_empty() {
            struct_ser.serialize_field("initialDeposit", &self.initial_deposit)?;
        }
        if !self.proposer.is_empty() {
            struct_ser.serialize_field("proposer", &self.proposer)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.summary.is_empty() {
            struct_ser.serialize_field("summary", &self.summary)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgSubmitProposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "messages",
            "initial_deposit",
            "initialDeposit",
            "proposer",
            "metadata",
            "title",
            "summary",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Messages,
            InitialDeposit,
            Proposer,
            Metadata,
            Title,
            Summary,
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
                            "messages" => Ok(GeneratedField::Messages),
                            "initialDeposit" | "initial_deposit" => Ok(GeneratedField::InitialDeposit),
                            "proposer" => Ok(GeneratedField::Proposer),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "title" => Ok(GeneratedField::Title),
                            "summary" => Ok(GeneratedField::Summary),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgSubmitProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgSubmitProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgSubmitProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut messages__ = None;
                let mut initial_deposit__ = None;
                let mut proposer__ = None;
                let mut metadata__ = None;
                let mut title__ = None;
                let mut summary__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Messages => {
                            if messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messages"));
                            }
                            messages__ = Some(map.next_value()?);
                        }
                        GeneratedField::InitialDeposit => {
                            if initial_deposit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialDeposit"));
                            }
                            initial_deposit__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proposer => {
                            if proposer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposer"));
                            }
                            proposer__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map.next_value()?);
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgSubmitProposal {
                    messages: messages__.unwrap_or_default(),
                    initial_deposit: initial_deposit__.unwrap_or_default(),
                    proposer: proposer__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    title: title__.unwrap_or_default(),
                    summary: summary__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgSubmitProposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgSubmitProposalResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgSubmitProposalResponse", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgSubmitProposalResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgSubmitProposalResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgSubmitProposalResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgSubmitProposalResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgSubmitProposalResponse {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgSubmitProposalResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgUpdateParams", len)?;
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
                formatter.write_str("struct cosmos.gov.v1.MsgUpdateParams")
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
        deserializer.deserialize_struct("cosmos.gov.v1.MsgUpdateParams", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgUpdateParamsResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1.MsgUpdateParamsResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1.MsgUpdateParamsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.voter.is_empty() {
            len += 1;
        }
        if self.option != 0 {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgVote", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        if self.option != 0 {
            let v = VoteOption::from_i32(self.option)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.option)))?;
            struct_ser.serialize_field("option", &v)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "voter",
            "option",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
            Option,
            Metadata,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "voter" => Ok(GeneratedField::Voter),
                            "option" => Ok(GeneratedField::Option),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgVote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                let mut option__ = None;
                let mut metadata__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Option => {
                            if option__.is_some() {
                                return Err(serde::de::Error::duplicate_field("option"));
                            }
                            option__ = Some(map.next_value::<VoteOption>()? as i32);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgVote {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    option: option__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgVoteResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgVoteResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgVoteResponse {
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
            type Value = MsgVoteResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgVoteResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgVoteResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgVoteResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgVoteResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgVoteWeighted {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.voter.is_empty() {
            len += 1;
        }
        if !self.options.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgVoteWeighted", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        if !self.options.is_empty() {
            struct_ser.serialize_field("options", &self.options)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgVoteWeighted {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "voter",
            "options",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
            Options,
            Metadata,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "voter" => Ok(GeneratedField::Voter),
                            "options" => Ok(GeneratedField::Options),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgVoteWeighted;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgVoteWeighted")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgVoteWeighted, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                let mut options__ = None;
                let mut metadata__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Options => {
                            if options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("options"));
                            }
                            options__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgVoteWeighted {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    options: options__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgVoteWeighted", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgVoteWeightedResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.gov.v1.MsgVoteWeightedResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgVoteWeightedResponse {
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
            type Value = MsgVoteWeightedResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.MsgVoteWeightedResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgVoteWeightedResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgVoteWeightedResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.MsgVoteWeightedResponse", FIELDS, GeneratedVisitor)
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
        if !self.min_deposit.is_empty() {
            len += 1;
        }
        if self.max_deposit_period.is_some() {
            len += 1;
        }
        if self.voting_period.is_some() {
            len += 1;
        }
        if !self.quorum.is_empty() {
            len += 1;
        }
        if !self.threshold.is_empty() {
            len += 1;
        }
        if !self.veto_threshold.is_empty() {
            len += 1;
        }
        if !self.min_initial_deposit_ratio.is_empty() {
            len += 1;
        }
        if self.burn_vote_quorum {
            len += 1;
        }
        if self.burn_proposal_deposit_prevote {
            len += 1;
        }
        if self.burn_vote_veto {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.Params", len)?;
        if !self.min_deposit.is_empty() {
            struct_ser.serialize_field("minDeposit", &self.min_deposit)?;
        }
        if let Some(v) = self.max_deposit_period.as_ref() {
            struct_ser.serialize_field("maxDepositPeriod", v)?;
        }
        if let Some(v) = self.voting_period.as_ref() {
            struct_ser.serialize_field("votingPeriod", v)?;
        }
        if !self.quorum.is_empty() {
            struct_ser.serialize_field("quorum", &self.quorum)?;
        }
        if !self.threshold.is_empty() {
            struct_ser.serialize_field("threshold", &self.threshold)?;
        }
        if !self.veto_threshold.is_empty() {
            struct_ser.serialize_field("vetoThreshold", &self.veto_threshold)?;
        }
        if !self.min_initial_deposit_ratio.is_empty() {
            struct_ser.serialize_field("minInitialDepositRatio", &self.min_initial_deposit_ratio)?;
        }
        if self.burn_vote_quorum {
            struct_ser.serialize_field("burnVoteQuorum", &self.burn_vote_quorum)?;
        }
        if self.burn_proposal_deposit_prevote {
            struct_ser.serialize_field("burnProposalDepositPrevote", &self.burn_proposal_deposit_prevote)?;
        }
        if self.burn_vote_veto {
            struct_ser.serialize_field("burnVoteVeto", &self.burn_vote_veto)?;
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
            "min_deposit",
            "minDeposit",
            "max_deposit_period",
            "maxDepositPeriod",
            "voting_period",
            "votingPeriod",
            "quorum",
            "threshold",
            "veto_threshold",
            "vetoThreshold",
            "min_initial_deposit_ratio",
            "minInitialDepositRatio",
            "burn_vote_quorum",
            "burnVoteQuorum",
            "burn_proposal_deposit_prevote",
            "burnProposalDepositPrevote",
            "burn_vote_veto",
            "burnVoteVeto",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MinDeposit,
            MaxDepositPeriod,
            VotingPeriod,
            Quorum,
            Threshold,
            VetoThreshold,
            MinInitialDepositRatio,
            BurnVoteQuorum,
            BurnProposalDepositPrevote,
            BurnVoteVeto,
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
                            "minDeposit" | "min_deposit" => Ok(GeneratedField::MinDeposit),
                            "maxDepositPeriod" | "max_deposit_period" => Ok(GeneratedField::MaxDepositPeriod),
                            "votingPeriod" | "voting_period" => Ok(GeneratedField::VotingPeriod),
                            "quorum" => Ok(GeneratedField::Quorum),
                            "threshold" => Ok(GeneratedField::Threshold),
                            "vetoThreshold" | "veto_threshold" => Ok(GeneratedField::VetoThreshold),
                            "minInitialDepositRatio" | "min_initial_deposit_ratio" => Ok(GeneratedField::MinInitialDepositRatio),
                            "burnVoteQuorum" | "burn_vote_quorum" => Ok(GeneratedField::BurnVoteQuorum),
                            "burnProposalDepositPrevote" | "burn_proposal_deposit_prevote" => Ok(GeneratedField::BurnProposalDepositPrevote),
                            "burnVoteVeto" | "burn_vote_veto" => Ok(GeneratedField::BurnVoteVeto),
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
                formatter.write_str("struct cosmos.gov.v1.Params")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Params, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut min_deposit__ = None;
                let mut max_deposit_period__ = None;
                let mut voting_period__ = None;
                let mut quorum__ = None;
                let mut threshold__ = None;
                let mut veto_threshold__ = None;
                let mut min_initial_deposit_ratio__ = None;
                let mut burn_vote_quorum__ = None;
                let mut burn_proposal_deposit_prevote__ = None;
                let mut burn_vote_veto__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MinDeposit => {
                            if min_deposit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minDeposit"));
                            }
                            min_deposit__ = Some(map.next_value()?);
                        }
                        GeneratedField::MaxDepositPeriod => {
                            if max_deposit_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxDepositPeriod"));
                            }
                            max_deposit_period__ = map.next_value()?;
                        }
                        GeneratedField::VotingPeriod => {
                            if voting_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPeriod"));
                            }
                            voting_period__ = map.next_value()?;
                        }
                        GeneratedField::Quorum => {
                            if quorum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("quorum"));
                            }
                            quorum__ = Some(map.next_value()?);
                        }
                        GeneratedField::Threshold => {
                            if threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("threshold"));
                            }
                            threshold__ = Some(map.next_value()?);
                        }
                        GeneratedField::VetoThreshold => {
                            if veto_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vetoThreshold"));
                            }
                            veto_threshold__ = Some(map.next_value()?);
                        }
                        GeneratedField::MinInitialDepositRatio => {
                            if min_initial_deposit_ratio__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minInitialDepositRatio"));
                            }
                            min_initial_deposit_ratio__ = Some(map.next_value()?);
                        }
                        GeneratedField::BurnVoteQuorum => {
                            if burn_vote_quorum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("burnVoteQuorum"));
                            }
                            burn_vote_quorum__ = Some(map.next_value()?);
                        }
                        GeneratedField::BurnProposalDepositPrevote => {
                            if burn_proposal_deposit_prevote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("burnProposalDepositPrevote"));
                            }
                            burn_proposal_deposit_prevote__ = Some(map.next_value()?);
                        }
                        GeneratedField::BurnVoteVeto => {
                            if burn_vote_veto__.is_some() {
                                return Err(serde::de::Error::duplicate_field("burnVoteVeto"));
                            }
                            burn_vote_veto__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Params {
                    min_deposit: min_deposit__.unwrap_or_default(),
                    max_deposit_period: max_deposit_period__,
                    voting_period: voting_period__,
                    quorum: quorum__.unwrap_or_default(),
                    threshold: threshold__.unwrap_or_default(),
                    veto_threshold: veto_threshold__.unwrap_or_default(),
                    min_initial_deposit_ratio: min_initial_deposit_ratio__.unwrap_or_default(),
                    burn_vote_quorum: burn_vote_quorum__.unwrap_or_default(),
                    burn_proposal_deposit_prevote: burn_proposal_deposit_prevote__.unwrap_or_default(),
                    burn_vote_veto: burn_vote_veto__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.Params", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Proposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.messages.is_empty() {
            len += 1;
        }
        if self.status != 0 {
            len += 1;
        }
        if self.final_tally_result.is_some() {
            len += 1;
        }
        if self.submit_time.is_some() {
            len += 1;
        }
        if self.deposit_end_time.is_some() {
            len += 1;
        }
        if !self.total_deposit.is_empty() {
            len += 1;
        }
        if self.voting_start_time.is_some() {
            len += 1;
        }
        if self.voting_end_time.is_some() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.summary.is_empty() {
            len += 1;
        }
        if !self.proposer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.Proposal", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", ToString::to_string(&self.id).as_str())?;
        }
        if !self.messages.is_empty() {
            struct_ser.serialize_field("messages", &self.messages)?;
        }
        if self.status != 0 {
            let v = ProposalStatus::from_i32(self.status)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.status)))?;
            struct_ser.serialize_field("status", &v)?;
        }
        if let Some(v) = self.final_tally_result.as_ref() {
            struct_ser.serialize_field("finalTallyResult", v)?;
        }
        if let Some(v) = self.submit_time.as_ref() {
            struct_ser.serialize_field("submitTime", v)?;
        }
        if let Some(v) = self.deposit_end_time.as_ref() {
            struct_ser.serialize_field("depositEndTime", v)?;
        }
        if !self.total_deposit.is_empty() {
            struct_ser.serialize_field("totalDeposit", &self.total_deposit)?;
        }
        if let Some(v) = self.voting_start_time.as_ref() {
            struct_ser.serialize_field("votingStartTime", v)?;
        }
        if let Some(v) = self.voting_end_time.as_ref() {
            struct_ser.serialize_field("votingEndTime", v)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.summary.is_empty() {
            struct_ser.serialize_field("summary", &self.summary)?;
        }
        if !self.proposer.is_empty() {
            struct_ser.serialize_field("proposer", &self.proposer)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Proposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "messages",
            "status",
            "final_tally_result",
            "finalTallyResult",
            "submit_time",
            "submitTime",
            "deposit_end_time",
            "depositEndTime",
            "total_deposit",
            "totalDeposit",
            "voting_start_time",
            "votingStartTime",
            "voting_end_time",
            "votingEndTime",
            "metadata",
            "title",
            "summary",
            "proposer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Messages,
            Status,
            FinalTallyResult,
            SubmitTime,
            DepositEndTime,
            TotalDeposit,
            VotingStartTime,
            VotingEndTime,
            Metadata,
            Title,
            Summary,
            Proposer,
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
                            "messages" => Ok(GeneratedField::Messages),
                            "status" => Ok(GeneratedField::Status),
                            "finalTallyResult" | "final_tally_result" => Ok(GeneratedField::FinalTallyResult),
                            "submitTime" | "submit_time" => Ok(GeneratedField::SubmitTime),
                            "depositEndTime" | "deposit_end_time" => Ok(GeneratedField::DepositEndTime),
                            "totalDeposit" | "total_deposit" => Ok(GeneratedField::TotalDeposit),
                            "votingStartTime" | "voting_start_time" => Ok(GeneratedField::VotingStartTime),
                            "votingEndTime" | "voting_end_time" => Ok(GeneratedField::VotingEndTime),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "title" => Ok(GeneratedField::Title),
                            "summary" => Ok(GeneratedField::Summary),
                            "proposer" => Ok(GeneratedField::Proposer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Proposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.Proposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Proposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut messages__ = None;
                let mut status__ = None;
                let mut final_tally_result__ = None;
                let mut submit_time__ = None;
                let mut deposit_end_time__ = None;
                let mut total_deposit__ = None;
                let mut voting_start_time__ = None;
                let mut voting_end_time__ = None;
                let mut metadata__ = None;
                let mut title__ = None;
                let mut summary__ = None;
                let mut proposer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Messages => {
                            if messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messages"));
                            }
                            messages__ = Some(map.next_value()?);
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map.next_value::<ProposalStatus>()? as i32);
                        }
                        GeneratedField::FinalTallyResult => {
                            if final_tally_result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finalTallyResult"));
                            }
                            final_tally_result__ = map.next_value()?;
                        }
                        GeneratedField::SubmitTime => {
                            if submit_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("submitTime"));
                            }
                            submit_time__ = map.next_value()?;
                        }
                        GeneratedField::DepositEndTime => {
                            if deposit_end_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositEndTime"));
                            }
                            deposit_end_time__ = map.next_value()?;
                        }
                        GeneratedField::TotalDeposit => {
                            if total_deposit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalDeposit"));
                            }
                            total_deposit__ = Some(map.next_value()?);
                        }
                        GeneratedField::VotingStartTime => {
                            if voting_start_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingStartTime"));
                            }
                            voting_start_time__ = map.next_value()?;
                        }
                        GeneratedField::VotingEndTime => {
                            if voting_end_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingEndTime"));
                            }
                            voting_end_time__ = map.next_value()?;
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map.next_value()?);
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proposer => {
                            if proposer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposer"));
                            }
                            proposer__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Proposal {
                    id: id__.unwrap_or_default(),
                    messages: messages__.unwrap_or_default(),
                    status: status__.unwrap_or_default(),
                    final_tally_result: final_tally_result__,
                    submit_time: submit_time__,
                    deposit_end_time: deposit_end_time__,
                    total_deposit: total_deposit__.unwrap_or_default(),
                    voting_start_time: voting_start_time__,
                    voting_end_time: voting_end_time__,
                    metadata: metadata__.unwrap_or_default(),
                    title: title__.unwrap_or_default(),
                    summary: summary__.unwrap_or_default(),
                    proposer: proposer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.Proposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PROPOSAL_STATUS_UNSPECIFIED",
            Self::DepositPeriod => "PROPOSAL_STATUS_DEPOSIT_PERIOD",
            Self::VotingPeriod => "PROPOSAL_STATUS_VOTING_PERIOD",
            Self::Passed => "PROPOSAL_STATUS_PASSED",
            Self::Rejected => "PROPOSAL_STATUS_REJECTED",
            Self::Failed => "PROPOSAL_STATUS_FAILED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ProposalStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PROPOSAL_STATUS_UNSPECIFIED",
            "PROPOSAL_STATUS_DEPOSIT_PERIOD",
            "PROPOSAL_STATUS_VOTING_PERIOD",
            "PROPOSAL_STATUS_PASSED",
            "PROPOSAL_STATUS_REJECTED",
            "PROPOSAL_STATUS_FAILED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalStatus;

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
                    .and_then(ProposalStatus::from_i32)
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
                    .and_then(ProposalStatus::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "PROPOSAL_STATUS_UNSPECIFIED" => Ok(ProposalStatus::Unspecified),
                    "PROPOSAL_STATUS_DEPOSIT_PERIOD" => Ok(ProposalStatus::DepositPeriod),
                    "PROPOSAL_STATUS_VOTING_PERIOD" => Ok(ProposalStatus::VotingPeriod),
                    "PROPOSAL_STATUS_PASSED" => Ok(ProposalStatus::Passed),
                    "PROPOSAL_STATUS_REJECTED" => Ok(ProposalStatus::Rejected),
                    "PROPOSAL_STATUS_FAILED" => Ok(ProposalStatus::Failed),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDepositRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.depositor.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryDepositRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.depositor.is_empty() {
            struct_ser.serialize_field("depositor", &self.depositor)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDepositRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "depositor",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
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
            type Value = QueryDepositRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryDepositRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDepositRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut depositor__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Depositor => {
                            if depositor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositor"));
                            }
                            depositor__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryDepositRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    depositor: depositor__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryDepositRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDepositResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.deposit.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryDepositResponse", len)?;
        if let Some(v) = self.deposit.as_ref() {
            struct_ser.serialize_field("deposit", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDepositResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "deposit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = QueryDepositResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryDepositResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDepositResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut deposit__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Deposit => {
                            if deposit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deposit"));
                            }
                            deposit__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDepositResponse {
                    deposit: deposit__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryDepositResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDepositsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryDepositsRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDepositsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
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
            type Value = QueryDepositsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryDepositsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDepositsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
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
                Ok(QueryDepositsRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryDepositsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryDepositsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.deposits.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryDepositsResponse", len)?;
        if !self.deposits.is_empty() {
            struct_ser.serialize_field("deposits", &self.deposits)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryDepositsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "deposits",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Deposits,
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
                            "deposits" => Ok(GeneratedField::Deposits),
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
            type Value = QueryDepositsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryDepositsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryDepositsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut deposits__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Deposits => {
                            if deposits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deposits"));
                            }
                            deposits__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryDepositsResponse {
                    deposits: deposits__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryDepositsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryParamsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.params_type.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryParamsRequest", len)?;
        if !self.params_type.is_empty() {
            struct_ser.serialize_field("paramsType", &self.params_type)?;
        }
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
            "params_type",
            "paramsType",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ParamsType,
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
                            "paramsType" | "params_type" => Ok(GeneratedField::ParamsType),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryParamsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryParamsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryParamsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut params_type__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ParamsType => {
                            if params_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("paramsType"));
                            }
                            params_type__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryParamsRequest {
                    params_type: params_type__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryParamsRequest", FIELDS, GeneratedVisitor)
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
        if self.voting_params.is_some() {
            len += 1;
        }
        if self.deposit_params.is_some() {
            len += 1;
        }
        if self.tally_params.is_some() {
            len += 1;
        }
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryParamsResponse", len)?;
        if let Some(v) = self.voting_params.as_ref() {
            struct_ser.serialize_field("votingParams", v)?;
        }
        if let Some(v) = self.deposit_params.as_ref() {
            struct_ser.serialize_field("depositParams", v)?;
        }
        if let Some(v) = self.tally_params.as_ref() {
            struct_ser.serialize_field("tallyParams", v)?;
        }
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
            "voting_params",
            "votingParams",
            "deposit_params",
            "depositParams",
            "tally_params",
            "tallyParams",
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VotingParams,
            DepositParams,
            TallyParams,
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
                            "votingParams" | "voting_params" => Ok(GeneratedField::VotingParams),
                            "depositParams" | "deposit_params" => Ok(GeneratedField::DepositParams),
                            "tallyParams" | "tally_params" => Ok(GeneratedField::TallyParams),
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
                formatter.write_str("struct cosmos.gov.v1.QueryParamsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryParamsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut voting_params__ = None;
                let mut deposit_params__ = None;
                let mut tally_params__ = None;
                let mut params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::VotingParams => {
                            if voting_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingParams"));
                            }
                            voting_params__ = map.next_value()?;
                        }
                        GeneratedField::DepositParams => {
                            if deposit_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositParams"));
                            }
                            deposit_params__ = map.next_value()?;
                        }
                        GeneratedField::TallyParams => {
                            if tally_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tallyParams"));
                            }
                            tally_params__ = map.next_value()?;
                        }
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryParamsResponse {
                    voting_params: voting_params__,
                    deposit_params: deposit_params__,
                    tally_params: tally_params__,
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryParamsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryProposalRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryProposalRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryProposalRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryProposalRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryProposalRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryProposalRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryProposalRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryProposalRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryProposalResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryProposalResponse", len)?;
        if let Some(v) = self.proposal.as_ref() {
            struct_ser.serialize_field("proposal", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryProposalResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryProposalResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryProposalResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryProposalResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryProposalResponse {
                    proposal: proposal__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryProposalResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryProposalsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_status != 0 {
            len += 1;
        }
        if !self.voter.is_empty() {
            len += 1;
        }
        if !self.depositor.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryProposalsRequest", len)?;
        if self.proposal_status != 0 {
            let v = ProposalStatus::from_i32(self.proposal_status)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.proposal_status)))?;
            struct_ser.serialize_field("proposalStatus", &v)?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        if !self.depositor.is_empty() {
            struct_ser.serialize_field("depositor", &self.depositor)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryProposalsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_status",
            "proposalStatus",
            "voter",
            "depositor",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalStatus,
            Voter,
            Depositor,
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
                            "proposalStatus" | "proposal_status" => Ok(GeneratedField::ProposalStatus),
                            "voter" => Ok(GeneratedField::Voter),
                            "depositor" => Ok(GeneratedField::Depositor),
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
            type Value = QueryProposalsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryProposalsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryProposalsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_status__ = None;
                let mut voter__ = None;
                let mut depositor__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalStatus => {
                            if proposal_status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalStatus"));
                            }
                            proposal_status__ = Some(map.next_value::<ProposalStatus>()? as i32);
                        }
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Depositor => {
                            if depositor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositor"));
                            }
                            depositor__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryProposalsRequest {
                    proposal_status: proposal_status__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    depositor: depositor__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryProposalsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryProposalsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.proposals.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryProposalsResponse", len)?;
        if !self.proposals.is_empty() {
            struct_ser.serialize_field("proposals", &self.proposals)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryProposalsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposals",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposals,
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
                            "proposals" => Ok(GeneratedField::Proposals),
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
            type Value = QueryProposalsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryProposalsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryProposalsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposals__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposals => {
                            if proposals__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposals"));
                            }
                            proposals__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryProposalsResponse {
                    proposals: proposals__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryProposalsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTallyResultRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryTallyResultRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTallyResultRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTallyResultRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryTallyResultRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTallyResultRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryTallyResultRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryTallyResultRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTallyResultResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tally.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryTallyResultResponse", len)?;
        if let Some(v) = self.tally.as_ref() {
            struct_ser.serialize_field("tally", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTallyResultResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tally",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tally,
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
                            "tally" => Ok(GeneratedField::Tally),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTallyResultResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryTallyResultResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTallyResultResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tally__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Tally => {
                            if tally__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tally"));
                            }
                            tally__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryTallyResultResponse {
                    tally: tally__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryTallyResultResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVoteRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.voter.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryVoteRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVoteRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "voter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "voter" => Ok(GeneratedField::Voter),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryVoteRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryVoteRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVoteRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryVoteRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryVoteRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVoteResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryVoteResponse", len)?;
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVoteResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vote,
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
                            "vote" => Ok(GeneratedField::Vote),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryVoteResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryVoteResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVoteResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryVoteResponse {
                    vote: vote__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryVoteResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVotesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryVotesRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVotesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
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
            type Value = QueryVotesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryVotesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVotesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
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
                Ok(QueryVotesRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryVotesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVotesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.votes.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.QueryVotesResponse", len)?;
        if !self.votes.is_empty() {
            struct_ser.serialize_field("votes", &self.votes)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVotesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "votes",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Votes,
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
                            "votes" => Ok(GeneratedField::Votes),
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
            type Value = QueryVotesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.QueryVotesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVotesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut votes__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Votes => {
                            if votes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votes"));
                            }
                            votes__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryVotesResponse {
                    votes: votes__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.QueryVotesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TallyParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.quorum.is_empty() {
            len += 1;
        }
        if !self.threshold.is_empty() {
            len += 1;
        }
        if !self.veto_threshold.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.TallyParams", len)?;
        if !self.quorum.is_empty() {
            struct_ser.serialize_field("quorum", &self.quorum)?;
        }
        if !self.threshold.is_empty() {
            struct_ser.serialize_field("threshold", &self.threshold)?;
        }
        if !self.veto_threshold.is_empty() {
            struct_ser.serialize_field("vetoThreshold", &self.veto_threshold)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TallyParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "quorum",
            "threshold",
            "veto_threshold",
            "vetoThreshold",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Quorum,
            Threshold,
            VetoThreshold,
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
                            "quorum" => Ok(GeneratedField::Quorum),
                            "threshold" => Ok(GeneratedField::Threshold),
                            "vetoThreshold" | "veto_threshold" => Ok(GeneratedField::VetoThreshold),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TallyParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.TallyParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TallyParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut quorum__ = None;
                let mut threshold__ = None;
                let mut veto_threshold__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Quorum => {
                            if quorum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("quorum"));
                            }
                            quorum__ = Some(map.next_value()?);
                        }
                        GeneratedField::Threshold => {
                            if threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("threshold"));
                            }
                            threshold__ = Some(map.next_value()?);
                        }
                        GeneratedField::VetoThreshold => {
                            if veto_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vetoThreshold"));
                            }
                            veto_threshold__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TallyParams {
                    quorum: quorum__.unwrap_or_default(),
                    threshold: threshold__.unwrap_or_default(),
                    veto_threshold: veto_threshold__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.TallyParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TallyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.yes_count.is_empty() {
            len += 1;
        }
        if !self.abstain_count.is_empty() {
            len += 1;
        }
        if !self.no_count.is_empty() {
            len += 1;
        }
        if !self.no_with_veto_count.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.TallyResult", len)?;
        if !self.yes_count.is_empty() {
            struct_ser.serialize_field("yesCount", &self.yes_count)?;
        }
        if !self.abstain_count.is_empty() {
            struct_ser.serialize_field("abstainCount", &self.abstain_count)?;
        }
        if !self.no_count.is_empty() {
            struct_ser.serialize_field("noCount", &self.no_count)?;
        }
        if !self.no_with_veto_count.is_empty() {
            struct_ser.serialize_field("noWithVetoCount", &self.no_with_veto_count)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TallyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "yes_count",
            "yesCount",
            "abstain_count",
            "abstainCount",
            "no_count",
            "noCount",
            "no_with_veto_count",
            "noWithVetoCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            YesCount,
            AbstainCount,
            NoCount,
            NoWithVetoCount,
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
                            "yesCount" | "yes_count" => Ok(GeneratedField::YesCount),
                            "abstainCount" | "abstain_count" => Ok(GeneratedField::AbstainCount),
                            "noCount" | "no_count" => Ok(GeneratedField::NoCount),
                            "noWithVetoCount" | "no_with_veto_count" => Ok(GeneratedField::NoWithVetoCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TallyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.TallyResult")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TallyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut yes_count__ = None;
                let mut abstain_count__ = None;
                let mut no_count__ = None;
                let mut no_with_veto_count__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::YesCount => {
                            if yes_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("yesCount"));
                            }
                            yes_count__ = Some(map.next_value()?);
                        }
                        GeneratedField::AbstainCount => {
                            if abstain_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("abstainCount"));
                            }
                            abstain_count__ = Some(map.next_value()?);
                        }
                        GeneratedField::NoCount => {
                            if no_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noCount"));
                            }
                            no_count__ = Some(map.next_value()?);
                        }
                        GeneratedField::NoWithVetoCount => {
                            if no_with_veto_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noWithVetoCount"));
                            }
                            no_with_veto_count__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TallyResult {
                    yes_count: yes_count__.unwrap_or_default(),
                    abstain_count: abstain_count__.unwrap_or_default(),
                    no_count: no_count__.unwrap_or_default(),
                    no_with_veto_count: no_with_veto_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.TallyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Vote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.voter.is_empty() {
            len += 1;
        }
        if !self.options.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.Vote", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        if !self.options.is_empty() {
            struct_ser.serialize_field("options", &self.options)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Vote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "voter",
            "options",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
            Options,
            Metadata,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "voter" => Ok(GeneratedField::Voter),
                            "options" => Ok(GeneratedField::Options),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Vote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.Vote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Vote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                let mut options__ = None;
                let mut metadata__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Options => {
                            if options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("options"));
                            }
                            options__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Vote {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    options: options__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.Vote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VoteOption {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "VOTE_OPTION_UNSPECIFIED",
            Self::Yes => "VOTE_OPTION_YES",
            Self::Abstain => "VOTE_OPTION_ABSTAIN",
            Self::No => "VOTE_OPTION_NO",
            Self::NoWithVeto => "VOTE_OPTION_NO_WITH_VETO",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for VoteOption {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "VOTE_OPTION_UNSPECIFIED",
            "VOTE_OPTION_YES",
            "VOTE_OPTION_ABSTAIN",
            "VOTE_OPTION_NO",
            "VOTE_OPTION_NO_WITH_VETO",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VoteOption;

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
                    .and_then(VoteOption::from_i32)
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
                    .and_then(VoteOption::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "VOTE_OPTION_UNSPECIFIED" => Ok(VoteOption::Unspecified),
                    "VOTE_OPTION_YES" => Ok(VoteOption::Yes),
                    "VOTE_OPTION_ABSTAIN" => Ok(VoteOption::Abstain),
                    "VOTE_OPTION_NO" => Ok(VoteOption::No),
                    "VOTE_OPTION_NO_WITH_VETO" => Ok(VoteOption::NoWithVeto),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for VotingParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.voting_period.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.VotingParams", len)?;
        if let Some(v) = self.voting_period.as_ref() {
            struct_ser.serialize_field("votingPeriod", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VotingParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "voting_period",
            "votingPeriod",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VotingPeriod,
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
                            "votingPeriod" | "voting_period" => Ok(GeneratedField::VotingPeriod),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VotingParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.VotingParams")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<VotingParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut voting_period__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::VotingPeriod => {
                            if voting_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPeriod"));
                            }
                            voting_period__ = map.next_value()?;
                        }
                    }
                }
                Ok(VotingParams {
                    voting_period: voting_period__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.VotingParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WeightedVoteOption {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.option != 0 {
            len += 1;
        }
        if !self.weight.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1.WeightedVoteOption", len)?;
        if self.option != 0 {
            let v = VoteOption::from_i32(self.option)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.option)))?;
            struct_ser.serialize_field("option", &v)?;
        }
        if !self.weight.is_empty() {
            struct_ser.serialize_field("weight", &self.weight)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WeightedVoteOption {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "option",
            "weight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Option,
            Weight,
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
                            "option" => Ok(GeneratedField::Option),
                            "weight" => Ok(GeneratedField::Weight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WeightedVoteOption;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1.WeightedVoteOption")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<WeightedVoteOption, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut option__ = None;
                let mut weight__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Option => {
                            if option__.is_some() {
                                return Err(serde::de::Error::duplicate_field("option"));
                            }
                            option__ = Some(map.next_value::<VoteOption>()? as i32);
                        }
                        GeneratedField::Weight => {
                            if weight__.is_some() {
                                return Err(serde::de::Error::duplicate_field("weight"));
                            }
                            weight__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(WeightedVoteOption {
                    option: option__.unwrap_or_default(),
                    weight: weight__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1.WeightedVoteOption", FIELDS, GeneratedVisitor)
    }
}
