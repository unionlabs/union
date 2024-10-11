# Upgrading Cosmos SDK

This guide provides instructions for upgrading to specific versions of Cosmos SDK.
Note, always read the **SimApp** section for more information on application wiring updates.

## [v0.52.x](https://github.com/cosmos/cosmos-sdk/releases/tag/v0.52.0-alpha.0)

Documentation to migrate an application from v0.50.x to server/v2 is available elsewhere.
It is additional to the changes described here.

### SimApp

In this section we describe the changes made in Cosmos SDK' SimApp.
**These changes are directly applicable to your application wiring.**
Please read this section first, but for an exhaustive list of changes, refer to the [CHANGELOG](./simapp/CHANGELOG.md).

#### Client (`root.go`)

The `client` package has been refactored to make use of the address codecs (address, validator address, consensus address, etc.)
and address bech32 prefixes (address and validator address).
This is part of the work of abstracting the SDK from the global bech32 config.

This means the address codecs and prefixes must be provided in the `client.Context` in the application client (usually `root.go`).

```diff
clientCtx = clientCtx.
+ WithAddressCodec(addressCodec).
+ WithValidatorAddressCodec(validatorAddressCodec).
+ WithConsensusAddressCodec(consensusAddressCodec).
+ WithAddressPrefix("cosmos").
+ WithValidatorPrefix("cosmosvaloper")
```

**When using `depinject` / `app_di`, the client codecs can be provided directly from application config.**

Refer to SimApp `root_v2.go` and `root.go` for an example with an app di and a legacy app.

Additionally, a simplification of the start command leads to the following change:

```diff
- server.AddCommands(rootCmd, newApp, func(startCmd *cobra.Command) {})
+ server.AddCommands(rootCmd, newApp, server.StartCmdOptions[servertypes.Application]{})
```

#### Server (`app.go`)

##### Module Manager

The basic module manager has been deleted. It was not necessary anymore and was simplified to use the `module.Manager` directly.
It can be removed from your `app.go`.

For depinject users, it isn't necessary anymore to supply a `map[string]module.AppModuleBasic` for customizing the app module basic instantiation.
The custom parameters (such as genutil message validator or gov proposal handler, or evidence handler) can directly be supplied.
When requiring a module manager in `root.go`, inject `*module.Manager` using `depinject.Inject`. 

For non depinject users, simply call `RegisterLegacyAminoCodec` and `RegisterInterfaces` on the module manager:

```diff
-app.BasicModuleManager = module.NewBasicManagerFromManager(...)
-app.BasicModuleManager.RegisterLegacyAminoCodec(legacyAmino)
-app.BasicModuleManager.RegisterInterfaces(interfaceRegistry)
+app.ModuleManager.RegisterLegacyAminoCodec(legacyAmino)
+app.ModuleManager.RegisterInterfaces(interfaceRegistry)
```

