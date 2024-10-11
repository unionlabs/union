+++ 
title = "Frequently Asked Question" 
aliases = ["/faq"] 
+++

## My writes are getting stuck. Why?

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

## My writes are really slow. Why?

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

## I don't see any disk writes. Why?

If you're using Badger with `SyncWrites=false`, then your writes might not be written to value log
and won't get synced to disk immediately. Writes to LSM tree are done inmemory first, before they
get compacted to disk. The compaction would only happen once `BaseTableSize` has been reached. So, if
you're doing a few writes and then checking, you might not see anything on disk. Once you `Close`
the database, you'll see these writes on disk.

## Reverse iteration doesn't give me the right results.

Just like forward iteration goes to the first key which is equal or greater than the SEEK key, reverse iteration goes to the first key which is equal or lesser than the SEEK key. Therefore, SEEK key would not be part of the results. You can typically add a `0xff` byte as a suffix to the SEEK key to include it in the results. See the following issues: [#436](https://github.com/dgraph-io/badger/issues/436) and [#347](https://github.com/dgraph-io/badger/issues/347).

## Which instances should I use for Badger?

We recommend using instances which provide local SSD storage, without any limit
on the maximum IOPS. In AWS, these are storage optimized instances like i3. They
provide local SSDs which clock 100K IOPS over 4KB blocks easily.

## I'm getting a closed channel error. Why?

```
panic: close of closed channel
panic: send on closed channel
```

If you're seeing panics like above, this would be because you're operating on a closed DB. This can happen, if you call `Close()` before sending a write, or multiple times. You should ensure that you only call `Close()` once, and all your read/write operations finish before closing.

## Are there any Go specific settings that I should use?

We *highly* recommend setting a high number for `GOMAXPROCS`, which allows Go to
observe the full IOPS throughput provided by modern SSDs. In Dgraph, we have set
it to 128. For more details, [see this
thread](https://groups.google.com/d/topic/golang-nuts/jPb_h3TvlKE/discussion).

## Are there any Linux specific settings that I should use?

We recommend setting `max file descriptors` to a high number depending upon the expected size of
your data. On Linux and Mac, you can check the file descriptor limit with `ulimit -n -H` for the
hard limit and `ulimit -n -S` for the soft limit. A soft limit of `65535` is a good lower bound.
You can adjust the limit as needed.

## I see "manifest has unsupported version: X (we support Y)" error.

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
4. Restore data from backup
    - `badger restore --dir path/to/new/badger/directory -f badger.backup`

      This will create a new directory on `path/to/new/badger/directory` and add badger data in
      newer format to it.

NOTE - The above steps shouldn't cause any data loss but please ensure the new data is valid before
deleting the old badger directory.

## Why do I need gcc to build badger? Does badger need CGO?

Badger does not directly use CGO but it relies on https://github.com/DataDog/zstd library for
zstd compression and the library requires `gcc/cgo`. You can build badger without cgo by running
`CGO_ENABLED=0 go build`. This will build badger without the support for ZSTD compression algorithm.

As of Badger versions [v2.2007.4](https://github.com/dgraph-io/badger/releases/tag/v2.2007.4) and
[v3.2103.1](https://github.com/dgraph-io/badger/releases/tag/v3.2103.1) the DataDog ZSTD library
was replaced by pure Golang version and CGO is no longer required. The new library is
[backwards compatible in nearly all cases](https://discuss.dgraph.io/t/use-pure-go-zstd-implementation/8670/10):

	> Yes they are compatible both ways. The only exception is 0 bytes of input which will give
	> 0 bytes output with the Go zstd. But you already have the zstd.WithZeroFrames(true) which
	> will wrap 0 bytes in a header so it can be fed to DD zstd. This will of course only be relevant
	> when downgrading.
