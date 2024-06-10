use macros::model;

use crate::{
    berachain::BerachainChainSpec,
    errors::{required, MissingField},
    ibc::{
        core::{
            client::height::Height,
            commitment::merkle_proof::{MerkleProof, TryFromMerkleProofError},
        },
        lightclients::{
            ethereum::{
                account_proof::{AccountProof, TryFromAccountProofError},
                execution_payload_header::{
                    ExecutionPayloadHeader, TryFromExecutionPayloadHeaderError,
                },
            },
            tendermint,
        },
    },
    traits,
};

#[model(proto(
    raw(protos::union::ibc::lightclients::berachain::v1::Header),
    into,
    from
))]
pub struct Header {
    pub cometbft_header: tendermint::header::Header,
    pub execution_header: ExecutionPayloadHeader<BerachainChainSpec>,
    pub execution_header_proof: MerkleProof,
    pub account_proof: AccountProof,
}

impl traits::Header for Header {
    fn trusted_height(&self) -> Height {
        Height {
            // TODO: Change to `ETHEREUM_REVISION_NUMBER` once that has been moved into this crate
            revision_number: 0,
            revision_height: self.execution_header.block_number,
        }
    }
}

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
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid cometbft header")]
    CometbftHeader(#[source] tendermint::header::TryFromHeaderError),
    #[error("invalid execution header")]
    ExecutionHeader(#[source] TryFromExecutionPayloadHeaderError),
    #[error("invalid execution header proof")]
    ExecutionHeaderProof(#[source] TryFromMerkleProofError),
    #[error("invalid account proof")]
    AccountProof(#[source] TryFromAccountProofError),
}

impl TryFrom<protos::union::ibc::lightclients::berachain::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::berachain::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            cometbft_header: required!(value.cometbft_header)?
                .try_into()
                .map_err(TryFromHeaderError::CometbftHeader)?,
            execution_header: required!(value.execution_header)?
                .try_into()
                .map_err(TryFromHeaderError::ExecutionHeader)?,
            execution_header_proof: required!(value.execution_header_proof)?
                .try_into()
                .map_err(TryFromHeaderError::ExecutionHeaderProof)?,
            account_proof: required!(value.account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::AccountProof)?,
        })
    }
}
