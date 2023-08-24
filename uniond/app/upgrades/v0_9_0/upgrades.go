package v0_9_0

import (
	"union/app/upgrades"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	upgradetypes "github.com/cosmos/cosmos-sdk/x/upgrade/types"
)

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers) upgradetypes.UpgradeHandler {
	return func(ctx sdk.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		stakingParams := keepers.StakingKeeper.GetParams(ctx)
		stakingParams.EpochLength = 100
		stakingParams.JailedValidatorThreshold = 50
		keepers.StakingKeeper.SetParams(ctx, stakingParams)
		return mm.RunMigrations(ctx, configurator, vm)
	}
}
