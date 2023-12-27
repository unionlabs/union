# Wiring up app modules for use with appconfig

The `appconfig` framework allows Cosmos SDK modules to be composed declaratively using a configuration file without
requiring the app developer to understand the details of inter-module dependencies.

## 1. Create a module config protobuf message

The first step in creating a module that works with `appconfig`, is to create a protobuf message for the module configuration. The best practices for defining the module configuration message are:

* Use a dedicated protobuf package for the module configuration message  instead of placing it in the API protobuf package. For example, the module configuration for bank would go in `cosmos.bank.module.v1` instead of just `cosmos.bank.v1`. This decouples the state machine version from the API version.
* The module configuration message is usually called simply `Module`, ex. `cosmos.bank.module.v1.Module`.
* Create a new protobuf package and configuration message for each state machine breaking version of the module, ex. `cosmos.bank.module.v2.Module`, etc.

The module configuration message should include any parameters which should be initialized at application startup. For example, the auth module needs to know the bech32 prefix of the app and the permissions of module accounts.

In the future, it may be possible to update the app config through a governance proposal at runtime.

All module configuration messages should define a module descriptor, using the `cosmos.app.v1alpha1.module` message option.

Here is an example module configuration message for the `auth` module:

```protobuf
package cosmos.auth.module.v1;

import "cosmos/app/v1alpha1/module.proto";

message Module {
  option (cosmos.app.v1alpha1.module) = {
    go_import: "github.com/cosmos/cosmos-sdk/x/auth"
  };
  string bech32_prefix = 1;
  repeated ModuleAccountPermission module_account_permissions = 2;
}
```

## 2. Register module depinject providers and invokers

Once we have a module config object, we need to register depinject providers and invokers for the module using the `cosmossdk.io/core/appmodule` package.

At the most basic level, we must define an `init` function in the package listed as the `go_import` in the module descriptor. This `init` function must call `appmodule.Register` with an empty instance of the config object and some options for initializing the module, ex:

```go
func init() {
	appmodule.Register(&modulev1.Module{},
    // options
  )
}
```

### `depinject` Provider and Invoker Basics

A `depinject` "provider" is a function which takes dependencies from other modules as inputs and returns outputs for
other modules to use as dependencies. A `depinject` "invoker" is function which takes optional dependencies as inputs,
returns no outputs, and is run at the end of initializing the dependency graph. Providers are much more common than
invokers and should be the preferred method of wiring up modules when possible. Providers and invokers can be registered
for modules by using `appmodule.Provide` and `appmodule.Invoke` to create options which get passed
to `appmodule.Register` in the module `init` function, ex:

```go
func init() {
  appmodule.Register(&modulev1.Module{},
	  appmodule.Provide(provideSomething, provideSomethingElse),
	  appmodule.Invoke(invokeSomething),
  )
}
```

### `depinject` Types

`depinject` constructor functions support these classes of input and output parameter types:

* regular golang types (with special treatment of interface types as input parameters)
* structs with `depinject.In` and `depinject.Out` embedded
* `depinject.OnePerModuleType`s
* `depinject.ManyPerContainerType`s
* `depinject.ModuleKey` (which can only be defined as an input type)
* `error` (which gets special treatment as an output type)

#### Regular Golang Types

Regular golang types (besides the special cases described above) can be provided as both input and output parameters
to providers and invokers. For `depinject` to match an output parameter of one provider to an input parameter of 
another, there must be an exact match for the type unless the input parameter is an input type. For instance, if
a provider defines a dependency on `Foo` and some module provides `*Foo`, these two types will not match and there
will be an error.

#### Interface Input Types

When interfaces are used as input parameters to providers and invokers, `depinject` will search the container for
all types that implement this interface. If there is an unambiguously matching type, then this type will be used
to satisfy that interface. If there is a conflict between two types matching the interface, the app developer
can use `golang_bindings` options in their app config in order to resolve the conflict.

#### Structs with embedded `depinject.In` and `depinject.Out`

Structs that have `depinject.In` or `depinject.Out` as an embedded field are treated specially by `depinject`, where
all of these structs fields are treated as input or output parameters, respectively. These structs allow custom options
to be defined using struct field tags. Currently, the only supported custom option is `optional:"true"` which marks
a field as optional.

#### `depinject.OnePerModuleType`s

Any type which implements the `depinject.OnePerModuleType` interface can be provided at most once by every module.
These types can be collected as an input parameter to some provider or invoker by defining an input parameter which
is a map of module names as strings to this parameter type. For example if `Foo` is a `OnePerModuleType`, then
`map[string]Foo` can be declared as an input parameter by some provider (which obviously cannot provide an instance of
`Foo` itself because that would cause a circular dependency).

`OnePerModuleType`s should be used whenever different modules may provide the type *and* there is a need to provide
an ordering of these types based on the module name. Generally, in blockchains there is always a need for deterministic
orderings so using module names to provide that ordering is generally a good strategy for this use case. Ordering based
on module names can either be done implicitly by sorting the module names or explicitly as a parameter in the module
configuration object.

#### `depinject.ManyPerContainerType`s

`ManyPerContainerType`s can be provided by as many providers in as many modules as the user would like. If a type `Bar`
is a `ManyPerContainerType`, a provider may define an output parameter of `Bar` or `[]Bar` to provide `Bar` instances
to the container. A provider may define an input parameter of `[]Bar` to get all of the `Bar` instances in the
container (such a provider may not also return `Bar` as that would cause a circular dependency). The ordering of `Bar`
instances in the `[]Bar` input type should be assumed to be deterministic.

