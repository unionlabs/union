# ADR 009: ABCI UX Improvements

## Changelog

23-06-2018: Some minor fixes from review
07-06-2018: Some updates based on discussion with Jae
07-06-2018: Initial draft to match what was released in ABCI v0.11

## Context

The ABCI was first introduced in late 2015. It's purpose is to be:

- a generic interface between state machines and their replication engines
- agnostic to the language the state machine is written in
- agnostic to the replication engine that drives it

This means ABCI should provide an interface for both pluggable applications and
pluggable consensus engines.

To achieve this, it uses Protocol Buffers (proto3) for message types. The dominant
implementation is in Go.

After some recent discussions with the community on github, the following were
identified as pain points:

- Amino encoded types
- Managing validator sets
- Imports in the protobuf file

See the [references](#references) for more.

### Imports

The native proto library in Go generates inflexible and verbose code.
Many in the Go community have adopted a fork called
[gogoproto](https://github.com/cosmos/gogoproto) that provides a
variety of features aimed to improve the developer experience.
While `gogoproto` is nice, it creates an additional dependency, and compiling
the protobuf types for other languages has been reported to fail when `gogoproto` is used.

### Amino

Amino is an encoding protocol designed to improve over insufficiencies of protobuf.
It's goal is to be proto4.

Many people are frustrated by incompatibility with protobuf,
and with the requirement for Amino to be used at all within ABCI.

We intend to make Amino successful enough that we can eventually use it for ABCI
message types directly. By then it should be called proto4. In the meantime,
we want it to be easy to use.

### PubKey

PubKeys are encoded using Amino (and before that, go-wire).
Ideally, PubKeys are an interface type where we don't know all the
implementation types, so its unfitting to use `oneof` or `enum`.

### Addresses

The address for ED25519 pubkey is the RIPEMD160 of the Amino
encoded pubkey. This introduces an Amino dependency in the address generation,
a functionality that is widely required and should be easy to compute as
possible.

### Validators

To change the validator set, applications can return a list of validator updates
with ResponseEndBlock. In these updates, the public key _must_ be included,
because Tendermint requires the public key to verify validator signatures. This
means ABCI developers have to work with PubKeys. That said, it would also be
convenient to work with address information, and for it to be simple to do so.

### AbsentValidators

Tendermint also provides a list of validators in BeginBlock who did not sign the
last block. This allows applications to reflect availability behavior in the
application, for instance by punishing validators for not having votes included
in commits.

### InitChain

Tendermint passes in a list of validators here, and nothing else. It would
benefit the application to be able to control the initial validator set. For
instance the genesis file could include application-based information about the
initial validator set that the application could process to determine the
initial validator set. Additionally, InitChain would benefit from getting all
the genesis information.

### Header

ABCI provides the Header in RequestBeginBlock so the application can have
important information about the latest state of the blockchain.

## Decision

### Imports

Move away from gogoproto. In the short term, we will just maintain a second
protobuf file without the gogoproto annotations. In the medium term, we will
make copies of all the structs in Golang and shuttle back and forth. In the long
term, we will use Amino.

### Amino

To simplify ABCI application development in the short term,
Amino will be completely removed from the ABCI:

- It will not be required for PubKey encoding
- It will not be required for computing PubKey addresses

That said, we are working to make Amino a huge success, and to become proto4.
To facilitate adoption and cross-language compatibility in the near-term, Amino
v1 will:

- be fully compatible with the subset of proto3 that excludes `oneof`
- use the Amino prefix system to provide interface types, as opposed to `oneof`
  style union types.

That said, an Amino v2 will be worked on to improve the performance of the
format and its usability in cryptographic applications.

### PubKey

Encoding schemes infect software. As a generic middleware, ABCI aims to have
some cross scheme compatibility. For this it has no choice but to include opaque
bytes from time to time. While we will not enforce Amino encoding for these
bytes yet, we need to provide a type system. The simplest way to do this is to
use a type string.

PubKey will now look like:

```
message PubKey {
    string type
    bytes data
}
```

where `type` can be:

- "ed225519", with `data = <raw 32-byte pubkey>`
- "secp256k1", with `data = <33-byte OpenSSL compressed pubkey>`

As we want to retain flexibility here, and since ideally, PubKey would be an
interface type, we do not use `enum` or `oneof`.

### Addresses

To simplify and improve computing addresses, we change it to the first 20-bytes of the SHA256
of the raw 32-byte public key.

We continue to use the Bitcoin address scheme for secp256k1 keys.

### Validators

Add a `bytes address` field:

```
message Validator {
    bytes address
    PubKey pub_key
    int64 power
}
```

### RequestBeginBlock and AbsentValidators

To simplify this, RequestBeginBlock will include the complete validator set,
including the address, and voting power of each validator, along
with a boolean for whether or not they voted:

```
message RequestBeginBlock {
  bytes hash
  Header header
  LastCommitInfo last_commit_info
  repeated Evidence byzantine_validators
}

message LastCommitInfo {
  int32 CommitRound
  repeated SigningValidator validators
}

message SigningValidator {
    Validator validator
    bool signed_last_block
}
```

Note that in Validators in RequestBeginBlock, we DO NOT include public keys. Public keys are
larger than addresses and in the future, with quantum computers, will be much
larger. The overhead of passing them, especially during fast-sync, is
significant.

Additional, addresses are changing to be simpler to compute, further removing
the need to include pubkeys here.

In short, ABCI developers must be aware of both addresses and public keys.

### ResponseEndBlock

Since ResponseEndBlock includes Validator, it must now include their address.

### InitChain

Change RequestInitChain to give the app all the information from the genesis file:

```
message RequestInitChain {
    int64 time
    string chain_id
    ConsensusParams consensus_params
    repeated Validator validators
    bytes app_state_bytes
}
```

Change ResponseInitChain to allow the app to specify the initial validator set
and consensus parameters.

```
message ResponseInitChain {
    ConsensusParams consensus_params
    repeated Validator validators
}
```

### Header

Now that Tendermint Amino will be compatible with proto3, the Header in ABCI
should exactly match the Tendermint header - they will then be encoded
identically in ABCI and in Tendermint Core.

## Status

Implemented

## Consequences

### Positive

- Easier for developers to build on the ABCI
- ABCI and Tendermint headers are identically serialized

### Negative

- Maintenance overhead of alternative type encoding scheme
- Performance overhead of passing all validator info every block (at least its
  only addresses, and not also pubkeys)
- Maintenance overhead of duplicate types

### Neutral

- ABCI developers must know about validator addresses

## References

- [ABCI v0.10.3 Specification (before this
  proposal)](https://github.com/tendermint/abci/blob/v0.10.3/specification.rst)
- [ABCI v0.11.0 Specification (implementing first draft of this
  proposal)](https://github.com/tendermint/abci/blob/v0.11.0/specification.md)
- [Ed25519 addresses](https://github.com/tendermint/go-crypto/issues/103)
- [InitChain contains the
  Genesis](https://github.com/tendermint/abci/issues/216)
- [PubKeys](https://github.com/tendermint/tendermint/issues/1524)
- [Notes on
  Header](https://github.com/tendermint/tendermint/issues/1605)
- [Gogoproto issues](https://github.com/tendermint/abci/issues/256)
- [Absent Validators](https://github.com/tendermint/abci/issues/231)
