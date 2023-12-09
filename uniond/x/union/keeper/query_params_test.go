package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	testkeeper "union/testutil/keeper"
	"union/x/union/types"
)

func TestParamsQuery(t *testing.T) {
	keeper, ctx := testkeeper.UnionKeeper(t)
	params := types.DefaultParams()
	keeper.SetParams(ctx, params)

	response, err := keeper.Params(ctx, &types.QueryParamsRequest{})
	require.NoError(t, err)
	require.Equal(t, &types.QueryParamsResponse{Params: params}, response)
}
