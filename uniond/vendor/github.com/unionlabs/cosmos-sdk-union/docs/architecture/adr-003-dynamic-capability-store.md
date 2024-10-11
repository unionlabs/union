# ADR 3: Dynamic Capability Store

## Changelog

* 12 December 2019: Initial version
* 02 April 2020: Memory Store Revisions

## Context

Full implementation of the [IBC specification](https://github.com/cosmos/ibc) requires the ability to create and authenticate object-capability keys at runtime (i.e., during transaction execution),
as described in [ICS 5](https://github.com/cosmos/ibc/tree/master/spec/core/ics-005-port-allocation#technical-specification). In the IBC specification, capability keys are created for each newly initialised
port & channel, and are used to authenticate future usage of the port or channel. Since channels and potentially ports can be initialised during transaction execution, the state machine must be able to create
object-capability keys at this time.

At present, the Cosmos SDK does not have the ability to do this. Object-capability keys are currently pointers (memory addresses) of `StoreKey` structs created at application initialisation in `app.go` ([example](https://github.com/cosmos/gaia/blob/dcbddd9f04b3086c0ad07ee65de16e7adedc7da4/app/app.go#L132))
and passed to Keepers as fixed arguments ([example](https://github.com/cosmos/gaia/blob/dcbddd9f04b3086c0ad07ee65de16e7adedc7da4/app/app.go#L160)). Keepers cannot create or store capability keys during transaction execution — although they could call `NewKVStoreKey` and take the memory address
of the returned struct, storing this in the Merklised store would result in a consensus fault, since the memory address will be different on each machine (this is intentional — were this not the case, the keys would be predictable and couldn't serve as object capabilities).

Keepers need a way to keep a private map of store keys which can be altered during transaction execution, along with a suitable mechanism for regenerating the unique memory addresses (capability keys) in this map whenever the application is started or restarted, along with a mechanism to revert capability creation on tx failure.
This ADR proposes such an interface & mechanism.

## Decision

The Cosmos SDK will include a new `CapabilityKeeper` abstraction, which is responsible for provisioning,
tracking, and authenticating capabilities at runtime. During application initialisation in `app.go`,
the `CapabilityKeeper` will be hooked up to modules through unique function references
(by calling `ScopeToModule`, defined below) so that it can identify the calling module when later
invoked.

When the initial state is loaded from disk, the `CapabilityKeeper`'s `Initialise` function will create
new capability keys for all previously allocated capability identifiers (allocated during execution of
past transactions and assigned to particular modes), and keep them in a memory-only store while the
chain is running.

The `CapabilityKeeper` will include a persistent `KVStore`, a `MemoryStore`, and an in-memory map.
The persistent `KVStore` tracks which capability is owned by which modules.
The `MemoryStore` stores a forward mapping that map from module name, capability tuples to capability names and
a reverse mapping that map from module name, capability name to the capability index.
Since we cannot marshal the capability into a `KVStore` and unmarshal without changing the memory location of the capability,
the reverse mapping in the KVStore will simply map to an index. This index can then be used as a key in the ephemeral
go-map to retrieve the capability at the original memory location.

The `CapabilityKeeper` will define the following types & functions:

The `Capability` is similar to `StoreKey`, but has a globally unique `Index()` instead of
a name. A `String()` method is provided for debugging.

A `Capability` is simply a struct, the address of which is taken for the actual capability.

```go
type Capability struct {
  index uint64
}
```

A `CapabilityKeeper` contains a persistent store key, memory store key, and mapping of allocated module names.

```go
type CapabilityKeeper struct {
  persistentKey StoreKey
  memKey        StoreKey
  capMap        map[uint64]*Capability
  moduleNames   map[string]interface{}
  sealed        bool
}
```

The `CapabilityKeeper` provides the ability to create *scoped* sub-keepers which are tied to a
particular module name. These `ScopedCapabilityKeeper`s must be created at application initialisation
and passed to modules, which can then use them to claim capabilities they receive and retrieve
capabilities which they own by name, in addition to creating new capabilities & authenticating capabilities
passed by other modules.

```go
type ScopedCapabilityKeeper struct {
  persistentKey StoreKey
  memKey        StoreKey
  capMap        map[uint64]*Capability
  moduleName    string
}
```

`ScopeToModule` is used to create a scoped sub-keeper with a particular name, which must be unique.
It MUST be called before `InitialiseAndSeal`.

```go
func (ck CapabilityKeeper) ScopeToModule(moduleName string) ScopedCapabilityKeeper {
	if ck.sealed {
		panic("cannot scope to module via a sealed capability keeper")
	}

	if _, ok := ck.scopedModules[moduleName]; ok {
		panic(fmt.Sprintf("cannot create multiple scoped keepers for the same module name: %s", moduleName))
	}

	ck.scopedModules[moduleName] = struct{}{}

	return ScopedKeeper{
		cdc:      ck.cdc,
		storeKey: ck.storeKey,
		memKey:   ck.memKey,
		capMap:   ck.capMap,
		module:   moduleName,
	}
}
```

`InitialiseAndSeal` MUST be called exactly once, after loading the initial state and creating all
necessary `ScopedCapabilityKeeper`s, in order to populate the memory store with newly-created
capability keys in accordance with the keys previously claimed by particular modules and prevent the
creation of any new `ScopedCapabilityKeeper`s.

```go
func (ck CapabilityKeeper) InitialiseAndSeal(ctx Context) {
  if ck.sealed {
    panic("capability keeper is sealed")
  }

  persistentStore := ctx.KVStore(ck.persistentKey)
  map := ctx.KVStore(ck.memKey)
  
  // initialise memory store for all names in persistent store
  for index, value := range persistentStore.Iter() {
    capability = &CapabilityKey{index: index}

    for moduleAndCapability := range value {
      moduleName, capabilityName := moduleAndCapability.Split("/")
      memStore.Set(moduleName + "/fwd/" + capability, capabilityName)
      memStore.Set(moduleName + "/rev/" + capabilityName, index)

      ck.capMap[index] = capability
    }
  }

  ck.sealed = true
}
```

`NewCapability` can be called by any module to create a new unique, unforgeable object-capability
reference. The newly created capability is automatically persisted; the calling module need not
call `ClaimCapability`.

```go
func (sck ScopedCapabilityKeeper) NewCapability(ctx Context, name string) (Capability, error) {
  // check name not taken in memory store
  if capStore.Get("rev/" + name) != nil {
    return nil, errors.New("name already taken")
  }

  // fetch the current index
  index := persistentStore.Get("index")
  
  // create a new capability
  capability := &CapabilityKey{index: index}
  
  // set persistent store
  persistentStore.Set(index, Set.singleton(sck.moduleName + "/" + name))
  
  // update the index
  index++
  persistentStore.Set("index", index)
  
  // set forward mapping in memory store from capability to name
  memStore.Set(sck.moduleName + "/fwd/" + capability, name)
  
  // set reverse mapping in memory store from name to index
  memStore.Set(sck.moduleName + "/rev/" + name, index)

  // set the in-memory mapping from index to capability pointer
  capMap[index] = capability
  
  // return the newly created capability
  return capability
}
```

`AuthenticateCapability` can be called by any module to check that a capability
does in fact correspond to a particular name (the name can be untrusted user input)
with which the calling module previously associated it.

```go
func (sck ScopedCapabilityKeeper) AuthenticateCapability(name string, capability Capability) bool {
  // return whether forward mapping in memory store matches name
  return memStore.Get(sck.moduleName + "/fwd/" + capability) === name
}
```

`ClaimCapability` allows a module to claim a capability key which it has received from another module
so that future `GetCapability` calls will succeed.

`ClaimCapability` MUST be called if a module which receives a capability wishes to access it by name
in the future. Capabilities are multi-owner, so if multiple modules have a single `Capability` reference,
they will all own it.

```go
func (sck ScopedCapabilityKeeper) ClaimCapability(ctx Context, capability Capability, name string) error {
  persistentStore := ctx.KVStore(sck.persistentKey)

  // set forward mapping in memory store from capability to name
  memStore.Set(sck.moduleName + "/fwd/" + capability, name)

  // set reverse mapping in memory store from name to capability
  memStore.Set(sck.moduleName + "/rev/" + name, capability)

  // update owner set in persistent store
  owners := persistentStore.Get(capability.Index())
  owners.add(sck.moduleName + "/" + name)
  persistentStore.Set(capability.Index(), owners)
}
```

`GetCapability` allows a module to fetch a capability which it has previously claimed by name.
The module is not allowed to retrieve capabilities which it does not own.

```go
func (sck ScopedCapabilityKeeper) GetCapability(ctx Context, name string) (Capability, error) {
  // fetch the index of capability using reverse mapping in memstore
  index := memStore.Get(sck.moduleName + "/rev/" + name)

  // fetch capability from go-map using index
  capability := capMap[index]

  // return the capability
  return capability
}
```

`ReleaseCapability` allows a module to release a capability which it had previously claimed. If no
more owners exist, the capability will be deleted globally.

```go
func (sck ScopedCapabilityKeeper) ReleaseCapability(ctx Context, capability Capability) err {
  persistentStore := ctx.KVStore(sck.persistentKey)

  name := capStore.Get(sck.moduleName + "/fwd/" + capability)
  if name == nil {
    return error("capability not owned by module")
  }

  // delete forward mapping in memory store
  memoryStore.Delete(sck.moduleName + "/fwd/" + capability, name)

  // delete reverse mapping in memory store
  memoryStore.Delete(sck.moduleName + "/rev/" + name, capability)

  // update owner set in persistent store
  owners := persistentStore.Get(capability.Index())
  owners.remove(sck.moduleName + "/" + name)
  if owners.size() > 0 {
    // there are still other owners, keep the capability around
    persistentStore.Set(capability.Index(), owners)
  } else {
    // no more owners, delete the capability
    persistentStore.Delete(capability.Index())
    delete(capMap[capability.Index()])
  }
}
```

### Usage patterns

#### Initialisation

Any modules which use dynamic capabilities must be provided a `ScopedCapabilityKeeper` in `app.go`:

```go
ck := NewCapabilityKeeper(persistentKey, memoryKey)
mod1Keeper := NewMod1Keeper(ck.ScopeToModule("mod1"), ....)
mod2Keeper := NewMod2Keeper(ck.ScopeToModule("mod2"), ....)

// other initialisation logic ...

// load initial state...

ck.InitialiseAndSeal(initialContext)
```

#### Creating, passing, claiming and using capabilities

Consider the case where `mod1` wants to create a capability, associate it with a resource (e.g. an IBC channel) by name, then pass it to `mod2` which will use it later:

Module 1 would have the following code:

```go
capability := scopedCapabilityKeeper.NewCapability(ctx, "resourceABC")
mod2Keeper.SomeFunction(ctx, capability, args...)
```

`SomeFunction`, running in module 2, could then claim the capability:

```go
func (k Mod2Keeper) SomeFunction(ctx Context, capability Capability) {
  k.sck.ClaimCapability(ctx, capability, "resourceABC")
  // other logic...
}
```

Later on, module 2 can retrieve that capability by name and pass it to module 1, which will authenticate it against the resource:

```go
func (k Mod2Keeper) SomeOtherFunction(ctx Context, name string) {
  capability := k.sck.GetCapability(ctx, name)
  mod1.UseResource(ctx, capability, "resourceABC")
}
```

Module 1 will then check that this capability key is authenticated to use the resource before allowing module 2 to use it:

```go
func (k Mod1Keeper) UseResource(ctx Context, capability Capability, resource string) {
  if !k.sck.AuthenticateCapability(name, capability) {
    return errors.New("unauthenticated")
  }
  // do something with the resource
}
```

If module 2 passed the capability key to module 3, module 3 could then claim it and call module 1 just like module 2 did
(in which case module 1, module 2, and module 3 would all be able to use this capability).

## Status

Proposed.

## Consequences

### Positive

* Dynamic capability support.
* Allows CapabilityKeeper to return same capability pointer from go-map while reverting any writes to the persistent `KVStore` and in-memory `MemoryStore` on tx failure.

### Negative

* Requires an additional keeper.
* Some overlap with existing `StoreKey` system (in the future they could be combined, since this is a superset functionality-wise).
* Requires an extra level of indirection in the reverse mapping, since MemoryStore must map to index which must then be used as key in a go map to retrieve the actual capability

### Neutral

(none known)

## References

* [Original discussion](https://github.com/cosmos/cosmos-sdk/pull/5230#discussion_r343978513)
