package keeper

import (
	"context"
	"errors"

	"union/x/tokenfactory/types"
)

// GetParams returns the total set params.
func (k Keeper) GetParams(ctx context.Context) (params types.Params) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get([]byte(types.ParamsKey))
	if err != nil {
		panic(err)
	}
	if bz == nil { // only panic on unset params and not on empty params
		panic(errors.New("tokenfactory params are not set in store"))
	}

	k.cdc.MustUnmarshal(bz, &params)
	return params
}

// SetParams sets the total set of params.
func (k Keeper) SetParams(ctx context.Context, params types.Params) {
	store := k.storeService.OpenKVStore(ctx)
	bz := k.cdc.MustMarshal(&params)
	if err := store.Set([]byte(types.ParamsKey), bz); err != nil {
		panic(err)
	}
}
