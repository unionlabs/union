use cometbls_light_client::client::CometblsLightClient;
use cosmwasm_std::StdError;
use ibc_union_light_client::IbcClientError;
use unionlabs::{
    encoding::{DecodeErrorOf, EthAbi},
    primitives::{Bytes, H256},
};

use crate::client::StateLensIcs23Ics23LightClient;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unimplemented feature")]
    Unimplemented,

    #[error("error while querying l1 state")]
    L1Error(#[from] IbcClientError<CometblsLightClient>),

    #[error("commitment key must be 32 bytes but found: {0:?}")]
    InvalidCommitmentKeyLength(Bytes),

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("verify storage proof error")]
    VerifyMembership(#[source] ics23::ibc_api::VerifyMembershipError),

    #[error("verify storage absence error")]
    VerifyNonMembership(#[source] ics23::ibc_api::VerifyMembershipError),

    #[error("commitment value must be 32 bytes but found: {0}")]
    InvalidCommitmentValueLength(Bytes),

    #[error("unable to decode l2 consensus state")]
    InvalidL2ConsensusState(
        #[source] DecodeErrorOf<EthAbi, tendermint_light_client_types::ConsensusState>,
    ),

    #[error("could not decode the proof: {0}")]
    ProofDecode(Bytes),
}

impl From<Error> for IbcClientError<StateLensIcs23Ics23LightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

impl From<Error> for StdError {
    fn from(value: Error) -> Self {
        StdError::generic_err(value.to_string())
    }
}