`ManyPerContainerType`s should be used only when 1) ordering is unimportant or 2) the ordering can be defined by
some parameter on the type. For instance, if `Bar` had a field `Name string`, that is supposed to be unique in the
container then that could be used to provide an ordering. An example of a type that could work as a
`ManyPerContainerType` in this way is a wrapper around `*cobra.Command`, ex.
`type QueryCommand struct {*cobra.Command}`. This could be used to collect all the query commands in an app and then
cobra would take care of ordering. If this type of ordering is not available, a `OnePerModuleType` is probably a better
bet.

#### Module-scoped Providers/`depinject.ModuleKey` as an input

If `depinject.ModuleKey` is used as input parameter for a provider, the provider function will be treated as a
"module-scoped provider" which means that the provider function will be called exactly once every time
one of its outputs is needed by a module so that the provider can provide a unique instance of the dependency to
each module. 

Module-scoped dependencies should be used to provide dependencies which are private and unique to each module. Examples
of these are store keys and param subspaces.

#### `error` as an output parameter

`error` can be used as the last output parameter on any provider or invoker. If a provider or invoker with an `error`
parameter returns a non-nil value for `error`, `depinject` will fail and propagate this error up to the caller.

### Provider Invocation Details

Providers are called lazily as they are needed and will be invoked at most once (except for "module-scoped providers"
described above) if and only if at least one of their outputs is needed somewhere in the dependency graph. Providers
will only get called if all of their non-optional inputs can successfully be resolved by some other module, otherwise an
error will occur. Modules should proactively mark dependencies as `optional` if the module can still be successfully
built without this dependency.

### Invoker Invocation Details

Invokers are called at the end of container initialization after all providers that were needed to build the graph
were called. All the dependencies of invokers are automatically marked as optional so invokers should `nil` check
every input parameter. Invokers may cause additional providers to get run if they have a dependency that wasn't built
yet. But if a dependency to an invoker cannot be provided for some reason, the invoker will still get called but with
`nil` for that input. This allows invokers to still work with the lazy invocation model of providers which only
builds things which are actually necessary as a dependency for some module or the caller.

Invokers should generally be used sparingly to perform some initialization logic which can't be done in the initial
provider, usually because of a circular dependency, and which may be optional.

### Best practices

* make dependencies `optional` whenever possible!
* interface types should be used whenever possible to avoid tight couplings between two modules.
* `OnePerModuleType`s should be used when there is something occurs at most once per module and the module name is a
convenient way for providing a deterministic order.
* `ManyPerContainerType`s should be used only when there is an obvious way to create an ordering from the types or
when ordering *really* doesn't matter (which is rare).
* module-scoped providers should be used for private, module-scoped dependencies
* use different providers for unrelated or loosely components or to resolve circular dependencies (see below)
* use invokers sparingly and always `nil`-check the inputs.

### Resolving Circular Dependencies

Circular dependencies are inevitable to crop up and there are ways to avoid them. While `depinject` cannot handle 
circular dependency graphs of providers, many of the above tools are designed to enable satisfying circular dependencies
between modules.

One of the key tactics for resolving circular dependencies is to use different providers and/or invokers to allow a
circular dependency between components. For example, say the slashing keeper depends on the keeper module but the
staking keeper also depends on the staking module indirectly (in the form of "staking hooks"). The slashing module
can declare a dependency directly on the staking keeper (using an interface to avoid actually importing the staking
keeper package). It can also provide an instance of the slashing keeper wrapped as staking hooks in a `OnePerModuleType`
we'll call `StakingHooksWrapper`. Now, if the staking module directly depended on the staking hooks wrappers
(`map[string]StakingHooksWrapper`) we would have a circular dependency graph and `depinject` would fail. To fix this,
the staking module can define an invoker which depends on `map[string]StakingHooksWrapper` and the staking keeper
(which was provided by the staking module already in a separate provided). In this way `depinject` will be able to
satisfy this dependency graph which allows staking and slashing to depend on each other in this order:

* provide staking keeper -> slashing keeper
* provide slashing keeper wrapped as `StakingHooksWrapper` 
* get `map[string]StakingHooksWrapper` and the staking keeper and wire them together

## 3. Testing and Debugging The Module

In order to test and debug the module configuration, we need to build an app config, generally defined in a YAML file.
This configuration should be passed first to `appconfig.LoadYAML` to get an `depinject.Config` instance.Then the
`depinject.Config` can be passed to `depinject.Inject` and we can try to resolve dependencies in the app config.
Alternatively, the `depinject.Config` can be created via [pure Go code](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app_config.go).

Ex:

```go
//go:embed app.yaml
var appConfig []byte

var AppConfig = appconfig.LoadYAML(appConfig)

func TestModule(t *testing.T) {
	var keeper Keeper
	assert.NilError(t, depinject.Inject(AppConfig, &keeper))
}
```

### Debugging `depinject` Graphs

Whenever there is an error in a `depinject` graph, by default `depinject` will dump a bunch of logging output to the
console, print the error message, and save the dependency graph in [GraphViz](https://graphviz.org) DOT format to
the file `debug_container.dot`. Inspecting the GraphViz output by converting it to an SVG and viewing it in a web
browser or using some other GraphViz tool is *highly recommended*.

If `depinject` does not return an error but there is still some weird issue wiring up modules, inspecting the GraphViz
and logging output is still *highly recommended* and can be done using `depinject.InjectDebug` with the debug option
`depinject.Debug`.

App developers should attempt to familiarize themselves with the GraphViz graph of their app to see which modules
depend on which other modules.
