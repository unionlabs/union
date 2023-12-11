use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    encoding::Decode, errors::InvalidLength, hash::H256, ibc::core::client::height::Height,
    IntoProto, Proto, TypeUrl,
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientState<Data> {
    pub data: Data,
    pub checksum: H256,
    pub latest_height: Height,
}

impl TypeUrl for protos::ibc::lightclients::wasm::v1::ClientState {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ClientState";
}

impl<Data> From<ClientState<Data>> for protos::ibc::lightclients::wasm::v1::ClientState
where
    Data: IntoProto,
{
    fn from(val: ClientState<Data>) -> Self {
        Self {
            data: val.data.into_proto_bytes(),
            checksum: val.checksum.into_bytes(),
            latest_height: Some(val.latest_height.into()),
        }
    }
}

impl<Data> Proto for ClientState<Data> {
    type Proto = protos::ibc::lightclients::wasm::v1::ClientState;
}

#[derive(Debug)]
pub enum TryFromWasmClientStateError<Err> {
    TryFromProto(Err),
    Prost(prost::DecodeError),
    CodeId(InvalidLength),
}

impl<Data> TryFrom<protos::ibc::lightclients::wasm::v1::ClientState> for ClientState<Data>
where
    Data: Decode<crate::encoding::Proto>,
{
    type Error = TryFromWasmClientStateError<Data::Error>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::decode(&value.data).map_err(TryFromWasmClientStateError::TryFromProto)?,
            checksum: value
                .checksum
                .try_into()
                .map_err(TryFromWasmClientStateError::CodeId)?,
            latest_height: value.latest_height.unwrap().into(),
        })
    }
}
