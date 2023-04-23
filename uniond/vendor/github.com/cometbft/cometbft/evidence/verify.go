package evidence

import (
	"bytes"
	"errors"
	"fmt"
	"time"

	"github.com/cometbft/cometbft/light"
	"github.com/cometbft/cometbft/types"
)

// verify verifies the evidence fully by checking:
// - It has not already been committed
// - it is sufficiently recent (MaxAge)
// - it is from a key who was a validator at the given height
// - it is internally consistent with state
// - it was properly signed by the alleged equivocator and meets the individual evidence verification requirements
func (evpool *Pool) verify(evidence types.Evidence) error {
	var (
		state          = evpool.State()
		height         = state.LastBlockHeight
		evidenceParams = state.ConsensusParams.Evidence
		ageNumBlocks   = height - evidence.Height()
	)

	// verify the time of the evidence
	blockMeta := evpool.blockStore.LoadBlockMeta(evidence.Height())
	if blockMeta == nil {
		return fmt.Errorf("don't have header #%d", evidence.Height())
	}
	evTime := blockMeta.Header.Time
	if evidence.Time() != evTime {
		return fmt.Errorf("evidence has a different time to the block it is associated with (%v != %v)",
			evidence.Time(), evTime)
	}
	ageDuration := state.LastBlockTime.Sub(evTime)

	// check that the evidence hasn't expired
	if ageDuration > evidenceParams.MaxAgeDuration && ageNumBlocks > evidenceParams.MaxAgeNumBlocks {
		return fmt.Errorf(
			"evidence from height %d (created at: %v) is too old; min height is %d and evidence can not be older than %v",
			evidence.Height(),
			evTime,
			height-evidenceParams.MaxAgeNumBlocks,
			state.LastBlockTime.Add(evidenceParams.MaxAgeDuration),
		)
	}

	// apply the evidence-specific verification logic
	switch ev := evidence.(type) {
	case *types.DuplicateVoteEvidence:
		valSet, err := evpool.stateDB.LoadValidators(evidence.Height())
		if err != nil {
			return err
		}
		return VerifyDuplicateVote(ev, state.ChainID, valSet)

	case *types.LightClientAttackEvidence:
		commonHeader, err := getSignedHeader(evpool.blockStore, evidence.Height())
		if err != nil {
			return err
		}
		commonVals, err := evpool.stateDB.LoadValidators(evidence.Height())
		if err != nil {
			return err
		}
		trustedHeader := commonHeader
		// in the case of lunatic the trusted header is different to the common header
		if evidence.Height() != ev.ConflictingBlock.Height {
			trustedHeader, err = getSignedHeader(evpool.blockStore, ev.ConflictingBlock.Height)
			if err != nil {
				// FIXME: This multi step process is a bit unergonomic. We may want to consider a more efficient process
				// that doesn't require as much io and is atomic.

				// If the node doesn't have a block at the height of the conflicting block, then this could be
				// a forward lunatic attack. Thus the node must get the latest height it has
				latestHeight := evpool.blockStore.Height()
				trustedHeader, err = getSignedHeader(evpool.blockStore, latestHeight)
				if err != nil {
					return err
				}
				if trustedHeader.Time.Before(ev.ConflictingBlock.Time) {
					return fmt.Errorf("latest block time (%v) is before conflicting block time (%v)",
						trustedHeader.Time, ev.ConflictingBlock.Time,
					)
				}
			}
		}

		err = VerifyLightClientAttack(ev, commonHeader, trustedHeader, commonVals, state.LastBlockTime,
			state.ConsensusParams.Evidence.MaxAgeDuration)
		if err != nil {
			return err
		}
		return nil
	default:
		return fmt.Errorf("unrecognized evidence type: %T", evidence)
	}

}

