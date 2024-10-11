package keeper

import (
	"context"

	"cosmossdk.io/math"
	v1 "cosmossdk.io/x/gov/types/v1"
)

// CalculateVoteResultsAndVotingPowerFn is a function signature for calculating vote results and voting power
// It can be overridden to customize the voting power calculation for proposals
// It gets the proposal tallied and the validators governance infos (bonded tokens, voting power, etc.)
// It must return the total voting power and the results of the vote
type CalculateVoteResultsAndVotingPowerFn func(
	ctx context.Context,
	keeper Keeper,
	proposalID uint64,
	validators map[string]v1.ValidatorGovInfo,
) (totalVoterPower math.LegacyDec, results map[v1.VoteOption]math.LegacyDec, err error)

// Config is a config struct used for initializing the gov module to avoid using globals.
type Config struct {
	// MaxTitleLen defines the amount of characters that can be used for proposal title
	MaxTitleLen uint64
	// MaxMetadataLen defines the amount of characters that can be used for proposal metadata
	MaxMetadataLen uint64
	// MaxSummaryLen defines the amount of characters that can be used for proposal summary
	MaxSummaryLen uint64
	// MaxVoteOptionsLen defines the maximum number of vote options a proposal can have.
	// This only applies to WeightedVoteOption messages and not to the VoteOption messages
	// 0 means this param is disabled, hence all supported options are allowed
	MaxVoteOptionsLen uint64
	// CalculateVoteResultsAndVotingPowerFn is a function signature for calculating vote results and voting power
	// Keeping it nil will use the default implementation
	CalculateVoteResultsAndVotingPowerFn CalculateVoteResultsAndVotingPowerFn
}

// DefaultConfig returns the default config for gov.
func DefaultConfig() Config {
	return Config{
		MaxTitleLen:                          255,
		MaxMetadataLen:                       255,
		MaxSummaryLen:                        10200,
		MaxVoteOptionsLen:                    0, // 0 means this param is disabled, hence all supported options are allowed
		CalculateVoteResultsAndVotingPowerFn: nil,
	}
}
