# Upgrading Cosmos SDK

This guide provides instructions for upgrading to specific versions of Cosmos SDK.

## [v0.47.x](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.47.0)

### Migration to CometBFT (Part 1)

The Cosmos SDK has migrated to CometBFT, as its default consensus engine.
CometBFT is an implementation of the Tendermint consensus algorithm, and the successor of Tendermint Core.
Due to the import changes, this is a breaking change. Chains need to remove **entirely** their imports of Tendermint Core in their codebase, from direct and indirects imports in their `go.mod`.

* Replace `github.com/tendermint/tendermint` by `github.com/cometbft/cometbft`
* Replace `github.com/tendermint/tm-db` by `github.com/cometbft/cometbft-db`
* Verify `github.com/tendermint/tendermint` is not an indirect or direct dependency
* Run `make proto-gen`

Other than that, the migration should be seamless.
On the SDK side, clean-up of variables, functions to reflect the new name will only happen from v0.48 (part 2).

Note: It is possible that these steps must first be performed by your dependencies before you can perform them on your own codebase.

### Simulation

Remove `RandomizedParams` from `AppModuleSimulation` interface. Previously, it used to generate random parameter changes during simulations, however, it does so through ParamChangeProposal which is now legacy. Since all modules were migrated, we can now safely remove this from `AppModuleSimulation` interface.

Moreover, to support the `MsgUpdateParams` governance proposals for each modules, `AppModuleSimulation` now defines a `AppModule.ProposalMsgs` method in addition to `AppModule.ProposalContents`. That method defines the messages that can be used to submit a proposal and that should be tested in simulation.

When a module has no proposal messages or proposal content to be tested by simulation, the `AppModule.ProposalMsgs` and `AppModule.ProposalContents` methods can be deleted.

### gRPC

A new gRPC service, `proto/cosmos/base/node/v1beta1/query.proto`, has been introduced
which exposes various operator configuration. App developers should be sure to
register the service with the gRPC-gateway service via
`nodeservice.RegisterGRPCGatewayRoutes` in their application construction, which
is typically found in `RegisterAPIRoutes`.

### AppModule Interface

Support for the `AppModule` `Querier`, `Route` and `LegacyQuerier` methods has been entirely removed from the `AppModule`
interface. This removes and fully deprecates all legacy queriers. All modules no longer support the REST API previously
known as the LCD, and the `sdk.Msg#Route` method won't be used anymore.

Most other existing `AppModule` methods have been moved to extension interfaces in preparation for the migration
to the `cosmossdk.io/core/appmodule` API in the next release. Most `AppModule` implementations should not be broken
by this change.

### SimApp

The `simapp` package **should not be imported in your own app**. Instead, you should import the `runtime.AppI` interface, that defines an `App`, and use the [`simtestutil` package](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/testutil/sims) for application testing.

#### App Wiring

SimApp's `app_v2.go` is using [App Wiring](https://docs.cosmos.network/main/building-apps/app-go-v2), the dependency injection framework of the Cosmos SDK.
This means that modules are injected directly into SimApp thanks to a [configuration file](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app_config.go).
The previous behavior, without the dependency injection framework, is still present in [`app.go`](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app.go) and is not going anywhere.

If you are using a `app.go` without dependency injection, add the following lines to your `app.go` in order to provide newer gRPC services:

```go
autocliv1.RegisterQueryServer(app.GRPCQueryRouter(), runtimeservices.NewAutoCLIQueryService(app.ModuleManager.Modules))

reflectionSvc, err := runtimeservices.NewReflectionService()
if err != nil {
    panic(err)
}
reflectionv1.RegisterReflectionServiceServer(app.GRPCQueryRouter(), reflectionSvc)
```

#### Constructor

The constructor, `NewSimApp` has been simplified:

* `NewSimApp` does not take encoding parameters (`encodingConfig`) as input, instead the encoding parameters are injected (when using app wiring), or directly created in the constructor. Instead, we can instantiate `SimApp` for getting the encoding configuration.
* `NewSimApp` now uses `AppOptions` for getting the home path (`homePath`) and the invariant checks period (`invCheckPeriod`). These were unnecessary given as arguments as they were already present in the `AppOptions`.

#### Encoding

`simapp.MakeTestEncodingConfig()` was deprecated and has been removed. Instead you can use the `TestEncodingConfig` from the `types/module/testutil` package.
This means you can replace your usage of `simapp.MakeTestEncodingConfig` in tests to `moduletestutil.MakeTestEncodingConfig`, which takes a series of relevant `AppModuleBasic` as input (the module being tested and any potential dependencies).

#### Export

`ExportAppStateAndValidators` takes an extra argument, `modulesToExport`, which is a list of module names to export.
That argument should be passed to the module maanager `ExportGenesisFromModules` method.

