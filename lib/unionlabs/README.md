# `unionlabs`

This crate contains types and traits used throughout our stack.

## Wrappers around generated code

Due to how prost generates protobuf bindings, working with the raw generated code is quite unergonomic and error-prone. This library provides a unified interface for all encodings (protobuf, ethabi, etc), allowing for both more type safety and ease of use for downstream users.

The general pattern is as follows, using `Channel` as an example:

- prost generated code: `protos::ibc::core::channel::v1::Channel`
- ethers generated code: `contracts::ibc_handler::IbcCoreChannelV1ChannelData`
- unionlabs type: `unionlabs::ibc::core::channel::Channel`

Note that the `unionlabs` type doesn't include the version - we currently only support the latest version of each module.

## Hash and UInt

The most commonly used crate for hash and uint types is parity's [primitive-types](https://docs.rs/primitive-types/latest/primitive_types/), which is quite old and has quite a lot of issues ([hidden panics](https://github.com/paritytech/parity-common/issues/764), [strange Display impl](https://github.com/paritytech/parity-common/issues/656), [parsing inconsistencies](https://github.com/paritytech/parity-common/issues/643), etc). There are alternatives, such as [ruint](https://github.com/recmo/uint), which is used in [alloy](https://github.com/alloy-rs/core), but for simplicity we currently define our own hash and uint types (note that our uint is just a wrapper around `primitive-types::U256` that provides sane string parsing and Display defaults). We will likely migrate to `ruint` in the future, once we migrate to `alloy` from `ethers`.

## Encoding and Decoding

This library exposes generic Encoding and Decoding traits, to allow for abstracting over the encoding used for a type (essentially a poor man's HKTs). See the Voyager source code for examples of how this can be used.
