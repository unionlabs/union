use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgUpdateInstantiateConfigResponse {}

pub mod proto {
    use super::MsgUpdateInstantiateConfigResponse;
    use crate::{
        encoding::{DecodeErrorOf, Proto},
        impl_proto_via_try_from_into,
    };

    impl_proto_via_try_from_into!(MsgUpdateInstantiateConfigResponse => protos::cosmwasm::wasm::v1::MsgUpdateInstantiateConfigResponse);

    impl From<MsgUpdateInstantiateConfigResponse>
        for protos::cosmwasm::wasm::v1::MsgUpdateInstantiateConfigResponse
    {
        fn from(_: MsgUpdateInstantiateConfigResponse) -> Self {
            Self {}
        }
    }

    impl From<protos::cosmwasm::wasm::v1::MsgUpdateInstantiateConfigResponse>
        for MsgUpdateInstantiateConfigResponse
    {
        fn from(_: protos::cosmwasm::wasm::v1::MsgUpdateInstantiateConfigResponse) -> Self {
            Self {}
        }
    }

    static_assertions::assert_impl!(DecodeErrorOf<Proto, MsgUpdateInstantiateConfigResponse>: core::error::Error);
}
