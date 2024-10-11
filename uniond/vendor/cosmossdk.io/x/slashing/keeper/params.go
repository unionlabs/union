package keeper

import (
	"context"
	"time"

	sdkmath "cosmossdk.io/math"
)

// SignedBlocksWindow - sliding window for downtime slashing
func (k Keeper) SignedBlocksWindow(ctx context.Context) (int64, error) {
	params, err := k.Params.Get(ctx)
	return params.SignedBlocksWindow, err
}

// MinSignedPerWindow - minimum blocks signed per window
func (k Keeper) MinSignedPerWindow(ctx context.Context) (int64, error) {
	params, err := k.Params.Get(ctx)
	if err != nil {
		return 0, err
	}

	return params.MinSignedPerWindowInt(), nil
}

// DowntimeJailDuration - Downtime unbond duration
func (k Keeper) DowntimeJailDuration(ctx context.Context) (time.Duration, error) {
	params, err := k.Params.Get(ctx)
	return params.DowntimeJailDuration, err
}

// SlashFractionDoubleSign - fraction of power slashed in case of double sign
func (k Keeper) SlashFractionDoubleSign(ctx context.Context) (sdkmath.LegacyDec, error) {
	params, err := k.Params.Get(ctx)
	return params.SlashFractionDoubleSign, err
}

// SlashFractionDowntime - fraction of power slashed for downtime
func (k Keeper) SlashFractionDowntime(ctx context.Context) (sdkmath.LegacyDec, error) {
	params, err := k.Params.Get(ctx)
	return params.SlashFractionDowntime, err
}
