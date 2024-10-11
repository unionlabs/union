- Feature Name: Virtual sstables
- Status: in-progress
- Start Date: 2022-10-27
- Authors: Arjun Nair
- RFC PR: https://github.com/cockroachdb/pebble/pull/2116
- Pebble Issues:
  https://github.com/cockroachdb/pebble/issues/1683


** Design Draft**

# Summary

The RFC outlines the design to enable virtualizing of physical sstables
in Pebble.

A virtual sstable has no associated physical data on disk, and is instead backed
by an existing physical sstable. Each physical sstable may be shared by one, or
more than one virtual sstable.

Initially, the design will be used to lower the read-amp and the write-amp
caused by certain ingestions. Sometimes, ingestions are unable to place incoming
files, which have no data overlap with other files in the lsm, lower in the lsm
because of file boundary overlap with files in the lsm. In this case, we are
forced to place files higher in the lsm, sometimes in L0, which can cause higher
read-amp and unnecessary write-amp as the file is moved lower down the lsm. See
https://github.com/cockroachdb/cockroach/issues/80589 for the problem occurring
in practice.

Eventually, the design will also be used for the disaggregated storage masking
use-case: https://github.com/cockroachdb/cockroach/pull/70419/files.

This document describes the design of virtual sstables in Pebble with enough
detail to aid the implementation and code review.

# Design

### Ingestion

When an sstable is ingested into Pebble, we try to place it in the lowest level
without any data overlap, or any file boundary overlap. We can make use of
virtual sstables in the cases where we're forced to place the ingested sstable
at a higher level due to file boundary overlap, but no data overlap.

```
                                  s2
ingest:                     [i-j-------n]
                                  s1
L6:                 [e---g-----------------p---r]
             a b c d e f g h i j k l m n o p q r s t u v w x y z
```

Consider the sstable s1 in L6 and the ingesting sstable s2. It is clear that
the file boundaries of s1 and s2 overlap, but there is no data overlap as shown
in the diagram. Currently, we will be forced to ingest the sstable s2 into a
level higher than L6. With virtual sstables, we can split the existing sstable
s1 into two sstables s3 and s4 as shown in the following diagram.

```
                       s3         s2        s4
L6:                 [e---g]-[i-j-------n]-[p---r]
             a b c d e f g h i j k l m n o p q r s t u v w x y z
```

The sstable s1 will be deleted from the lsm. If s1 was a physical sstable, then
we will keep the file on disk as long as we need to so that it can back the
virtual sstables.

There are cases where the ingesting sstables have no data overlap with existing
sstables, but we can't make use of virtual sstables. Consider:
```
                                  s2
ingest:               [f-----i-j-------n]
                                  s1
L6:                 [e---g-----------------p---r]
             a b c d e f g h i j k l m n o p q r s t u v w x y z
```
We cannot use virtual sstables in the above scenario for two reasons:
1. We don't have a quick method of detecting no data overlap.
2. We will be forced to split the sstable in L6 into more than two virtual
   sstables, but we want to avoid many small virtual sstables in the lsm.

Note that in Cockroach, the easier-to-solve case happens very regularly when an
sstable spans a range boundary (which pebble has no knowledge of), and we ingest
a snapshot of a range in between the two already-present ranges.

slide in between two existing sstables is more likely to happen. It occurs when
we ingest a snapshot of a range in between two already present ranges.

`ingestFindTargetLevel` changes:
- The `ingestFindTargetLevel` function is used to determine the target level
  of the file which is being ingested. Currently, this function returns an `int`
  which is the target level for the ingesting file. Two additional return
  parameters, `[]manifest.NewFileEntry` and `*manifest.DeletedFileEntry`, will be
  added to the function.
- If `ingestFindTargetLevel` decides to split an existing sstable into virtual
  sstables, then it will return new and deleted entries. Otherwise, it will only
  return the target level of the ingesting file.
