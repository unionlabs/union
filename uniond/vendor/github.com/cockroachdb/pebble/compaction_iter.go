// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"sort"
	"strconv"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/bytealloc"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/rangekey"
	"github.com/cockroachdb/redact"
)

// compactionIter provides a forward-only iterator that encapsulates the logic
// for collapsing entries during compaction. It wraps an internal iterator and
// collapses entries that are no longer necessary because they are shadowed by
// newer entries. The simplest example of this is when the internal iterator
// contains two keys: a.PUT.2 and a.PUT.1. Instead of returning both entries,
// compactionIter collapses the second entry because it is no longer
// necessary. The high-level structure for compactionIter is to iterate over
// its internal iterator and output 1 entry for every user-key. There are four
// complications to this story.
//
// 1. Eliding Deletion Tombstones
//
// Consider the entries a.DEL.2 and a.PUT.1. These entries collapse to
// a.DEL.2. Do we have to output the entry a.DEL.2? Only if a.DEL.2 possibly
// shadows an entry at a lower level. If we're compacting to the base-level in
// the LSM tree then a.DEL.2 is definitely not shadowing an entry at a lower
// level and can be elided.
//
// We can do slightly better than only eliding deletion tombstones at the base
// level by observing that we can elide a deletion tombstone if there are no
// sstables that contain the entry's key. This check is performed by
// elideTombstone.
//
// 2. Merges
//
// The MERGE operation merges the value for an entry with the existing value
// for an entry. The logical value of an entry can be composed of a series of
// merge operations. When compactionIter sees a MERGE, it scans forward in its
// internal iterator collapsing MERGE operations for the same key until it
// encounters a SET or DELETE operation. For example, the keys a.MERGE.4,
// a.MERGE.3, a.MERGE.2 will be collapsed to a.MERGE.4 and the values will be
// merged using the specified Merger.
//
// An interesting case here occurs when MERGE is combined with SET. Consider
// the entries a.MERGE.3 and a.SET.2. The collapsed key will be a.SET.3. The
// reason that the kind is changed to SET is because the SET operation acts as
// a barrier preventing further merging. This can be seen better in the
// scenario a.MERGE.3, a.SET.2, a.MERGE.1. The entry a.MERGE.1 may be at lower
// (older) level and not involved in the compaction. If the compaction of
// a.MERGE.3 and a.SET.2 produced a.MERGE.3, a subsequent compaction with
// a.MERGE.1 would merge the values together incorrectly.
//
// 3. Snapshots
//
// Snapshots are lightweight point-in-time views of the DB state. At its core,
// a snapshot is a sequence number along with a guarantee from Pebble that it
// will maintain the view of the database at that sequence number. Part of this
// guarantee is relatively straightforward to achieve. When reading from the
// database Pebble will ignore sequence numbers that are larger than the
// snapshot sequence number. The primary complexity with snapshots occurs
// during compaction: the collapsing of entries that are shadowed by newer
// entries is at odds with the guarantee that Pebble will maintain the view of
// the database at the snapshot sequence number. Rather than collapsing entries
// up to the next user key, compactionIter can only collapse entries up to the
// next snapshot boundary. That is, every snapshot boundary potentially causes
// another entry for the same user-key to be emitted. Another way to view this
// is that snapshots define stripes and entries are collapsed within stripes,
// but not across stripes. Consider the following scenario:
//
//	a.PUT.9
//	a.DEL.8
//	a.PUT.7
//	a.DEL.6
//	a.PUT.5
//
// In the absence of snapshots these entries would be collapsed to
// a.PUT.9. What if there is a snapshot at sequence number 7? The entries can
// be divided into two stripes and collapsed within the stripes:
//
//	a.PUT.9        a.PUT.9
//	a.DEL.8  --->
//	a.PUT.7
//	--             --
//	a.DEL.6  --->  a.DEL.6
//	a.PUT.5
//
// All of the rules described earlier still apply, but they are confined to
// operate within a snapshot stripe. Snapshots only affect compaction when the
// snapshot sequence number lies within the range of sequence numbers being
// compacted. In the above example, a snapshot at sequence number 10 or at
// sequence number 5 would not have any effect.
//
// 4. Range Deletions
//
// Range deletions provide the ability to delete all of the keys (and values)
// in a contiguous range. Range deletions are stored indexed by their start
// key. The end key of the range is stored in the value. In order to support
// lookup of the range deletions which overlap with a particular key, the range
// deletion tombstones need to be fragmented whenever they overlap. This
// fragmentation is performed by keyspan.Fragmenter. The fragments are then
// subject to the rules for snapshots. For example, consider the two range
// tombstones [a,e)#1 and [c,g)#2:
//
//	2:     c-------g
//	1: a-------e
//
// These tombstones will be fragmented into:
//
//	2:     c---e---g
//	1: a---c---e
//
// Do we output the fragment [c,e)#1? Since it is covered by [c-e]#2 the answer
// depends on whether it is in a new snapshot stripe.
//
// In addition to the fragmentation of range tombstones, compaction also needs
// to take the range tombstones into consideration when outputting normal
// keys. Just as with point deletions, a range deletion covering an entry can
// cause the entry to be elided.
//
// A note on the stability of keys and values.
//
// The stability guarantees of keys and values returned by the iterator tree
// that backs a compactionIter is nuanced and care must be taken when
// referencing any returned items.
//
// Keys and values returned by exported functions (i.e. First, Next, etc.) have
// lifetimes that fall into two categories:
//
// Lifetime valid for duration of compaction. Range deletion keys and values are
// stable for the duration of the compaction, due to way in which a
// compactionIter is typically constructed (i.e. via (*compaction).newInputIter,
// which wraps the iterator over the range deletion block in a noCloseIter,
// preventing the release of the backing memory until the compaction is
// finished).
//
// Lifetime limited to duration of sstable block liveness. Point keys (SET, DEL,
// etc.) and values must be cloned / copied following the return from the
// exported function, and before a subsequent call to Next advances the iterator
// and mutates the contents of the returned key and value.
type compactionIter struct {
	equal Equal
	merge Merge
	iter  internalIterator
	err   error
	// `key.UserKey` is set to `keyBuf` caused by saving `i.iterKey.UserKey`
	// and `key.Trailer` is set to `i.iterKey.Trailer`. This is the
	// case on return from all public methods -- these methods return `key`.
	// Additionally, it is the internal state when the code is moving to the
	// next key so it can determine whether the user key has changed from
	// the previous key.
	key InternalKey
	// keyTrailer is updated when `i.key` is updated and holds the key's
	// original trailer (eg, before any sequence-number zeroing or changes to
	// key kind).
	keyTrailer  uint64
	value       []byte
	valueCloser io.Closer
	// Temporary buffer used for storing the previous user key in order to
	// determine when iteration has advanced to a new user key and thus a new
	// snapshot stripe.
	keyBuf []byte
	// Temporary buffer used for storing the previous value, which may be an
	// unsafe, i.iter-owned slice that could be altered when the iterator is
	// advanced.
	valueBuf []byte
	// Is the current entry valid?
	valid            bool
	iterKey          *InternalKey
	iterValue        []byte
	iterStripeChange stripeChangeType
	// `skip` indicates whether the remaining skippable entries in the current
	// snapshot stripe should be skipped or processed. An example of a non-
	// skippable entry is a range tombstone as we need to return it from the
	// `compactionIter`, even if a key covering its start key has already been
	// seen in the same stripe. `skip` has no effect when `pos == iterPosNext`.
	//
	// TODO(jackson): If we use keyspan.InterleavingIter for range deletions,
	// like we do for range keys, the only remaining 'non-skippable' key is
	// the invalid key. We should be able to simplify this logic and remove this
	// field.
	skip bool
	// `pos` indicates the iterator position at the top of `Next()`. Its type's
	// (`iterPos`) values take on the following meanings in the context of
	// `compactionIter`.
	//
	// - `iterPosCur`: the iterator is at the last key returned.
	// - `iterPosNext`: the iterator has already been advanced to the next
	//   candidate key. For example, this happens when processing merge operands,
	//   where we advance the iterator all the way into the next stripe or next
	//   user key to ensure we've seen all mergeable operands.
	// - `iterPosPrev`: this is invalid as compactionIter is forward-only.
	pos iterPos
	// `snapshotPinned` indicates whether the last point key returned by the
	// compaction iterator was only returned because an open snapshot prevents
	// its elision. This field only applies to point keys, and not to range
	// deletions or range keys.
	//
	// For MERGE, it is possible that doing the merge is interrupted even when
	// the next point key is in the same stripe. This can happen if the loop in
	// mergeNext gets interrupted by sameStripeNonSkippable.
	// sameStripeNonSkippable occurs due to RANGEDELs that sort before
	// SET/MERGE/DEL with the same seqnum, so the RANGEDEL does not necessarily
	// delete the subsequent SET/MERGE/DEL keys.
	snapshotPinned bool
	// forceObsoleteDueToRangeDel is set to true in a subset of the cases that
	// snapshotPinned is true. This value is true when the point is obsolete due
	// to a RANGEDEL but could not be deleted due to a snapshot.
	//
	// NB: it may seem that the additional cases that snapshotPinned captures
	// are harmless in that they can also be used to mark a point as obsolete
	// (it is merely a duplication of some logic that happens in
	// Writer.AddWithForceObsolete), but that is not quite accurate as of this
	// writing -- snapshotPinned originated in stats collection and for a
	// sequence MERGE, SET, where the MERGE cannot merge with the (older) SET
	// due to a snapshot, the snapshotPinned value for the SET is true.
	//
	// TODO(sumeer,jackson): improve the logic of snapshotPinned and reconsider
	// whether we need forceObsoleteDueToRangeDel.
	forceObsoleteDueToRangeDel bool
	// The index of the snapshot for the current key within the snapshots slice.
	curSnapshotIdx    int
	curSnapshotSeqNum uint64
	// The snapshot sequence numbers that need to be maintained. These sequence
	// numbers define the snapshot stripes (see the Snapshots description
	// above). The sequence numbers are in ascending order.
	snapshots []uint64
	// frontiers holds a heap of user keys that affect compaction behavior when
	// they're exceeded. Before a new key is returned, the compaction iterator
	// advances the frontier, notifying any code that subscribed to be notified
	// when a key was reached. The primary use today is within the
	// implementation of compactionOutputSplitters in compaction.go. Many of
	// these splitters wait for the compaction iterator to call Advance(k) when
	// it's returning a new key. If the key that they're waiting for is
	// surpassed, these splitters update internal state recording that they
	// should request a compaction split next time they're asked in
	// [shouldSplitBefore].
	frontiers frontiers
	// Reference to the range deletion tombstone fragmenter (e.g.,
	// `compaction.rangeDelFrag`).
	rangeDelFrag *keyspan.Fragmenter
	rangeKeyFrag *keyspan.Fragmenter
	// The fragmented tombstones.
	tombstones []keyspan.Span
	// The fragmented range keys.
	rangeKeys []keyspan.Span
	// Byte allocator for the tombstone keys.
	alloc                                  bytealloc.A
	allowZeroSeqNum                        bool
	elideTombstone                         func(key []byte) bool
	elideRangeTombstone                    func(start, end []byte) bool
	ineffectualSingleDeleteCallback        func(userKey []byte)
	singleDeleteInvariantViolationCallback func(userKey []byte)
	// The on-disk format major version. This informs the types of keys that
	// may be written to disk during a compaction.
	formatVersion FormatMajorVersion
	stats         struct {
		// count of DELSIZED keys that were missized.
		countMissizedDels uint64
	}
}

