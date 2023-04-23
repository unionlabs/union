package v2

import (
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

// migrateJSONWeightedVotes migrates the ADR-037 weighted votes.
func migrateJSONWeightedVotes(oldVotes v1beta1.Votes) v1beta1.Votes {
	newVotes := make(v1beta1.Votes, len(oldVotes))
	for i, oldVote := range oldVotes {
		newVotes[i] = migrateVote(oldVote)
	}

	return newVotes
}

// MigrateJSON accepts exported v1 (v0.40) x/gov genesis state and migrates it to
// v2 (v0.43) x/gov genesis state. The migration includes:
//
// - Gov weighted votes.
func MigrateJSON(oldState *v1beta1.GenesisState) *v1beta1.GenesisState {
	return &v1beta1.GenesisState{
		StartingProposalId: oldState.StartingProposalId,
		Deposits:           oldState.Deposits,
		Votes:              migrateJSONWeightedVotes(oldState.Votes),
		Proposals:          oldState.Proposals,
		DepositParams:      oldState.DepositParams,
		VotingParams:       oldState.VotingParams,
		TallyParams:        oldState.TallyParams,
	}
}
