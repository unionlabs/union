use core::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::primitives::{Bytes, H256, encoding::Base64};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgStoreAndMigrateContractResponse {
    pub code_id: NonZeroU64,
    pub checksum: H256,
    pub data: Bytes<Base64>,
}

#[doc(hidden)] // TODO: Do this to all proto and ethabi modules
pub mod proto {
    use unionlabs_primitives::FixedBytesError;

    use super::MsgStoreAndMigrateContractResponse;
    use crate::impl_proto_via_try_from_into;

    impl_proto_via_try_from_into!(MsgStoreAndMigrateContractResponse => protos::cosmwasm::wasm::v1::MsgStoreAndMigrateContractResponse);

    impl From<MsgStoreAndMigrateContractResponse>
        for protos::cosmwasm::wasm::v1::MsgStoreAndMigrateContractResponse
    {
        fn from(value: MsgStoreAndMigrateContractResponse) -> Self {
            Self {
                code_id: value.code_id.into(),
                checksum: value.checksum.into(),
                data: value.data.into(),
            }
        }
    }

    impl TryFrom<protos::cosmwasm::wasm::v1::MsgStoreAndMigrateContractResponse>
        for MsgStoreAndMigrateContractResponse
    {
        type Error = Error;

        fn try_from(
            value: protos::cosmwasm::wasm::v1::MsgStoreAndMigrateContractResponse,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                code_id: value.code_id.try_into().map_err(|_| Error::CodeId)?,
                checksum: value.checksum.try_into()?,
                data: value.data.into(),
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
