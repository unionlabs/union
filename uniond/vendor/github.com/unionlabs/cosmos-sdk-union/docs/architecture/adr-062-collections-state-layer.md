# ADR 062: Collections, a simplified storage layer for cosmos-sdk modules.

## Changelog

* 30/11/2022: PROPOSED

## Status

PROPOSED - Implemented

## Abstract

We propose a simplified module storage layer which leverages golang generics to allow module developers to handle module
storage in a simple and straightforward manner, whilst offering safety, extensibility and standardisation.

## Context

Module developers are forced into manually implementing storage functionalities in their modules, those functionalities include
but are not limited to:

- Defining key to bytes formats.
- Defining value to bytes formats.
- Defining secondary indexes.
- Defining query methods to expose outside to deal with storage.
- Defining local methods to deal with storage writing.
- Dealing with genesis imports and exports.
- Writing tests for all the above.


This brings in a lot of problems:
- It blocks developers from focusing on the most important part: writing business logic.
- Key to bytes formats are complex and their definition is error-prone, for example:
  - how do I format time to bytes in such a way that bytes are sorted?
  - how do I ensure when I don't have namespace collisions when dealing with secondary indexes?
- The lack of standardisation makes life hard for clients, and the problem is exacerbated when it comes to providing proofs for objects present in state. Clients are forced to maintain a list of object paths to gather proofs.

### Current Solution: ORM

The current SDK proposed solution to this problem is [ORM](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-055-orm.md).
Whilst ORM offers a lot of good functionality aimed at solving these specific problems, it has some downsides:
- It requires migrations.
- It uses the newest protobuf golang API, whilst the SDK still mainly uses gogoproto. 
- Integrating ORM into a module would require the developer to deal with two different golang frameworks (golang protobuf + gogoproto) representing the same API objects.
- It has a high learning curve, even for simple storage layers as it requires developers to have knowledge around protobuf options, custom cosmos-sdk storage extensions, and tooling download. Then after this they still need to learn the code-generated API.

### CosmWasm Solution: cw-storage-plus

The collections API takes inspiration from [cw-storage-plus](https://docs.cosmwasm.com/docs/smart-contracts/state/cw-plus),
which has demonstrated to be a powerful tool for dealing with storage in CosmWasm contracts.
It's simple, does not require extra tooling, it makes it easy to deal with complex storage structures (indexes, snapshot, etc).
The API is straightforward and explicit.

## Decision

We propose to port the `collections` API, whose implementation lives in [NibiruChain/collections](https://github.com/NibiruChain/collections) to cosmos-sdk.

Collections implements four different storage handlers types:

- `Map`: which deals with simple `key=>object` mappings.
- `KeySet`: which acts as a `Set` and only retains keys and no object (usecase: allow-lists).
- `Item`: which always contains only one object (usecase: Params)
- `Sequence`: which implements a simple always increasing number (usecase: Nonces)
- `IndexedMap`: builds on top of `Map` and `KeySet` and allows to create relationships with `Objects` and `Objects` secondary keys.

All the collection APIs build on top of the simple `Map` type.

Collections is fully generic, meaning that anything can be used as `Key` and `Value`. It can be a protobuf object or not.

Collections types, in fact, delegate the duty of serialisation of keys and values to a secondary collections API component called `ValueEncoders` and `KeyEncoders`.

`ValueEncoders` take care of converting a value to bytes (relevant only for `Map`). And offers a plug and play layer which allows us to change how we encode objects, 
which is relevant for swapping serialisation frameworks and enhancing performance.
`Collections` already comes in with default `ValueEncoders`, specifically for: protobuf objects, special SDK types (sdk.Int, sdk.Dec).

`KeyEncoders` take care of converting keys to bytes, `collections` already comes in with some default `KeyEncoders` for some privimite golang types
(uint64, string, time.Time, ...) and some widely used sdk types (sdk.Acc/Val/ConsAddress, sdk.Int/Dec, ...).
These default implementations also offer safety around proper lexicographic ordering and namespace-collision.

Examples of the collections API can be found here:
- introduction: https://github.com/NibiruChain/collections/tree/main/examples
- usage in nibiru: [x/oracle](https://github.com/NibiruChain/nibiru/blob/master/x/oracle/keeper/keeper.go#L32), [x/epoch](https://github.com/NibiruChain/nibiru/blob/4566d9f6d22807abbd78c01454664d64f6e108e0/x/epochs/keeper/epoch.go)
- cosmos-sdk's x/staking migrated: https://github.com/testinginprod/cosmos-sdk/pull/22


## Consequences

### Backwards Compatibility

The design of `ValueEncoders` and `KeyEncoders` allows modules to retain the same `byte(key)=>byte(value)` mappings, making
the upgrade to the new storage layer non-state breaking.


### Positive

- ADR aimed at removing code from the SDK rather than adding it. Migrating just `x/staking` to collections would yield to a net decrease in LOC (even considering the addition of collections itself).
- Simplifies and standardises storage layers across modules in the SDK.
- Does not require to have to deal with protobuf.
- It's pure golang code.
- Generalisation over `KeyEncoders` and `ValueEncoders` allows us to not tie ourself to the data serialisation framework.
- `KeyEncoders` and `ValueEncoders` can be extended to provide schema reflection.

### Negative

- Golang generics are not as battle-tested as other Golang features, despite being used in production right now.
- Collection types instantiation needs to be improved.

### Neutral

{neutral consequences}

## Further Discussions

- Automatic genesis import/export (not implemented because of API breakage)
- Schema reflection


## References
