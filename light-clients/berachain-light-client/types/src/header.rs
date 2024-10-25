use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::commitment::merkle_proof::MerkleProof;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub cometbft_header: tendermint_light_client_types::Header,
    pub execution_header: ExecutionPayloadHeader<BerachainChainSpec>,
    pub execution_header_proof: MerkleProof,
    pub account_proof: AccountProof,
}

#[cfg(feature = "proto")]
pub mod proto {
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
                execution_header: Some(value.execution_header.into()),
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
        CometbftHeader(#[source] tendermint_light_client_types::header::proto::Error),
        #[error("invalid execution header")]
        ExecutionHeader(#[source] execution_payload_header_proto::Error),
        #[error("invalid execution header proof")]
        ExecutionHeaderProof(#[source] TryFromMerkleProofError),
        #[error("invalid account proof")]
        AccountProof(#[source] account_proof::proto::Error),
    }

    impl TryFrom<protos::union::ibc::lightclients::berachain::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::berachain::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                cometbft_header: required!(value.cometbft_header)?
                    .try_into()
                    .map_err(Error::CometbftHeader)?,
                execution_header: required!(value.execution_header)?
                    .try_into()
                    .map_err(Error::ExecutionHeader)?,
                execution_header_proof: required!(value.execution_header_proof)?
                    .try_into()
                    .map_err(Error::ExecutionHeaderProof)?,
                account_proof: required!(value.account_proof)?
                    .try_into()
                    .map_err(Error::AccountProof)?,
            })
        }
    }
}
