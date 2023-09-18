package simulation_test

import (
	"fmt"
	"math/rand"
	"testing"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/params/simulation"
	"github.com/cosmos/cosmos-sdk/x/params/types/proposal"
)

type MockParamChange struct {
	n int
}

func (pc MockParamChange) Subspace() string {
	return fmt.Sprintf("test-Subspace%d", pc.n)
}

func (pc MockParamChange) Key() string {
	return fmt.Sprintf("test-Key%d", pc.n)
}

func (pc MockParamChange) ComposedKey() string {
	return fmt.Sprintf("test-ComposedKey%d", pc.n)
}

func (pc MockParamChange) SimValue() simtypes.SimValFn {
	return func(r *rand.Rand) string {
		return fmt.Sprintf("test-value %d%d ", pc.n, int64(simtypes.RandIntBetween(r, 10, 1000)))
	}
}

// make sure that the MockParamChange satisfied the ParamChange interface
var _ simtypes.LegacyParamChange = MockParamChange{}

func TestSimulateParamChangeProposalContent(t *testing.T) {
	s := rand.NewSource(1)
	r := rand.New(s)

	ctx := sdk.NewContext(nil, tmproto.Header{}, true, nil)
	accounts := simtypes.RandomAccounts(r, 3)
	paramChangePool := []simtypes.LegacyParamChange{MockParamChange{1}, MockParamChange{2}, MockParamChange{3}}

	// execute operation
	op := simulation.SimulateParamChangeProposalContent(paramChangePool)
	content := op(r, ctx, accounts)

	require.Equal(t, "desc from SimulateParamChangeProposalContent-0. Random short desc: IivHSlcxgdXhhuTSkuxK", content.GetDescription())
	require.Equal(t, "title from SimulateParamChangeProposalContent-0", content.GetTitle())
	require.Equal(t, "params", content.ProposalRoute())
	require.Equal(t, "ParameterChange", content.ProposalType())

	pcp, ok := content.(*proposal.ParameterChangeProposal)
	require.True(t, ok)

	require.Equal(t, "test-Key2", pcp.Changes[0].GetKey())
	require.Equal(t, "test-value 2791 ", pcp.Changes[0].GetValue())
	require.Equal(t, "test-Subspace2", pcp.Changes[0].GetSubspace())
}
