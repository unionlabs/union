use serde::{Deserialize, Deserializer, Serialize};
use unionlabs::{
    bounded::{BoundedI32, BoundedI64},
    primitives::Bytes,
    result_unwrap,
};

use crate::{BlockId, SignedMsgType, Vote};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Commit {
    pub block_id: BlockId,
    #[serde(deserialize_with = "null_to_default")]
    pub precommits: Vec<Option<Vote>>,
}

fn null_to_default<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let key = Option::<T>::deserialize(de)?;
    Ok(key.unwrap_or_default())
}

impl Commit {
    /// ValidateBasic performs basic validation that doesn't involve state data.
    /// Does not actually check the cryptographic signatures.
    ///
    /// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/block.go#L555>
    pub fn validate_basic(&self) -> Result<(), CommitValidateBasicError> {
        // Genesis-shape: an empty-but-non-nil commit produced by
        // types.NewCommit(BlockID{}, nil). Treat as structurally valid;
        // stateful ValidateBlock distinguishes via
        // block.Height == state.InitialHeight.
        if self.block_id.is_zero() && self.precommits.is_empty() {
            return Ok(());
        }
        if self.block_id.is_zero() {
            return Err(CommitValidateBasicError::NilBlock);
        }
        if self.precommits.is_empty() {
            return Err(CommitValidateBasicError::NoPrecommits);
        }

        let height = self.height();
        let round = self.round();

        // Validate the precommits.
        for precommit in &self.precommits {
            // It's OK for precommits to be missing.
            let Some(precommit) = precommit else { continue };
            // Ensure that all votes are precommits.
            if precommit.ty != SignedMsgType::Precommit {
                return Err(CommitValidateBasicError::InvalidVote { ty: precommit.ty });
            }
            // Ensure that all heights are the same.
            if precommit.height != height {
                return Err(CommitValidateBasicError::InvalidPrecommitHeight {
                    expected: height,
                    got: precommit.height,
                });
            }
            // Ensure that all rounds are the same.
            if precommit.round != round {
                return Err(CommitValidateBasicError::InvalidPrecommitRound {
                    expected: round,
                    got: precommit.round,
                });
            }
        }

        Ok(())
    }

    /// Return the height of this commit.
    ///
    /// The height is memoized in the original Go implementation: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/block.go#L487>
    ///
    /// The value returned from this function is only sound after this commit has been validated by [`Self::validate_basic`].
    pub fn height(&self) -> BoundedI64<0> {
        self.precommits
            .first()
            .and_then(|v| v.as_ref().map(|v| v.height))
            .unwrap_or(const { result_unwrap!(<BoundedI64<0>>::new_const(0)) })
    }

    /// Return the round of this commit.
    ///
    /// The round is memoized in the original Go implementation: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/block.go#L487>
    ///
    /// The value returned from this function is only sound after this commit has been validated by [`Self::validate_basic`].
    pub fn round(&self) -> BoundedI32<-1> {
        self.precommits
            .first()
            .and_then(|v| v.as_ref().map(|v| v.round))
            .unwrap_or(const { result_unwrap!(<BoundedI32<-1>>::new_const(0)) })
    }

    /// VoteSignBytes constructs the SignBytes for the given CommitSig.
    /// The only unique part of the SignBytes is the Timestamp - all other fields
    /// signed over are otherwise the same for all validators.
    /// Panics if valIdx >= commit.Size().
    pub fn vote_sign_bytes(&self, chain_id: String, val_idx: usize) -> Bytes {
        self.get_vote(val_idx)
            .map_or(Bytes::default(), |vote| vote.sign_bytes(chain_id))
    }

    // GetVote converts the CommitSig for the given valIdx to a Vote.
    // Returns nil if the precommit at valIdx is nil.
    // Panics if valIdx >= commit.Size().
    pub fn get_vote(&self, val_idx: usize) -> Option<Vote> {
        let commit_sig = self.precommits[val_idx].clone()?;

        // NOTE: this commitSig might be for a nil blockID,
        // so we can't just use commit.BlockID here.
        // For #1648, CommitSig will need to indicate what BlockID it's for !
        let block_id = commit_sig.block_id;

        Some(Vote {
            ty: SignedMsgType::Precommit,
            height: self.height(),
            round: self.round(),
            block_id,
            timestamp: commit_sig.timestamp,
            validator_address: commit_sig.validator_address,
            validator_index: val_idx as i32,
            signature: commit_sig.signature,
        })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CommitValidateBasicError {
    #[error("No precommits in commit")]
    NilBlock,
    #[error("Commit cannot be for nil block")]
    NoPrecommits,
    #[error("invalid commit vote. Expected precommit, got {}", i32::from(*ty))]
    InvalidVote { ty: SignedMsgType },
    #[error("invalid commit precommit height. Expected {expected}, got {got}")]
    InvalidPrecommitHeight {
        expected: BoundedI64<0>,
        got: BoundedI64<0>,
    },
    #[error("invalid commit precommit round. Expected {expected}, got {got}")]
    InvalidPrecommitRound {
        expected: BoundedI32<-1>,
        got: BoundedI32<-1>,
    },
}
