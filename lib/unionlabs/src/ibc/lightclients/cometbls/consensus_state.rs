use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, InvalidLength, MissingField},
    google::protobuf::any::Any,
    hash::H256,
    ibc::{core::commitment::merkle_root::MerkleRoot, lightclients::wasm},
    EthAbi, Proto, TryFromProtoErrorOf, TypeUrl,
};

#[cfg_attr(
    feature = "ethabi",
    derive(
        ethers_contract_derive::EthAbiType,
        ethers_contract_derive::EthAbiCodec
    )
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusState {
    pub timestamp: u64,
    pub root: MerkleRoot,
    pub next_validators_hash: H256,
}

impl From<Any<wasm::consensus_state::ConsensusState<ConsensusState>>> for ConsensusState {
    fn from(value: Any<wasm::consensus_state::ConsensusState<ConsensusState>>) -> Self {
        value.0.data
    }
}

#[derive(Debug)]
pub enum TryFromConsensusStateError {
    MissingField(MissingField),
    Root(TryFromProtoErrorOf<MerkleRoot>),
    NextValidatorsHash(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ConsensusState> for ConsensusState {
    type Error = TryFromConsensusStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: value.timestamp,
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

#[cfg(feature = "ethabi")]
impl EthAbi for ConsensusState {
    type EthAbi = ConsensusState;
}

impl TypeUrl for protos::union::ibc::lightclients::cometbls::v1::ConsensusState {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.cometbls.v1.ConsensusState";
}

impl Proto for ConsensusState {
    type Proto = protos::union::ibc::lightclients::cometbls::v1::ConsensusState;
}

impl From<ConsensusState> for protos::union::ibc::lightclients::cometbls::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            timestamp: value.timestamp,
            root: Some(value.root.into()),
            next_validators_hash: value.next_validators_hash.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<ConsensusState> for contracts::glue::UnionIbcLightclientsCometblsV1ConsensusStateData {
    fn from(value: ConsensusState) -> Self {
        Self {
            timestamp: value.timestamp,
            root: value.root.into(),
            next_validators_hash: value.next_validators_hash.into(),
        }
    }
}
