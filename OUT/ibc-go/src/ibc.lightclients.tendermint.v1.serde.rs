// @generated
impl serde::Serialize for ClientState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.chain_id.is_empty() {
            len += 1;
        }
        if self.trust_level.is_some() {
            len += 1;
        }
        if self.trusting_period.is_some() {
            len += 1;
        }
        if self.unbonding_period.is_some() {
            len += 1;
        }
        if self.max_clock_drift.is_some() {
            len += 1;
        }
        if self.frozen_height.is_some() {
            len += 1;
        }
        if self.latest_height.is_some() {
            len += 1;
        }
        if !self.proof_specs.is_empty() {
            len += 1;
        }
        if !self.upgrade_path.is_empty() {
            len += 1;
        }
        if self.allow_update_after_expiry {
            len += 1;
        }
        if self.allow_update_after_misbehaviour {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.tendermint.v1.ClientState", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.trust_level.as_ref() {
            struct_ser.serialize_field("trustLevel", v)?;
        }
        if let Some(v) = self.trusting_period.as_ref() {
            struct_ser.serialize_field("trustingPeriod", v)?;
        }
        if let Some(v) = self.unbonding_period.as_ref() {
            struct_ser.serialize_field("unbondingPeriod", v)?;
        }
        if let Some(v) = self.max_clock_drift.as_ref() {
            struct_ser.serialize_field("maxClockDrift", v)?;
        }
        if let Some(v) = self.frozen_height.as_ref() {
            struct_ser.serialize_field("frozenHeight", v)?;
        }
        if let Some(v) = self.latest_height.as_ref() {
            struct_ser.serialize_field("latestHeight", v)?;
        }
        if !self.proof_specs.is_empty() {
            struct_ser.serialize_field("proofSpecs", &self.proof_specs)?;
        }
        if !self.upgrade_path.is_empty() {
            struct_ser.serialize_field("upgradePath", &self.upgrade_path)?;
        }
        if self.allow_update_after_expiry {
            struct_ser.serialize_field("allowUpdateAfterExpiry", &self.allow_update_after_expiry)?;
        }
        if self.allow_update_after_misbehaviour {
            struct_ser.serialize_field("allowUpdateAfterMisbehaviour", &self.allow_update_after_misbehaviour)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "trust_level",
            "trustLevel",
            "trusting_period",
            "trustingPeriod",
            "unbonding_period",
            "unbondingPeriod",
            "max_clock_drift",
            "maxClockDrift",
            "frozen_height",
            "frozenHeight",
            "latest_height",
            "latestHeight",
            "proof_specs",
            "proofSpecs",
            "upgrade_path",
            "upgradePath",
            "allow_update_after_expiry",
            "allowUpdateAfterExpiry",
            "allow_update_after_misbehaviour",
            "allowUpdateAfterMisbehaviour",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            TrustLevel,
            TrustingPeriod,
            UnbondingPeriod,
            MaxClockDrift,
            FrozenHeight,
            LatestHeight,
            ProofSpecs,
            UpgradePath,
            AllowUpdateAfterExpiry,
            AllowUpdateAfterMisbehaviour,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "trustLevel" | "trust_level" => Ok(GeneratedField::TrustLevel),
                            "trustingPeriod" | "trusting_period" => Ok(GeneratedField::TrustingPeriod),
                            "unbondingPeriod" | "unbonding_period" => Ok(GeneratedField::UnbondingPeriod),
                            "maxClockDrift" | "max_clock_drift" => Ok(GeneratedField::MaxClockDrift),
                            "frozenHeight" | "frozen_height" => Ok(GeneratedField::FrozenHeight),
                            "latestHeight" | "latest_height" => Ok(GeneratedField::LatestHeight),
                            "proofSpecs" | "proof_specs" => Ok(GeneratedField::ProofSpecs),
                            "upgradePath" | "upgrade_path" => Ok(GeneratedField::UpgradePath),
                            "allowUpdateAfterExpiry" | "allow_update_after_expiry" => Ok(GeneratedField::AllowUpdateAfterExpiry),
                            "allowUpdateAfterMisbehaviour" | "allow_update_after_misbehaviour" => Ok(GeneratedField::AllowUpdateAfterMisbehaviour),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.tendermint.v1.ClientState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ClientState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut trust_level__ = None;
                let mut trusting_period__ = None;
                let mut unbonding_period__ = None;
                let mut max_clock_drift__ = None;
                let mut frozen_height__ = None;
                let mut latest_height__ = None;
                let mut proof_specs__ = None;
                let mut upgrade_path__ = None;
                let mut allow_update_after_expiry__ = None;
                let mut allow_update_after_misbehaviour__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::TrustLevel => {
                            if trust_level__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trustLevel"));
                            }
                            trust_level__ = map.next_value()?;
                        }
                        GeneratedField::TrustingPeriod => {
                            if trusting_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trustingPeriod"));
                            }
                            trusting_period__ = map.next_value()?;
                        }
                        GeneratedField::UnbondingPeriod => {
                            if unbonding_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingPeriod"));
                            }
                            unbonding_period__ = map.next_value()?;
                        }
                        GeneratedField::MaxClockDrift => {
                            if max_clock_drift__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxClockDrift"));
                            }
                            max_clock_drift__ = map.next_value()?;
                        }
                        GeneratedField::FrozenHeight => {
                            if frozen_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("frozenHeight"));
                            }
                            frozen_height__ = map.next_value()?;
                        }
                        GeneratedField::LatestHeight => {
                            if latest_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("latestHeight"));
                            }
                            latest_height__ = map.next_value()?;
                        }
                        GeneratedField::ProofSpecs => {
                            if proof_specs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofSpecs"));
                            }
                            proof_specs__ = Some(map.next_value()?);
                        }
                        GeneratedField::UpgradePath => {
                            if upgrade_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("upgradePath"));
                            }
                            upgrade_path__ = Some(map.next_value()?);
                        }
                        GeneratedField::AllowUpdateAfterExpiry => {
                            if allow_update_after_expiry__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowUpdateAfterExpiry"));
                            }
                            allow_update_after_expiry__ = Some(map.next_value()?);
                        }
                        GeneratedField::AllowUpdateAfterMisbehaviour => {
                            if allow_update_after_misbehaviour__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowUpdateAfterMisbehaviour"));
                            }
                            allow_update_after_misbehaviour__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ClientState {
                    chain_id: chain_id__.unwrap_or_default(),
                    trust_level: trust_level__,
                    trusting_period: trusting_period__,
                    unbonding_period: unbonding_period__,
                    max_clock_drift: max_clock_drift__,
                    frozen_height: frozen_height__,
                    latest_height: latest_height__,
                    proof_specs: proof_specs__.unwrap_or_default(),
                    upgrade_path: upgrade_path__.unwrap_or_default(),
                    allow_update_after_expiry: allow_update_after_expiry__.unwrap_or_default(),
                    allow_update_after_misbehaviour: allow_update_after_misbehaviour__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.tendermint.v1.ClientState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConsensusState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timestamp.is_some() {
            len += 1;
        }
        if self.root.is_some() {
            len += 1;
        }
        if !self.next_validators_hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.tendermint.v1.ConsensusState", len)?;
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        if let Some(v) = self.root.as_ref() {
            struct_ser.serialize_field("root", v)?;
        }
        if !self.next_validators_hash.is_empty() {
            struct_ser.serialize_field("nextValidatorsHash", pbjson::private::base64::encode(&self.next_validators_hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConsensusState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timestamp",
            "root",
            "next_validators_hash",
            "nextValidatorsHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timestamp,
            Root,
            NextValidatorsHash,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "root" => Ok(GeneratedField::Root),
                            "nextValidatorsHash" | "next_validators_hash" => Ok(GeneratedField::NextValidatorsHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConsensusState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.tendermint.v1.ConsensusState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConsensusState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timestamp__ = None;
                let mut root__ = None;
                let mut next_validators_hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map.next_value()?;
                        }
                        GeneratedField::Root => {
                            if root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("root"));
                            }
                            root__ = map.next_value()?;
                        }
                        GeneratedField::NextValidatorsHash => {
                            if next_validators_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextValidatorsHash"));
                            }
                            next_validators_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ConsensusState {
                    timestamp: timestamp__,
                    root: root__,
                    next_validators_hash: next_validators_hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.tendermint.v1.ConsensusState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Fraction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.numerator != 0 {
            len += 1;
        }
        if self.denominator != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.tendermint.v1.Fraction", len)?;
        if self.numerator != 0 {
            struct_ser.serialize_field("numerator", ToString::to_string(&self.numerator).as_str())?;
        }
        if self.denominator != 0 {
            struct_ser.serialize_field("denominator", ToString::to_string(&self.denominator).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Fraction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "numerator",
            "denominator",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Numerator,
            Denominator,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
                            "numerator" => Ok(GeneratedField::Numerator),
                            "denominator" => Ok(GeneratedField::Denominator),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Fraction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.tendermint.v1.Fraction")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Fraction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut numerator__ = None;
                let mut denominator__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Numerator => {
                            if numerator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("numerator"));
                            }
                            numerator__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Denominator => {
                            if denominator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denominator"));
                            }
                            denominator__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Fraction {
                    numerator: numerator__.unwrap_or_default(),
                    denominator: denominator__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.tendermint.v1.Fraction", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Header {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.signed_header.is_some() {
            len += 1;
        }
        if self.validator_set.is_some() {
            len += 1;
        }
        if self.trusted_height.is_some() {
            len += 1;
        }
        if self.trusted_validators.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.tendermint.v1.Header", len)?;
        if let Some(v) = self.signed_header.as_ref() {
            struct_ser.serialize_field("signedHeader", v)?;
        }
        if let Some(v) = self.validator_set.as_ref() {
            struct_ser.serialize_field("validatorSet", v)?;
        }
        if let Some(v) = self.trusted_height.as_ref() {
            struct_ser.serialize_field("trustedHeight", v)?;
        }
        if let Some(v) = self.trusted_validators.as_ref() {
            struct_ser.serialize_field("trustedValidators", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Header {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signed_header",
            "signedHeader",
            "validator_set",
            "validatorSet",
            "trusted_height",
            "trustedHeight",
            "trusted_validators",
            "trustedValidators",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignedHeader,
            ValidatorSet,
            TrustedHeight,
            TrustedValidators,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
                            "signedHeader" | "signed_header" => Ok(GeneratedField::SignedHeader),
                            "validatorSet" | "validator_set" => Ok(GeneratedField::ValidatorSet),
                            "trustedHeight" | "trusted_height" => Ok(GeneratedField::TrustedHeight),
                            "trustedValidators" | "trusted_validators" => Ok(GeneratedField::TrustedValidators),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Header;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.tendermint.v1.Header")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Header, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signed_header__ = None;
                let mut validator_set__ = None;
                let mut trusted_height__ = None;
                let mut trusted_validators__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SignedHeader => {
                            if signed_header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signedHeader"));
                            }
                            signed_header__ = map.next_value()?;
                        }
                        GeneratedField::ValidatorSet => {
                            if validator_set__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorSet"));
                            }
                            validator_set__ = map.next_value()?;
                        }
                        GeneratedField::TrustedHeight => {
                            if trusted_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trustedHeight"));
                            }
                            trusted_height__ = map.next_value()?;
                        }
                        GeneratedField::TrustedValidators => {
                            if trusted_validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trustedValidators"));
                            }
                            trusted_validators__ = map.next_value()?;
                        }
                    }
                }
                Ok(Header {
                    signed_header: signed_header__,
                    validator_set: validator_set__,
                    trusted_height: trusted_height__,
                    trusted_validators: trusted_validators__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.tendermint.v1.Header", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Misbehaviour {
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
        if self.header_1.is_some() {
            len += 1;
        }
        if self.header_2.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.tendermint.v1.Misbehaviour", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if let Some(v) = self.header_1.as_ref() {
            struct_ser.serialize_field("header1", v)?;
        }
        if let Some(v) = self.header_2.as_ref() {
            struct_ser.serialize_field("header2", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Misbehaviour {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
            "header_1",
            "header1",
            "header_2",
            "header2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            Header1,
            Header2,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
                            "header1" | "header_1" => Ok(GeneratedField::Header1),
                            "header2" | "header_2" => Ok(GeneratedField::Header2),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Misbehaviour;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ibc.lightclients.tendermint.v1.Misbehaviour")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Misbehaviour, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut header_1__ = None;
                let mut header_2__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Header1 => {
                            if header_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("header1"));
                            }
                            header_1__ = map.next_value()?;
                        }
                        GeneratedField::Header2 => {
                            if header_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("header2"));
                            }
                            header_2__ = map.next_value()?;
                        }
                    }
                }
                Ok(Misbehaviour {
                    client_id: client_id__.unwrap_or_default(),
                    header_1: header_1__,
                    header_2: header_2__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.tendermint.v1.Misbehaviour", FIELDS, GeneratedVisitor)
    }
}
