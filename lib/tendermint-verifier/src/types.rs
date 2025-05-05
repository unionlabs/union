use cometbft_types::{
    crypto::public_key::PublicKey,
    types::{canonical_vote::CanonicalVote, commit_sig::CommitSigRaw},
};
use unionlabs::{google::protobuf::timestamp::Timestamp, primitives::H160};

use crate::error::Error;

pub struct ValidatorSig {
    pub validator_address: H160,
    pub timestamp: Timestamp,
    pub signature: Option<Vec<u8>>,
}

pub trait Verification {
    type Error: Into<Error>;

    /// The proto type for `CanonicalVote`. This must be the exact type that is proto encoded
    /// and signed in the consensus algorithm.
    type CanonicalVoteProto: From<CanonicalVote> + prost::Message;

    /// This filter is being used during the signature verification and voting power checking loop.
    /// This filter should filter out all of the non-voters and abstains.
    ///
    /// Return:
    ///     - `Err` to abort the verification entirely,
    ///     - `Ok(None)` to continue with the next iteration in the loop and filter out the signature.
    ///     - `Ok(Some(..))` to process the signature. The expected return values are (validator_address, timestamp, signature),
    ///        if the signature check is aggregated such as BLS, you can return the `signature` as `None` so that
    ///        `process_signature` won't be called but the voting count will be processed.
    fn filter_commit(&self, commit_sig: &CommitSigRaw)
        -> Result<Option<ValidatorSig>, Self::Error>;

    /// On every iteration of the verification loop, if the `filter_commit` returns `Ok(Some(..))` with the signature value
    /// `Some(..)`, this function is being called to be able to process the signature. For example, for the case where you do
    /// ed25519 signature verification one by one, you should do the signature check in this function for every signed validator.
    /// But when it's the verification is batched, you should save the given values for later to check the signature. For the case of
    /// signature aggregation where there is only a single - aggregated - `msg` and `signature`, this function will only be called once
    /// at the `Commit` that holds the `signature`. Returning `Some(..)` as the signature in `filter_commit` triggers this to be called.
    fn process_signature(
        &mut self,
        public_key: PublicKey,
        msg: Option<Vec<u8>>,
        signature: Option<Vec<u8>>,
    ) -> Result<(), Self::Error>;

    /// After the loop finishes, we are providing a way for the light clients to do batched or aggregated signature verification.
    /// But they can also choose to implement this as noop if they don't need it.
    ///
    /// IMPORTANT NOTE: Reset the inner values if you save any intermediate states, so that this type can be reused by the verifier.
    fn finish(&mut self) -> Result<(), Self::Error>;
}
