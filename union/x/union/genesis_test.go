package union_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	keepertest "union/testutil/keeper"
	"union/testutil/nullify"
	"union/x/union"
	"union/x/union/types"
)

func TestGenesis(t *testing.T) {
	genesisState := types.GenesisState{
		Params: types.DefaultParams(),

		// this line is used by starport scaffolding # genesis/test/state
	}

	k, ctx := keepertest.UnionKeeper(t)
	union.InitGenesis(ctx, *k, genesisState)
	got := union.ExportGenesis(ctx, *k)
	require.NotNil(t, got)

	nullify.Fill(&genesisState)
	nullify.Fill(got)

	// this line is used by starport scaffolding # genesis/test/assert
}
