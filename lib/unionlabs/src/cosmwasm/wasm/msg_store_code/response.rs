use core::num::NonZeroU64;

use serde::{Deserialize, Serialize};
use unionlabs_primitives::H256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgStoreCodeResponse {
    pub code_id: NonZeroU64,
    pub checksum: H256,
}

#[doc(hidden)] // TODO: Do this to all proto and ethabi modules
pub mod proto {
    use unionlabs_primitives::FixedBytesError;

    use super::MsgStoreCodeResponse;
    use crate::impl_proto_via_try_from_into;

    impl_proto_via_try_from_into!(MsgStoreCodeResponse => protos::cosmwasm::wasm::v1::MsgStoreCodeResponse);

    impl From<MsgStoreCodeResponse> for protos::cosmwasm::wasm::v1::MsgStoreCodeResponse {
        fn from(value: MsgStoreCodeResponse) -> Self {
            Self {
                code_id: value.code_id.into(),
                checksum: value.checksum.into(),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgStoreCodeResponse> for MsgStoreCodeResponse {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgStoreCodeResponse,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                code_id: value.code_id.try_into().map_err(|_| Error::CodeId)?,
                checksum: value.checksum.try_into()?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid code id, must be > 0")]
        CodeId,
        #[error("invalid checksum")]
        Checksum(#[from] FixedBytesError),
    }
}
