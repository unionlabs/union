use std::fmt::Debug;

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{IntoProto, Proto, TryFromProto, TryFromProtoBytesError, TryFromProtoErrorOf, TypeUrl};

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

impl<Data: Proto> Proto for ConsensusState<Data> {
    type Proto = protos::ibc::lightclients::wasm::v1::ConsensusState;
}

#[derive(Debug)]
pub enum TryFromWasmConsensusStateError<Err> {
    TryFromProto(Err),
    Prost(prost::DecodeError),
}

impl<Data> TryFrom<protos::ibc::lightclients::wasm::v1::ConsensusState> for ConsensusState<Data>
where
    Data: TryFromProto,
    <Data as Proto>::Proto: prost::Message + Default,
    TryFromProtoErrorOf<Data>: Debug,
{
    type Error = TryFromWasmConsensusStateError<TryFromProtoBytesError<TryFromProtoErrorOf<Data>>>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::try_from_proto_bytes(&value.data)
                .map_err(TryFromWasmConsensusStateError::TryFromProto)?,
        })
    }
}
