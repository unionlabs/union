# Pebble vs RocksDB: Implementation Differences

RocksDB is a key-value store implemented using a Log-Structured
Merge-Tree (LSM). This document is not a primer on LSMs. There exist
some decent
[introductions](http://www.benstopford.com/2015/02/14/log-structured-merge-trees/)
on the web, or try chapter 3 of [Designing Data-Intensive
Applications](https://www.amazon.com/Designing-Data-Intensive-Applications-Reliable-Maintainable/dp/1449373321).

Pebble inherits the RocksDB file formats, has a similar API, and
shares many implementation details, but it also has many differences
that improve performance, reduce implementation complexity, or extend
functionality. This document highlights some of the more important
differences.

* [Internal Keys](#internal-keys)
* [Indexed Batches](#indexed-batches)
* [Large Batches](#large-batches)
* [Commit Pipeline](#commit-pipeline)
* [Range Deletions](#range-deletions)
* [Flush and Compaction Pacing](#flush-and-compaction-pacing)
* [Write Throttling](#write-throttling)
* [Other Differences](#other-differences)

## Internal Keys

The external RocksDB API accepts keys and values. Due to the LSM
structure, keys are never updated in place, but overwritten with new
versions. Inside RocksDB, these versioned keys are known as Internal
Keys. An Internal Key is composed of the user specified key, a
sequence number and a kind. On disk, sstables always store Internal
Keys.

```
  +-------------+------------+----------+
  | UserKey (N) | SeqNum (7) | Kind (1) |
  +-------------+------------+----------+
```

The `Kind` field indicates the type of key: set, merge, delete, etc.

While Pebble inherits the Internal Key encoding for format
compatibility, it diverges from RocksDB in how it manages Internal
Keys in its implementation. In RocksDB, Internal Keys are represented
either in encoded form (as a string) or as a `ParsedInternalKey`. The
latter is a struct with the components of the Internal Key as three
separate fields.

```c++
struct ParsedInternalKey {
  Slice  user_key;
  uint64 seqnum;
  uint8  kind;
}
```

The component format is convenient: changing the `SeqNum` or `Kind` is
field assignment. Extracting the `UserKey` is a field
reference. However, RocksDB tends to only use `ParsedInternalKey`
locally. The major internal APIs, such as `InternalIterator`, operate
using encoded internal keys (i.e. strings) for parameters and return
values.

To give a concrete example of the overhead this causes, consider
`Iterator::Seek(user_key)`. The external `Iterator` is implemented on
top of an `InternalIterator`. `Iterator::Seek` ends up calling
`InternalIterator::Seek`. Both Seek methods take a key, but
`InternalIterator::Seek` expects an encoded Internal Key. This is both
error prone and expensive. The key passed to `Iterator::Seek` needs to
be copied into a temporary string in order to append the `SeqNum` and
`Kind`. In Pebble, Internal Keys are represented in memory using an
`InternalKey` struct that is the analog of `ParsedInternalKey`. All
internal APIs use `InternalKeys`, with the exception of the lowest
level routines for decoding data from sstables. In Pebble, since the
interfaces all take and return the `InternalKey` struct, we don’t need
to allocate to construct the Internal Key from the User Key, but
RocksDB sometimes needs to allocate, and encode (i.e. make a
copy). The use of the encoded form also causes RocksDB to pass encoded
keys to the comparator routines, sometimes decoding the keys multiple
times during the course of processing.

## Indexed Batches

In RocksDB, a batch is the unit for all write operations. Even writing
a single key is transformed internally to a batch. The batch internal
representation is a contiguous byte buffer with a fixed 12-byte
header, followed by a series of records.

```
  +------------+-----------+--- ... ---+
  | SeqNum (8) | Count (4) |  Entries  |
  +------------+-----------+--- ... ---+
```

Each record has a 1-byte kind tag prefix, followed by 1 or 2 length
prefixed strings (varstring):

```
  +----------+-----------------+-------------------+
  | Kind (1) | Key (varstring) | Value (varstring) |
  +----------+-----------------+-------------------+
```

(The `Kind` indicates if there are 1 or 2 varstrings. `Set`, `Merge`,
and `DeleteRange` have 2 varstrings, while `Delete` has 1.)

Adding a mutation to a batch involves appending a new record to the
buffer. This format is extremely fast for writes, but the lack of
indexing makes it untenable to use directly for reads. In order to
support iteration, a separate indexing structure is created. Both
RocksDB and Pebble use a skiplist for the indexing structure, but with
a clever twist. Rather than the skiplist storing a copy of the key, it
simply stores the offset of the record within the mutation buffer. The
result is that the skiplist acts a multi-map (i.e. a map that can have
duplicate entries for a given key). The iteration order for this map
is constructed so that records sort on key, and for equal keys they
sort on descending offset. Newer records for the same key appear
before older records.

While the indexing structure for batches is nearly identical between
RocksDB and Pebble, how the index structure is used is completely
different. In RocksDB, a batch is indexed using the
`WriteBatchWithIndex` class. The `WriteBatchWithIndex` class provides
a `NewIteratorWithBase` method that allows iteration over the merged
view of the batch contents and an underlying "base" iterator created
from the database. `BaseDeltaIterator` contains logic to iterate over
the batch entries and the base iterator in parallel which allows us to
perform reads on a snapshot of the database as though the batch had
been applied to it. On the surface this sounds reasonable, yet the
implementation is incomplete. Merge and DeleteRange operations are not
supported. The reason they are not supported is because handling them
is complex and requires duplicating logic that already exists inside
RocksDB for normal iterator processing.

Pebble takes a different approach to iterating over a merged view of a
batch's contents and the underlying database: it treats the batch as
another level in the LSM. Recall that an LSM is composed of zero or
more memtable layers and zero or more sstable layers. Internally, both
RocksDB and Pebble contain a `MergingIterator` that knows how to merge
the operations from different levels, including processing overwritten
keys, merge operations, and delete range operations. The challenge
with treating the batch as another level to be used by a
`MergingIterator` is that the records in a batch do not have a
sequence number. The sequence number in the batch header is not
assigned until the batch is committed. The solution is to give the
batch records temporary sequence numbers. We need these temporary
sequence numbers to be larger than any other sequence number in the
database so that the records in the batch are considered newer than
any committed record. This is accomplished by reserving the high-bit
in the 56-bit sequence number for use as a marker for batch sequence
numbers. The sequence number for a record in an uncommitted batch is:

```
  RecordOffset | (1<<55)
```

Newer records in a given batch will have a larger sequence number than
older records in the batch. And all of the records in a batch will
have larger sequence numbers than any committed record in the
database.

The end result is that Pebble's batch iterators support all of the
functionality of regular database iterators with minimal additional
code.

## Large Batches

The size of a batch is limited only by available memory, yet the
required memory is not just the batch representation. When a batch is
committed, the commit operation iterates over the records in the batch
from oldest to newest and inserts them into the current memtable. The
memtable is an in-memory structure that buffers mutations that have
been committed (written to the Write Ahead Log), but not yet written
to an sstable. Internally, a memtable uses a skiplist to index
records. Each skiplist entry has overhead for the index links and
other metadata that is a dozen bytes at minimum. A large batch
composed of many small records can require twice as much memory when
inserted into a memtable than it required in the batch. And note that
this causes a temporary increase in memory requirements because the
batch memory is not freed until it is completely committed.

A non-obvious implementation restriction present in both RocksDB and
Pebble is that there is a one-to-one correspondence between WAL files
and memtables. That is, a given WAL file has a single memtable
associated with it and vice-versa. While this restriction could be
removed, doing so is onerous and intricate. It should also be noted
that committing a batch involves writing it to a single WAL file. The
combination of restrictions results in a batch needing to be written
entirely to a single memtable.

What happens if a batch is too large to fit in a memtable?  Memtables
are generally considered to have a fixed size, yet this is not
actually true in RocksDB. In RocksDB, the memtable skiplist is
implemented on top of an arena structure. An arena is composed of a
list of fixed size chunks, with no upper limit set for the number of
chunks that can be associated with an arena. So RocksDB handles large
batches by allowing a memtable to grow beyond its configured
size. Concretely, while RocksDB may be configured with a 64MB memtable
size, a 1GB batch will cause the memtable to grow to accomodate
it. Functionally, this is good, though there is a practical problem: a
large batch is first written to the WAL, and then added to the
memtable. Adding the large batch to the memtable may consume so much
memory that the system runs out of memory and is killed by the
kernel. This can result in a death loop because upon restarting as the
batch is read from the WAL and applied to the memtable again.

In Pebble, the memtable is also implemented using a skiplist on top of
an arena. Significantly, the Pebble arena is a fixed size. While the
RocksDB skiplist uses pointers, the Pebble skiplist uses offsets from
the start of the arena. The fixed size arena means that the Pebble
memtable cannot expand arbitrarily. A batch that is too large to fit
in the memtable causes the current mutable memtable to be marked as
immutable and the batch is wrapped in a `flushableBatch` structure and
added to the list of immutable memtables. Because the `flushableBatch`
is readable as another layer in the LSM, the batch commit can return
as soon as the `flushableBatch` has been added to the immutable
memtable list.

Internally, a `flushableBatch` provides iterator support by sorting
the batch contents (the batch is sorted once, when it is added to the
memtable list). Sorting the batch contents and insertion of the
contents into a memtable have the same big-O time, but the constant
factor dominates here. Sorting is significantly faster and uses
significantly less memory due to not having to copy the batch records.

Note that an effect of this large batch support is that Pebble can be
configured as an efficient on-disk sorter: specify a small memtable
size, disable the WAL, and set a large L0 compaction threshold. In
order to sort a large amount of data, create batches that are larger
than the memtable size and commit them. When committed these batches
will not be inserted into a memtable, but instead sorted and then
written out to L0. The fully sorted data can later be read and the
normal merging process will take care of the final ordering.

## Commit Pipeline

The commit pipeline is the component which manages the steps in
committing write batches, such as writing the batch to the WAL and
applying its contents to the memtable. While simple conceptually, the
commit pipeline is crucial for high performance. In the absence of
concurrency, commit performance is limited by how fast a batch can be
written (and synced) to the WAL and then added to the memtable, both
of which are outside of the purview of the commit pipeline.

To understand the challenge here, it is useful to have a conception of
the WAL (write-ahead log). The WAL contains a record of all of the
batches that have been committed to the database. As a record is
written to the WAL it is added to the memtable. Each record is
assigned a sequence number which is used to distinguish newer updates
from older ones. Conceptually the WAL looks like:

```
+--------------------------------------+
| Batch(SeqNum=1,Count=9,Records=...)  |
+--------------------------------------+
| Batch(SeqNum=10,Count=5,Records=...) |
+--------------------------------------+
| Batch(SeqNum=15,Count=7,Records...)  |
+--------------------------------------+
| ...                                  |
+--------------------------------------+
```

Note that each WAL entry is precisely the batch representation
described earlier in the [Indexed Batches](#indexed-batches)
section. The monotonically increasing sequence numbers are a critical
component in allowing RocksDB and Pebble to provide fast snapshot
views of the database for reads.

If concurrent performance was not a concern, the commit pipeline could
simply be a mutex which serialized writes to the WAL and application
of the batch records to the memtable. Concurrent performance is a
concern, though.

The primary challenge in concurrent performance in the commit pipeline
is maintaining two invariants:

1. Batches need to be written to the WAL in sequence number order.
2. Batches need to be made visible for reads in sequence number
   order. This invariant arises from the use of a single sequence
   number which indicates which mutations are visible.

The second invariant deserves explanation. RocksDB and Pebble both
keep track of a visible sequence number. This is the sequence number
for which records in the database are visible during reads. The
visible sequence number exists because committing a batch is an atomic
operation, yet adding records to the memtable is done without an
exclusive lock (the skiplists used by both Pebble and RocksDB are
lock-free). When the records from a batch are being added to the
memtable, a concurrent read operation may see those records, but will
skip over them because they are newer than the visible sequence
number. Once all of the records in the batch have been added to the
memtable, the visible sequence number is atomically incremented.

So we have four steps in committing a write batch:

1. Write the batch to the WAL
2. Apply the mutations in the batch to the memtable
3. Bump the visible sequence number
4. (Optionally) sync the WAL

Writing the batch to the WAL is actually very fast as it is just a
memory copy. Applying the mutations in the batch to the memtable is by
far the most CPU intensive part of the commit pipeline. Syncing the
WAL is the most expensive from a wall clock perspective.

With that background out of the way, let's examine how RocksDB commits
batches. This description is of the traditional commit pipeline in
RocksDB (i.e. the one used by CockroachDB).

RocksDB achieves concurrency in the commit pipeline by grouping
concurrently committed batches into a batch group. Each group is
assigned a "leader" which is the first batch to be added to the
group. The batch group is written atomically to the WAL by the leader
thread, and then the individual batches making up the group are
concurrently applied to the memtable. Lastly, the visible sequence
number is bumped such that all of the batches in the group become
visible in a single atomic step. While a batch group is being applied,
other concurrent commits are added to a waiting list. When the group
commit finishes, the waiting commits form the next group.

There are two criticisms of the batch grouping approach. The first is
that forming a batch group involves copying batch contents. RocksDB
partially alleviates this for large batches by placing a limit on the
total size of a group. A large batch will end up in its own group and
not be copied, but the criticism still applies for small batches. Note
that there are actually two copies here. The batch contents are
concatenated together to form the group, and then the group contents
are written into an in memory buffer for the WAL before being written
to disk.

The second criticism is about the thread synchronization points. Let's
consider what happens to a commit which becomes the leader:

1. Lock commit mutex
2. Wait to become leader
3. Form (concatenate) batch group and write to the WAL
4. Notify followers to apply their batch to the memtable
5. Apply own batch to memtable
6. Wait for followers to finish
7. Bump visible sequence number
8. Unlock commit mutex
9. Notify followers that the commit is complete

The follower's set of operations looks like:

1. Lock commit mutex
2. Wait to become follower
3. Wait to be notified that it is time to apply batch
4. Unlock commit mutex
5. Apply batch to memtable
6. Wait to be notified that commit is complete

The thread synchronization points (all of the waits and notifies) are
overhead. Reducing that overhead can improve performance.

The Pebble commit pipeline addresses both criticisms. The main
innovation is a commit queue that mirrors the commit order. The Pebble
commit pipeline looks like:

1. Lock commit mutex
  * Add batch to commit queue
  * Assign batch sequence number
  * Write batch to the WAL
2. Unlock commit mutex
3. Apply batch to memtable (concurrently)
4. Publish batch sequence number

Pebble does not use the concept of a batch group. Each batch is
individually written to the WAL, but note that the WAL write is just a
memory copy into an internal buffer in the WAL.

Step 4 deserves further scrutiny as it is where the invariant on the
visible batch sequence number is maintained. Publishing the batch
sequence number cannot simply bump the visible sequence number because
batches with earlier sequence numbers may still be applying to the
memtable. If we were to ratchet the visible sequence number without
waiting for those applies to finish, a concurrent reader could see
partial batch contents. Note that RocksDB has experimented with
allowing these semantics with its unordered writes option.

We want to retain the atomic visibility of batch commits. The publish
batch sequence number step needs to ensure that we don't ratchet the
visible sequence number until all batches with earlier sequence
numbers have applied. Enter the commit queue: a lock-free
single-producer, multi-consumer queue. Batches are added to the commit
queue with the commit mutex held, ensuring the same order as the
sequence number assignment. After a batch finishes applying to the
memtable, it atomically marks the batch as applied. It then removes
the prefix of applied batches from the commit queue, bumping the
visible sequence number, and marking the batch as committed (via a
`sync.WaitGroup`). If the first batch in the commit queue has not be
applied we wait for our batch to be committed, relying on another
concurrent committer to perform the visible sequence ratcheting for
our batch. We know a concurrent commit is taking place because if
there was only one batch committing it would be at the head of the
commit queue.

There are two possibilities when publishing a sequence number. The
first is that there is an unapplied batch at the head of the
queue. Consider the following scenario where we're trying to publish
the sequence number for batch `B`.

```
  +---------------+-------------+---------------+-----+
  | A (unapplied) | B (applied) | C (unapplied) | ... |
  +---------------+-------------+---------------+-----+
```

The publish routine will see that `A` is unapplied and then simply
wait for `B's` done `sync.WaitGroup` to be signalled. This is safe
because `A` must still be committing. And if `A` has concurrently been
marked as applied, the goroutine publishing `A` will then publish
`B`. What happens when `A` publishes its sequence number? The commit
queue state becomes:

```
  +-------------+-------------+---------------+-----+
  | A (applied) | B (applied) | C (unapplied) | ... |
  +-------------+-------------+---------------+-----+
```

The publish routine pops `A` from the queue, ratchets the sequence
number, then pops `B` and ratchets the sequence number again, and then
finds `C` and stops. A detail that it is important to notice is that
the committer for batch `B` didn't have to do any more work. An
alternative approach would be to have `B` wakeup and ratchet its own
sequence number, but that would serialize the remainder of the commit
queue behind that goroutine waking up.

The commit queue reduces the number of thread synchronization
operations required to commit a batch. There is no leader to notify,
or followers to wait for. A commit either publishes its own sequence
number, or performs one synchronization operation to wait for a
concurrent committer to publish its sequence number.

## Range Deletions

Deletion of an individual key in RocksDB and Pebble is accomplished by
writing a deletion tombstone. A deletion tombstone shadows an existing
value for a key, causing reads to treat the key as not present. The
deletion tombstone mechanism works well for deleting small sets of
keys, but what happens if you want to all of the keys within a range
of keys that might number in the thousands or millions? A range
deletion is an operation which deletes an entire range of keys with a
single record. In contrast to a point deletion tombstone which
specifies a single key, a range deletion tombstone (a.k.a. range
tombstone) specifies a start key (inclusive) and an end key
(exclusive). This single record is much faster to write than thousands
or millions of point deletion tombstones, and can be done blindly --
without iterating over the keys that need to be deleted. The downside
to range tombstones is that they require additional processing during
reads. How the processing of range tombstones is done significantly
affects both the complexity of the implementation, and the efficiency
of read operations in the presence of range tombstones.

A range tombstone is composed of a start key, end key, and sequence
number. Any key that falls within the range is considered deleted if
the key's sequence number is less than the range tombstone's sequence
number. RocksDB stores range tombstones segregated from point
operations in a special range deletion block within each sstable.
Conceptually, the range tombstones stored within an sstable are
truncated to the boundaries of the sstable, though there are
complexities that cause this to not actually be physically true.

In RocksDB, the main structure implementing range tombstone processing
is the `RangeDelAggregator`. Each read operation and iterator has its
own `RangeDelAggregator` configured for the sequence number the read
is taking place at. The initial implementation of `RangeDelAggregator`
built up a "skyline" for the range tombstones visible at the read
sequence number.

```
10   +---+
 9   |   |
 8   |   |
 7   |   +----+
 6   |        |
 5 +-+        |  +----+
 4 |          |  |    |
 3 |          |  |    +---+
 2 |          |  |        |
 1 |          |  |        |
 0 |          |  |        |
  abcdefghijklmnopqrstuvwxyz
```

The above diagram shows the skyline created for the range tombstones
`[b,j)#5`, `[d,h)#10`, `[f,m)#7`, `[p,u)#5`, and `[t,y)#3`. The
skyline is queried for each key read to see if the key should be
considered deleted or not. The skyline structure is stored in a binary
tree, making the queries an O(logn) operation in the number of
tombstones, though there is an optimization to make this O(1) for
`next`/`prev` iteration. Note that the skyline representation loses
information about the range tombstones. This requires the structure to
be rebuilt on every read which has a significant performance impact.

The initial skyline range tombstone implementation has since been
replaced with a more efficient lookup structure. See the
[DeleteRange](https://rocksdb.org/blog/2018/11/21/delete-range.html)
blog post for a good description of both the original implementation
and the new (v2) implementation. The key change in the new
implementation is to "fragment" the range tombstones that are stored
in an sstable. The fragmented range tombstones provide the same
benefit as the skyline representation: the ability to binary search
the fragments in order to find the tombstone covering a key. But
unlike the skyline approach, the fragmented tombstones can be cached
on a per-sstable basis. In the v2 approach, `RangeDelAggregator` keeps
track of the fragmented range tombstones for each sstable encountered
during a read or iterator, and logically merges them together.

Fragmenting range tombstones involves splitting range tombstones at
overlap points. Let's consider the tombstones in the skyline example
above:

```
10:   d---h
 7:     f------m
 5: b-------j     p----u
 3:                   t----y
```

Fragmenting the range tombstones at the overlap points creates a
larger number of range tombstones:

```
10:   d-f-h
 7:     f-h-j--m
 5: b-d-f-h-j     p---tu
 3:                   tu---y
```

While the number of tombstones is larger there is a significant
advantage: we can order the tombstones by their start key and then
binary search to find the set of tombstones overlapping a particular
point. This is possible because due to the fragmenting, all the
tombstones that overlap a range of keys will have the same start and
end key. The v2 `RangeDelAggregator` and associated classes perform
fragmentation of range tombstones stored in each sstable and those
fragmented tombstones are then cached.

In summary, in RocksDB `RangeDelAggregator` acts as an oracle for
answering whether a key is deleted at a particular sequence
number. Due to caching of fragmented tombstones, the v2 implementation
of `RangeDelAggregator` implementation is significantly faster to
populate than v1, yet the overall approach to processing range
tombstones remains similar.

Pebble takes a different approach: it integrates range tombstones
processing directly into the `mergingIter` structure. `mergingIter` is
the internal structure which provides a merged view of the levels in
an LSM. RocksDB has a similar class named
`MergingIterator`. Internally, `mergingIter` maintains a heap over the
levels in the LSM (note that each memtable and L0 table is a separate
"level" in `mergingIter`). In RocksDB, `MergingIterator` knows nothing
about range tombstones, and it is thus up to higher-level code to
process range tombstones using `RangeDelAggregator`.

While the separation of `MergingIterator` and range tombstones seems
reasonable at first glance, there is an optimization that RocksDB does
not perform which is awkward with the `RangeDelAggregator` approach:
skipping swaths of deleted keys. A range tombstone often shadows more
than one key. Rather than iterating over the deleted keys, it is much
quicker to seek to the end point of the range tombstone. The challenge
in implementing this optimization is that a key might be newer than
the range tombstone and thus shouldn't be skipped. An insight to be
utilized is that the level structure itself provides sufficient
information. A range tombstone at `Ln` is guaranteed to be newer than
any key it overlaps in `Ln+1`.

Pebble utilizes the insight above to integrate range deletion
processing with `mergingIter`. A `mergingIter` maintains a point
iterator and a range deletion iterator per level in the LSM. In this
context, every L0 table is a separate level, as is every
memtable. Within a level, when a range deletion contains a point
operation the sequence numbers must be checked to determine if the
point operation is newer or older than the range deletion
tombstone. The `mergingIter` maintains the invariant that the range
deletion iterators for all levels newer that the current iteration key
are positioned at the next (or previous during reverse iteration)
range deletion tombstone. We know those levels don't contain a range
deletion tombstone that covers the current key because if they did the
current key would be deleted. The range deletion iterator for the
current key's level is positioned at a range tombstone covering or
past the current key. The position of all of other range deletion
iterators is unspecified. Whenever a key from those levels becomes the
current key, their range deletion iterators need to be
positioned. This lazy positioning avoids seeking the range deletion
iterators for keys that are never considered.

For a full example, consider the following setup:

```
  p0:               o
  r0:             m---q

  p1:              n p
  r1:       g---k

  p2:  b d    i
  r2: a---e           q----v

  p3:     e
  r3:
```

The diagram above shows is showing 4 levels, with `pX` indicating the
point operations in a level and `rX` indicating the range tombstones.

If we start iterating from the beginning, the first key we encounter
is `b` in `p2`. When the mergingIter is pointing at a valid entry, the
range deletion iterators for all of the levels less that the current
key's level are positioned at the next range tombstone past the
current key. So `r0` will point at `[m,q)` and `r1` at `[g,k)`. When
the key `b` is encountered, we check to see if the current tombstone
for `r0` or `r1` contains it, and whether the tombstone for `r2`,
`[a,e)`, contains and is newer than `b`.

Advancing the iterator finds the next key at `d`. This is in the same
level as the previous key `b` so we don't have to reposition any of
the range deletion iterators, but merely check whether `d` is now
contained by any of the range tombstones at higher levels or has
stepped past the range tombstone in its own level. In this case, there
is nothing to be done.

Advancing the iterator again finds `e`. Since `e` comes from `p3`, we
have to position the `r3` range deletion iterator, which is empty. `e`
is past the `r2` tombstone of `[a,e)` so we need to advance the `r2`
range deletion iterator to `[q,v)`.

The next key is `i`. Because this key is in `p2`, a level above `e`,
we don't have to reposition any range deletion iterators and instead
see that `i` is covered by the range tombstone `[g,k)`. The iterator
is immediately advanced to `n` which is covered by the range tombstone
`[m,q)` causing the iterator to advance to `o` which is visible.

## Flush and Compaction Pacing

Flushes and compactions in LSM trees are problematic because they
contend with foreground traffic, resulting in write and read latency
spikes. Without throttling the rate of flushes and compactions, they
occur "as fast as possible" (which is not entirely true, since we
have a `bytes_per_sync` option). This instantaneous usage of CPU and
disk IO results in potentially huge latency spikes for writes and
reads which occur in parallel to the flushes and compactions.

RocksDB attempts to solve this issue by offering an option to limit
the speed of flushes and compactions. A maximum `bytes/sec` can be
specified through the options, and background IO usage will be limited
to the specified amount. Flushes are given priority over compactions,
but they still use the same rate limiter. Though simple to implement
and understand, this option is fragile for various reasons.

1) If the rate limit is configured too low, the DB will stall and
write throughput will be affected.
2) If the rate limit is configured too high, the write and read
latency spikes will persist.
3) A different configuration is needed per system depending on the
speed of the storage device.
4) Write rates typically do not stay the same throughout the lifetime
of the DB (higher throughput during certain times of the day, etc) but
the rate limit cannot be configured during runtime.

RocksDB also offers an
["auto-tuned" rate limiter](https://rocksdb.org/blog/2017/12/18/17-auto-tuned-rate-limiter.html)
which uses a simple multiplicative-increase, multiplicative-decrease
algorithm to dynamically adjust the background IO rate limit depending
on how much of the rate limiter has been exhausted in an interval.
This solves the problem of having a static rate limit, but Pebble
attempts to improve on this with a different pacing mechanism.

Pebble's pacing mechanism uses separate rate limiters for flushes and
compactions. Both the flush and compaction pacing mechanisms work by
attempting to flush and compact only as fast as needed and no faster.
This is achieved differently for flushes versus compactions.

For flush pacing, Pebble keeps the rate at which the memtable is
flushed at the same rate as user writes. This ensures that disk IO
used by flushes remains steady. When a mutable memtable becomes full
and is marked immutable, it is typically flushed as fast as possible.
Instead of flushing as fast as possible, what we do is look at the
total number of bytes in all the memtables (mutable + queue of
immutables) and subtract the number of bytes that have been flushed in
the current flush. This number gives us the total number of bytes
which remain to be flushed. If we keep this number steady at a constant
level, we have the invariant that the flush rate is equal to the write
rate.

When the number of bytes remaining to be flushed falls below our
target level, we slow down the speed of flushing. We keep a minimum
rate at which the memtable is flushed so that flushes proceed even if
writes have stopped. When the number of bytes remaining to be flushed
goes above our target level, we allow the flush to proceed as fast as
possible, without applying any rate limiting. However, note that the
second case would indicate that writes are occurring faster than the
memtable can flush, which would be an unsustainable rate. The LSM
would soon hit the memtable count stall condition and writes would be
completely stopped.

For compaction pacing, Pebble uses an estimation of compaction debt,
which is the number of bytes which need to be compacted before no
further compactions are needed. This estimation is calculated by
looking at the number of bytes that have been flushed by the current
flush routine, adding those bytes to the size of the level 0 sstables,
then seeing how many bytes exceed the target number of bytes for the
level 0 sstables. We multiply the number of bytes exceeded by the
level ratio and add that number to the compaction debt estimate.
We repeat this process until the final level, which gives us a final
compaction debt estimate for the entire LSM tree.

Like with flush pacing, we want to keep the compaction debt at a
constant level. This ensures that compactions occur only as fast as
needed and no faster. If the compaction debt estimate falls below our
target level, we slow down compactions. We maintain a minimum
compaction rate so that compactions proceed even if flushes have
stopped. If the compaction debt goes above our target level, we let
compactions proceed as fast as possible without any rate limiting.
Just like with flush pacing, this would indicate that writes are
occurring faster than the background compactions can keep up with,
which is an unsustainable rate. The LSM's read amplification would
increase and the L0 file count stall condition would be hit.

With the combined flush and compaction pacing mechanisms, flushes and
compactions only occur as fast as needed and no faster, which reduces
latency spikes for user read and write operations.

## Write throttling

RocksDB adds artificial delays to user writes when certain thresholds
are met, such as `l0_slowdown_writes_threshold`. These artificial
delays occur when the system is close to stalling to lessen the write
pressure so that flushing and compactions can catch up. On the surface
this seems good, since write stalls would seemingly be eliminated and
replaced with gradual slowdowns. Closed loop write latency benchmarks
would show the elimination of abrupt write stalls, which seems
desirable.

However, this doesn't do anything to improve latencies in an open loop
model, which is the model more likely to resemble real world use
cases. Artificial delays increase write latencies without a clear
benefit. Writes stalls in an open loop system would indicate that
writes are generated faster than the system could possibly handle,
which adding artificial delays won't solve.

For this reason, Pebble doesn't add artificial delays to user writes
and writes are served as quickly as possible.

### Other Differences

* `internalIterator` API which minimizes indirect (virtual) function
  calls
* Previous pointers in the memtable and indexed batch skiplists
* Elision of per-key lower/upper bound checks in long range scans
* Improved `Iterator` API
  + `SeekPrefixGE` for prefix iteration
  + `SetBounds` for adjusting the bounds on an existing `Iterator`
* Simpler `Get` implementation