- Within the `ingestFindTargetLevel` function, the `overlapWithIterator`
  function is used to quickly detect data overlap. In the case with file
  boundary overlap, but no data overlap, in the lowest possible level, we will
  split the existing sstable into virtual sstables and generate the
  `NewFileEntry`s and the `DeletedFileEntry`. The `FilemetaData` section
  describes how the various fields in the `FilemetaData` will be computed for
  the newly created virtual sstables.

- Note that we will not split physical sstables into virtual sstables in L0 for
  the use case described in this RFC. The benefit of doing so would be to reduce
  the number of L0 sublevels, but the cost would be additional implementation
  complexity(see the `FilemetaData` section). We also want to avoid too many
  virtual sstables in the lsm as they can lead to space amp(see `Compaction`
  section). However, in the future, for the disaggregated storage masking case,
  we would need to support ingestion and use of virtual sstables in L0.

- Note that we may need an upper bound on the number of times an sstable is
  split into smaller virtual sstables. We can further reduce the risk of many
  small sstables:
  1. For CockroachDB's snapshot ingestion, there is one large sst (up to 512MB)
     and many tiny ones. We can choose the apply this splitting logic only for
     the large sst. It is ok for the tiny ssts to be ingested into L0.
  2. Split only if the ingested sst is at least half the size of the sst being
     split. So if we have a smaller ingested sst, we will pick a higher level to
     split at (where the ssts are smaller). The lifetime of virtual ssts at a
     higher level is smaller, so there is lower risk of littering the LSM with
     long-lived small virtual ssts.
  3. For disaggregated storage implementation, we can avoid masking for tiny
     sstables being ingested and instead write a range delete like we currently
     do. Precise details on the masking use case are out of the scope of this
     RFC.

`ingestApply` changes:
- The new and deleted file entries returned by the `ingestFindTargetLevel`
  function will be added to the version edit in `ingestApply`.
- We will appropriately update the `levelMetrics` based on the new information
  returned by `ingestFindTargetLevel`.


### `FilemetaData` changes

Each virtual sstables will have a unique file metadata value associated with it.
The metadata may be borrowed from the backing physical sstable, or it may be
unique to the virtual sstable.

This rfc lists out the fields in the `FileMetadata` struct with information on
how each field will be populated.

`Atomic.AllowedSeeks`: Field is used for read triggered compactions, and we can
populate this field for each virtual sstable since virtual sstables can be
picked for compactions.

`Atomic.statsValid`: We can set this to true(`1`) when the virtual sstable is
created. On virtual sstable creation we will estimate the table stats of the
virtual sstable based on the table stats of the physical sstable. We can also
set this to `0` and let the table stats job asynchronously compute the stats.

`refs`: The will be turned into a pointer which will be shared by the
virtual/physical sstables. See the deletion section of the RFC to learn how the
`refs` count will be used.

`FileNum`: We could give each virtual sstable its own file number or share
the file number between all the virtual sstables. In the former case, the virtual
sstables will be distinguished by the file number, and will have an additional
metadata field to indicate the file number of the parent sstable. In the latter
case, we can use a few of the most significant bits of the 64 bit file number to
distinguish the virtual sstables.

The benefit of using a single file number for each virtual sstable, is that we
don't need to use additional space to store the file number of the backing
physical sstable.

It might make sense to give each virtual sstable its own file number. Virtual
sstables are picked for compactions, and compactions and compaction picking
expect a unique file number for each of the files which it is compacting.
For example, read compactions will use the file number of the file to determine
if a file picked for compaction has already been compacted, the version edit
will expect a different file number for each virtual sstable, etc.

There are direct references to the `FilemetaData.FileNum` throughout Pebble. For
example, the file number is accessed when the the `DB.Checkpoint` function is
called. This function iterates through the files in each level of the lsm,
constructs the filepath using the file number, and reads the file from disk. In
such cases, it is important to exclude virtual sstables.

`Size`: We compute this using linear interpolation on the number of blocks in
the parent sstable and the number of blocks in the newly created virtual sstable.

`SmallestSeqNum/LargestSeqNum`: These fields depend on the parent sstable,
but we would need to perform a scan of the physical sstable to compute these
accurately for the virtual sstable upon creation. Instead, we could convert
these fields into lower and upper bounds of the sequence numbers in a file.