func newCompactionIter(
	cmp Compare,
	equal Equal,
	formatKey base.FormatKey,
	merge Merge,
	iter internalIterator,
	snapshots []uint64,
	rangeDelFrag *keyspan.Fragmenter,
	rangeKeyFrag *keyspan.Fragmenter,
	allowZeroSeqNum bool,
	elideTombstone func(key []byte) bool,
	elideRangeTombstone func(start, end []byte) bool,
	ineffectualSingleDeleteCallback func(userKey []byte),
	singleDeleteInvariantViolationCallback func(userKey []byte),
	formatVersion FormatMajorVersion,
) *compactionIter {
	i := &compactionIter{
		equal:                                  equal,
		merge:                                  merge,
		iter:                                   iter,
		snapshots:                              snapshots,
		frontiers:                              frontiers{cmp: cmp},
		rangeDelFrag:                           rangeDelFrag,
		rangeKeyFrag:                           rangeKeyFrag,
		allowZeroSeqNum:                        allowZeroSeqNum,
		elideTombstone:                         elideTombstone,
		elideRangeTombstone:                    elideRangeTombstone,
		ineffectualSingleDeleteCallback:        ineffectualSingleDeleteCallback,
		singleDeleteInvariantViolationCallback: singleDeleteInvariantViolationCallback,
		formatVersion:                          formatVersion,
	}
	i.rangeDelFrag.Cmp = cmp
	i.rangeDelFrag.Format = formatKey
	i.rangeDelFrag.Emit = i.emitRangeDelChunk
	i.rangeKeyFrag.Cmp = cmp
	i.rangeKeyFrag.Format = formatKey
	i.rangeKeyFrag.Emit = i.emitRangeKeyChunk
	return i
}

func (i *compactionIter) First() (*InternalKey, []byte) {
	if i.err != nil {
		return nil, nil
	}
	var iterValue LazyValue
	i.iterKey, iterValue = i.iter.First()
	i.iterValue, _, i.err = iterValue.Value(nil)
	if i.err != nil {
		return nil, nil
	}
	if i.iterKey != nil {
		i.curSnapshotIdx, i.curSnapshotSeqNum = snapshotIndex(i.iterKey.SeqNum(), i.snapshots)
	}
	i.pos = iterPosNext
	i.iterStripeChange = newStripeNewKey
	return i.Next()
}

