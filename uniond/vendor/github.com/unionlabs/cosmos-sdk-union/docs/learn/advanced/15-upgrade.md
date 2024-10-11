---
sidebar_position: 1
---

# In-Place Store Migrations

:::warning
Read and understand all the in-place store migration documentation before you run a migration on a live chain.
:::

:::note Synopsis
Upgrade your app modules smoothly with custom in-place store migration logic.
:::

The Cosmos SDK uses two methods to perform upgrades:

* Exporting the entire application state to a JSON file using the `export` CLI command, making changes, and then starting a new binary with the changed JSON file as the genesis file.

* Perform upgrades in place, which significantly decrease the upgrade time for chains with a larger state. Use the [Module Upgrade Guide](../../build/building-modules/13-upgrade.md) to set up your application modules to take advantage of in-place upgrades.

This document provides steps to use the In-Place Store Migrations upgrade method.

## Tracking Module Versions

Each module gets assigned a consensus version by the module developer. The consensus version serves as the breaking change version of the module. The Cosmos SDK keeps track of all module consensus versions in the x/upgrade `VersionMap` store. During an upgrade, the difference between the old `VersionMap` stored in state and the new `VersionMap` is calculated by the Cosmos SDK. For each identified difference, the module-specific migrations are run and the respective consensus version of each upgraded module is incremented.

### Consensus Version

The consensus version is defined on each app module by the module developer and serves as the breaking change version of the module. The consensus version informs the Cosmos SDK on which modules need to be upgraded. For example, if the bank module was version 2 and an upgrade introduces bank module 3, the Cosmos SDK upgrades the bank module and runs the "version 2 to 3" migration script.

### Version Map

The version map is a mapping of module names to consensus versions. The map is persisted to x/upgrade's state for use during in-place migrations. When migrations finish, the updated version map is persisted in the state.

## Upgrade Handlers

Upgrades use an `UpgradeHandler` to facilitate migrations. The `UpgradeHandler` functions implemented by the app developer must conform to the following function signature. These functions retrieve the `VersionMap` from x/upgrade's state and return the new `VersionMap` to be stored in x/upgrade after the upgrade. The diff between the two `VersionMap`s determines which modules need upgrading.

```go
type UpgradeHandler func(ctx sdk.Context, plan Plan, fromVM VersionMap) (VersionMap, error)
```

Inside these functions, you must perform any upgrade logic to include in the provided `plan`. All upgrade handler functions must end with the following line of code:

```go
  return app.mm.RunMigrations(ctx, cfg, fromVM)
```

## Running Migrations

Migrations are run inside of an `UpgradeHandler` using `app.mm.RunMigrations(ctx, cfg, vm)`. The `UpgradeHandler` functions describe the functionality to occur during an upgrade. The `RunMigration` function loops through the `VersionMap` argument and runs the migration scripts for all versions that are less than the versions of the new binary app module. After the migrations are finished, a new `VersionMap` is returned to persist the upgraded module versions to state.

```go
cfg := module.NewConfigurator(...)
app.UpgradeKeeper.SetUpgradeHandler("my-plan", func(ctx sdk.Context, plan upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {

    // ...
    // additional upgrade logic
    // ...

    // returns a VersionMap with the updated module ConsensusVersions
    return app.mm.RunMigrations(ctx, fromVM)
})
```

To learn more about configuring migration scripts for your modules, see the [Module Upgrade Guide](../../build/building-modules/13-upgrade.md).

### Order Of Migrations

