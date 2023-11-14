use std::fmt::Debug;

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{encoding::Decode, IntoProto, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusState<Data> {
    pub data: Data,
}

impl<Data: IntoProto> From<ConsensusState<Data>>
    for protos::ibc::lightclients::wasm::v1::ConsensusState
{
    fn from(value: ConsensusState<Data>) -> Self {
        protos::ibc::lightclients::wasm::v1::ConsensusState {
            data: value.data.into_proto().encode_to_vec(),
        }
    }
}

impl TypeUrl for protos::ibc::lightclients::wasm::v1::ConsensusState {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ConsensusState";
}

impl<Data> Proto for ConsensusState<Data> {
    type Proto = protos::ibc::lightclients::wasm::v1::ConsensusState;
}

#[derive(Debug)]
pub enum TryFromWasmConsensusStateError<Err> {
    TryFromProto(Err),
    Prost(prost::DecodeError),
}

impl<Data> TryFrom<protos::ibc::lightclients::wasm::v1::ConsensusState> for ConsensusState<Data>
where
    Data: Decode<crate::encoding::Proto>,
{
    type Error = TryFromWasmConsensusStateError<Data::Error>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::decode(&value.data)
                .map_err(TryFromWasmConsensusStateError::TryFromProto)?,
        })
    }
}
