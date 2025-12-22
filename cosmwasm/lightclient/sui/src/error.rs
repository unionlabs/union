use ibc_union_light_client::IbcClientError;

use crate::client::SuiLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    CwStd(#[from] cosmwasm_std::StdError),

    #[error(transparent)]
    CwVerification(#[from] cosmwasm_std::VerificationError),

    #[error("initial committee not set")]
    NoInitialCommittee,

    #[error(transparent)]
    Verifier(#[from] sui_verifier::Error),

    #[error("failed signature verification")]
    SignatureVerification,
}

impl From<Error> for IbcClientError<SuiLightClient> {
    fn from(value: Error) -> Self {
        Self::ClientSpecific(value)
    }
}

impl From<Error> for sui_verifier::Error {
    fn from(value: Error) -> Self {
        sui_verifier::Error::Client(Box::new(value))
    }
}
