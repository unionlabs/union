# ADR 055: ORM

## Changelog

* 2022-04-27: First draft

## Status

ACCEPTED Implemented

## Abstract

In order to make it easier for developers to build Cosmos SDK modules and for clients to query, index and verify proofs
against state data, we have implemented an ORM (object-relational mapping) layer for the Cosmos SDK.

## Context

Historically modules in the Cosmos SDK have always used the key-value store directly and created various handwritten
functions for managing key format as well as constructing secondary indexes. This consumes a significant amount of
time when building a module and is error-prone. Because key formats are non-standard, sometimes poorly documented,
and subject to change, it is hard for clients to generically index, query and verify merkle proofs against state data.

The known first instance of an "ORM" in the Cosmos ecosystem was in [weave](https://github.com/iov-one/weave/tree/master/orm).
A later version was built for [regen-ledger](https://github.com/regen-network/regen-ledger/tree/157181f955823149e1825263a317ad8e16096da4/orm) for
use in the group module and later [ported to the SDK](https://github.com/cosmos/cosmos-sdk/tree/35d3312c3be306591fcba39892223f1244c8d108/x/group/internal/orm)
just for that purpose.

While these earlier designs made it significantly easier to write state machines, they still required a lot of manual
configuration, didn't expose state format directly to clients, and were limited in their support of different types
of index keys, composite keys, and range queries.

Discussions about the design continued in https://github.com/cosmos/cosmos-sdk/discussions/9156 and more
sophisticated proofs of concept were created in https://github.com/allinbits/cosmos-sdk-poc/tree/master/runtime/orm
and https://github.com/cosmos/cosmos-sdk/pull/10454.

## Decision

These prior efforts culminated in the creation of the Cosmos SDK `orm` go module which uses protobuf annotations
for specifying ORM table definitions. This ORM is based on the new `google.golang.org/protobuf/reflect/protoreflect`
API and supports:

* sorted indexes for all simple protobuf types (except `bytes`, `enum`, `float`, `double`) as well as `Timestamp` and `Duration`
* unsorted `bytes` and `enum` indexes
* composite primary and secondary keys
* unique indexes
* auto-incrementing `uint64` primary keys
* complex prefix and range queries
* paginated queries
* complete logical decoding of KV-store data

Almost all the information needed to decode state directly is specified in .proto files. Each table definition specifies
an ID which is unique per .proto file and each index within a table is unique within that table. Clients then only need
to know the name of a module and the prefix ORM data for a specific .proto file within that module in order to decode
state data directly. This additional information will be exposed directly through app configs which will be explained
in a future ADR related to app wiring.

The ORM makes optimizations around storage space by not repeating values in the primary key in the key value
when storing primary key records. For example, if the object `{"a":0,"b":1}` has the primary key `a`, it will
be stored in the key value store as `Key: '0', Value: {"b":1}` (with more efficient protobuf binary encoding).
Also, the generated code from https://github.com/cosmos/cosmos-proto does optimizations around the
`google.golang.org/protobuf/reflect/protoreflect` API to improve performance.

A code generator is included with the ORM which creates type safe wrappers around the ORM's dynamic `Table`
implementation and is the recommended way for modules to use the ORM.

The ORM tests provide a simplified bank module demonstration which illustrates:
* [ORM proto options](https://github.com/cosmos/cosmos-sdk/blob/0d846ae2f0424b2eb640f6679a703b52d407813d/orm/internal/testpb/bank.proto)
* [Generated Code](https://github.com/cosmos/cosmos-sdk/blob/0d846ae2f0424b2eb640f6679a703b52d407813d/orm/internal/testpb/bank.cosmos_orm.go)
* [Example Usage in a Module Keeper](https://github.com/cosmos/cosmos-sdk/blob/0d846ae2f0424b2eb640f6679a703b52d407813d/orm/model/ormdb/module_test.go)

## Consequences

### Backwards Compatibility

State machine code that adopts the ORM will need migrations as the state layout is generally backwards incompatible.
These state machines will also need to migrate to https://github.com/cosmos/cosmos-proto at least for state data.

### Positive

* easier to build modules
* easier to add secondary indexes to state
* possible to write a generic indexer for ORM state
* easier to write clients that do state proofs
* possible to automatically write query layers rather than needing to manually implement gRPC queries

### Negative

* worse performance than handwritten keys (for now). See [Further Discussions](#further-discussions)
for potential improvements

### Neutral

## Further Discussions

Further discussions will happen within the Cosmos SDK Framework Working Group. Current planned and ongoing work includes:

* automatically generate client-facing query layer
* client-side query libraries that transparently verify light client proofs
* index ORM data to SQL databases
* improve performance by:
    * optimizing existing reflection based code to avoid unnecessary gets when doing deletes & updates of simple tables
    * more sophisticated code generation such as making fast path reflection even faster (avoiding `switch` statements),
  or even fully generating code that equals handwritten performance


## References

* https://github.com/iov-one/weave/tree/master/orm).
* https://github.com/regen-network/regen-ledger/tree/157181f955823149e1825263a317ad8e16096da4/orm
* https://github.com/cosmos/cosmos-sdk/tree/35d3312c3be306591fcba39892223f1244c8d108/x/group/internal/orm
* https://github.com/cosmos/cosmos-sdk/discussions/9156
* https://github.com/allinbits/cosmos-sdk-poc/tree/master/runtime/orm
* https://github.com/cosmos/cosmos-sdk/pull/10454
