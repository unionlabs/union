use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, InvalidLength, MissingField},
    ethereum::H256,
    ibc::{core::commitment::merkle_root::MerkleRoot, google::protobuf::timestamp::Timestamp},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusState {
    pub timestamp: Timestamp,
    pub root: MerkleRoot,
    pub next_validators_hash: H256,
}

#[derive(Debug)]
pub enum TryFromConsensusStateError {
    MissingField(MissingField),
    Root(TryFromProtoErrorOf<MerkleRoot>),
    NextValidatorsHash(InvalidLength),
    Timestamp(TryFromProtoErrorOf<Timestamp>),
}

impl TryFrom<protos::ibc::lightclients::tendermint::v1::ConsensusState> for ConsensusState {
    type Error = TryFromConsensusStateError;

    fn try_from(
        value: protos::ibc::lightclients::tendermint::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: required!(value.timestamp)?
                .try_into()
                .map_err(TryFromConsensusStateError::Timestamp)?,
            root: required!(value.root)?
                .try_into()
                .map_err(TryFromConsensusStateError::Root)?,
            next_validators_hash: value
                .next_validators_hash
                .try_into()
                .map_err(TryFromConsensusStateError::NextValidatorsHash)?,
        })
    }
}

impl TypeUrl for protos::ibc::lightclients::tendermint::v1::ConsensusState {
    const TYPE_URL: &'static str = "/cosmos.ibc.lightclients.tendermint.v1.ConsensusState";
}

impl Proto for ConsensusState {
    type Proto = protos::ibc::lightclients::tendermint::v1::ConsensusState;
}

impl From<ConsensusState> for protos::ibc::lightclients::tendermint::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            timestamp: Some(value.timestamp.into()),
            root: Some(value.root.into()),
            next_validators_hash: value.next_validators_hash.into(),
        }
    }
}
