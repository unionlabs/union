use serde::{Deserialize, Serialize};
use unionlabs_primitives::{encoding::Base64, Bytes};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgMigrateContractResponse {
    pub data: Bytes<Base64>,
}

#[doc(hidden)] // TODO: Do this to all proto and ethabi modules
pub mod proto {
    use super::MsgMigrateContractResponse;
    use crate::impl_proto_via_try_from_into;

    impl_proto_via_try_from_into!(MsgMigrateContractResponse => protos::cosmwasm::wasm::v1::MsgMigrateContractResponse);

    impl From<MsgMigrateContractResponse> for protos::cosmwasm::wasm::v1::MsgMigrateContractResponse {
        fn from(value: MsgMigrateContractResponse) -> Self {
            Self {
                data: value.data.into(),
            }
        }
    }

    impl From<protos::cosmwasm::wasm::v1::MsgMigrateContractResponse> for MsgMigrateContractResponse {
        fn from(value: protos::cosmwasm::wasm::v1::MsgMigrateContractResponse) -> Self {
            Self {
                data: value.data.into(),
            }
        }
    }
}
