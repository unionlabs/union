# ADR 055: Protobuf Design

## Changelog

- 2020-4-15: Created (@marbar3778)
- 2020-6-18: Updated (@marbar3778)

## Context

Currently we use [go-amino](https://github.com/tendermint/go-amino) throughout Tendermint. Amino is not being maintained anymore (April 15, 2020) by the Tendermint team and has been found to have issues:

- https://github.com/tendermint/go-amino/issues/286
- https://github.com/tendermint/go-amino/issues/230
- https://github.com/tendermint/go-amino/issues/121

These are a few of the known issues that users could run into.

Amino enables quick prototyping and development of features. While this is nice, amino does not provide the performance and developer convenience that is expected. For Tendermint to see wider adoption as a BFT protocol engine a transition to an adopted encoding format is needed. Below are some possible options that can be explored.

There are a few options to pick from:

- `Protobuf`: Protocol buffers are Google's language-neutral, platform-neutral, extensible mechanism for serializing structured data – think XML, but smaller, faster, and simpler. It is supported in countless languages and has been proven in production for many years.

- `FlatBuffers`: FlatBuffers is an efficient cross platform serialization library. Flatbuffers are more efficient than Protobuf due to the fast that there is no parsing/unpacking to a second representation. FlatBuffers has been tested and used in production but is not widely adopted.

- `CapnProto`: Cap’n Proto is an insanely fast data interchange format and capability-based RPC system. Cap'n Proto does not have a encoding/decoding step. It has not seen wide adoption throughout the industry.

- @erikgrinaker - https://github.com/tendermint/tendermint/pull/4623#discussion_r401163501
  ```
  Cap'n'Proto is awesome. It was written by one of the original Protobuf developers to fix some of its issues, and supports e.g. random access to process huge messages without loading them into memory and an (opt-in) canonical form which would be very useful when determinism is needed (e.g. in the state machine). That said, I suspect Protobuf is the better choice due to wider adoption, although it makes me kind of sad since Cap'n'Proto is technically better.
  ```

## Decision

Transition Tendermint to Protobuf because of its performance and tooling. The Ecosystem behind Protobuf is vast and has outstanding [support for many languages](https://developers.google.com/protocol-buffers/docs/tutorials).

We will be making this possible by keeping the current types in there current form (handwritten) and creating a `/proto` directory in which all the `.proto` files will live. Where encoding is needed, on disk and over the wire, we will call util functions that will transition the types from handwritten go types to protobuf generated types. This is inline with the recommended file structure from [buf](https://buf.build). You can find more information on this file structure [here](https://buf.build/docs/lint-checkers#file_layout).

By going with this design we will enable future changes to types and allow for a more modular codebase.

## Status

Implemented

## Consequences

### Positive

- Allows for modular types in the future
- Less refactoring
- Allows the proto files to be pulled into the spec repo in the future.
- Performance
- Tooling & support in multiple languages

### Negative

- When a developer is updating a type they need to make sure to update the proto type as well

### Neutral

## References
