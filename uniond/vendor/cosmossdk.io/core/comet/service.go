package comet

import (
	"context"
	"time"
)

// Service is an interface that can be used to get information specific to Comet
type Service interface {
	CometInfo(context.Context) Info
}

// Info is the information comet provides apps in ABCI
type Info struct {
	Evidence []Evidence // Evidence misbehavior of the block
	// ValidatorsHash returns the hash of the validators
	// For Comet, it is the hash of the next validator set
	ValidatorsHash  []byte
	ProposerAddress []byte     // ProposerAddress is  the address of the block proposer
	LastCommit      CommitInfo // DecidedLastCommit returns the last commit info
}

// MisbehaviorType is the type of misbehavior for a validator
type MisbehaviorType int32

const (
	Unknown           MisbehaviorType = 0
	DuplicateVote     MisbehaviorType = 1
	LightClientAttack MisbehaviorType = 2
)

// Evidence is the misbehavior information of ABCI
type Evidence struct {
	Type             MisbehaviorType
	Validator        Validator
	Height           int64
	Time             time.Time
	TotalVotingPower int64
}

// CommitInfo is the commit information of ABCI
type CommitInfo struct {
	Round int32
	Votes []VoteInfo
}

// VoteInfo is the vote information of ABCI
type VoteInfo struct {
	Validator   Validator
	BlockIDFlag BlockIDFlag
}

// BlockIDFlag indicates which BlockID the signature is for
type BlockIDFlag int32

const (
	BlockIDFlagUnknown BlockIDFlag = 0
	// BlockIDFlagAbsent - no vote was received from a validator.
	BlockIDFlagAbsent BlockIDFlag = 1
	// BlockIDFlagCommit - voted for the Commit.BlockID.
	BlockIDFlagCommit BlockIDFlag = 2
	// BlockIDFlagNil - voted for nil.
	BlockIDFlagNil BlockIDFlag = 3
)

// Validator is the validator information of ABCI
type Validator struct {
	Address []byte
	Power   int64
}
