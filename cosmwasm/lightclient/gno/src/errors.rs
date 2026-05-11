use ibc_union_light_client::IbcClientError;
use unionlabs::google::protobuf::timestamp::TryFromCosmwasmTimestampError;

use crate::client::GnoLightClient;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
// TODO: Use an error reporter at the top level of ics008-wasm-client so we don't have to include the sources manually in the display impl
pub enum Error {
    #[error(transparent)]
    MathOverflow(#[from] MathOverflow),

    #[error(transparent)]
    IbcHeightTooLargeForTendermintHeight(#[from] IbcHeightTooLargeForGnoHeight),

    #[error("verify membership error")]
    VerifyMembership(#[from] ics23::ibc_api::VerifyMembershipError),

    #[error(transparent)]
    GnoVerify(#[from] gno_verifier::error::Error),

    #[error(transparent)]
    InvalidHostTimestamp(#[from] InvalidHostTimestamp),

    #[error("invalid or empty validator set, supported keys are: bls12381 and ed25519")]
    InvalidValidatorSet,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("the ibc height.revision_height does not fit in an i64 ({0})")]
pub struct IbcHeightTooLargeForGnoHeight(pub u64);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid timestamp from the host")]
pub struct InvalidHostTimestamp(#[from] pub TryFromCosmwasmTimestampError);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("math operation with overflow")]
pub struct MathOverflow;

// required for IbcClient trait
impl From<Error> for IbcClientError<GnoLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

// would be nice, but both foreign types :(
// impl From<ics23::ibc_api::VerifyMembershipError> for IbcClientError<TendermintLightClient> {
//     fn from(value: ics23::ibc_api::VerifyMembershipError) -> Self {
//         IbcClientError::ClientSpecific(Error::VerifyMembership(value))
//     }
// }
