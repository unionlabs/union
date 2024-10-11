package keeper

import (
	"context"

	"cosmossdk.io/x/epochs/types"
)

// Hooks gets the hooks for governance Keeper
func (k Keeper) Hooks() types.EpochHooks {
	if k.hooks == nil {
		// return a no-op implementation if no hooks are set
		return types.MultiEpochHooks{}
	}

	return k.hooks
}

// AfterEpochEnd gets called at the end of the epoch, end of epoch is the timestamp of first block produced after epoch duration.
func (k Keeper) AfterEpochEnd(ctx context.Context, identifier string, epochNumber int64) error {
	return k.Hooks().AfterEpochEnd(ctx, identifier, epochNumber)
}

// BeforeEpochStart new epoch is next block of epoch end block
func (k Keeper) BeforeEpochStart(ctx context.Context, identifier string, epochNumber int64) error {
	return k.Hooks().BeforeEpochStart(ctx, identifier, epochNumber)
}
