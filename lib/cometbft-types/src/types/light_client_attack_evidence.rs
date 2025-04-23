use serde::{Deserialize, Serialize};
use unionlabs::{bounded::BoundedI64, google::protobuf::timestamp::Timestamp};

use crate::types::{light_block::LightBlock, validator::Validator};

// TODO: Figure out serde for this type, I'm pretty sure this isn't quite correct
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightClientAttackEvidence {
    pub conflicting_block: LightBlock,
    pub common_height: BoundedI64<1, { i64::MAX }>,
    pub byzantine_validators: Vec<Validator>,
    pub total_voting_power: i64,
    pub timestamp: Timestamp,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        bounded::BoundedIntError, errors::MissingField,
        google::protobuf::timestamp::TryFromTimestampError, required,
    };

    use crate::types::{
        light_block, light_client_attack_evidence::LightClientAttackEvidence, validator,
    };

    impl From<LightClientAttackEvidence> for protos::cometbft::types::v1::LightClientAttackEvidence {
        fn from(value: LightClientAttackEvidence) -> Self {
            Self {
                conflicting_block: Some(value.conflicting_block.into()),
                common_height: value.common_height.into(),
                byzantine_validators: value
                    .byzantine_validators
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                total_voting_power: value.total_voting_power,
                timestamp: Some(value.timestamp.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid conflicting block")]
        ConflictingBlock(#[from] light_block::proto::Error),
        #[error("invalid common height")]
        CommonHeight(#[from] BoundedIntError<i64>),
        #[error("invalid byzantine validators")]
        ByzantineValidators(#[from] validator::proto::Error),
        #[error("invalid timestamp")]
        ValidatorSet(#[from] TryFromTimestampError),
    }

    impl TryFrom<protos::cometbft::types::v1::LightClientAttackEvidence> for LightClientAttackEvidence {
        type Error = Error;

        fn try_from(
            value: protos::cometbft::types::v1::LightClientAttackEvidence,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                conflicting_block: required!(value.conflicting_block)?.try_into()?,
                common_height: value.common_height.try_into()?,
                byzantine_validators: value
                    .byzantine_validators
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                total_voting_power: value.total_voting_power,
                timestamp: required!(value.timestamp)?.try_into()?,
            })
        }
    }
}
