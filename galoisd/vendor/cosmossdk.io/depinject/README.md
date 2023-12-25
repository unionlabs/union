---
sidebar_position: 1
---

# Depinject

> **DISCLAIMER**: This is a **beta** package. The SDK team is actively working on this feature and we are looking for feedback from the community. Please try it out and let us know what you think.

## Overview

`depinject` is a dependency injection (DI) framework for the Cosmos SDK, designed to streamline the process of building and configuring blockchain applications. It works in conjunction with the `core/appconfig` module to replace the majority of boilerplate code in `app.go` with a configuration file in Go, YAML, or JSON format.

`depinject` is particularly useful for developing blockchain applications:

*   With multiple interdependent components, modules, or services. Helping manage their dependencies effectively.
*   That require decoupling of these components, making it easier to test, modify, or replace individual parts without affecting the entire system.
*   That are wanting to simplify the setup and initialisation of modules and their dependencies by reducing boilerplate code and automating dependency management.

By using `depinject`, developers can achieve:

*   Cleaner and more organised code.
*   Improved modularity and maintainability.
*   A more maintainable and modular structure for their blockchain applications, ultimately enhancing development velocity and code quality.

* [Go Doc](https://pkg.go.dev/cosmossdk.io/depinject)

## Usage

The `depinject` framework, based on dependency injection concepts, streamlines the management of dependencies within your blockchain application using its Configuration API. This API offers a set of functions and methods to create easy to use configurations, making it simple to define, modify, and access dependencies and their relationships.

A core component of the [Configuration API](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/depinject#Config) is the `Provide` function, which allows you to register provider functions that supply dependencies. Inspired by constructor injection, these provider functions form the basis of the dependency tree, enabling the management and resolution of dependencies in a structured and maintainable manner. Additionally, `depinject` supports interface types as inputs to provider functions, offering flexibility and decoupling between components, similar to interface injection concepts.

By leveraging `depinject` and its Configuration API, you can efficiently handle dependencies in your blockchain application, ensuring a clean, modular, and well-organised codebase.

Example:

```go
package main

import (
	"fmt"

	"cosmossdk.io/depinject"
)

type AnotherInt int

func main() {
	var (
	  x int
	  y AnotherInt
	)

	fmt.Printf("Before (%v, %v)\n", x, y)
	depinject.Inject(
		depinject.Provide(
			func() int { return 1 },
			func() AnotherInt { return AnotherInt(2) },
		),
		&x,
		&y,
	)
	fmt.Printf("After (%v, %v)\n", x, y)
}
```

In this example, `depinject.Provide` registers two provider functions that return `int` and `AnotherInt` values. The `depinject.Inject` function is then used to inject these values into the variables `x` and `y`.

Provider functions serve as the basis for the dependency tree. They are analysed to identify their inputs as dependencies and their outputs as dependents. These dependents can either be used by another provider function or be stored outside the DI container (e.g., `&x` and `&y` in the example above).

### Interface type resolution

`depinject` supports the use of interface types as inputs to provider functions, which helps decouple dependencies between modules. This approach is particularly useful for managing complex systems with multiple modules, such as the Cosmos SDK, where dependencies need to be flexible and maintainable.

For example, `x/bank` expects an [AccountKeeper](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/x/bank/types#AccountKeeper) interface as [input to ProvideModule](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/bank/module.go#L208-L260). `SimApp` uses the implementation in `x/auth`, but the modular design allows for easy changes to the implementation if needed.

Consider the following example:

```go
package duck

type Duck interface {
	quack()
}

type AlsoDuck interface {
	quack()
}

type Mallard struct{}
type Canvasback struct{}

func (duck Mallard) quack()    {}
func (duck Canvasback) quack() {}

type Pond struct {
	Duck AlsoDuck
}
```

In this example, there's a `Pond` struct that has a `Duck` field of type `AlsoDuck`. The `depinject` framework can automatically resolve the appropriate implementation when there's only one available, as shown below:

```go
var pond Pond

depinject.Inject(
  depinject.Provide(
    func() Mallard { return Mallard{} },
    func(duck Duck) Pond {
      return Pond{Duck: duck}
    }),
   &pond)
```

This code snippet results in the `Duck` field of `Pond` being implicitly bound to the `Mallard` implementation because it's the only implementation of the `Duck` interface in the container.

However, if there are multiple implementations of the `Duck` interface, as in the following example, you'll encounter an error:

```go
var pond Pond

depinject.Inject(
  depinject.Provide(
    func() Mallard { return Mallard{} },
    func() Canvasback { return Canvasback{} },
    func(duck Duck) Pond {
      return Pond{Duck: duck}
    }),
   &pond)
```

A specific binding preference for `Duck` is required.

#### `BindInterface` API

In the above situation registering a binding for a given interface binding may look like:

```go
depinject.Inject(
  depinject.Configs(
    depinject.BindInterface(
      "duck.Duck",
      "duck.Mallard"),
     depinject.Provide(
       func() Mallard { return Mallard{} },
       func() Canvasback { return Canvasback{} },
       func(duck Duck) APond {
         return Pond{Duck: duck}
      })),
   &pond)
```

Now `depinject` has enough information to provide `Mallard` as an input to `APond`. 

### Full example in real app

:::warning
When using `depinject.Inject`, the injected types must be pointers.
:::

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app_v2.go#L219-L244
```

## Debugging

Issues with resolving dependencies in the container can be done with logs and [Graphviz](https://graphviz.org) renderings of the container tree.
By default, whenever there is an error, logs will be printed to stderr and a rendering of the dependency graph in Graphviz DOT format will be saved to `debug_container.dot`.

Here is an example Graphviz rendering of a successful build of a dependency graph:
![Graphviz Example](https://raw.githubusercontent.com/cosmos/cosmos-sdk/ff39d243d421442b400befcd959ec3ccd2525154/depinject/testdata/example.svg)

Rectangles represent functions, ovals represent types, rounded rectangles represent modules and the single hexagon
represents the function which called `Build`. Black-colored shapes mark functions and types that were called/resolved
without an error. Gray-colored nodes mark functions and types that could have been called/resolved in the container but
were left unused.

Here is an example Graphviz rendering of a dependency graph build which failed:
![Graphviz Error Example](https://raw.githubusercontent.com/cosmos/cosmos-sdk/ff39d243d421442b400befcd959ec3ccd2525154/depinject/testdata/example_error.svg)

Graphviz DOT files can be converted into SVG's for viewing in a web browser using the `dot` command-line tool, ex:

```txt
dot -Tsvg debug_container.dot > debug_container.svg
```

Many other tools including some IDEs support working with DOT files.