func (i *compactionIter) Next() (*InternalKey, []byte) {
	if i.err != nil {
		return nil, nil
	}

	// Close the closer for the current value if one was open.
	if i.closeValueCloser() != nil {
		return nil, nil
	}

	// Prior to this call to `Next()` we are in one of four situations with
	// respect to `iterKey` and related state:
	//
	// - `!skip && pos == iterPosNext`: `iterKey` is already at the next key.
	// - `!skip && pos == iterPosCurForward`: We are at the key that has been returned.
	//   To move forward we advance by one key, even if that lands us in the same
	//   snapshot stripe.
	// - `skip && pos == iterPosCurForward`: We are at the key that has been returned.
	//   To move forward we skip skippable entries in the stripe.
	// - `skip && pos == iterPosNext && i.iterStripeChange == sameStripeNonSkippable`:
	//    This case may occur when skipping within a snapshot stripe and we
	//    encounter either:
	//      a) an invalid key kind; The previous call will have returned
	//         whatever key it was processing and deferred handling of the
	//         invalid key to this invocation of Next(). We're responsible for
	//         ignoring skip=true and falling into the invalid key kind case
	//         down below.
	//      b) an interleaved range delete; This is a wart of the current code
	//         structure. While skipping within a snapshot stripe, a range
	//         delete interleaved at its start key and sequence number
	//         interrupts the sequence of point keys. After we return the range
	//         delete to the caller, we need to pick up skipping at where we
	//         left off, so we preserve skip=true.
	//    TODO(jackson): This last case is confusing and can be removed if we
	//    interleave range deletions at the maximal sequence number using the
	//    keyspan interleaving iterator. This is the treatment given to range
	//    keys today.
	if i.pos == iterPosCurForward {
		if i.skip {
			i.skipInStripe()
		} else {
			i.nextInStripe()
		}
	} else if i.skip {
		if i.iterStripeChange != sameStripeNonSkippable {
			panic(errors.AssertionFailedf("compaction iterator has skip=true, but iterator is at iterPosNext"))
		}
	}

	i.pos = iterPosCurForward
	i.valid = false

	for i.iterKey != nil {
		// If we entered a new snapshot stripe with the same key, any key we
		// return on this iteration is only returned because the open snapshot
		// prevented it from being elided or merged with the key returned for
		// the previous stripe. Mark it as pinned so that the compaction loop
		// can correctly populate output tables' pinned statistics. We might
		// also set snapshotPinned=true down below if we observe that the key is
		// deleted by a range deletion in a higher stripe or that this key is a
		// tombstone that could be elided if only it were in the last snapshot
		// stripe.
		i.snapshotPinned = i.iterStripeChange == newStripeSameKey

		if i.iterKey.Kind() == InternalKeyKindRangeDelete || rangekey.IsRangeKey(i.iterKey.Kind()) {
			// Return the span so the compaction can use it for file truncation and add
			// it to the relevant fragmenter. We do not set `skip` to true before
			// returning as there may be a forthcoming point key with the same user key
			// and sequence number. Such a point key must be visible (i.e., not skipped
			// over) since we promise point keys are not deleted by range tombstones at
			// the same sequence number.
			//
			// Although, note that `skip` may already be true before reaching here
			// due to an earlier key in the stripe. Then it is fine to leave it set
			// to true, as the earlier key must have had a higher sequence number.
			//
			// NOTE: there is a subtle invariant violation here in that calling
			// saveKey and returning a reference to the temporary slice violates
			// the stability guarantee for range deletion keys. A potential
			// mediation could return the original iterKey and iterValue
			// directly, as the backing memory is guaranteed to be stable until
			// the compaction completes. The violation here is only minor in
			// that the caller immediately clones the range deletion InternalKey
			// when passing the key to the deletion fragmenter (see the
			// call-site in compaction.go).
			// TODO(travers): address this violation by removing the call to
			// saveKey and instead return the original iterKey and iterValue.
			// This goes against the comment on i.key in the struct, and
			// therefore warrants some investigation.
			i.saveKey()
			// TODO(jackson): Handle tracking pinned statistics for range keys
			// and range deletions. This would require updating
			// emitRangeDelChunk and rangeKeyCompactionTransform to update
			// statistics when they apply their own snapshot striping logic.
			i.snapshotPinned = false
			i.value = i.iterValue
			i.valid = true
			return &i.key, i.value
		}

		// TODO(sumeer): we could avoid calling Covers if i.iterStripeChange ==
		// sameStripeSameKey since that check has already been done in
		// nextInStripeHelper. However, we also need to handle the case of
		// CoversInvisibly below.
		if cover := i.rangeDelFrag.Covers(*i.iterKey, i.curSnapshotSeqNum); cover == keyspan.CoversVisibly {
			// A pending range deletion deletes this key. Skip it.
			i.saveKey()
			i.skipInStripe()
			continue
		} else if cover == keyspan.CoversInvisibly {
			// i.iterKey would be deleted by a range deletion if there weren't
			// any open snapshots. Mark it as pinned.
			//
			// NB: there are multiple places in this file where we call
			// i.rangeDelFrag.Covers and this is the only one where we are writing
			// to i.snapshotPinned. Those other cases occur in mergeNext where the
			// caller is deciding whether the value should be merged or not, and the
			// key is in the same snapshot stripe. Hence, snapshotPinned is by
			// definition false in those cases.
			i.snapshotPinned = true
			i.forceObsoleteDueToRangeDel = true
		} else {
			i.forceObsoleteDueToRangeDel = false
		}

		switch i.iterKey.Kind() {
		case InternalKeyKindDelete, InternalKeyKindSingleDelete, InternalKeyKindDeleteSized:
			if i.elideTombstone(i.iterKey.UserKey) {
				if i.curSnapshotIdx == 0 {
					// If we're at the last snapshot stripe and the tombstone
					// can be elided skip skippable keys in the same stripe.
					i.saveKey()
					if i.key.Kind() == InternalKeyKindSingleDelete {
						i.skipDueToSingleDeleteElision()
					} else {
						i.skipInStripe()
						if !i.skip && i.iterStripeChange != newStripeNewKey {
							panic(errors.AssertionFailedf("pebble: skipInStripe in last stripe disabled skip without advancing to new key"))
						}
					}
					if i.iterStripeChange == newStripeSameKey {
						panic(errors.AssertionFailedf("pebble: skipInStripe in last stripe found a new stripe within the same key"))
					}
					continue
				} else {
					// We're not at the last snapshot stripe, so the tombstone
					// can NOT yet be elided. Mark it as pinned, so that it's
					// included in table statistics appropriately.
					i.snapshotPinned = true
				}
			}

			switch i.iterKey.Kind() {
			case InternalKeyKindDelete:
				i.saveKey()
				i.value = i.iterValue
				i.valid = true
				i.skip = true
				return &i.key, i.value

			case InternalKeyKindDeleteSized:
				// We may skip subsequent keys because of this tombstone. Scan
				// ahead to see just how much data this tombstone drops and if
				// the tombstone's value should be updated accordingly.
				return i.deleteSizedNext()

			case InternalKeyKindSingleDelete:
				if i.singleDeleteNext() {
					return &i.key, i.value
				} else if i.err != nil {
					return nil, nil
				}
				continue

			default:
				panic(errors.AssertionFailedf(
					"unexpected kind %s", redact.SafeString(i.iterKey.Kind().String())))
			}

		case InternalKeyKindSet, InternalKeyKindSetWithDelete:
			// The key we emit for this entry is a function of the current key
			// kind, and whether this entry is followed by a DEL/SINGLEDEL
			// entry. setNext() does the work to move the iterator forward,
			// preserving the original value, and potentially mutating the key
			// kind.
			i.setNext()
			if i.err != nil {
				return nil, nil
			}
			return &i.key, i.value

		case InternalKeyKindMerge:
			// Record the snapshot index before mergeNext as merging
			// advances the iterator, adjusting curSnapshotIdx.
			origSnapshotIdx := i.curSnapshotIdx
			var valueMerger ValueMerger
			valueMerger, i.err = i.merge(i.iterKey.UserKey, i.iterValue)
			var change stripeChangeType
			if i.err == nil {
				change = i.mergeNext(valueMerger)
			}
			var needDelete bool
			if i.err == nil {
				// includesBase is true whenever we've transformed the MERGE record
				// into a SET.
				var includesBase bool
				switch i.key.Kind() {
				case InternalKeyKindSet, InternalKeyKindSetWithDelete:
					includesBase = true
				case InternalKeyKindMerge:
				default:
					panic(errors.AssertionFailedf(
						"unexpected kind %s", redact.SafeString(i.key.Kind().String())))
				}
				i.value, needDelete, i.valueCloser, i.err = finishValueMerger(valueMerger, includesBase)
			}
			if i.err == nil {
				if needDelete {
					i.valid = false
					if i.closeValueCloser() != nil {
						return nil, nil
					}
					continue
				}
				// A non-skippable entry does not necessarily cover later merge
				// operands, so we must not zero the current merge result's seqnum.
				//
				// For example, suppose the forthcoming two keys are a range
				// tombstone, `[a, b)#3`, and a merge operand, `a#3`. Recall that
				// range tombstones do not cover point keys at the same seqnum, so
				// `a#3` is not deleted. The range tombstone will be seen first due
				// to its larger value type. Since it is a non-skippable key, the
				// current merge will not include `a#3`. If we zeroed the current
				// merge result's seqnum, then it would conflict with the upcoming
				// merge including `a#3`, whose seqnum will also be zeroed.
				if change != sameStripeNonSkippable {
					i.maybeZeroSeqnum(origSnapshotIdx)
				}
				return &i.key, i.value
			}
			if i.err != nil {
				i.valid = false
				// TODO(sumeer): why is MarkCorruptionError only being called for
				// MERGE?
				i.err = base.MarkCorruptionError(i.err)
			}
			return nil, nil

		default:
			i.err = base.CorruptionErrorf("invalid internal key kind: %d", errors.Safe(i.iterKey.Kind()))
			i.valid = false
			return nil, nil
		}
	}

	return nil, nil
}

