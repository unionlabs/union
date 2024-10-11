# Range Deletions

TODO: The following explanation of range deletions does not take into account
the recent change to prohibit splitting of a user key between sstables. This
change simplifies the logic, removing 'improperly truncated range tombstones.'

TODO: The following explanation of range deletions ignores the
kind/trailer that appears at the end of keys after the sequence
number. This should be harmless but need to add a justification on why
it is harmless.

## Background and Notation

Range deletions are represented as `[start, end)#seqnum`. Points
(set/merge/...) are represented as `key#seqnum`. A range delete `[s, e)#n1`
deletes every point `k#n2` where `k \in [s, e)` and `n2 < n1`.
The inequality `n2 < n1` is to handle the case where a range delete and
a point have the same sequence number -- this happens during sstable
ingestion where the whole sstable is assigned a single sequence number
that applies to all the data in it.

There is additionally an infinity sequence number, represented as
`inf`, which is not used for any point, that we can use for reasoning
about range deletes.

It has been asked why range deletes use an exclusive end key instead
of an inclusive end key. For string keys, one can convert a desired
range delete on `[s, e]` into a range delete on `[s, ImmediateSuccessor(e))`.
For strings, the immediate successor of a key
is that key with a \0 appended to it. However one cannot go in the
other direction: if one could represent only inclusive end keys in a
range delete and one desires to delete a range with an exclusive end
key `[s, e)#n`, one needs to compute `ImmediatePredecessor(e)` which
is an infinite length string. For example,
`ImmediatePredecessor("ab")` is `"aa\xff\xff...."`. Additionally,
regardless of user needs, the exclusive end key helps with splitting a
range delete as we will see later. 

We will sometimes use ImmediatePredecessor and ImmediateSuccessor in
the following for illustrating an idea, but we do not rely on them as
something that is viable to produce for a particular kind of key. And
even if viable, these functions are not currently provided to
RockDB/Pebble.

### Visualization

If we consider a 2 dimensional space with increasing keys on the X
axis (with every possible user key represented) and increasing
sequence numbers on the Y axis, range deletes apply to a rectangle
whose bottom edge sits on the X axis.

The actual space represented by the ordering in our sstables is a one
dimensional space where `k1#n1` is less than `k2#n2` if either of the
following holds:

- k1 < k2

- k1 = k2 and n1 > n2 (under the assumption that no two points with
the same key have the same sequence number).

```
  ^
  |   .       > .        > .        > yy
  |   .      >  .       >  .       >  .
  |   .     >   .      >   .      >   .
n |   V    >    xx    >    .     >    V
  |   .   >     x.   >     x.   >     . 
  |   .  >      x.  >      x.  >      .
  |   . >       x. >       x. >       .
  |   .>        x.>        x.>        .
  ------------------------------------------>
                k        IS(k)    IS(IS(k))
```

The above figure uses `.` to represent points and the X axis is dense in
that it represents all possible keys. `xx` represents the start of a
range delete and `x.` are the points which it deletes. The arrows `V` and
`>` represent the ordering of the points in the one dimensional space.
`IS` is shorthand for `ImmediateSuccessor` and the range delete represented
there is `[k, IS(IS(k)))#n`. Ignore `yy` for now.

The one dimensional space works fine in a world with only points. But
issues arise when storing range deletes, that represent an action in 2
dimensional space, into this one dimensional space.

## Range Delete Boundaries and the Simplest World

RocksDB/Pebble store the inclusive bounds of each sstable in one dimensional
space. The range deletes two dimensional behavior and exclusive end key needs
to be adapted to this requirement. For a range delete `[s, e)#n`,
the smallest key it acts on is `s#(n-1)` and the largest key it
acts on is `ImmediatePredecessor(e)#0`. So if we position the range delete
immediately before the smallest key it acts on and immediately after
the largest key it acts on we can give it a tight inclusive bound of
`[s#n, e#inf]`.  

Note again that this range delete does not delete everything in its
inclusive bound. For example, range delete `["c", "h")#10` has a tight
inclusive bound of `["c"#10, "h"#inf]` but does not delete `"d"#11`
which lies in that bound. Going back to our earlier diagram, the one
dimensional inclusive bounds go from the `xx` to `yy` but there are
`.`s in between, in the one dimensional order, that are not deleted.

This is the reason why one cannot in general
use a range delete to seek over all points within its bounds. The one
exception to this seeking behaviour is that when we can order sstables
from new to old, one can "blindly" use this range delete in a newer
sstable to seek to `"h"` in all older sstables since we know those
older sstables must only have point keys with sequence numbers `< 10`
for the keys in interval `["c", "h")`. This partial order across
sstables exists in RocksDB/Pebble between memtable, L0 sstables (where
it is a total order) and across sstables in different levels.

