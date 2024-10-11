package keeper

import (
	"context"

	"cosmossdk.io/x/group"

	"github.com/cosmos/cosmos-sdk/telemetry"
)

// EndBlocker called at every block, updates proposal's `FinalTallyResult` and
// prunes expired proposals.
func (k Keeper) EndBlocker(ctx context.Context) error {
	start := telemetry.Now()
	defer telemetry.ModuleMeasureSince(group.ModuleName, start, telemetry.MetricKeyEndBlocker)

	if err := k.TallyProposalsAtVPEnd(ctx); err != nil {
		return err
	}

	return k.PruneProposals(ctx)
}
