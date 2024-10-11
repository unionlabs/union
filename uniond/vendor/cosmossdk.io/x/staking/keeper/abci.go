package keeper

import (
	"context"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/telemetry"
)

// EndBlocker called at every block, update validator set
func (k *Keeper) EndBlocker(ctx context.Context) ([]appmodule.ValidatorUpdate, error) {
	start := telemetry.Now()
	defer telemetry.ModuleMeasureSince(types.ModuleName, start, telemetry.MetricKeyEndBlocker)

	return k.BlockValidatorUpdates(ctx)
}
