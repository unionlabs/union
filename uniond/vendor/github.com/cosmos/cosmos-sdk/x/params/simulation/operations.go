package simulation

import (
	"fmt"
	"math/rand"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/params/types/proposal"
)

func min(a int, b int) int {
	if a <= b {
		return a
	}
	return b
}

// SimulateParamChangeProposalContent returns random parameter change content.
// It will generate a ParameterChangeProposal object with anywhere between 1 and
// the total amount of defined parameters changes, all of which have random valid values.
func SimulateParamChangeProposalContent(paramChangePool []simulation.LegacyParamChange) simulation.ContentSimulatorFn { //nolint:staticcheck
	numProposals := 0
	// Bound the maximum number of simultaneous parameter changes
	maxSimultaneousParamChanges := min(len(paramChangePool), 1000)
	if maxSimultaneousParamChanges == 0 {
		panic("param changes array is empty")
	}

	return func(r *rand.Rand, _ sdk.Context, _ []simulation.Account) simulation.Content { //nolint:staticcheck
		numChanges := simulation.RandIntBetween(r, 1, maxSimultaneousParamChanges)
		paramChanges := make([]proposal.ParamChange, numChanges)

		// perm here takes at most len(paramChangePool) calls to random
		paramChoices := r.Perm(len(paramChangePool))

		for i := 0; i < numChanges; i++ {
			spc := paramChangePool[paramChoices[i]]
			// add a new distinct parameter to the set of changes
			paramChanges[i] = proposal.NewParamChange(spc.Subspace(), spc.Key(), spc.SimValue()(r))
		}

		title := fmt.Sprintf("title from SimulateParamChangeProposalContent-%d", numProposals)
		desc := fmt.Sprintf("desc from SimulateParamChangeProposalContent-%d. Random short desc: %s",
			numProposals, simulation.RandStringOfLength(r, 20))
		numProposals++
		return proposal.NewParameterChangeProposal(
			title,        // title
			desc,         // description
			paramChanges, // set of changes
		)
	}
}
