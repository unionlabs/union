---
sidebar_position: 1
---

# Anatomy of a Cosmos SDK Application

:::note Synopsis
This document describes the core parts of a Cosmos SDK application, represented throughout the document as a placeholder application named `app`.
:::

## Node Client

The Daemon, or [Full-Node Client](../core/03-node.md), is the core process of a Cosmos SDK-based blockchain. Participants in the network run this process to initialize their state-machine, connect with other full-nodes, and update their state-machine as new blocks come in.

```text
                ^  +-------------------------------+  ^
                |  |                               |  |
                |  |  State-machine = Application  |  |
                |  |                               |  |   Built with Cosmos SDK
                |  |            ^      +           |  |
                |  +----------- | ABCI | ----------+  v
                |  |            +      v           |  ^
                |  |                               |  |
Blockchain Node |  |           Consensus           |  |
                |  |                               |  |
                |  +-------------------------------+  |   CometBFT
                |  |                               |  |
                |  |           Networking          |  |
                |  |                               |  |
                v  +-------------------------------+  v
```

The blockchain full-node presents itself as a binary, generally suffixed by `-d` for "daemon" (e.g. `appd` for `app` or `gaiad` for `gaia`). This binary is built by running a simple [`main.go`](../core/03-node.md#main-function) function placed in `./cmd/appd/`. This operation usually happens through the [Makefile](#dependencies-and-makefile).

Once the main binary is built, the node can be started by running the [`start` command](../core/03-node.md#start-command). This command function primarily does three things:

1. Create an instance of the state-machine defined in [`app.go`](#core-application-file).
2. Initialize the state-machine with the latest known state, extracted from the `db` stored in the `~/.app/data` folder. At this point, the state-machine is at height `appBlockHeight`.
3. Create and start a new CometBFT instance. Among other things, the node performs a handshake with its peers. It gets the latest `blockHeight` from them and replays blocks to sync to this height if it is greater than the local `appBlockHeight`. The node starts from genesis and CometBFT sends an `InitChain` message via the ABCI to the `app`, which triggers the [`InitChainer`](#initchainer).

:::note
When starting a CometBFT instance, the genesis file is the `0` height and the state within the genesis file is committed at block height `1`. When querying the state of the node, querying block height 0 will return an error.
::: 

## Core Application File

In general, the core of the state-machine is defined in a file called `app.go`. This file mainly contains the **type definition of the application** and functions to **create and initialize it**.

### Type Definition of the Application

The first thing defined in `app.go` is the `type` of the application. It is generally comprised of the following parts:

* **A reference to [`baseapp`](../core/00-baseapp.md).** The custom application defined in `app.go` is an extension of `baseapp`. When a transaction is relayed by CometBFT to the application, `app` uses `baseapp`'s methods to route them to the appropriate module. `baseapp` implements most of the core logic for the application, including all the [ABCI methods](https://docs.cometbft.com/v0.37/spec/abci/) and the [routing logic](../core/00-baseapp.md#routing).
* **A list of store keys**. The [store](../core/04-store.md), which contains the entire state, is implemented as a [`multistore`](../core/04-store.md#multistore) (i.e. a store of stores) in the Cosmos SDK. Each module uses one or multiple stores in the multistore to persist their part of the state. These stores can be accessed with specific keys that are declared in the `app` type. These keys, along with the `keepers`, are at the heart of the [object-capabilities model](../core/10-ocap.md) of the Cosmos SDK.
* **A list of module's `keeper`s.** Each module defines an abstraction called [`keeper`](../building-modules/06-keeper.md), which handles reads and writes for this module's store(s). The `keeper`'s methods of one module can be called from other modules (if authorized), which is why they are declared in the application's type and exported as interfaces to other modules so that the latter can only access the authorized functions.
* **A reference to an [`appCodec`](../core/05-encoding.md).** The application's `appCodec` is used to serialize and deserialize data structures in order to store them, as stores can only persist `[]bytes`. The default codec is [Protocol Buffers](../core/05-encoding.md).
* **A reference to a [`legacyAmino`](../core/05-encoding.md) codec.** Some parts of the Cosmos SDK have not been migrated to use the `appCodec` above, and are still hardcoded to use Amino. Other parts explicitly use Amino for backwards compatibility. For these reasons, the application still holds a reference to the legacy Amino codec. Please note that the Amino codec will be removed from the SDK in the upcoming releases.
* **A reference to a [module manager](../building-modules/01-module-manager.md#manager)** and a [basic module manager](../building-modules/01-module-manager.md#basicmanager). The module manager is an object that contains a list of the application's modules. It facilitates operations related to these modules, like registering their [`Msg` service](../core/00-baseapp.md#msg-services) and [gRPC `Query` service](../core/00-baseapp.md#grpc-query-services), or setting the order of execution between modules for various functions like [`InitChainer`](#initchainer), [`BeginBlocker` and `EndBlocker`](#beginblocker-and-endblocker).

See an example of application type definition from `simapp`, the Cosmos SDK's own app used for demo and testing purposes:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app.go#L161-L203
```

### Constructor Function

Also defined in `app.go` is the constructor function, which constructs a new application of the type defined in the preceding section. The function must fulfill the `AppCreator` signature in order to be used in the [`start` command](../core/03-node.md#start-command) of the application's daemon command.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/server/types/app.go#L64-L66
```

Here are the main actions performed by this function:

* Instantiate a new [`codec`](../core/05-encoding.md) and initialize the `codec` of each of the application's modules using the [basic manager](../building-modules/01-module-manager.md#basicmanager).
* Instantiate a new application with a reference to a `baseapp` instance, a codec, and all the appropriate store keys.
* Instantiate all the [`keeper`](#keeper) objects defined in the application's `type` using the `NewKeeper` function of each of the application's modules. Note that keepers must be instantiated in the correct order, as the `NewKeeper` of one module might require a reference to another module's `keeper`.
* Instantiate the application's [module manager](../building-modules/01-module-manager.md#manager) with the [`AppModule`](#application-module-interface) object of each of the application's modules.
* With the module manager, initialize the application's [`Msg` services](../core/00-baseapp.md#msg-services), [gRPC `Query` services](../core/00-baseapp.md#grpc-query-services), [legacy `Msg` routes](../core/00-baseapp.md#routing), and [legacy query routes](../core/00-baseapp.md#query-routing). When a transaction is relayed to the application by CometBFT via the ABCI, it is routed to the appropriate module's [`Msg` service](#msg-services) using the routes defined here. Likewise, when a gRPC query request is received by the application, it is routed to the appropriate module's [`gRPC query service`](#grpc-query-services) using the gRPC routes defined here. The Cosmos SDK still supports legacy `Msg`s and legacy CometBFT queries, which are routed using the legacy `Msg` routes and the legacy query routes, respectively.
* With the module manager, register the [application's modules' invariants](../building-modules/07-invariants.md). Invariants are variables (e.g. total supply of a token) that are evaluated at the end of each block. The process of checking invariants is done via a special module called the [`InvariantsRegistry`](../building-modules/07-invariants.md#invariant-registry). The value of the invariant should be equal to a predicted value defined in the module. Should the value be different than the predicted one, special logic defined in the invariant registry is triggered (usually the chain is halted). This is useful to make sure that no critical bug goes unnoticed, producing long-lasting effects that are hard to fix.
* With the module manager, set the order of execution between the `InitGenesis`, `BeginBlocker`, and `EndBlocker` functions of each of the [application's modules](#application-module-interface). Note that not all modules implement these functions.
* Set the remaining application parameters:
    * [`InitChainer`](#initchainer): used to initialize the application when it is first started.
    * [`BeginBlocker`, `EndBlocker`](#beginblocker-and-endlbocker): called at the beginning and at the end of every block.
    * [`anteHandler`](../core/00-baseapp.md#antehandler): used to handle fees and signature verification.
* Mount the stores.
* Return the application.

Note that the constructor function only creates an instance of the app, while the actual state is either carried over from the `~/.app/data` folder if the node is restarted, or generated from the genesis file if the node is started for the first time.

See an example of application constructor from `simapp`:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app.go#L214-L522
```

### InitChainer

The `InitChainer` is a function that initializes the state of the application from a genesis file (i.e. token balances of genesis accounts). It is called when the application receives the `InitChain` message from the CometBFT engine, which happens when the node is started at `appBlockHeight == 0` (i.e. on genesis). The application must set the `InitChainer` in its [constructor](#constructor-function) via the [`SetInitChainer`](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/baseapp#BaseApp.SetInitChainer) method.

In general, the `InitChainer` is mostly composed of the [`InitGenesis`](../building-modules/08-genesis.md#initgenesis) function of each of the application's modules. This is done by calling the `InitGenesis` function of the module manager, which in turn calls the `InitGenesis` function of each of the modules it contains. Note that the order in which the modules' `InitGenesis` functions must be called has to be set in the module manager using the [module manager's](../building-modules/01-module-manager.md) `SetOrderInitGenesis` method. This is done in the [application's constructor](#application-constructor), and the `SetOrderInitGenesis` has to be called before the `SetInitChainer`.

See an example of an `InitChainer` from `simapp`:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app.go#L569-L577
```

### BeginBlocker and EndBlocker

The Cosmos SDK offers developers the possibility to implement automatic execution of code as part of their application. This is implemented through two functions called `BeginBlocker` and `EndBlocker`. They are called when the application receives the `BeginBlock` and `EndBlock` messages from the CometBFT engine, which happens respectively at the beginning and at the end of each block. The application must set the `BeginBlocker` and `EndBlocker` in its [constructor](#constructor-function) via the [`SetBeginBlocker`](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/baseapp#BaseApp.SetBeginBlocker) and [`SetEndBlocker`](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/baseapp#BaseApp.SetEndBlocker) methods.

In general, the `BeginBlocker` and `EndBlocker` functions are mostly composed of the [`BeginBlock` and `EndBlock`](../building-modules/05-beginblock-endblock.md) functions of each of the application's modules. This is done by calling the `BeginBlock` and `EndBlock` functions of the module manager, which in turn calls the `BeginBlock` and `EndBlock` functions of each of the modules it contains. Note that the order in which the modules' `BeginBlock` and `EndBlock` functions must be called has to be set in the module manager using the `SetOrderBeginBlockers` and `SetOrderEndBlockers` methods, respectively. This is done via the [module manager](../building-modules/01-module-manager.md) in the [application's constructor](#application-constructor), and the `SetOrderBeginBlockers` and `SetOrderEndBlockers` methods have to be called before the `SetBeginBlocker` and `SetEndBlocker` functions.

As a sidenote, it is important to remember that application-specific blockchains are deterministic. Developers must be careful not to introduce non-determinism in `BeginBlocker` or `EndBlocker`, and must also be careful not to make them too computationally expensive, as [gas](./04-gas-fees.md) does not constrain the cost of `BeginBlocker` and `EndBlocker` execution.

See an example of `BeginBlocker` and `EndBlocker` functions from `simapp`

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app.go#L555-L563
```

### Register Codec

The `EncodingConfig` structure is the last important part of the `app.go` file. The goal of this structure is to define the codecs that will be used throughout the app.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/params/encoding.go#L9-L16
```

Here are descriptions of what each of the four fields means:

* `InterfaceRegistry`: The `InterfaceRegistry` is used by the Protobuf codec to handle interfaces that are encoded and decoded (we also say "unpacked") using [`google.protobuf.Any`](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto). `Any` could be thought as a struct that contains a `type_url` (name of a concrete type implementing the interface) and a `value` (its encoded bytes). `InterfaceRegistry` provides a mechanism for registering interfaces and implementations that can be safely unpacked from `Any`. Each application module implements the `RegisterInterfaces` method that can be used to register the module's own interfaces and implementations.
    * You can read more about `Any` in [ADR-019](../architecture/adr-019-protobuf-state-encoding.md).
    * To go more into details, the Cosmos SDK uses an implementation of the Protobuf specification called [`gogoprotobuf`](https://github.com/cosmos/gogoproto). By default, the [gogo protobuf implementation of `Any`](https://pkg.go.dev/github.com/cosmos/gogoproto/types) uses [global type registration](https://github.com/cosmos/gogoproto/blob/master/proto/properties.go#L540) to decode values packed in `Any` into concrete Go types. This introduces a vulnerability where any malicious module in the dependency tree could register a type with the global protobuf registry and cause it to be loaded and unmarshaled by a transaction that referenced it in the `type_url` field. For more information, please refer to [ADR-019](../architecture/adr-019-protobuf-state-encoding.md).
* `Codec`: The default codec used throughout the Cosmos SDK. It is composed of a `BinaryCodec` used to encode and decode state, and a `JSONCodec` used to output data to the users (for example, in the [CLI](#cli)). By default, the SDK uses Protobuf as `Codec`.
* `TxConfig`: `TxConfig` defines an interface a client can utilize to generate an application-defined concrete transaction type. Currently, the SDK handles two transaction types: `SIGN_MODE_DIRECT` (which uses Protobuf binary as over-the-wire encoding) and `SIGN_MODE_LEGACY_AMINO_JSON` (which depends on Amino). Read more about transactions [here](../core/01-transactions.md).
* `Amino`: Some legacy parts of the Cosmos SDK still use Amino for backwards-compatibility. Each module exposes a `RegisterLegacyAmino` method to register the module's specific types within Amino. This `Amino` codec should not be used by app developers anymore, and will be removed in future releases.

An application should create its own encoding config.
See an example of a `simappparams.EncodingConfig` from `simapp`:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/simapp/app.go#L731-L738
```

## Modules

[Modules](../building-modules/01-intro.md) are the heart and soul of Cosmos SDK applications. They can be considered as state-machines nested within the state-machine. When a transaction is relayed from the underlying CometBFT engine via the ABCI to the application, it is routed by [`baseapp`](../core/00-baseapp.md) to the appropriate module in order to be processed. This paradigm enables developers to easily build complex state-machines, as most of the modules they need often already exist. **For developers, most of the work involved in building a Cosmos SDK application revolves around building custom modules required by their application that do not exist yet, and integrating them with modules that do already exist into one coherent application**. In the application directory, the standard practice is to store modules in the `x/` folder (not to be confused with the Cosmos SDK's `x/` folder, which contains already-built modules).

### Application Module Interface

Modules must implement [interfaces](../building-modules/01-module-manager.md#application-module-interfaces) defined in the Cosmos SDK, [`AppModuleBasic`](../building-modules/01-module-manager.md#appmodulebasic) and [`AppModule`](../building-modules/01-module-manager.md#appmodule). The former implements basic non-dependent elements of the module, such as the `codec`, while the latter handles the bulk of the module methods (including methods that require references to other modules' `keeper`s). Both the `AppModule` and `AppModuleBasic` types are, by convention, defined in a file called `module.go`.

`AppModule` exposes a collection of useful methods on the module that facilitates the composition of modules into a coherent application. These methods are called from the [`module manager`](../building-modules/01-module-manager.md#manager), which manages the application's collection of modules.

### `Msg` Services

Each application module defines two [Protobuf services](https://developers.google.com/protocol-buffers/docs/proto#services): one `Msg` service to handle messages, and one gRPC `Query` service to handle queries. If we consider the module as a state-machine, then a `Msg` service is a set of state transition RPC methods.
Each Protobuf `Msg` service method is 1:1 related to a Protobuf request type, which must implement `sdk.Msg` interface.
Note that `sdk.Msg`s are bundled in [transactions](../core/01-transactions.md), and each transaction contains one or multiple messages.

When a valid block of transactions is received by the full-node, CometBFT relays each one to the application via [`DeliverTx`](https://docs.cometbft.com/v0.37/spec/abci/abci++_app_requirements#specifics-of-responsedelivertx). Then, the application handles the transaction:

1. Upon receiving the transaction, the application first unmarshalls it from `[]byte`.
2. Then, it verifies a few things about the transaction like [fee payment and signatures](./04-gas-fees.md#antehandler) before extracting the `Msg`(s) contained in the transaction.
3. `sdk.Msg`s are encoded using Protobuf [`Any`s](#register-codec). By analyzing each `Any`'s `type_url`, baseapp's `msgServiceRouter` routes the `sdk.Msg` to the corresponding module's `Msg` service.
4. If the message is successfully processed, the state is updated.

For more details, see [transaction lifecycle](./01-tx-lifecycle.md).

Module developers create custom `Msg` services when they build their own module. The general practice is to define the `Msg` Protobuf service in a `tx.proto` file. For example, the `x/bank` module defines a service with two methods to transfer tokens:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/bank/v1beta1/tx.proto#L13-L36
```

Service methods use `keeper` in order to update the module state.

Each module should also implement the `RegisterServices` method as part of the [`AppModule` interface](#application-module-interface). This method should call the `RegisterMsgServer` function provided by the generated Protobuf code.

### gRPC `Query` Services

gRPC `Query` services allow users to query the state using [gRPC](https://grpc.io). They are enabled by default, and can be configured under the `grpc.enable` and `grpc.address` fields inside [`app.toml`](../run-node/02-interact-node.md#configuring-the-node-using-apptoml).

gRPC `Query` services are defined in the module's Protobuf definition files, specifically inside `query.proto`. The `query.proto` definition file exposes a single `Query` [Protobuf service](https://developers.google.com/protocol-buffers/docs/proto#services). Each gRPC query endpoint corresponds to a service method, starting with the `rpc` keyword, inside the `Query` service.

Protobuf generates a `QueryServer` interface for each module, containing all the service methods. A module's [`keeper`](#keeper) then needs to implement this `QueryServer` interface, by providing the concrete implementation of each service method. This concrete implementation is the handler of the corresponding gRPC query endpoint.

Finally, each module should also implement the `RegisterServices` method as part of the [`AppModule` interface](#application-module-interface). This method should call the `RegisterQueryServer` function provided by the generated Protobuf code.

### Keeper

[`Keepers`](../building-modules/06-keeper.md) are the gatekeepers of their module's store(s). To read or write in a module's store, it is mandatory to go through one of its `keeper`'s methods. This is ensured by the [object-capabilities](../core/10-ocap.md) model of the Cosmos SDK. Only objects that hold the key to a store can access it, and only the module's `keeper` should hold the key(s) to the module's store(s).

`Keepers` are generally defined in a file called `keeper.go`. It contains the `keeper`'s type definition and methods.

The `keeper` type definition generally consists of the following:

* **Key(s)** to the module's store(s) in the multistore.
* Reference to **other module's `keepers`**. Only needed if the `keeper` needs to access other module's store(s) (either to read or write from them).
* A reference to the application's **codec**. The `keeper` needs it to marshal structs before storing them, or to unmarshal them when it retrieves them, because stores only accept `[]bytes` as value.

Along with the type definition, the next important component of the `keeper.go` file is the `keeper`'s constructor function, `NewKeeper`. This function instantiates a new `keeper` of the type defined above with a `codec`, stores `keys` and potentially references other modules' `keeper`s as parameters. The `NewKeeper` function is called from the [application's constructor](#constructor-function). The rest of the file defines the `keeper`'s methods, which are primarily getters and setters.

### Command-Line, gRPC Services and REST Interfaces

Each module defines command-line commands, gRPC services, and REST routes to be exposed to the end-user via the [application's interfaces](#application-interfaces). This enables end-users to create messages of the types defined in the module, or to query the subset of the state managed by the module.

#### CLI

Generally, the [commands related to a module](../building-modules/09-module-interfaces.md#cli) are defined in a folder called `client/cli` in the module's folder. The CLI divides commands into two categories, transactions and queries, defined in `client/cli/tx.go` and `client/cli/query.go`, respectively. Both commands are built on top of the [Cobra Library](https://github.com/spf13/cobra):

* Transactions commands let users generate new transactions so that they can be included in a block and eventually update the state. One command should be created for each [message type](#message-types) defined in the module. The command calls the constructor of the message with the parameters provided by the end-user, and wraps it into a transaction. The Cosmos SDK handles signing and the addition of other transaction metadata.
* Queries let users query the subset of the state defined by the module. Query commands forward queries to the [application's query router](../core/00-baseapp.md#query-routing), which routes them to the appropriate [querier](#querier) the `queryRoute` parameter supplied.

#### gRPC

[gRPC](https://grpc.io) is a modern open-source high performance RPC framework that has support in multiple languages. It is the recommended way for external clients (such as wallets, browsers and other backend services) to interact with a node.

Each module can expose gRPC endpoints called [service methods](https://grpc.io/docs/what-is-grpc/core-concepts/#service-definition), which are defined in the [module's Protobuf `query.proto` file](#grpc-query-services). A service method is defined by its name, input arguments, and output response. The module then needs to perform the following actions:

* Define a `RegisterGRPCGatewayRoutes` method on `AppModuleBasic` to wire the client gRPC requests to the correct handler inside the module.
* For each service method, define a corresponding handler. The handler implements the core logic necessary to serve the gRPC request, and is located in the `keeper/grpc_query.go` file.

#### gRPC-gateway REST Endpoints

Some external clients may not wish to use gRPC. In this case, the Cosmos SDK provides a gRPC gateway service, which exposes each gRPC service as a corresponding REST endpoint. Please refer to the [grpc-gateway](https://grpc-ecosystem.github.io/grpc-gateway/) documentation to learn more.

The REST endpoints are defined in the Protobuf files, along with the gRPC services, using Protobuf annotations. Modules that want to expose REST queries should add `google.api.http` annotations to their `rpc` methods. By default, all REST endpoints defined in the SDK have a URL starting with the `/cosmos/` prefix.

The Cosmos SDK also provides a development endpoint to generate [Swagger](https://swagger.io/) definition files for these REST endpoints. This endpoint can be enabled inside the [`app.toml`](../run-node/01-run-node.md#configuring-the-node-using-apptoml) config file, under the `api.swagger` key.

## Application Interface

[Interfaces](#command-line-grpc-services-and-rest-interfaces) let end-users interact with full-node clients. This means querying data from the full-node or creating and sending new transactions to be relayed by the full-node and eventually included in a block.

The main interface is the [Command-Line Interface](../core/07-cli.md). The CLI of a Cosmos SDK application is built by aggregating [CLI commands](#cli) defined in each of the modules used by the application. The CLI of an application is the same as the daemon (e.g. `appd`), and is defined in a file called `appd/main.go`. The file contains the following:

* **A `main()` function**, which is executed to build the `appd` interface client. This function prepares each command and adds them to the `rootCmd` before building them. At the root of `appd`, the function adds generic commands like `status`, `keys`, and `config`, query commands, tx commands, and `rest-server`.
* **Query commands**, which are added by calling the `queryCmd` function. This function returns a Cobra command that contains the query commands defined in each of the application's modules (passed as an array of `sdk.ModuleClients` from the `main()` function), as well as some other lower level query commands such as block or validator queries. Query command are called by using the command `appd query [query]` of the CLI.
* **Transaction commands**, which are added by calling the `txCmd` function. Similar to `queryCmd`, the function returns a Cobra command that contains the tx commands defined in each of the application's modules, as well as lower level tx commands like transaction signing or broadcasting. Tx commands are called by using the command `appd tx [tx]` of the CLI.

See an example of an application's main command-line file from the [Cosmos Hub](https://github.com/cosmos/gaia).

```go reference
https://github.com/cosmos/gaia/blob/26ae7c2/cmd/gaiad/cmd/root.go#L39-L80
```

## Dependencies and Makefile

This section is optional, as developers are free to choose their dependency manager and project building method. That said, the current most used framework for versioning control is [`go.mod`](https://github.com/golang/go/wiki/Modules). It ensures each of the libraries used throughout the application are imported with the correct version.

The following is the `go.mod` of the [Cosmos Hub](https://github.com/cosmos/gaia), provided as an example.

```go reference
https://github.com/cosmos/gaia/blob/26ae7c2/go.mod#L1-L28
```

For building the application, a [Makefile](https://en.wikipedia.org/wiki/Makefile) is generally used. The Makefile primarily ensures that the `go.mod` is run before building the two entrypoints to the application, [`appd`](#node-client) and [`appd`](#application-interface).

Here is an example of the [Cosmos Hub Makefile](https://github.com/cosmos/gaia/blob/main/Makefile).
