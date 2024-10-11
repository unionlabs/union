package simulation

import (
	"context"
	"math/rand"

	"cosmossdk.io/x/gov/types/v1beta1"

	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// SimulateLegacyTextProposalContent returns a random text proposal content.
//
//nolint:staticcheck // used for legacy testing
func SimulateLegacyTextProposalContent(r *rand.Rand, _ context.Context, _ []simtypes.Account) simtypes.Content {
	return v1beta1.NewTextProposal(
		simtypes.RandStringOfLength(r, 140),
		simtypes.RandStringOfLength(r, 5000),
	)
}