These fields are used for l0 sublevels, pebble tooling, delete compaction hints,
and a lot of plumbing. We don't need to worry about the L0 sublevels use case
because we won't have virtual sstables in L0 for the use case in this RFC. For
the rest of the use cases we can use lower bound for the smallest seq number,
and an upper bound for the largest seq number work.

TODO(bananabrick): Add more detail for any delete compaction hint changes if
necessary.

`Smallest/Largest`: These, along with the smallest/largest ranges for the range
and point keys can be computed upon virtual sstable creation. Precisely, these
can be computed when we try and detect data overlap in the `overlapWithIterator`
function during ingestion.

`Stats`: `TableStats` will either be computed upon virtual sstable creation
using linear interpolation on the block counts of the virtual/physical sstables
or asynchronously using the file bounds of the virtual sstable.

`PhysicalState`: We can add an additional struct with state associated with
physical ssts which have been virtualized.

```
type PhysicalState struct {
  // Total refs across all virtual ssts * versions. That is, if the same virtual
  // sst is present in multiple versions, it may have multiple refs, if the
  // btree node is not the same.
  totalRefs int32

  // Number of virtual ssts in the latest version that refer to this physical
  // SST. Will be 1 if there is only a physical sst, or there is only 1 virtual
  // sst referencing this physical sst.
  // INVARIANT: refsInLatestVersion <= totalRefs
  // refsInLatestVersion == 0 is a zombie sstable.
  refsInLatestVersion int32

  fileSize uint64

  // If sst is not virtualized and in latest version
  // virtualSizeSumInLatestVersion == fileSize. If
  // virtualSizeSumInLatestVersion > 0 and
  // virtualSizeSumInLatestVersion/fileSize is very small, the corresponding
  // virtual sst(s) should be candidates for compaction. These candidates can be
  // tracked via btree annotations. Incrementlly updated in
  // BulkVersionEdit.Apply, when updating refsInLatestVersion.
  virtualSizeSumInLatestVersion uint64
}
```

The `Deletion` section and the `Compactions` section describe why we need to
store the `PhysicalState`.

### Deletion of physical and virtual sstables

We want to ensure that the physical sstable is only deleted from disk when no
version references it, and when there are no virtual sstables which are backed
by the physical sstable.

Since `FilemetaData.refs` is a pointer which is shared by the physical and
virtual sstables, the physical sstable won't be deleted when it is removed
from the latest version as the `FilemetaData.refs` will have been increased
when the virtual sstable is added to a version. Therefore, we only need to
ensure that the physical sstable is eventually deleted when there are no
versions which reference it.

Sstables are deleted from disk by the `DB.doDeleteObsoleteFiles` function which
looks for files to delete in the the `DB.mu.versions.obsoleteTables` slice.
So we need to ensure that any physical sstable which was virtualized is added to
the obsolete tables list iff `FilemetaData.refs` is 0.

Sstable are added to the obsolete file list when a `Version` is unrefed and
when `DB.scanObsoleteFiles` is called when Pebble is opened.

When a `Version` is unrefed, sstables referenced by it are only added to the
obsolete table list if the `FilemetaData.refs` hits 0 for the sstable. With
virtual sstables, we can have a case where the last version which directly
references a physical sstable is unrefed, but the physical sstable is not added
to the obsolete table list because its `FilemetaData.refs` count is not 0
because of indirect references through virtual sstables. Since the last Version
which directly references the physical sstable is deleted, the physical sstable
will never get added to the obsolete table list. Since virtual sstables keep
track of their parent physical sstable, we can just add the physical sstable to
the obsolete table list when the last virtual sstable which references it is
deleted.

`DB.scanObsoleteFiles` will delete any file which isn't referenced by the
`VersionSet.versions` list. So, it's possible that a physical sstable associated
with a virtual sstable will be deleted. This problem can be fixed by a small
tweak in the `d.mu.versions.addLiveFileNums` to treat the parent sstable of
a virtual sstable as a live file.

