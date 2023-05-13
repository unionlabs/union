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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.Deposit", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.Deposit")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.Deposit", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.DepositParams", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.DepositParams")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.DepositParams", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.GenesisState", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.GenesisState")
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
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.GenesisState", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.MsgDeposit", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.MsgDeposit")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.MsgDeposit", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.MsgDepositResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.MsgDepositResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.MsgDepositResponse", FIELDS, GeneratedVisitor)
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
        if self.content.is_some() {
            len += 1;
        }
        if !self.initial_deposit.is_empty() {
            len += 1;
        }
        if !self.proposer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.MsgSubmitProposal", len)?;
        if let Some(v) = self.content.as_ref() {
            struct_ser.serialize_field("content", v)?;
        }
        if !self.initial_deposit.is_empty() {
            struct_ser.serialize_field("initialDeposit", &self.initial_deposit)?;
        }
        if !self.proposer.is_empty() {
            struct_ser.serialize_field("proposer", &self.proposer)?;
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
            "content",
            "initial_deposit",
            "initialDeposit",
            "proposer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Content,
            InitialDeposit,
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
                            "content" => Ok(GeneratedField::Content),
                            "initialDeposit" | "initial_deposit" => Ok(GeneratedField::InitialDeposit),
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
            type Value = MsgSubmitProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1beta1.MsgSubmitProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgSubmitProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut content__ = None;
                let mut initial_deposit__ = None;
                let mut proposer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Content => {
                            if content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("content"));
                            }
                            content__ = map.next_value()?;
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
                    }
                }
                Ok(MsgSubmitProposal {
                    content: content__,
                    initial_deposit: initial_deposit__.unwrap_or_default(),
                    proposer: proposer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.MsgSubmitProposal", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.MsgSubmitProposalResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.MsgSubmitProposalResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.MsgSubmitProposalResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.MsgVote", len)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
            Option,
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
                formatter.write_str("struct cosmos.gov.v1beta1.MsgVote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                let mut option__ = None;
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
                    }
                }
                Ok(MsgVote {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    option: option__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.MsgVote", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.MsgVoteResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.MsgVoteResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.MsgVoteResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.MsgVoteWeighted", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        if !self.options.is_empty() {
            struct_ser.serialize_field("options", &self.options)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
            Options,
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
                formatter.write_str("struct cosmos.gov.v1beta1.MsgVoteWeighted")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgVoteWeighted, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                let mut options__ = None;
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
                    }
                }
                Ok(MsgVoteWeighted {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    options: options__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.MsgVoteWeighted", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.MsgVoteWeightedResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.MsgVoteWeightedResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.MsgVoteWeightedResponse", FIELDS, GeneratedVisitor)
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
        if self.proposal_id != 0 {
            len += 1;
        }
        if self.content.is_some() {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.Proposal", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if let Some(v) = self.content.as_ref() {
            struct_ser.serialize_field("content", v)?;
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
            "proposal_id",
            "proposalId",
            "content",
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Content,
            Status,
            FinalTallyResult,
            SubmitTime,
            DepositEndTime,
            TotalDeposit,
            VotingStartTime,
            VotingEndTime,
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
                            "content" => Ok(GeneratedField::Content),
                            "status" => Ok(GeneratedField::Status),
                            "finalTallyResult" | "final_tally_result" => Ok(GeneratedField::FinalTallyResult),
                            "submitTime" | "submit_time" => Ok(GeneratedField::SubmitTime),
                            "depositEndTime" | "deposit_end_time" => Ok(GeneratedField::DepositEndTime),
                            "totalDeposit" | "total_deposit" => Ok(GeneratedField::TotalDeposit),
                            "votingStartTime" | "voting_start_time" => Ok(GeneratedField::VotingStartTime),
                            "votingEndTime" | "voting_end_time" => Ok(GeneratedField::VotingEndTime),
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
                formatter.write_str("struct cosmos.gov.v1beta1.Proposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Proposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut content__ = None;
                let mut status__ = None;
                let mut final_tally_result__ = None;
                let mut submit_time__ = None;
                let mut deposit_end_time__ = None;
                let mut total_deposit__ = None;
                let mut voting_start_time__ = None;
                let mut voting_end_time__ = None;
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
                        GeneratedField::Content => {
                            if content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("content"));
                            }
                            content__ = map.next_value()?;
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
                    }
                }
                Ok(Proposal {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    content: content__,
                    status: status__.unwrap_or_default(),
                    final_tally_result: final_tally_result__,
                    submit_time: submit_time__,
                    deposit_end_time: deposit_end_time__,
                    total_deposit: total_deposit__.unwrap_or_default(),
                    voting_start_time: voting_start_time__,
                    voting_end_time: voting_end_time__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.Proposal", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryDepositRequest", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryDepositRequest")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryDepositRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryDepositResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryDepositResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryDepositResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryDepositsRequest", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryDepositsRequest")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryDepositsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryDepositsResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryDepositsResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryDepositsResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryParamsRequest", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryParamsRequest")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryParamsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryParamsResponse", len)?;
        if let Some(v) = self.voting_params.as_ref() {
            struct_ser.serialize_field("votingParams", v)?;
        }
        if let Some(v) = self.deposit_params.as_ref() {
            struct_ser.serialize_field("depositParams", v)?;
        }
        if let Some(v) = self.tally_params.as_ref() {
            struct_ser.serialize_field("tallyParams", v)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VotingParams,
            DepositParams,
            TallyParams,
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryParamsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryParamsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut voting_params__ = None;
                let mut deposit_params__ = None;
                let mut tally_params__ = None;
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
                    }
                }
                Ok(QueryParamsResponse {
                    voting_params: voting_params__,
                    deposit_params: deposit_params__,
                    tally_params: tally_params__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryParamsResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryProposalRequest", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryProposalRequest")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryProposalRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryProposalResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryProposalResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryProposalResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryProposalsRequest", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryProposalsRequest")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryProposalsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryProposalsResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryProposalsResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryProposalsResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryTallyResultRequest", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryTallyResultRequest")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryTallyResultRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryTallyResultResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryTallyResultResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryTallyResultResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryVoteRequest", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryVoteRequest")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryVoteRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryVoteResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryVoteResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryVoteResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryVotesRequest", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryVotesRequest")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryVotesRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.QueryVotesResponse", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.QueryVotesResponse")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.QueryVotesResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.TallyParams", len)?;
        if !self.quorum.is_empty() {
            struct_ser.serialize_field("quorum", pbjson::private::base64::encode(&self.quorum).as_str())?;
        }
        if !self.threshold.is_empty() {
            struct_ser.serialize_field("threshold", pbjson::private::base64::encode(&self.threshold).as_str())?;
        }
        if !self.veto_threshold.is_empty() {
            struct_ser.serialize_field("vetoThreshold", pbjson::private::base64::encode(&self.veto_threshold).as_str())?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.TallyParams")
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
                            quorum__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Threshold => {
                            if threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("threshold"));
                            }
                            threshold__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VetoThreshold => {
                            if veto_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vetoThreshold"));
                            }
                            veto_threshold__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.TallyParams", FIELDS, GeneratedVisitor)
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
        if !self.yes.is_empty() {
            len += 1;
        }
        if !self.abstain.is_empty() {
            len += 1;
        }
        if !self.no.is_empty() {
            len += 1;
        }
        if !self.no_with_veto.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.TallyResult", len)?;
        if !self.yes.is_empty() {
            struct_ser.serialize_field("yes", &self.yes)?;
        }
        if !self.abstain.is_empty() {
            struct_ser.serialize_field("abstain", &self.abstain)?;
        }
        if !self.no.is_empty() {
            struct_ser.serialize_field("no", &self.no)?;
        }
        if !self.no_with_veto.is_empty() {
            struct_ser.serialize_field("noWithVeto", &self.no_with_veto)?;
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
            "yes",
            "abstain",
            "no",
            "no_with_veto",
            "noWithVeto",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Yes,
            Abstain,
            No,
            NoWithVeto,
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
                            "yes" => Ok(GeneratedField::Yes),
                            "abstain" => Ok(GeneratedField::Abstain),
                            "no" => Ok(GeneratedField::No),
                            "noWithVeto" | "no_with_veto" => Ok(GeneratedField::NoWithVeto),
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
                formatter.write_str("struct cosmos.gov.v1beta1.TallyResult")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TallyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut yes__ = None;
                let mut abstain__ = None;
                let mut no__ = None;
                let mut no_with_veto__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Yes => {
                            if yes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("yes"));
                            }
                            yes__ = Some(map.next_value()?);
                        }
                        GeneratedField::Abstain => {
                            if abstain__.is_some() {
                                return Err(serde::de::Error::duplicate_field("abstain"));
                            }
                            abstain__ = Some(map.next_value()?);
                        }
                        GeneratedField::No => {
                            if no__.is_some() {
                                return Err(serde::de::Error::duplicate_field("no"));
                            }
                            no__ = Some(map.next_value()?);
                        }
                        GeneratedField::NoWithVeto => {
                            if no_with_veto__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noWithVeto"));
                            }
                            no_with_veto__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TallyResult {
                    yes: yes__.unwrap_or_default(),
                    abstain: abstain__.unwrap_or_default(),
                    no: no__.unwrap_or_default(),
                    no_with_veto: no_with_veto__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.TallyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TextProposal {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.TextProposal", len)?;
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TextProposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "title",
            "description",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Title,
            Description,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TextProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.gov.v1beta1.TextProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TextProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut title__ = None;
                let mut description__ = None;
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
                    }
                }
                Ok(TextProposal {
                    title: title__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.TextProposal", FIELDS, GeneratedVisitor)
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
        if self.option != 0 {
            len += 1;
        }
        if !self.options.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.Vote", len)?;
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
        if !self.options.is_empty() {
            struct_ser.serialize_field("options", &self.options)?;
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
            "option",
            "options",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
            Option,
            Options,
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
                            "options" => Ok(GeneratedField::Options),
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
                formatter.write_str("struct cosmos.gov.v1beta1.Vote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Vote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                let mut option__ = None;
                let mut options__ = None;
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
                        GeneratedField::Options => {
                            if options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("options"));
                            }
                            options__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Vote {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    option: option__.unwrap_or_default(),
                    options: options__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.gov.v1beta1.Vote", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.VotingParams", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.VotingParams")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.VotingParams", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("cosmos.gov.v1beta1.WeightedVoteOption", len)?;
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
                formatter.write_str("struct cosmos.gov.v1beta1.WeightedVoteOption")
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
        deserializer.deserialize_struct("cosmos.gov.v1beta1.WeightedVoteOption", FIELDS, GeneratedVisitor)
    }
}
