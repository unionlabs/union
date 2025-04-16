use serde::{Deserialize, Serialize};
use unionlabs_primitives::{encoding::Base64, Bytes};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgExecuteContractResponse {
    pub data: Bytes<Base64>,
}

#[doc(hidden)] // TODO: Do this to all proto and ethabi modules
pub mod proto {
    use super::MsgExecuteContractResponse;
    use crate::impl_proto_via_try_from_into;

    impl_proto_via_try_from_into!(MsgExecuteContractResponse => protos::cosmwasm::wasm::v1::MsgExecuteContractResponse);

    impl From<MsgExecuteContractResponse> for protos::cosmwasm::wasm::v1::MsgExecuteContractResponse {
        fn from(value: MsgExecuteContractResponse) -> Self {
            Self {
                data: value.data.into(),
            }
        }
    }

    impl From<protos::cosmwasm::wasm::v1::MsgExecuteContractResponse> for MsgExecuteContractResponse {
        fn from(value: protos::cosmwasm::wasm::v1::MsgExecuteContractResponse) -> Self {
            Self {
                data: value.data.into(),
            }
        }
    }
}
