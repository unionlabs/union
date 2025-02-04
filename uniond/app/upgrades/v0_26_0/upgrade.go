package v0_26_0

import (
	"context"
	"cosmossdk.io/core/appmodule"
	upgradetypes "cosmossdk.io/x/upgrade/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"union/app/upgrades"
)

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm appmodule.VersionMap) (appmodule.VersionMap, error) {
		return mm.RunMigrations(ctx, configurator, vm)
	}
}
