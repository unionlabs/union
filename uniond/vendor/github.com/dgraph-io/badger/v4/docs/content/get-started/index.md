+++ 
title = "Get Started - Quickstart Guide" 
aliases = ["/get-started"] 
+++


## Installing
To start using Badger, install Go 1.19 or above. Run the following command to retrieve the library.

```sh
$ go get github.com/dgraph-io/badger/v4
```
This will retrieve the library.

### Installing Badger Command Line Tool

```sh
$ go install github.com/dgraph-io/badger/v4/badger@latest
```
This will install the badger command line utility into your $GOBIN path.

## Opening a database
The top-level object in Badger is a `DB`. It represents multiple files on disk
in specific directories, which contain the data for a single database.

To open your database, use the `badger.Open()` function, with the appropriate
options. The `Dir` and `ValueDir` options are mandatory and must be
specified by the client. They can be set to the same value to simplify things.

```go
package main

import (
	"log"

	badger "github.com/dgraph-io/badger/v4"
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

### In-Memory Mode/Diskless Mode
By default, Badger ensures all the data is persisted to the disk. It also supports a pure
in-memory mode. When Badger is running in in-memory mode, all the data is stored in the memory.
Reads and writes are much faster in in-memory mode, but all the data stored in Badger will be lost
in case of a crash or close. To open badger in in-memory mode, set the `InMemory` option.

```go
opt := badger.DefaultOptions("").WithInMemory(true)
```

### Encryption Mode

If you enable encryption on Badger, you also need to set the index cache size.

{{% notice "tip" %}}
Having a cache improves the performance. Otherwise, your reads would be very slow while encryption is enabled.
{{% /notice %}}

For example, to set a `100 Mb` cache:

```go
opts.IndexCache = 100 << 20 // 100 mb or some other size based on the amount of data
```

## Transactions

### Read-only transactions
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

### Read-write transactions
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

### Managing transactions manually
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

## Using key/value pairs
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

## Monotonically increasing integers

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

## Merge Operations
Badger provides support for ordered merge operations. You can define a func
of type `MergeFunc` which takes in an existing value, and a value to be
_merged_ with it. It returns a new value which is the result of the _merge_
operation. All values are specified in byte arrays. For e.g., here is a merge
function (`add`) which appends a  `[]byte` value to an existing `[]byte` value.

```go
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

```go
key := []byte("merge")

m := db.GetMergeOperator(key, add, 200*time.Millisecond)
defer m.Stop()

m.Add([]byte("A"))
m.Add([]byte("B"))
m.Add([]byte("C"))

res, _ := m.Get() // res should have value ABC encoded
```

Example: Merge operator which increments a counter

```go
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
```
key := []byte("merge")

m := db.GetMergeOperator(key, add, 200*time.Millisecond)
defer m.Stop()

m.Add(uint64ToBytes(1))
m.Add(uint64ToBytes(2))
m.Add(uint64ToBytes(3))

res, _ := m.Get() // res should have value 6 encoded
```

## Setting Time To Live(TTL) and User Metadata on Keys
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

## Iterating over keys
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

### Prefix scans
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

### Key-only iteration
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

## Stream
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

## Garbage Collection
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

{{% notice "note" %}} The RunValueLogGC method would not garbage collect the latest value log.{{% /notice %}}

## Database backup
There are two public API methods `DB.Backup()` and `DB.Load()` which can be
used to do online backups and restores. Badger v0.9 provides a CLI tool
`badger`, which can do offline backup/restore. Make sure you have `$GOPATH/bin`
in your PATH to use this tool.

The command below will create a version-agnostic backup of the database, to a
file `badger.bak` in the current working directory

```sh
badger backup --dir <path/to/badgerdb>
```

To restore `badger.bak` in the current working directory to a new database:

```sh
badger restore --dir <path/to/badgerdb>
```

See `badger --help` for more details.

If you have a Badger database that was created using v0.8 (or below), you can
use the `badger_backup` tool provided in v0.8.1, and then restore it using the
command above to upgrade your database to work with the latest version.

```sh
badger_backup --dir <path/to/badgerdb> --backup-file badger.bak
```

We recommend all users to use the `Backup` and `Restore` APIs and tools. However,
Badger is also rsync-friendly because all files are immutable, barring the
latest value log which is append-only. So, rsync can be used as rudimentary way
to perform a backup. In the following script, we repeat rsync to ensure that the
LSM tree remains consistent with the MANIFEST file while doing a full backup.

```sh
#!/bin/bash
set -o history
set -o histexpand
# Makes a complete copy of a Badger database directory.
# Repeat rsync if the MANIFEST and SSTables are updated.
rsync -avz --delete db/ dst
while !! | grep -q "(MANIFEST\|\.sst)$"; do :; done
```

## Memory usage
Badger's memory usage can be managed by tweaking several options available in
the `Options` struct that is passed in when opening the database using
`DB.Open`.

- Number of memtables (`Options.NumMemtables`)
  - If you modify `Options.NumMemtables`, also adjust `Options.NumLevelZeroTables` and
   `Options.NumLevelZeroTablesStall` accordingly.
- Number of concurrent compactions (`Options.NumCompactors`)
- Size of table (`Options.BaseTableSize`)
- Size of value log file (`Options.ValueLogFileSize`)

If you want to decrease the memory usage of Badger instance, tweak these
options (ideally one at a time) until you achieve the desired
memory usage.
