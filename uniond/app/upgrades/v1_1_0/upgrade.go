package v1_1_0

import (
	"context"

	upgradetypes "cosmossdk.io/x/upgrade/types"
	"github.com/cosmos/cosmos-sdk/types/module"

	"github.com/unionlabs/union/uniond/app/upgrades"
)

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers, getKey upgrades.GetKeyFunc) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		return mm.RunMigrations(ctx, configurator, vm)
	}
}