func (i *compactionIter) closeValueCloser() error {
	if i.valueCloser == nil {
		return nil
	}

	i.err = i.valueCloser.Close()
	i.valueCloser = nil
	if i.err != nil {
		i.valid = false
	}
	return i.err
}

// snapshotIndex returns the index of the first sequence number in snapshots
// which is greater than or equal to seq.
func snapshotIndex(seq uint64, snapshots []uint64) (int, uint64) {
	index := sort.Search(len(snapshots), func(i int) bool {
		return snapshots[i] > seq
	})
	if index >= len(snapshots) {
		return index, InternalKeySeqNumMax
	}
	return index, snapshots[index]
}

// skipInStripe skips over skippable keys in the same stripe and user key. It
// may set i.err, in which case i.iterKey will be nil.
func (i *compactionIter) skipInStripe() {
	i.skip = true
	// TODO(sumeer): we can avoid the overhead of calling i.rangeDelFrag.Covers,
	// in this case of nextInStripe, since we are skipping all of them anyway.
	for i.nextInStripe() == sameStripeSkippable {
		if i.err != nil {
			panic(i.err)
		}
	}
	// Reset skip if we landed outside the original stripe. Otherwise, we landed
	// in the same stripe on a non-skippable key. In that case we should preserve
	// `i.skip == true` such that later keys in the stripe will continue to be
	// skipped.
	if i.iterStripeChange == newStripeNewKey || i.iterStripeChange == newStripeSameKey {
		i.skip = false
	}
}

func (i *compactionIter) iterNext() bool {
	var iterValue LazyValue
	i.iterKey, iterValue = i.iter.Next()
	i.iterValue, _, i.err = iterValue.Value(nil)
	if i.err != nil {
		i.iterKey = nil
	}
	return i.iterKey != nil
}

// stripeChangeType indicates how the snapshot stripe changed relative to the
// previous key. If no change, it also indicates whether the current entry is
// skippable. If the snapshot stripe changed, it also indicates whether the new
// stripe was entered because the iterator progressed onto an entirely new key
// or entered a new stripe within the same key.
type stripeChangeType int

const (
	newStripeNewKey stripeChangeType = iota
	newStripeSameKey
	sameStripeSkippable
	sameStripeNonSkippable
)

// nextInStripe advances the iterator and returns one of the above const ints
// indicating how its state changed.
//
// All sameStripeSkippable keys that are covered by a RANGEDEL will be skipped
// and not returned.
//
// Calls to nextInStripe must be preceded by a call to saveKey to retain a
// temporary reference to the original key, so that forward iteration can
// proceed with a reference to the original key. Care should be taken to avoid
// overwriting or mutating the saved key or value before they have been returned
// to the caller of the exported function (i.e. the caller of Next, First, etc.)
//
// nextInStripe may set i.err, in which case the return value will be
// newStripeNewKey, and i.iterKey will be nil.
func (i *compactionIter) nextInStripe() stripeChangeType {
	i.iterStripeChange = i.nextInStripeHelper()
	return i.iterStripeChange
}

// nextInStripeHelper is an internal helper for nextInStripe; callers should use
// nextInStripe and not call nextInStripeHelper.
func (i *compactionIter) nextInStripeHelper() stripeChangeType {
	origSnapshotIdx := i.curSnapshotIdx
	for {
		if !i.iterNext() {
			return newStripeNewKey
		}
		key := i.iterKey

		if !i.equal(i.key.UserKey, key.UserKey) {
			i.curSnapshotIdx, i.curSnapshotSeqNum = snapshotIndex(key.SeqNum(), i.snapshots)
			return newStripeNewKey
		}

		// If i.key and key have the same user key, then
		//   1. i.key must not have had a zero sequence number (or it would've be the last
		//      key with its user key).
		//   2. i.key must have a strictly larger sequence number
		// There's an exception in that either key may be a range delete. Range
		// deletes may share a sequence number with a point key if the keys were
		// ingested together. Range keys may also share the sequence number if they
		// were ingested, but range keys are interleaved into the compaction
		// iterator's input iterator at the maximal sequence number so their
		// original sequence number will not be observed here.
		if prevSeqNum := base.SeqNumFromTrailer(i.keyTrailer); (prevSeqNum == 0 || prevSeqNum <= key.SeqNum()) &&
			i.key.Kind() != InternalKeyKindRangeDelete && key.Kind() != InternalKeyKindRangeDelete {
			prevKey := i.key
			prevKey.Trailer = i.keyTrailer
			panic(errors.AssertionFailedf("pebble: invariant violation: %s and %s out of order", prevKey, key))
		}

		i.curSnapshotIdx, i.curSnapshotSeqNum = snapshotIndex(key.SeqNum(), i.snapshots)
		switch key.Kind() {
		case InternalKeyKindRangeDelete:
			// Range tombstones need to be exposed by the compactionIter to the upper level
			// `compaction` object, so return them regardless of whether they are in the same
			// snapshot stripe.
			if i.curSnapshotIdx == origSnapshotIdx {
				return sameStripeNonSkippable
			}
			return newStripeSameKey
		case InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete:
			// Range keys are interleaved at the max sequence number for a given user
			// key, so we should not see any more range keys in this stripe.
			panic("unreachable")
		case InternalKeyKindInvalid:
			if i.curSnapshotIdx == origSnapshotIdx {
				return sameStripeNonSkippable
			}
			return newStripeSameKey
		case InternalKeyKindDelete, InternalKeyKindSet, InternalKeyKindMerge, InternalKeyKindSingleDelete,
			InternalKeyKindSetWithDelete, InternalKeyKindDeleteSized:
			// Fall through
		default:
			i.iterKey = nil
			i.err = base.CorruptionErrorf("invalid internal key kind: %d", errors.Safe(i.iterKey.Kind()))
			i.valid = false
			return newStripeNewKey
		}
		if i.curSnapshotIdx == origSnapshotIdx {
			// Same snapshot.
			if i.rangeDelFrag.Covers(*i.iterKey, i.curSnapshotSeqNum) == keyspan.CoversVisibly {
				continue
			}
			return sameStripeSkippable
		}
		return newStripeSameKey
	}
}

