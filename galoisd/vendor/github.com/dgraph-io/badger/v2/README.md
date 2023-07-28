# BadgerDB [![GoDoc](https://godoc.org/github.com/dgraph-io/badger?status.svg)](https://godoc.org/github.com/dgraph-io/badger) [![Go Report Card](https://goreportcard.com/badge/github.com/dgraph-io/badger)](https://goreportcard.com/report/github.com/dgraph-io/badger) [![Sourcegraph](https://sourcegraph.com/github.com/dgraph-io/badger/-/badge.svg)](https://sourcegraph.com/github.com/dgraph-io/badger?badge) [![Build Status](https://teamcity.dgraph.io/guestAuth/app/rest/builds/buildType:(id:Badger_UnitTests)/statusIcon.svg)](https://teamcity.dgraph.io/viewLog.html?buildTypeId=Badger_UnitTests&buildId=lastFinished&guest=1) ![Appveyor](https://ci.appveyor.com/api/projects/status/github/dgraph-io/badger?branch=master&svg=true) [![Coverage Status](https://coveralls.io/repos/github/dgraph-io/badger/badge.svg?branch=master)](https://coveralls.io/github/dgraph-io/badger?branch=master)

![Badger mascot](images/diggy-shadow.png)

BadgerDB is an embeddable, persistent and fast key-value (KV) database written
in pure Go. It is the underlying database for [Dgraph](https://dgraph.io), a
fast, distributed graph database. It's meant to be a performant alternative to
non-Go-based key-value stores like RocksDB.

## Project Status [March 24, 2020]

Badger is stable and is being used to serve data sets worth hundreds of
terabytes. Badger supports concurrent ACID transactions with serializable
snapshot isolation (SSI) guarantees. A Jepsen-style bank test runs nightly for
8h, with `--race` flag and ensures the maintenance of transactional guarantees.
Badger has also been tested to work with filesystem level anomalies, to ensure
persistence and consistency. Badger is being used by a number of projects which
includes Dgraph, Jaeger Tracing, UsenetExpress, and many more.

The list of projects using Badger can be found [here](#projects-using-badger).

Badger v1.0 was released in Nov 2017, and the latest version that is data-compatible
with v1.0 is v1.6.0.

Badger v2.0 was released in Nov 2019 with a new storage format which won't
be compatible with all of the v1.x. Badger v2.0 supports compression, encryption and uses a cache to speed up lookup.

The [Changelog] is kept fairly up-to-date.

For more details on our version naming schema please read [Choosing a version](#choosing-a-version).

[Changelog]:https://github.com/dgraph-io/badger/blob/master/CHANGELOG.md

## Table of Contents
 * [Getting Started](#getting-started)
    + [Installing](#installing)
      - [Choosing a version](#choosing-a-version)
    + [Opening a database](#opening-a-database)
    + [Transactions](#transactions)
      - [Read-only transactions](#read-only-transactions)
      - [Read-write transactions](#read-write-transactions)
      - [Managing transactions manually](#managing-transactions-manually)
    + [Using key/value pairs](#using-keyvalue-pairs)
    + [Monotonically increasing integers](#monotonically-increasing-integers)
    * [Merge Operations](#merge-operations)
    + [Setting Time To Live(TTL) and User Metadata on Keys](#setting-time-to-livettl-and-user-metadata-on-keys)
    + [Iterating over keys](#iterating-over-keys)
      - [Prefix scans](#prefix-scans)
      - [Key-only iteration](#key-only-iteration)
    + [Stream](#stream)
    + [Garbage Collection](#garbage-collection)
    + [Database backup](#database-backup)
    + [Memory usage](#memory-usage)
    + [Statistics](#statistics)
  * [Resources](#resources)
    + [Blog Posts](#blog-posts)
  * [Contact](#contact)
  * [Design](#design)
    + [Comparisons](#comparisons)
    + [Benchmarks](#benchmarks)
  * [Projects Using Badger](#projects-using-badger)
  * [Contributing](#contributing)
  * [Frequently Asked Questions](#frequently-asked-questions)

## Getting Started

### Installing
To start using Badger, install Go 1.12 or above and run `go get`:

```sh
$ go get github.com/dgraph-io/badger/v2
```

This will retrieve the library and install the `badger` command line
utility into your `$GOBIN` path.


#### Choosing a version

BadgerDB is a pretty special package from the point of view that the most important change we can
make to it is not on its API but rather on how data is stored on disk.

This is why we follow a version naming schema that differs from Semantic Versioning.

- New major versions are released when the data format on disk changes in an incompatible way.
- New minor versions are released whenever the API changes but data compatibility is maintained.
 Note that the changes on the API could be backward-incompatible - unlike Semantic Versioning.
- New patch versions are released when there's no changes to the data format nor the API.

Following these rules:

- v1.5.0 and v1.6.0 can be used on top of the same files without any concerns, as their major
 version is the same, therefore the data format on disk is compatible.
- v1.6.0 and v2.0.0 are data incompatible as their major version implies, so files created with
 v1.6.0 will need to be converted into the new format before they can be used by v2.0.0.

For a longer explanation on the reasons behind using a new versioning naming schema, you can read
[VERSIONING.md](VERSIONING.md).

### Opening a database
The top-level object in Badger is a `DB`. It represents multiple files on disk
in specific directories, which contain the data for a single database.

To open your database, use the `badger.Open()` function, with the appropriate
options. The `Dir` and `ValueDir` options are mandatory and must be
specified by the client. They can be set to the same value to simplify things.

```go
package main

import (
	"log"

	badger "github.com/dgraph-io/badger/v2"
)

func main() {
  // Open the Badger database located in the /tmp/badger directory.
  // It will be created if it doesn't exist.
  db, err := badger.Open(badger.DefaultOptions("/tmp/badger"))
  if err != nil {
	  log.Fatal(err)
  }
  defer db.Close()
  // Your code here…
}
```

Please note that Badger obtains a lock on the directories so multiple processes
cannot open the same database at the same time.

#### In-Memory Mode/Diskless Mode
By default, Badger ensures all the data is persisted to the disk. It also supports a pure
in-memory mode. When Badger is running in in-memory mode, all the data is stored in the memory.
Reads and writes are much faster in in-memory mode, but all the data stored in Badger will be lost
in case of a crash or close. To open badger in in-memory mode, set the `InMemory` option.

```
opt := badger.DefaultOptions("").WithInMemory(true)
```

### Transactions

#### Read-only transactions
To start a read-only transaction, you can use the `DB.View()` method:

```go
err := db.View(func(txn *badger.Txn) error {
  // Your code here…
  return nil
})
```

You cannot perform any writes or deletes within this transaction. Badger
ensures that you get a consistent view of the database within this closure. Any
writes that happen elsewhere after the transaction has started, will not be
seen by calls made within the closure.

#### Read-write transactions
To start a read-write transaction, you can use the `DB.Update()` method:

```go
err := db.Update(func(txn *badger.Txn) error {
  // Your code here…
  return nil
})
```

All database operations are allowed inside a read-write transaction.

Always check the returned error value. If you return an error
within your closure it will be passed through.

An `ErrConflict` error will be reported in case of a conflict. Depending on the state
of your application, you have the option to retry the operation if you receive
this error.

An `ErrTxnTooBig` will be reported in case the number of pending writes/deletes in
the transaction exceeds a certain limit. In that case, it is best to commit the
transaction and start a new transaction immediately. Here is an example (we are
not checking for errors in some places for simplicity):

```go
updates := make(map[string]string)
txn := db.NewTransaction(true)
for k,v := range updates {
  if err := txn.Set([]byte(k),[]byte(v)); err == badger.ErrTxnTooBig {
    _ = txn.Commit()
    txn = db.NewTransaction(true)
    _ = txn.Set([]byte(k),[]byte(v))
  }
}
_ = txn.Commit()
```

#### Managing transactions manually
The `DB.View()` and `DB.Update()` methods are wrappers around the
`DB.NewTransaction()` and `Txn.Commit()` methods (or `Txn.Discard()` in case of
read-only transactions). These helper methods will start the transaction,
execute a function, and then safely discard your transaction if an error is
returned. This is the recommended way to use Badger transactions.

However, sometimes you may want to manually create and commit your
transactions. You can use the `DB.NewTransaction()` function directly, which
takes in a boolean argument to specify whether a read-write transaction is
required. For read-write transactions, it is necessary to call `Txn.Commit()`
to ensure the transaction is committed. For read-only transactions, calling
`Txn.Discard()` is sufficient. `Txn.Commit()` also calls `Txn.Discard()`
internally to cleanup the transaction, so just calling `Txn.Commit()` is
sufficient for read-write transaction. However, if your code doesn’t call
`Txn.Commit()` for some reason (for e.g it returns prematurely with an error),
then please make sure you call `Txn.Discard()` in a `defer` block. Refer to the
code below.

```go
// Start a writable transaction.
txn := db.NewTransaction(true)
defer txn.Discard()

// Use the transaction...
err := txn.Set([]byte("answer"), []byte("42"))
if err != nil {
    return err
}

// Commit the transaction and check for error.
if err := txn.Commit(); err != nil {
    return err
}
```

The first argument to `DB.NewTransaction()` is a boolean stating if the transaction
should be writable.

Badger allows an optional callback to the `Txn.Commit()` method. Normally, the
callback can be set to `nil`, and the method will return after all the writes
have succeeded. However, if this callback is provided, the `Txn.Commit()`
method returns as soon as it has checked for any conflicts. The actual writing
to the disk happens asynchronously, and the callback is invoked once the
writing has finished, or an error has occurred. This can improve the throughput
of the application in some cases. But it also means that a transaction is not
durable until the callback has been invoked with a `nil` error value.

### Using key/value pairs
To save a key/value pair, use the `Txn.Set()` method:

```go
err := db.Update(func(txn *badger.Txn) error {
  err := txn.Set([]byte("answer"), []byte("42"))
  return err
})
```

Key/Value pair can also be saved by first creating `Entry`, then setting this
`Entry` using `Txn.SetEntry()`. `Entry` also exposes methods to set properties
on it.

```go
err := db.Update(func(txn *badger.Txn) error {
  e := badger.NewEntry([]byte("answer"), []byte("42"))
  err := txn.SetEntry(e)
  return err
})
```

This will set the value of the `"answer"` key to `"42"`. To retrieve this
value, we can use the `Txn.Get()` method:

```go
err := db.View(func(txn *badger.Txn) error {
  item, err := txn.Get([]byte("answer"))
  handle(err)

  var valNot, valCopy []byte
  err := item.Value(func(val []byte) error {
    // This func with val would only be called if item.Value encounters no error.

    // Accessing val here is valid.
    fmt.Printf("The answer is: %s\n", val)

    // Copying or parsing val is valid.
    valCopy = append([]byte{}, val...)

    // Assigning val slice to another variable is NOT OK.
    valNot = val // Do not do this.
    return nil
  })
  handle(err)

  // DO NOT access val here. It is the most common cause of bugs.
  fmt.Printf("NEVER do this. %s\n", valNot)

  // You must copy it to use it outside item.Value(...).
  fmt.Printf("The answer is: %s\n", valCopy)

  // Alternatively, you could also use item.ValueCopy().
  valCopy, err = item.ValueCopy(nil)
  handle(err)
  fmt.Printf("The answer is: %s\n", valCopy)

  return nil
})
```

`Txn.Get()` returns `ErrKeyNotFound` if the value is not found.

Please note that values returned from `Get()` are only valid while the
transaction is open. If you need to use a value outside of the transaction
then you must use `copy()` to copy it to another byte slice.

Use the `Txn.Delete()` method to delete a key.

### Monotonically increasing integers

To get unique monotonically increasing integers with strong durability, you can
use the `DB.GetSequence` method. This method returns a `Sequence` object, which
is thread-safe and can be used concurrently via various goroutines.

Badger would lease a range of integers to hand out from memory, with the
bandwidth provided to `DB.GetSequence`. The frequency at which disk writes are
done is determined by this lease bandwidth and the frequency of `Next`
invocations. Setting a bandwidth too low would do more disk writes, setting it
too high would result in wasted integers if Badger is closed or crashes.
To avoid wasted integers, call `Release` before closing Badger.

```go
seq, err := db.GetSequence(key, 1000)
defer seq.Release()
for {
  num, err := seq.Next()
}
```

### Merge Operations
Badger provides support for ordered merge operations. You can define a func
of type `MergeFunc` which takes in an existing value, and a value to be
_merged_ with it. It returns a new value which is the result of the _merge_
operation. All values are specified in byte arrays. For e.g., here is a merge
function (`add`) which appends a  `[]byte` value to an existing `[]byte` value.

```Go
// Merge function to append one byte slice to another
func add(originalValue, newValue []byte) []byte {
  return append(originalValue, newValue...)
}
```

This function can then be passed to the `DB.GetMergeOperator()` method, along
with a key, and a duration value. The duration specifies how often the merge
function is run on values that have been added using the `MergeOperator.Add()`
method.

`MergeOperator.Get()` method can be used to retrieve the cumulative value of the key
associated with the merge operation.

```Go
key := []byte("merge")

m := db.GetMergeOperator(key, add, 200*time.Millisecond)
defer m.Stop()

m.Add([]byte("A"))
m.Add([]byte("B"))
m.Add([]byte("C"))

res, _ := m.Get() // res should have value ABC encoded
```

Example: Merge operator which increments a counter

```Go
func uint64ToBytes(i uint64) []byte {
  var buf [8]byte
  binary.BigEndian.PutUint64(buf[:], i)
  return buf[:]
}

func bytesToUint64(b []byte) uint64 {
  return binary.BigEndian.Uint64(b)
}

// Merge function to add two uint64 numbers
func add(existing, new []byte) []byte {
  return uint64ToBytes(bytesToUint64(existing) + bytesToUint64(new))
}
```
It can be used as
```Go
key := []byte("merge")

m := db.GetMergeOperator(key, add, 200*time.Millisecond)
defer m.Stop()

m.Add(uint64ToBytes(1))
m.Add(uint64ToBytes(2))
m.Add(uint64ToBytes(3))

res, _ := m.Get() // res should have value 6 encoded
```

### Setting Time To Live(TTL) and User Metadata on Keys
Badger allows setting an optional Time to Live (TTL) value on keys. Once the TTL has
elapsed, the key will no longer be retrievable and will be eligible for garbage
collection. A TTL can be set as a `time.Duration` value using the `Entry.WithTTL()`
and `Txn.SetEntry()` API methods.

```go
err := db.Update(func(txn *badger.Txn) error {
  e := badger.NewEntry([]byte("answer"), []byte("42")).WithTTL(time.Hour)
  err := txn.SetEntry(e)
  return err
})
```

An optional user metadata value can be set on each key. A user metadata value
is represented by a single byte. It can be used to set certain bits along
with the key to aid in interpreting or decoding the key-value pair. User
metadata can be set using `Entry.WithMeta()` and `Txn.SetEntry()` API methods.

```go
err := db.Update(func(txn *badger.Txn) error {
  e := badger.NewEntry([]byte("answer"), []byte("42")).WithMeta(byte(1))
  err := txn.SetEntry(e)
  return err
})
```

`Entry` APIs can be used to add the user metadata and TTL for same key. This `Entry`
then can be set using `Txn.SetEntry()`.

```go
err := db.Update(func(txn *badger.Txn) error {
  e := badger.NewEntry([]byte("answer"), []byte("42")).WithMeta(byte(1)).WithTTL(time.Hour)
  err := txn.SetEntry(e)
  return err
})
```

### Iterating over keys
To iterate over keys, we can use an `Iterator`, which can be obtained using the
`Txn.NewIterator()` method. Iteration happens in byte-wise lexicographical sorting
order.


```go
err := db.View(func(txn *badger.Txn) error {
  opts := badger.DefaultIteratorOptions
  opts.PrefetchSize = 10
  it := txn.NewIterator(opts)
  defer it.Close()
  for it.Rewind(); it.Valid(); it.Next() {
    item := it.Item()
    k := item.Key()
    err := item.Value(func(v []byte) error {
      fmt.Printf("key=%s, value=%s\n", k, v)
      return nil
    })
    if err != nil {
      return err
    }
  }
  return nil
})
```

The iterator allows you to move to a specific point in the list of keys and move
forward or backward through the keys one at a time.

By default, Badger prefetches the values of the next 100 items. You can adjust
that with the `IteratorOptions.PrefetchSize` field. However, setting it to
a value higher than `GOMAXPROCS` (which we recommend to be 128 or higher)
shouldn’t give any additional benefits. You can also turn off the fetching of
values altogether. See section below on key-only iteration.

#### Prefix scans
To iterate over a key prefix, you can combine `Seek()` and `ValidForPrefix()`:

```go
db.View(func(txn *badger.Txn) error {
  it := txn.NewIterator(badger.DefaultIteratorOptions)
  defer it.Close()
  prefix := []byte("1234")
  for it.Seek(prefix); it.ValidForPrefix(prefix); it.Next() {
    item := it.Item()
    k := item.Key()
    err := item.Value(func(v []byte) error {
      fmt.Printf("key=%s, value=%s\n", k, v)
      return nil
    })
    if err != nil {
      return err
    }
  }
  return nil
})
```

#### Key-only iteration
Badger supports a unique mode of iteration called _key-only_ iteration. It is
several order of magnitudes faster than regular iteration, because it involves
access to the LSM-tree only, which is usually resident entirely in RAM. To
enable key-only iteration, you need to set the `IteratorOptions.PrefetchValues`
field to `false`. This can also be used to do sparse reads for selected keys
during an iteration, by calling `item.Value()` only when required.

```go
err := db.View(func(txn *badger.Txn) error {
  opts := badger.DefaultIteratorOptions
  opts.PrefetchValues = false
  it := txn.NewIterator(opts)
  defer it.Close()
  for it.Rewind(); it.Valid(); it.Next() {
    item := it.Item()
    k := item.Key()
    fmt.Printf("key=%s\n", k)
  }
  return nil
})
```

### Stream
Badger provides a Stream framework, which concurrently iterates over all or a
portion of the DB, converting data into custom key-values, and streams it out
serially to be sent over network, written to disk, or even written back to
Badger. This is a lot faster way to iterate over Badger than using a single
Iterator. Stream supports Badger in both managed and normal mode.

Stream uses the natural boundaries created by SSTables within the LSM tree, to
quickly generate key ranges. Each goroutine then picks a range and runs an
iterator to iterate over it. Each iterator iterates over all versions of values
and is created from the same transaction, thus working over a snapshot of the
DB. Every time a new key is encountered, it calls `ChooseKey(item)`, followed
by `KeyToList(key, itr)`. This allows a user to select or reject that key, and
if selected, convert the value versions into custom key-values. The goroutine
batches up 4MB worth of key-values, before sending it over to a channel.
Another goroutine further batches up data from this channel using *smart
batching* algorithm and calls `Send` serially.

This framework is designed for high throughput key-value iteration, spreading
the work of iteration across many goroutines. `DB.Backup` uses this framework to
provide full and incremental backups quickly.  Dgraph is a heavy user of this
framework.  In fact, this framework was developed and used within Dgraph, before
getting ported over to Badger.

```go
stream := db.NewStream()
// db.NewStreamAt(readTs) for managed mode.

// -- Optional settings
stream.NumGo = 16                     // Set number of goroutines to use for iteration.
stream.Prefix = []byte("some-prefix") // Leave nil for iteration over the whole DB.
stream.LogPrefix = "Badger.Streaming" // For identifying stream logs. Outputs to Logger.

// ChooseKey is called concurrently for every key. If left nil, assumes true by default.
stream.ChooseKey = func(item *badger.Item) bool {
  return bytes.HasSuffix(item.Key(), []byte("er"))
}

// KeyToList is called concurrently for chosen keys. This can be used to convert
// Badger data into custom key-values. If nil, uses stream.ToList, a default
// implementation, which picks all valid key-values.
stream.KeyToList = nil

// -- End of optional settings.

// Send is called serially, while Stream.Orchestrate is running.
stream.Send = func(list *pb.KVList) error {
  return proto.MarshalText(w, list) // Write to w.
}

// Run the stream
if err := stream.Orchestrate(context.Background()); err != nil {
  return err
}
// Done.
```

### Garbage Collection
Badger values need to be garbage collected, because of two reasons:

* Badger keeps values separately from the LSM tree. This means that the compaction operations
that clean up the LSM tree do not touch the values at all. Values need to be cleaned up
separately.

* Concurrent read/write transactions could leave behind multiple values for a single key, because they
are stored with different versions. These could accumulate, and take up unneeded space beyond the
time these older versions are needed.

Badger relies on the client to perform garbage collection at a time of their choosing. It provides
the following method, which can be invoked at an appropriate time:

* `DB.RunValueLogGC()`: This method is designed to do garbage collection while
  Badger is online. Along with randomly picking a file, it uses statistics generated by the
  LSM-tree compactions to pick files that are likely to lead to maximum space
  reclamation. It is recommended to be called during periods of low activity in
  your system, or periodically. One call would only result in removal of at max
  one log file. As an optimization, you could also immediately re-run it whenever
  it returns nil error (indicating a successful value log GC), as shown below.

	```go
	ticker := time.NewTicker(5 * time.Minute)
	defer ticker.Stop()
	for range ticker.C {
	again:
		err := db.RunValueLogGC(0.7)
		if err == nil {
			goto again
		}
	}
	```

* `DB.PurgeOlderVersions()`: This method is **DEPRECATED** since v1.5.0. Now, Badger's LSM tree automatically discards older/invalid versions of keys.

**Note: The RunValueLogGC method would not garbage collect the latest value log.**

### Database backup
There are two public API methods `DB.Backup()` and `DB.Load()` which can be
used to do online backups and restores. Badger v0.9 provides a CLI tool
`badger`, which can do offline backup/restore. Make sure you have `$GOPATH/bin`
in your PATH to use this tool.

The command below will create a version-agnostic backup of the database, to a
file `badger.bak` in the current working directory

```
badger backup --dir <path/to/badgerdb>
```

To restore `badger.bak` in the current working directory to a new database:

```
badger restore --dir <path/to/badgerdb>
```

See `badger --help` for more details.

If you have a Badger database that was created using v0.8 (or below), you can
use the `badger_backup` tool provided in v0.8.1, and then restore it using the
command above to upgrade your database to work with the latest version.

```
badger_backup --dir <path/to/badgerdb> --backup-file badger.bak
```

We recommend all users to use the `Backup` and `Restore` APIs and tools. However,
Badger is also rsync-friendly because all files are immutable, barring the
latest value log which is append-only. So, rsync can be used as rudimentary way
to perform a backup. In the following script, we repeat rsync to ensure that the
LSM tree remains consistent with the MANIFEST file while doing a full backup.

```
#!/bin/bash
set -o history
set -o histexpand
# Makes a complete copy of a Badger database directory.
# Repeat rsync if the MANIFEST and SSTables are updated.
rsync -avz --delete db/ dst
while !! | grep -q "(MANIFEST\|\.sst)$"; do :; done
```

### Memory usage
Badger's memory usage can be managed by tweaking several options available in
the `Options` struct that is passed in when opening the database using
`DB.Open`.

- `Options.ValueLogLoadingMode` can be set to `options.FileIO` (instead of the
  default `options.MemoryMap`) to avoid memory-mapping log files. This can be
  useful in environments with low RAM.
- Number of memtables (`Options.NumMemtables`)
  - If you modify `Options.NumMemtables`, also adjust `Options.NumLevelZeroTables` and
   `Options.NumLevelZeroTablesStall` accordingly.
- Number of concurrent compactions (`Options.NumCompactors`)
- Mode in which LSM tree is loaded (`Options.TableLoadingMode`)
- Size of table (`Options.MaxTableSize`)
- Size of value log file (`Options.ValueLogFileSize`)

If you want to decrease the memory usage of Badger instance, tweak these
options (ideally one at a time) until you achieve the desired
memory usage.

### Statistics
Badger records metrics using the [expvar] package, which is included in the Go
standard library. All the metrics are documented in [y/metrics.go][metrics]
file.

`expvar` package adds a handler in to the default HTTP server (which has to be
started explicitly), and serves up the metrics at the `/debug/vars` endpoint.
These metrics can then be collected by a system like [Prometheus], to get
better visibility into what Badger is doing.

[expvar]: https://golang.org/pkg/expvar/
[metrics]: https://github.com/dgraph-io/badger/blob/master/y/metrics.go
[Prometheus]: https://prometheus.io/

## Resources

### Blog Posts
1. [Introducing Badger: A fast key-value store written natively in
Go](https://open.dgraph.io/post/badger/)
2. [Make Badger crash resilient with ALICE](https://blog.dgraph.io/post/alice/)
3. [Badger vs LMDB vs BoltDB: Benchmarking key-value databases in Go](https://blog.dgraph.io/post/badger-lmdb-boltdb/)
4. [Concurrent ACID Transactions in Badger](https://blog.dgraph.io/post/badger-txn/)

## Design
Badger was written with these design goals in mind:

- Write a key-value database in pure Go.
- Use latest research to build the fastest KV database for data sets spanning terabytes.
- Optimize for SSDs.

Badger’s design is based on a paper titled _[WiscKey: Separating Keys from
Values in SSD-conscious Storage][wisckey]_.

[wisckey]: https://www.usenix.org/system/files/conference/fast16/fast16-papers-lu.pdf

### Comparisons
| Feature                        | Badger                                     | RocksDB                       | BoltDB    |
| -------                        | ------                                     | -------                       | ------    |
| Design                         | LSM tree with value log                    | LSM tree only                 | B+ tree   |
| High Read throughput           | Yes                                        | No                            | Yes       |
| High Write throughput          | Yes                                        | Yes                           | No        |
| Designed for SSDs              | Yes (with latest research <sup>1</sup>)    | Not specifically <sup>2</sup> | No        |
| Embeddable                     | Yes                                        | Yes                           | Yes       |
| Sorted KV access               | Yes                                        | Yes                           | Yes       |
| Pure Go (no Cgo)               | Yes                                        | No                            | Yes       |
| Transactions                   | Yes, ACID, concurrent with SSI<sup>3</sup> | Yes (but non-ACID)            | Yes, ACID |
| Snapshots                      | Yes                                        | Yes                           | Yes       |
| TTL support                    | Yes                                        | Yes                           | No        |
| 3D access (key-value-version)  | Yes<sup>4</sup>                            | No                            | No        |

<sup>1</sup> The [WISCKEY paper][wisckey] (on which Badger is based) saw big
wins with separating values from keys, significantly reducing the write
amplification compared to a typical LSM tree.

<sup>2</sup> RocksDB is an SSD optimized version of LevelDB, which was designed specifically for rotating disks.
As such RocksDB's design isn't aimed at SSDs.

<sup>3</sup> SSI: Serializable Snapshot Isolation. For more details, see the blog post [Concurrent ACID Transactions in Badger](https://blog.dgraph.io/post/badger-txn/)

<sup>4</sup> Badger provides direct access to value versions via its Iterator API.
Users can also specify how many versions to keep per key via Options.

### Benchmarks
We have run comprehensive benchmarks against RocksDB, Bolt and LMDB. The
benchmarking code, and the detailed logs for the benchmarks can be found in the
[badger-bench] repo. More explanation, including graphs can be found the blog posts (linked
above).

[badger-bench]: https://github.com/dgraph-io/badger-bench

## Projects Using Badger
Below is a list of known projects that use Badger:

* [Dgraph](https://github.com/dgraph-io/dgraph) - Distributed graph database.
* [Jaeger](https://github.com/jaegertracing/jaeger) - Distributed tracing platform.
* [go-ipfs](https://github.com/ipfs/go-ipfs) - Go client for the InterPlanetary File System (IPFS), a new hypermedia distribution protocol.
* [Riot](https://github.com/go-ego/riot) - An open-source, distributed search engine.
* [emitter](https://github.com/emitter-io/emitter) - Scalable, low latency, distributed pub/sub broker with message storage, uses MQTT, gossip and badger.
* [OctoSQL](https://github.com/cube2222/octosql) - Query tool that allows you to join, analyse and transform data from multiple databases using SQL.
* [Dkron](https://dkron.io/) - Distributed, fault tolerant job scheduling system.
* [Sandglass](https://github.com/celrenheit/sandglass) - distributed, horizontally scalable, persistent, time sorted message queue.
* [TalariaDB](https://github.com/grab/talaria) - Grab's Distributed, low latency time-series database.
* [Sloop](https://github.com/salesforce/sloop) - Salesforce's Kubernetes History Visualization Project.
* [Immudb](https://github.com/codenotary/immudb) - Lightweight, high-speed immutable database for systems and applications.
* [Usenet Express](https://usenetexpress.com/) - Serving over 300TB of data with Badger.
* [gorush](https://github.com/appleboy/gorush) - A push notification server written in Go.
* [0-stor](https://github.com/zero-os/0-stor) - Single device object store.
* [Dispatch Protocol](https://github.com/dispatchlabs/disgo) - Blockchain protocol for distributed application data analytics.
* [GarageMQ](https://github.com/valinurovam/garagemq) - AMQP server written in Go.
* [RedixDB](https://alash3al.github.io/redix/) - A real-time persistent key-value store with the same redis protocol.
* [BBVA](https://github.com/BBVA/raft-badger) - Raft backend implementation using BadgerDB for Hashicorp raft.
* [Fantom](https://github.com/Fantom-foundation/go-lachesis) - aBFT Consensus platform for distributed applications.
* [decred](https://github.com/decred/dcrdata) - An open, progressive, and self-funding cryptocurrency with a system of community-based governance integrated into its blockchain.
* [OpenNetSys](https://github.com/opennetsys/c3-go) - Create useful dApps in any software language.
* [HoneyTrap](https://github.com/honeytrap/honeytrap) - An extensible and opensource system for running, monitoring and managing honeypots.
* [Insolar](https://github.com/insolar/insolar) - Enterprise-ready blockchain platform.
* [IoTeX](https://github.com/iotexproject/iotex-core) - The next generation of the decentralized network for IoT powered by scalability- and privacy-centric blockchains.
* [go-sessions](https://github.com/kataras/go-sessions) - The sessions manager for Go net/http and fasthttp.
* [Babble](https://github.com/mosaicnetworks/babble) - BFT Consensus platform for distributed applications.
* [Tormenta](https://github.com/jpincas/tormenta) - Embedded object-persistence layer / simple JSON database for Go projects.
* [BadgerHold](https://github.com/timshannon/badgerhold) - An embeddable NoSQL store for querying Go types built on Badger
* [Goblero](https://github.com/didil/goblero) - Pure Go embedded persistent job queue backed by BadgerDB
* [Surfline](https://www.surfline.com) - Serving global wave and weather forecast data with Badger.
* [Cete](https://github.com/mosuka/cete) - Simple and highly available distributed key-value store built on Badger. Makes it easy bringing up a cluster of Badger with Raft consensus algorithm by hashicorp/raft. 
* [Volument](https://volument.com/) - A new take on website analytics backed by Badger.
* [KVdb](https://kvdb.io/) - Hosted key-value store and serverless platform built on top of Badger.

If you are using Badger in a project please send a pull request to add it to the list.

## Contributing

If you're interested in contributing to Badger see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Frequently Asked Questions
### My writes are getting stuck. Why?

**Update: With the new `Value(func(v []byte))` API, this deadlock can no longer
happen.**

The following is true for users on Badger v1.x.

This can happen if a long running iteration with `Prefetch` is set to false, but
a `Item::Value` call is made internally in the loop. That causes Badger to
acquire read locks over the value log files to avoid value log GC removing the
file from underneath. As a side effect, this also blocks a new value log GC
file from being created, when the value log file boundary is hit.

Please see Github issues [#293](https://github.com/dgraph-io/badger/issues/293)
and [#315](https://github.com/dgraph-io/badger/issues/315).

There are multiple workarounds during iteration:

1. Use `Item::ValueCopy` instead of `Item::Value` when retrieving value.
1. Set `Prefetch` to true. Badger would then copy over the value and release the
   file lock immediately.
1. When `Prefetch` is false, don't call `Item::Value` and do a pure key-only
   iteration. This might be useful if you just want to delete a lot of keys.
1. Do the writes in a separate transaction after the reads.

### My writes are really slow. Why?

Are you creating a new transaction for every single key update, and waiting for
it to `Commit` fully before creating a new one? This will lead to very low
throughput.

We have created `WriteBatch` API which provides a way to batch up
many updates into a single transaction and `Commit` that transaction using
callbacks to avoid blocking. This amortizes the cost of a transaction really
well, and provides the most efficient way to do bulk writes.

```go
wb := db.NewWriteBatch()
defer wb.Cancel()

for i := 0; i < N; i++ {
  err := wb.Set(key(i), value(i), 0) // Will create txns as needed.
  handle(err)
}
handle(wb.Flush()) // Wait for all txns to finish.
```

Note that `WriteBatch` API does not allow any reads. For read-modify-write
workloads, you should be using the `Transaction` API.

### I don't see any disk writes. Why?

If you're using Badger with `SyncWrites=false`, then your writes might not be written to value log
and won't get synced to disk immediately. Writes to LSM tree are done inmemory first, before they
get compacted to disk. The compaction would only happen once `MaxTableSize` has been reached. So, if
you're doing a few writes and then checking, you might not see anything on disk. Once you `Close`
the database, you'll see these writes on disk.

### Reverse iteration doesn't give me the right results.

Just like forward iteration goes to the first key which is equal or greater than the SEEK key, reverse iteration goes to the first key which is equal or lesser than the SEEK key. Therefore, SEEK key would not be part of the results. You can typically add a `0xff` byte as a suffix to the SEEK key to include it in the results. See the following issues: [#436](https://github.com/dgraph-io/badger/issues/436) and [#347](https://github.com/dgraph-io/badger/issues/347).

### Which instances should I use for Badger?

We recommend using instances which provide local SSD storage, without any limit
on the maximum IOPS. In AWS, these are storage optimized instances like i3. They
provide local SSDs which clock 100K IOPS over 4KB blocks easily.

### I'm getting a closed channel error. Why?

```
panic: close of closed channel
panic: send on closed channel
```

If you're seeing panics like above, this would be because you're operating on a closed DB. This can happen, if you call `Close()` before sending a write, or multiple times. You should ensure that you only call `Close()` once, and all your read/write operations finish before closing.

### Are there any Go specific settings that I should use?

We *highly* recommend setting a high number for `GOMAXPROCS`, which allows Go to
observe the full IOPS throughput provided by modern SSDs. In Dgraph, we have set
it to 128. For more details, [see this
thread](https://groups.google.com/d/topic/golang-nuts/jPb_h3TvlKE/discussion).

### Are there any Linux specific settings that I should use?

We recommend setting `max file descriptors` to a high number depending upon the expected size of
your data. On Linux and Mac, you can check the file descriptor limit with `ulimit -n -H` for the
hard limit and `ulimit -n -S` for the soft limit. A soft limit of `65535` is a good lower bound.
You can adjust the limit as needed.

### I see "manifest has unsupported version: X (we support Y)" error.

This error means you have a badger directory which was created by an older version of badger and
you're trying to open in a newer version of badger. The underlying data format can change across
badger versions and users will have to migrate their data directory.
Badger data can be migrated from version X of badger to version Y of badger by following the steps
listed below.
Assume you were on badger v1.6.0 and you wish to migrate to v2.0.0 version.
1. Install badger version v1.6.0
    - `cd $GOPATH/src/github.com/dgraph-io/badger`
    - `git checkout v1.6.0`
    - `cd badger && go install`

      This should install the old badger binary in your $GOBIN.
2. Create Backup
    - `badger backup --dir path/to/badger/directory -f badger.backup`
3. Install badger version v2.0.0
    - `cd $GOPATH/src/github.com/dgraph-io/badger`
    - `git checkout v2.0.0`
    - `cd badger && go install`

      This should install new badger binary in your $GOBIN
4. Install badger version v2.0.0
    - `badger restore --dir path/to/new/badger/directory -f badger.backup`

      This will create a new directory on `path/to/new/badger/directory` and add badger data in
      newer format to it.

NOTE - The above steps shouldn't cause any data loss but please ensure the new data is valid before
deleting the old badger directory.

### Why do I need gcc to build badger? Does badger need CGO?

Badger does not directly use CGO but it relies on https://github.com/DataDog/zstd library for
zstd compression and the library requires `gcc/cgo`. You can build badger without cgo by running
`CGO_ENABLED=0 go build`. This will build badger without the support for ZSTD compression algorithm.

## Contact
- Please use [discuss.dgraph.io](https://discuss.dgraph.io) for questions, feature requests and discussions.
- Please use [Github issue tracker](https://github.com/dgraph-io/badger/issues) for filing bugs or feature requests.
- Join [![Slack Status](http://slack.dgraph.io/badge.svg)](http://slack.dgraph.io).
- Follow us on Twitter [@dgraphlabs](https://twitter.com/dgraphlabs).

