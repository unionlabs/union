---
sidebar_position: 1
---

# Capability Module

## Overview

`modules/capability` is an implementation of a Cosmos SDK module, per [ADR 003](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-003-dynamic-capability-store.md),
that allows for provisioning, tracking, and authenticating multi-owner capabilities
at runtime.

The keeper maintains two states: persistent and ephemeral in-memory. The persistent
store maintains a globally unique auto-incrementing index and a mapping from
capability index to a set of capability owners that are defined as a module and
capability name tuple. The in-memory ephemeral state keeps track of the actual
capabilities, represented as addresses in local memory, with both forward and reverse indexes.
The forward index maps module name and capability tuples to the capability name. The
reverse index maps between the module and capability name and the capability itself.

The keeper allows the creation of "scoped" sub-keepers which are tied to a particular
module by name. Scoped keepers must be created at application initialization and
passed to modules, which can then use them to claim capabilities they receive and
retrieve capabilities which they own by name, in addition to creating new capabilities
& authenticating capabilities passed by other modules. A scoped keeper cannot escape its scope,
so a module cannot interfere with or inspect capabilities owned by other modules.

The keeper provides no other core functionality that can be found in other modules
like queriers, REST and CLI handlers, and genesis state.

## Initialization

During application initialization, the keeper must be instantiated with a persistent
store key and an in-memory store key.

```go
type App struct {
  // ...

  capabilityKeeper *capability.Keeper
}

func NewApp(...) *App {
  // ...

  app.capabilityKeeper = capability.NewKeeper(codec, persistentStoreKey, memStoreKey)
}
```

After the keeper is created, it can be used to create scoped sub-keepers which
are passed to other modules that can create, authenticate, and claim capabilities.
After all the necessary scoped keepers are created and the state is loaded, the
main capability keeper must be sealed to prevent further scoped keepers from
being created.

```go
func NewApp(...) *App {
  // ...

  // Creating a scoped keeper
  scopedIBCKeeper := app.CapabilityKeeper.ScopeToModule(ibchost.ModuleName)

  // Seal the capability keeper to prevent any further modules from creating scoped
  // sub-keepers.
  app.capabilityKeeper.Seal()

  return app
}
```

## Contents

- [`modules/capability`](#capability-module)
  - [Overview](#overview)
  - [Initialization](#initialization)
  - [Contents](#contents)
  - [Concepts](#concepts)
    - [Capabilities](#capabilities)
    - [Stores](#stores)
  - [State](#state)
    - [In persisted KV store](#in-persisted-kv-store)
    - [In-memory KV store](#in-memory-kv-store)

## Concepts

### Capabilities

Capabilities are multi-owner. A scoped keeper can create a capability via `NewCapability`
which creates a new unique, unforgeable object-capability reference. The newly
created capability is automatically persisted; the calling module need not call
`ClaimCapability`. Calling `NewCapability` will create the capability with the
calling module and name as a tuple to be treated the capabilities first owner.

Capabilities can be claimed by other modules which add them as owners. `ClaimCapability`
allows a module to claim a capability key which it has received from another
module so that future `GetCapability` calls will succeed. `ClaimCapability` MUST
be called if a module which receives a capability wishes to access it by name in
the future. Again, capabilities are multi-owner, so if multiple modules have a
single Capability reference, they will all own it. If a module receives a capability
from another module but does not call `ClaimCapability`, it may use it in the executing
transaction but will not be able to access it afterwards.

`AuthenticateCapability` can be called by any module to check that a capability
does in fact correspond to a particular name (the name can be un-trusted user input)
with which the calling module previously associated it.

`GetCapability` allows a module to fetch a capability which it has previously
claimed by name. The module is not allowed to retrieve capabilities which it does
not own.

### Stores

* MemStore
* KeyStore

## State

### In persisted KV store

1. Global unique capability index
2. Capability owners

Indexes:

* Unique index: `[]byte("index") -> []byte(currentGlobalIndex)`
* Capability Index: `[]byte("capability_index") | []byte(index) -> ProtocolBuffer(CapabilityOwners)`

### In-memory KV store

1. Initialized flag
2. Mapping between the module and capability tuple and the capability name
3. Mapping between the module and capability name and its index

Indexes:

* Initialized flag: `[]byte("mem_initialized")`
* RevCapabilityKey: `[]byte(moduleName + "/rev/" + capabilityName) -> []byte(index)`
* FwdCapabilityKey: `[]byte(moduleName + "/fwd/" + capabilityPointerAddress) -> []byte(capabilityName)`