func (i *compactionIter) setNext() {
	// Save the current key.
	i.saveKey()
	i.value = i.iterValue
	i.valid = true
	i.maybeZeroSeqnum(i.curSnapshotIdx)

	// There are two cases where we can early return and skip the remaining
	// records in the stripe:
	// - If the DB does not SETWITHDEL.
	// - If this key is already a SETWITHDEL.
	if i.formatVersion < FormatSetWithDelete ||
		i.iterKey.Kind() == InternalKeyKindSetWithDelete {
		i.skip = true
		return
	}

	// We are iterating forward. Save the current value.
	i.valueBuf = append(i.valueBuf[:0], i.iterValue...)
	i.value = i.valueBuf

	// Else, we continue to loop through entries in the stripe looking for a
	// DEL. Note that we may stop *before* encountering a DEL, if one exists.
	//
	// NB: nextInStripe will skip sameStripeSkippable keys that are visibly
	// covered by a RANGEDEL. This can include DELs -- this is fine since such
	// DELs don't need to be combined with SET to make SETWITHDEL.
	for {
		switch i.nextInStripe() {
		case newStripeNewKey, newStripeSameKey:
			i.pos = iterPosNext
			return
		case sameStripeNonSkippable:
			i.pos = iterPosNext
			// We iterated onto a key that we cannot skip. We can
			// conservatively transform the original SET into a SETWITHDEL
			// as an indication that there *may* still be a DEL/SINGLEDEL
			// under this SET, even if we did not actually encounter one.
			//
			// This is safe to do, as:
			//
			// - in the case that there *is not* actually a DEL/SINGLEDEL
			// under this entry, any SINGLEDEL above this now-transformed
			// SETWITHDEL will become a DEL when the two encounter in a
			// compaction. The DEL will eventually be elided in a
			// subsequent compaction. The cost for ensuring correctness is
			// that this entry is kept around for an additional compaction
			// cycle(s).
			//
			// - in the case there *is* indeed a DEL/SINGLEDEL under us
			// (but in a different stripe or sstable), then we will have
			// already done the work to transform the SET into a
			// SETWITHDEL, and we will skip any additional iteration when
			// this entry is encountered again in a subsequent compaction.
			//
			// Ideally, this codepath would be smart enough to handle the
			// case of SET <- RANGEDEL <- ... <- DEL/SINGLEDEL <- ....
			// This requires preserving any RANGEDEL entries we encounter
			// along the way, then emitting the original (possibly
			// transformed) key, followed by the RANGEDELs. This requires
			// a sizable refactoring of the existing code, as nextInStripe
			// currently returns a sameStripeNonSkippable when it
			// encounters a RANGEDEL.
			// TODO(travers): optimize to handle the RANGEDEL case if it
			// turns out to be a performance problem.
			i.key.SetKind(InternalKeyKindSetWithDelete)

			// By setting i.skip=true, we are saying that after the
			// non-skippable key is emitted (which is likely a RANGEDEL),
			// the remaining point keys that share the same user key as this
			// saved key should be skipped.
			i.skip = true
			return
		case sameStripeSkippable:
			// We're still in the same stripe. If this is a
			// DEL/SINGLEDEL/DELSIZED, we stop looking and emit a SETWITHDEL.
			// Subsequent keys are eligible for skipping.
			switch i.iterKey.Kind() {
			case InternalKeyKindDelete, InternalKeyKindSingleDelete, InternalKeyKindDeleteSized:
				i.key.SetKind(InternalKeyKindSetWithDelete)
				i.skip = true
				return
			case InternalKeyKindSet, InternalKeyKindMerge, InternalKeyKindSetWithDelete:
				// Do nothing
			default:
				i.err = base.CorruptionErrorf("invalid internal key kind: %d", errors.Safe(i.iterKey.Kind()))
				i.valid = false
			}
		default:
			panic("pebble: unexpected stripeChangeType: " + strconv.Itoa(int(i.iterStripeChange)))
		}
	}
}

func (i *compactionIter) mergeNext(valueMerger ValueMerger) stripeChangeType {
	// Save the current key.
	i.saveKey()
	i.valid = true

	// Loop looking for older values in the current snapshot stripe and merge
	// them.
	for {
		if i.nextInStripe() != sameStripeSkippable {
			i.pos = iterPosNext
			return i.iterStripeChange
		}
		if i.err != nil {
			panic(i.err)
		}
		// NB: MERGE#10+RANGEDEL#9 stays a MERGE, since nextInStripe skips
		// sameStripeSkippable keys that are visibly covered by a RANGEDEL. There
		// may be MERGE#7 that is invisibly covered and will be preserved, but
		// there is no risk that MERGE#10 and MERGE#7 will get merged in the
		// future as the RANGEDEL still exists and will be used in user-facing
		// reads that see MERGE#10, and will also eventually cause MERGE#7 to be
		// deleted in a compaction.
		key := i.iterKey
		switch key.Kind() {
		case InternalKeyKindDelete, InternalKeyKindSingleDelete, InternalKeyKindDeleteSized:
			// We've hit a deletion tombstone. Return everything up to this point and
			// then skip entries until the next snapshot stripe. We change the kind
			// of the result key to a Set so that it shadows keys in lower
			// levels. That is, MERGE+DEL -> SETWITHDEL.
			//
			// We do the same for SingleDelete since SingleDelete is only
			// permitted (with deterministic behavior) for keys that have been
			// set once since the last SingleDelete/Delete, so everything
			// older is acceptable to shadow. Note that this is slightly
			// different from singleDeleteNext() which implements stricter
			// semantics in terms of applying the SingleDelete to the single
			// next Set. But those stricter semantics are not observable to
			// the end-user since Iterator interprets SingleDelete as Delete.
			// We could do something more complicated here and consume only a
			// single Set, and then merge in any following Sets, but that is
			// complicated wrt code and unnecessary given the narrow permitted
			// use of SingleDelete.
			i.key.SetKind(InternalKeyKindSetWithDelete)
			i.skip = true
			return sameStripeSkippable

		case InternalKeyKindSet, InternalKeyKindSetWithDelete:
			// We've hit a Set or SetWithDel value. Merge with the existing
			// value and return. We change the kind of the resulting key to a
			// Set so that it shadows keys in lower levels. That is:
			// MERGE + (SET*) -> SET.
			i.err = valueMerger.MergeOlder(i.iterValue)
			if i.err != nil {
				i.valid = false
				return sameStripeSkippable
			}
			i.key.SetKind(InternalKeyKindSet)
			i.skip = true
			return sameStripeSkippable

		case InternalKeyKindMerge:
			// We've hit another Merge value. Merge with the existing value and
			// continue looping.
			i.err = valueMerger.MergeOlder(i.iterValue)
			if i.err != nil {
				i.valid = false
				return sameStripeSkippable
			}

		default:
			i.err = base.CorruptionErrorf("invalid internal key kind: %d", errors.Safe(i.iterKey.Kind()))
			i.valid = false
			return sameStripeSkippable
		}
	}
}

// singleDeleteNext processes a SingleDelete point tombstone. A SingleDelete, or
// SINGLEDEL, is unique in that it deletes exactly 1 internal key. It's a
// performance optimization when the client knows a user key has not been
// overwritten, allowing the elision of the tombstone earlier, avoiding write
// amplification.
//
// singleDeleteNext returns a boolean indicating whether or not the caller
// should yield the SingleDelete key to the consumer of the compactionIter. If
// singleDeleteNext returns false, the caller may consume/elide the
// SingleDelete.
func (i *compactionIter) singleDeleteNext() bool {
	// Save the current key.
	i.saveKey()
	i.value = i.iterValue
	i.valid = true

	// Loop until finds a key to be passed to the next level.
	for {
		// If we find a key that can't be skipped, return true so that the
		// caller yields the SingleDelete to the caller.
		if i.nextInStripe() != sameStripeSkippable {
			// This defers additional error checking regarding single delete
			// invariants to the compaction where the keys with the same user key as
			// the single delete are in the same stripe.
			i.pos = iterPosNext
			return i.err == nil
		}
		if i.err != nil {
			panic(i.err)
		}
		// INVARIANT: sameStripeSkippable.
		key := i.iterKey
		kind := key.Kind()
		switch kind {
		case InternalKeyKindDelete, InternalKeyKindSetWithDelete, InternalKeyKindDeleteSized:
			if (kind == InternalKeyKindDelete || kind == InternalKeyKindDeleteSized) &&
				i.ineffectualSingleDeleteCallback != nil {
				i.ineffectualSingleDeleteCallback(i.key.UserKey)
			}
			// We've hit a Delete, DeleteSized, SetWithDelete, transform
			// the SingleDelete into a full Delete.
			i.key.SetKind(InternalKeyKindDelete)
			i.skip = true
			return true

		case InternalKeyKindSet, InternalKeyKindMerge:
			// This SingleDelete deletes the Set/Merge, and we can now elide the
			// SingleDel as well. We advance past the Set and return false to
			// indicate to the main compaction loop that we should NOT yield the
			// current SingleDel key to the compaction loop.
			//
			// NB: singleDeleteNext was called with i.pos == iterPosCurForward, and
			// after the call to nextInStripe, we are still at iterPosCurForward,
			// since we are at the key after the Set/Merge that was single deleted.
			change := i.nextInStripe()
			switch change {
			case sameStripeSkippable, newStripeSameKey:
				// On the same user key.
				nextKind := i.iterKey.Kind()
				switch nextKind {
				case InternalKeyKindSet, InternalKeyKindSetWithDelete, InternalKeyKindMerge:
					if i.singleDeleteInvariantViolationCallback != nil {
						// sameStripeSkippable keys returned by nextInStripe() are already
						// known to not be covered by a RANGEDEL, so it is an invariant
						// violation. The rare case is newStripeSameKey, where it is a
						// violation if not covered by a RANGEDEL.
						if change == sameStripeSkippable ||
							i.rangeDelFrag.Covers(*i.iterKey, i.curSnapshotSeqNum) == keyspan.NoCover {
							i.singleDeleteInvariantViolationCallback(i.key.UserKey)
						}
					}
				case InternalKeyKindDelete, InternalKeyKindDeleteSized, InternalKeyKindSingleDelete,
					InternalKeyKindRangeDelete:
				default:
					panic(errors.AssertionFailedf(
						"unexpected internal key kind: %d", errors.Safe(i.iterKey.Kind())))
				}
			case sameStripeNonSkippable:
				// No ability to check whether there is another Set/Merge below with
				// the same user key.
				//
				// TODO(sumeer): once range deletions are interleaved at the maximal
				// sequence number, this case will go away.
			case newStripeNewKey:
			default:
				panic("unreachable")
			}
			i.valid = false
			return false

		case InternalKeyKindSingleDelete:
			// Two single deletes met in a compaction. The first single delete is
			// ineffectual.
			if i.ineffectualSingleDeleteCallback != nil {
				i.ineffectualSingleDeleteCallback(i.key.UserKey)
			}
			// Continue to apply the second single delete.
			continue

		default:
			i.err = base.CorruptionErrorf("invalid internal key kind: %d", errors.Safe(i.iterKey.Kind()))
			i.valid = false
			return false
		}
	}
}

