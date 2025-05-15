use ibc_union_light_client::IbcClientError;

use crate::client::SuiLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("initial committee not set")]
    NoInitialCommittee,

    #[error(transparent)]
    Verifier(#[from] sui_verifier::Error),
}

impl From<Error> for IbcClientError<SuiLightClient> {
    fn from(value: Error) -> Self {
        Self::ClientSpecific(value)
    }
}

impl Into<sui_verifier::Error> for Error {
    fn into(self) -> sui_verifier::Error {
        sui_verifier::Error::Client(Box::new(self))
    }
}
