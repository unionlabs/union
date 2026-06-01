use gno_types::{BlockId, Vote, commit::CommitValidateBasicError};
use unionlabs::{
    bounded::BoundedI64,
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    primitives::{H256, encoding::HexUnprefixed},
};

/// Source: <https://github.com/atomone-hub/atomone/blob/5e3a5d733d818c1fd3d8b08aac9baf329737d27d/modules/10-gno/errors.go>
mod ibc_sentinel_errors {
    pub const ERR_INVALID_HEADER: &str = "10-gno: invalid header";
    pub const ERR_OLD_HEADER_EXPIRED: &str = "10-gno: old header has expired";
    pub const ERR_NEW_VAL_SET_CANT_BE_TRUSTED: &str = "10-gno: new val set cannot be trusted";
}

use ibc_sentinel_errors::*;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("integer overflow")]
    IntegerOverflow,

    #[error(
        "{ERR_OLD_HEADER_EXPIRED}: trusted header expired at {} (now: {header_timestamp})",
        header_timestamp.checked_add(*trusting_period).expect("valid")
    )]
    HeaderExpired {
        trusting_period: Duration,
        header_timestamp: Timestamp,
    },

    #[error("headers must be non adjacent in height")]
    HeadersMustBeNonAdjacent,

    #[error("headers must be adjacent in height")]
    HeadersMustBeAdjacent,

    #[error("{ERR_INVALID_HEADER}: failed to verify new header and vals")]
    VerifyNewHeaderAndVals(#[from] VerifyNewHeaderAndValsError),

    #[error("{ERR_NEW_VAL_SET_CANT_BE_TRUSTED}: trusted validators failed to verify commit")]
    VerifyLightCommitError(#[from] VerifyLightCommitError),

    #[error("{ERR_INVALID_HEADER}: failed to verify commit")]
    TrustedValidatorsVerifyCommitError(#[from] TrustedValidatorsVerifyCommitError),

    #[error(
        "expected old header next validators ({next_validators_hash}) to match those from new header ({validators_hash})"
    )]
    NextValidatorsHashMismatch {
        next_validators_hash: H256<HexUnprefixed>,
        validators_hash: H256<HexUnprefixed>,
    },
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VerifyNewHeaderAndValsError {
    #[error(
        "expected new header height {untrusted_header_height} to be greater than one of old header {trusted_header_height}"
    )]
    NewHeaderHeightMustBeGreater {
        untrusted_header_height: BoundedI64<0>,
        trusted_header_height: BoundedI64<0>,
    },

    #[error(
        "expected new header time {untrusted_header_time} to be after old header time {trusted_header_time}"
    )]
    NewHeaderTimeMustBeGreater {
        untrusted_header_time: Timestamp,
        trusted_header_time: Timestamp,
    },

    #[error(
        "new header has a time from the future {untrusted_header_time} (now: {now}; max clock drift: {max_clock_drift})"
    )]
    NewHeaderFromFuture {
        untrusted_header_time: Timestamp,
        now: Timestamp,
        max_clock_drift: Duration,
    },

    #[error(
        "expected new header validators ({untrusted_header_validators_hash}) to match those that were supplied ({untrusted_validators_hash}) at height {untrusted_header_height}"
    )]
    UntrustedValidatorSetMismatch {
        untrusted_header_validators_hash: H256,
        untrusted_validators_hash: H256,
        untrusted_header_height: BoundedI64<0>,
    },
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VerifyLightCommitError {
    #[error(transparent)]
    CommitValidateBasic(#[from] CommitValidateBasicError),

    #[error(
        "{ERR_NEW_VAL_SET_CANT_BE_TRUSTED}: Invalid commit -- wrong height: {expected} vs {actual}"
    )]
    InvalidCommitHeight {
        expected: BoundedI64<0>,
        actual: BoundedI64<0>,
    },

    #[error("invalid commit -- wrong block id: want {want} got {got}")]
    InvalidBlockId { want: BlockId, got: BlockId },

    #[error("trusted validators failed to verify commit: {vote}")]
    InvalidSignature { vote: Vote },

    #[error(
        "{ERR_NEW_VAL_SET_CANT_BE_TRUSTED}: Invalid commit -- insufficient old voting power: got {got}, needed more than {min}"
    )]
    InsufficientTrustedVotingPower { got: i64, min: i64 },

    #[error(
        "{ERR_NEW_VAL_SET_CANT_BE_TRUSTED}: int64 overflow while calculating voting power needed. please provide smaller trustLevel numerator"
    )]
    VotingPowerOverflow,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TrustedValidatorsVerifyCommitError {
    #[error(transparent)]
    CommitValidateBasic(#[from] CommitValidateBasicError),

    #[error("Invalid commit -- wrong height: {expected} vs {actual}")]
    InvalidCommitHeightError {
        expected: BoundedI64<0>,
        actual: BoundedI64<0>,
    },

    #[error("Invalid commit -- wrong set size: {expected} vs {actual}")]
    InvalidCommitPrecommitsError { expected: usize, actual: usize },

    #[error("invalid commit -- invalid signature: {vote}")]
    InvalidSignature { vote: Vote },

    #[error("Invalid commit -- insufficient old voting power: got {got}, needed {needed}")]
    TooMuchChangeError { got: i64, needed: i64 },

    #[error("invalid commit -- wrong block id: want {expected} got {actual}")]
    InvalidCommitWrongBlockId { expected: BlockId, actual: BlockId },
}
