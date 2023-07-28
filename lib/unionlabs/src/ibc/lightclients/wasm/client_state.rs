use std::fmt::Debug;

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    errors::InvalidLength, ethereum::H256, ibc::core::client::height::Height, IntoProto,
    TryFromProto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientState<Data> {
    pub data: Data,
    pub code_id: H256,
    pub latest_height: Height,
}

// impl<Data> crate::chain::ClientState for ClientState<Data> {
//     fn height(&self) -> Height {
//         self.latest_height
//     }
// }

impl TypeUrl for protos::ibc::lightclients::wasm::v1::ClientState {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ClientState";
}

impl<Data> From<ClientState<Data>> for protos::ibc::lightclients::wasm::v1::ClientState
where
    Data: IntoProto,
{
    fn from(val: ClientState<Data>) -> Self {
        Self {
            data: val.data.into_proto().encode_to_vec(),
            code_id: val.code_id.into_bytes(),
            latest_height: Some(val.latest_height.into()),
        }
    }
}

impl<Data: IntoProto> IntoProto for ClientState<Data> {
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
    Data: TryFromProto,
    <Data as TryFromProto>::Proto: prost::Message + Default,
    TryFromProtoErrorOf<Data>: Debug,
{
    type Error =
        TryFromWasmClientStateError<<Data as TryFrom<<Data as TryFromProto>::Proto>>::Error>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::try_from_proto(
                <Data as TryFromProto>::Proto::decode(&*value.data)
                    .map_err(TryFromWasmClientStateError::Prost)?,
            )
            .map_err(TryFromWasmClientStateError::TryFromProto)?,
            code_id: value
                .code_id
                .try_into()
                .map_err(TryFromWasmClientStateError::CodeId)?,
            latest_height: value.latest_height.unwrap().into(),
        })
    }
}

// , ibc::lightclients::wasm::client_state::ClientState<Data>: std::convert::TryFrom<protos::ibc::lightclients::wasm::v1::ClientState>

impl<Data> TryFromProto for ClientState<Data>
where
    Data: TryFromProto,
    <Data as TryFromProto>::Proto: prost::Message + Default,
    TryFromProtoErrorOf<Data>: Debug,
{
    type Proto = protos::ibc::lightclients::wasm::v1::ClientState;
}