Additionally, thanks to the genesis simplification, as explained in [the genesis interface update](#genesis-interface), the module manager `InitGenesis` and `ExportGenesis` methods do not require the codec anymore.

##### gRPC Web

Grpc-web embedded client has been removed from the server. If you would like to use grpc-web, you can use the [envoy proxy](https://www.envoyproxy.io/docs/envoy/latest/start/start). Here's how to set it up:

<details>
<summary>Step by step guide</summary>

1. Install Envoy following the [official installation guide](https://www.envoyproxy.io/docs/envoy/latest/start/install).

2. Create an Envoy configuration file named `envoy.yaml` with the following content:

   ```yaml
	static_resources:
	listeners:
	- name: listener_0
		address:
		socket_address: { address: 0.0.0.0, port_value: 8080 }
		filter_chains:
		- filters:
		- name: envoy.filters.network.http_connection_manager
			typed_config:
			"@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
			codec_type: auto
			stat_prefix: ingress_http
			route_config:
				name: local_route
				virtual_hosts:
				- name: local_service
				domains: ["*"]
				routes:
				- match: { prefix: "/" }
					route:
					cluster: grpc_service
					timeout: 0s
					max_stream_duration:
						grpc_timeout_header_max: 0s
				cors:
					allow_origin_string_match:
					- prefix: "*"
					allow_methods: GET, PUT, DELETE, POST, OPTIONS
					allow_headers: keep-alive,user-agent,cache-control,content-type,content-transfer-encoding,custom-header-1,x-accept-content-transfer-encoding,x-accept-response-streaming,x-user-agent,x-grpc-web,grpc-timeout
					max_age: "1728000"
					expose_headers: custom-header-1,grpc-status,grpc-message
			http_filters:
			- name: envoy.filters.http.grpc_web
				typed_config:
				"@type": type.googleapis.com/envoy.extensions.filters.http.grpc_web.v3.GrpcWeb
			- name: envoy.filters.http.cors
				typed_config:
				"@type": type.googleapis.com/envoy.extensions.filters.http.cors.v3.Cors
			- name: envoy.filters.http.router
				typed_config:
				"@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router
	clusters:
	- name: grpc_service
		connect_timeout: 0.25s
		type: logical_dns
		http2_protocol_options: {}
		lb_policy: round_robin
		load_assignment:
		cluster_name: cluster_0
		endpoints:
			- lb_endpoints:
				- endpoint:
					address:
					socket_address:
						address: 0.0.0.0
						port_value: 9090
   ```

   This configuration tells Envoy to listen on port 8080 and forward requests to your gRPC service on port 9090. Note that this configuration is a starting point and can be modified according to your specific needs and preferences. You may need to adjust ports, addresses, or add additional settings based on your particular setup and requirements.

3. Start your Cosmos SDK application, ensuring it's configured to serve gRPC on port 9090.

4. Start Envoy with the configuration file:

   ```bash
   envoy -c envoy.yaml
   ```

5. If Envoy starts successfully, you should see output similar to this:

   ```bash
   [2024-08-29 10:47:08.753][6281320][info][config] [source/common/listener_manager/listener_manager_impl.cc:930] all dependencies initialized. starting workers
   [2024-08-29 10:47:08.754][6281320][info][main] [source/server/server.cc:978] starting main dispatch loop
   ```

   This indicates that Envoy has started and is ready to proxy requests.

6. Update your client applications to connect to Envoy (http://localhost:8080 by default).

</details>

By following these steps, Envoy will handle the translation between gRPC-Web and gRPC, allowing your existing gRPC-Web clients to continue functioning without modifications to your Cosmos SDK application.

To test the setup, you can use a tool like [grpcurl](https://github.com/fullstorydev/grpcurl). For example:

```bash
grpcurl -plaintext localhost:8080 cosmos.base.tendermint.v1beta1.Service/GetLatestBlock
```

##### AnteHandlers

The `GasConsumptionDecorator` and `IncreaseSequenceDecorator` have been merged with the SigVerificationDecorator, so you'll
need to remove them both from your app.go code, they will yield to unresolvable symbols when compiling.

#### Unordered Transactions

The Cosmos SDK now supports unordered transactions. This means that transactions
can be executed in any order and doesn't require the client to deal with or manage
nonces. This also means the order of execution is not guaranteed.

Unordered transactions are automatically enabled when using `depinject` / app di, simply supply the `servertypes.AppOptions` in `app.go`:

```diff
	depinject.Supply(
+		// supply the application options
+		appOpts,
		// supply the logger
		logger,
	)
```

<details>
<summary>Step-by-step Wiring </summary>
If you are still using the legacy wiring, you must enable unordered transactions manually:

* Update the `App` constructor to create, load, and save the unordered transaction
  manager.

	```go
	func NewApp(...) *App {
		// ...

		// create, start, and load the unordered tx manager
		utxDataDir := filepath.Join(cast.ToString(appOpts.Get(flags.FlagHome)), "data")
		app.UnorderedTxManager = unorderedtx.NewManager(utxDataDir)
		app.UnorderedTxManager.Start()

		if err := app.UnorderedTxManager.OnInit(); err != nil {
			panic(fmt.Errorf("failed to initialize unordered tx manager: %w", err))
		}
	}
	```

* Add the decorator to the existing AnteHandler chain, which should be as early
  as possible.

	```go
	anteDecorators := []sdk.AnteDecorator{
		ante.NewSetUpContextDecorator(),
		// ...
		ante.NewUnorderedTxDecorator(unorderedtx.DefaultMaxTimeoutDuration, options.TxManager, options.Environment),
		// ...
	}

	return sdk.ChainAnteDecorators(anteDecorators...), nil
	```

* If the App has a SnapshotManager defined, you must also register the extension
  for the TxManager.

	```go
	if manager := app.SnapshotManager(); manager != nil {
		err := manager.RegisterExtensions(unorderedtx.NewSnapshotter(app.UnorderedTxManager))
		if err != nil {
			panic(fmt.Errorf("failed to register snapshot extension: %s", err))
		}
	}
	```

* Create or update the App's `Preblocker()` method to call the unordered tx
  manager's `OnNewBlock()` method.

	```go
	...
	app.SetPreblocker(app.PreBlocker)
	...

	func (app *SimApp) PreBlocker(ctx sdk.Context, req *abci.RequestFinalizeBlock) (*sdk.ResponsePreBlock, error) {
		app.UnorderedTxManager.OnNewBlock(ctx.BlockTime())
		return app.ModuleManager.PreBlock(ctx, req)
	}
	```

* Create or update the App's `Close()` method to close the unordered tx manager.
  Note, this is critical as it ensures the manager's state is written to file
  such that when the node restarts, it can recover the state to provide replay
  protection.

	```go
	func (app *App) Close() error {
		// ...

		// close the unordered tx manager
		if e := app.UnorderedTxManager.Close(); e != nil {
			err = errors.Join(err, e)
		}

		return err
	}
	```

</details>

To submit an unordered transaction, the client must set the `unordered` flag to
`true` and ensure a reasonable `timeout_height` is set. The `timeout_height` is
used as a TTL for the transaction and is used to provide replay protection. See
[ADR-070](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-070-unordered-transactions.md)
for more details.

#### Sign Mode Textual

With the split of `x/auth/tx/config` in two (x/auth/tx/config as depinject module for txconfig and tx options) and `x/validate`, sign mode textual is no more automatically configured when using runtime (it was previously the case).
For the same instructions than for legacy app wiring to enable sign mode textual (see in v0.50 UPGRADING documentation).

### Depinject `app_config.go` / `app.yml`

With the introduction of [environment in modules](#core-api), depinject automatically creates the environment for all modules.
Learn more about environment [here](https://example.com) <!-- TODO -->. Given the fields of environment, this means runtime creates a kv store service for all modules by default. It can happen that some modules do not have a store necessary (such as `x/auth/tx` for instance). In this case, the store creation should be skipped in `app_config.go`:

```diff
InitGenesis: []string{
	"..."
},
+ // SkipStoreKeys is an optional list of store keys to skip when constructing the
+ // module's keeper. This is useful when a module does not have a store key.
+ SkipStoreKeys: []string{
+ 	"tx",
+ },
```

### Protobuf

The `cosmossdk.io/api/tendermint` package has been removed as CometBFT now publishes its protos to `buf.build/tendermint` and `buf.build/cometbft`.
There is no longer a need for the Cosmos SDK to host these protos for itself and its dependencies.
That package containing proto v2 generated code, but the SDK now uses [buf generated go SDK instead](https://buf.build/docs/bsr/generated-sdks/go).
If you were depending on `cosmossdk.io/api/tendermint`, please use the buf generated go SDK instead, or ask CometBFT host the generated proto v2 code.

The `codectypes.Any` has moved to `github.com/cosmos/gogoproto/types/any`. Module developers need to update the `buf.gen.gogo.yaml` configuration files by adjusting the corresponding `opt` option to `Mgoogle/protobuf/any.proto=github.com/cosmos/gogoproto/types/any` for directly mapping the`Any` type to its new location:

```diff
version: v1
plugins:
  - name: gocosmos
    out: ..
- 	 opt: plugins=grpc,Mgoogle/protobuf/any.proto=github.com/cosmos/cosmos-sdk/codec/types,Mcosmos/orm/v1/orm.proto=cosmossdk.io/orm
+    opt: plugins=grpc,Mgoogle/protobuf/any.proto=github.com/cosmos/gogoproto/types/any,Mcosmos/orm/v1/orm.proto=cosmossdk.io/orm
  - name: grpc-gateway
    out: ..
    opt: logtostderr=true,allow_colon_final_segments=true

```

Also, any usages of the interfaces `AnyUnpacker` and `UnpackInterfacesMessage` must be replaced with the interfaces of the same name in the `github.com/cosmos/gogoproto/types/any` package.

### Modules

#### `**all**`

All modules (expect `auth`) were spun out into their own `go.mod`. Replace their imports by `cosmossdk.io/x/{moduleName}`.

##### Core API

Core API has been introduced for modules since v0.47. With the deprecation of `sdk.Context`, we strongly recommend to use the `cosmossdk.io/core/appmodule` interfaces for the modules. This will allow the modules to work out of the box with server/v2 and baseapp, as well as limit their dependencies on the SDK.

Additionally, the `appmodule.Environment` struct is introduced to fetch different services from the application.
This should be used as an alternative to using `sdk.UnwrapContext(ctx)` to fetch the services.
It needs to be passed into a module at instantiation (or depinject will inject the correct environment). 

`x/circuit` is used as an example:

```go
app.CircuitKeeper = circuitkeeper.NewKeeper(runtime.NewEnvironment(runtime.NewKVStoreService(keys[circuittypes.StoreKey]), logger.With(log.ModuleKey, "x/circuit")), appCodec, authtypes.NewModuleAddress(govtypes.ModuleName).String(), app.AuthKeeper.AddressCodec())
```

If your module requires a message server or query server, it should be passed in the environment as well.

```diff
-govKeeper := govkeeper.NewKeeper(appCodec, runtime.NewKVStoreService(keys[govtypes.StoreKey]), app.AuthKeeper, app.BankKeeper,app.StakingKeeper, app.PoolKeeper, app.MsgServiceRouter(), govConfig, authtypes.NewModuleAddress(govtypes.ModuleName).String())
+govKeeper := govkeeper.NewKeeper(appCodec, runtime.NewEnvironment(runtime.NewKVStoreService(keys[govtypes.StoreKey]), logger.With(log.ModuleKey, "x/circuit"), runtime.EnvWithMsgRouterService(app.MsgServiceRouter()), runtime.EnvWithQueryRouterService(app.GRPCQueryRouter())), app.AuthKeeper, app.BankKeeper, app.StakingKeeper, app.PoolKeeper, govConfig, authtypes.NewModuleAddress(govtypes.ModuleName).String())
```

The signature of the extension interface `HasRegisterInterfaces` has been changed to accept a `cosmossdk.io/core/registry.InterfaceRegistrar` instead of a `codec.InterfaceRegistry`.   `HasRegisterInterfaces` is now a part of `cosmossdk.io/core/appmodule`.  Modules should update their `HasRegisterInterfaces` implementation to accept a `cosmossdk.io/core/registry.InterfaceRegistrar` interface.

```diff
-func (AppModule) RegisterInterfaces(registry codectypes.InterfaceRegistry) {
+func (AppModule) RegisterInterfaces(registry registry.InterfaceRegistrar) {
```

The signature of the extension interface `HasAminoCodec` has been changed to accept a `cosmossdk.io/core/registry.AminoRegistrar` instead of a `codec.LegacyAmino`. Modules should update their `HasAminoCodec` implementation to accept a `cosmossdk.io/core/registry.AminoRegistrar` interface.

```diff
-func (AppModule) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
+func (AppModule) RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
```

##### Simulation

`MsgSimulatorFn` has been updated to return an error. Its context argument has been removed, and an address.Codec has
been added to avoid the use of the Accounts.String() method.

```diff
-type MsgSimulatorFn func(r *rand.Rand, ctx sdk.Context, accs []Account) sdk.Msg
+type MsgSimulatorFn func(r *rand.Rand, accs []Account, cdc address.Codec) (sdk.Msg, error)
```

##### Depinject

Previously `cosmossdk.io/core` held functions `Invoke`, `Provide` and `Register` were moved to `cosmossdk.io/depinject/appconfig`.
All modules using dependency injection must update their imports.

##### Params

Previous module migrations have been removed. It is required to migrate to v0.50 prior to upgrading to v0.52 for not missing any module migrations.

##### Genesis Interface

All genesis interfaces have been migrated to take `context.Context` instead of `sdk.Context`.
Secondly, the codec is no longer passed in by the framework. The codec is now passed in by the module.
Lastly, all InitGenesis and ExportGenesis functions now return an error.

```go
// InitGenesis performs genesis initialization for the module.
func (am AppModule) InitGenesis(ctx context.Context, data json.RawMessage) error {
}

// ExportGenesis returns the exported genesis state as raw bytes for the module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
}
```

##### Migration to Collections

Most of Cosmos SDK modules have migrated to [collections](https://docs.cosmos.network/main/build/packages/collections).
Many functions have been removed due to this changes as the API can be smaller thanks to collections.
For modules that have migrated, verify you are checking against `collections.ErrNotFound` when applicable.

#### `x/auth`

Vesting accounts messages (and CLIs) have been removed. Existing vesting accounts will keep working but no new vesting accounts can be created.
Use `x/accounts` lockup accounts or implement an `x/accounts` vesting account instead.

#### `x/accounts`

Accounts's AccountNumber will be used as a global account number tracking replacing Auth legacy AccountNumber. Must set accounts's AccountNumber with auth's AccountNumber value in upgrade handler. This is done through auth keeper MigrateAccountNumber function.

```go
import authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper" 
...
err := authkeeper.MigrateAccountNumberUnsafe(ctx, &app.AuthKeeper)
if err != nil {
	return nil, err
}
```

#### `x/crisis`

The `x/crisis` module was removed due to it not being supported or functional any longer. 

#### `x/distribution`

Existing chains using `x/distribution` module must add the new `x/protocolpool` module.

#### `x/gov`

Gov v1beta1 proposal handler has been changed to take in a `context.Context` instead of `sdk.Context`.
This change was made to allow legacy proposals to be compatible with server/v2.
If you wish to migrate to server/v2, you should update your proposal handler to take in a `context.Context` and use services.
On the other hand, if you wish to keep using baseapp, simply unwrap the sdk context in your proposal handler.

#### `x/protocolpool`

Introducing a new `x/protocolpool` module to handle community pool funds. Its store must be added while upgrading to v0.52.x.

Example:

```go
func (app SimApp) RegisterUpgradeHandlers() {
  	app.UpgradeKeeper.SetUpgradeHandler(
 		UpgradeName,
 		func(ctx sdk.Context, _ upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
 			return app.ModuleManager.RunMigrations(ctx, app.Configurator(), fromVM)
 		},
 	)

  // ...
}
```

Add `x/protocolpool` store while upgrading to v0.52.x:

```go
storetypes.StoreUpgrades{
			Added: []string{
				protocolpooltypes.ModuleName,
			},
}
```

#### `x/validate`

Introducing `x/validate` a module that is solely used for registering default ante/post handlers and global tx validators when using runtime and runtime/v2. If you wish to set your custom ante/post handlers, no need to use this module.
You can however always extend them by adding extra tx validators (see `x/validate` documentation).
