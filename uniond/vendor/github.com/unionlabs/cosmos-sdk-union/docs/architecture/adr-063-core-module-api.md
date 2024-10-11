# ADR 063: Core Module API

## Changelog

* 2022-08-18 First Draft
* 2022-12-08 First Draft
* 2023-01-24 Updates

## Status

ACCEPTED Partially Implemented

## Abstract

A new core API is proposed as a way to develop cosmos-sdk applications that will eventually replace the existing
`AppModule` and `sdk.Context` frameworks a set of core services and extension interfaces. This core API aims to:

* be simpler
* more extensible
* more stable than the current framework
* enable deterministic events and queries,
* support event listeners
* [ADR 033: Protobuf-based Inter-Module Communication](./adr-033-protobuf-inter-module-comm.md) clients.

## Context

Historically modules have exposed their functionality to the framework via the `AppModule` and `AppModuleBasic`
interfaces which have the following shortcomings:

* both `AppModule` and `AppModuleBasic` need to be defined and registered which is counter-intuitive
* apps need to implement the full interfaces, even parts they don't need (although there are workarounds for this),
* interface methods depend heavily on unstable third party dependencies, in particular Comet,
* legacy required methods have littered these interfaces for far too long

In order to interact with the state machine, modules have needed to do a combination of these things:

* get store keys from the app
* call methods on `sdk.Context` which contains more or less the full set of capability available to modules.

By isolating all the state machine functionality into `sdk.Context`, the set of functionalities available to
modules are tightly coupled to this type. If there are changes to upstream dependencies (such as Comet)
or new functionalities are desired (such as alternate store types), the changes need impact `sdk.Context` and all
consumers of it (basically all modules). Also, all modules now receive `context.Context` and need to convert these
to `sdk.Context`'s with a non-ergonomic unwrapping function.

Any breaking changes to these interfaces, such as ones imposed by third-party dependencies like Comet, have the
side effect of forcing all modules in the ecosystem to update in lock-step. This means it is almost impossible to have
a version of the module which can be run with 2 or 3 different versions of the SDK or 2 or 3 different versions of
another module. This lock-step coupling slows down overall development within the ecosystem and causes updates to
components to be delayed longer than they would if things were more stable and loosely coupled.

## Decision

The `core` API proposes a set of core APIs that modules can rely on to interact with the state machine and expose their
functionalities to it that are designed in a principled way such that:

* tight coupling of dependencies and unrelated functionalities is minimized or eliminated
* APIs can have long-term stability guarantees
* the SDK framework is extensible in a safe and straightforward way

The design principles of the core API are as follows:

* everything that a module wants to interact with in the state machine is a service
* all services coordinate state via `context.Context` and don't try to recreate the "bag of variables" approach of `sdk.Context`
* all independent services are isolated in independent packages with minimal APIs and minimal dependencies
* the core API should be minimalistic and designed for long-term support (LTS)
* a "runtime" module will implement all the "core services" defined by the core API and can handle all module
  functionalities exposed by core extension interfaces
* other non-core and/or non-LTS services can be exposed by specific versions of runtime modules or other modules 
following the same design principles, this includes functionality that interacts with specific non-stable versions of
third party dependencies such as Comet
* the core API doesn't implement *any* functionality, it just defines types
* go stable API compatibility guidelines are followed: https://go.dev/blog/module-compatibility

A "runtime" module is any module which implements the core functionality of composing an ABCI app, which is currently
handled by `BaseApp` and the `ModuleManager`. Runtime modules which implement the core API are *intentionally* separate
from the core API in order to enable more parallel versions and forks of the runtime module than is possible with the
SDK's current tightly coupled `BaseApp` design while still allowing for a high degree of composability and
compatibility.

Modules which are built only against the core API don't need to know anything about which version of runtime,
`BaseApp` or Comet in order to be compatible. Modules from the core mainline SDK could be easily composed
with a forked version of runtime with this pattern.

This design is intended to enable matrices of compatible dependency versions. Ideally a given version of any module
is compatible with multiple versions of the runtime module and other compatible modules. This will allow dependencies
to be selectively updated based on battle-testing. More conservative projects may want to update some dependencies
slower than more fast moving projects.

### Core Services

