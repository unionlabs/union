use macros::model;

use crate::{
    cosmos::{
        crypto::AnyPubKey,
        staking::{bond_status::BondStatus, commission::Commission, description::Description},
    },
    google::protobuf::timestamp::Timestamp,
};

#[model(proto(raw(protos::cosmos::staking::v1beta1::Validator), into, from))]
pub struct Validator {
    /// `operator_address` defines the address of the validator's operator; bech encoded in JSON.
    pub operator_address: String,
    /// `consensus_pubkey` is the consensus public key of the validator, as a Protobuf Any.
    pub consensus_pubkey: AnyPubKey,
    /// jailed defined whether the validator has been jailed from bonded status or not.
    pub jailed: bool,
    /// status is the validator status (bonded/unbonding/unbonded).
    pub status: BondStatus,
    /// tokens define the delegated tokens (incl. self-delegation).
    // FIXME: This is actually cosmos.Int
    pub tokens: String,
    /// `delegator_shares` defines total shares issued to a validator's delegators.
    // FIXME: This is actually cosmos.Dec
    pub delegator_shares: String,
    /// `description` defines the description terms for the validator.
    pub description: Description,
    /// `unbonding_height` defines, if unbonding, the height at which this validator has begun unbonding.
    pub unbonding_height: u32,
    /// `unbonding_time` defines, if unbonding, the min time for the validator to complete unbonding.
    pub unbonding_time: Timestamp,
    /// `commission` defines the commission parameters.
    pub commission: Commission,
    /// `min_self_delegation` is the validator's self declared minimum self delegation.
    ///
    /// Since: cosmos-sdk 0.46
    // FIXME: This is actually cosmos.Int
    pub min_self_delegation: String,
    /// strictly positive if this validator's unbonding has been stopped by external modules
    pub unbonding_on_hold_ref_count: i64,
    /// list of unbonding ids, each uniquely identifing an unbonding of this validator
    pub unbonding_ids: Vec<u64>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::num::TryFromIntError;

    use crate::{
        cosmos::{
            crypto::proto::TryFromAnyPubKeyError,
            staking::{commission::proto::TryFromCommissionError, validator::Validator},
        },
        errors::{required, MissingField, UnknownEnumVariant},
        google::protobuf::timestamp::proto::TryFromTimestampError,
    };

    #[derive(Debug)]
    pub enum TryFromValidatorError {
        MissingField(MissingField),
        ConsensusPubKey(TryFromAnyPubKeyError),
        BondStatus(UnknownEnumVariant<i32>),
        UnbondingHeight(TryFromIntError),
        Commission(TryFromCommissionError),
        Timestamp(TryFromTimestampError),
    }

    impl TryFrom<protos::cosmos::staking::v1beta1::Validator> for Validator {
        type Error = TryFromValidatorError;

        fn try_from(
            value: protos::cosmos::staking::v1beta1::Validator,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                operator_address: value.operator_address,
                consensus_pubkey: required!(value.consensus_pubkey)?
                    .try_into()
                    .map_err(TryFromValidatorError::ConsensusPubKey)?,
                jailed: value.jailed,
                status: value
                    .status
                    .try_into()
                    .map_err(TryFromValidatorError::BondStatus)?,
                tokens: value.tokens,
                delegator_shares: value.delegator_shares,
                description: required!(value.description)?.into(),
                unbonding_height: value
                    .unbonding_height
                    .try_into()
                    .map_err(TryFromValidatorError::UnbondingHeight)?,
                unbonding_time: required!(value.unbonding_time)?
                    .try_into()
                    .map_err(TryFromValidatorError::Timestamp)?,
                commission: required!(value.commission)?
                    .try_into()
                    .map_err(TryFromValidatorError::Commission)?,
                min_self_delegation: value.min_self_delegation,
                unbonding_on_hold_ref_count: value.unbonding_on_hold_ref_count,
                unbonding_ids: value.unbonding_ids,
            })
        }
    }

    impl From<Validator> for protos::cosmos::staking::v1beta1::Validator {
        fn from(value: Validator) -> Self {
            Self {
                operator_address: value.operator_address,
                consensus_pubkey: Some(value.consensus_pubkey.into()),
                jailed: value.jailed,
                status: value.status.into(),
                tokens: value.tokens,
                delegator_shares: value.delegator_shares,
                description: Some(value.description.into()),
                unbonding_height: value.unbonding_height.into(),
                unbonding_time: Some(value.unbonding_time.into()),
                commission: Some(value.commission.into()),
                min_self_delegation: value.min_self_delegation,
                unbonding_on_hold_ref_count: value.unbonding_on_hold_ref_count,
                unbonding_ids: value.unbonding_ids,
            }
        }
    }
}
