package keeper

import (
	"context"

	"cosmossdk.io/x/distribution/types"

	"github.com/cosmos/cosmos-sdk/telemetry"
)

// BeginBlocker sets the proposer for determining distribution during endblock
// and distribute rewards for the previous block.
func (k Keeper) BeginBlocker(ctx context.Context) error {
	start := telemetry.Now()
	defer telemetry.ModuleMeasureSince(types.ModuleName, start, telemetry.MetricKeyBeginBlocker)

	// determine the total power signing the block
	var previousTotalPower int64
	header := k.HeaderService.HeaderInfo(ctx)
	ci := k.cometService.CometInfo(ctx)
	for _, vote := range ci.LastCommit.Votes {
		previousTotalPower += vote.Validator.Power
	}

	// TODO this is Tendermint-dependent
	// ref https://github.com/cosmos/cosmos-sdk/issues/3095
	if header.Height > 1 {
		if err := k.AllocateTokens(ctx, previousTotalPower, ci.LastCommit.Votes); err != nil {
			return err
		}

		// every 1000 blocks send whole coins from decimal pool to community pool
		if header.Height%1000 == 0 {
			if err := k.sendDecimalPoolToCommunityPool(ctx); err != nil {
				return err
			}
		}
	}

	return nil
}
