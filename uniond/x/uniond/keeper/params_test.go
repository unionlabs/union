package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	keepertest "github.com/unionlabs/union/uniond/testutil/keeper"
	"github.com/unionlabs/union/uniond/x/uniond/types"
)

func TestGetParams(t *testing.T) {
	k, ctx := keepertest.UniondKeeper(t)
	params := types.DefaultParams()

	require.NoError(t, k.SetParams(ctx, params))
	require.EqualValues(t, params, k.GetParams(ctx))
}