// skipDueToSingleDeleteElision is called when the SingleDelete is being
// elided because it is in the final snapshot stripe and there are no keys
// with the same user key in lower levels in the LSM (below the files in this
// compaction).
//
// TODO(sumeer): the only difference between singleDeleteNext and
// skipDueToSingleDeleteElision is the fact that the caller knows it will be
// eliding the single delete in the latter case. There are some similar things
// happening in both implementations. My first attempt at combining them into
// a single method was hard to comprehend. Try again.
func (i *compactionIter) skipDueToSingleDeleteElision() {
	for {
		stripeChange := i.nextInStripe()
		if i.err != nil {
			panic(i.err)
		}
		switch stripeChange {
		case newStripeNewKey:
			// The single delete is only now being elided, meaning it did not elide
			// any keys earlier in its descent down the LSM. We stepped onto a new
			// user key, meaning that even now at its moment of elision, it still
			// hasn't elided any other keys. The single delete was ineffectual (a
			// no-op).
			if i.ineffectualSingleDeleteCallback != nil {
				i.ineffectualSingleDeleteCallback(i.key.UserKey)
			}
			i.skip = false
			return
		case newStripeSameKey:
			// This should be impossible. If we're eliding a single delete, we
			// determined that the tombstone is in the final snapshot stripe, but we
			// stepped into a new stripe of the same key.
			panic(errors.AssertionFailedf("eliding single delete followed by same key in new stripe"))
		case sameStripeNonSkippable:
			// There's a key that we cannot skip. There are two possible cases:
			//   a. The key is invalid. This is an error.
			//   b. The key is a range deletion.
			// The second case may also be an ineffectual single delete. However, it
			// is possible that there is a SET that is at the same seqnum as the
			// RANGEDEL, and so is not deleted by that RANGEDEL, and will be deleted
			// by this single delete. So we cannot be certain that this is an
			// ineffectual single delete.
			//
			// TODO(sumeer): the existing todo to interleave range deletions at the
			// maximal sequence number will allow us to address this ambiguity.
			//
			// TODO(sumeer): by setting skip to true, the compactionIter is making a
			// single delete stronger (like a del), which will hide bugs in the use of
			// single delete.
			i.skip = true
			return
		case sameStripeSkippable:
			kind := i.iterKey.Kind()
			switch kind {
			case InternalKeyKindDelete, InternalKeyKindDeleteSized, InternalKeyKindSingleDelete:
				if i.ineffectualSingleDeleteCallback != nil {
					i.ineffectualSingleDeleteCallback(i.key.UserKey)
				}
				switch kind {
				case InternalKeyKindDelete, InternalKeyKindDeleteSized:
					i.skipInStripe()
					return
				case InternalKeyKindSingleDelete:
					// Repeat the same with this SingleDelete. We don't want to simply
					// call skipInStripe(), since it increases the strength of the
					// SingleDel, which hides bugs in the use of single delete.
					continue
				default:
					panic(errors.AssertionFailedf(
						"unexpected internal key kind: %d", errors.Safe(i.iterKey.Kind())))
				}
			case InternalKeyKindSetWithDelete:
				// The SingleDelete should behave like a Delete.
				i.skipInStripe()
				return
			case InternalKeyKindSet, InternalKeyKindMerge:
				// This SingleDelete deletes the Set/Merge, and we are eliding the
				// SingleDel as well. Step to the next key (this is not deleted by the
				// SingleDelete).
				//
				// NB: skipDueToSingleDeleteElision was called with i.pos ==
				// iterPosCurForward, and after the call to nextInStripe, we are still
				// at iterPosCurForward, since we are at the key after the Set/Merge
				// that was single deleted.
				change := i.nextInStripe()
				if i.err != nil {
					panic(i.err)
				}
				switch change {
				case newStripeSameKey:
					panic(errors.AssertionFailedf("eliding single delete followed by same key in new stripe"))
				case newStripeNewKey:
				case sameStripeSkippable:
					// On the same key.
					nextKind := i.iterKey.Kind()
					switch nextKind {
					case InternalKeyKindSet, InternalKeyKindSetWithDelete, InternalKeyKindMerge:
						if i.singleDeleteInvariantViolationCallback != nil {
							i.singleDeleteInvariantViolationCallback(i.key.UserKey)
						}
					case InternalKeyKindDelete, InternalKeyKindDeleteSized, InternalKeyKindSingleDelete,
						InternalKeyKindRangeDelete:
					default:
						panic(errors.AssertionFailedf(
							"unexpected internal key kind: %d", errors.Safe(i.iterKey.Kind())))
					}
				case sameStripeNonSkippable:
					// No ability to check whether there is another Set/Merge below with
					// the same user key.
					//
					// TODO(sumeer): once range deletions are interleaved at the maximal
					// sequence number, this case will go away.
				default:
					panic("unreachable")
				}
				// Whether in same stripe or new stripe, this key is not consumed by
				// the SingleDelete.
				i.skip = false
				return
			default:
				panic(errors.AssertionFailedf(
					"unexpected internal key kind: %d", errors.Safe(i.iterKey.Kind())))
			}
		default:
			panic("unreachable")
		}
	}
}

