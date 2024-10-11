---
sidebar_position: 1
---

# Overview of `app_di.go`

:::note Synopsis

The Cosmos SDK allows much easier wiring of an `app.go` thanks to App Wiring and [`depinject`](../packages/01-depinject.md).
Learn more about the rationale of App Wiring in [ADR-057](../architecture/adr-057-app-wiring.md).

:::

:::note Pre-requisite Readings

* [ADR 057: App Wiring](../architecture/adr-057-app-wiring.md)
* [Depinject Documentation](../packages/01-depinject.md)
* [Modules depinject-ready](../building-modules/15-depinject.md)

:::

This section is intended to provide an overview of the `SimApp` `app_di.go` file with App Wiring.

## `app_config.go`

The `app_config.go` file is the single place to configure all modules parameters.

1. Create the `AppConfig` variable:

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_config.go#L103
    ```

2. Configure the `runtime` module:

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_config.go#L103-L167
    ```

3. Configure the modules defined in the `PreBlocker`, `BeginBlocker` and `EndBlocker` and the `tx` module:

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_config.go#L112-L129
    ```

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_config.go#L200-L203
    ```

### Complete `app_config.go`

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_config.go
```

### Alternative formats

:::tip
The example above shows how to create an `AppConfig` using Go. However, it is also possible to create an `AppConfig` using YAML, or JSON.  
The configuration can then be embed with `go:embed` and read with [`appconfig.LoadYAML`](https://pkg.go.dev/cosmossdk.io/core/appconfig#LoadYAML), or [`appconfig.LoadJSON`](https://pkg.go.dev/cosmossdk.io/core/appconfig#LoadJSON), in `app_di.go`.

```go
//go:embed app_config.yaml
var (
    appConfigYaml []byte
    appConfig = appconfig.LoadYAML(appConfigYaml)
)
```

:::

```yaml
modules:
  - name: runtime
    config:
      "@type": cosmos.app.runtime.v1alpha1.Module
      app_name: SimApp
      begin_blockers: [staking, auth, bank]
      end_blockers: [bank, auth, staking]
      init_genesis: [bank, auth, staking]
  - name: auth
    config:
      "@type": cosmos.auth.module.v1.Module
      bech32_prefix: cosmos
  - name: bank
    config:
      "@type": cosmos.bank.module.v1.Module
  - name: staking
    config:
      "@type": cosmos.staking.module.v1.Module
  - name: tx
    config:
      "@type": cosmos.tx.config.v1.Config
```

A more complete example of `app.yaml` can be found [here](https://github.com/cosmos/cosmos-sdk/blob/91b1d83f1339e235a1dfa929ecc00084101a19e3/simapp/app.yaml).

## `app_di.go`

`app_di.go` is the place where `SimApp` is constructed. `depinject.Inject` facilitates that by automatically wiring the app modules and keepers, provided an application configuration `AppConfig` is provided. `SimApp` is constructed, when calling the injected `*runtime.AppBuilder`, with `appBuilder.Build(...)`.    
In short `depinject` and the [`runtime` package](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/runtime) abstract the wiring of the app, and the `AppBuilder` is the place where the app is constructed. [`runtime`](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/runtime) takes care of registering the codecs, KV store, subspaces and instantiating `baseapp`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_v2.go#L101-L245
```

:::warning
When using `depinject.Inject`, the injected types must be pointers.
:::

### Advanced Configuration

In advanced cases, it is possible to inject extra (module) configuration in a way that is not (yet) supported by `AppConfig`.  
In this case, use `depinject.Configs` for combining the extra configuration and `AppConfig`, and `depinject.Supply` to providing that extra configuration.
More information on how work `depinject.Configs` and `depinject.Supply` can be found in the [`depinject` documentation](https://pkg.go.dev/cosmossdk.io/depinject).

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_v2.go#L114-L146
```

### Registering non app wiring modules

It is possible to combine app wiring / depinject enabled modules with non app wiring modules.
To do so, use the `app.RegisterModules` method to register the modules on your app, as well as `app.RegisterStores` for registering the extra stores needed.

```go
// ....
app.App = appBuilder.Build(db, traceStore, baseAppOptions...)

// register module manually
app.RegisterStores(storetypes.NewKVStoreKey(example.ModuleName))
app.ExampleKeeper = examplekeeper.NewKeeper(app.appCodec, app.AccountKeeper.AddressCodec(), runtime.NewKVStoreService(app.GetKey(example.ModuleName)), authtypes.NewModuleAddress(govtypes.ModuleName).String())
exampleAppModule := examplemodule.NewAppModule(app.ExampleKeeper)
if err := app.RegisterModules(&exampleAppModule); err != nil {
	panic(err)
}

// ....
```

:::warning
When using AutoCLI and combining app wiring and non app wiring modules. The AutoCLI options should be manually constructed instead of injected.
Otherwise it will miss the non depinject modules and not register their CLI.
:::

### Complete `app_di.go`

:::tip
Note that in the complete `SimApp` `app_di.go` file, testing utilities are also defined, but they could as well be defined in a separate file.
:::

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_v2.go
```
