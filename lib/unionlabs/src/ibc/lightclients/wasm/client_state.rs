use frame_support_procedural::DebugNoBound;
use macros::model;

use crate::{
    encoding::{Decode, DecodeErrorOf, Encode, Proto},
    errors::{required, InvalidLength, MissingField},
    hash::H256,
    ibc::core::client::height::Height,
};

#[model(proto(
    raw(protos::ibc::lightclients::wasm::v1::ClientState),
    into,
    from,
    no_static_assert
))]
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

#[derive(DebugNoBound, thiserror::Error)]
pub enum TryFromWasmClientStateError<Data: Decode<Proto, Error: core::error::Error>> {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("unable to decode wasm client state data")]
    Data(#[source] DecodeErrorOf<Proto, Data>),
    #[error("invalid checksum")]
    Checksum(#[from] InvalidLength),
}

impl<Data: Decode<Proto, Error: core::error::Error + Clone>> Clone
    for TryFromWasmClientStateError<Data>
{
    fn clone(&self) -> Self {
        match self {
            Self::MissingField(err) => Self::MissingField(err.clone()),
            Self::Data(err) => Self::Data(err.clone()),
            Self::Checksum(err) => Self::Checksum(err.clone()),
        }
    }
}

impl<Data: Decode<Proto, Error: core::error::Error + PartialEq>> PartialEq
    for TryFromWasmClientStateError<Data>
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MissingField(this), Self::MissingField(other)) => this == other,
            (Self::Data(this), Self::Data(other)) => this == other,
            (Self::Checksum(this), Self::Checksum(other)) => this == other,
            _ => false,
        }
    }
}

impl<Data> TryFrom<protos::ibc::lightclients::wasm::v1::ClientState> for ClientState<Data>
where
    Data: Decode<Proto, Error: core::error::Error>,
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
                .map_err(TryFromWasmClientStateError::Checksum)?,
            latest_height: required!(value.latest_height)?.into(),
        })
    }
}
