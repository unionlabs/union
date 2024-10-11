package keeper

import (
	"context"

	"cosmossdk.io/x/mint/types"

	"github.com/cosmos/cosmos-sdk/telemetry"
)

// BeginBlocker mints new tokens for the previous block.
func (k Keeper) BeginBlocker(ctx context.Context, mintFn types.MintFn) error {
	start := telemetry.Now()
	defer telemetry.ModuleMeasureSince(types.ModuleName, start, telemetry.MetricKeyBeginBlocker)

	// fetch stored minter & params
	minter, err := k.Minter.Get(ctx)
	if err != nil {
		return err
	}

	oldMinter := minter

	// we pass -1 as epoch number to indicate that this is not an epoch minting,
	// but a regular block minting. Same with epoch id "block".
	err = mintFn(ctx, k.Environment, &minter, "block", -1)
	if err != nil {
		return err
	}

	if minter.IsEqual(oldMinter) {
		return nil
	}

	return k.Minter.Set(ctx, minter)
}