The following "core services" are defined by the core API. All valid runtime module implementations should provide
implementations of these services to modules via both [dependency injection](./adr-057-app-wiring.md) and
manual wiring. The individual services described below are all bundled in a convenient `appmodule.Service`
"bundle service" so that for simplicity modules can declare a dependency on a single service.

#### Store Services

Store services will be defined in the `cosmossdk.io/core/store` package.

The generic `store.KVStore` interface is the same as current SDK `KVStore` interface. Store keys have been refactored
into store services which, instead of expecting the context to know about stores, invert the pattern and allow
retrieving a store from a generic context. There are three store services for the three types of currently supported
stores - regular kv-store, memory, and transient:

```go
type KVStoreService interface {
    OpenKVStore(context.Context) KVStore
}

type MemoryStoreService interface {
    OpenMemoryStore(context.Context) KVStore
}
type TransientStoreService interface {
    OpenTransientStore(context.Context) KVStore
}
```

Modules can use these services like this:

```go
func (k msgServer) Send(ctx context.Context, msg *types.MsgSend) (*types.MsgSendResponse, error) {
    store := k.kvStoreSvc.OpenKVStore(ctx)
}
```

Just as with the current runtime module implementation, modules will not need to explicitly name these store keys,
but rather the runtime module will choose an appropriate name for them and modules just need to request the
type of store they need in their dependency injection (or manual) constructors.

#### Event Service

The event `Service` will be defined in the `cosmossdk.io/core/event` package.

The event `Service` allows modules to emit typed and legacy untyped events:

```go
package event

type Service interface {
  // EmitProtoEvent emits events represented as a protobuf message (as described in ADR 032).
  //
  // Callers SHOULD assume that these events may be included in consensus. These events
  // MUST be emitted deterministically and adding, removing or changing these events SHOULD
  // be considered state-machine breaking.
  EmitProtoEvent(ctx context.Context, event protoiface.MessageV1) error

  // EmitKVEvent emits an event based on an event and kv-pair attributes.
  //
  // These events will not be part of consensus and adding, removing or changing these events is
  // not a state-machine breaking change.
  EmitKVEvent(ctx context.Context, eventType string, attrs ...KVEventAttribute) error

  // EmitProtoEventNonConsensus emits events represented as a protobuf message (as described in ADR 032), without
  // including it in blockchain consensus.
  //
  // These events will not be part of consensus and adding, removing or changing events is
  // not a state-machine breaking change.
  EmitProtoEventNonConsensus(ctx context.Context, event protoiface.MessageV1) error
}
```

Typed events emitted with `EmitProto`  should be assumed to be part of blockchain consensus (whether they are part of
the block or app hash is left to the runtime to specify).

Events emitted by `EmitKVEvent` and `EmitProtoEventNonConsensus` are not considered to be part of consensus and cannot be observed
by other modules. If there is a client-side need to add events in patch releases, these methods can be used.

#### Logger

A logger (`cosmossdk.io/log`) must be supplied using `depinject`, and will
be made available for modules to use via `depinject.In`.
Modules using it should follow the current pattern in the SDK by adding the module name before using it.

```go
type ModuleInputs struct {
  depinject.In

  Logger log.Logger
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
  keeper := keeper.NewKeeper(
    in.logger,
  )
}

func NewKeeper(logger log.Logger) Keeper {
  return Keeper{
    logger: logger.With(log.ModuleKey, "x/"+types.ModuleName),
  }
}
```

```

### Core `AppModule` extension interfaces


Modules will provide their core services to the runtime module via extension interfaces built on top of the
`cosmossdk.io/core/appmodule.AppModule` tag interface. This tag interface requires only two empty methods which
allow `depinject` to identify implementers as `depinject.OnePerModule` types and as app module implementations:

```go
type AppModule interface {
  depinject.OnePerModuleType

  // IsAppModule is a dummy method to tag a struct as implementing an AppModule.
  IsAppModule()
}
```

Other core extension interfaces will be defined in `cosmossdk.io/core` should be supported by valid runtime
implementations.

#### `MsgServer` and `QueryServer` registration

`MsgServer` and `QueryServer` registration is done by implementing the `HasServices` extension interface:

```go
type HasServices interface {
	AppModule

	RegisterServices(grpc.ServiceRegistrar)
}

