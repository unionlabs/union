use contracts::glue::UnionIbcLightclientsCometblsV1ConsensusStateData;

use crate::{
    core::commitment::merkle_root::MerkleRoot, errors::MissingField, IntoProto, TryFromProto,
    TypeUrl,
};

#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub root: MerkleRoot,
    pub next_validators_hash: Vec<u8>,
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ConsensusState> for ConsensusState {
    type Error = MissingField;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            root: value.root.ok_or(MissingField("root"))?.into(),
            next_validators_hash: value.next_validators_hash,
        })
    }
}

impl TypeUrl for protos::union::ibc::lightclients::cometbls::v1::ConsensusState {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.cometbls.v1.ConsensusState";
}

impl IntoProto for ConsensusState {
    type Proto = protos::union::ibc::lightclients::cometbls::v1::ConsensusState;
}

impl TryFromProto for ConsensusState {
    type Proto = protos::union::ibc::lightclients::cometbls::v1::ConsensusState;
}

impl From<ConsensusState> for protos::union::ibc::lightclients::cometbls::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            root: Some(value.root.into()),
            next_validators_hash: value.next_validators_hash,
        }
    }
}

impl From<ConsensusState> for UnionIbcLightclientsCometblsV1ConsensusStateData {
    fn from(value: ConsensusState) -> Self {
        Self {
            root: value.root.into(),
            next_validators_hash: value.next_validators_hash.into(),
        }
    }
}
