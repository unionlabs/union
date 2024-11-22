package uniond_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	keepertest "github.com/unionlabs/union/uniond/testutil/keeper"
	"github.com/unionlabs/union/uniond/testutil/nullify"
	uniond "github.com/unionlabs/union/uniond/x/uniond/module"
	"github.com/unionlabs/union/uniond/x/uniond/types"
)

func TestGenesis(t *testing.T) {
	genesisState := types.GenesisState{
		Params: types.DefaultParams(),

		// this line is used by starport scaffolding # genesis/test/state
	}

	k, ctx := keepertest.UniondKeeper(t)
	uniond.InitGenesis(ctx, k, genesisState)
	got := uniond.ExportGenesis(ctx, k)
	require.NotNil(t, got)

	nullify.Fill(&genesisState)
	nullify.Fill(got)

	// this line is used by starport scaffolding # genesis/test/assert
}