// VerifyLightClientAttack verifies LightClientAttackEvidence against the state of the full node. This involves
// the following checks:
//   - the common header from the full node has at least 1/3 voting power which is also present in
//     the conflicting header's commit
//   - 2/3+ of the conflicting validator set correctly signed the conflicting block
//   - the nodes trusted header at the same height as the conflicting header has a different hash
//
// CONTRACT: must run ValidateBasic() on the evidence before verifying
//
//	must check that the evidence has not expired (i.e. is outside the maximum age threshold)
func VerifyLightClientAttack(e *types.LightClientAttackEvidence, commonHeader, trustedHeader *types.SignedHeader,
	commonVals *types.ValidatorSet, now time.Time, trustPeriod time.Duration) error {
	// In the case of lunatic attack there will be a different commonHeader height. Therefore the node perform a single
	// verification jump between the common header and the conflicting one
	if commonHeader.Height != e.ConflictingBlock.Height {
		err := commonVals.VerifyCommitLightTrusting(trustedHeader.ChainID, e.ConflictingBlock.Commit, light.DefaultTrustLevel)
		if err != nil {
			return fmt.Errorf("skipping verification of conflicting block failed: %w", err)
		}

		// In the case of equivocation and amnesia we expect all header hashes to be correctly derived
	} else if e.ConflictingHeaderIsInvalid(trustedHeader.Header) {
		return errors.New("common height is the same as conflicting block height so expected the conflicting" +
			" block to be correctly derived yet it wasn't")
	}

	// Verify that the 2/3+ commits from the conflicting validator set were for the conflicting header
	if err := e.ConflictingBlock.ValidatorSet.VerifyCommitLight(trustedHeader.ChainID, e.ConflictingBlock.Commit.BlockID,
		e.ConflictingBlock.Height, e.ConflictingBlock.Commit); err != nil {
		return fmt.Errorf("invalid commit from conflicting block: %w", err)
	}

	// Assert the correct amount of voting power of the validator set
	if evTotal, valsTotal := e.TotalVotingPower, commonVals.TotalVotingPower(); evTotal != valsTotal {
		return fmt.Errorf("total voting power from the evidence and our validator set does not match (%d != %d)",
			evTotal, valsTotal)
	}

	// check in the case of a forward lunatic attack that monotonically increasing time has been violated
	if e.ConflictingBlock.Height > trustedHeader.Height && e.ConflictingBlock.Time.After(trustedHeader.Time) {
		return fmt.Errorf("conflicting block doesn't violate monotonically increasing time (%v is after %v)",
			e.ConflictingBlock.Time, trustedHeader.Time,
		)

		// In all other cases check that the hashes of the conflicting header and the trusted header are different
	} else if bytes.Equal(trustedHeader.Hash(), e.ConflictingBlock.Hash()) {
		return fmt.Errorf("trusted header hash matches the evidence's conflicting header hash: %X",
			trustedHeader.Hash())
	}

	return validateABCIEvidence(e, commonVals, trustedHeader)
}

