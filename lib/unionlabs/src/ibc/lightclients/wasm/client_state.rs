use frame_support_procedural::DebugNoBound;
use macros::model;

use crate::{
    encoding::{Decode, DecodeErrorOf, Encode, Proto},
    errors::InvalidLength,
    hash::H256,
    ibc::core::client::height::Height,
};

#[model(proto(raw(protos::ibc::lightclients::wasm::v1::ClientState), into, from))]
pub struct ClientState<Data> {
    pub data: Data,
    pub checksum: H256,
    // #[deprecated = "use data.height()"]
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
            // #[allow(deprecated)]
            latest_height: Some(val.latest_height.into()),
        }
    }
}

impl<Data: Decode<Proto, Error: PartialEq>> PartialEq for TryFromWasmClientStateError<Data> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Data(this), Self::Data(other)) => this == other,
            (Self::CodeId(this), Self::CodeId(other)) => this == other,
            _ => false,
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
        // #[allow(deprecated)]
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
