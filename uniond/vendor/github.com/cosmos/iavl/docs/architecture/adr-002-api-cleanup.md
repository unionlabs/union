# ADR ADR-002: API Cleanup to Improve Commit Performance

## Changelog

* 2023-06-06: First draft

## Status

DRAFT

## Abstract

This ADR proposes a cleanup of the API to make more understandable and maintainable the codebase of `iavl`.

There is a lot of legacy code in the SDK that is not used anymore and can be removed. See the [Discussion](https://github.com/cosmos/iavl/issues/737) for more details.

There are some proposals for the speedup of the `Commit` by the async writes. See the [Discussion](https://github.com/cosmos/cosmos-sdk/issues/16173) for more details.

## Context

The current implementation of `iavl` suffers from performance issues due to synchronous writes during the `Commit` process. To address this, the proposed changes aim to finalize the current version in memory and introduce asynchronous writes in the background.

It is also necessary to refactor the `Set` function to accept a batch object for the atomic commitments.

Moreover, the existing architecture of `iavl` lacks modularity and code organization:

* The boundary between `ImmutableTree` and `MutableTree` is not clear.
* There are many public methods in `nodeDB`, making it less structured.

## Decision

To make the codebase more clear and improve `Commit` performance, we propose the following changes:

### Batch Set

* Refactor the `Set` function to accept a batch object.
* Eliminate the usage of `dbm.Batch` in `nodeDB`.

### Async Commit

* Finalize the current version in memory during `Commit`.
* Perform async writes in the background.

### API Cleanup

* Make `nodeDB` methods private and remove all unused methods.
* Follow the naming convention of the `store` module.

The exposed API of `iavl` will be as follows:

```go
    MutableTree interface {
        SaveChangeSet(cs *ChangeSet) ([]byte, error) // this is for batch set
        WorkingHash() []byte
        Commit() ([]byte, int64, error) // SaveVersion -> Commit
        Close() error // this is to make sure the async write is finished when shutdown
        DeleteVersionsTo(version int64) error
        LoadVersionForOverwriting(targetVersion int64) error
        GetImmutableTree(version int64) (*ImmutableTree, error)
    }

    ImmutableTree interface {
        Has(key []byte) (bool, error)
        Get(key []byte) ([]byte, error)
        Hash() []byte
        Iterator(start, end []byte, ascending bool) (types.Iterator, error)
        VersionExists(version int64) bool
        GetVersionedProof(key []byte, version int64) ([]byte, *cmtprotocrypto.ProofOps, error)
    }
```

## Consequences

We expect the proposed changes to improve the performance of Commit through async writes. Additionally, the codebase will become more organized and maintainable.

### Backwards Compatibility

While this ADR will break the external API of `iavl`, it will not affect the internal state of nodeDB. Compatibility measures and migration steps will be necessary during the API cleanup to handle the breaking changes.

### Positive

* Atomicity of the `Commitments`.
* Improved Commit performance through async writes.
* Increased flexibility and ease of modification and refactoring for iavl.

### Negative

* Async Commit may result in increased memory usage and increased code complexity.

## References

* <https://github.com/cosmos/cosmos-sdk/issues/16173>
* <https://github.com/cosmos/iavl/issues/737>
