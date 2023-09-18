---
sidebar_position: 1
---

# Messages and Queries

:::note Synopsis
`Msg`s and `Queries` are the two primary objects handled by modules. Most of the core components defined in a module, like `Msg` services, `keeper`s and `Query` services, exist to process `message`s and `queries`.
:::

:::note Pre-requisite Readings

* [Introduction to Cosmos SDK Modules](./01-intro.md)

:::

## Messages

`Msg`s are objects whose end-goal is to trigger state-transitions. They are wrapped in [transactions](../core/01-transactions.md), which may contain one or more of them.

When a transaction is relayed from the underlying consensus engine to the Cosmos SDK application, it is first decoded by [`BaseApp`](../core/00-baseapp.md). Then, each message contained in the transaction is extracted and routed to the appropriate module via `BaseApp`'s `MsgServiceRouter` so that it can be processed by the module's [`Msg` service](./03-msg-services.md). For a more detailed explanation of the lifecycle of a transaction, click [here](../basics/01-tx-lifecycle.md).

### `Msg` Services

Defining Protobuf `Msg` services is the recommended way to handle messages. A Protobuf `Msg` service should be created for each module, typically in `tx.proto` (see more info about [conventions and naming](../core/05-encoding.md#faq)). It must have an RPC service method defined for each message in the module.

See an example of a `Msg` service definition from `x/bank` module:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/bank/v1beta1/tx.proto#L13-L36
```

Each `Msg` service method must have exactly one argument, which must implement the `sdk.Msg` interface, and a Protobuf response. The naming convention is to call the RPC argument `Msg<service-rpc-name>` and the RPC response `Msg<service-rpc-name>Response`. For example:

```protobuf
  rpc Send(MsgSend) returns (MsgSendResponse);
```

`sdk.Msg` interface is a simplified version of the Amino `LegacyMsg` interface described [below](#legacy-amino-msgs) with only `ValidateBasic()` and `GetSigners()` methods. For backwards compatibility with [Amino `LegacyMsg`s](#legacy-amino-msgs), existing `LegacyMsg` types should be used as the request parameter for `service` RPC definitions. Newer `sdk.Msg` types, which only support `service` definitions, should use canonical `Msg...` name.

The Cosmos SDK uses Protobuf definitions to generate client and server code:

* `MsgServer` interface defines the server API for the `Msg` service and its implementation is described as part of the [`Msg` services](./03-msg-services.md) documentation.
* Structures are generated for all RPC request and response types.

A `RegisterMsgServer` method is also generated and should be used to register the module's `MsgServer` implementation in `RegisterServices` method from the [`AppModule` interface](./01-module-manager.md#appmodule).

In order for clients (CLI and grpc-gateway) to have these URLs registered, the Cosmos SDK provides the function `RegisterMsgServiceDesc(registry codectypes.InterfaceRegistry, sd *grpc.ServiceDesc)` that should be called inside module's [`RegisterInterfaces`](01-module-manager.md#appmodulebasic) method, using the proto-generated `&_Msg_serviceDesc` as `*grpc.ServiceDesc` argument.

### Legacy Amino `LegacyMsg`s

The following way of defining messages is deprecated and using [`Msg` services](#msg-services) is preferred.

Amino `LegacyMsg`s can be defined as protobuf messages. The messages definition usually includes a list of parameters needed to process the message that will be provided by end-users when they want to create a new transaction containing said message.

A `LegacyMsg` is typically accompanied by a standard constructor function, that is called from one of the [module's interface](./09-module-interfaces.md). `message`s also need to implement the `sdk.Msg` interface:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/types/tx_msg.go#L14-L26
```

It extends `proto.Message` and contains the following methods:

* `Route() string`: Name of the route for this message. Typically all `message`s in a module have the same route, which is most often the module's name.
* `Type() string`: Type of the message, used primarily in [events](../core/08-events.md). This should return a message-specific `string`, typically the denomination of the message itself.
* [`ValidateBasic() error`](../basics/01-tx-lifecycle.md#ValidateBasic).
* `GetSignBytes() []byte`: Return the canonical byte representation of the message. Used to generate a signature.
* `GetSigners() []AccAddress`: Return the list of signers. The Cosmos SDK will make sure that each `message` contained in a transaction is signed by all the signers listed in the list returned by this method.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/auth/migrations/legacytx/stdsign.go#L20-L36
```

See an example implementation of a `message` from the `gov` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/gov/types/v1/msgs.go#L121-L153
```

## Queries

A `query` is a request for information made by end-users of applications through an interface and processed by a full-node. A `query` is received by a full-node through its consensus engine and relayed to the application via the ABCI. It is then routed to the appropriate module via `BaseApp`'s `QueryRouter` so that it can be processed by the module's query service (./04-query-services.md). For a deeper look at the lifecycle of a `query`, click [here](../basics/02-query-lifecycle.md).

### gRPC Queries

Queries should be defined using [Protobuf services](https://developers.google.com/protocol-buffers/docs/proto#services). A `Query` service should be created per module in `query.proto`. This service lists endpoints starting with `rpc`.

Here's an example of such a `Query` service definition:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/auth/v1beta1/query.proto#L14-L89
```

As `proto.Message`s, generated `Response` types implement by default `String()` method of [`fmt.Stringer`](https://pkg.go.dev/fmt#Stringer).

A `RegisterQueryServer` method is also generated and should be used to register the module's query server in the `RegisterServices` method from the [`AppModule` interface](./01-module-manager.md#appmodule).

### Legacy Queries

Before the introduction of Protobuf and gRPC in the Cosmos SDK, there was usually no specific `query` object defined by module developers, contrary to `message`s. Instead, the Cosmos SDK took the simpler approach of using a simple `path` to define each `query`. The `path` contains the `query` type and all the arguments needed to process it. For most module queries, the `path` should look like the following:

```text
queryCategory/queryRoute/queryType/arg1/arg2/...
```

where:

* `queryCategory` is the category of the `query`, typically `custom` for module queries. It is used to differentiate between different kinds of queries within `BaseApp`'s [`Query` method](../core/00-baseapp.md#query).
* `queryRoute` is used by `BaseApp`'s [`queryRouter`](../core/00-baseapp.md#query-routing) to map the `query` to its module. Usually, `queryRoute` should be the name of the module.
* `queryType` is used by the module's [`querier`](./04-query-services.md#legacy-queriers) to map the `query` to the appropriate `querier function` within the module.
* `args` are the actual arguments needed to process the `query`. They are filled out by the end-user. Note that for bigger queries, you might prefer passing arguments in the `Data` field of the request `req` instead of the `path`.

The `path` for each `query` must be defined by the module developer in the module's [command-line interface file](./09-module-interfaces.md#query-commands).Overall, there are 3 mains components module developers need to implement in order to make the subset of the state defined by their module queryable:

* A [`querier`](./04-query-services.md#legacy-queriers), to process the `query` once it has been [routed to the module](../core/00-baseapp.md#query-routing).
* [Query commands](./09-module-interfaces.md#query-commands) in the module's CLI file, where the `path` for each `query` is specified.
* `query` return types. Typically defined in a file `types/querier.go`, they specify the result type of each of the module's `queries`. These custom types must implement the `String()` method of [`fmt.Stringer`](https://pkg.go.dev/fmt#Stringer).

### Store Queries

Store queries query directly for store keys. They use `clientCtx.QueryABCI(req abci.RequestQuery)` to return the full `abci.ResponseQuery` with inclusion Merkle proofs.

See following examples:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/baseapp/abci.go#L881-L902
```
