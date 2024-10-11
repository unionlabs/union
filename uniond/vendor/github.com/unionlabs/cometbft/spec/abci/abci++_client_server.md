---
order: 5
title: Client and Server
---

# Client and Server

This section is for those looking to implement their own ABCI Server, perhaps in
a new programming language.

You are expected to have read all previous sections of ABCI specification, namely
[Basic Concepts](./abci%2B%2B_basic_concepts.md),
[Methods](./abci%2B%2B_methods.md),
[Application Requirements](./abci%2B%2B_app_requirements.md), and
[Expected Behavior](./abci%2B%2B_comet_expected_behavior.md).

## Message Protocol and Synchrony

The message protocol consists of pairs of requests and responses defined in the
[protobuf file](https://github.com/cometbft/cometbft/blob/main/proto/cometbft/abci/v1/types.proto).

Some messages have no fields, while others may include byte-arrays, strings, integers,
or custom protobuf types.

For more details on protobuf, see the [documentation](https://developers.google.com/protocol-buffers/docs/overview).

## Server Implementations

To use ABCI in your programming language of choice, there must be an ABCI
server in that language. There are a few implementations of the ABCI server:

- In CometBFT repository:
    - In-process
    - [ABCI-socket server](../../abci/server/socket_server.go)
    - [GRPC server](../../abci/server/grpc_server.go)
- [tendermint-rs](https://github.com/informalsystems/tendermint-rs)
- [tower-abci](https://github.com/penumbra-zone/tower-abci)

The implementations in CometBFT repository can be tested using the [ABCI-CLI](../../docs/guides/app-dev/abci-cli.md) tool.

### In Process

The simplest implementation uses function calls in Golang.
This means ABCI applications written in Golang can be linked with CometBFT and run as a single binary.

### GRPC

If you are not using Golang,
but [GRPC](https://grpc.io/) is available in your language, this is the easiest approach,
though it will have significant performance overhead.

Please check GRPC's documentation to know to set up the Application as an
ABCI GRPC server.

### Socket

The CometBFT Socket Protocol is an asynchronous, raw socket server protocol which provides ordered
message passing over Unix or TCP sockets. Messages are serialized using Protobuf3 and length-prefixed
with an [unsigned varint](https://developers.google.com/protocol-buffers/docs/encoding?csw=1#varints)

If gRPC is not available in your language, or you require higher performance, or
otherwise enjoy programming, you may implement your own ABCI server using the
CometBFT Socket Protocol. The first step is still to auto-generate the
relevant data types and codec in your language using `protoc`, and then you need to
ensure you handle the unsigned `varint`-based message length encoding scheme
when reading and writing messages to the socket.

Note that our length prefixing scheme does not apply to gRPC.

Also note that your ABCI server must be able to handle multiple connections,
as CometBFT uses four connections.

## Client

There are currently two use-cases for an ABCI client. One is testing
tools that allow ABCI requests to be sent to the actual application via
command line. An example of this is `abci-cli`, which accepts CLI commands
to send corresponding ABCI requests.
The other is a consensus engine, such as CometBFT,
which makes ABCI requests to the application as prescribed by the consensus
algorithm used.
