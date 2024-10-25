use beacon_api_types::execution_payload_header::ExecutionPayloadHeader;
use ethereum_light_client_types::AccountProof;
use unionlabs::ibc::core::commitment::merkle_proof::MerkleProof;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub cometbft_header: tendermint_light_client_types::Header,
    pub execution_header: ExecutionPayloadHeader,
    pub execution_header_proof: MerkleProof,
    pub account_proof: AccountProof,
}

#[cfg(feature = "proto")]
pub mod proto {
    use ethereum_light_client_types::{account_proof, execution_payload_header_proto};
    use unionlabs::{
        errors::MissingField, ibc::core::commitment::merkle_proof::TryFromMerkleProofError,
        impl_proto_via_try_from_into, required,
    };

    use super::Header;

    impl_proto_via_try_from_into!(Header => protos::union::ibc::lightclients::berachain::v1::Header);

    impl From<Header> for protos::union::ibc::lightclients::berachain::v1::Header {
        fn from(value: Header) -> Self {
            Self {
                cometbft_header: Some(value.cometbft_header.into()),
                execution_header: Some(execution_payload_header_proto::into_proto(
                    value.execution_header,
                )),
                execution_header_proof: Some(value.execution_header_proof.into()),
                account_proof: Some(value.account_proof.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid cometbft header")]
        CometbftHeader(#[from] tendermint_light_client_types::header::proto::Error),
        #[error("invalid execution header")]
        ExecutionHeader(#[from] execution_payload_header_proto::Error),
        #[error("invalid execution header proof")]
        ExecutionHeaderProof(#[from] TryFromMerkleProofError),
        #[error("invalid account proof")]
        AccountProof(#[from] account_proof::proto::Error),
    }

    impl TryFrom<protos::union::ibc::lightclients::berachain::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::berachain::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                cometbft_header: required!(value.cometbft_header)?.try_into()?,
                execution_header: execution_payload_header_proto::try_from_proto(required!(
                    value.execution_header
                )?)?,
                execution_header_proof: required!(value.execution_header_proof)?.try_into()?,
                account_proof: required!(value.account_proof)?.try_into()?,
            })
        }
    }
}