```

Because of the `cosmos.msg.v1.service` protobuf option, required for `Msg` services, the same `ServiceRegitrar` can be
used to register both `Msg` and query services.

#### Genesis

The genesis `Handler` functions - `DefaultGenesis`, `ValidateGenesis`, `InitGenesis` and `ExportGenesis` - are specified
against the `GenesisSource` and `GenesisTarget` interfaces which will abstract over genesis sources which may be a single
JSON object or collections of JSON objects that can be efficiently streamed.

```go
// GenesisSource is a source for genesis data in JSON format. It may abstract over a
// single JSON object or separate files for each field in a JSON object that can
// be streamed over. Modules should open a separate io.ReadCloser for each field that
// is required. When fields represent arrays they can efficiently be streamed
// over. If there is no data for a field, this function should return nil, nil. It is
// important that the caller closes the reader when done with it.
type GenesisSource = func(field string) (io.ReadCloser, error)

// GenesisTarget is a target for writing genesis data in JSON format. It may
// abstract over a single JSON object or JSON in separate files that can be
// streamed over. Modules should open a separate io.WriteCloser for each field
// and should prefer writing fields as arrays when possible to support efficient
// iteration. It is important the caller closers the writer AND checks the error
// when done with it. It is expected that a stream of JSON data is written
// to the writer.
type GenesisTarget = func(field string) (io.WriteCloser, error)
```

All genesis objects for a given module are expected to conform to the semantics of a JSON object.
Each field in the JSON object should be read and written separately to support streaming genesis.
The [ORM](./adr-055-orm.md) and [collections](./adr-062-collections-state-layer.md) both support
streaming genesis and modules using these frameworks generally do not need to write any manual
genesis code.

To support genesis, modules should implement the `HasGenesis` extension interface:

```go
type HasGenesis interface {
	AppModule

	// DefaultGenesis writes the default genesis for this module to the target.
	DefaultGenesis(GenesisTarget) error

	// ValidateGenesis validates the genesis data read from the source.
	ValidateGenesis(GenesisSource) error

	// InitGenesis initializes module state from the genesis source.
	InitGenesis(context.Context, GenesisSource) error

	// ExportGenesis exports module state to the genesis target.
	ExportGenesis(context.Context, GenesisTarget) error
}
```

#### Pre Blockers

Modules that have functionality that runs before BeginBlock and should implement the `HasPreBlocker` interfaces:

```go
type HasPreBlocker interface {
  AppModule
  PreBlock(context.Context) error
}
```

#### Begin and End Blockers

Modules that have functionality that runs before transactions (begin blockers) or after transactions
(end blockers) should implement the `HasBeginBlocker` and/or `HasEndBlocker` interfaces:

```go
type HasBeginBlocker interface {
  AppModule
  BeginBlock(context.Context) error
}

type HasEndBlocker interface {
  AppModule
  EndBlock(context.Context) error
}
```

The `BeginBlock` and `EndBlock` methods will take a `context.Context`, because:

* most modules don't need Comet information other than `BlockInfo` so we can eliminate dependencies on specific
Comet versions
* for the few modules that need Comet block headers and/or return validator updates, specific versions of the
runtime module will provide specific functionality for interacting with the specific version(s) of Comet
supported

In order for `BeginBlock`, `EndBlock` and `InitGenesis` to send back validator updates and retrieve full Comet
block headers, the runtime module for a specific version of Comet could provide services like this:

```go
type ValidatorUpdateService interface {
    SetValidatorUpdates(context.Context, []abci.ValidatorUpdate)
}
```

Header Service defines a way to get header information about a block. This information is generalized for all implementations: 

```go 

type Service interface {
	HeaderInfo(context.Context) Info
}

type Info struct {
	Height int64      // Height returns the height of the block
	Hash []byte       // Hash returns the hash of the block header
	Time time.Time    // Time returns the time of the block
	ChainID string    // ChainId returns the chain ID of the block
}
```

Comet Service provides a way to get comet specific information: 

```go
type Service interface {
	GetCometInfo(context.Context) Info
}