Deleted files still referenced by older versions are considered zombie sstables.
We can extend the definition of zombie sstables to be any sstable which is not
directly, or indirectly through virtual sstables, referenced by the latest
version. See the `PhysicalState` subsection of the `FilemetaData` section
where we describe how the references in the latest version will be tracked.


### Reading from virtual sstables

Since virtual sstables do not exist on disk, we will have to redirect reads
to the physical sstable which backs the virtual sstable.

All reads to the physical files go through the table cache which opens the file
on disk and creates a `Reader` for the reads. The table cache currently creates
a `FileNum` -> `Reader` mapping for the physical sstables.

Most of the functions in table cache API take the file metadata of the file as
a parameter. Examples include `newIters`, `newRangeKeyIter`, `withReader`, etc.
Each of these functions then calls a subsequent function on the sstable
`Reader`.

In the `Reader` API, some functions only really need to be called on physical
sstables, whereas some functions need to be called on both physical and virtual
sstables. For example, the `Reader.EstimateDiskUsage` usage function, or the
`Reader.Layout` function only need to be called on physical sstables, whereas,
some function like, `Reader.NewIter`, and `Reader.NewCompactionIter` need to
work with virtual sstables.

We could either have an abstraction over the physical sstable `Reader` per
virtual sstable, or update the `Reader` API to accept file bounds of the
sstable. In the latter case, we would create one `Reader` on the physical
sstable for all of the virtual sstables, and update the `Reader` API to accept
the file bounds of the sstable.

Changes required to share a `Reader` on the physical sstable among the virtual
sstable:
- If the file metadata of the virtual sstable is passed into the table cache, on
  a table cache miss, the table cache will load the Reader for the physical
  sstable. This step can be performed in the `tableCacheValue.load` function. On
  a table cache hit, the file number of the parent sstable will be used to fetch
  the appropriate sstable `Reader`.
- The `Reader` api will be updated to support reads from virtual sstables. For
  example, the `NewCompactionIter` function will take additional
  `lower,upper []byte` parameters.

Updates to iterators:
- `Reader.NewIter` already has `lower,upper []byte` parameters so this requires
   no change.
- Add `lower,upper` fields to the `Reader.NewCompactionIter`. The function
  initializes single level and two level iterators, and we can pass in the
  `lower,upper` values to those. TODO(bananabrick): Make sure that the value
  of `bytesIterated` in the compaction iterator is still accurate.
- `Reader.NewRawRangeKeyIter/NewRawRangeDelIter`: We need to add `lower/upper`
   fields to the functions. Both iterators make use of a `fragmentBlockIter`. We
   could filter keys above the `fragmentBlockIter` or add filtering within the
   `fragmentBlockIter`. To add filtering within the `fragmentBlockIter` we will
   initialize it with two additional `lower/upper []byte` fields.
- We would need to update the `SetBounds` logic for the sstable iterators to
  never set bounds for the iterators outside the virtual sstable bounds. This
  could lead to keys outside the virtual sstable bounds, but inside the physical
  sstable bounds, to be surfaced.

TODO(bananabrick): Add a section about sstable properties, if necessary.

### Compactions

Virtual sstables can be picked for compactions. If the `FilemetaData` and the
iterator stack changes work, then compaction shouldn't require much, if any,
additional work.

Virtual sstables which are picked for compactions may cause space amplification.
For example, if we have two virtual sstables `a` and `b` in L5, backed by a
physical sstable `c`, and the sstable `a` is picked for a compaction. We will
write some additional data into L6, but we won't delete sstable `c` because
sstable `b` still refers to it. In the worst case, sstable `b` will never be
picked for compaction and will never be compacted into and we'll have permanent
space amplification. We should try prioritize compaction of sstable `b` to
prevent such a scenario.

See the `PhysicalState` subsection in the `FilemetaData` section to see how
we'll store compaction picking metrics to reduce virtual sstable space-amp.

### `VersionEdit` decode/encode
Any additional fields added to the `FilemetaData` need to be supported in the
version edit `decode/encode` functions.
