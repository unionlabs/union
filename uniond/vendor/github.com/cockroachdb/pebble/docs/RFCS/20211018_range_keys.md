- Feature Name: Range Keys
- Status: draft
- Start Date: 2021-10-18
- Authors: Sumeer Bhola, Jackson Owens
- RFC PR: #1341
- Pebble Issues:
  https://github.com/cockroachdb/pebble/issues/1339
- Cockroach Issues:
  https://github.com/cockroachdb/cockroach/issues/70429
  https://github.com/cockroachdb/cockroach/issues/70412

** Design Draft**

# Summary

An ongoing effort within CockroachDB to preserve MVCC history across all SQL
operations (see cockroachdb/cockroach#69380) requires a more efficient method of
deleting ranges of MVCC history.

This document describes an extension to Pebble introducing first-class support
for range keys. Range keys map a range of keyspace to a value. Optionally, the
key range may include an suffix encoding a version (eg, MVCC timestamp). Pebble
iterators may be configured to surface range keys during iteration, or to mask
point keys at lower MVCC timestamps covered by range keys.

CockroachDB will make use of these range keys to enable history-preserving
removal of contiguous ranges of MVCC keys with constant writes, and efficient
iteration past deleted versions.

# Background

A previous CockroachDB RFC cockroach/cockroachdb#69380 describes the motivation
for the larger project of migrating MVCC-noncompliant operations into MVCC
compliance. Implemented with the existing MVCC primitives, some operations like
removal of an index or table would require performing writes linearly
proportional to the size of the table. Dropping a large table using existing
MVCC point-delete primitives would be prohibitively expensive. The desire for a
sublinear delete of an MVCC range motivates this work.

The detailed design for MVCC compliant bulk operations ([high-level
description](https://github.com/cockroachdb/cockroach/blob/master/docs/RFCS/20210825_mvcc_bulk_ops.md);
detailed design draft for DeleteRange in internal
[doc](https://docs.google.com/document/d/1ItxpitNwuaEnwv95RJORLCGuOczuS2y_GoM2ckJCnFs/edit#heading=h.x6oktstoeb9t)),
ran into complexity by placing range operations above the Pebble layer, such
that Pebble sees these as points. The complexity causes are various: (a) which
key (start or end) to anchor this range on, when represented as a point (there
are performance consequences), (b) rewriting on CockroachDB range splits (and
concerns about rewrite volume), (c) fragmentation on writes and complexity
thereof (and performance concerns for reads when not fragmenting), (d) inability
to efficiently skip older MVCC versions that are masked by a `[k1,k2)@ts` (where
ts is the MVCC timestamp).

Pebble currently has only one kind of key that is associated with a range:
`RANGEDEL [k1, k2)#seq`, where [k1, k2) is supplied by the caller, and is used
to efficiently remove a set of point keys.

First-class support for range keys in Pebble eliminates all these issues.
Additionally, it allows for future extensions like efficient transactional range
operations. This issue describes how this feature would work from the
perspective of a user of Pebble (like CockroachDB), and sketches some
implementation details.

# Design

## Interface

### New `Comparer` requirements

The Pebble `Comparer` type allows users to optionally specify a `Split` function
that splits a user key into a prefix and a suffix. This Split allows users
implementing MVCC (Multi-Version Concurrency Control) to inform Pebble which
part of the key encodes the user key and which part of the key encodes the
version (eg, a timestamp). Pebble does not dictate the encoding of an MVCC
version, only that the version form a suffix on keys.

The range keys design described in this RFC introduces stricter requirements for
user-provided `Split` implementations and the ordering of keys:

1. The user key consisting of just a key prefix `k` must sort before all
   other user keys containing that prefix. Specifically
   `Compare(k[:Split(k)], k) < 0` where `Split(k) < len(k)`.
2. A key consisting of a bare suffix must be a valid key and comparable. The
   ordering of the empty key prefix with any suffixes must be consistent with
   the ordering of those same suffixes applied to any other key prefix.
   Specifically `Compare(k[Split(k):], k2[Split(k2):]) == Compare(k, k2)` where
   `Compare(k[:Split(k)], k2[:Split(k2)]) == 0`.

The details of why these new requirements are necessary are explained in the
implementation section.

### Writes

This design introduces three new write operations:

- `RangeKeySet([k1, k2), [optional suffix], <value>)`: This represents the
  mapping `[k1, k2)@suffix => value`. Keys `k1` and `k2` must not contain a
  suffix (i.e., `Split(k1)==len(k1)` and `Split(k2)==len(k2))`.

- `RangeKeyUnset([k1, k2), [optional suffix])`: This removes a mapping
  previously applied by `RangeKeySet`. The unset may use a smaller key range
  than the original `RangeKeySet`, in which case only part of the range is
  deleted. The unset only applies to range keys with a matching optional suffix.
  If the optional suffix is absent in both the RangeKeySet and RangeKeyUnset,
  they are considered matching.

- `RangeKeyDelete([k1, k2))`: This removes all range keys within the provided
  key span. It behaves like an `Unset` unencumbered by suffix restrictions.

For example, consider `RangeKeySet([a,d), foo)` (i.e., no suffix). If
there is a later call `RangeKeyUnset([b,c))`, the resulting state seen by
a reader is `[a,b) => foo`, `[c,d) => foo`. Note that the value is not
modified when the key is fragmented.

Partially overlapping `RangeKeySet`s with the same suffix overwrite one
another.  For example, consider `RangeKeySet([a,d), foo)`, followed by
`RangeKeySet([c,e), bar)`.  The resulting state is `[a,c) => foo`, `[c,e)
=> bar`.

Point keys (eg, traditional keys defined at a singular byte string key) and
range keys do not overwrite one another. They have a parallel existence. Point
deletes only apply to points. Range unsets only apply to range keys. However,
users may configure iterators to mask point keys covered by newer range keys.
This masking behavior is explicitly requested by the user in the context of the
iteration. Masking is described in more detail below.

There exist separate range delete operations for point keys and range keys. A
`RangeKeyDelete` may remove part of a range key, just like the new
`RangeKeyUnset` operation introduced earlier. `RangeKeyDelete`s differ from
`RangeKeyUnset`s, because the latter requires that the suffix matches and
applies only to range keys. `RangeKeyDelete`s completely clear all existing
range keys within their span at all suffix values.

The optional suffix in `RangeKeySet` and `RangeKeyUnset` operations is related
to the pebble `Comparer.Split` operation which is explicitly documented as being
for [MVCC
keys](https://github.com/cockroachdb/pebble/blob/e95e73745ce8a85d605ef311d29a6574db8ed3bf/internal/base/comparer.go#L69-L88),
without mandating exactly how the versions are represented. `RangeKeySet` and
`RangeKeyUnset` keys with different suffixes do not interact logically, although
Pebble will observably fragment ranges at intersection points.

### Iteration

A user iterating over a key interval [k1,k2) can request:

- **[I1]** An iterator over only point keys.

- **[I2]** A combined iterator over point and range keys. This is what
  we mainly discuss below in the implementation discussion.

- **[I3]** An iterator over only range keys. In the CockroachDB use
    case, range keys will need to be subject to MVCC GC just like
    point keys — this iterator may be useful for that purpose.

The `pebble.Iterator` type will be extended to provide accessors for
range keys for use in the combined and exclusively range iteration
modes.

```
// HasPointAndRange indicates whether there exists a point key, a range key or
// both at the current iterator position.
HasPointAndRange() (hasPoint, hasRange bool)

// RangeKeyChanged indicates whether the most recent iterator positioning
// operation resulted in the iterator stepping into or out of a new range key.
// If true previously returned range key bounds and data has been invalidated.
// If false, previously obtained range key bounds, suffix and value slices are
// still valid and may continue to be read.
RangeKeyChanged() bool

// Key returns the key of the current key/value pair, or nil if done. If
// positioned at an iterator position that only holds a range key, Key()
// always returns the start bound of the range key. Otherwise, it returns
// the point key's key.
Key() []byte

// RangeBounds returns the start (inclusive) and end (exclusive) bounds of the
// range key covering the current iterator position. RangeBounds returns nil
// bounds if there is no range key covering the current iterator position, or
// the iterator is not configured to surface range keys.
//
// If valid, the returned start bound is less than or equal to Key() and the
// returned end bound is greater than Key().
RangeBounds() (start, end []byte)

// Value returns the value of the current key/value pair, or nil if done.
// The caller should not modify the contents of the returned slice, and
// its contents may change on the next call to Next.
//
// Only valid if HasPointAndRange() returns true for hasPoint.
Value() []byte

// RangeKeys returns the range key values and their suffixes covering the
// current iterator position. The range bounds may be retrieved separately
// through RangeBounds().
RangeKeys() []RangeKey

type RangeKey struct {
    Suffix []byte
    Value  []byte
}
```

When a combined iterator exposes range keys, it exposes all the range
keys covering `Key`. During iteration with a combined iterator, an
iteration position may surface just a point key, just a range key or
both at the currently-positioned `Key`.

Described another way, a Pebble combined iterator guarantees that it
will stop at all positions within the keyspace where:
1. There exists a point key at that position.
2. There exists a range key that logically begins at that postition.

In addition to the above positions, a Pebble iterator may also stop at keys
in-between the above positions due to fragmentation. Range keys are defined over
continuous spans of keyspace. Range keys with different suffix values may
overlap each other arbitrarily. To surface these arbitrarily overlapping spans
in an understandable and efficient way, the Pebble iterator surfaces range keys
fragmented at intersection points. Consider the following sequence of writes:

```
    RangeKeySet([a,z), @1, 'apple')
    RangeKeySet([c,e), @3, 'banana')
    RangeKeySet([e,m), @5, 'orange')
    RangeKeySet([b,k), @7, 'kiwi')
```

This yields a database containing overlapping range keys:
```
  @7 → kiwi     |-----------------)
  @5 → orange         |---------------)
  @3 → banana     |---)
  @1 → apple  |-------------------------------------------------)
              a b c d e f g h i j k l m n o p q r s t u v w x y z
```

During iteration, these range keys are surfaced using the bounds of their
intersection points. For example, a scan across the keyspace containing only
these range keys would observe the following iterator positions:

```
  Key() = a   RangeKeyBounds() = [a,b)   RangeKeys() = {(@1,apple)}
  Key() = b   RangeKeyBounds() = [b,c)   RangeKeys() = {(@7,kiwi), (@1,apple)}
  Key() = c   RangeKeyBounds() = [c,e)   RangeKeys() = {(@7,kiwi), (@3,banana), (@1,apple)}
  Key() = e   RangeKeyBounds() = [e,k)   RangeKeys() = {(@7,kiwi), (@5,orange), (@1,apple)}
  Key() = k   RangeKeyBounds() = [k,m)   RangeKeys() = {(@5,orange), (@1,apple)}
  Key() = m   RangeKeyBounds() = [m,z)   RangeKeys() = {(@1,apple)}
```

This fragmentation produces a more understandable interface, and avoids forcing
iterators to read all range keys within the bounds of the broadest range key.
Consider this example:

```
                   iterator pos          [ ] - sstable bounds
                         |
L1:         [a----v1@t2--|-h]     [l-----unset@t1----u]
L2:                 [e---|------v1@t1----------r]
             a b c d e f g h i j k l m n o p q r s t u v w x y z
```

If the iterator is positioned at a point key `g`, there are two overlapping
physical range keys: `[a,h)@t2→v1` and `[e,r)@t1→v1`.

However, the `RangeKeyUnset([l,u), @t1)` removes part of the `[e,r)@t1→v1` range
key, truncating it to the bounds `[e,l)`. The iterator must return the truncated
bounds that correctly respect the `RangeKeyUnset`. However, when the range keys
are stored within a log-structured merge tree like Pebble, the `RangeKeyUnset`
may not be contained within the level's sstable that overlaps the current point
key. Searching for the unset could require reading an unbounded number of
sstables, losing the log-structured merge tree's property that bounds read
amplification to the number of levels in the tree.

Fragmenting range keys to intersection points avoids this problem. The iterator
positioned at `g` only surfaces range key state with the bounds `[e,h)`, the
widest bounds in which it can guarantee t2→v1 and t1→v1 without loading
additional sstables.

#### Iteration order

Recall that the user-provided `Comparer.Split(k)` function divides all user keys
into a prefix and a suffix, such that the prefix is `k[:Split(k)]`, and the
suffix is `k[Split(k):]`. If a key does not contain a suffix, the key equals the
prefix.

An iterator that is configured to surface range keys alongside point keys will
surface all range keys covering the current `Key()` position. Revisiting an
earlier example with the addition of three new point key-value pairs:
a→artichoke, b@2→beet and t@3→turnip. Consider '@<number>' to form the suffix
where present, with `<number>` denoting a MVCC timestamp. Higher, more-recent
timestamps sort before lower, older timestamps.

```
              .                                                   a   → artichoke
  @7 → kiwi     |-----------------)
  @5 → orange         |---------------)
                . b@2                                             b@2 → beet
  @3 → banana     |---)                             . t@3         t@3 → turnip
  @1 → apple  |-------------------------------------------------)
              a b c d e f g h i j k l m n o p q r s t u v w x y z
```

An iterator configured to surface both point and range keys will visit the
following iterator positions during forward iteration:

```
  Key()   HasPointAndRange()   Value()      RangeKeyBounds()    RangeKeys()
  a       (true,  true)        artichoke    [a,b)               {(@1,apple)}
  b       (false, true)        -            [b,c)               {(@7,kiwi), (@1,apple)}
  b@2     (true,  true)        beet         [b,c)               {(@7,kiwi), (@1,apple)}
  c       (false, true)        -            [c,e)               {(@7,kiwi), (@3,banana), (@1,apple)}
  e       (false, true)        -            [e,k)               {(@7,kiwi), (@5,orange), (@1,apple)}
  k       (false, true)        -            [k,m)               {(@5,orange), (@1,apple)}
  m       (false, true)        -            [m,z)               {(@1,apple)}
  t@3     (true,  true)        turnip       [m,z)               {(@1,apple)}
```

Note that:

- While positioned over a point key (eg, Key() = 'a', 'b@2' or t@3'), the
  iterator exposes both the point key's value through Value() and the
  overlapping range keys values through `RangeKeys()`.

- There can be multiple range keys covering a `Key()`, each with a different
  suffix.

- There cannot be multiple range keys covering a `Key()` with the same suffix,
  since the most-recently committed one (eg, the one with the highest sequence
  number) will win, just like for point keys.

- If the iterator has configured lower and/or upper bounds, they will truncate
  the range key to those bounds. For example, if the above iterator had an upper
  bound 'y', the `[m,z)` range key would be surfaced with the bounds `[m,y)`
  instead.

#### Masking

Range key masking provides additional, optional functionality designed
specifically for the use case of implementing a MVCC-compatible delete range.

When constructing an iterator that iterators over both point and range keys, a
user may request that range keys mask point keys. Masking is configured with a
suffix parameter that determines which range keys may mask point keys. Only
range keys with suffixes that sort after the mask's suffix mask point keys. A
range key that meets this condition only masks points with suffixes that sort
after the range key's suffix.

```
type IterOptions struct {
    // ...
    RangeKeyMasking RangeKeyMasking
}

// RangeKeyMasking configures automatic hiding of point keys by range keys.
// A non-nil Suffix enables range-key masking. When enabled, range keys with
// suffixes ≥ Suffix behave as masks. All point keys that are contained within
// a masking range key's bounds and have suffixes greater than the range key's
// suffix are automatically skipped.
//
// Specifically, when configured with a RangeKeyMasking.Suffix _s_, and there
// exists a range key with suffix _r_ covering a point key with suffix _p_, and
//
//   _s_ ≤ _r_ < _p_
//
// then the point key is elided.
//
// Range-key masking may only be used when iterating over both point keys and
// range keys.
type RangeKeyMasking struct {
	// Suffix configures which range keys may mask point keys. Only range keys
	// that are defined at suffixes greater than or equal to Suffix will mask
	// point keys.
	Suffix []byte
	// Filter is an optional field that may be used to improve performance of
	// range-key masking through a block-property filter defined over key
	// suffixes. If non-nil, Filter is called by Pebble to construct a
	// block-property filter mask at iterator creation. The filter is used to
	// skip whole point-key blocks containing point keys with suffixes greater
	// than a covering range-key's suffix.
	//
	// To use this functionality, the caller must create and configure (through
	// Options.BlockPropertyCollectors) a block-property collector that records
	// the maxmimum suffix contained within a block. The caller then must write
	// and provide a BlockPropertyFilterMask implementation on that same
	// property. See the BlockPropertyFilterMask type for more information.
	Filter func() BlockPropertyFilterMask
}
```

Example: A user may construct an iterator with `RangeKeyMasking.Suffix` set to
`@50`. The range key `[a, c)@60` would mask nothing, because `@60` is a more
recent timestamp than `@50`. However a range key `[a,c)@30` would mask `a@20`
and `apple@10` but not `apple@40`. A range key can only mask keys with MVCC
timestamps older than the range key's own timestamp. Only range keys with
suffixes (eg, MVCC timestamps) may mask anything at all.

The pebble Iterator surfaces all range keys when masking is enabled. Only point
keys are ever skipped, and only when they are contained within the bounds of a
range key with a more-recent suffix, and the range key's suffix is older than
the timestamp encoded in `RangeKeyMasking.Sufffix`.

## Implementation

### Write operations

This design introduces three new Pebble write operations: `RangeKeySet`,
`RangeKeyUnset` and `RangeKeyDelete`. Internally, these operations are
represented as internal keys with new corresponding key kinds encoded as a part
of the key trailer. These keys are stored within special range key blocks
separate from point keys, but within the same sstable. The range key blocks hold
`RangeKeySet`, `RangeKeyUnset` and `RangeKeyDelete` keys, but do not hold keys
of any other kind.  Within the memtables, these range keys are stored in a
separate skip list.

- `RangeKeySet([k1,k2), @suffix, value)` is encoded as a `k1.RANGEKEYSET` key
  with a value encoding the tuple `(k2,@suffix,value)`.
- `RangeKeyUnset([k1,k2), @suffix)` is encoded as a `k1.RANGEUNSET` key
  with a value encoding the tuple `(k2,@suffix)`.
- `RangeKeyDelete([k1,k2)` is encoded as a `k1.RANGEKEYDELETE` key with a value
  encoding `k2`.

Range keys are physically fragmented as an artifact of the log-structured merge
tree structure and internal sstable boundaries. This fragmentation is essential
for preserving the performance characteristics of a log-structured merge tree.
Although the public interface operations for `RangeKeySet` and `RangeKeyUnset`
require both boundary keys `[k1,k2)` to always be bare prefixes (eg, to not have
a suffix), internally these keys may be fragmented to bounds containing
suffixes.

Example: If a user attempts to write `RangeKeySet([a@v1, c@v2), @v3, value)`,
Pebble will return an error to the user. If a user writes `RangeKeySet([a, c),
@v3, value)`, Pebble will allow the write and may later internally fragment the
`RangeKeySet` into three internal keys:
 - `RangeKeySet([a, a@v1), @v3, value)`
 - `RangeKeySet([a@v1, c@v2), @v3, value)`
 - `RangeKeySet([c@v2, c), @v3, value)`

This fragmentation preserve log-structured merge tree performance
characteristics because it allows a range key to be split across many sstables,
while preserving locality between range keys and point keys. Consider a
`RangeKeySet([a,z), @1, foo)` on a database that contains millions of point keys
in the range [a,z). If the [a,z) range key was not permitted to be fragmented
internally, it would either need to be stored completely separately from the
point keys in a separate sstable or in a single intractably large sstable
containing all the overlapping point keys. Fragmentation allows locality,
ensuring point keys and range keys in the same region of the keyspace can be
stored in the same sstable.

`RangeKeySet`, `RangeKeyUnset` and `RangeKeyDelete` keys are assigned sequence
numbers, like other internal keys. Log-structured merge tree level invariants
are valid across range key, point keys and between the two. That is:

  1. The point key `k1#s2` cannot be at a lower level than `k2#s1` where
     `k1==k2` and `s1 < s2`. This is the invariant implemented by all LSMs.
  2. `RangeKeySet([k1,k2))#s2` cannot be at a lower level than
     `RangeKeySet([k3,k4))#s1` where `[k1,k2)` overlaps `[k3,k4)` and `s1 < s2`.
  3. `RangeKeySet([k1,k2))#s2` cannot be at a lower level than a point key
     `k3#s1` where `k3 \in [k1,k2)` and `s1 < s2`.

Like other tombstones, the `RangeKeyUnset` and `RangeKeyDelete` keys are elided
when they fall to the bottomost level of the LSM and there is no snapshot
preventing its elision. There is no additional garbage collection problem
introduced by these keys.

There is no Merge operation that affects range keys.

#### Physical representation

`RangeKeySet`, `RangeKeyUnset` and `RangeKeyDelete` keys are keyed by their
start key. This poses an obstacle. We must be able to support multiple range
keys at the same sequence number, because all keys within an ingested sstable
adopt the same sequence number. Duplicate internal keys (keys with equal user
keys, sequence numbers and kinds) are prohibited within Pebble. To resolve this
issue, fragments with the same bounds are merged within snapshot stripes into a
single physical key-value, representing multiple logical key-value pairs:

```
k1.RangeKeySet#s2 → (k2,[(@t2,v2),(@t1,v1)])
```

Within a physical key-value pair, suffix-value pairs are stored sorted by
suffix, descending. This has a minor advantage of reducing iteration-time
user-key comparisons when there exist multiple range keys in a table.

Unlike other Pebble keys, the `RangeKeySet` and `RangeKeyUnset` keys have values
that encode fields of data known to Pebble. The value that the user sets in a
call to `RangeKeySet` is opaque to Pebble, but the physical representation of
the `RangeKeySet`'s value is known. This encoding is a sequence of fields:

* End key, `varstring`, encodes the end user key of the fragment.
* A series of (suffix, value) tuples representing the logical range keys that
  were merged into this one physical `RangeKeySet` key:
  * Suffix, `varstring`
  * Value, `varstring`

Similarly, `RangeKeyUnset` keys are merged within snapshot stripes and have a
physical representation like:

```
k1.RangeKeyUnset#s2 → (k2,[(@t2),(@t1)])
```

A `RangeKeyUnset` key's value is encoded as:
* End key, `varstring`, encodes the end user key of the fragment.
* A series of suffix `varstring`s.

When `RangeKeySet` and `RangeKeyUnset` fragments with identical bounds meet
within the same snapshot stripe within a compaction, any of the
`RangeKeyUnset`'s suffixes that exist within the `RangeKeySet` key are removed.

A `RangeKeyDelete` key has no additional data beyond its end key, which is
encoded directly in the value.

NB: `RangeKeySet` and `RangeKeyUnset` keys are not merged within batches or the
memtable. That's okay, because batches are append-only and indexed batches will
refragment and merge the range keys on-demand. In the memtable, every key is
guaranteed to have a unique sequence number.

### Sequence numbers

Like all Pebble keys, `RangeKeySet`, `RangeKeyUnset` and `RangeKeyDelete` are
assigned sequence numbers when committed. As described above, overlapping
`RangeKeySet`s and `RangeKeyUnset`s are fragmented to have matching start and
end bounds. Then the resulting exactly-overlapping range key fragments are
merged into a single internal key-value pair, within the same snapshot stripe
and sstable. The original, unmerged internal keys each have their own sequence
numbers, indicating the moment they were committed within the history of all
write operations.

Recall that sequence numbers are used within Pebble to determine which keys
appear live to which iterators. When an iterator is constructed, it takes note
of the current _visible sequence number_, and for the lifetime of the iterator,
only surfaces keys less than that sequence number. Similarly, snapshots read the
current _visible sequence number_, remember it, but also leave a note asking
compactions to preserve history at that sequence number. The space between
snapshotted sequence numbers is referred to as a _snapshot stripe_, and
operations cannot drop or otherwise mutate keys unless they fall within the same
_snapshot stripe_. For example a `k.MERGE#5` key may not be merged with a
`k.MERGE#1` operation if there's an open snapshot at `#3`.

The new `RangeKeySet`, `RangeKeyUnset` and `RangeKeyDelete` keys behave
similarly. Overlapping range keys won't be merged if there's an open snapshot
separating them. Consider a range key `a-z` written at sequence number `#1` and
a point key `d.SET#2`. A combined point-and-range iterator using a sequence
number `#3` and positioned at `d` will surface both the range key `a-z` and the
point key `d`.

In the context of masking, the suffix-based masking of range keys can cause
potentially unexpected behavior. A range key `[a,z)@10` may be committed as
sequence number `#1`. Afterwards, a point key `d@5#2` may be committed. An
iterator that is configured with range-key masking with suffix `@20` would mask
the point key `d@5#2` because although `d@5#2`'s sequence number is higher,
range-key masking uses suffixes to impose order, not sequence numbers.

### Boundaries for sstables

Range keys follow the same relationship to sstable bounadries as the existing
`RANGEDEL` tombstones. The bounds of an internal range key are user keys. Every
range key is limited by its containing sstable's bounds.

Consider these keys, annotated with sequence numbers:

```
Point keys: a#50, b#70, b#49, b#48, c#47, d#46, e#45, f#44
Range key: [a,e)#60
```

We have created three versions of `b` in this example. In previous versions,
Pebble could split output sstables during a compaction such that the different
`b` versions span more than one sstable. This creates problems for `RANGEDEL`s
which span these two sstables which are discussed in the section on [improperly
truncated RANGEDELS](https://github.com/cockroachdb/pebble/blob/master/docs/range_deletions.md#improperly-truncated-range-deletes).
We manage to tolerate this for `RANGEDEL`s since their semantics are defined by
the system, which is not true for these range keys where the actual semantics
are up to the user.

Pebble now disallows such sstable split points. In this example, by postponing
the sstable split point to the user key c, we can cleanly split the range key
into `[a,c)#60` and `[c,e)#60`. The sstable end bound for the first sstable
(sstable bounds are inclusive) will be c#inf (where inf is the largest possible
seqnum, which is unused except for these cases), and sstable start bound for the
second sstable will be c#60.

The above example deals exclusively with point and range keys without suffixes.
Consider this example with suffixed keys, and compaction outputs split in the
middle of the `b` prefix:

```
first sstable: points: a@100, a@30, b@100, b@40 ranges: [a,c)@50
second sstable: points: b@30, c@40, d@40, e@30, ranges: [c,e)@50
```

When the compaction code decides to defer `b@30` to the next sstable and finish
the first sstable, the range key `[a,c)@50` is sitting in the fragmenter. The
compaction must split the range key at the bounds determined by the user key.
The compaction uses the first point key of the next sstable, in this case
`b@30`, to truncate the range key. The compaction flushes the fragment
`[a,b@30)@50` to the first sstable and updates the existing fragment to begin at
`b@30`.

If a range key extends into the next file, the range key's truncated end is used
for the purposes of determining the sstable end boundary. The first sstable's
end boundary becomes `b@30#inf`, signifying the range key does not cover `b@30`.
The second sstable's start boundary is `b@30`.

### Block property collectors

Separate block property collectors may be configured to collect separate
properties about range keys. This is necessary for CockroachDB's MVCC block
property collectors to ensure the sstable-level properties are correct.

### Iteration

This design extends the `*pebble.Iterator` with the ability to iterate over
exclusively range keys, range keys and point keys together or exclusively point
keys (the previous behavior).

- Pebble already requires that the prefix `k` follows the same key validity
  rules as `k@suffix`.

- Previously, Pebble did not require that a user key consisting of just a prefix
  `k` sort before the same prefix with a non-empty suffix. CockroachDB has
  adopted this behavior since it results in the following clean behavior:
  `RANGEDEL` over [k1, k2) deletes all versioned keys which have prefixes in the
  interval [k1, k2). Pebble will now require this behavior for all users using
  MVCC keys. Specifically, it must hold that `Compare(k[:Split(k)], k) < 0` if
  `Split(k) < len(k)`.

# TKTK: Discuss merging iterator

#### Determinism

Range keys will be split based on boundaries of sstables in an LSM. Users of an
LSM typically expect that two different LSMs with different sstable settings
that receive the same writes should output the same key-value pairs when
iterating. To provide this behavior, the iterator implementation may be
configured to defragment range keys during iteration time. The defragmentation
behavior would be:

- Two visible ranges `[k1,k2)@suffix1=>val1`, `[k2,k3)@suffix2=>val2` are
  defragmented if suffix1==suffix2 and val1==val2, and become [k1,k3).

- Defragmentation during user iteration does not consider the sequence number.
  This is necessary since LSM state can be exported to another LSM via the use
  of sstable ingestion, which can collapse different seqnums to the same seqnum.
  We would like both LSMs to look identical to the user when iterating.

The above defragmentation is conceptually simple, but hard to implement
efficiently, since it requires stepping ahead from the current position to
defragment range keys. This stepping ahead could switch sstables while there are
still points to be consumed in a previous sstable. This determinism is useful
for testing and verification purposes:

- Randomized and metamorphic testing is used extensively to reliably test
  software including Pebble and CockroachDB. Defragmentation provides
  the determinism necessary for this form of testing.

- CockroachDB's replica divergence detector requires a consistent view of the
  database on each replica.

In order to provide determinism, Pebble constructs an internal range key
iterator stack that's separate from the point iterator stack, even when
performing combined iteration over both range and point keys. The separate range
key iterator allows the internal range key iterator to move independently of the
point key iterator. This allows the range key iterator to independently visit
adjacent sstables in order to defragment their range keys if necessary, without
repositioning the point iterator.

Two spans [k1,k2) and [k3, k4) of range keys are defragmented if their bounds
abut and their user observable-state is identical. That is, `k2==k3` and each
spans' contains exactly the same set of range key (<suffix>, <tuple>) pairs.  In
order to support `RangeKeyUnset` and `RangeKeyDelete`, defragmentation must be
applied _after_ resolving unset and deletes.

#### Merging iteration

Recall that range keys are stored in the same sstables as point keys. In a
log-structured merge tree, these sstables are distributed across levels. Within
a level, sstables are non-overlapping but between levels sstables may overlap
arbitrarily. During iteration, keys across levels must be merged together. For
point keys, this is typically done with a heap.

Range keys too must be merged across levels, and the earlier described
fragmentation at intersection boundaries must be applied. To implement this, a
range key merging iterator is defined.

A merging iterator is initialized with an arbitrary number of child iterators
over fragmented spans. Each child iterator exposes fragmented range keys, such
that overlapping range keys are surfaced in a single span with a single set of
bounds. Range keys from one child iterator may overlap key spans from another
child iterator arbitrarily. The high-level algorithm is:

1. Initialize a heap with bound keys from child iterators' range keys.
2. Find the next [or previous, if in reverse] two unique user keys' from bounds.
3. Consider the span formed between the two unique user keys a candidate span.
4. Determine if any of the child iterators' spans overlap the candidate span.
  4a. If any of the child iterator's current bounds are end keys (during
      forward iteration) or start keys (during reverse iteration), then all the
      spans with that bound overlap the candidate span.
  4b. If no spans overlap, forget the smallest (forward iteration) or largest
      (reverse iteration) unique user key and advance the iterators to the next
      unique user key. Start again from 3.

Consider the example:

```
       i0:     b---d e-----h
       i1:   a---c         h-----k
       i2:   a------------------------------p

fragments:   a-b-c-d-e-----h-----k----------p
```

None of the individual child iterators contain a span with the exact bounds
[c,d), but the merging iterator must produce a span [c,d). To accomplish this,
the merging iterator visits every span between unique boundary user keys. In the
above example, this is:

```
[a,b), [b,c), [c,d), [d,e), [e, h), [h, k), [k, p)
```

The merging iterator first initializes the heap to prepare for iteration. The
description below discusses the mechanics of forward iteration after a call to
First, but the mechanics are similar for reverse iteration and other positioning
methods.

During a call to First, the heap is initialized by seeking every level to the
first bound of the first fragment. In the above example, this seeks the child
iterators to:

```
i0: (b, boundKindStart, [ [b,d) ])
i1: (a, boundKindStart, [ [a,c) ])
i2: (a, boundKindStart, [ [a,p) ])
```

After fixing up the heap, the root of the heap is the bound with the smallest
user key ('a' in the example). During forward iteration, the root of the heap's
user key is the start key of next merged span. The merging iterator records this
key as the start key. The heap may contain other levels with range keys that
also have the same user key as a bound of a range key, so the merging iterator
pulls from the heap until it finds the first bound greater than the recorded
start key.

In the above example, this results in the bounds `[a,b)` and child iterators in
the following positions:

```
i0: (b, boundKindStart, [ [b,d) ])
i1: (c, boundKindEnd,   [ [a,c) ])
i2: (p, boundKindEnd,   [ [a,p) ])
```

With the user key bounds of the next merged span established, the merging
iterator must determine which, if any, of the range keys overlap the span.
During forward iteration any child iterator that is now positioned at an end
boundary has an overlapping span. (Justification: The child iterator's end
boundary is ≥ the new end bound. The child iterator's range key's corresponding
start boundary must be ≤ the new start bound since there were no other user keys
between the new span's bounds. So the fragments associated with the iterator's
current end boundary have start and end bounds such that start ≤ <new start
bound> < <new end bound> ≤ end).

The merging iterator iterates over the levels, collecting keys from any child
iterators positioned at end boundaries. In the above example, i1 and i2 are
positioned at end boundaries, so the merging iterator collects the keys of [a,c)
and [a,p). These spans contain the merging iterator's [a,b) span, but they may
also extend beyond the new span's start and end. The merging iterator returns
the keys with the new start and end bounds, preserving the underlying keys'
sequence numbers, key kinds and values.

It may be the case that the merging iterator finds no levels positioned at span
end boundaries in which case the span overlaps with nothing. In this case the
merging iterator loops, repeating the above process again until it finds a span
that does contain keys.

#### Efficient masking

Recollect that in the earlier example from the iteration interface, during
forward iteration an iterator would output the following keys:

```
  Key()   HasPointAndRange()   Value()      RangeKeyBounds()    RangeKeys()
  a       (true,  true)        artichoke    [a,b)               {(@1,apple)}
  b       (false, true)        -            [b,c)               {(@7,kiwi), (@1,apple)}
  b@2     (true,  true)        beet         [b,c)               {(@7,kiwi), (@1,apple)}
  c       (false, true)        -            [c,e)               {(@7,kiwi), (@3,banana), (@1,apple)}
  e       (false, true)        -            [e,k)               {(@7,kiwi), (@5,orange), (@1,apple)}
  k       (false, true)        -            [k,m)               {(@5,orange), (@1,apple)}
  m       (false, true)        -            [m,z)               {(@1,apple)}
  t@3     (true,  true)        turnip       [m,z)               {(@1,apple)}
```

When implementing an MVCC "soft delete range" operation using range keys, the
range key `[b,k)@7→kiwi` may represent that all keys within the range [b,k) are
deleted at MVCC timestamp @7. During iteration, it would be desirable if the
caller could indicate that it does not want to observe any "soft deleted" point
keys, and the iterator can safely skip them. Note that in a MVCC system, whether
or not a key is soft deleted depends on the timestamp at which the database is
read.

This is implemented through "range key masking," where a range key may act as a
mask, hiding point keys with MVCC timestamps beneath the range key. This
iterator option requires that the client configure the iterator with a MVCC
timestamp `suffix` representing the timestamp at which history should be read.
All range keys with suffixes (MVCC timestamps) less than or equal to the
configured suffix serve as masks. All point keys with suffixes (MVCC timestamps)
less than a covering, masking range key's suffix are hidden.

Specifically, when configured with a RangeKeyMasking.Suffix _s_, and there
exists a range key with suffix _r_ covering a point key with suffix _p_, and _s_
≤ _r_ < _p_ then the point key is elided.

In the above example, if `RangeKeyMasking.Suffix` is set to `@7`, every range
key serves as a mask and the point key `b@2` is hidden during iteration because
it's contained within the masking `[b,k)@7→kiwi` range key. Note that `t@3`
would _not_ be masked, because its timestamp `@3` is more recent than the only
range key that covers it (`[a,z)@1→apple`).

If `RangeKeyMasking.Suffix` were set to `@6` (a historical, point-in-time read),
the `[b,k)@7→kiwi` range key would no longer serve as a mask, and `b@2` would be
visible.

To efficiently implement masking, we cannot rely on the LSM invariant since
`b@100` can be at a lower level than `[a,e)@50`. Instead, we build on
block-property filters, supporting special use of a MVCC timestamp block
property in order to skip blocks wholly containing point keys that are masked by
a range key. The client may configure a block-property collector to record the
highest MVCC timestamps of point keys within blocks.

During read time, when positioned within a range key with a suffix ≤
`RangeKeyMasking.Suffix`, the iterator configures sstable readers to use a
block-property filter to skip any blocks for which the highest MVCC timestamp is
less than the provided suffix. Additionally, these iterators must consult index
block bounds to ensure the block-property filter is not applied beyond the
bounds of the masking range key.

### CockroachDB use

CockroachDB initially will only use range keys to represent MVCC range
tombstones. See the MVCC range tombstones tech note for more details:

https://github.com/cockroachdb/cockroach/blob/master/docs/tech-notes/mvcc-range-tombstones.md

### Alternatives

#### A1. Automatic elision of range keys that don't cover keys

We could decide that range keys:

- Don't contribute to `MVCCStats` themselves.
- May be elided by Pebble when they cover zero point keys.

This means that CockroachDB garbage collection does not need to explicitly
remove the range keys, only the point keys they deleted. This option is clean
when paired with `RANGEDEL`s dropping both point and range keys. CockroachDB can
issue `RANGEDEL`s whenever it wants to drop a contiguous swath of points, and
not worry about the fact that it might also need to update the MVCC stats for
overlapping range keys.

However, this option makes deterministic iteration over defragmented range keys
for replica divergence detection challenging, because internal fragmentation may
elide regions of a range key at any point.  Producing a normalized form would
require storing state in the value (ie, the original start key) and
recalculating the smallest and largest extant covered point keys within the
range key and replica bounds. This would require maintaining _O_(range-keys)
state during the `storage.ComputeStatsForRange` pass over a replica's combined
point and range iterator.

This likely forces replica divergence detection to use other means (eg, altering
the checksum of covered points) to incorporate MVCC range tombstone state.

This option is also highly tailored to the MVCC Delete Range use case.  Other
range key usages, like ranged intents, would not want this behavior, so we don't
consider it further.

#### A2. Separate LSM of range keys

There are two viable options for where to store range keys. They may be encoded
within the same sstables as points in separate blocks, or in separate sstables
forming a parallel range-key LSM. We examine the tradeoffs between storing range
keys in the same sstable in different blocks ("shared sstables") or separate
sstables forming a parallel LSM ("separate sstables"):

- Storing range keys in separate sstables is possible because the only
  iteractions between range keys and point keys happens at a global level.
  Masking is defined over suffixes. It may be extended to be defined over
  sequence numbers too (see 'Sequence numbers' section below), but that is
  optional. Unlike range deletion tombstones, range keys have no effect on point
  keys during compactions.

- With separate sstables, reads may need to open additional sstable(s) and read
  additional blocks. The number of additional sstables is the number of nonempty
  levels in the range-key LSM, so it grows logarithmically with the number of
  range keys. For each sstable, a read must read the index block and a data
  block.

- With our expectation of few range keys, the range-key LSM is expected to be
  small, with one or two levels. Heuristics around sstable boundaries may
  prevent unnecessary range-key reads when there is no covering range key. Range
  key sstables and blocks are expected to have much higher table and block cache
  hit rates, since they are orders of magnitude less dense. Reads in any
  overlapping point sstables all access the same range key sstables.

- With shared sstables, `SeekPrefixGE` cannot use bloom filters to entirely
  eliminate sstables that contain range keys. Pebble does not always use bloom
  filters in L6, so once a range key is compacted into L6 its impact to
  `SeekPrefixGE` is lessened. With separate sstables, `SeekPrefixGE` can always
  use bloom filters for point-key sstables. If there are any overlapping
  range-key sstables, the read must read them.

- With shared sstables, range keys create dense sstable boundaries. A range key
  spanning an sstable boundary leaves no gap between the sstables' bounds. This
  can force ingested sstables into higher levels of the LSM, even if the
  sstables' point key spans don't overlap. This problem was previously observed
  with wide `RANGEDEL` tombstones and was mitigated by prioritizing compaction
  of sstables that contain `RANGEDEL` keys. We could do the same with range
  keys, but the write amplification is expected to be much worse. The `RANGEDEL`
  tombstones drop keys and eventually are dropped themselves as long as there is
  not an open snapshot. Range keys do not drop data and are expected to persist
  in L6 for long durations, always requiring ingested sstables to be inserted
  into L5 or above.

- With separate sstables, compaction logic is separate, which helps avoid
  complexity of tricky sstable boundary conditions. Because there are expected
  to be an order of magnitude fewer range keys, we could impose the constraint
  that a prefix cannot be split across multiple range key sstables. The
  simplified compaction logic comes at the cost of higher levels, iterators, etc
  all needing to deal with the concept of two parallel LSMs.

- With shared sstables, the LSM invariant is maintained between range keys and
  point keys. For example, if the point key `b@20` is committed, and
  subsequently a range key `RangeKey([a,c), @25, ...)` is committed, the range
  key will never fall below the covered point `b@20` within the LSM.

We decide to share sstables, because preserving the LSM invariant between range
keys and point keys is expected to be useful in the long-term.

#### A3. Sequence number masking

In the CockroachDB MVCC range tombstone use case, a point key should never be
written below an existing range key with a higher timestamp. The MVCC range
tombstone use case would allow us to dictate that an overlapping range key with
a higher sequence number always masks range keys with lower sequence numbers.
Adding this additional masking scope would avoid the comparatively costly suffix
comparison when a point key _is_ masked by a range key. We need to consider how
sequence number masking might be affected by the merging of range keys within
snapshot stripes.

Consider the committing of range key `[a,z)@{t1}#10`, followed by point keys
`d@t2#11` and `m@t2#11`, followed by range key `[j,z)@{t3}#12`.  This sequencing
respects the expected timestamp, sequence number relationship in CockroachDB's
use case. If all keys are flushed within the same sstable, fragmentation and
merging overlapping fragments yields range keys `[a,j)@{t1}#10`,
`[j,z)@{t3,t1}#12`. The key `d@t2#11` must not be masked because it's not
covered by the new range key, and indeed that's the case because the covering
range key's fragment is unchanged `[a,j)@{t1}#10`.

For now we defer this optimization, with the expectation that we may not be able
to preserve this relationship between sequence numbers and suffixes in all range
key use cases.
