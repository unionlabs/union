use frame_support_procedural::DebugNoBound;
use macros::model;
use static_assertions::{assert_impl, assert_impl_all};

use crate::encoding::{Decode, DecodeErrorOf, Encode, Proto};

#[model(proto(
    raw(protos::ibc::lightclients::wasm::v1::ConsensusState),
    into,
    from,
    no_static_assert
))]
pub struct ConsensusState<Data> {
    pub data: Data,
}

impl<Data: Encode<Proto>> From<ConsensusState<Data>>
    for protos::ibc::lightclients::wasm::v1::ConsensusState
{
    fn from(value: ConsensusState<Data>) -> Self {
        protos::ibc::lightclients::wasm::v1::ConsensusState {
            data: value.data.encode(),
        }
    }
}

#[derive(DebugNoBound, thiserror::Error)]
pub enum TryFromWasmConsensusStateError<Data: Decode<Proto, Error: std::error::Error>> {
    #[error("unable to decode wasm consensus state data")]
    Data(#[source] DecodeErrorOf<Proto, Data>),
}

impl<Data: Decode<Proto, Error: std::error::Error + Clone>> Clone
    for TryFromWasmConsensusStateError<Data>
{
    fn clone(&self) -> Self {
        match self {
            Self::Data(err) => Self::Data(err.clone()),
        }
    }
}

impl<Data: Decode<Proto, Error: std::error::Error + PartialEq>> PartialEq
    for TryFromWasmConsensusStateError<Data>
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Data(this), Self::Data(other)) => this == other,
        }
    }
}

impl<Data> TryFrom<protos::ibc::lightclients::wasm::v1::ConsensusState> for ConsensusState<Data>
where
    Data: Decode<Proto, Error: std::error::Error>,
{
    type Error = TryFromWasmConsensusStateError<Data>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::decode(&value.data).map_err(TryFromWasmConsensusStateError::Data)?,
        })
    }
}

assert_impl_all!(TryFromWasmConsensusStateError<crate::ibc::lightclients::scroll::consensus_state::ConsensusState>: std::error::Error);