// deleteSizedNext processes a DELSIZED point tombstone. Unlike ordinary DELs,
// these tombstones carry a value that's a varint indicating the size of the
// entry (len(key)+len(value)) that the tombstone is expected to delete.
//
// When a deleteSizedNext is encountered, we skip ahead to see which keys, if
// any, are elided as a result of the tombstone.
func (i *compactionIter) deleteSizedNext() (*base.InternalKey, []byte) {
	i.saveKey()
	i.valid = true
	i.skip = true

	// The DELSIZED tombstone may have no value at all. This happens when the
	// tombstone has already deleted the key that the user originally predicted.
	// In this case, we still peek forward in case there's another DELSIZED key
	// with a lower sequence number, in which case we'll adopt its value.
	if len(i.iterValue) == 0 {
		i.value = i.valueBuf[:0]
	} else {
		i.valueBuf = append(i.valueBuf[:0], i.iterValue...)
		i.value = i.valueBuf
	}

	// Loop through all the keys within this stripe that are skippable.
	i.pos = iterPosNext
	for i.nextInStripe() == sameStripeSkippable {
		if i.err != nil {
			panic(i.err)
		}
		switch i.iterKey.Kind() {
		case InternalKeyKindDelete, InternalKeyKindDeleteSized, InternalKeyKindSingleDelete:
			// We encountered a tombstone (DEL, or DELSIZED) that's deleted by
			// the original DELSIZED tombstone. This can happen in two cases:
			//
			// (1) These tombstones were intended to delete two distinct values,
			//     and this DELSIZED has already dropped the relevant key. For
			//     example:
			//
			//     a.DELSIZED.9   a.SET.7   a.DELSIZED.5   a.SET.4
			//
			//     If a.DELSIZED.9 has already deleted a.SET.7, its size has
			//     already been zeroed out. In this case, we want to adopt the
			//     value of the DELSIZED with the lower sequence number, in
			//     case the a.SET.4 key has not yet been elided.
			//
			// (2) This DELSIZED was missized. The user thought they were
			//     deleting a key with this user key, but this user key had
			//     already been deleted.
			//
			// We can differentiate these two cases by examining the length of
			// the DELSIZED's value. A DELSIZED's value holds the size of both
			// the user key and value that it intends to delete. For any user
			// key with a length > 0, a DELSIZED that has not deleted a key must
			// have a value with a length > 0.
			//
			// We treat both cases the same functionally, adopting the identity
			// of the lower-sequence numbered tombstone. However in the second
			// case, we also increment the stat counting missized tombstones.
			if len(i.value) > 0 {
				// The original DELSIZED key was missized. The key that the user
				// thought they were deleting does not exist.
				i.stats.countMissizedDels++
			}
			i.valueBuf = append(i.valueBuf[:0], i.iterValue...)
			i.value = i.valueBuf
			if i.iterKey.Kind() != InternalKeyKindDeleteSized {
				// Convert the DELSIZED to a DEL—The DEL/SINGLEDEL we're eliding
				// may not have deleted the key(s) it was intended to yet. The
				// ordinary DEL compaction heuristics are better suited at that,
				// plus we don't want to count it as a missized DEL. We early
				// exit in this case, after skipping the remainder of the
				// snapshot stripe.
				i.key.SetKind(InternalKeyKindDelete)
				// NB: We skipInStripe now, rather than returning leaving
				// i.skip=true and returning early, because Next() requires
				// that i.skip=true only if i.iterPos = iterPosCurForward.
				//
				// Ignore any error caused by skipInStripe since it does not affect
				// the key/value being returned here, and the next call to Next() will
				// expose it.
				i.skipInStripe()
				return &i.key, i.value
			}
			// Continue, in case we uncover another DELSIZED or a key this
			// DELSIZED deletes.

		case InternalKeyKindSet, InternalKeyKindMerge, InternalKeyKindSetWithDelete:
			// If the DELSIZED is value-less, it already deleted the key that it
			// was intended to delete. This is possible with a sequence like:
			//
			//      DELSIZED.8     SET.7     SET.3
			//
			// The DELSIZED only describes the size of the SET.7, which in this
			// case has already been elided. We don't count it as a missizing,
			// instead converting the DELSIZED to a DEL. Skip the remainder of
			// the snapshot stripe and return.
			if len(i.value) == 0 {
				i.key.SetKind(InternalKeyKindDelete)
				// NB: We skipInStripe now, rather than returning leaving
				// i.skip=true and returning early, because Next() requires
				// that i.skip=true only if i.iterPos = iterPosCurForward.
				//
				// Ignore any error caused by skipInStripe since it does not affect
				// the key/value being returned here, and the next call to Next() will
				// expose it.
				i.skipInStripe()
				return &i.key, i.value
			}
			// The deleted key is not a DEL, DELSIZED, and the DELSIZED in i.key
			// has a positive size.
			expectedSize, n := binary.Uvarint(i.value)
			if n != len(i.value) {
				i.err = base.CorruptionErrorf("DELSIZED holds invalid value: %x", errors.Safe(i.value))
				i.valid = false
				return nil, nil
			}
			elidedSize := uint64(len(i.iterKey.UserKey)) + uint64(len(i.iterValue))
			if elidedSize != expectedSize {
				// The original DELSIZED key was missized. It's unclear what to
				// do. The user-provided size was wrong, so it's unlikely to be
				// accurate or meaningful. We could:
				//
				//   1. return the DELSIZED with the original user-provided size unmodified
				//   2. return the DELZIZED with a zeroed size to reflect that a key was
				//   elided, even if it wasn't the anticipated size.
				//   3. subtract the elided size from the estimate and re-encode.
				//   4. convert the DELSIZED into a value-less DEL, so that
				//      ordinary DEL heuristics apply.
				//
				// We opt for (4) under the rationale that we can't rely on the
				// user-provided size for accuracy, so ordinary DEL heuristics
				// are safer.
				i.stats.countMissizedDels++
				i.key.SetKind(InternalKeyKindDelete)
				i.value = i.valueBuf[:0]
				// NB: We skipInStripe now, rather than returning leaving
				// i.skip=true and returning early, because Next() requires
				// that i.skip=true only if i.iterPos = iterPosCurForward.
				//
				// Ignore any error caused by skipInStripe since it does not affect
				// the key/value being returned here, and the next call to Next() will
				// expose it.
				i.skipInStripe()
				return &i.key, i.value
			}
			// NB: We remove the value regardless of whether the key was sized
			// appropriately. The size encoded is 'consumed' the first time it
			// meets a key that it deletes.
			i.value = i.valueBuf[:0]

		default:
			i.err = base.CorruptionErrorf("invalid internal key kind: %d", errors.Safe(i.iterKey.Kind()))
			i.valid = false
			return nil, nil
		}
	}
	// Reset skip if we landed outside the original stripe. Otherwise, we landed
	// in the same stripe on a non-skippable key. In that case we should preserve
	// `i.skip == true` such that later keys in the stripe will continue to be
	// skipped.
	if i.iterStripeChange == newStripeNewKey || i.iterStripeChange == newStripeSameKey {
		i.skip = false
	}
	if i.err != nil {
		return nil, nil
	}
	return &i.key, i.value
}

func (i *compactionIter) saveKey() {
	i.keyBuf = append(i.keyBuf[:0], i.iterKey.UserKey...)
	i.key.UserKey = i.keyBuf
	i.key.Trailer = i.iterKey.Trailer
	i.keyTrailer = i.iterKey.Trailer
	i.frontiers.Advance(i.key.UserKey)
}

func (i *compactionIter) cloneKey(key []byte) []byte {
	i.alloc, key = i.alloc.Copy(key)
	return key
}

func (i *compactionIter) Key() InternalKey {
	return i.key
}

func (i *compactionIter) Value() []byte {
	return i.value
}

func (i *compactionIter) Valid() bool {
	return i.valid
}

func (i *compactionIter) Error() error {
	return i.err
}

func (i *compactionIter) Close() error {
	err := i.iter.Close()
	if i.err == nil {
		i.err = err
	}

	// Close the closer for the current value if one was open.
	if i.valueCloser != nil {
		i.err = firstError(i.err, i.valueCloser.Close())
		i.valueCloser = nil
	}

	return i.err
}

// Tombstones returns a list of pending range tombstones in the fragmenter
// up to the specified key, or all pending range tombstones if key = nil.
func (i *compactionIter) Tombstones(key []byte) []keyspan.Span {
	if key == nil {
		i.rangeDelFrag.Finish()
	} else {
		// The specified end key is exclusive; no versions of the specified
		// user key (including range tombstones covering that key) should
		// be flushed yet.
		i.rangeDelFrag.TruncateAndFlushTo(key)
	}
	tombstones := i.tombstones
	i.tombstones = nil
	return tombstones
}

// RangeKeys returns a list of pending fragmented range keys up to the specified
// key, or all pending range keys if key = nil.
func (i *compactionIter) RangeKeys(key []byte) []keyspan.Span {
	if key == nil {
		i.rangeKeyFrag.Finish()
	} else {
		// The specified end key is exclusive; no versions of the specified
		// user key (including range tombstones covering that key) should
		// be flushed yet.
		i.rangeKeyFrag.TruncateAndFlushTo(key)
	}
	rangeKeys := i.rangeKeys
	i.rangeKeys = nil
	return rangeKeys
}

