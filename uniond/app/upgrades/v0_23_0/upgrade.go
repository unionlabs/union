package v0_23_0

import (
	"context"
	"union/app/upgrades"
	"union/x/deferredack"
	datypes "union/x/deferredack/types"

	upgradetypes "cosmossdk.io/x/upgrade/types"
	"github.com/cosmos/cosmos-sdk/types/module"
)

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		vm[datypes.ModuleName] = deferredack.ConsensusVersion
		return mm.RunMigrations(ctx, configurator, vm)
	}
}
