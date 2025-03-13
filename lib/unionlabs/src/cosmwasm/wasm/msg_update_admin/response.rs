use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgUpdateAdminResponse {}

pub mod proto {
    use super::MsgUpdateAdminResponse;
    use crate::{
        encoding::{DecodeErrorOf, Proto},
        impl_proto_via_try_from_into,
    };

    impl_proto_via_try_from_into!(MsgUpdateAdminResponse => protos::cosmwasm::wasm::v1::MsgUpdateAdminResponse);

    impl From<MsgUpdateAdminResponse> for protos::cosmwasm::wasm::v1::MsgUpdateAdminResponse {
        fn from(_: MsgUpdateAdminResponse) -> Self {
            Self {}
        }
    }

    impl From<protos::cosmwasm::wasm::v1::MsgUpdateAdminResponse> for MsgUpdateAdminResponse {
        fn from(_: protos::cosmwasm::wasm::v1::MsgUpdateAdminResponse) -> Self {
            Self {}
        }
    }

    static_assertions::assert_impl!(DecodeErrorOf<Proto, MsgUpdateAdminResponse>: core::error::Error);
}
