---
sidebar_position: 1
---

# Modules depinject-ready

:::note Pre-requisite Readings

* [Depinject Documentation](../packages/01-depinject.md)

:::

[`depinject`](../packages/01-depinject.md) is used to wire any module in `app.go`.
All core modules are already configured to support dependency injection.

To work with `depinject` a module must define its configuration and requirements so that `depinject` can provide the right dependencies.

In brief, as a module developer, the following steps are required:

1. Define the module configuration using Protobuf
2. Define the module dependencies in `x/{moduleName}/module.go`

A chain developer can then use the module by following these two steps:

1. Configure the module in `app_config.go` or `app.yaml`
2. Inject the module in `app.go`

## Module Configuration

The module available configuration is defined in a Protobuf file, located at `{moduleName}/module/v1/module.proto`.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/proto/cosmos/group/module/v1/module.proto
```

* `go_import` must point to the Go package of the custom module.
* Message fields define the module configuration.
  That configuration can be set in the `app_config.go` / `app.yaml` file for a chain developer to configure the module.  
  Taking `group` as example, a chain developer is able to decide, thanks to `uint64 max_metadata_len`, what the maximum metadata length allowed for a group proposal is.

  ```go reference
  https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/app_config.go#L228-L234
  ```

That message is generated using [`pulsar`](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/scripts/protocgen-pulsar.sh) (by running `make proto-gen`).
In the case of the `group` module, this file is generated here: https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/api/cosmos/group/module/v1/module.pulsar.go.

The part that is relevant for the module configuration is:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/api/cosmos/group/module/v1/module.pulsar.go#L515-L527
```

:::note
Pulsar is optional. The official [`protoc-gen-go`](https://protobuf.dev/reference/go/go-generated/) can be used as well.
:::

## Dependency Definition

Once the configuration proto is defined, the module's `module.go` must define what dependencies are required by the module.
The boilerplate is similar for all modules.

:::warning
All methods, structs and their fields must be public for `depinject`.
:::

1. Import the module configuration generated package:

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/group/module/module.go#L12-L14
    ```

    Define an `init()` function for defining the `providers` of the module configuration:  
    This registers the module configuration message and the wiring of the module.

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/group/module/module.go#L194-L199
    ```

2. Ensure that the module implements the `appmodule.AppModule` interface:

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.47.0/x/group/module/module.go#L58-L64
    ```

3. Define a struct that inherits `depinject.In` and define the module inputs (i.e. module dependencies):
   * `depinject` provides the right dependencies to the module.
   * `depinject` also checks that all dependencies are provided.

    :::tip
    For making a dependency optional, add the `optional:"true"` struct tag.  
    :::

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/group/module/module.go#L201-L211
    ```

4. Define the module outputs with a public struct that inherits `depinject.Out`:
   The module outputs are the dependencies that the module provides to other modules. It is usually the module itself and its keeper.

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/group/module/module.go#L213-L218
    ```

5. Create a function named `ProvideModule` (as called in 1.) and use the inputs for instantiating the module outputs.

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/group/module/module.go#L220-L235
    ```

    The `ProvideModule` function should return an instance of `cosmossdk.io/core/appmodule.AppModule` which implements
    one or more app module extension interfaces for initializing the module.

    Following is the complete app wiring configuration for `group`:

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/group/module/module.go#L194-L235
    ```

6. All modules must implement `depinject.OnePerModuleType` interface. This is used in order to tell the dependency injection framework that the module can only be instantiated once. 

    ```go reference
    https://github.com/cosmos/cosmos-sdk/blob/f4bdec3433373cc4950f4680743e969495763fbb/x/group/module/module.go#L64-L65
    ```

The module is now ready to be used with `depinject` by a chain developer.

## Integrate in an application

The App Wiring is done in `app_config.go` / `app.yaml` and `app_di.go` and is explained in detail in the [overview of `app_di.go`](https://docs.cosmos.network/main/build/building-apps/app-go-di).
