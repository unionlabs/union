# Application BlockChain Interface (ABCI)

Blockchains are systems for multi-master state machine replication.
**ABCI** is an interface that defines the boundary between the replication engine (the blockchain),
and the state machine (the application).
Using a socket protocol, a consensus engine running in one process
can manage an application state running in another.

Previously, the ABCI was referred to as TMSP.



## Installation & Usage

To get up and running quickly, see the [getting started guide](../docs/guides/app-dev/getting-started.md) along with the [abci-cli documentation](../docs/guides/app-dev/abci-cli.md) which will go through the examples found in the [examples](./example/) directory.

## Specification

A detailed description of the ABCI methods and message types is contained in:

- [The main spec](https://github.com/cometbft/cometbft/blob/main/spec/abci/README.md)
- [A protobuf file](../proto/cometbft/types/v1/types.proto)
- [A Go interface](./types/application.go)

## Protocol Buffers

To compile the protobuf file, run (from the root of the repo):

```sh
make protoc_abci
```

See `protoc --help` and [the Protocol Buffers site](https://developers.google.com/protocol-buffers)
for details on compiling for other languages. Note we also include a [GRPC](https://www.grpc.io/docs)
service definition.
