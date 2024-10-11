package simapp

import (
	"context"

	storetypes "cosmossdk.io/store/types"
	circuittypes "cosmossdk.io/x/circuit/types"
	upgradetypes "cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/types/module"
)

const (
	IBCWasmUpgrade = "ibcwasm-v8"
)

// registerUpgradeHandlers registers all supported upgrade handlers
func (app *SimApp) registerUpgradeHandlers() {
	app.UpgradeKeeper.SetUpgradeHandler(
		IBCWasmUpgrade,
		createWasmStoreUpgradeHandler(app.ModuleManager, app.configurator),
	)

	upgradeInfo, err := app.UpgradeKeeper.ReadUpgradeInfoFromDisk()
	if err != nil {
		panic(err)
	}

	if upgradeInfo.Name == IBCWasmUpgrade && !app.UpgradeKeeper.IsSkipHeight(upgradeInfo.Height) {
		storeUpgrades := storetypes.StoreUpgrades{
			Added: []string{
				circuittypes.ModuleName,
			},
		}
		// configure store loader that checks if version == upgradeHeight and applies store upgrades
		app.SetStoreLoader(upgradetypes.UpgradeStoreLoader(upgradeInfo.Height, &storeUpgrades))
	}
}

// createWasmStoreUpgradeHandler creates an upgrade handler for the 08-wasm ibc-go/v8 SimApp upgrade.
func createWasmStoreUpgradeHandler(mm *module.Manager, configurator module.Configurator) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, _ upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		return mm.RunMigrations(ctx, configurator, vm)
	}
}
