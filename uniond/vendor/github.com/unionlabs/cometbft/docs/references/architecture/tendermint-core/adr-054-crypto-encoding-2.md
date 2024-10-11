# ADR 054: Crypto encoding (part 2)

## Changelog

2020-2-27: Created
2020-4-16: Update

## Context

Amino has been a pain point of many users in the ecosystem. While Tendermint does not suffer greatly from the performance degradation introduced by amino, we are making an effort in moving the encoding format to a widely adopted format, [Protocol Buffers](https://developers.google.com/protocol-buffers). With this migration a new standard is needed for the encoding of keys. This will cause ecosystem wide breaking changes.

Currently amino encodes keys as `<PrefixBytes> <Length> <ByteArray>`.

## Decision

Previously Tendermint defined all the key types for use in Tendermint and the Cosmos-SDK. Going forward the Cosmos-SDK will define its own protobuf type for keys. This will allow Tendermint to only define the keys that are being used in the codebase (ed25519).
There is the the opportunity to only define the usage of ed25519 (`bytes`) and not have it be a `oneof`, but this would mean that the `oneof` work is only being postponed to a later date. When using the `oneof` protobuf type we will have to manually switch over the possible key types and then pass them to the interface which is needed.

The approach that will be taken to minimize headaches for users is one where all encoding of keys will shift to protobuf and where amino encoding is relied on, there will be custom marshal and unmarshal functions.

Protobuf messages:

```proto
message PubKey {
  oneof key {
    bytes ed25519 = 1;
  }

message PrivKey {
  oneof sum {
    bytes ed25519 = 1;
  }
}
```

> Note: The places where backwards compatibility is needed is still unclear.

All modules currently do not rely on amino encoded bytes and keys are not amino encoded for genesis, therefore a hardfork upgrade is what will be needed to adopt these changes.

This work will be broken out into a few PRs, this work will be merged into a proto-breakage branch, all PRs will be reviewed prior to being merged:

1. Encoding of keys to protobuf and protobuf messages
2. Move Tendermint types to protobuf, mainly the ones that are being encoded.
3. Go one by one through the reactors and transition amino encoded messages to protobuf.
4. Test with cosmos-sdk and/or testnets repo.

## Status

Implemented

## Consequences

- Move keys to protobuf encoding, where backwards compatibility is needed, amino marshal and unmarshal functions will be used.

### Positive

- Protocol Buffer encoding will not change going forward.
- Removing amino overhead from keys will help with the KSM.
- Have a large ecosystem of supported languages.

### Negative

- Hardfork is required to integrate this into running chains.

### Neutral

## References

> Are there any relevant PR comments, issues that led up to this, or articles referenced for why we made the given design choice? If so link them here!

- {reference link}
