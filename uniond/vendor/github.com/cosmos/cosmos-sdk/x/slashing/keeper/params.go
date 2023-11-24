package keeper

import (
	"context"
	"time"

	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

// SignedBlocksWindow - sliding window for downtime slashing
func (k Keeper) SignedBlocksWindow(ctx context.Context) (int64, error) {
	params, err := k.GetParams(ctx)
	return params.SignedBlocksWindow, err
}

// MinSignedPerWindow - minimum blocks signed per window
func (k Keeper) MinSignedPerWindow(ctx context.Context) (int64, error) {
	params, err := k.GetParams(ctx)
	if err != nil {
		return 0, err
	}

	signedBlocksWindow := params.SignedBlocksWindow
	minSignedPerWindow := params.MinSignedPerWindow

	// NOTE: RoundInt64 will never panic as minSignedPerWindow is
	//       less than 1.
	return minSignedPerWindow.MulInt64(signedBlocksWindow).RoundInt64(), nil
}

// DowntimeJailDuration - Downtime unbond duration
func (k Keeper) DowntimeJailDuration(ctx context.Context) (time.Duration, error) {
	params, err := k.GetParams(ctx)
	return params.DowntimeJailDuration, err
}

// SlashFractionDoubleSign - fraction of power slashed in case of double sign
func (k Keeper) SlashFractionDoubleSign(ctx context.Context) (sdkmath.LegacyDec, error) {
	params, err := k.GetParams(ctx)
	return params.SlashFractionDoubleSign, err
}

// SlashFractionDowntime - fraction of power slashed for downtime
func (k Keeper) SlashFractionDowntime(ctx context.Context) (sdkmath.LegacyDec, error) {
	params, err := k.GetParams(ctx)
	return params.SlashFractionDowntime, err
}

// GetParams returns the current x/slashing module parameters.
func (k Keeper) GetParams(ctx context.Context) (params types.Params, err error) {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := store.Get(types.ParamsKey)
	if err != nil {
		return params, err
	}
	if bz == nil {
		return params, nil
	}

	err = k.cdc.Unmarshal(bz, &params)
	return params, err
}

// SetParams sets the x/slashing module parameters.
// CONTRACT: This method performs no validation of the parameters.
func (k Keeper) SetParams(ctx context.Context, params types.Params) error {
	store := k.storeService.OpenKVStore(ctx)
	bz, err := k.cdc.Marshal(&params)
	if err != nil {
		return err
	}
	return store.Set(types.ParamsKey, bz)
}
