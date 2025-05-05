use cometbft_types::types::block_id::BlockId;
use unionlabs::{
    errors::InvalidLength,
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    primitives::{encoding::HexUnprefixed, H160, H256},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("client specific")]
    ClientSpecific(#[source] Box<dyn core::error::Error>),
    #[error("integer overflow")]
    IntegerOverflow,
    // TODO: More descriptive message and name
    #[error("invalid header")]
    InvalidHeader,
    #[error("headers must be non-adjacent")]
    HeadersMustBeNonAdjacent,
    #[error("headers must be adjacent")]
    HeadersMustBeAdjacent,
    #[error(
        "header with the timestamp ({header_timestamp}) is \
        expired (trusting period {trusting_period})"
    )]
    HeaderExpired {
        trusting_period: Duration,
        header_timestamp: Timestamp,
    },
    #[error(
        "untrusted ({untrusted_header_chain_id}) and trusted \
        header ({trusted_header_chain_id}) chain id mismatch"
    )]
    ChainIdMismatch {
        untrusted_header_chain_id: String,
        trusted_header_chain_id: String,
    },
    #[error("height of the signed header ({sh_height}) and commit ({commit_height}) mismatch")]
    SignedHeaderCommitHeightMismatch { sh_height: i64, commit_height: i64 },
    #[error("hash of the signed header ({sh_hash}) and commit ({commit_hash}) mismatch")]
    SignedHeaderCommitHashMismatch {
        sh_hash: H256<HexUnprefixed>,
        commit_hash: H256<HexUnprefixed>,
    },
    #[error(
        "trusted header height ({untrusted_header_height}) cannot be greater \
        than or equal to the untrusted height ({untrusted_header_height})"
    )]
    UntrustedHeaderHeightIsLE {
        untrusted_header_height: i64,
        trusted_header_height: i64,
    },
    #[error(
        "trusted header timestamp ({untrusted_header_timestamp}) cannot be greater \
        than or equal to the untrusted timestamp ({untrusted_header_timestamp})"
    )]
    UntrustedHeaderTimestampIsLE {
        untrusted_header_timestamp: Timestamp,
        trusted_header_timestamp: Timestamp,
    },
    #[error("expected the untrusted validator set to match the validators hash")]
    UntrustedValidatorSetMismatch {
        expected: H256<HexUnprefixed>,
        found: H256<HexUnprefixed>,
    },
    #[error("invalid index ({index}) while getting a validator with len ({val_len})")]
    InvalidIndexInValidatorSet { index: usize, val_len: usize },
    #[error("double vote from ({0})")]
    DoubleVote(H160),
    #[error("not enough voting power, have ({have}), need ({need})")]
    NotEnoughVotingPower { have: u64, need: u64 },
    #[error("signature cannot be verified")]
    SignatureVerification,
    #[error("max clock drift ({max_clock_drift:?}) check failed against ({timestamp:?})")]
    MaxClockDriftCheckFailed {
        max_clock_drift: Duration,
        timestamp: Timestamp,
    },
    #[error(
        "next validators hash ({next_validators_hash}) of the trusted \
        header does not match the adjacent header's validators hash ({validators_hash})"
    )]
    NextValidatorsHashMismatch {
        next_validators_hash: H256<HexUnprefixed>,
        validators_hash: H256<HexUnprefixed>,
    },
    #[error("commit signatures length ({sig_len}) does not match the validators len ({val_len})")]
    InvalidCommitSignaturesLength { sig_len: usize, val_len: usize },
    #[error("commit height ({commit_height}) does not match the expected height ({height})")]
    InvalidCommitHeight { commit_height: i64, height: i64 },
    #[error(
        "commit block_id ({commit_block_id:?}) does not \
        match the expected block id ({block_id:?})"
    )]
    InvalidCommitBlockId {
        commit_block_id: Box<BlockId>,
        block_id: Box<BlockId>,
    },
    #[error("voting power ({0}) cannot be negative")]
    NegativeVotingPower(i64),
    #[error("signature count ({count}) is below the batch verify threshold ({threshold})")]
    SignatureCountBelowBatchVerifyThreshold { threshold: usize, count: usize },
    #[error("missing block id hash")]
    MissingBlockIdHash,
    #[error("missing part set header hash")]
    MissingPartSetHeaderHash,
    #[error("invalid signature length")]
    InvalidSignatureLength(#[source] InvalidLength),
}
