# Memory Management

## Background

Pebble has two significant sources of memory usage: MemTables and the
Block Cache. MemTables buffer data that has been written to the WAL
but not yet flushed to an SSTable. The Block Cache provides a cache of
uncompressed SSTable data blocks.

Originally, Pebble used regular Go memory allocation for the memory
backing both MemTables and the Block Cache. This was problematic as it
put significant pressure on the Go GC. The higher the bandwidth of
memory allocations, the more work GC has to do to reclaim the
memory. In order to lessen the pressure on the Go GC, an "allocation
cache" was introduced to the Block Cache which allowed reusing the
memory backing cached blocks in most circumstances. This produced a
dramatic reduction in GC pressure and a measurable performance
improvement in CockroachDB workloads.

Unfortunately, the use of Go allocated memory still caused a
problem. CockroachDB running on top of Pebble often resulted in an RSS
(resident set size) 2x what it was when using RocksDB. The cause of
this effect is due to the Go runtime's heuristic for triggering GC:

> A collection is triggered when the ratio of freshly allocated data
> to live data remaining after the previous collection reaches this
> percentage.

This percentage can be configured by the
[`GOGC`](https://golang.org/pkg/runtime/) environment variable or by
calling
[`debug.SetGCPercent`](https://golang.org/pkg/runtime/debug/#SetGCPercent). The
default value is `100`, which means that GC is triggered when the
freshly allocated data is equal to the amount of live data at the end
of the last collection period. This generally works well in practice,
but the Pebble Block Cache is often configured to be 10s of gigabytes
in size. Waiting for 10s of gigabytes of data to be allocated before
triggering a GC results in very large Go heap sizes.

## Manual Memory Management

Attempting to adjust `GOGC` to account for the significant amount of
memory used by the Block Cache is fraught. What value should be used?
`10%`? `20%`? Should the setting be tuned dynamically? Rather than
introducing a heuristic which may have cascading effects on the
application using Pebble, we decided to move the Block Cache and
MemTable memory out of the Go heap. This is done by using the C memory
allocator, though it could also be done by providing a simple memory
allocator in Go which uses `mmap` to allocate memory.

In order to support manual memory management for the Block Cache and
MemTables, Pebble needs to precisely track their lifetime. This was
already being done for the MemTable in order to account for its memory
usage in metrics. It was mostly being done for the Block Cache. Values
stores in the Block Cache are reference counted and are returned to
the "alloc cache" when their reference count falls
to 0. Unfortunately, this tracking wasn't precise and there were
numerous cases where the cache values were being leaked. This was
acceptable in a world where the Go GC would clean up after us. It is
unacceptable if the leak becomes permanent.

## Leak Detection

In order to find all of the cache value leaks, Pebble has a leak
detection facility built on top of
[`runtime.SetFinalizer`](https://golang.org/pkg/runtime/#SetFinalizer). A
finalizer is a function associated with an object which is run when
the object is no longer reachable. On the surface, this sounds perfect
as a facility for performing all memory reclamation. Unfortunately,
finalizers are generally frowned upon by the Go implementors, and come
with very loose guarantees:

> The finalizer is scheduled to run at some arbitrary time after the
> program can no longer reach the object to which obj points. There is
> no guarantee that finalizers will run before a program exits, so
> typically they are useful only for releasing non-memory resources
> associated with an object during a long-running program

This language is somewhat frightening, but in practice finalizers are run at the
end of every GC period. Pebble primarily relies on finalizers for its leak
detection facility. In the block cache, a finalizer is associated with the Go
allocated `cache.Value` object. When the finalizer is run, it checks that the
buffer backing the `cache.Value` has been freed. This leak detection facility is
enabled by the `"invariants"` build tag which is enabled by the Pebble unit
tests.

There also exists a very specific memory reclamation use case in the block cache
that ensures that structs with transitively reachable fields backed by manually
allocated memory that are pooled in a `sync.Pool` are freed correctly when their
parent struct is released from the pool and consequently garbage collected by
the Go runtime (see `cache/entry_normal.go`). The loose guarantees provided by
the runtime are reasonable to rely on in this case to prevent a memory leak.