By default, all migrations are run in module name alphabetical ascending order, except `x/auth` which is run last. The reason is state dependencies between x/auth and other modules (you can read more in [issue #10606](https://github.com/cosmos/cosmos-sdk/issues/10606)).

If you want to change the order of migration, then you should call `app.mm.SetOrderMigrations(module1, module2, ...)` in your app.go file. The function will panic if you forget to include a module in the argument list.

## Adding New Modules During Upgrades

You can introduce entirely new modules to the application during an upgrade. New modules are recognized because they have not yet been registered in `x/upgrade`'s `VersionMap` store. In this case, `RunMigrations` calls the `InitGenesis` function from the corresponding module to set up its initial state.

### Add StoreUpgrades for New Modules

All chains preparing to run in-place store migrations will need to manually add store upgrades for new modules and then configure the store loader to apply those upgrades. This ensures that the new module's stores are added to the multistore before the migrations begin.

```go
upgradeInfo, err := app.UpgradeKeeper.ReadUpgradeInfoFromDisk()
if err != nil {
	panic(err)
}

if upgradeInfo.Name == "my-plan" && !app.UpgradeKeeper.IsSkipHeight(upgradeInfo.Height) {
	storeUpgrades := storetypes.StoreUpgrades{
		// add store upgrades for new modules
		// Example:
		//    Added: []string{"foo", "bar"},
		// ...
	}

	// configure store loader that checks if version == upgradeHeight and applies store upgrades
	app.SetStoreLoader(upgradetypes.UpgradeStoreLoader(upgradeInfo.Height, &storeUpgrades))
}
```

## Genesis State

When starting a new chain, the consensus version of each module MUST be saved to state during the application's genesis. To save the consensus version, add the following line to the `InitChainer` method in `app.go`:

```diff
func (app *MyApp) InitChainer(ctx sdk.Context, req abci.InitChainRequest) abci.InitChainResponse {
  ...
+ app.UpgradeKeeper.SetModuleVersionMap(ctx, app.mm.GetVersionMap())
  ...
}
```

This information is used by the Cosmos SDK to detect when modules with newer versions are introduced to the app.

For a new module `foo`, `InitGenesis` is called by `RunMigration` only when `foo` is registered in the module manager but it's not set in the `fromVM`. Therefore, if you want to skip `InitGenesis` when a new module is added to the app, then you should set its module version in `fromVM` to the module consensus version:

```go
app.UpgradeKeeper.SetUpgradeHandler("my-plan", func(ctx sdk.Context, plan upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
    // ...

    // Set foo's version to the latest ConsensusVersion in the VersionMap.
    // This will skip running InitGenesis on Foo
    fromVM[foo.ModuleName] = foo.AppModule{}.ConsensusVersion()

    return app.mm.RunMigrations(ctx, fromVM)
})
```

### Overwriting Genesis Functions

The Cosmos SDK offers modules that the application developer can import in their app. These modules often have an `InitGenesis` function already defined.

You can write your own `InitGenesis` function for an imported module. To do this, manually trigger your custom genesis function in the upgrade handler.

:::warning
You MUST manually set the consensus version in the version map passed to the `UpgradeHandler` function. Without this, the SDK will run the Module's existing `InitGenesis` code even if you triggered your custom function in the `UpgradeHandler`.
:::

```go
import foo "github.com/my/module/foo"

app.UpgradeKeeper.SetUpgradeHandler("my-plan", func(ctx sdk.Context, plan upgradetypes.Plan, fromVM module.VersionMap)  (module.VersionMap, error) {

    // Register the consensus version in the version map
    // to avoid the SDK from triggering the default
    // InitGenesis function.
    fromVM["foo"] = foo.AppModule{}.ConsensusVersion()

    // Run custom InitGenesis for foo
    app.mm["foo"].InitGenesis(ctx, app.appCodec, myCustomGenesisState)

    return app.mm.RunMigrations(ctx, cfg, fromVM)
})
```

## Syncing a Full Node to an Upgraded Blockchain

You can sync a full node to an existing blockchain which has been upgraded using Cosmovisor

To successfully sync, you must start with the initial binary that the blockchain started with at genesis. If all Software Upgrade Plans contain binary instruction, then you can run Cosmovisor with auto-download option to automatically handle downloading and switching to the binaries associated with each sequential upgrade. Otherwise, you need to manually provide all binaries to Cosmovisor.

To learn more about Cosmovisor, see the [Cosmovisor Quick Start](../../build/tooling/01-cosmovisor.md).
