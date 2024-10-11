---
sidebar_position: 1
---

# Messages and Queries

:::note Synopsis
`Msg`s and `Queries` are the two primary objects handled by modules. Most of the core components defined in a module, like `Msg` services, `keeper`s and `Query` services, exist to process `message`s and `queries`.
:::

:::note Pre-requisite Readings

* [Introduction to Cosmos SDK Modules](./00-intro.md)

:::

## Messages

`Msg`s are objects whose end-goal is to trigger state-transitions. They are wrapped in [transactions](../../learn/advanced/01-transactions.md), which may contain one or more of them.

When a transaction is relayed from the underlying consensus engine to the Cosmos SDK application, it is first decoded by [`BaseApp`](../../learn/advanced/00-baseapp.md). Then, each message contained in the transaction is extracted and routed to the appropriate module via `BaseApp`'s `MsgServiceRouter` so that it can be processed by the module's [`Msg` service](./03-msg-services.md). For a more detailed explanation of the lifecycle of a transaction, click [here](../../learn/beginner/01-tx-lifecycle.md).

### `Msg` Services

Defining Protobuf `Msg` services is the recommended way to handle messages. A Protobuf `Msg` service should be created for each module, typically in `tx.proto` (see more info about [conventions and naming](../../learn/advanced/05-encoding.md#faq)). It must have an RPC service method defined for each message in the module.


Each `Msg` service method must have exactly one argument, which must implement the `transaction.Msg` interface, and a Protobuf response. The naming convention is to call the RPC argument `Msg<service-rpc-name>` and the RPC response `Msg<service-rpc-name>Response`. For example:

```protobuf
  rpc Send(MsgSend) returns (MsgSendResponse);
```

See an example of a `Msg` service definition from `x/bank` module:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/28fa3b8/x/bank/proto/cosmos/bank/v1beta1/tx.proto#L13-L41
```

### `transaction.Msg` Interface

`transaction.Msg` is an alias of `proto.Message`. 

```go reference 
https://github.com/cosmos/cosmos-sdk/blob/main/core/transaction/transaction.go#L8
```

To attach a `ValidateBasic()` method to a message, then you must add methods to the type adhereing to the `HasValidateBasic`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/9c1e8b247cd47b5d3decda6e86fbc3bc996ee5d7/types/tx_msg.go#L84-L88
```

In 0.50+ signers from the `GetSigners()` call is automated via a protobuf annotation. 

Read more about the signer field [here](./05-protobuf-annotations.md).

```protobuf reference 
https://github.com/cosmos/cosmos-sdk/blob/e6848d99b55a65d014375b295bdd7f9641aac95e/proto/cosmos/bank/v1beta1/tx.proto#L40
```

If there is a need for custom signers then there is an alternative path which can be taken. A function which returns `signing.CustomGetSigner` for a specific message can be defined. 

```go
func ProvideBankSendTransactionGetSigners() signing.CustomGetSigner {

			// Extract the signer from the signature.
			signer, err := coretypes.LatestSigner(Tx).Sender(ethTx)
      if err != nil {
				return nil, err
			}

			// Return the signer in the required format.
			return [][]byte{signer.Bytes()}, nil
}
```

When using dependency injection (depinject) this can be provided to the application via the provide method.

```go
depinject.Provide(banktypes.ProvideBankSendTransactionGetSigners)
```

The Cosmos SDK uses Protobuf definitions to generate client and server code:

* `MsgServer` interface defines the server API for the `Msg` service and its implementation is described as part of the [`Msg` services](./03-msg-services.md) documentation.
* Structures are generated for all RPC request and response types.

A `RegisterMsgServer` method is also generated and should be used to register the module's `MsgServer` implementation in `RegisterServices` method from the [`AppModule` interface](./01-module-manager.md#appmodule).

In order for clients (CLI and grpc-gateway) to have these URLs registered, the Cosmos SDK provides the function `RegisterMsgServiceDesc(registry codectypes.InterfaceRegistry, sd *grpc.ServiceDesc)` that should be called inside module's [`RegisterInterfaces`](01-module-manager.md#hasregisterinterfaces) method, using the proto-generated `&_Msg_serviceDesc` as `*grpc.ServiceDesc` argument.


## Queries

A `query` is a request for information made by end-users of applications through an interface and processed by a full-node. A `query` is received by a full-node through its consensus engine and relayed to the application via the ABCI. It is then routed to the appropriate module via `BaseApp`'s `QueryRouter` so that it can be processed by the module's query service (./04-query-services.md). For a deeper look at the lifecycle of a `query`, click [here](../../learn/beginner/02-query-lifecycle.md).

### gRPC Queries

Queries should be defined using [Protobuf services](https://protobuf.dev/programming-guides/proto2/). A `Query` service should be created per module in `query.proto`. This service lists endpoints starting with `rpc`.

Here's an example of such a `Query` service definition:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/proto/cosmos/auth/v1beta1/query.proto#L14-L89
```

As `proto.Message`s, generated `Response` types implement by default `String()` method of [`fmt.Stringer`](https://pkg.go.dev/fmt#Stringer).

A `RegisterQueryServer` method is also generated and should be used to register the module's query server in the `RegisterServices` method from the [`AppModule` interface](./01-module-manager.md#appmodule).


### Store Queries

Store queries query directly for store keys. They use `clientCtx.QueryABCI(req abci.QueryRequest)` to return the full `abci.QueryResponse` with inclusion Merkle proofs.

See following examples:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/baseapp/abci.go#L864-L894
```
