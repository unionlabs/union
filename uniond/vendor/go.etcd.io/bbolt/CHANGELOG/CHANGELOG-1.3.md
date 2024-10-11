Note that we start to track changes starting from v1.3.7.

<hr>

## v1.3.10(2024-05-06)

### BoltDB
- [Remove deprecated `UnsafeSlice` and use `unsafe.Slice`](https://github.com/etcd-io/bbolt/pull/717)
- [Stabilize the behaviour of Prev when the cursor already points to the first element](https://github.com/etcd-io/bbolt/pull/744)

### Other
- [Bump go version to 1.21.9](https://github.com/etcd-io/bbolt/pull/713)

<hr>

## v1.3.9(2024-02-24)

### BoltDB
- [Clone the key before operating data in bucket against the key](https://github.com/etcd-io/bbolt/pull/639)

### CMD
- [Fix `bbolt keys` and `bbolt get` to prevent them from panicking when no parameter provided](https://github.com/etcd-io/bbolt/pull/683)

<hr>

## v1.3.8(2023-10-26)

### BoltDB
- Fix [db.close() doesn't unlock the db file if db.munnmap() fails](https://github.com/etcd-io/bbolt/pull/439).
- [Avoid syscall.Syscall use on OpenBSD](https://github.com/etcd-io/bbolt/pull/406).
- Fix [rollback panicking after mlock failed or both meta pages corrupted](https://github.com/etcd-io/bbolt/pull/444).
- Fix [bbolt panicking due to 64bit unaligned on arm32](https://github.com/etcd-io/bbolt/pull/584).

### CMD
- [Update the usage of surgery command](https://github.com/etcd-io/bbolt/pull/411).

<hr>

## v1.3.7(2023-01-31)

### BoltDB
- Add [recursive checker to confirm database consistency](https://github.com/etcd-io/bbolt/pull/225).
- Add [support to get the page size from the second meta page if the first one is invalid](https://github.com/etcd-io/bbolt/pull/294).
- Add [support for loong64 arch](https://github.com/etcd-io/bbolt/pull/303).
- Add [internal iterator to Bucket that goes over buckets](https://github.com/etcd-io/bbolt/pull/356).
- Add [validation on page read and write](https://github.com/etcd-io/bbolt/pull/358).
- Add [PreLoadFreelist option to support loading free pages in readonly mode](https://github.com/etcd-io/bbolt/pull/381).
- Add [(*Tx) CheckWithOption to support generating human-readable diagnostic messages](https://github.com/etcd-io/bbolt/pull/395).
- Fix [Use `golang.org/x/sys/windows` for `FileLockEx`/`UnlockFileEx`](https://github.com/etcd-io/bbolt/pull/283).
- Fix [readonly file mapping on windows](https://github.com/etcd-io/bbolt/pull/307).
- Fix [the "Last" method might return no data due to not skipping the empty pages](https://github.com/etcd-io/bbolt/pull/341).
- Fix [panic on db.meta when rollback](https://github.com/etcd-io/bbolt/pull/362).

### CMD
- Add [support for get keys in sub buckets in `bbolt get` command](https://github.com/etcd-io/bbolt/pull/295).
- Add [support for `--format` flag for `bbolt keys` command](https://github.com/etcd-io/bbolt/pull/306).
- Add [safeguards to bbolt CLI commands](https://github.com/etcd-io/bbolt/pull/354).
- Add [`bbolt page` supports --all and --value-format=redacted formats](https://github.com/etcd-io/bbolt/pull/359).
- Add [`bbolt surgery` commands](https://github.com/etcd-io/bbolt/issues/370).
- Fix [open db file readonly mode for commands which shouldn't update the db file](https://github.com/etcd-io/bbolt/pull/365), see also [pull/292](https://github.com/etcd-io/bbolt/pull/292).

### Other
- [Build bbolt CLI tool, test and format the source code using golang 1.17.13](https://github.com/etcd-io/bbolt/pull/297).
- [Bump golang.org/x/sys to v0.4.0](https://github.com/etcd-io/bbolt/pull/397).

### Summary
Release v1.3.7 contains following critical fixes:
- fix to problem that `Last` method might return incorrect value ([#341](https://github.com/etcd-io/bbolt/pull/341))
- fix of potential panic when performing transaction's rollback ([#362](https://github.com/etcd-io/bbolt/pull/362))

Other changes focused on defense-in-depth ([#358](https://github.com/etcd-io/bbolt/pull/358), [#294](https://github.com/etcd-io/bbolt/pull/294), [#225](https://github.com/etcd-io/bbolt/pull/225), [#395](https://github.com/etcd-io/bbolt/pull/395))

`bbolt` command line tool was expanded to:
- allow fixing simple corruptions by `bbolt surgery` ([#370](https://github.com/etcd-io/bbolt/pull/370))
- be flexible about output formatting ([#306](https://github.com/etcd-io/bbolt/pull/306), [#359](https://github.com/etcd-io/bbolt/pull/359))
- allow accessing data in subbuckets ([#295](https://github.com/etcd-io/bbolt/pull/295))
