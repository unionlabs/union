use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    ibc::core::client::height::Height,
    IntoProto, Proto, TryFromProto, TryFromProtoBytesError, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header<Data> {
    pub data: Data,
    pub height: Height,
}

impl<Data: Proto> Proto for Header<Data> {
    type Proto = protos::ibc::lightclients::wasm::v1::Header;
}

impl<Data: IntoProto> From<Header<Data>> for protos::ibc::lightclients::wasm::v1::Header {
    fn from(value: Header<Data>) -> Self {
        Self {
            data: value.data.into_proto_bytes(),
            height: Some(value.height.into()),
        }
    }
}

impl TypeUrl for protos::ibc::lightclients::wasm::v1::Header {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.Header";
}

#[derive(Debug)]
pub enum TryFromHeaderError<Err> {
    MissingField(MissingField),
    Data(Err),
}

impl<Data: TryFromProto> TryFrom<protos::ibc::lightclients::wasm::v1::Header> for Header<Data> {
    type Error = TryFromHeaderError<TryFromProtoBytesError<TryFromProtoErrorOf<Data>>>;

    fn try_from(value: protos::ibc::lightclients::wasm::v1::Header) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::try_from_proto_bytes(&value.data).map_err(TryFromHeaderError::Data)?,
            height: required!(value.height)?.into(),
        })
    }
}
