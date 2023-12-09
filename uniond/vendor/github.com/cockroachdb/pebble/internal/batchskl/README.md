# batchskl

Fast, non-concurrent skiplist implementation in Go that supports
forward and backward iteration.

## Limitations

* The interface is tailored for use in indexing pebble batches. Keys
  and values are stored outside of the skiplist making the skiplist
  awkward for general purpose use.
* Deletion is not supported. Instead, higher-level code is expected to
  add deletion tombstones and needs to process those tombstones
  appropriately.

## Pedigree

This code is based on Andy Kimball's arenaskl code.

The arenaskl code is based on the skiplist found in Badger, a Go-based
KV store:

https://github.com/dgraph-io/badger/tree/master/skl

The skiplist in Badger is itself based on a C++ skiplist built for
Facebook's RocksDB:

https://github.com/facebook/rocksdb/tree/master/memtable

## Benchmarks

The benchmarks consist of a mix of reads and writes executed in parallel. The
fraction of reads is indicated in the run name: "frac_X" indicates a run where
X percent of the operations are reads.

```
name                  time/op
ReadWrite/frac_0      1.03µs ± 2%
ReadWrite/frac_10     1.32µs ± 1%
ReadWrite/frac_20     1.26µs ± 1%
ReadWrite/frac_30     1.18µs ± 1%
ReadWrite/frac_40     1.09µs ± 1%
ReadWrite/frac_50      987ns ± 2%
ReadWrite/frac_60     1.07µs ± 1%
ReadWrite/frac_70      909ns ± 1%
ReadWrite/frac_80      693ns ± 2%
ReadWrite/frac_90      599ns ± 2%
ReadWrite/frac_100    45.3ns ± 3%
```

Forward and backward iteration are also fast:

```
name                  time/op
IterNext              4.49ns ± 3%
IterPrev              4.48ns ± 3%
```