#### Replaces

The `GoLevelDB` version must pinned to `v1.0.1-0.20210819022825-2ae1ddf74ef7` in the application, following versions might cause unexpected behavior.
This can be done adding `replace github.com/syndtr/goleveldb => github.com/syndtr/goleveldb v1.0.1-0.20210819022825-2ae1ddf74ef7` to the `go.mod` file.

* [issue #14949 on cosmos-sdk](https://github.com/cosmos/cosmos-sdk/issues/14949)
* [issue #25413 on go-ethereum](https://github.com/ethereum/go-ethereum/pull/25413)

### Protobuf

The SDK has migrated from `gogo/protobuf` (which is currently unmaintained), to our own maintained fork, [`cosmos/gogoproto`](https://github.com/cosmos/gogoproto).

This means you should replace all imports of `github.com/gogo/protobuf` to `github.com/cosmos/gogoproto`.
This allows you to remove the replace directive `replace github.com/gogo/protobuf => github.com/regen-network/protobuf v1.3.3-alpha.regen.1` from your `go.mod` file.

Please use the `ghcr.io/cosmos/proto-builder` image (version >= `0.11.5`) for generating protobuf files.

See which buf commit for `cosmos/cosmos-sdk` to pin in your `buf.yaml` file [here](./proto/README.md).

#### Gogoproto Import Paths

The SDK made a [patch fix](https://github.com/cosmos/gogoproto/pull/32) on its gogoproto repository to require that each proto file's package name matches its OS import path (relatively to a protobuf root import path, usually the root `proto/` folder, set by the `protoc -I` flag).

For example, assuming you put all your proto files in subfolders inside your root `proto/` folder, then a proto file with package name `myapp.mymodule.v1` should be found in the `proto/myapp/mymodule/v1/` folder. If it is in another folder, the proto generation command will throw an error.

If you are using a custom folder structure for your proto files, please reorganize them so that their OS path matches their proto package name.

This is to allow the proto FileDescriptSets to be correctly registered, and this standardized OS import paths allows [Hubl](https://github.com/cosmos/cosmos-sdk/tree/main/tools/hubl) to reflectively talk to any chain.

#### `{accepts,implements}_interface` proto annotations

The SDK is normalizing the strings inside the Protobuf `accepts_interface` and `implements_interface` annotations. We require them to be fully-scoped names. They will soon be used by code generators like Pulsar and Telescope to match which messages can or cannot be packed inside `Any`s.

Here are the following replacements that you need to perform on your proto files:

```diff
- "Content"
+ "cosmos.gov.v1beta1.Content"
- "Authorization"
+ "cosmos.authz.v1beta1.Authorization"
- "sdk.Msg"
+ "cosmos.base.v1beta1.Msg"
- "AccountI"
+ "cosmos.auth.v1beta1.AccountI"
- "ModuleAccountI"
+ "cosmos.auth.v1beta1.ModuleAccountI"
- "FeeAllowanceI"
+ "cosmos.feegrant.v1beta1.FeeAllowanceI"
```

Please also check that in your own app's proto files that there are no single-word names for those two proto annotations. If so, then replace them with fully-qualified names, even though those names don't actually resolve to an actual protobuf entity.

For more information, see the [encoding guide](./docs/docs/core/05-encoding.md).

### Transactions

#### Broadcast Mode

Broadcast mode `block` was deprecated and has been removed. Please use `sync` mode
instead. When upgrading your tests from `block` to `sync` and checking for a
transaction code, you need to query the transaction first (with its hash) to get
the correct code.

### Modules

#### `**all**`

`EventTypeMessage` events, with `sdk.AttributeKeyModule` and `sdk.AttributeKeySender` are now emitted directly at message excecution (in `baseapp`).
This means that the following boilerplate should be removed from all your custom modules:

```go
ctx.EventManager().EmitEvent(
	sdk.NewEvent(
		sdk.EventTypeMessage,
		sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		sdk.NewAttribute(sdk.AttributeKeySender, `signer/sender`),
	),
)
```

The module name is assumed by `baseapp` to be the second element of the message route: `"cosmos.bank.v1beta1.MsgSend" -> "bank"`.
In case a module does not follow the standard message path, (e.g. IBC), it is advised to keep emitting the module name event.
`Baseapp` only emits that event if the module has not already done so.

#### `x/params`

The `params` module was deprecated since v0.46. The Cosmos SDK has migrated away from `x/params` for its own modules.
Cosmos SDK modules now store their parameters directly in its repective modules.
The `params` module will be removed in `v0.48`, as mentioned [in v0.46 release](https://github.com/cosmos/cosmos-sdk/blob/v0.46.1/UPGRADING.md#xparams). It is strongly encouraged to migrate away from `x/params` before `v0.48`.

When performing a chain migration, the params table must be initizalied manually. This was done in the modules keepers in previous versions.
Have a look at `simapp.RegisterUpgradeHandlers()` for an example.

#### `x/crisis`

With the migrations of all modules away from `x/params`, the crisis module now has a store.
The store must be created during a chain upgrade to v0.47.x.

```go
storetypes.StoreUpgrades{
	Added: []string{
		crisistypes.ModuleName,
	},
}
```

#### `x/gov`

##### Minimum Proposal Deposit At Time of Submission

The `gov` module has been updated to support a minimum proposal deposit at submission time. It is determined by a new
parameter called `MinInitialDepositRatio`. When multiplied by the existing `MinDeposit` parameter, it produces
the necessary proportion of coins needed at the proposal submission time. The motivation for this change is to prevent proposal spamming.

By default, the new `MinInitialDepositRatio` parameter is set to zero during migration. The value of zero signifies that this 
feature is disabled. If chains wish to utilize the minimum proposal deposits at time of submission, the migration logic needs to be 
modified to set the new parameter to the desired value.

##### New `Proposal.Proposer` field

The `Proposal` proto has been updated with proposer field. For proposal state migraton developers can call `v4.AddProposerAddressToProposal` in their upgrade handler to update all existing proposal and make them compatible and **this migration is optional**.

```go
import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	v4 "github.com/cosmos/cosmos-sdk/x/gov/migrations/v4"
	upgradetypes "github.com/cosmos/cosmos-sdk/x/upgrade/types"
)

func (app SimApp) RegisterUpgradeHandlers() {
	app.UpgradeKeeper.SetUpgradeHandler(UpgradeName,
		func(ctx sdk.Context, plan upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
			// this migration is optional
			// add proposal ids with proposers which are active (deposit or voting period)
			proposals := make(map[uint64]string)
			proposals[1] = "cosmos1luyncewxk4lm24k6gqy8y5dxkj0klr4tu0lmnj" ...
			v4.AddProposerAddressToProposal(ctx, sdk.NewKVStoreKey(v4.ModuleName), app.appCodec, proposals)
			return app.ModuleManager.RunMigrations(ctx, app.Configurator(), fromVM)
		})
}

```

#### `x/consensus`

Introducing a new `x/consensus` module to handle managing Tendermint consensus
parameters. For migration it is required to call a specific migration to migrate
existing parameters from the deprecated `x/params` to `x/consensus` module. App
developers should ensure to call `baseapp.MigrateParams` in their upgrade handler.

Example:

```go
func (app SimApp) RegisterUpgradeHandlers() {
 	----> baseAppLegacySS := app.ParamsKeeper.Subspace(baseapp.Paramspace).WithKeyTable(paramstypes.ConsensusParamsKeyTable()) <----

 	app.UpgradeKeeper.SetUpgradeHandler(
 		UpgradeName,
 		func(ctx sdk.Context, _ upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
 			// Migrate Tendermint consensus parameters from x/params module to a
 			// dedicated x/consensus module.
 			----> baseapp.MigrateParams(ctx, baseAppLegacySS, &app.ConsensusParamsKeeper) <----

			// ...

 			return app.ModuleManager.RunMigrations(ctx, app.Configurator(), fromVM)
 		},
 	)

  // ...
}
```

The `x/params` module should still be imported in your app.go in order to handle this migration.

Because the `x/consensus` module is a new module, its store must be added while upgrading to v0.47.x:

```go
storetypes.StoreUpgrades{
	Added: []string{
		consensustypes.ModuleName,
	},
}
```

##### `app.go` changes

When using an `app.go` without App Wiring, the following changes are required:

```diff
- bApp.SetParamStore(app.ParamsKeeper.Subspace(baseapp.Paramspace).WithKeyTable(paramstypes.ConsensusParamsKeyTable()))
+ app.ConsensusParamsKeeper = consensusparamkeeper.NewKeeper(appCodec, keys[consensusparamstypes.StoreKey], authtypes.NewModuleAddress(govtypes.ModuleName).String())
+ bApp.SetParamStore(&app.ConsensusParamsKeeper)
```

When using App Wiring, the paramater store is automatically set for you.

#### `x/nft`

The SDK does not validate anymore the `classID` and `nftID` of an NFT, for extra flexibility in your NFT implementation.
This means chain developers need to validate the `classID` and `nftID` of an NFT.

### Ledger

Ledger support has been generalized to enable use of different apps and keytypes that use `secp256k1`. The Ledger interface remains the same, but it can now be provided through the Keyring `Options`, allowing higher-level chains to connect to different Ledger apps or use custom implementations. In addition, higher-level chains can provide custom key implementations around the Ledger public key, to enable greater flexibility with address generation and signing.

This is not a breaking change, as all values will default to use the standard Cosmos app implementation unless specified otherwise.
