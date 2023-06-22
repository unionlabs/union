use prost::Message;

use crate::{IntoProto, TryFromProto, TypeUrl};

#[derive(Debug, Clone)]
pub struct ConsensusState<Data> {
    pub data: Data,
    pub timestamp: u64,
}

impl<Data: IntoProto> From<ConsensusState<Data>>
    for protos::ibc::lightclients::wasm::v1::ConsensusState
{
    fn from(value: ConsensusState<Data>) -> Self {
        protos::ibc::lightclients::wasm::v1::ConsensusState {
            data: value.data.into_proto().encode_to_vec(),
            timestamp: value.timestamp,
        }
    }
}

impl TypeUrl for protos::ibc::lightclients::wasm::v1::ConsensusState {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ConsensusState";
}

impl<Data: IntoProto> IntoProto for ConsensusState<Data> {
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
    <Data as TryFromProto>::Proto: prost::Message + Default,
{
    type Error =
        TryFromWasmConsensusStateError<<Data as TryFrom<<Data as TryFromProto>::Proto>>::Error>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::try_from_proto(
                <Data as TryFromProto>::Proto::decode(&*value.data)
                    .map_err(TryFromWasmConsensusStateError::Prost)?,
            )
            .map_err(TryFromWasmConsensusStateError::TryFromProto)?,
            timestamp: value.timestamp,
        })
    }
}

impl<Data> TryFromProto for ConsensusState<Data>
where
    Data: TryFromProto,
    <Data as TryFromProto>::Proto: prost::Message + Default,
{
    type Proto = protos::ibc::lightclients::wasm::v1::ConsensusState;
}
