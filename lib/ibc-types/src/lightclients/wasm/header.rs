use prost::Message;

use crate::{core::client::height::Height, IntoProto, TypeUrl};

#[derive(Debug, Clone)]
pub struct Header<Data> {
    pub data: Data,
    pub height: Height,
}

impl<Data: IntoProto> IntoProto for Header<Data> {
    type Proto = protos::ibc::lightclients::wasm::v1::Header;
}

impl<Data: IntoProto> From<Header<Data>> for protos::ibc::lightclients::wasm::v1::Header {
    fn from(value: Header<Data>) -> Self {
        Self {
            data: value.data.into_proto().encode_to_vec(),
            height: Some(value.height.into()),
        }
    }
}

impl TypeUrl for protos::ibc::lightclients::wasm::v1::Header {
    const TYPE_URL: &'static str = "/ibc.lightclients.wasm.v1.Header";
}
