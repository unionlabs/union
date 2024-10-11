# ADR 057: App Wiring

## Changelog

* 2022-05-04: Initial Draft
* 2022-08-19: Updates
* 2024-01-12: Updates

## Status

PROPOSED Implemented

## Abstract

In order to make it easier to build Cosmos SDK modules and apps, we propose a new app wiring system based on
dependency injection and declarative app configurations to replace the current `app.go` code.

## Context

A number of factors have made the SDK and SDK apps in their current state hard to maintain. A symptom of the current
state of complexity is [`simapp/app.go`](https://github.com/cosmos/cosmos-sdk/blob/c3edbb22cab8678c35e21fe0253919996b780c01/simapp/app.go)
which contains almost 100 lines of imports and is otherwise over 600 lines of mostly boilerplate code that is
generally copied to each new project. (Not to mention the additional boilerplate which gets copied in `simapp/simd`.)

The large amount of boilerplate needed to bootstrap an app has made it hard to release independently versioned go
modules for Cosmos SDK modules as described in [ADR 053: Go Module Refactoring](./adr-053-go-module-refactoring.md).

In addition to being very verbose and repetitive, `app.go` also exposes a large surface area for breaking changes
as most modules instantiate themselves with positional parameters which forces breaking changes anytime a new parameter
(even an optional one) is needed.

Several attempts were made to improve the current situation including [ADR 033: Internal-Module Communication](./adr-033-protobuf-inter-module-comm.md)
and [a proof-of-concept of a new SDK](https://github.com/allinbits/cosmos-sdk-poc). The discussions around these
designs led to the current solution described here.

## Decision

In order to improve the current situation, a new "app wiring" paradigm has been designed to replace `app.go` which
involves:

* declaration configuration of the modules in an app which can be serialized to JSON or YAML
* a dependency-injection (DI) framework for instantiating apps from the that configuration

### Dependency Injection

When examining the code in `app.go` most of the code simply instantiates modules with dependencies provided either
by the framework (such as store keys) or by other modules (such as keepers). It is generally pretty obvious given
the context what the correct dependencies actually should be, so dependency-injection is an obvious solution. Rather
than making developers manually resolve dependencies, a module will tell the DI container what dependency it needs
and the container will figure out how to provide it.

We explored several existing DI solutions in golang and felt that the reflection-based approach in [uber/dig](https://github.com/uber-go/dig)
was closest to what we needed but not quite there. Assessing what we needed for the SDK, we designed and built
the Cosmos SDK [depinject module](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/depinject), which has the following
features:

* dependency resolution and provision through functional constructors, ex: `func(need SomeDep) (AnotherDep, error)`
* dependency injection `In` and `Out` structs which support `optional` dependencies
* grouped-dependencies (many-per-container) through the `ManyPerContainerType` tag interface
* module-scoped dependencies via `ModuleKey`s (where each module gets a unique dependency)
* one-per-module dependencies through the `OnePerModuleType` tag interface
* sophisticated debugging information and container visualization via GraphViz

Here are some examples of how these would be used in an SDK module:

* `StoreKey` could be a module-scoped dependency which is unique per module
* a module's `AppModule` instance (or the equivalent) could be a `OnePerModuleType`
* CLI commands could be provided with `ManyPerContainerType`s

Note that even though dependency resolution is dynamic and based on reflection, which could be considered a pitfall
of this approach, the entire dependency graph should be resolved immediately on app startup and only gets resolved
once (except in the case of dynamic config reloading which is a separate topic). This means that if there are any
errors in the dependency graph, they will get reported immediately on startup so this approach is only slightly worse
than fully static resolution in terms of error reporting and much better in terms of code complexity.

### Declarative App Config

In order to compose modules into an app, a declarative app configuration will be used. This configuration is based off
of protobuf and its basic structure is very simple:

```protobuf
package cosmos.app.v1;
message Config {
  repeated ModuleConfig modules = 1;
}

message ModuleConfig {
  string name = 1;
  google.protobuf.Any config = 2;
}
```

The configuration for every module is a protobuf message and modules will be identified and loaded based
on the protobuf type URL of their config object (ex. `cosmos.bank.module.v1.Module`). Modules are given a unique short `name`
to share resources across different versions of the same module which might have a different protobuf package
versions (ex. `cosmos.bank.module.v2.Module`). All module config objects should define the `cosmos.app.v1alpha1.module`
descriptor option which will provide additional useful metadata for the framework and which can also be indexed
in module registries.

An example app config in YAML might look like this:

```yaml
modules:
  - name: baseapp
    config:
      "@type": cosmos.baseapp.module.v1.Module
      begin_blockers: [staking, auth, bank]
      end_blockers: [bank, auth, staking]
      init_genesis: [bank, auth, staking]
  - name: auth
    config:
      "@type": cosmos.auth.module.v1.Module
      bech32_prefix: "foo"
  - name: bank
    config:
      "@type": cosmos.bank.module.v1.Module
  - name: staking
    config:
      "@type": cosmos.staking.module.v1.Module
```

In the above example, there is a hypothetical `baseapp` module which contains the information around ordering of
begin blockers, end blockers, and init genesis. Rather than lifting these concerns up to the module config layer,
they are themselves handled by a module which could allow a convenient way of swapping out different versions of
baseapp (for instance to target different versions of tendermint), without needing to change the rest of the config.
The `baseapp` module would then provide to the server framework (which sort of sits outside the ABCI app) an instance
of `abci.Application`.

In this model, an app is *modules all the way down* and the dependency injection/app config layer is very much
protocol-agnostic and can adapt to even major breaking changes at the protocol layer.

### Module & Protobuf Registration

In order for the two components of dependency injection and declarative configuration to work together as described,
we need a way for modules to actually register themselves and provide dependencies to the container.

One additional complexity that needs to be handled at this layer is protobuf registry initialization. Recall that
in both the current SDK `codec` and the proposed [ADR 054: Protobuf Semver Compatible Codegen](https://github.com/cosmos/cosmos-sdk/pull/11802),
protobuf types need to be explicitly registered. Given that the app config itself is based on protobuf and
uses protobuf `Any` types, protobuf registration needs to happen before the app config itself can be decoded. Because
we don't know which protobuf `Any` types will be needed a priori and modules themselves define those types, we need
to decode the app config in separate phases:

1. parse app config JSON/YAML as raw JSON and collect required module type URLs (without doing proto JSON decoding)
2. build a [protobuf type registry](https://pkg.go.dev/google.golang.org/protobuf@v1.28.0/reflect/protoregistry) based
   on file descriptors and types provided by each required module
3. decode the app config as proto JSON using the protobuf type registry

Because in [ADR 054: Protobuf Semver Compatible Codegen](https://github.com/cosmos/cosmos-sdk/pull/11802), each module
might use `internal` generated code which is not registered with the global protobuf registry, this code should provide
an alternate way to register protobuf types with a type registry. In the same way that `.pb.go` files currently have a
`var File_foo_proto protoreflect.FileDescriptor` for the file `foo.proto`, generated code should have a new member
`var Types_foo_proto TypeInfo` where `TypeInfo` is an interface or struct with all the necessary info to register both
the protobuf generated types and file descriptor.

So a module must provide dependency injection providers and protobuf types, and takes as input its module
config object which uniquely identifies the module based on its type URL.

With this in mind, we define a global module register which allows module implementations to register themselves
with the following API:

```go
// Register registers a module with the provided type name (ex. cosmos.bank.module.v1.Module)
// and the provided options.
func Register(configTypeName protoreflect.FullName, option ...Option) { ... }

type Option { /* private methods */ }

// Provide registers dependency injection provider functions which work with the
// cosmos-sdk container module. These functions can also accept an additional
// parameter for the module's config object.
func Provide(providers ...interface{}) Option { ... }

// Types registers protobuf TypeInfo's with the protobuf registry.
func Types(types ...TypeInfo) Option { ... }
```

Ex:

```go
func init() {
	appconfig.Register("cosmos.bank.module.v1.Module",
		appconfig.Types(
			types.Types_tx_proto,
            types.Types_query_proto,
            types.Types_types_proto,
	    ),
	    appconfig.Provide(
			provideBankModule,
	    )
	)
}

type Inputs struct {
	container.In
	
	AuthKeeper auth.Keeper
	DB ormdb.ModuleDB
}

type Outputs struct {
	Keeper bank.Keeper
	AppModule appmodule.AppModule
}

func ProvideBankModule(config *bankmodulev1.Module, Inputs) (Outputs, error) { ... }
```

Note that in this module, a module configuration object *cannot* register different dependency providers at runtime
based on the configuration. This is intentional because it allows us to know globally which modules provide which
dependencies, and it will also allow us to do code generation of the whole app initialization. This
can help us figure out issues with missing dependencies in an app config if the needed modules are loaded at runtime.
In cases where required modules are not loaded at runtime, it may be possible to guide users to the correct module if
through a global Cosmos SDK module registry.

The `*appmodule.Handler` type referenced above is a replacement for the legacy `AppModule` framework, and
described in [ADR 063: Core Module API](./adr-063-core-module-api.md).

### New `app.go`

With this setup, `app.go` might now look something like this:

```go
package main

import (
	// Each go package which registers a module must be imported just for side-effects
	// so that module implementations are registered.
	_ "github.com/cosmos/cosmos-sdk/x/auth/module"
	_ "cosmossdk.io/x/bank/module"
	_ "cosmossdk.io/x/staking/module"
	"github.com/cosmos/cosmos-sdk/core/app"
)

// go:embed app.yaml
var appConfigYAML []byte

func main() {
	app.Run(app.LoadYAML(appConfigYAML))
}
```

### Application to existing SDK modules

So far we have described a system which is largely agnostic to the specifics of the SDK such as store keys, `AppModule`,
`BaseApp`, etc. Improvements to these parts of the framework that integrate with the general app wiring framework
defined here are described in [ADR 063: Core Module API](./adr-063-core-module-api.md).

### Registration of Inter-Module Hooks

Some modules define a hooks interface (ex. `StakingHooks`) which allows one module to call back into another module
when certain events happen.

With the app wiring framework, these hooks interfaces can be defined as a `OnePerModuleType`s and then the module
which consumes these hooks can collect these hooks as a map of module name to hook type (ex. `map[string]FooHooks`). Ex:

```go
func init() {
    appconfig.RegisterModule(
        &foomodulev1.Module{},
        appconfig.Invoke(InvokeSetFooHooks),
	    ...
    )
}
func InvokeSetFooHooks(
    keeper *keeper.Keeper,
    fooHooks map[string]FooHooks,
) error {
	for k in sort.Strings(maps.Keys(fooHooks)) {
		keeper.AddFooHooks(fooHooks[k])
    }
}
```

Optionally, the module consuming hooks can allow app's to define an order for calling these hooks based on module name
in its config object.

An alternative way for registering hooks via reflection was considered where all keeper types are inspected to see if
they implement the hook interface by the modules exposing hooks. This has the downsides of:

* needing to expose all the keepers of all modules to the module providing hooks,
* not allowing for encapsulating hooks on a different type which doesn't expose all keeper methods,
* harder to know statically which module expose hooks or are checking for them.

With the approach proposed here, hooks registration will be obviously observable in `app.go` if `depinject` codegen
(described below) is used.

### Code Generation

> Not yet implemented

The `depinject` framework will optionally allow the app configuration and dependency injection wiring to be code
generated. This will allow:

* dependency injection wiring to be inspected as regular go code just like the existing `app.go`,
* dependency injection to be opt-in with manual wiring 100% still possible.

Code generation requires that all providers and invokers and their parameters are exported and in non-internal packages.

### Module Semantic Versioning

When we start creating semantically versioned SDK modules that are in standalone go modules, a state machine breaking
change to a module should be handled as follows:

* the semantic major version should be incremented, and
* a new semantically versioned module config protobuf type should be created.

For instance, if we have the SDK module for bank in the go module `cosmossdk.io/x/bank` with the module config type
`cosmos.bank.module.v1.Module`, and we want to make a state machine breaking change to the module, we would:

* create a new go module `cosmossdk.io/x/bank/v2`,
* with the module config protobuf type `cosmos.bank.module.v2.Module`.

This *does not* mean that we need to increment the protobuf API version for bank. Both modules can support
`cosmos.bank.v1`, but `cosmossdk.io/x/bank/v2` will be a separate go module with a separate module config type.

This practice will eventually allow us to use appconfig to load new versions of a module via a configuration change.

Effectively, there should be a 1:1 correspondence between a semantically versioned go module and a 
versioned module config protobuf type, and major versioning bumps should occur whenever state machine breaking changes
are made to a module.

NOTE: SDK modules that are standalone go modules *should not* adopt semantic versioning until the concerns described in
[ADR 054: Module Semantic Versioning](./adr-054-semver-compatible-modules.md) are
addressed. The short-term solution for this issue was left somewhat unresolved. However, the easiest tactic is
likely to use a standalone API go module and follow the guidelines described in this comment: https://github.com/cosmos/cosmos-sdk/pull/11802#issuecomment-1406815181. For the time-being, it is recommended that
Cosmos SDK modules continue to follow tried and true [0-based versioning](https://0ver.org) until an officially
recommended solution is provided. This section of the ADR will be updated when that happens and for now, this section
should be considered as a design recommendation for future adoption of semantic versioning.

## Consequences

### Backwards Compatibility

Modules which work with the new app wiring system do not need to drop their existing `AppModule` and `NewKeeper`
registration paradigms. These two methods can live side-by-side for as long as is needed.

### Positive

* wiring up new apps will be simpler, more succinct and less error-prone
* it will be easier to develop and test standalone SDK modules without needing to replicate all of simapp
* it may be possible to dynamically load modules and upgrade chains without needing to do a coordinated stop and binary
  upgrade using this mechanism
* easier plugin integration
* dependency injection framework provides more automated reasoning about dependencies in the project, with a graph visualization.

### Negative

* it may be confusing when a dependency is missing although error messages, the GraphViz visualization, and global
  module registration may help with that

### Neutral

* it will require work and education

## Further Discussions

The protobuf type registration system described in this ADR has not been implemented and may need to be reconsidered in
light of code generation. It may be better to do this type registration with a DI provider.

## References

* https://github.com/cosmos/cosmos-sdk/blob/c3edbb22cab8678c35e21fe0253919996b780c01/simapp/app.go
* https://github.com/allinbits/cosmos-sdk-poc
* https://github.com/uber-go/dig
* https://github.com/google/wire
* https://pkg.go.dev/github.com/cosmos/cosmos-sdk/container
* https://github.com/cosmos/cosmos-sdk/pull/11802
* [ADR 063: Core Module API](./adr-063-core-module-api.md)
