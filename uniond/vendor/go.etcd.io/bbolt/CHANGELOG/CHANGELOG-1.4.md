
<hr>

## v1.4.0-alpha.1(TBD)

### BoltDB
- [Enhance check functionality to support checking starting from a pageId](https://github.com/etcd-io/bbolt/pull/659)
- [Optimize the logger performance for frequent called methods](https://github.com/etcd-io/bbolt/pull/741)
- [Stabilize the behaviour of Prev when the cursor already points to the first element](https://github.com/etcd-io/bbolt/pull/734)

### CMD
- [Fix `bbolt keys` and `bbolt get` to prevent them from panicking when no parameter provided](https://github.com/etcd-io/bbolt/pull/682)
- [Fix surgery freelist command in info logs](https://github.com/etcd-io/bbolt/pull/700)
- [Remove txid references in surgery meta command's comment and description](https://github.com/etcd-io/bbolt/pull/703)
- [Add rnd read capabilities to bbolt bench](https://github.com/etcd-io/bbolt/pull/711)
- [Use `cobra.ExactArgs` to simplify the argument number check](https://github.com/etcd-io/bbolt/pull/728)
- [Migrate `bbolt check` command to cobra style](https://github.com/etcd-io/bbolt/pull/723)
- [Simplify the naming of cobra commands](https://github.com/etcd-io/bbolt/pull/732)
- [Aggregate adding completed ops for read test of the `bbolt bench` command](https://github.com/etcd-io/bbolt/pull/721)
- [Add `--from-page` flag to `bbolt check` command](https://github.com/etcd-io/bbolt/pull/737)

### Document
- [Add document for a known issue on the writing a value with a length of 0](https://github.com/etcd-io/bbolt/pull/730)

### Test
- [Enhance robustness test to cover XFS](https://github.com/etcd-io/bbolt/pull/707)

### Other
- [Bump go toolchain version to 1.22.2](https://github.com/etcd-io/bbolt/pull/712)

<hr>

## v1.4.0-alpha.0(2024-01-12)

### BoltDB
- [Improve the performance of hashmapGetFreePageIDs](https://github.com/etcd-io/bbolt/pull/419)
- [Improve CreateBucketIfNotExists to avoid double searching the same key](https://github.com/etcd-io/bbolt/pull/532)
- [Support Android platform](https://github.com/etcd-io/bbolt/pull/571)
- [Record the count of free page to improve the performance of hashmapFreeCount](https://github.com/etcd-io/bbolt/pull/585)
- [Add logger to bbolt](https://github.com/etcd-io/bbolt/issues/509)
- [Support moving bucket inside the same db](https://github.com/etcd-io/bbolt/pull/635)
- [Support inspecting database structure](https://github.com/etcd-io/bbolt/pull/674)

### CMD
- [Add `surgery clear-page-elements` command](https://github.com/etcd-io/bbolt/pull/417)
- [Add `surgery abandon-freelist` command](https://github.com/etcd-io/bbolt/pull/443)
- [Add `bbolt version` command](https://github.com/etcd-io/bbolt/pull/552)
- [Add `bbolt inspect` command](https://github.com/etcd-io/bbolt/pull/674)
- [Add `--no-sync` option to `bbolt compact` command](https://github.com/etcd-io/bbolt/pull/290)
