package app

import (
	"fmt"

	upgradetypes "cosmossdk.io/x/upgrade/types"

	"github.com/unionlabs/union/uniond/app/upgrades"
	"github.com/unionlabs/union/uniond/app/upgrades/v1_1_0"
	"github.com/unionlabs/union/uniond/app/upgrades/v1_2_0"
)

var Upgrades = []upgrades.Upgrade{v1_1_0.Upgrade, v1_2_0.Upgrade}

// configure store loader that checks if version == upgradeHeight and applies store upgrades
func (app *App) setupUpgradeStoreLoaders() {
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

func (app *App) setupUpgradeHandlers() {
	for _, upgrade := range Upgrades {
		app.UpgradeKeeper.SetUpgradeHandler(
			upgrade.UpgradeName,
			upgrade.CreateUpgradeHandler(
				app.ModuleManager,
				app.configurator,
				&upgrades.AppKeepers{
					AuthKeeper:         app.AccountKeeper,
					BankKeeper:         app.BankKeeper,
					ConsensusKeeper:    &app.ConsensusParamsKeeper,
					CrisisKeeper:       *app.CrisisKeeper,
					DistributionKeeper: app.DistrKeeper,
					FeeMarketKeeper:    app.FeeMarketKeeper,
					GovKeeper:          *app.GovKeeper,
					MintKeeper:         app.MintKeeper,
					StakingKeeper:      app.StakingKeeper,
				},
			),
		)
	}
}
