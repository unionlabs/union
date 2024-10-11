- Feature Name: Flushable Ingested SSTable
- Status: in-progress
- Start Date: 2022-03-11
- Authors: Mufeez Amjad
- RFC PR: [#1586](https://github.com/cockroachdb/pebble/pull/1586)
- Pebble Issues: [#25](https://github.com/cockroachdb/pebble/issues/25)
- Cockroach Issues:

## Summary

To avoid a forced flush when ingesting SSTables that have an overlap with a
memtable, we "lazily" add the SSTs to the LSM as a `*flushableEntry` to
`d.mu.mem.queue`. In comparison to a regular ingest which adds the SSTs to the
lowest possible level, the SSTs will get placed in the memtable queue before
they are eventually flushed (to the lowest level possible). This state is only
persisted in memory until a flush occurs, thus we require a WAL entry to replay
the ingestion in the event of a crash.

## Motivation

Currently, if any of the SSTs that need to be ingested have an overlap with a
memtable, we
[wait](https://github.com/cockroachdb/pebble/blob/56c5aebe151977964db7e464bb6c87ebd3451bd5/ingest.go#L671)
for the memtable to be flushed before the ingestion can proceed. This is to
satisfy the invariant that newer entries (those in the ingested SSTs) in the LSM
have a higher sequence number than old entries (those in the memtables). This
problem is also present for subsequent normal writes that are blocked behind the
ingest waiting for their sequence number to be made visible.

## Technical Design

The proposed design is mostly taken from Peter's suggestion in #25. The core
requirements are:
1. Replayable WAL entry for the ingest.
2. Implementation of the `flushable` interface for a new `ingestedSSTables` struct.
3. Lazily adding the ingested SSTs to the LSM.
4. Flushing logic to move SSTs into L0-L6.

<br>

### 1. WAL Entry

We require a WAL entry to make the ingestion into the flushable queue
replayable, and there is a need for a new type of WAL entry that does not get
applied to the memtable. 2 approaches were considered:
1. Using `seqnum=0` to differentiate this new WAL entry.
2. Introduce a new `InternalKeyKind` for the new WAL entry,
   `InternalKeyKindIngestSST`.

We believe the second approach is better because it avoids modifying batch
headers which can be messy/hacky and because `seqnum=0` is already used for
unapplied batches. The second approach also gives way for a simpler/cleaner
implementation because it utilizes the extensibility of `InternalKeyKind` and is
similar to the treatment of `InternalKeyKindLogData`. It also follows the
correct seqnum semantics for SSTable ingestion in the event of a WAL replay —
each SST in the ingestion batch already gets its own sequence number.

This change will need to be gated on a `FormatMajorVersion` because if the store
is opened with an older version of Pebble, Pebble will not understand any WAL
entry that contains the new `InternalKeyKind`.

<br>

When performing an ingest (with overlap), we create a batch with the header:

```
+-------------+------------+--- ... ---+
| SeqNum (8B) | Count (4B) |  Entries  |
+-------------+------------+--- ... ---+
```

where`SeqNum` is the current running sequence number in the WAL, `Count` is the
number of ingested SSTs, and each entry has the form:

```
+-----------+-----------------+-------------------+
| Kind (1B) | Key (varstring) | Value (varstring) |
+-----------+-----------------+-------------------+
```

where `Kind` is `InternalKeyKindIngestSST`, and `Key` is a path to the
ingested SST on disk.

When replaying the WAL, we check every batch's first entry and if `keykind ==
InternalKeyKindIngestSSTs` then we continue reading the rest of the entries in
the batch of SSTs and replay the ingestion steps - we construct a
`flushableEntry` and add it to the flushable queue:

```go
b = Batch{db: d}
b.SetRepr(buf.Bytes())
seqNum := b.SeqNum()
maxSeqNum = seqNum + uint64(b.Count())
br := b.Reader()
if kind, _, _, _ := br.Next(); kind == InternalKeyKindIngestSST {
  // Continue reading the rest of the batch and construct flushable 
  // of sstables with correct seqnum and add to queue.
  buf.Reset()
  continue
}
```


### 2. `flushable` Implementation

Introduce a new flushable type: `ingestedSSTables`.

```go
type ingestedSSTables struct {
    files []*fileMetadata
    size  uint64

    cmp      Compare
    newIters tableNewIters
}
```
which implements the following functions from the `flushable` interface:

#### 1. `newIter(o *IterOptions) internalIterator`

We return a `levelIter` since the ingested SSTables have no overlap, and we can
treat them like a level in the LSM.

```go
levelSlice := manifest.NewLevelSliceKeySorted(s.cmp, s.files)
return newLevelIter(*o, s.cmp, nil, s.newIters, levelSlice.Iter(), 0, nil)
```

<br>

On the client-side, this iterator would have to be used like this:
```go
var iter internalIteratorWithStats
var rangeDelIter keyspan.FragmentIterator
iter = base.WrapIterWithStats(mem.newIter(&dbi.opts))
switch mem.flushable.(type) {
case *ingestedSSTables:
    iter.(*levelIter).initRangeDel(&rangeDelIter)
default:
    rangeDelIter = mem.newRangeDelIter(&dbi.opts)
}

mlevels = append(mlevels, mergingIterLevel{
    iter:         iter,
    rangeDelIter: rangeDelIter,
})
```

#### 2. `newFlushIter(o *IterOptions, bytesFlushed *uint64) internalIterator`

#### 3. `newRangeDelIter(o *IterOptions) keyspan.FragmentIterator`

The above two methods would return `nil`. By doing so, in `c.newInputIter()`:
```go
if flushIter := f.newFlushIter(nil, &c.bytesIterated); flushIter != nil {
    iters = append(iters, flushIter)
}
if rangeDelIter := f.newRangeDelIter(nil); rangeDelIter != nil {
    iters = append(iters, rangeDelIter)
}
```
we ensure that no iterators on `ingestedSSTables` will be used while flushing in
`c.runCompaction()`.

The special-cased flush process for this flushable is described in [Section
4](#4-flushing-logic-to-move-ssts-into-l0).

#### 4. `newRangeKeyIter(o *IterOptions) keyspan.FragmentIterator`

Will wait on range key support in `levelIter` to land before implementing.

#### 5. `inuseBytes() uint64` and `totalBytes() uint64`

For both functions, we return 0.

Returning 0 for `inuseBytes()` means that the calculation of `c.maxOverlapBytes`
is not affected by the SSTs (the ingested SSTs don't participate in the
compaction).

We don't want the size of the ingested SSTs to contribute to the size of the
memtable when determining whether or not to stall writes
(`MemTableStopWritesThreshold`); they should contribute to the L0 read-amp
instead (`L0StopWritesThreshold`). Thus, we'll have to special case for ingested
SSTs in `d.makeRoomForWrite()` to address this detail.

`totalBytes()` represents the number of bytes allocated by the flushable, which
in our case is 0. A consequence for this is that the size of the SSTs do not
count towards the flush threshold calculation. However, by setting
`flushableEntry.flushForced` we can achieve the same behaviour.

#### 6. `readyForFlush() bool`

The flushable of ingested SSTs can always be flushed because the files are
already on disk, so we return true.

### 3. Lazily adding the ingested SSTs to the LSM

The steps to add the ingested SSTs to the flushable queue are:
1. Detect an overlap exists (existing logic).

Add a check that falls back to the old ingestion logic of blocking the ingest on
the flush when `len(d.mu.mem.queue) >= MemtablesStopWritesThreshold - 1`. This
reduces the chance that many short, overlapping, and successive ingestions cause
a memtable write stall.

Additionally, to mitigate the hiccup on subsequent normal writes, we could wait
before the call to `d.commit.AllocateSeqNum` until:
1. the number of immutable memtables and `ingestedSSTs` in the flushable queue
   is below a certain threshold (to prevent building up too many sublevels)
2. the number of immutable memtables is low. This could lead to starvation if
   there is a high rate of normal writes.

2. Create a batch with the list of ingested SSTs.
```go
b := newBatch()
for _, path := range paths:
    b.IngestSSTs([]byte(path), nil)
```
3. Apply the batch.

In the call to `d.commit.AllocateSeqNum`, `b.count` sequence numbers are already
allocated before the `prepare` step. When we identify a memtable overlap, we
commit the batch to the WAL manually (through logic similar to
`commitPipeline.prepare`). The `apply` step would be a no-op if we performed a
WAL write in the `prepare` step. We would also need to truncate the memtable/WAL
after this step.

5. Create `ingestedSSTables` flushable and `flushableEntry`.

We'd need to call `ingestUpdateSeqNum` on these SSTs before adding them to the
flushable. This is to respect the sequence number ordering invariant while the
SSTs reside in the flushable queue.

6. Add to flushable queue.

Pebble requires that the last entry in `d.mu.mem.queue` is the mutable memtable
with value `d.mu.mem.mutable`. When adding a `flushableEntry` to the queue, we
want to maintain this invariant. To do this we pass `nil` as the batch to
`d.makeRoomForWrite()`. The result is

```
| immutable old memtable | mutable new memtable |
```

We then append our new `flushableEntry`, and swap the last two elements in
`d.mu.mem.queue`:

```
| immutable old memtable | ingestedSSTables | mutable new memtable |
```

Because we add the ingested SSTs to the flushable queue when there is overlap,
and are skipping applying the version edit through the `apply` step of the
ingestion, we ensure that the SSTs are only added to the LSM once.

7. Call `d.maybeScheduleFlush()`.

Because we've added an immutable memtable to the flushable queue and set
`flushForced` on the `flushableEntry`, this will surely result in a flush. This
call can be done asynchronously.

We can then return to caller without waiting for the flush to finish.

### 4. Flushing logic to move SSTs into L0-L6

By returning `nil` for both `flushable.newFlushIter()` and
`flushable.newRangeDelIter()`, the `ingestedSSTables` flushable will not be
flushed normally.

The suggestion in issue #25 is to move the SSTs from the flushable queue into
L0. However, only the tables that overlap with the memtable will need to target
L0 (because they will likely overlap with L0 post flush), the others can be
moved to lower levels in the LSM. We can use the existing logic in
`ingestTargetLevel` to determine which level to move the ingested SSTables to
during `c.runCompaction()`. However, it's important to do this step after the
memtable has been flushed to use the correct `version` when determining overlap.

The flushable of ingested SSTs should not influence the bounds on the
compaction, so we will have to skip updating `c.smallest` and `c.largest` in
`d.newFlush()` for this flushable.
