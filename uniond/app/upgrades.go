package app

import (
	"fmt"
	"union/app/upgrades"
	"union/app/upgrades/v0_10_0"
	"union/app/upgrades/v0_11_0"
	"union/app/upgrades/v0_12_0"
	"union/app/upgrades/v0_13_0"
	"union/app/upgrades/v0_14_0"
	"union/app/upgrades/v0_9_0"

	upgradetypes "github.com/cosmos/cosmos-sdk/x/upgrade/types"
)

var Upgrades = []upgrades.Upgrade{v0_9_0.Upgrade, v0_10_0.Upgrade, v0_11_0.Upgrade, v0_12_0.Upgrade, v0_13_0.Upgrade, v0_14_0.Upgrade}

// configure store loader that checks if version == upgradeHeight and applies store upgrades
func (app *UnionApp) setupUpgradeStoreLoaders() {
	upgradeInfo, err := app.UpgradeKeeper.ReadUpgradeInfoFromDisk()
	if err != nil {
		panic(fmt.Sprintf("failed to read upgrade info from disk %s", err))
	}

	if app.UpgradeKeeper.IsSkipHeight(upgradeInfo.Height) {
		return
	}

	for _, upgrade := range Upgrades {
		if upgradeInfo.Name == upgrade.UpgradeName {
			app.SetStoreLoader(upgradetypes.UpgradeStoreLoader(upgradeInfo.Height, &upgrade.StoreUpgrades))
		}
	}
}

func (app *UnionApp) setupUpgradeHandlers() {
	for _, upgrade := range Upgrades {
		app.UpgradeKeeper.SetUpgradeHandler(
			upgrade.UpgradeName,
			upgrade.CreateUpgradeHandler(
				app.mm,
				app.configurator,
				&upgrades.AppKeepers{
					StakingKeeper: app.StakingKeeper,
					TfKeeper:      &app.TfKeeper,
				},
			),
		)
	}
}
