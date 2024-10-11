[NB]: # (
  Ensure that all hyperlinks in this doc are absolute URLs, not relative ones,
  as this doc gets published to the Buf registry and relative URLs will fail
  to resolve.
)

# CometBFT Protocol Buffers Definitions

This is the set of [Protobuf][protobuf] definitions of types used by various
parts of [CometBFT]:

- The [Application Blockchain Interface][abci] (ABCI), especially in the context
  of _remote_ applications.
- The P2P layer, in how CometBFT nodes interact with each other over the
  network.
- In interaction with remote signers ("privval").
- The RPC, in that the native JSON serialization of certain Protobuf types is
  used when accepting and responding to RPC requests.
- The storage layer, in how data is serialized to and deserialized from on-disk
  storage.

The canonical Protobuf definitions live in the `proto` folder of the relevant
release branch of CometBFT. These definitions are published to the [Buf
registry][buf] for integrators' convenience.

The Protobuf files are organized under two domains: `cometbft` and `tendermint`.
The `cometbft.*` packages use version suffixes to let application developers
target versions of the protocols as they have evolved between CometBFT releases.

## Which CometBFT release does each package belong to?

By the 1.0.0 release, the entire set of Protobuf definitions used by CometBFT
is published in packages suffixed with `.v1`. Earlier revisions of the
definitions, where they differed, are provided alongside in `.v1beta`_N_
packages. The correspondence between package suffixes and releases is as follows:

| Domain          | 0.34      | 0.37      | 0.38      | 1.0  |
|-----------------|-----------|-----------|-----------|------|
| `abci`          | `v1beta1` | `v1beta2` | `v1beta3` | `v1` |
| `blocksync`     |           | `v1beta1` | `v1`      | `v1` |
| `consensus`     | `v1beta1` | `v1beta1` | `v1beta1` | `v1` |
| `crypto`        | `v1`      | `v1`      | `v1`      | `v1` |
| `libs/bits`     | `v1`      | `v1`      | `v1`      | `v1` |
| `mempool`       | `v1`      | `v1`      | `v1`      | `v1` |
| `p2p`           | `v1`      | `v1`      | `v1`      | `v1` |
| `privval`       | `v1beta1` | `v1beta1` | `v1beta2` | `v1` |
| `rpc/grpc`[^1]  | `v1beta1` | `v1beta2` | `v1beta3` |      |
| `state`         | `v1beta1` | `v1beta2` | `v1beta3` | `v1` |
| `statesync`     | `v1`      | `v1`      | `v1`      | `v1` |
| `types`         | `v1beta1` | `v1beta2` | `v1`      | `v1` |
| `version`       | `v1`      | `v1`      | `v1`      | `v1` |

[^1]: Retired in 1.0

## Why does CometBFT provide `tendermint` Protobuf definitions?

This is as a result of CometBFT being a fork of [Tendermint Core][tmcore] and
wanting to provide integrators with as painless a way as possible of
transitioning from Tendermint Core to CometBFT.

As of CometBFT v1, however, the project will transition to using and providing a
`cometbft` package of Protobuf definitions (see [\#1330]).

Protobuf definitions for each respective release are also, for convenience,
published to a corresponding branch in the `tendermint/tendermint` Buf repository.

| CometBFT version | Canonical Protobufs                         | Buf registry                              |
|------------------|---------------------------------------------|-------------------------------------------|
| v0.38.x          | [v0.38.x Protobuf definitions][v038-protos] | [Buf repository v0.38.x branch][v038-buf] |
| v0.37.x          | [v0.37.x Protobuf definitions][v037-protos] | [Buf repository v0.37.x branch][v037-buf] |
| v0.34.x          | [v0.34.x Protobuf definitions][v034-protos] | [Buf repository v0.34.x branch][v034-buf] |

[protobuf]: https://protobuf.dev/
[CometBFT]: https://github.com/cometbft/cometbft
[abci]: https://github.com/cometbft/cometbft/tree/main/spec/abci
[buf]: https://buf.build/tendermint/tendermint
[tmcore]: https://github.com/tendermint/tendermint
[\#1330]: https://github.com/cometbft/cometbft/issues/1330
[v034-protos]: https://github.com/cometbft/cometbft/tree/v0.34.x/proto
[v034-buf]: https://buf.build/tendermint/tendermint/docs/v0.34.x
[v037-protos]: https://github.com/cometbft/cometbft/tree/v0.37.x/proto
[v037-buf]: https://buf.build/tendermint/tendermint/docs/v0.37.x
[v038-protos]: https://github.com/cometbft/cometbft/tree/v0.38.x/proto
[v038-buf]: https://buf.build/tendermint/tendermint/docs/v0.38.x
