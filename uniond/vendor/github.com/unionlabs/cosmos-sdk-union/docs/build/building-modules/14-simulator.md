---
sidebar_position: 1
---

# Module Simulation

:::note Pre-requisite Readings

* [Cosmos Blockchain Simulator](../../learn/advanced/12-simulation.md)
:::

## Synopsis

This document details how to define each module simulation functions to be
integrated with the application `SimulationManager`.
  
* [Simulation package](#simulation-package)
    * [Store decoders](#store-decoders)
    * [Randomized genesis](#randomized-genesis)
    * [Random weighted operations](#random-weighted-operations)
    * [Random proposal contents](#random-proposal-contents)
* [Registering simulation functions](#registering-simulation-functions)
* [App Simulator manager](#app-simulator-manager)

## Simulation package

Every module that implements the Cosmos SDK simulator needs to have a `x/<module>/simulation`
package which contains the primary functions required by the fuzz tests: store
decoders, randomized genesis state and parameters, weighted operations and proposal
contents.

### Store decoders

Registering the store decoders is required for the `AppImportExport`. This allows
for the key-value pairs from the stores to be decoded (_i.e_ unmarshalled)
to their corresponding types. In particular, it matches the key to a concrete type
and then unmarshals the value from the `KVPair` to the type provided.

You can use the example [here](https://github.com/cosmos/cosmos-sdk/blob/main/x/distribution/simulation/decoder.go) from the distribution module to implement your store decoders.

If the module uses the `collections` package, you can use the example [here](https://github.com/cosmos/cosmos-sdk/blob/23cf89cce1882ba9c8280e64735ae200504bfdce/x/bank/module.go#L166) from the Bank module to implement your store decoders.

### Randomized genesis

The simulator tests different scenarios and values for genesis parameters
in order to fully test the edge cases of specific modules. The `simulator` package from each module must expose a `RandomizedGenState` function to generate the initial random `GenesisState` from a given seed.

Once the module genesis parameter are generated randomly (or with the key and
values defined in a `params` file), they are marshaled to JSON format and added
to the app genesis JSON to use it on the simulations.

You can check an example on how to create the randomized genesis [here](https://github.com/cosmos/cosmos-sdk/blob/main/x/staking/simulation/genesis.go).

### Random weighted operations

Operations are one of the crucial parts of the Cosmos SDK simulation. They are the transactions
(`Msg`) that are simulated with random field values. The sender of the operation
is also assigned randomly.

Operations on the simulation are simulated using the full [transaction cycle](../../learn/advanced/01-transactions.md) of a
`ABCI` application that exposes the `BaseApp`.

Shown below is how weights are set:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/23cf89cce1882ba9c8280e64735ae200504bfdce/x/staking/depinject.go#L144-L154
```

As you can see, the weights are predefined in this case. Options exist to override this behavior with different weights. One option is to use `*rand.Rand` to define a random weight for the operation, or you can inject your own predefined weights.

The SDK simulations can be executed like normal tests in Go from the shell or within an IDE.
Make sure that you pass the `-tags='sims` parameter to enable them and other params that make sense for your scenario.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/23cf89cce1882ba9c8280e64735ae200504bfdce/scripts/build/simulations.mk#L19
```

### Random proposal contents

Randomized governance proposals are also supported on the Cosmos SDK simulator. Each
module must register the message to be used for governance proposals.  

```go reference
https://github.com/cosmos/cosmos-sdk/blob/23cf89cce1882ba9c8280e64735ae200504bfdce/x/staking/depinject.go#L139-L142
```

## Registering simulation functions

Now that all the required functions are defined, we need to integrate them into the module pattern within the `module.go`:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/23cf89cce1882ba9c8280e64735ae200504bfdce/x/staking/depinject.go#L127-L154
```

## App Simulator manager

The following step is setting up the `SimulatorManager` at the app level. This
is required for the simulation test files on the next step.

```go
type CustomApp struct {
  ...
  sm *module.SimulationManager
}
```

Then at the instantiation of the application, we create the `SimulationManager`
instance in the same way we create the `ModuleManager` but this time we only pass
the modules that implement the simulation functions from the `AppModuleSimulation`
interface described above.

```go
func NewCustomApp(...) {
  // create the simulation manager and define the order of the modules for deterministic simulations
  app.sm = module.NewSimulationManager(
    auth.NewAppModule(app.accountKeeper),
    bank.NewAppModule(app.bankKeeper, app.accountKeeper),
    supply.NewAppModule(app.supplyKeeper, app.accountKeeper),
    gov.NewAppModule(app.govKeeper, app.accountKeeper, app.supplyKeeper),
    mint.NewAppModule(app.mintKeeper),
    distr.NewAppModule(app.distrKeeper, app.accountKeeper, app.supplyKeeper, app.stakingKeeper),
    staking.NewAppModule(cdc, app.stakingKeeper),
    slashing.NewAppModule(app.slashingKeeper, app.accountKeeper, app.stakingKeeper),
  )

  // register the store decoders for simulation tests
  app.sm.RegisterStoreDecoders()
  ...
}
```

## Integration with the Go fuzzer framework

The simulations provide deterministic behaviour already. The integration with the [Go fuzzer](https://go.dev/doc/security/fuzz/)
can be done at a high level with the deterministic pseudo random number generator where the fuzzer provides varying numbers. 

```go reference
https://github.com/cosmos/cosmos-sdk/blob/23cf89cce1882ba9c8280e64735ae200504bfdce/scripts/build/simulations.mk#L80-L84
```
