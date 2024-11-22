package keeper_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	keepertest "github.com/unionlabs/union/uniond/testutil/keeper"
	"github.com/unionlabs/union/uniond/x/uniond/types"
)

func TestParamsQuery(t *testing.T) {
	keeper, ctx := keepertest.UniondKeeper(t)
	params := types.DefaultParams()
	require.NoError(t, keeper.SetParams(ctx, params))

	response, err := keeper.Params(ctx, &types.QueryParamsRequest{})
	require.NoError(t, err)
	require.Equal(t, &types.QueryParamsResponse{Params: params}, response)
}
