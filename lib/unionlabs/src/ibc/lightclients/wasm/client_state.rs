use core::fmt::Debug;

use frame_support_procedural::DebugNoBound;
use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{
    encoding::{Decode, DecodeErrorOf, Encode, Proto},
    errors::InvalidLength,
    hash::H256,
    ibc::core::client::height::Height,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::ibc::lightclients::wasm::v1::ClientState, into, from)]
pub struct ClientState<Data> {
    pub data: Data,
    pub checksum: H256,
    pub latest_height: Height,
}

impl<Data> From<ClientState<Data>> for protos::ibc::lightclients::wasm::v1::ClientState
where
    Data: Encode<Proto>,
{
    fn from(val: ClientState<Data>) -> Self {
        Self {
            data: val.data.encode(),
            checksum: val.checksum.into(),
            latest_height: Some(val.latest_height.into()),
        }
    }
}

#[derive(DebugNoBound)]
pub enum TryFromWasmClientStateError<Data: Decode<Proto>> {
    Data(DecodeErrorOf<Proto, Data>),
    CodeId(InvalidLength),
}

impl<Data> TryFrom<protos::ibc::lightclients::wasm::v1::ClientState> for ClientState<Data>
where
    Data: Decode<Proto>,
{
    type Error = TryFromWasmClientStateError<Data>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::decode(&value.data).map_err(TryFromWasmClientStateError::Data)?,
            checksum: value
                .checksum
                .try_into()
                .map_err(TryFromWasmClientStateError::CodeId)?,
            latest_height: value.latest_height.unwrap().into(),
        })
    }
}