type CometInfo struct {
  Evidence []abci.Misbehavior // Misbehavior returns the misbehavior of the block
	// ValidatorsHash returns the hash of the validators
	// For Comet, it is the hash of the next validators
	ValidatorsHash []byte
	ProposerAddress []byte            // ProposerAddress returns the address of the block proposer
	DecidedLastCommit abci.CommitInfo // DecidedLastCommit returns the last commit info
}
```

If a user would like to provide a module other information they would need to implement another service like:

```go
type RollKit Interface {
  ...
}
```

We know these types will change at the Comet level and that also a very limited set of modules actually need this
functionality, so they are intentionally kept out of core to keep core limited to the necessary, minimal set of stable
APIs.

#### Remaining Parts of AppModule

The current `AppModule` framework handles a number of additional concerns which aren't addressed by this core API.
These include:

* gas
* block headers
* upgrades
* registration of gogo proto and amino interface types
* cobra query and tx commands
* gRPC gateway 
* crisis module invariants
* simulations

Additional `AppModule` extension interfaces either inside or outside of core will need to be specified to handle
these concerns.

In the case of gogo proto and amino interfaces, the registration of these generally should happen as early
as possible during initialization and in [ADR 057: App Wiring](./adr-057-app-wiring.md), protobuf type registration  
happens before dependency injection (although this could alternatively be done dedicated DI providers).

gRPC gateway registration should probably be handled by the runtime module, but the core API shouldn't depend on gRPC
gateway types as 1) we are already using an older version and 2) it's possible the framework can do this registration
automatically in the future. So for now, the runtime module should probably provide some sort of specific type for doing
this registration ex:

```go
type GrpcGatewayInfo struct {
    Handlers []GrpcGatewayHandler
}

type GrpcGatewayHandler func(ctx context.Context, mux *runtime.ServeMux, client QueryClient) error
```

which modules can return in a provider:

```go
func ProvideGrpcGateway() GrpcGatewayInfo {
    return GrpcGatewayinfo {
        Handlers: []Handler {types.RegisterQueryHandlerClient}
    }
}
```

Crisis module invariants and simulations are subject to potential redesign and should be managed with types
defined in the crisis and simulation modules respectively.

Extension interface for CLI commands will be provided via the `cosmossdk.io/client/v2` module and its
[autocli](./adr-058-auto-generated-cli.md) framework.

#### Example Usage

Here is an example of setting up a hypothetical `foo` v2 module which uses the [ORM](./adr-055-orm.md) for its state
management and genesis.

```go

type Keeper struct {
	db orm.ModuleDB
	evtSrv event.Service
}

func (k Keeper) RegisterServices(r grpc.ServiceRegistrar) {
  foov1.RegisterMsgServer(r, k)
  foov1.RegisterQueryServer(r, k)
}

func (k Keeper) BeginBlock(context.Context) error {
	return nil
}

