use cosmwasm_std::VerificationError;
use ics008_wasm_client::IbcClientError;
use near_primitives_core::hash::CryptoHash;

use crate::client::NearLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(u64),
    #[error("epoch block producer not found for epoch {0}")]
    EpochBlockProducerNotFound(CryptoHash),
    #[error(transparent)]
    Verifier(#[from] near_verifier::error::Error),
}

impl From<Error> for IbcClientError<NearLightClient> {
    fn from(_value: Error) -> Self {
        todo!()
    }
}
