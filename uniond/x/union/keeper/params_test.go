package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	testkeeper "union/testutil/keeper"
	"union/x/union/types"
)

func TestGetParams(t *testing.T) {
	k, ctx := testkeeper.UnionKeeper(t)
	params := types.DefaultParams()

	k.SetParams(ctx, params)

	require.EqualValues(t, params, k.GetParams(ctx))
}