Coming back to the inclusive bounds of the range delete, `[s#n, e#inf]`:
these bounds participate in deciding the bounds of the
sstable. In this world, one can read all the entries in an sstable and
compute its bounds. However being able to construct these bounds by
reading an sstable is not essential -- RocksDB/Pebble store these
bounds in the `MANIFEST`. This latter fact has been exploited to
construct a real world (later section) where the bounds of an sstable
are not computable by reading all its keys.

If we had a system with one sstable per level, for each level lower
than L0, we are effectively done. We have represented the tight bounds
of each range delete and it is within the bounds of the sstable. This
works even with L0 => L0 compactions assuming they output exactly one
sstable.

## The Mostly Simple World

Here we have multiple files for levels lower than L0 that are non
overlapping in the file bounds. These multiple files occur because
compactions produce multiple files. This introduces the need to split a
range delete across the files being produced by a compaction.

There is a clean way to split a range delete `[s, e)#n` into 2 parts
(which can be recursively applied to split into arbitrarily many
parts): split into `[s, m)#n` and `[m, e)#n`. These range deletes
apply to non-overlapping points and their tight bounds are `[s#m,
m#inf]`, `[m#n, e#inf]` which are also non-overlapping.

Consider the following example of an input range delete `["c", "h")#10` and
the following two output files from a compaction:

```
          sst1            sst2
last point is "e"#7 | first point is "f"#20
```

The range delete can be split into `["c", "f")#10` and `["f",
"h")#10`, by using the first point key of sst2 as the split
point. Then the bounds of sst1 and sst2 will be `[..., "f"#inf]` and
`["f"#20, ...]` which are non-overlapping. It is still possible to compute
the sstable bounds by looking at all the entries in the sstable.

## The Real World

Continuing with the same range delete `["c", "h")#10`, we can have the
following sstables produced during a compaction:

```
         sst1       sst2         sst3        sst4     sst5
points: "e"#7 | "f"#12 "f"#7 | "f"#4 "f"#3 | "f"#1 | "g"#15
```

The range deletes written to these ssts are

```
      sst1           sst2            sst3           sst4          sst5
["c", "h")#10 | ["f", "h")#10 | ["f", "h")#10 | ["f", "h")#10 | ["g", "h")#10
```

The Pebble code that achieves this effect is in
`rangedel.Fragmenter`. It is a code structuring artifact that sst1
does not contain a range delete equal to `["c", "f")#10` and sst4 does
not contain `["f", "g")#10`. However for the range deletes in sst2 and
sst3 we cannot do any better because we don't know what the key
following "f" will be (the compaction cannot look ahead) and because
we don't have an `ImmediateSuccessor` function (otherwise we could
have written `["f", ImmediateSuccessor("f"))#10` to sst2, sst3). But
the code artifacts are not the ones introducing the real complexity.

The range delete bounds are

```
      sst1        sst2, sst3, sst4          sst5
["c"#10, "h"#inf] ["f"#10, "h"#inf]   ["g"#10, "h"#inf]

```

We note the following:

- The bounds of range deletes are overlapping since we have been
  unable to split the range deletes. If these decide the sstable
  bounds, the sstables will have overlapping bounds. This is not
  permissible.

- The range deletes included in each sstable result in that sstable
  being "self-sufficient" wrt having the range delete that deletes
  some of the points in the sstable (let us assume that the points in
  this example have not been dropped from that sstable because of a
  snapshot).

- The transitions from sst1 to sst2 and sst4 to sst5 are **clean** in
  that we can pretend that the range deletes in those files are actually:

```
      sst1           sst2            sst3           sst4          sst5
["c", "f")#10 | ["f", "g")#10 | ["f", "g")#10 | ["f", "g")#10 | ["g", "h")#10
```

We could achieve some of these **clean** transitions (but not all) with a
code change. Also note that these better range deletes maintain the
"self-sufficient" property.

### Making Non-overlapping SSTable bounds

We force the sstable bounds to be non-overlapping by setting them to:

```
      sst1              sst2           sst3            sst4              sst5
["c"#10, "f"#inf] ["f"#12, "f"#7] ["f"#4, "f"#3] ["f"#1, "g"#inf] ["g"#15, "h"#inf]
```

Note that for sst1...sst4 the sstable bounds are smaller than the
bounds of the range deletes contained in them. The code that
accomplishes this is Pebble is in `compaction.go` -- we will not discuss the
details of that logic here but note that it is placing an `inf`
sequence number for a clean transition and for an unclean transition
it is using the point keys as the bounds.

Associated with these narrower bounds, we add the following
requirement: a range delete in an sstable must **act-within** the bounds of
the sstable it is contained in. In the above example:

