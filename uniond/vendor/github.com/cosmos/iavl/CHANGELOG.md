# Changelog

## v1.1.2 April 8, 2024

### Bug Fixes

- [#928](https://github.com/cosmos/iavl/pull/928) Fix the reformatted root node issue.

## v1.1.1 March 16, 2024

### Bug Fixes

- [#910](https://github.com/cosmos/iavl/pull/910) Fix the reference root format from (prefix, version) to (prefix, version, nonce)

### Improvements

- [#910](https://github.com/cosmos/iavl/pull/910) Async pruning of legacy orphan nodes.

## v1.1.0 February 29, 2024

### API Breaking Changes

- [#874](https://github.com/cosmos/iavl/pull/874) Decouple `cosmos-db` and implement own `db` package.

## v1.0.1 February 16, 2024

### Improvements

- [#876](https://github.com/cosmos/iavl/pull/876) Make pruning of legacy orphan nodes asynchronous.

## v1.0.0 (October 30, 2023)

### Improvements

- [#695](https://github.com/cosmos/iavl/pull/695) Add API `SaveChangeSet` to save the changeset as a new version.
- [#703](https://github.com/cosmos/iavl/pull/703) New APIs `NewCompressExporter`/`NewCompressImporter` to support more compact snapshot format.
- [#729](https://github.com/cosmos/iavl/pull/729) Speedup Genesis writes for IAVL, by writing in small batches.
- [#726](https://github.com/cosmos/iavl/pull/726) Make `KVPair` and `ChangeSet` serializable with protobuf.
- [#718](https://github.com/cosmos/iavl/pull/718) Fix `traverseNodes` unexpected behaviour
- [#770](https://github.com/cosmos/iavl/pull/770) Add `WorkingVersion()int64` API.

### Bug Fixes

- [#773](https://github.com/cosmos/iavl/pull/773) Fix memory leak in `Import`.
- [#801](https://github.com/cosmos/iavl/pull/801) Fix rootKey empty check by len equals 0.
- [#805](https://github.com/cosmos/iavl/pull/805) Use `sync.Map` instead of map to prevent concurrent writes at the fast node level

### Breaking Changes

- [#735](https://github.com/cosmos/iavl/pull/735) Pass logger to `NodeDB`, `MutableTree` and `ImmutableTree`
- [#646](https://github.com/cosmos/iavl/pull/646) Remove the `orphans` from the storage
- [#777](https://github.com/cosmos/iavl/pull/777) Don't return errors from ImmutableTree.Hash, NewImmutableTree
- [#815](https://github.com/cosmos/iavl/pull/815) `NewMutableTreeWithOpts` was removed in favour of accepting options via a variadic in `NewMutableTree`
- [#815](https://github.com/cosmos/iavl/pull/815) `NewImmutableTreeWithOpts` is removed in favour of accepting options via a variadic in `NewImmutableTree`
- [#646](https://github.com/cosmos/iavl/pull/646) Remove the `DeleteVersion`, `DeleteVersions`, `DeleteVersionsRange` and introduce a new endpoint of `DeleteVersionsTo` instead

## 0.20.0 (March 14, 2023)

### Breaking Changes

- [#586](https://github.com/cosmos/iavl/pull/586) Remove the `RangeProof` and refactor the ics23_proof to use the internal methods.

## 0.19.5 (Februrary 23, 2022)

### Breaking Changes

- [#622](https://github.com/cosmos/iavl/pull/622) `export/newExporter()` and `ImmutableTree.Export()` returns error for nil arguements

- [#640](https://github.com/cosmos/iavl/pull/640) commit `NodeDB` batch in `LoadVersionForOverwriting`.
- [#636](https://github.com/cosmos/iavl/pull/636) Speed up rollback method: `LoadVersionForOverwriting`.
- [#654](https://github.com/cosmos/iavl/pull/654) Add API `TraverseStateChanges` to extract state changes from iavl versions.
- [#638](https://github.com/cosmos/iavl/pull/638) Make LazyLoadVersion check the opts.InitialVersion, add API `LazyLoadVersionForOverwriting`.

## 0.19.4 (October 28, 2022)

- [#599](https://github.com/cosmos/iavl/pull/599) Populate ImmutableTree creation in copy function with missing field
- [#589](https://github.com/cosmos/iavl/pull/589) Wrap `tree.addUnsavedRemoval()` with missing `if !tree.skipFastStorageUpgrade` statement

## 0.19.3 (October 8, 2022)

- `ProofInner.Hash()` prevents both right and left from both being set. Only one is allowed to be set.

## 0.19.2 (October 6, 2022)

- [#547](https://github.com/cosmos/iavl/pull/547) Implement `skipFastStorageUpgrade` in order to skip fast storage upgrade and usage.
- [#531](https://github.com/cosmos/iavl/pull/531) Upgrade to fast storage in batches.

## 0.19.1 (August 3, 2022)

### Improvements

- [#525](https://github.com/cosmos/iavl/pull/525) Optimization: use fast unsafe bytes->string conversion.
- [#506](https://github.com/cosmos/iavl/pull/506) Implement cache abstraction.

### Bug Fixes

- [#524](https://github.com/cosmos/iavl/pull/524) Fix: `MutableTree.Get`.

## 0.19.0 (July 6, 2022)

### Breaking Changes

- [#514](https://github.com/cosmos/iavl/pull/514) Downgrade Tendermint to 0.34.x
- [#500](https://github.com/cosmos/iavl/pull/500) Return errors instead of panicking.

### Improvements

- [#514](https://github.com/cosmos/iavl/pull/514) Use Go v1.18

## 0.18.0 (March 10, 2022)

### Breaking Changes

- Bumped Tendermint to 0.35.1

### Improvements

- [\#468](https://github.com/cosmos/iavl/pull/468) Fast storage optimization for queries and iterations
- [\#452](https://github.com/cosmos/iavl/pull/452) Optimization: remove unnecessary (\*bytes.Buffer).Reset right after creating buffer.
- [\#445](https://github.com/cosmos/iavl/pull/445) Bump github.com/tendermint/tendermint to v0.35.0
- [\#453](https://github.com/cosmos/iavl/pull/453),[\#456](https://github.com/cosmos/iavl/pull/456) Optimization: buffer reuse
- [\#474](https://github.com/cosmos/iavl/pull/474) bump github.com/confio/ics23 to v0.7
- [\#475](https://github.com/cosmos/iavl/pull/475) Use go v1.17

For previous changelogs visit: <https://github.com/cosmos/iavl/blob/v0.18.0/CHANGELOG.md>
