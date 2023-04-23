package v4

import (
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
)

// MigrateJSON accepts exported v3 (v0.46) x/gov genesis state and migrates it to
// v4 (v0.47) x/gov genesis state. The migration includes:
//
// Params migrations from x/params to gov
// Addition of the new min initial deposit ratio parameter that is set to 0 by default.
// Proposals in voting period are tracked in a separate index.
func MigrateJSON(oldState *v1.GenesisState) (*v1.GenesisState, error) {
	defaultParams := v1.DefaultParams()

	params := v1.NewParams(
		oldState.DepositParams.MinDeposit,
		*oldState.DepositParams.MaxDepositPeriod,
		*oldState.VotingParams.VotingPeriod,
		oldState.TallyParams.Quorum,
		oldState.TallyParams.Threshold,
		oldState.TallyParams.VetoThreshold,
		defaultParams.MinInitialDepositRatio,
		defaultParams.BurnProposalDepositPrevote,
		defaultParams.BurnVoteQuorum,
		defaultParams.BurnVoteVeto,
	)

	return &v1.GenesisState{
		StartingProposalId: oldState.StartingProposalId,
		Deposits:           oldState.Deposits,
		Votes:              oldState.Votes,
		Proposals:          oldState.Proposals,
		Params:             &params,
	}, nil
}
