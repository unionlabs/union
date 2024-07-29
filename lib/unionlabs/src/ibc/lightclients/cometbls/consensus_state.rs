use macros::model;

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::H256,
    ibc::core::commitment::merkle_root::{MerkleRoot, TryFromMerkleRootError},
};

#[model(
    proto(
        raw(protos::union::ibc::lightclients::cometbls::v1::ConsensusState),
        into,
        from
    ),
    ethabi(raw(contracts::glue::OptimizedConsensusState), into, from)
)]
pub struct ConsensusState {
    pub timestamp: u64,
    pub app_hash: MerkleRoot,
    pub next_validators_hash: H256,
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromConsensusStateError {
    #[error(transparent)]
    MissingField(MissingField),
    #[error("invalid root")]
    Root(#[from] TryFromMerkleRootError),
    #[error("invalid next validators hash")]
    NextValidatorsHash(#[from] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ConsensusState> for ConsensusState {
    type Error = TryFromConsensusStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: value.timestamp,
            app_hash: required!(value.root)?
                .try_into()
                .map_err(TryFromConsensusStateError::Root)?,
            next_validators_hash: value
                .next_validators_hash
                .try_into()
                .map_err(TryFromConsensusStateError::NextValidatorsHash)?,
        })
    }
}

impl From<ConsensusState> for protos::union::ibc::lightclients::cometbls::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            timestamp: value.timestamp,
            root: Some(value.app_hash.into()),
            next_validators_hash: value.next_validators_hash.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<ConsensusState> for contracts::glue::OptimizedConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            timestamp: value.timestamp,
            app_hash: value.app_hash.hash.into(),
            next_validators_hash: value.next_validators_hash.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::OptimizedConsensusState> for ConsensusState {
    type Error = TryFromConsensusStateError;

    fn try_from(value: contracts::glue::OptimizedConsensusState) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: value.timestamp,
            app_hash: MerkleRoot {
                hash: H256::from(value.app_hash),
            },
            next_validators_hash: value.next_validators_hash.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;
    use crate::encoding::{Bcs, DecodeAs, EncodeAs};

    #[test]
    fn bcs() {
        let consensus_state = ConsensusState {
            timestamp: 42,
            app_hash: MerkleRoot {
                hash: hex!("0000000000000000000000000000000000000000000000000000000000000000")
                    .into(),
            },
            next_validators_hash: hex!(
                "0000000000000000000000000000000000000000000000000000000000000000"
            )
            .into(),
        };

        let bz = consensus_state.encode_as::<Bcs>();

        dbg!(hex::encode(bz));

        let bz = hex!("2a00000000000000200000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000");

        let cs = ConsensusState::decode_as::<Bcs>(&bz).unwrap();

        dbg!(cs);
    }
}
