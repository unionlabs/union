# Cosmos DB

[![version](https://img.shields.io/github/tag/cosmos/cosmos-db.svg)](https://github.com/cosmos/cosmos-db/releases/latest)
[![license](https://img.shields.io/github/license/cosmos/cosmos-db.svg)](https://github.com/cosmos/cosmos-db/blob/master/LICENSE)
[![API Reference](https://camo.githubusercontent.com/915b7be44ada53c290eb157634330494ebe3e30a/68747470733a2f2f676f646f632e6f72672f6769746875622e636f6d2f676f6c616e672f6764646f3f7374617475732e737667)](https://pkg.go.dev/github.com/cosmos/cosmos-db)
[![codecov](https://codecov.io/gh/cosmos/cosmos-db/branch/master/graph/badge.svg)](https://codecov.io/gh/cosmos/cosmos-db)
![Lint](https://github.com/cosmos/cosmos-db/workflows/Lint/badge.svg?branch=master)
![Test](https://github.com/cosmos/cosmos-db/workflows/Test/badge.svg?branch=master)
[![Discord chat](https://img.shields.io/discord/669268347736686612.svg)](https://discord.gg/cosmosnetwork)

Common database interface for various database backends. Primarily meant for applications built on [Tendermint](https://github.com/tendermint/tendermint), such as the [Cosmos SDK](https://github.com/cosmos/cosmos-sdk), but can be used independently of these as well.

### Minimum Go Version

Go 1.19+

## Supported Database Backends

- **MemDB [stable]:** An in-memory database using [Google's B-tree package](https://github.com/google/btree). Has very high performance both for reads, writes, and range scans, but is not durable and will lose all data on process exit. Does not support transactions. Suitable for e.g. caches, working sets, and tests. Used for [IAVL](https://github.com/tendermint/iavl) working sets when the pruning strategy allows it.

- **[GoLevelDB](https://github.com/syndtr/goleveldb)**: a pure Go implementation of [LevelDB](https://github.com/google/leveldb) (see below). Currently the default on-disk database used in the Cosmos SDK.

- **[LevelDB](https://github.com/google/leveldb)** using [levigo Go wrapper](https://github.com/jmhodges/levigo). Uses LSM-trees for on-disk storage, which have good performance for write-heavy workloads, particularly on spinning disks, but requires periodic compaction to maintain decent read performance and reclaim disk space. Does not support transactions.

- **[RocksDB](https://github.com/cosmos/gorocksdb):** A [Go wrapper](https://github.com/cosmos/gorocksdb) around [RocksDB](https://rocksdb.org). Similarly to LevelDB (above) it uses LSM-trees for on-disk storage, but is optimized for fast storage media such as SSDs and memory. Supports atomic transactions, but not full ACID transactions.

- **[Pebble](https://github.com/cockroachdb/pebble):** a RocksDB/LevelDB inspired key-value database in Go using RocksDB file format and LSM-trees for on-disk storage. Supports snapshots.

## Meta-databases

- **PrefixDB [stable]:** A database which wraps another database and uses a static prefix for all keys. This allows multiple logical databases to be stored in a common underlying databases by using different namespaces. Used by the Cosmos SDK to give different modules their own namespaced database in a single application database.

## Tests

To test common databases, run `make test`. If all databases are available on the local machine, use `make test-all` to test them all.