// VerifyDuplicateVote verifies DuplicateVoteEvidence against the state of full node. This involves the
// following checks:
//   - the validator is in the validator set at the height of the evidence
//   - the height, round, type and validator address of the votes must be the same
//   - the block ID's must be different
//   - The signatures must both be valid
func VerifyDuplicateVote(e *types.DuplicateVoteEvidence, chainID string, valSet *types.ValidatorSet) error {
	_, val := valSet.GetByAddress(e.VoteA.ValidatorAddress)
	if val == nil {
		return fmt.Errorf("address %X was not a validator at height %d", e.VoteA.ValidatorAddress, e.Height())
	}
	pubKey := val.PubKey

	// H/R/S must be the same
	if e.VoteA.Height != e.VoteB.Height ||
		e.VoteA.Round != e.VoteB.Round ||
		e.VoteA.Type != e.VoteB.Type {
		return fmt.Errorf("h/r/s does not match: %d/%d/%v vs %d/%d/%v",
			e.VoteA.Height, e.VoteA.Round, e.VoteA.Type,
			e.VoteB.Height, e.VoteB.Round, e.VoteB.Type)
	}

	// Address must be the same
	if !bytes.Equal(e.VoteA.ValidatorAddress, e.VoteB.ValidatorAddress) {
		return fmt.Errorf("validator addresses do not match: %X vs %X",
			e.VoteA.ValidatorAddress,
			e.VoteB.ValidatorAddress,
		)
	}

	// BlockIDs must be different
	if e.VoteA.BlockID.Equals(e.VoteB.BlockID) {
		return fmt.Errorf(
			"block IDs are the same (%v) - not a real duplicate vote",
			e.VoteA.BlockID,
		)
	}

	// pubkey must match address (this should already be true, sanity check)
	addr := e.VoteA.ValidatorAddress
	if !bytes.Equal(pubKey.Address(), addr) {
		return fmt.Errorf("address (%X) doesn't match pubkey (%v - %X)",
			addr, pubKey, pubKey.Address())
	}

	// validator voting power and total voting power must match
	if val.VotingPower != e.ValidatorPower {
		return fmt.Errorf("validator power from evidence and our validator set does not match (%d != %d)",
			e.ValidatorPower, val.VotingPower)
	}
	if valSet.TotalVotingPower() != e.TotalVotingPower {
		return fmt.Errorf("total voting power from the evidence and our validator set does not match (%d != %d)",
			e.TotalVotingPower, valSet.TotalVotingPower())
	}

	va := e.VoteA.ToProto()
	vb := e.VoteB.ToProto()
	// Signatures must be valid
	if !pubKey.VerifySignature(types.VoteSignBytes(chainID, va), e.VoteA.Signature) {
		return fmt.Errorf("verifying VoteA: %w", types.ErrVoteInvalidSignature)
	}
	if !pubKey.VerifySignature(types.VoteSignBytes(chainID, vb), e.VoteB.Signature) {
		return fmt.Errorf("verifying VoteB: %w", types.ErrVoteInvalidSignature)
	}

	return nil
}

// validateABCIEvidence validates the ABCI component of the light client attack
// evidence i.e voting power and byzantine validators
func validateABCIEvidence(
	ev *types.LightClientAttackEvidence,
	commonVals *types.ValidatorSet,
	trustedHeader *types.SignedHeader,
) error {
	if evTotal, valsTotal := ev.TotalVotingPower, commonVals.TotalVotingPower(); evTotal != valsTotal {
		return fmt.Errorf("total voting power from the evidence and our validator set does not match (%d != %d)",
			evTotal, valsTotal)
	}

	// Find out what type of attack this was and thus extract the malicious
	// validators. Note, in the case of an Amnesia attack we don't have any
	// malicious validators.
	validators := ev.GetByzantineValidators(commonVals, trustedHeader)

	// Ensure this matches the validators that are listed in the evidence. They
	// should be ordered based on power.
	if validators == nil && ev.ByzantineValidators != nil {
		return fmt.Errorf(
			"expected nil validators from an amnesia light client attack but got %d",
			len(ev.ByzantineValidators),
		)
	}

	if exp, got := len(validators), len(ev.ByzantineValidators); exp != got {
		return fmt.Errorf("expected %d byzantine validators from evidence but got %d", exp, got)
	}

	for idx, val := range validators {
		if !bytes.Equal(ev.ByzantineValidators[idx].Address, val.Address) {
			return fmt.Errorf(
				"evidence contained an unexpected byzantine validator address; expected: %v, got: %v",
				val.Address, ev.ByzantineValidators[idx].Address,
			)
		}

		if ev.ByzantineValidators[idx].VotingPower != val.VotingPower {
			return fmt.Errorf(
				"evidence contained unexpected byzantine validator power; expected %d, got %d",
				val.VotingPower, ev.ByzantineValidators[idx].VotingPower,
			)
		}
	}

	return nil
}

func getSignedHeader(blockStore BlockStore, height int64) (*types.SignedHeader, error) {
	blockMeta := blockStore.LoadBlockMeta(height)
	if blockMeta == nil {
		return nil, fmt.Errorf("don't have header at height #%d", height)
	}
	commit := blockStore.LoadBlockCommit(height)
	if commit == nil {
		return nil, fmt.Errorf("don't have commit at height #%d", height)
	}
	return &types.SignedHeader{
		Header: &blockMeta.Header,
		Commit: commit,
	}, nil
}
