# ADR 041: In-Place Store Migrations

## Changelog

* 17.02.2021: Initial Draft

## Status

Accepted

## Abstract

This ADR introduces a mechanism to perform in-place state store migrations during chain software upgrades.

## Context

When a chain upgrade introduces state-breaking changes inside modules, the current procedure consists of exporting the whole state into a JSON file (via the `simd export` command), running migration scripts on the JSON file (`simd genesis migrate` command), clearing the stores (`simd unsafe-reset-all` command), and starting a new chain with the migrated JSON file as new genesis (optionally with a custom initial block height). An example of such a procedure can be seen [in the Cosmos Hub 3->4 migration guide](https://github.com/cosmos/gaia/blob/v4.0.3/docs/migration/cosmoshub-3.md#upgrade-procedure).

This procedure is cumbersome for multiple reasons:

* The procedure takes time. It can take hours to run the `export` command, plus some additional hours to run `InitChain` on the fresh chain using the migrated JSON.
* The exported JSON file can be heavy (~100MB-1GB), making it difficult to view, edit and transfer, which in turn introduces additional work to solve these problems (such as [streaming genesis](https://github.com/cosmos/cosmos-sdk/issues/6936)).

## Decision

We propose a migration procedure based on modifying the KV store in-place without involving the JSON export-process-import flow described above.

### Module `ConsensusVersion`

We introduce a new method on the `AppModule` interface:

```go
type AppModule interface {
    // --snip--
    ConsensusVersion() uint64
}
```

This methods returns an `uint64` which serves as state-breaking version of the module. It MUST be incremented on each consensus-breaking change introduced by the module. To avoid potential errors with default values, the initial version of a module MUST be set to 1. In the Cosmos SDK, version 1 corresponds to the modules in the v0.41 series.

### Module-Specific Migration Functions

For each consensus-breaking change introduced by the module, a migration script from ConsensusVersion `N` to version `N+1` MUST be registered in the `Configurator` using its newly-added `RegisterMigration` method. All modules receive a reference to the configurator in their `RegisterServices` method on `AppModule`, and this is where the migration functions should be registered. The migration functions should be registered in increasing order.

```go
func (am AppModule) RegisterServices(cfg module.Configurator) {
    // --snip--
    cfg.RegisterMigration(types.ModuleName, 1, func(ctx sdk.Context) error {
        // Perform in-place store migrations from ConsensusVersion 1 to 2.
    })
     cfg.RegisterMigration(types.ModuleName, 2, func(ctx sdk.Context) error {
        // Perform in-place store migrations from ConsensusVersion 2 to 3.
    })
    // etc.
}
```

For example, if the new ConsensusVersion of a module is `N` , then `N-1` migration functions MUST be registered in the configurator.

In the Cosmos SDK, the migration functions are handled by each module's keeper, because the keeper holds the `sdk.StoreKey` used to perform in-place store migrations. To not overload the keeper, a `Migrator` wrapper is used by each module to handle the migration functions:

```go
// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
  BaseKeeper
}
```

Migration functions should live inside the `migrations/` folder of each module, and be called by the Migrator's methods. We propose the format `Migrate{M}to{N}` for method names.

```go
// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx sdk.Context) error {
	return v2bank.MigrateStore(ctx, m.keeper.storeKey) // v043bank is package `x/bank/migrations/v2`.
}
```

Each module's migration functions are specific to the module's store evolutions, and are not described in this ADR. An example of x/bank store key migrations after the introduction of ADR-028 length-prefixed addresses can be seen in this [store.go code](https://github.com/cosmos/cosmos-sdk/blob/36f68eb9e041e20a5bb47e216ac5eb8b91f95471/x/bank/legacy/v043/store.go#L41-L62).

### Tracking Module Versions in `x/upgrade`

We introduce a new prefix store in `x/upgrade`'s store. This store will track each module's current version, it can be modelized as a `map[string]uint64` of module name to module ConsensusVersion, and will be used when running the migrations (see next section for details). The key prefix used is `0x1`, and the key/value format is:

```text
0x2 | {bytes(module_name)} => BigEndian(module_consensus_version)
```

The initial state of the store is set from `app.go`'s `InitChainer` method.

The UpgradeHandler signature needs to be updated to take a `VersionMap`, as well as return an upgraded `VersionMap` and an error:

```diff
- type UpgradeHandler func(ctx sdk.Context, plan Plan)
+ type UpgradeHandler func(ctx sdk.Context, plan Plan, versionMap VersionMap) (VersionMap, error)
```

To apply an upgrade, we query the `VersionMap` from the `x/upgrade` store and pass it into the handler. The handler runs the actual migration functions (see next section), and if successful, returns an updated `VersionMap` to be stored in state.

```diff
func (k UpgradeKeeper) ApplyUpgrade(ctx sdk.Context, plan types.Plan) {
    // --snip--
-   handler(ctx, plan)
+   updatedVM, err := handler(ctx, plan, k.GetModuleVersionMap(ctx)) // k.GetModuleVersionMap() fetches the VersionMap stored in state.
+   if err != nil {
+       return err
+   }
+
+   // Set the updated consensus versions to state
+   k.SetModuleVersionMap(ctx, updatedVM)
}
```

A gRPC query endpoint to query the `VersionMap` stored in `x/upgrade`'s state will also be added, so that app developers can double-check the `VersionMap` before the upgrade handler runs.

### Running Migrations

Once all the migration handlers are registered inside the configurator (which happens at startup), running migrations can happen by calling the `RunMigrations` method on `module.Manager`. This function will loop through all modules, and for each module:

* Get the old ConsensusVersion of the module from its `VersionMap` argument (let's call it `M`).
* Fetch the new ConsensusVersion of the module from the `ConsensusVersion()` method on `AppModule` (call it `N`).
* If `N>M`, run all registered migrations for the module sequentially `M -> M+1 -> M+2...` until `N`.
    * There is a special case where there is no ConsensusVersion for the module, as this means that the module has been newly added during the upgrade. In this case, no migration function is run, and the module's current ConsensusVersion is saved to `x/upgrade`'s store.

If a required migration is missing (e.g. if it has not been registered in the `Configurator`), then the `RunMigrations` function will error.

In practice, the `RunMigrations` method should be called from inside an `UpgradeHandler`.

```go
app.UpgradeKeeper.SetUpgradeHandler("my-plan", func(ctx sdk.Context, plan upgradetypes.Plan, vm module.VersionMap)  (module.VersionMap, error) {
    return app.mm.RunMigrations(ctx, vm)
})
```

Assuming a chain upgrades at block `n`, the procedure should run as follows:

* the old binary will halt in `BeginBlock` when starting block `N`. In its store, the ConsensusVersions of the old binary's modules are stored.
* the new binary will start at block `N`. The UpgradeHandler is set in the new binary, so will run at `BeginBlock` of the new binary. Inside `x/upgrade`'s `ApplyUpgrade`, the `VersionMap` will be retrieved from the (old binary's) store, and passed into the `RunMigrations` functon, migrating all module stores in-place before the modules' own `BeginBlock`s.

## Consequences

### Backwards Compatibility

This ADR introduces a new method `ConsensusVersion()` on `AppModule`, which all modules need to implement. It also alters the UpgradeHandler function signature. As such, it is not backwards-compatible.

While modules MUST register their migration functions when bumping ConsensusVersions, running those scripts using an upgrade handler is optional. An application may perfectly well decide to not call the `RunMigrations` inside its upgrade handler, and continue using the legacy JSON migration path.

### Positive

* Perform chain upgrades without manipulating JSON files.
* While no benchmark has been made yet, it is probable that in-place store migrations will take less time than JSON migrations. The main reason supporting this claim is that both the `simd export` command on the old binary and the `InitChain` function on the new binary will be skipped.

### Negative

* Module developers MUST correctly track consensus-breaking changes in their modules. If a consensus-breaking change is introduced in a module without its corresponding `ConsensusVersion()` bump, then the `RunMigrations` function won't detect the migration, and the chain upgrade might be unsuccessful. Documentation should clearly reflect this.

### Neutral

* The Cosmos SDK will continue to support JSON migrations via the existing `simd export` and `simd genesis migrate` commands.
* The current ADR does not allow creating, renaming or deleting stores, only modifying existing store keys and values. The Cosmos SDK already has the `StoreLoader` for those operations.

## Further Discussions

## References

* Initial discussion: https://github.com/cosmos/cosmos-sdk/discussions/8429
* Implementation of `ConsensusVersion` and `RunMigrations`: https://github.com/cosmos/cosmos-sdk/pull/8485
* Issue discussing `x/upgrade` design: https://github.com/cosmos/cosmos-sdk/issues/8514