- sst1: range delete `["c", "h")#10` must act-within the bound `["c"#10, "f"#inf]`

- sst2: range delete `["f", "h")#10` must act-within the bound `["f"#12, "f"#7]`

- sst3: range delete `["f", "h")#10` must act-within the bound `["f"#4, "f"#3]`

- sst4: range delete `["f", "h")#10` must act-within the bound ["f"#1, "g"#inf]

- And so on.

The intuitive reason for the **act-within** requirement is that 
sst5 can be compacted and moved down to a lower level independent of
sst1-sst4, since it was at a **clean** boundary. We do not want the
range delete `["f", "h")#10` sitting in sst1...sst4 at the higher
level to act on `"g"#15` that has been moved to the lower level. Note
that this incorrect action can happen due to 2 reasons:
  
1. the invariant that lower levels have older data for keys
   that also exist in higher levels means we can (a) seek a lower level
   sstable to the end of a range delete from a higher level, (b) for a key
   lookup, stop searching in lower levels once a range delete is encountered
   for that key in a higher level.
  
2. Sequence number zeroing logic can change the sequence number of
  `"g"#15` to `"g"#0` (for better compression) once it realizes that
   there are no older versions of `"g"`. It would be incorrect for this
  `"g"#0` to be deleted.  


#### Loss of Power

This act-within behavior introduces some "loss of power" for
the original range delete `["c", "h")#10`. By acting within sst2...sst4
it can no longer delete keys `"f"#6`, `"f"#5`, `"f"#2`.

Luckily for us, this is harmless since these keys cannot have existed
in the system due to the levelling behavior: we cannot be writing
sst2...sst4 to level `i` if versions of `"f"` younger than `"f"#4` are
already in level `i` or version older than `"f"#7` have been left in
level i - 1. There is some trickery possible to prevent this "loss of
power" for queries (see the "Putting it together" section), but given
the history of bugs in this area, we should be cautious.

### Improperly truncated Range Deletes

We refer to range deletes that have experienced this "loss of power"
as **improper**. In the above example the range deletions in sst2, sst3, sst4
are improper. The problem with improper range deletions occurs
when they need to participate in a future compaction: even though we
have restricted them to act-within their current sstable boundary, we
don't have a way of **"writing"** this restriction to a new sstable,
since they still need to be written in the `[s, e)#n` format.

For example, sst2 has delete `["f", "h")#10` that must act-within
the bound `["f"#12, "f"#7]`. If sst2 was compacted down to the next
level into a new sstable (whose bounds we cannot predict because they
depend on other data written to that sstable) we need to be able to
write a range delete entry that follows the original restriction. But
the narrowest we can write is `["f", ImmediateSuccessor("f"))#10`. This
is an expansion of the act-within restriction with potentially
unintended consequences. In this case the expansion happened in the suffix.
For sst4, the range deletion `["f", "h")#10` must act-within `["f"#1, "g"#inf]`,
and we can precisely represent the constraint on the suffix by writing
`["f", "g")#10` but it does not precisely represent that this range delete
should not apply to `"f"#9`...`"f"#2`.

In comparison, the sst1 range delete `["c", "h")#10` that must act-within
the bound `["c"#10, "f"#inf]` is not improper. This restriction can
be applied precisely to get a range delete `["c", "f")#10`. 

The solution to this is to note that while individual sstables have
improper range deletes, if we look at a collection of sstables we
can restore the improper range deletes spread across them to their proper self
(and their full power). To accurately find these improper range
deletes would require looking into the contents of a file, which is
expensive. But we can construct a pessimistic set based on
looking at the sequence of all files in a level and partitioning them:
adjacent files `f1`, `f2` with largest and smallest bound `k1#n1`,
`k2#n2` must be in the same partition if

```
k1 = k2 and n1 != inf
```

In the above example sst2, sst3, sst4 are one partition. The
**spanning bound** of this partition is `["f"#12, "g"#inf]` and the
range delete `["f", "h")#10` when constrained to act-within this
spanning bound is precisely the range delete `["f",
"g")#10`. Intuitively, the "loss of power" of this range delete has
been restored for the sake of making it proper, so it can be
accurately "written" in the output of the compaction (it may be
improperly fragmented again in the output, but we have already
discussed that). Such partitions are called "atomic compaction groups"
and must participate as a whole in a compaction (and a
compaction can use multiple atomic compaction groups as input).

Consider another example:

```
          sst1              sst2
points:  "e"#12         |  "e"#10
delete: ["c", "g")#8    | ["c", "g")#8
bounds  ["c"#8, "e"#12] | ["e"#10, "g"#inf]
```

sst1, sst2 are an atomic compaction group. Say we violated the
requirement that both be inputs in a compaction and only compacted
sst2 down to level `i + 1` and then down to level `i + 2`. Then we add
sst3 with bounds `["h"#10, "j"#5]` to level `i` and sst1 and sst3 are
compacted to level `i + 1` into a single sstable. This new sstable
will have bounds `["c"#8, "j"#5]` so these bounds do not help with the
original apply-witin constraint on `["c", "g")#8` (that it should
apply-within `["c"#8, "e"#12]`). The narrowest we can construct (if we had
`ImmediateSuccessor`) would be `["c", ImmediateSuccessor("e"))#8`.  Now we
can incorrectly apply this range delete that is in level `i + 1` to `"e"#10`
sitting in level `i + 2`. Note that this example can be made worse using
sequence number zeroing -- `"e"#10` may have been rewritten to `"e"#0`.  

If a range delete `[s, e)#n` is in an atomic compaction group with
spanning bounds `[k1#n1, k2#n2]` our construction above guarantees the
following properties

- `k1#n1 <= s#n`, so the bounds do not constrain the start of the
  range delete.

- `k2 >= e` or `n2 = inf`, so if `k2` is constraining the range delete
  it will properly truncate the range delete.


#### New sstable at sequence number 0

A new sstable can be assigned sequence number 0 (and be written to L0)
if the keys in the sstable are not in any other sstable. This
comparison uses the keys and not key#seqnum, so the loss and
restoration of power does not cause problems since that occurs within
the versions of a single key.

#### Flawed optimizations

For the case where the atomic compaction group correspond to the lower
level of a compaction, it may initially seem to be correct to use only
a prefix or suffix of that group in a compaction. In this case the
prefix (suffix) will correspond to the largest key (smallest key) in
the input sstables in the compaction and so can continue to constrain
the range delete.  For example, sst1 and sst2 are in the same atomic
compaction group

```
          sst1               sst2
points: "c"#10 "e"#12    |  "e"#10
delete: ["c", "g")#8     | ["c", "g")#8
bounds  ["c"#10, "e"#12] | ["e"#10, "g"#inf]
```

and this is the lower level of a compaction with

```
          sst3
points: "a"#14 "d"#15
bounds  ["a"#14, "d"#15]
```

we could allow for a compaction involving sst1 and sst3 which would produce

```
          sst4
points: "a"#14 "c"#10 "d"#15 "e"#12
delete: ["c", "g")#8
bounds  ["a"#14, "e"#12]
```

and the range delete is still improper but its act-within constraint has
not expanded.

But we have to be very careful to not have a more significant loss of power
of this range delete. Consider a situation where sst3 had a single delete
`"e"#16`. It still does not overlap in bounds with sst2 and we again pick
sst1 and sst3 for compaction. This single delete will cause `"e"#12` to be deleted
and sst4 bounds would be (unless we had complicated code preventing it):

```
          sst4
points: "a"#14 "c"#10 "d"#15
delete: ["c", "g")#8
bounds  ["a"#14, "d"#15]
```

Now this delete cannot delete `"dd"#6` and we have lost the ability to know
that sst4 and sst2 are in the same atomic compaction group.


### Putting it together

Summarizing the above, we have:

- SStable bounds logic that ensures sstables are not
overlapping. These sstables contain range deletes that extend outside
these bounds. But these range deletes should **apply-within** the
sstable bounds.

- Compactions: they need to constrain the range deletes in the inputs
to **apply-within**, but this can create problems with **writing** the
**improper** range deletes. The solution is to include the full
**atomic compaction group** in a compaction so we can restore the
**improper** range deletes to their **proper** self and then apply the
constraints of the atomic compaction group.

- Queries: We need to act-within the file bound constraint on the range delete.
  Say the range delete is `[s, e)#n` and the file bound is `[b1#n1,
  b2#n2]`. We are guaranteed that `b1#n1 <= s#n` so the only
  constraint can come from `b2#n2`.
  
  - Deciding whether a range delete covers a key in the same or lower levels.

    - `b2 >= e`: there is no act-within constraint.
    - `b2 < e`: to be precise we cannot let it delete `b2#n2-1` or
      later keys. But it is likely that allowing it to delete up to
      `b2#0` would be ok due to the atomic compaction group. This
      would prevent the so-called "loss of power" discussed earlier if
      one also includes the argument that the gap in the file bounds
      that also represents the loss of power is harmless (the gap
      exists within versions of key, and anyone doing a query for that
      key will start from the sstable to the left of the gap). But it
      may be better to be cautious.

  - For using the range delete to seek sstables at lower levels.
    - `b2 >= e`: seek to `e` since there is no act-within constraint.
    - `b2 < e`: seek to `b2`. We are ignoring that this range delete
      is allowed to  delete some versions of `b2` since this is just a
      performance optimization.






