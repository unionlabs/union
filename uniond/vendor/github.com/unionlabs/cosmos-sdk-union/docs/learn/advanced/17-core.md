---
sidebar_position: 1
---

# Core

Core is package which specifies the interfaces for core components of the Cosmos SDK.  Other
packages in the SDK implement these interfaces to provide the core functionality.  This design
provides modularity and flexibility to the SDK, allowing developers to swap out implementations
of core components as needed.  As such it is often referred to as the Core API.

## Environment

The `Environment` struct is a core component of the Cosmos SDK.  It provides access to the core
services of the SDK, such as the KVStore, EventManager, and Logger.  The `Environment` struct is
passed to modules and other components of the SDK to provide access to these services.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/core/v1.0.0-alpha.4/core/appmodule/v2/environment.go#L16-L29
```

Historically the SDK has used an [sdk.Context](02-context.md) to pass around services and data.
`Environment` is a newer construct that is intended to replace an `sdk.Context` in many cases.
`sdk.Context` will be deprecated in the future on the same timeline as [Baseapp](00-baseapp.md).

## Branch Service

The [BranchService](https://pkg.go.dev/cosmossdk.io/core/branch#Service.Execute) provides an
interface to execute arbitrary code in a branched store.  This is useful for executing code
that needs to make changes to the store, but may need to be rolled back if an error occurs.
Below is a contrived example based on the `x/epoch` module's BeginBlocker logic.

```go
func (k Keeper) BeginBlocker(ctx context.Context) error {
	err := k.EpochInfo.Walk(
        // ...
		ctx,
		nil,
		func(key string, epochInfo types.EpochInfo) (stop bool, err error) {
            // ...  
				if err := k.BranchService.Execute(ctx, func(ctx context.Context) error {
					return k.AfterEpochEnd(ctx, epochInfo.Identifier, epochInfo.CurrentEpoch)
                }); err != nil {
                    return true, err
                }
        })
}
```

Note that calls to `BranchService.Execute` are atomic and cannot share state with each other
except when the transaction is successful. If successful, the changes made to the store will be
committed.  If an error occurs, the changes will be rolled back.

## Event Service

The Event Service returns a handle to an [EventManager](https://pkg.go.dev/cosmossdk.io/core@v1.0.0-alpha.4/event#Manager) 
which can be used to emit events.  For information on how to emit events and their meaning
in the SDK see the [Events](08-events.md) document.

Note that core's `EventManager` API is a subset of the EventManager API described above; the
latter will be deprecated and removed in the future.  Roughly speaking legacy `EmitTypeEvent`
maps to `Emit` and legacy `EmitEvent` maps to `EmitKV`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/core/v1.0.0-alpha.4/core/event/service.go#L18-L29
```

## Gas Service

The gas service encapsulates both gas configuration and a gas meter.  Gas consumption is largely
handled at the framework level for transaction processing and state access but modules can
choose to use the gas service directly if needed.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/core/v1.0.0-alpha.4/core/gas/service.go#L26-L54
```

## Header Service

The header service provides access to the current block header.  This is useful for modules that
need to access the block header fields like `Time` and `Height` during transaction processing.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/a3729c1ad6ba2fb46f879ec7ea67c3afc02e9859/core/header/service.go#L11-L23
```

### Custom Header Service

Core's service oriented architecture (SOA) allows for chain developers to define a custom
implementation of the `HeaderService` interface.  This would involve creating a new struct that
satisfies `HeaderService` but composes additional logic on top.  An example of where this would
happen (when using depinject is shown below).  Note this example is taken from `runtime/v2` but
could easily be adapted to `runtime/v1` (the default runtime 0.52).  This same pattern can be
replicated for any core service.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/489aaae40234f1015a7bbcfa9384a89dc8de8153/runtime/v2/module.go#L262-L288
```

These bindings are applied to the `depinject` container in simapp/v2 as shown below.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/489aaae40234f1015a7bbcfa9384a89dc8de8153/simapp/v2/app_di.go#L72-L74
```

## Query and Message Router Service

Both the query and message router services are implementation of the same interface, `router.Service`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/core/v1.0.0-alpha.4/core/router/service.go#L11-L16
```

Both are exposed to modules so that arbitrary messages and queries can be routed to the
appropriate handler.  This powerful abstraction allows module developers to fully decouple
modules from each other by using only the proto message for dispatching.   This is particularly
useful for modules like `x/accounts` which require a dynamic dispatch mechanism in order to
function.

## TransactionService

```go reference
https://github.com/cosmos/cosmos-sdk/blob/core/v1.0.0-alpha.4/core/transaction/service.go#L21-L25
```

The transaction service provides access to the execution mode a state machine transaction is
running in, which may be one of `Check`, `Recheck`, `Simulate` or `Finalize`.  The SDK primarily
uses these flags in ante handlers to skip certain checks while in `Check` or `Simulate` modes,
but module developers may find uses for them as well.

## KVStore Service

```go reference
https://github.com/cosmos/cosmos-sdk/blob/core/v1.0.0-alpha.4/core/store/service.go#L5-L11
```

The KVStore service abstracts access to, and creation of, key-value stores.  Most use cases will
be backed by a merkle-tree store, but developers can provide their own implementations if
needed.  In the case of the `KVStoreService` implementation provided in `Environment`, module
developers should understand that calling `OpenKVStore` will return a store already scoped to
the module's prefix.  The wiring for this scoping is specified in `runtime`.
