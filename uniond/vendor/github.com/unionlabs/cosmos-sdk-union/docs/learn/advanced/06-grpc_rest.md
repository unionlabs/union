---
sidebar_position: 1
---

# gRPC, REST, and CometBFT Endpoints

:::note Synopsis
This document presents an overview of all the endpoints a node exposes: gRPC, REST as well as some other endpoints.
:::

## An Overview of All Endpoints

Each node exposes the following endpoints for users to interact with a node, each endpoint is served on a different port. Details on how to configure each endpoint is provided in the endpoint's own section.

* the gRPC server (default port: `9090`),
* the REST server (default port: `1317`),
* the CometBFT RPC endpoint (default port: `26657`).

:::tip
The node also exposes some other endpoints, such as the CometBFT P2P endpoint, or the [Prometheus endpoint](https://docs.cometbft.com/v1.0/explanation/core/metrics), which are not directly related to the Cosmos SDK. Please refer to the [CometBFT Configuration Manual](https://docs.cometbft.com/v1.0/references/config/) for more information about how to configure these CometBFT endpoints.
:::

:::note
All endpoints are defaulted to localhost and must be modified to be exposed to the public internet.
:::

## gRPC Server

In the Cosmos SDK, Protobuf is the main [encoding](https://docs.cosmos.network/main/learn/advanced/encoding) library. This brings a wide range of Protobuf-based tools that can be plugged into the Cosmos SDK. One such tool is [gRPC](https://grpc.io), a modern open-source high performance RPC framework that has decent client support in several languages.

Each module exposes a [Protobuf `Query` service](../../build/building-modules/02-messages-and-queries.md#queries) that defines state queries. The `Query` services and a transaction service used to broadcast transactions are hooked up to the gRPC server via the following function inside the application:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/server/types/app.go#L46-L48
```

Note: It is not possible to expose any [Protobuf `Msg` service](../../build/building-modules/02-messages-and-queries.md#messages) endpoints via gRPC. Transactions must be generated and signed using the CLI or programmatically before they can be broadcasted using gRPC. See [Generating, Signing, and Broadcasting Transactions](https://docs.cosmos.network/main/user/run-node/txs) for more information.

The `grpc.Server` is a concrete gRPC server, which spawns and serves all gRPC query requests and a broadcast transaction request. This server can be configured inside `~/.simapp/config/app.toml`:

* `grpc.enable = true|false` field defines if the gRPC server should be enabled. Defaults to `true`.
* `grpc.address = {string}` field defines the `ip:port` the server should bind to. Defaults to `localhost:9090`.

:::tip
`~/.simapp` is the directory where the node's configuration and databases are stored. By default, it's set to `~/.{app_name}`.
:::
<!-- markdown-link-check-disable-next-line -->
Once the gRPC server is started, you can send requests to it using a gRPC client. Some examples are given in our [Interact with the Node](../../user/run-node/01-run-node.md#configuring-the-node-using-apptoml-and-configtoml) tutorial.

An overview of all available gRPC endpoints shipped with the Cosmos SDK is [Protobuf documentation](https://buf.build/cosmos/cosmos-sdk).

## REST Server

Cosmos SDK supports REST routes via gRPC-gateway.

All routes are configured under the following fields in `~/.simapp/config/app.toml`:

* `api.enable = true|false` field defines if the REST server should be enabled. Defaults to `false`.
* `api.address = {string}` field defines the `ip:port` the server should bind to. Defaults to `tcp://localhost:1317`.
* some additional API configuration options are defined in `~/.simapp/config/app.toml`, along with comments, please refer to that file directly.

### gRPC-gateway REST Routes

If, for various reasons, you cannot use gRPC (for example, you are building a web application, and browsers don't support HTTP2 on which gRPC is built), then the Cosmos SDK offers REST routes via gRPC-gateway.

[gRPC-gateway](https://grpc-ecosystem.github.io/grpc-gateway/) is a tool to expose gRPC endpoints as REST endpoints. For each gRPC endpoint defined in a Protobuf `Query` service, the Cosmos SDK offers a REST equivalent. For instance, querying a balance could be done via the `/cosmos.bank.v1beta1.QueryAllBalances` gRPC endpoint, or alternatively via the gRPC-gateway `"/cosmos/bank/v1beta1/balances/{address}"` REST endpoint: both will return the same result. For each RPC method defined in a Protobuf `Query` service, the corresponding REST endpoint is defined as an option:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/proto/cosmos/bank/v1beta1/query.proto#L23-L30
```

For application developers, gRPC-gateway REST routes needs to be wired up to the REST server, this is done by calling the `RegisterGRPCGatewayRoutes` function on the ModuleManager.

### Swagger

A [Swagger](https://swagger.io/) (or OpenAPIv2) specification file is exposed under the `/swagger` route on the API server. Swagger is an open specification describing the API endpoints a server serves, including description, input arguments, return types and much more about each endpoint.

Enabling the `/swagger` endpoint is configurable inside `~/.simapp/config/app.toml` via the `api.swagger` field, which is set to false by default.

For application developers, you may want to generate your own Swagger definitions based on your custom modules.
The Cosmos SDK's [Swagger generation script](https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/scripts/protoc-swagger-gen.sh) is a good place to start.

## CometBFT RPC

Independently from the Cosmos SDK, CometBFT also exposes a RPC server. This RPC server can be configured by tuning parameters under the `rpc` table in the `~/.simapp/config/config.toml`, the default listening address is `tcp://localhost:26657`. An OpenAPI specification of all CometBFT RPC endpoints is available [here](https://docs.cometbft.com/v1.0/rpc/).

Some CometBFT RPC endpoints are directly related to the Cosmos SDK:

* `/abci_query`: this endpoint will query the application for state. As the `path` parameter, you can send the following strings:
    * any Protobuf fully-qualified service method, such as `/cosmos.bank.v1beta1.Query/AllBalances`. The `data` field should then include the method's request parameter(s) encoded as bytes using Protobuf.
    * `/app/simulate`: this will simulate a transaction, and return some information such as gas used.
    * `/app/version`: this will return the application's version.
    * `/store/{storeName}/key`: this will directly query the named store for data associated with the key represented in the `data` parameter.
    * `/store/{storeName}/subspace`: this will directly query the named store for key/value pairs in which the key has the value of the `data` parameter as a prefix.
    * `/p2p/filter/addr/{port}`: this will return a filtered list of the node's P2P peers by address port.
    * `/p2p/filter/id/{id}`: this will return a filtered list of the node's P2P peers by ID.
* `/broadcast_tx_{sync,async,commit}`: these 3 endpoints will broadcast a transaction to other peers. CLI, gRPC and REST expose [a way to broadcast transactions](./01-transactions.md#broadcasting-the-transaction), but they all use these 3 CometBFT RPCs under the hood.

## Comparison Table

| Name           | Advantages                                                                                                                                                            | Disadvantages                                                                                                 |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| gRPC           | - can use code-generated stubs in various languages <br /> - supports streaming and bidirectional communication (HTTP2) <br /> - small wire binary sizes, faster transmission | - based on HTTP2, not available in browsers <br /> - learning curve (mostly due to Protobuf)                      |
| REST           | - ubiquitous <br/> - client libraries in all languages, faster implementation <br />                                                                                        | - only supports unary request-response communication (HTTP1.1) <br/> - bigger over-the-wire message sizes (JSON) |
| CometBFT RPC | - easy to use                                                                                                                                                         | - bigger over-the-wire message sizes (JSON)                                                                   |
