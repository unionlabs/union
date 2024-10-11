---
sidebar_position: 1
---

# Node Client (Daemon)

:::note Synopsis
The main endpoint of a Cosmos SDK application is the daemon client, otherwise known as the full-node client. The full-node runs the state-machine, starting from a genesis file. It connects to peers running the same client in order to receive and relay transactions, block proposals and signatures. The full-node is constituted of the application, defined with the Cosmos SDK, and of a consensus engine connected to the application via the ABCI.
:::

:::note Pre-requisite Readings

* [Anatomy of an SDK application](../beginner/00-app-anatomy.md)

:::

## `main` function

The full-node client of any Cosmos SDK application is built by running a `main` function. The client is generally named by appending the `-d` suffix to the application name (e.g. `appd` for an application named `app`), and the `main` function is defined in a `./appd/cmd/main.go` file. Running this function creates an executable `appd` that comes with a set of commands. For an app named `app`, the main command is [`appd start`](#start-command), which starts the full-node.

In general, developers will implement the `main.go` function with the following structure:

* First, an [`encodingCodec`](https://docs.cosmos.network/main/learn/advanced/encoding) is instantiated for the application.
* Then, the `config` is retrieved and config parameters are set. This mainly involves setting the Bech32 prefixes for [addresses](../beginner/03-accounts.md#addresses).

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/types/config.go#L14-L29
```

* Using [cobra](https://github.com/spf13/cobra), the root command of the full-node client is created. After that, all the custom commands of the application are added using the `AddCommand()` method of `rootCmd`.
* Add default server commands to `rootCmd` using the `server.AddCommands()` method. These commands are separated from the ones added above since they are standard and defined at Cosmos SDK level. They should be shared by all Cosmos SDK-based applications. They include the most important command: the [`start` command](#start-command).
* Prepare and execute the `executor`.
  
```go reference
https://github.com/cometbft/cometbft/blob/v0.37.0/libs/cli/setup.go#L74-L78
```

See an example of `main` function from the `simapp` application, the Cosmos SDK's application for demo purposes:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/simd/main.go
```

## `start` command

The `start` command is defined in the `/server` folder of the Cosmos SDK. It is added to the root command of the full-node client in the [`main` function](#main-function) and called by the end-user to start their node:

```bash
# For an example app named "app", the following command starts the full-node.
appd start

# Using the Cosmos SDK's own simapp, the following commands start the simapp node.
simd start
```

As a reminder, the full-node is composed of three conceptual layers: the networking layer, the consensus layer and the application layer. The first two are generally bundled together in an entity called the consensus engine (CometBFT by default), while the third is the state-machine defined with the help of the Cosmos SDK. Currently, the Cosmos SDK uses CometBFT as the default consensus engine, meaning the start command is implemented to boot up a CometBFT node.

The flow of the `start` command is pretty straightforward. First, it retrieves the `config` from the `context` in order to open the `db` (a [`levelDB`](https://github.com/syndtr/goleveldb) instance by default). This `db` contains the latest known state of the application (empty if the application is started from the first time).

With the `db`, the `start` command creates a new instance of the application using an `appCreator` function:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/server/start.go#L220
```

Note that an `appCreator` is a function that fulfills the `AppCreator` signature:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/server/types/app.go#L68
```

In practice, the [constructor of the application](../beginner/00-app-anatomy.md#constructor-function) is passed as the `appCreator`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/simapp/simd/cmd/root_v2.go#L294-L308
```

Then, the instance of `app` is used to instantiate a new CometBFT node:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/server/start.go#L341-L378
```

The CometBFT node can be created with `app` because the latter satisfies the [`abci.Application` interface](https://pkg.go.dev/github.com/cometbft/cometbft/api/cometbft/abci/v1#Application) (given that `app` extends [`baseapp`](./00-baseapp.md)). As part of the `node.New` method, CometBFT makes sure that the height of the application (i.e. number of blocks since genesis) is equal to the height of the CometBFT node. The difference between these two heights should always be negative or null. If it is strictly negative, `node.New` will replay blocks until the height of the application reaches the height of the CometBFT node. Finally, if the height of the application is `0`, the CometBFT node will call [`InitChain`](./00-baseapp.md#initchain) on the application to initialize the state from the genesis file.

Once the CometBFT node is instantiated and in sync with the application, the node can be started:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/server/start.go#L350-L352
```

Upon starting, the node will bootstrap its RPC and P2P server and start dialing peers. During handshake with its peers, if the node realizes they are ahead, it will query all the blocks sequentially in order to catch up. Then, it will wait for new block proposals and block signatures from validators in order to make progress.

## Other commands
<!-- markdown-link-check-disable-next-line -->
To discover how to concretely run a node and interact with it, please refer to our [Running a Node, API and CLI](../../user/run-node/01-run-node.md#configuring-the-node-using-apptoml-and-configtoml) guide.
