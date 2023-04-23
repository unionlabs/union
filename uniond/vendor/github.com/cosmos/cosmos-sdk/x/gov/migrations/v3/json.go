package v3

import (
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

// MigrateJSON accepts exported v2 (v0.43) x/gov genesis state and migrates it to
// v3 (V0.46) x/gov genesis state. The migration includes:
//
// - Updating everything to v1.
// - Migrating proposals to be Msg-based.
func MigrateJSON(oldState *v1beta1.GenesisState) (*v1.GenesisState, error) {
	newProps, err := convertToNewProposals(oldState.Proposals)
	if err != nil {
		return nil, err
	}
	newVotes, err := convertToNewVotes(oldState.Votes)
	if err != nil {
		return nil, err
	}

	depParams, votingParms, tallyParams := convertToNewDepParams(oldState.DepositParams), convertToNewVotingParams(oldState.VotingParams), convertToNewTallyParams(oldState.TallyParams)

	return &v1.GenesisState{
		StartingProposalId: oldState.StartingProposalId,
		Deposits:           convertToNewDeposits(oldState.Deposits),
		Votes:              newVotes,
		Proposals:          newProps,
		DepositParams:      &depParams,
		VotingParams:       &votingParms,
		TallyParams:        &tallyParams,
	}, nil
}
