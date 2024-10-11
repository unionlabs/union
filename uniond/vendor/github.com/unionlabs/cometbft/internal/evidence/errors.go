package evidence

import (
	"errors"
	"fmt"

	"github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/types"
)

var (
	ErrEvidenceAlreadyCommitted = errors.New("evidence was already committed")
	ErrDuplicateEvidence        = errors.New("duplicate evidence")
)

type (
	ErrNoHeaderAtHeight struct {
		Height int64
	}

	ErrNoCommitAtHeight struct {
		Height int64
	}

	ErrUnrecognizedEvidenceType struct {
		Evidence types.Evidence
	}

	// ErrVotingPowerDoesNotMatch is returned when voting power from trusted validator set does not match voting power from evidence.
	ErrVotingPowerDoesNotMatch struct {
		TrustedVotingPower  int64
		EvidenceVotingPower int64
	}

	ErrAddressNotValidatorAtHeight struct {
		Address bytes.HexBytes
		Height  int64
	}

	// ErrValidatorAddressesDoNotMatch is returned when provided DuplicateVoteEvidence's votes have different validators as signers.
	ErrValidatorAddressesDoNotMatch struct {
		ValidatorA bytes.HexBytes
		ValidatorB bytes.HexBytes
	}

	// ErrSameBlockIDs is returned if a duplicate vote evidence has votes from the same block id (should be different).
	ErrSameBlockIDs struct {
		BlockID types.BlockID
	}

	// ErrInvalidEvidenceValidators is returned when evidence validation spots an error related to validator set.
	ErrInvalidEvidenceValidators struct {
		ValError error
	}

	ErrConflictingBlock struct {
		ConflictingBlockError error
	}

	ErrInvalidEvidence struct {
		EvidenceError error
	}

	// ErrDuplicateEvidenceHRTMismatch is returned when double sign evidence's votes are not from the same height, round or type.
	ErrDuplicateEvidenceHRTMismatch struct {
		VoteA types.Vote
		VoteB types.Vote
	}
)

func (e ErrNoHeaderAtHeight) Error() string {
	return fmt.Sprintf("don't have header at height #%d", e.Height)
}

func (e ErrNoCommitAtHeight) Error() string {
	return fmt.Sprintf("don't have commit at height #%d", e.Height)
}

func (e ErrUnrecognizedEvidenceType) Error() string {
	return fmt.Sprintf("unrecognized evidence type: %T", e.Evidence)
}

func (e ErrVotingPowerDoesNotMatch) Error() string {
	return fmt.Sprintf("total voting power from the evidence and our validator set does not match (%d != %d)", e.TrustedVotingPower, e.EvidenceVotingPower)
}

func (e ErrAddressNotValidatorAtHeight) Error() string {
	return fmt.Sprintf("address %X was not a validator at height %d", e.Address, e.Height)
}

func (e ErrValidatorAddressesDoNotMatch) Error() string {
	return fmt.Sprintf("validator addresses do not match: %X vs %X",
		e.ValidatorA,
		e.ValidatorB,
	)
}

func (e ErrSameBlockIDs) Error() string {
	return fmt.Sprintf(
		"block IDs are the same (%v) - not a real duplicate vote",
		e.BlockID,
	)
}

func (e ErrInvalidEvidenceValidators) Error() string {
	return fmt.Sprintf("invalid evidence validators: %v", e.ValError)
}

func (e ErrInvalidEvidenceValidators) Unwrap() error {
	return e.ValError
}

func (e ErrConflictingBlock) Error() string {
	return fmt.Sprintf("conflicting block error: %v", e.ConflictingBlockError)
}

func (e ErrInvalidEvidence) Error() string {
	return fmt.Sprintf("evidence error: %v", e.EvidenceError)
}

func (e ErrDuplicateEvidenceHRTMismatch) Error() string {
	return fmt.Sprintf("h/r/t does not match: %d/%d/%v vs %d/%d/%v",
		e.VoteA.Height, e.VoteA.Round, e.VoteA.Type,
		e.VoteB.Height, e.VoteB.Round, e.VoteB.Type)
}