func (i *compactionIter) emitRangeDelChunk(fragmented keyspan.Span) {
	// Apply the snapshot stripe rules, keeping only the latest tombstone for
	// each snapshot stripe.
	currentIdx := -1
	keys := fragmented.Keys[:0]
	for _, k := range fragmented.Keys {
		idx, _ := snapshotIndex(k.SeqNum(), i.snapshots)
		if currentIdx == idx {
			continue
		}
		if idx == 0 && i.elideRangeTombstone(fragmented.Start, fragmented.End) {
			// This is the last snapshot stripe and the range tombstone
			// can be elided.
			break
		}

		keys = append(keys, k)
		if idx == 0 {
			// This is the last snapshot stripe.
			break
		}
		currentIdx = idx
	}
	if len(keys) > 0 {
		i.tombstones = append(i.tombstones, keyspan.Span{
			Start: fragmented.Start,
			End:   fragmented.End,
			Keys:  keys,
		})
	}
}

func (i *compactionIter) emitRangeKeyChunk(fragmented keyspan.Span) {
	// Elision of snapshot stripes happens in rangeKeyCompactionTransform, so no need to
	// do that here.
	if len(fragmented.Keys) > 0 {
		i.rangeKeys = append(i.rangeKeys, fragmented)
	}
}

// maybeZeroSeqnum attempts to set the seqnum for the current key to 0. Doing
// so improves compression and enables an optimization during forward iteration
// to skip some key comparisons. The seqnum for an entry can be zeroed if the
// entry is on the bottom snapshot stripe and on the bottom level of the LSM.
func (i *compactionIter) maybeZeroSeqnum(snapshotIdx int) {
	if !i.allowZeroSeqNum {
		// TODO(peter): allowZeroSeqNum applies to the entire compaction. We could
		// make the determination on a key by key basis, similar to what is done
		// for elideTombstone. Need to add a benchmark for compactionIter to verify
		// that isn't too expensive.
		return
	}
	if snapshotIdx > 0 {
		// This is not the last snapshot
		return
	}
	i.key.SetSeqNum(base.SeqNumZero)
}

// A frontier is used to monitor a compaction's progression across the user
// keyspace.
//
// A frontier hold a user key boundary that it's concerned with in its `key`
// field. If/when the compaction iterator returns an InternalKey with a user key
// _k_ such that k ≥ frontier.key, the compaction iterator invokes the
// frontier's `reached` function, passing _k_ as its argument.
//
// The `reached` function returns a new value to use as the key. If `reached`
// returns nil, the frontier is forgotten and its `reached` method will not be
// invoked again, unless the user calls [Update] to set a new key.
//
// A frontier's key may be updated outside the context of a `reached`
// invocation at any time, through its Update method.
type frontier struct {
	// container points to the containing *frontiers that was passed to Init
	// when the frontier was initialized.
	container *frontiers

	// key holds the frontier's current key. If nil, this frontier is inactive
	// and its reached func will not be invoked. The value of this key may only
	// be updated by the `frontiers` type, or the Update method.
	key []byte

	// reached is invoked to inform a frontier that its key has been reached.
	// It's invoked with the user key that reached the limit. The `key` argument
	// is guaranteed to be ≥ the frontier's key.
	//
	// After reached is invoked, the frontier's key is updated to the return
	// value of `reached`. Note bene, the frontier is permitted to update its
	// key to a user key ≤ the argument `key`.
	//
	// If a frontier is set to key k1, and reached(k2) is invoked (k2 ≥ k1), the
	// frontier will receive reached(k2) calls until it returns nil or a key
	// `k3` such that k2 < k3. This property is useful for frontiers that use
	// `reached` invocations to drive iteration through collections of keys that
	// may contain multiple keys that are both < k2 and ≥ k1.
	reached func(key []byte) (next []byte)
}

// Init initializes the frontier with the provided key and reached callback.
// The frontier is attached to the provided *frontiers and the provided reached
// func will be invoked when the *frontiers is advanced to a key ≥ this
// frontier's key.
func (f *frontier) Init(
	frontiers *frontiers, initialKey []byte, reached func(key []byte) (next []byte),
) {
	*f = frontier{
		container: frontiers,
		key:       initialKey,
		reached:   reached,
	}
	if initialKey != nil {
		f.container.push(f)
	}
}

// String implements fmt.Stringer.
func (f *frontier) String() string {
	return string(f.key)
}

// Update replaces the existing frontier's key with the provided key. The
// frontier's reached func will be invoked when the new key is reached.
func (f *frontier) Update(key []byte) {
	c := f.container
	prevKeyIsNil := f.key == nil
	f.key = key
	if prevKeyIsNil {
		if key != nil {
			c.push(f)
		}
		return
	}

	// Find the frontier within the heap (it must exist within the heap because
	// f.key was != nil). If the frontier key is now nil, remove it from the
	// heap. Otherwise, fix up its position.
	for i := 0; i < len(c.items); i++ {
		if c.items[i] == f {
			if key != nil {
				c.fix(i)
			} else {
				n := c.len() - 1
				c.swap(i, n)
				c.down(i, n)
				c.items = c.items[:n]
			}
			return
		}
	}
	panic("unreachable")
}

// frontiers is used to track progression of a task (eg, compaction) across the
// keyspace. Clients that want to be informed when the task advances to a key ≥
// some frontier may register a frontier, providing a callback. The task calls
// `Advance(k)` with each user key encountered, which invokes the `reached` func
// on all tracked frontiers with `key`s ≤ k.
//
// Internally, frontiers is implemented as a simple heap.
type frontiers struct {
	cmp   Compare
	items []*frontier
}

// String implements fmt.Stringer.
func (f *frontiers) String() string {
	var buf bytes.Buffer
	for i := 0; i < len(f.items); i++ {
		if i > 0 {
			fmt.Fprint(&buf, ", ")
		}
		fmt.Fprintf(&buf, "%s: %q", f.items[i], f.items[i].key)
	}
	return buf.String()
}

// Advance notifies all member frontiers with keys ≤ k.
func (f *frontiers) Advance(k []byte) {
	for len(f.items) > 0 && f.cmp(k, f.items[0].key) >= 0 {
		// This frontier has been reached. Invoke the closure and update with
		// the next frontier.
		f.items[0].key = f.items[0].reached(k)
		if f.items[0].key == nil {
			// This was the final frontier that this user was concerned with.
			// Remove it from the heap.
			f.pop()
		} else {
			// Fix up the heap root.
			f.fix(0)
		}
	}
}

func (f *frontiers) len() int {
	return len(f.items)
}

func (f *frontiers) less(i, j int) bool {
	return f.cmp(f.items[i].key, f.items[j].key) < 0
}

func (f *frontiers) swap(i, j int) {
	f.items[i], f.items[j] = f.items[j], f.items[i]
}

// fix, up and down are copied from the go stdlib.

func (f *frontiers) fix(i int) {
	if !f.down(i, f.len()) {
		f.up(i)
	}
}

func (f *frontiers) push(ff *frontier) {
	n := len(f.items)
	f.items = append(f.items, ff)
	f.up(n)
}

func (f *frontiers) pop() *frontier {
	n := f.len() - 1
	f.swap(0, n)
	f.down(0, n)
	item := f.items[n]
	f.items = f.items[:n]
	return item
}

func (f *frontiers) up(j int) {
	for {
		i := (j - 1) / 2 // parent
		if i == j || !f.less(j, i) {
			break
		}
		f.swap(i, j)
		j = i
	}
}

func (f *frontiers) down(i0, n int) bool {
	i := i0
	for {
		j1 := 2*i + 1
		if j1 >= n || j1 < 0 { // j1 < 0 after int overflow
			break
		}
		j := j1 // left child
		if j2 := j1 + 1; j2 < n && f.less(j2, j1) {
			j = j2 // = 2*i + 2  // right child
		}
		if !f.less(j, i) {
			break
		}
		f.swap(i, j)
		i = j
	}
	return i > i0
}
