---
sidebar_position: 1
---

# Module Interfaces

:::note Synopsis
This document details how to build CLI and REST interfaces for a module. Examples from various Cosmos SDK modules are included.
:::

:::note Pre-requisite Readings

* [Building Modules Intro](./00-intro.md)

:::

## CLI

One of the main interfaces for an application is the [command-line interface](../../learn/advanced/07-cli.md). This entrypoint adds commands from the application's modules enabling end-users to create [**messages**](./02-messages-and-queries.md#messages) wrapped in transactions and [**queries**](./02-messages-and-queries.md#queries). The CLI files are typically found in the module's `./client/cli` folder.

### Transaction Commands

In order to create messages that trigger state changes, end-users must create [transactions](../../learn/advanced/01-transactions.md) that wrap and deliver the messages. A transaction command creates a transaction that includes one or more messages.

Transaction commands typically have their own `tx.go` file that lives within the module's `./client/cli` folder. The commands are specified in getter functions and the name of the function should include the name of the command.

Here is an example from the `x/bank` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/bank/client/cli/tx.go#L37-L76
```

In the example, `NewSendTxCmd()` creates and returns the transaction command for a transaction that wraps and delivers `MsgSend`. `MsgSend` is the message used to send tokens from one account to another.

In general, the getter function does the following:

* **Constructs the command:** Read the [Cobra Documentation](https://pkg.go.dev/github.com/spf13/cobra) for more detailed information on how to create commands.
    * **Use:** Specifies the format of the user input required to invoke the command. In the example above, `send` is the name of the transaction command and `[from_key_or_address]`, `[to_address]`, and `[amount]` are the arguments.
    * **Args:** The number of arguments the user provides. In this case, there are exactly three: `[from_key_or_address]`, `[to_address]`, and `[amount]`.
    * **Short and Long:** Descriptions for the command. A `Short` description is expected. A `Long` description can be used to provide additional information that is displayed when a user adds the `--help` flag.
    * **RunE:** Defines a function that can return an error. This is the function that is called when the command is executed. This function encapsulates all of the logic to create a new transaction.
        * The function typically starts by getting the `clientCtx`, which can be done with `client.GetClientTxContext(cmd)`. The `clientCtx` contains information relevant to transaction handling, including information about the user. In this example, the `clientCtx` is used to retrieve the address of the sender by calling `clientCtx.GetFromAddress()`.
        * If applicable, the command's arguments are parsed. In this example, the arguments `[to_address]` and `[amount]` are both parsed.
        * A [message](./02-messages-and-queries.md) is created using the parsed arguments and information from the `clientCtx`. The constructor function of the message type is called directly. In this case, `types.NewMsgSend(fromAddr, toAddr, amount)`. Its good practice to call, if possible, the necessary [message validation methods](../building-modules/03-msg-services.md#Validation) before broadcasting the message.
        * Depending on what the user wants, the transaction is either generated offline or signed and broadcasted to the preconfigured node using `tx.GenerateOrBroadcastTxCLI(clientCtx, flags, msg)`.
* **Adds transaction flags:** All transaction commands must add a set of transaction flags. The transaction flags are used to collect additional information from the user (e.g. the amount of fees the user is willing to pay). The transaction flags are added to the constructed command using `AddTxFlagsToCmd(cmd)`.
* **Returns the command:** Finally, the transaction command is returned.

Each module can implement `NewTxCmd()`, which aggregates all of the transaction commands of the module. Here is an example from the `x/bank` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/bank/client/cli/tx.go#L20-L35
```

Each module then can also implement a `GetTxCmd()` method that simply returns `NewTxCmd()`. This allows the root command to easily aggregate all of the transaction commands for each module. Here is an example:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/bank/module.go#L84-L86
```

### Query Commands

:::warning
This section is being rewritten. Refer to [AutoCLI](https://docs.cosmos.network/main/core/autocli) while this section is being updated.
:::

## gRPC

[gRPC](https://grpc.io/) is a Remote Procedure Call (RPC) framework. RPC is the preferred way for external clients like wallets and exchanges to interact with a blockchain.

In addition to providing an ABCI query pathway, the Cosmos SDK provides a gRPC proxy server that routes gRPC query requests to ABCI query requests.

In order to do that, modules must implement the `module.HasGRPCGateway` interface on `AppModule` to wire the client gRPC requests to the correct handler inside the module.

Here's an example from the `x/auth` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/auth/module.go#L71-L76
```

## gRPC-gateway REST

Applications need to support web services that use HTTP requests (e.g. a web wallet like [Keplr](https://keplr.app)). [grpc-gateway](https://github.com/grpc-ecosystem/grpc-gateway) translates REST calls into gRPC calls, which might be useful for clients that do not use gRPC.

Modules that want to expose REST queries should add `google.api.http` annotations to their `rpc` methods, such as in the example below from the `x/auth` module:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/proto/cosmos/auth/v1beta1/query.proto#L14-L89
```

<!-- markdown-link-check-disable -->
gRPC gateway is started in-process along with the application and CometBFT. It can be enabled or disabled by setting gRPC Configuration `enable` in [`app.toml`](../run-node/01-run-node.md#configuring-the-node-using-apptoml-and-configtoml).

The Cosmos SDK provides a command for generating [Swagger](https://swagger.io/) documentation (`protoc-gen-swagger`). Setting `swagger` in [`app.toml`](../run-node/01-run-node.md#configuring-the-node-using-apptoml-and-configtoml) defines if swagger documentation should be automatically registered.