func ProvideApp(config *foomodulev2.Module, evtSvc event.EventService, db orm.ModuleDB) (Keeper, appmodule.AppModule){
    k := &Keeper{db: db, evtSvc: evtSvc}
    return k, k
}
```

### Runtime Compatibility Version

The `core` module will define a static integer var, `cosmossdk.io/core.RuntimeCompatibilityVersion`, which is
a minor version indicator of the core module that is accessible at runtime. Correct runtime module implementations
should check this compatibility version and return an error if the current `RuntimeCompatibilityVersion` is higher
than the version of the core API that this runtime version can support. When new features are added to the `core`
module API that runtime modules are required to support, this version should be incremented.

### Runtime Modules

The initial `runtime` module will simply be created within the existing `github.com/cosmos/cosmos-sdk` go module
under the `runtime` package. This module will be a small wrapper around the existing `BaseApp`, `sdk.Context` and
module manager and follow the Cosmos SDK's existing [0-based versioning](https://0ver.org). To move to semantic
versioning as well as runtime modularity, new officially supported runtime modules will be created under the
`cosmossdk.io/runtime` prefix. For each supported consensus engine a semantically-versioned go module should be created
with a runtime implementation for that consensus engine. For example:

* `cosmossdk.io/runtime/comet`
* `cosmossdk.io/runtime/comet/v2`
* `cosmossdk.io/runtime/rollkit`
* etc.

These runtime modules should attempt to be semantically versioned even if the underlying consensus engine is not. Also,
because a runtime module is also a first class Cosmos SDK module, it should have a protobuf module config type.
A new semantically versioned module config type should be created for each of these runtime module such that there is a
1:1 correspondence between the go module and module config type. This is the same practice should be followed for every 
semantically versioned Cosmos SDK module as described in [ADR 057: App Wiring](./adr-057-app-wiring.md).

Currently, `github.com/cosmos/cosmos-sdk/runtime` uses the protobuf config type `cosmos.app.runtime.v1alpha1.Module`.
When we have a standalone v1 comet runtime, we should use a dedicated protobuf module config type such as
`cosmos.runtime.comet.v1.Module1`. When we release v2 of the comet runtime (`cosmossdk.io/runtime/comet/v2`) we should
have a corresponding `cosmos.runtime.comet.v2.Module` protobuf type.

In order to make it easier to support different consensus engines that support the same core module functionality as
described in this ADR, a common go module should be created with shared runtime components. The easiest runtime components
to share initially are probably the message/query router, inter-module client, service register, and event router.
This common runtime module should be created initially as the `cosmossdk.io/runtime/common` go module.

When this new architecture has been implemented, the main dependency for a Cosmos SDK module would be
`cosmossdk.io/core` and that module should be able to be used with any supported consensus engine (to the extent
that it does not explicitly depend on consensus engine specific functionality such as Comet's block headers). An
app developer would then be able to choose which consensus engine they want to use by importing the corresponding
runtime module. The current `BaseApp` would be refactored into the `cosmossdk.io/runtime/comet` module, the router
infrastructure in `baseapp/` would be refactored into `cosmossdk.io/runtime/common` and support ADR 033, and eventually
a dependency on `github.com/cosmos/cosmos-sdk` would no longer be required.

In short, modules would depend primarily on `cosmossdk.io/core`, and each `cosmossdk.io/runtime/{consensus-engine}`
would implement the `cosmossdk.io/core` functionality for that consensus engine.

On additional piece that would need to be resolved as part of this architecture is how runtimes relate to the server.
Likely it would make sense to modularize the current server architecture so that it can be used with any runtime even
if that is based on a consensus engine besides Comet. This means that eventually the Comet runtime would need to
encapsulate the logic for starting Comet and the ABCI app.

### Testing

A mock implementation of all services should be provided in core to allow for unit testing of modules
without needing to depend on any particular version of runtime. Mock services should
allow tests to observe service behavior or provide a non-production implementation - for instance memory
stores can be used to mock stores.

For integration testing, a mock runtime implementation should be provided that allows composing different app modules
together for testing without a dependency on runtime or Comet.

## Consequences

### Backwards Compatibility

Early versions of runtime modules should aim to support as much as possible modules built with the existing
`AppModule`/`sdk.Context` framework. As the core API is more widely adopted, later runtime versions may choose to
drop support and only support the core API plus any runtime module specific APIs (like specific versions of Comet).

The core module itself should strive to remain at the go semantic version `v1` as long as possible and follow design
principles that allow for strong long-term support (LTS).

Older versions of the SDK can support modules built against core with adaptors that convert wrap core `AppModule`
implementations in implementations of `AppModule` that conform to that version of the SDK's semantics as well
as by providing service implementations by wrapping `sdk.Context`.

### Positive

* better API encapsulation and separation of concerns
* more stable APIs
* more framework extensibility
* deterministic events and queries
* event listeners
* inter-module msg and query execution support
* more explicit support for forking and merging of module versions (including runtime)

### Negative

### Neutral

* modules will need to be refactored to use this API
* some replacements for `AppModule` functionality still need to be defined in follow-ups
  (type registration, commands, invariants, simulations) and this will take additional design work

## Further Discussions

* gas
* block headers
* upgrades
* registration of gogo proto and amino interface types
* cobra query and tx commands
* gRPC gateway
* crisis module invariants
* simulations

## References

* [ADR 033: Protobuf-based Inter-Module Communication](./adr-033-protobuf-inter-module-comm.md)
* [ADR 057: App Wiring](./adr-057-app-wiring.md)
* [ADR 055: ORM](./adr-055-orm.md)
* [ADR 028: Public Key Addresses](./adr-028-public-key-addresses.md)
* [Keeping Your Modules Compatible](https://go.dev/blog/module-compatibility)
