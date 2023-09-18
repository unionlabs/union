package simulation_test

import (
	"math/rand"
	"testing"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/params/simulation"
	"github.com/cosmos/cosmos-sdk/x/params/types/proposal"
)

func TestProposalContents(t *testing.T) {
	// initialize parameters
	s := rand.NewSource(1)
	r := rand.New(s)

	ctx := sdk.NewContext(nil, tmproto.Header{}, true, nil)
	accounts := simtypes.RandomAccounts(r, 3)

	paramChangePool := []simtypes.LegacyParamChange{MockParamChange{1}, MockParamChange{2}, MockParamChange{3}}

	// execute ProposalContents function
	weightedProposalContent := simulation.ProposalContents(paramChangePool)
	require.Len(t, weightedProposalContent, 1)

	w0 := weightedProposalContent[0]

	// tests w0 interface:
	require.Equal(t, simulation.OpWeightSubmitParamChangeProposal, w0.AppParamsKey())
	require.Equal(t, simulation.DefaultWeightParamChangeProposal, w0.DefaultWeight())

	content := w0.ContentSimulatorFn()(r, ctx, accounts)

	require.Equal(t, "desc from SimulateParamChangeProposalContent-0. Random short desc: IivHSlcxgdXhhuTSkuxK", content.GetDescription())
	require.Equal(t, "title from SimulateParamChangeProposalContent-0", content.GetTitle())
	require.Equal(t, "params", content.ProposalRoute())
	require.Equal(t, "ParameterChange", content.ProposalType())

	pcp, ok := content.(*proposal.ParameterChangeProposal)
	require.True(t, ok)

	require.Len(t, pcp.Changes, 1)
	require.Equal(t, "test-Key2", pcp.Changes[0].GetKey())
	require.Equal(t, "test-value 2791 ", pcp.Changes[0].GetValue())
	require.Equal(t, "test-Subspace2", pcp.Changes[0].GetSubspace())
}
