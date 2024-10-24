use serde::{Deserialize, Serialize};
use unionlabs::{bounded::BoundedI64, hash::H160};

use crate::crypto::public_key::PublicKey;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Validator {
    pub address: H160,
    pub pub_key: PublicKey,
    pub voting_power: BoundedI64<0, { i64::MAX }>,
    pub proposer_priority: i64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        bounded::BoundedIntError,
        errors::{InvalidLength, MissingField},
        required,
    };

    use crate::{crypto::public_key, types::validator::Validator};

    impl From<Validator> for protos::tendermint::types::Validator {
        fn from(value: Validator) -> Self {
            Self {
                address: value.address.into(),
                pub_key: Some(value.pub_key.into()),
                voting_power: value.voting_power.into(),
                proposer_priority: value.proposer_priority,
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid address")]
        Address(#[source] InvalidLength),
        #[error("invalid voting power")]
        VotingPower(#[source] BoundedIntError<i64>),
        #[error("invalid pubkey")]
        PubKey(#[source] public_key::proto::Error),
    }

    impl TryFrom<protos::tendermint::types::Validator> for Validator {
        type Error = Error;

        fn try_from(value: protos::tendermint::types::Validator) -> Result<Self, Self::Error> {
            Ok(Self {
                address: value.address.try_into().map_err(Error::Address)?,
                pub_key: required!(value.pub_key)?
                    .try_into()
                    .map_err(Error::PubKey)?,
                voting_power: value.voting_power.try_into().map_err(Error::VotingPower)?,
                proposer_priority: value.proposer_priority,
            })
        }
    }
}
