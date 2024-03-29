# Changelog

## [v1.0.2] - 2024-02-26

* Downgrade Go version in go.mod to 1.19

## [v1.0.1] - 2024-02-25

## [v1.0.0] - 2023-05-25

> Note this repository was forked from [github.com/tendermint/tm-db](https://github.com/tendermint/tm-db). Minor modifications were made after the fork to better support the Cosmos SDK. Notably, this repo removes badger, boltdb and cleveldb.

* added bloom filter:  <https://github.com/cosmos/cosmos-db/pull/42/files>
* Removed Badger & Boltdb
* Add `NewBatchWithSize` to `DB` interface: <https://github.com/cosmos/cosmos-db/pull/64>
* Add `NewRocksDBWithRaw` to support different rocksdb open mode (read-only, secondary-standby).
