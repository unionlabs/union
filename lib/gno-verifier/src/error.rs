use gno_types::{BlockId, Vote, commit::CommitValidateBasicError};
use unionlabs::{
    bounded::BoundedI64,
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    primitives::{H256, encoding::HexUnprefixed},
};

/// Source: <https://github.com/atomone-hub/atomone/blob/5e3a5d733d818c1fd3d8b08aac9baf329737d27d/modules/10-gno/errors.go>
mod ibc_sentinel_errors {
    pub const ERR_INVALID_CHAIN_ID: &str = "10-gno: invalid chain-id";
    pub const ERR_INVALID_TRUSTING_PERIOD: &str = "10-gno: invalid trusting period";
    pub const ERR_INVALID_UNBONDING_PERIOD: &str = "10-gno: invalid unbonding period";
    pub const ERR_INVALID_HEADER_HEIGHT: &str = "10-gno: invalid header height";
    pub const ERR_INVALID_HEADER: &str = "10-gno: invalid header";
    pub const ERR_INVALID_MAX_CLOCK_DRIFT: &str = "10-gno: invalid max clock drift";
    pub const ERR_PROCESSED_TIME_NOT_FOUND: &str = "10-gno: processed time not found";
    pub const ERR_PROCESSED_HEIGHT_NOT_FOUND: &str = "10-gno: processed height not found";
    pub const ERR_DELAY_PERIOD_NOT_PASSED: &str =
        "10-gno: packet-specified delay period has not been reached";
    pub const ERR_TRUSTING_PERIOD_EXPIRED: &str =
        "10-gno: time since latest trusted state has passed the trusting period";
    pub const ERR_UNBONDING_PERIOD_EXPIRED: &str =
        "10-gno: time since latest trusted state has passed the unbonding period";
    pub const ERR_INVALID_PROOF_SPECS: &str = "10-gno: invalid proof specs";
    pub const ERR_INVALID_VALIDATOR_SET: &str = "10-gno: invalid validator set";
    pub const ERR_INVALID_TRUST_LEVEL: &str = "10-gno: invalid trust level";
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
    // #[error(
    //     "untrusted ({untrusted_header_chain_id}) and trusted header ({trusted_header_chain_id}) chain id mismatch"
    // )]
    // ChainIdMismatch {
    //     untrusted_header_chain_id: String,
    //     trusted_header_chain_id: String,
    // },
    // #[error("height of the signed header ({sh_height}) and commit ({commit_height}) mismatch")]
    // SignedHeaderCommitHeightMismatch { sh_height: i64, commit_height: i64 },
    // #[error("hash of the signed header ({sh_hash}) and commit ({commit_hash}) mismatch")]
    // SignedHeaderCommitHashMismatch {
    //     sh_hash: H256<HexUnprefixed>,
    //     commit_hash: H256<HexUnprefixed>,
    // },
    // #[error(
    //     "trusted header height ({untrusted_header_height}) cannot be greater than or equal to the untrusted height ({untrusted_header_height})"
    // )]
    // UntrustedHeaderHeightIsLE {
    //     untrusted_header_height: i64,
    //     trusted_header_height: i64,
    // },
    // #[error(
    //     "trusted header timestamp ({untrusted_header_timestamp}) cannot be greater than or equal to the untrusted timestamp ({untrusted_header_timestamp})"
    // )]
    // UntrustedHeaderTimestampIsLE {
    //     untrusted_header_timestamp: Timestamp,
    //     trusted_header_timestamp: Timestamp,
    // },
    // #[error("expected the untrusted validator set to match the validators hash")]
    // UntrustedValidatorSetMismatch {
    //     expected: H256<HexUnprefixed>,
    //     found: H256<HexUnprefixed>,
    // },
    // #[error("invalid index ({index}) while getting a validator with len ({val_len})")]
    // InvalidIndexInValidatorSet { index: usize, val_len: usize },
    // #[error("double vote from ({0})")]
    // DoubleVote(H160),
    // #[error("not enough voting power, have ({have}), need ({need})")]
    // NotEnoughVotingPower { have: u64, need: u64 },
    // #[error("signature cannot be verified")]
    // SignatureVerification,
    // #[error("max clock drift ({max_clock_drift:?}) check failed against ({timestamp:?})")]
    // MaxClockDriftCheckFailed {
    //     max_clock_drift: Duration,
    //     timestamp: Timestamp,
    // },
    // #[error("next validators hash ({next_validators_hash}) of the trusted header does not match the adjacent header's validators hash ({validators_hash})", next_validators_hash = serde_utils::to_hex(next_validators_hash), validators_hash = serde_utils::to_hex(validators_hash))]
    // NextValidatorsHashMismatch {
    //     next_validators_hash: H256<HexUnprefixed>,
    //     validators_hash: H256<HexUnprefixed>,
    // },
    // #[error("commit signatures length ({sig_len}) does not match the validators len ({val_len})")]
    // InvalidCommitSignaturesLength { sig_len: usize, val_len: usize },
    // #[error("commit height ({commit_height}) does not match the expected height ({height})")]
    // InvalidCommitHeight { commit_height: i64, height: i64 },
    // #[error(
    //     "commit block_id ({commit_block_id:?}) does not match the expected block id ({block_id:?})"
    // )]
    // InvalidCommitBlockId {
    //     commit_block_id: Box<BlockId>,
    //     block_id: Box<BlockId>,
    // },
    // #[error("voting power ({0}) cannot be negative")]
    // NegativeVotingPower(i64),
    // #[error("signature count ({count}) is below the batch verify threshold ({threshold})")]
    // SignatureCountBelowBatchVerifyThreshold { threshold: usize, count: usize },
    // #[error("missing block id hash")]
    // MissingBlockIdHash,
    // #[error("missing part set header hash")]
    // MissingPartSetHeaderHash,
    // #[error("invalid signature length")]
    // InvalidSignatureLength(#[source] InvalidLength),
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
