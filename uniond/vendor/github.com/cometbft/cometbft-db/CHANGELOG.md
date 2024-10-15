# CHANGELOG

## v1.0.1

*September 23, 2024*

This release reverts the addition of the `goleveldb` flag, which was deemed as
too disruptive to users.

## v1.0.0

*September 20, 2024*

This release swaps the "default" DB from goleveldb to pebbledb. There's now a
`goleveldb` build flag that must be used when using goleveldb. If you're using
`pebbledb`, you don't need a build flag anymore.

### BREAKING

- Add `goleveldb` build flag.
  ([\#202](https://github.com/cometbft/cometbft-db/pull/202))

## v0.15.0

*September 9, 2024*

This release bumps the Go version to 1.23.

### BREAKING CHANGES

- `[go/runtime]` Bump minimum Go version to v1.23
  ([\#4039](https://github.com/cometbft/cometbft/issues/4039))

## v0.14.0

*Aug 9, 2024*

This release reinstates boltdb and cleveldb as deprecated backend types.
Please note that we discourage the use of them, as we plan to discontinue support in a future release.

### DEPENDENCIES

- reinstate BoltDB and ClevelDB as backend DBs
  ([\#177](https://github.com/cometbft/cometbft-db/pull/177))

## v0.13.0

*Aug 2, 2024*

This release:
- changes the contract of the Iterator Key() and Value() APIs. Namely, the caller is now responsible for creating a copy of their returned value if they want to modify it.
- removes support for boltDB and clevelDB, which were marked as deprecated in release v0.12.0.

### BREAKING CHANGES

- removed deprecated boltdb and cleveldb ([\#155](https://github.com/cometbft/cometbft-db/pull/155))

### FEATURES

- Iterator Key and Value APIs now return an object that must be copied before
  use ([\#168](https://github.com/cometbft/cometbft-db/pull/168))

## v0.12.0

*Apr 10, 2024*

This release deprecates boltdb and cleveldb. Also, Go MSRV is bumped to 1.22.

### FEATURES

- Deprecate boltdb and cleveldb. If you're using either of those, please reach
  out ([\#153](https://github.com/cometbft/cometbft-db/pull/153))

## v0.11.0

*Feb 7, 2024*

This release adds support for explicit compaction. Please note that badger and
bolt do not support this.

### BREAKING CHANGES

- Expanded db interface to support compaction ([\#111](https://github.com/cometbft/cometbft-db/pull/111))

### FEATURES

- Add compaction support to the databases ([\#111](https://github.com/cometbft/cometbft-db/pull/111))

## v0.10.0

*Jan 26, 2024*

This release adds experimental support for
[pebble](https://github.com/cockroachdb/pebble) and drops `remotedb`. If you
experience any issues with pebble, please open an issue on Github.

Special thanks to @faddat and @baabeetaa for their contributions to this
release!

### BREAKING CHANGES

- Remove remotedb ([\#121](https://github.com/cometbft/cometbft-db/pull/121))

### FEATURES

- Add [pebbledb](https://github.com/cockroachdb/pebble) ([\#112](https://github.com/cometbft/cometbft-db/pull/112))

## v0.9.1

*December 4, 2023*

This release is precisely the same code-wise as v0.9.0, except that it builds
the `cometbft/cometbft-db-testing` Docker image for both `linux/amd64` and
`linux/arm64` platforms.

## v0.9.0

*December 1, 2023*

This release primarily updates some key dependencies, including adding support
for RocksDB v8. It also bumps the minimum Go version to v1.21 in order for
CometBFT to be able to use it in the E2E testing framework for the latest major
releases.

### DEPENDENCIES

- Use RocksDB v8, testing with v8.8.1
  ([\#97](https://github.com/cometbft/cometbft-db/pull/97))

### GO VERSION

- Bump minimum Go version to v1.21
  ([\#98](https://github.com/cometbft/cometbft-db/pull/98))

## v0.8.0

*Apr 26, 2023*

This release bumps the supported version of RocksDB, which requires cometbft-db
RocksDB users to update their builds (and hence requires a "major" release, but
does not introduce any other breaking changes). Special thanks to @yihuang for
this update!

While the minimum supported version of the Go compiler was bumped to 1.19, no
1.19-specific code changes were introduced and this should, therefore, still be
able to be compiled with earlier versions of Go. It is, however, recommended to
upgrade to the latest version(s) of Go ASAP.

### COMPILER

- Bump minimum Go version to 1.19
  ([\#40](https://github.com/cometbft/cometbft-db/pull/40))

### DEPENDENCIES

- Switch rocksdb binding from gorocksdb to grocksdb, bump librocksdb dependency
  to `v7.10.2` ([\#42](https://github.com/cometbft/cometbft-db/pull/42))
- Update to the latest version of golang.org/x/net
  ([\#40](https://github.com/cometbft/cometbft-db/pull/40))

## v0.7.0

*Jan 17, 2023*

This is the first official release of CometBFT DB, which is a fork of
[tm-db](https://github.com/tendermint/tm-db).

This fork is intended to be used by
[CometBFT](https://github.com/cometbft/cometbft) until such time that
[cometbft/cometbft\#48](https://github.com/cometbft/cometbft/issues/48) is
resolved, after which time this fork will be retired and archived. Do not use
this as a dependency in any new projects.

### BREAKING CHANGES

- Fork tm-db and rename fork to cometbft-db
  ([\#7](https://github.com/cometbft/cometbft-db/issues/7))

---

CometBFT DB is a fork of [tm-db](https://github.com/tendermint/tm-db)
effectively as of v0.6.6.

For changes prior to the creation of this fork, please refer to the upstream
[CHANGELOG.md](https://github.com/tendermint/tm-db/blob/774cdfe7e6b0a249b1144998d81a4de7b8037941/CHANGELOG.md)
for v0.6.6 and earlier.

