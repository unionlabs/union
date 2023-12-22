use serde::{Deserialize, Serialize};

use crate::{
    errors::MissingField, IntoProto, Proto, TryFromProto, TryFromProtoBytesError,
    TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientMessage<Data> {
    pub data: Data,
}

impl<Data: Proto> Proto for ClientMessage<Data> {
    type Proto = protos::ibc::lightclients::wasm::v1::ClientMessage;
}

impl<Data: IntoProto> From<ClientMessage<Data>>
    for protos::ibc::lightclients::wasm::v1::ClientMessage
{
    fn from(value: ClientMessage<Data>) -> Self {
        Self {
            data: value.data.into_proto_bytes(),
        }
    }
}

impl TypeUrl for protos::ibc::lightclients::wasm::v1::ClientMessage {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.ClientMessage";
}

#[derive(Debug)]
pub enum TryFromClientMessageError<Err> {
    MissingField(MissingField),
    Data(Err),
}

impl<Data: TryFromProto> TryFrom<protos::ibc::lightclients::wasm::v1::ClientMessage>
    for ClientMessage<Data>
{
    type Error = TryFromClientMessageError<TryFromProtoBytesError<TryFromProtoErrorOf<Data>>>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ClientMessage,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::try_from_proto_bytes(&value.data)
                .map_err(TryFromClientMessageError::Data)?,
        })
    }
}
