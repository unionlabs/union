package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v7/modules/core/03-connection/types"
)

// GetMaxExpectedTimePerBlock retrieves the maximum expected time per block from the paramstore
func (k Keeper) GetMaxExpectedTimePerBlock(ctx sdk.Context) uint64 {
	var res uint64
	k.paramSpace.Get(ctx, types.KeyMaxExpectedTimePerBlock, &res)
	return res
}

// GetParams returns the total set of ibc-connection parameters.
func (k Keeper) GetParams(ctx sdk.Context) types.Params {
	return types.NewParams(k.GetMaxExpectedTimePerBlock(ctx))
}

// SetParams sets the total set of ibc-connection parameters.
func (k Keeper) SetParams(ctx sdk.Context, params types.Params) {
	k.paramSpace.SetParamSet(ctx, &params)
}
