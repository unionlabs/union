// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"context"
	"fmt"
	"runtime/debug"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/sstable"
)

// tableNewIters creates a new point and range-del iterator for the given file
// number.
//
// On success, the internalIterator is not-nil and must be closed; the
// FragmentIterator can be nil.
// TODO(radu): always return a non-nil FragmentIterator.
//
// On error, the iterators are nil.
//
// The only (non-test) implementation of tableNewIters is tableCacheContainer.newIters().
type tableNewIters func(
	ctx context.Context,
	file *manifest.FileMetadata,
	opts *IterOptions,
	internalOpts internalIterOpts,
) (internalIterator, keyspan.FragmentIterator, error)

// tableNewRangeDelIter takes a tableNewIters and returns a TableNewSpanIter
// for the rangedel iterator returned by tableNewIters.
func tableNewRangeDelIter(ctx context.Context, newIters tableNewIters) keyspan.TableNewSpanIter {
	return func(file *manifest.FileMetadata, iterOptions keyspan.SpanIterOptions) (keyspan.FragmentIterator, error) {
		iter, rangeDelIter, err := newIters(ctx, file, nil, internalIterOpts{})
		if iter != nil {
			_ = iter.Close()
		}
		if rangeDelIter == nil {
			rangeDelIter = emptyKeyspanIter
		}
		return rangeDelIter, err
	}
}

type internalIterOpts struct {
	bytesIterated      *uint64
	bufferPool         *sstable.BufferPool
	stats              *base.InternalIteratorStats
	boundLimitedFilter sstable.BoundLimitedBlockPropertyFilter
}

// levelIter provides a merged view of the sstables in a level.
//
// levelIter is used during compaction and as part of the Iterator
// implementation. When used as part of the Iterator implementation, level
// iteration needs to "pause" at sstable boundaries if a range deletion
// tombstone is the source of that boundary. We know if a range tombstone is
// the smallest or largest key in a file because the kind will be
// InternalKeyKindRangeDeletion. If the boundary key is a range deletion
// tombstone, we materialize a fake entry to return from levelIter. This
// prevents mergingIter from advancing past the sstable until the sstable
// contains the smallest (or largest for reverse iteration) key in the merged
// heap. Note that mergingIter treats a range deletion tombstone returned by
// the point iterator as a no-op.
//
// SeekPrefixGE presents the need for a second type of pausing. If an sstable
// iterator returns "not found" for a SeekPrefixGE operation, we don't want to
// advance to the next sstable as the "not found" does not indicate that all of
// the keys in the sstable are less than the search key. Advancing to the next
// sstable would cause us to skip over range tombstones, violating
// correctness. Instead, SeekPrefixGE creates a synthetic boundary key with the
// kind InternalKeyKindRangeDeletion which will be used to pause the levelIter
// at the sstable until the mergingIter is ready to advance past it.
type levelIter struct {
	// The context is stored here since (a) iterators are expected to be
	// short-lived (since they pin sstables), (b) plumbing a context into every
	// method is very painful, (c) they do not (yet) respect context
	// cancellation and are only used for tracing.
	ctx      context.Context
	logger   Logger
	comparer *Comparer
	cmp      Compare
	split    Split
	// The lower/upper bounds for iteration as specified at creation or the most
	// recent call to SetBounds.
	lower []byte
	upper []byte
	// The iterator options for the currently open table. If
	// tableOpts.{Lower,Upper}Bound are nil, the corresponding iteration boundary
	// does not lie within the table bounds.
	tableOpts IterOptions
	// The LSM level this levelIter is initialized for.
	level manifest.Level
	// The keys to return when iterating past an sstable boundary and that
	// boundary is a range deletion tombstone. The boundary could be smallest
	// (i.e. arrived at with Prev), or largest (arrived at with Next).
	smallestBoundary *InternalKey
	largestBoundary  *InternalKey
	// combinedIterState may be set when a levelIter is used during user
	// iteration. Although levelIter only iterates over point keys, it's also
	// responsible for lazily constructing the combined range & point iterator
	// when it observes a file containing range keys. If the combined iter
	// state's initialized field is true, the iterator is already using combined
	// iterator, OR the iterator is not configured to use combined iteration. If
	// it's false, the levelIter must set the `triggered` and `key` fields when
	// the levelIter passes over a file containing range keys. See the
	// lazyCombinedIter for more details.
	combinedIterState *combinedIterState
	// A synthetic boundary key to return when SeekPrefixGE finds an sstable
	// which doesn't contain the search key, but which does contain range
	// tombstones.
	syntheticBoundary InternalKey
	// The iter for the current file. It is nil under any of the following conditions:
	// - files.Current() == nil
	// - err != nil
	// - some other constraint, like the bounds in opts, caused the file at index to not
	//   be relevant to the iteration.
	iter internalIterator
	// iterFile holds the current file. It is always equal to l.files.Current().
	iterFile *fileMetadata
	// filteredIter is an optional interface that may be implemented by internal
	// iterators that perform filtering of keys. When a new file's iterator is
	// opened, it's tested to see if it implements filteredIter. If it does,
	// it's stored here to allow the level iterator to recognize when keys were
	// omitted from iteration results due to filtering. This is important when a
	// file contains range deletions that may delete keys from other files. The
	// levelIter must not advance to the next file until the mergingIter has
	// advanced beyond the file's bounds. See
	// levelIterBoundaryContext.isIgnorableBoundaryKey.
	filteredIter filteredIter
	newIters     tableNewIters
	// When rangeDelIterPtr != nil, the caller requires that *rangeDelIterPtr must
	// point to a range del iterator corresponding to the current file. When this
	// iterator returns nil, *rangeDelIterPtr should also be set to nil. Whenever
	// a non-nil internalIterator is placed in rangeDelIterPtr, a copy is placed
	// in rangeDelIterCopy. This is done for the following special case:
	// when this iterator returns nil because of exceeding the bounds, we don't
	// close iter and *rangeDelIterPtr since we could reuse it in the next seek. But
	// we need to set *rangeDelIterPtr to nil because of the aforementioned contract.
	// This copy is used to revive the *rangeDelIterPtr in the case of reuse.
	rangeDelIterPtr  *keyspan.FragmentIterator
	rangeDelIterCopy keyspan.FragmentIterator
	files            manifest.LevelIterator
	err              error

	// Pointer into this level's entry in `mergingIterLevel::levelIterBoundaryContext`.
	// We populate it with the corresponding bounds for the currently opened file. It is used for
	// two purposes (described for forward iteration. The explanation for backward iteration is
	// similar.)
	// - To limit the optimization that seeks lower-level iterators past keys shadowed by a range
	//   tombstone. Limiting this seek to the file largestUserKey is necessary since
	//   range tombstones are stored untruncated, while they only apply to keys within their
	//   containing file's boundaries. For a detailed example, see comment above `mergingIter`.
	// - To constrain the tombstone to act-within the bounds of the sstable when checking
	//   containment. For forward iteration we need the smallestUserKey.
	//
	// An example is sstable bounds [c#8, g#12] containing a tombstone [b, i)#7.
	// - When doing a SeekGE to user key X, the levelIter is at this sstable because X is either within
	//   the sstable bounds or earlier than the start of the sstable (and there is no sstable in
	//   between at this level). If X >= smallestUserKey, and the tombstone [b, i) contains X,
	//   it is correct to SeekGE the sstables at lower levels to min(g, i) (i.e., min of
	//   largestUserKey, tombstone.End) since any user key preceding min(g, i) must be covered by this
	//   tombstone (since it cannot have a version younger than this tombstone as it is at a lower
	//   level). And even if X = smallestUserKey or equal to the start user key of the tombstone,
	//   if the above conditions are satisfied we know that the internal keys corresponding to X at
	//   lower levels must have a version smaller than that in this file (again because of the level
	//   argument). So we don't need to use sequence numbers for this comparison.
	// - When checking whether this tombstone deletes internal key X we know that the levelIter is at this
	//   sstable so (repeating the above) X.UserKey is either within the sstable bounds or earlier than the
	//   start of the sstable (and there is no sstable in between at this level).
	//   - X is at at a lower level. If X.UserKey >= smallestUserKey, and the tombstone contains
	//     X.UserKey, we know X is deleted. This argument also works when X is a user key (we use
	//     it when seeking to test whether a user key is deleted).
	//   - X is at the same level. X must be within the sstable bounds of the tombstone so the
	//     X.UserKey >= smallestUserKey comparison is trivially true. In addition to the tombstone containing
	//     X we need to compare the sequence number of X and the tombstone (we don't need to look
	//     at how this tombstone is truncated to act-within the file bounds, which are InternalKeys,
	//     since X and the tombstone are from the same file).
	//
	// Iterating backwards has one more complication when checking whether a tombstone deletes
	// internal key X at a lower level (the construction we do here also works for a user key X).
	// Consider sstable bounds [c#8, g#InternalRangeDelSentinel] containing a tombstone [b, i)#7.
	// If we are positioned at key g#10 at a lower sstable, the tombstone we will see is [b, i)#7,
	// since the higher sstable is positioned at a key <= g#10. We should not use this tombstone
	// to delete g#10. This requires knowing that the largestUserKey is a range delete sentinel,
	// which we set in a separate bool below.
	//
	// These fields differs from the `*Boundary` fields in a few ways:
	// - `*Boundary` is only populated when the iterator is positioned exactly on the sentinel key.
	// - `*Boundary` can hold either the lower- or upper-bound, depending on the iterator direction.
	// - `*Boundary` is not exposed to the next higher-level iterator, i.e., `mergingIter`.
	boundaryContext *levelIterBoundaryContext

	// internalOpts holds the internal iterator options to pass to the table
	// cache when constructing new table iterators.
	internalOpts internalIterOpts

	// Scratch space for the obsolete keys filter, when there are no other block
	// property filters specified. See the performance note where
	// IterOptions.PointKeyFilters is declared.
	filtersBuf [1]BlockPropertyFilter

	// Disable invariant checks even if they are otherwise enabled. Used by tests
	// which construct "impossible" situations (e.g. seeking to a key before the
	// lower bound).
	disableInvariants bool
}

// filteredIter is an additional interface implemented by iterators that may
// skip over point keys during iteration. The sstable.Iterator implements this
// interface.
type filteredIter interface {
	// MaybeFilteredKeys may be called when an iterator is exhausted, indicating
	// whether or not the iterator's last positioning method may have skipped
	// any keys due to low-level filters.
	//
	// When an iterator is configured to use block-property filters, the
	// low-level iterator may skip over blocks or whole sstables of keys.
	// Implementations that implement skipping must implement this interface.
	// Higher-level iterators require it to preserve invariants (eg, a levelIter
	// used in a mergingIter must keep the file's range-del iterator open until
	// the mergingIter has moved past the file's bounds, even if all of the
	// file's point keys were filtered).
	//
	// MaybeFilteredKeys may always return false positives, that is it may
	// return true when no keys were filtered. It should only be called when the
	// iterator is exhausted. It must never return false negatives when the
	// iterator is exhausted.
	MaybeFilteredKeys() bool
}

// levelIter implements the base.InternalIterator interface.
var _ base.InternalIterator = (*levelIter)(nil)

// newLevelIter returns a levelIter. It is permissible to pass a nil split
// parameter if the caller is never going to call SeekPrefixGE.
func newLevelIter(
	opts IterOptions,
	comparer *Comparer,
	newIters tableNewIters,
	files manifest.LevelIterator,
	level manifest.Level,
	internalOpts internalIterOpts,
) *levelIter {
	l := &levelIter{}
	l.init(context.Background(), opts, comparer, newIters, files, level,
		internalOpts)
	return l
}

func (l *levelIter) init(
	ctx context.Context,
	opts IterOptions,
	comparer *Comparer,
	newIters tableNewIters,
	files manifest.LevelIterator,
	level manifest.Level,
	internalOpts internalIterOpts,
) {
	l.ctx = ctx
	l.err = nil
	l.level = level
	l.logger = opts.getLogger()
	l.lower = opts.LowerBound
	l.upper = opts.UpperBound
	l.tableOpts.TableFilter = opts.TableFilter
	l.tableOpts.PointKeyFilters = opts.PointKeyFilters
	if len(opts.PointKeyFilters) == 0 {
		l.tableOpts.PointKeyFilters = l.filtersBuf[:0:1]
	}
	l.tableOpts.UseL6Filters = opts.UseL6Filters
	l.tableOpts.level = l.level
	l.tableOpts.snapshotForHideObsoletePoints = opts.snapshotForHideObsoletePoints
	l.comparer = comparer
	l.cmp = comparer.Compare
	l.split = comparer.Split
	l.iterFile = nil
	l.newIters = newIters
	l.files = files
	l.internalOpts = internalOpts
}

func (l *levelIter) initRangeDel(rangeDelIter *keyspan.FragmentIterator) {
	l.rangeDelIterPtr = rangeDelIter
}

func (l *levelIter) initBoundaryContext(context *levelIterBoundaryContext) {
	l.boundaryContext = context
}

func (l *levelIter) initCombinedIterState(state *combinedIterState) {
	l.combinedIterState = state
}

func (l *levelIter) maybeTriggerCombinedIteration(file *fileMetadata, dir int) {
	// If we encounter a file that contains range keys, we may need to
	// trigger a switch to combined range-key and point-key iteration,
	// if the *pebble.Iterator is configured for it. This switch is done
	// lazily because range keys are intended to be rare, and
	// constructing the range-key iterator substantially adds to the
	// cost of iterator construction and seeking.
	//
	// If l.combinedIterState.initialized is already true, either the
	// iterator is already using combined iteration or the iterator is not
	// configured to observe range keys. Either way, there's nothing to do.
	// If false, trigger the switch to combined iteration, using the the
	// file's bounds to seek the range-key iterator appropriately.
	//
	// We only need to trigger combined iteration if the file contains
	// RangeKeySets: if there are only Unsets and Dels, the user will observe no
	// range keys regardless. If this file has table stats available, they'll
	// tell us whether the file has any RangeKeySets. Otherwise, we must
	// fallback to assuming it does if HasRangeKeys=true.
	if file != nil && file.HasRangeKeys && l.combinedIterState != nil && !l.combinedIterState.initialized &&
		(l.upper == nil || l.cmp(file.SmallestRangeKey.UserKey, l.upper) < 0) &&
		(l.lower == nil || l.cmp(file.LargestRangeKey.UserKey, l.lower) > 0) &&
		(!file.StatsValid() || file.Stats.NumRangeKeySets > 0) {
		// The file contains range keys, and we're not using combined iteration yet.
		// Trigger a switch to combined iteration. It's possible that a switch has
		// already been triggered if multiple levels encounter files containing
		// range keys while executing a single mergingIter operation. In this case,
		// we need to compare the existing key recorded to l.combinedIterState.key,
		// adjusting it if our key is smaller (forward iteration) or larger
		// (backward iteration) than the existing key.
		//
		// These key comparisons are only required during a single high-level
		// iterator operation. When the high-level iter op completes,
		// iinitialized will be true, and future calls to this function will be
		// no-ops.
		switch dir {
		case +1:
			if !l.combinedIterState.triggered {
				l.combinedIterState.triggered = true
				l.combinedIterState.key = file.SmallestRangeKey.UserKey
			} else if l.cmp(l.combinedIterState.key, file.SmallestRangeKey.UserKey) > 0 {
				l.combinedIterState.key = file.SmallestRangeKey.UserKey
			}
		case -1:
			if !l.combinedIterState.triggered {
				l.combinedIterState.triggered = true
				l.combinedIterState.key = file.LargestRangeKey.UserKey
			} else if l.cmp(l.combinedIterState.key, file.LargestRangeKey.UserKey) < 0 {
				l.combinedIterState.key = file.LargestRangeKey.UserKey
			}
		}
	}
}

func (l *levelIter) findFileGE(key []byte, flags base.SeekGEFlags) *fileMetadata {
	// Find the earliest file whose largest key is >= key.

	// NB: if flags.TrySeekUsingNext()=true, the levelIter must respect it. If
	// the levelIter is positioned at the key P, it must return a key ≥ P. If
	// used within a merging iterator, the merging iterator will depend on the
	// levelIter only moving forward to maintain heap invariants.

	// Ordinarily we seek the LevelIterator using SeekGE. In some instances, we
	// Next instead. In other instances, we try Next-ing first, falling back to
	// seek:
	//   a) flags.TrySeekUsingNext(): The top-level Iterator knows we're seeking
	//      to a key later than the current iterator position. We don't know how
	//      much later the seek key is, so it's possible there are many sstables
	//      between the current position and the seek key. However in most real-
	//      world use cases, the seek key is likely to be nearby. Rather than
	//      performing a log(N) seek through the file metadata, we next a few
	//      times from from our existing location. If we don't find a file whose
	//      largest is >= key within a few nexts, we fall back to seeking.
	//
	//      Note that in this case, the file returned by findFileGE may be
	//      different than the file returned by a raw binary search (eg, when
	//      TrySeekUsingNext=false). This is possible because the most recent
	//      positioning operation may have already determined that previous
	//      files' keys that are ≥ key are all deleted. This information is
	//      encoded within the iterator's current iterator position and is
	//      unavailable to a fresh binary search.
	//
	//   b) flags.RelativeSeek(): The merging iterator decided to re-seek this
	//      level according to a range tombstone. When lazy combined iteration
	//      is enabled, the level iterator is responsible for watching for
	//      files containing range keys and triggering the switch to combined
	//      iteration when such a file is observed. If a range deletion was
	//      observed in a higher level causing the merging iterator to seek the
	//      level to the range deletion's end key, we need to check whether all
	//      of the files between the old position and the new position contain
	//      any range keys.
	//
	//      In this scenario, we don't seek the LevelIterator and instead we
	//      Next it, one file at a time, checking each for range keys. The
	//      merging iterator sets this flag to inform us that we're moving
	//      forward relative to the existing position and that we must examine
	//      each intermediate sstable's metadata for lazy-combined iteration.
	//      In this case, we only Next and never Seek. We set nextsUntilSeek=-1
	//      to signal this intention.
	//
	// NB: At most one of flags.RelativeSeek() and flags.TrySeekUsingNext() may
	// be set, because the merging iterator re-seeks relative seeks with
	// explicitly only the RelativeSeek flag set.
	var nextsUntilSeek int
	var nextInsteadOfSeek bool
	if flags.TrySeekUsingNext() {
		nextInsteadOfSeek = true
		nextsUntilSeek = 4 // arbitrary
	}
	if flags.RelativeSeek() && l.combinedIterState != nil && !l.combinedIterState.initialized {
		nextInsteadOfSeek = true
		nextsUntilSeek = -1
	}

	var m *fileMetadata
	if nextInsteadOfSeek {
		m = l.iterFile
	} else {
		m = l.files.SeekGE(l.cmp, key)
	}
	// The below loop has a bit of an unusual organization. There are several
	// conditions under which we need to Next to a later file. If none of those
	// conditions are met, the file in `m` is okay to return. The loop body is
	// structured with a series of if statements, each of which may continue the
	// loop to the next file. If none of the statements are met, the end of the
	// loop body is a break.
	for m != nil {
		if m.HasRangeKeys {
			l.maybeTriggerCombinedIteration(m, +1)

			// Some files may only contain range keys, which we can skip.
			// NB: HasPointKeys=true if the file contains any points or range
			// deletions (which delete points).
			if !m.HasPointKeys {
				m = l.files.Next()
				continue
			}
		}

		// This file has point keys.
		//
		// However, there are a couple reasons why `m` may not be positioned ≥
		// `key` yet:
		//
		// 1. If SeekGE(key) landed on a file containing range keys, the file
		//    may contain range keys ≥ `key` but no point keys ≥ `key`.
		// 2. When nexting instead of seeking, we must check to see whether
		//    we've nexted sufficiently far, or we need to next again.
		//
		// If the file does not contain point keys ≥ `key`, next to continue
		// looking for a file that does.
		if (m.HasRangeKeys || nextInsteadOfSeek) && l.cmp(m.LargestPointKey.UserKey, key) < 0 {
			// If nextInsteadOfSeek is set and nextsUntilSeek is non-negative,
			// the iterator has been nexting hoping to discover the relevant
			// file without seeking. It's exhausted the allotted nextsUntilSeek
			// and should seek to the sought key.
			if nextInsteadOfSeek && nextsUntilSeek == 0 {
				nextInsteadOfSeek = false
				m = l.files.SeekGE(l.cmp, key)
				continue
			} else if nextsUntilSeek > 0 {
				nextsUntilSeek--
			}
			m = l.files.Next()
			continue
		}

		// This file has a point key bound ≥ `key`. But the largest point key
		// bound may still be a range deletion sentinel, which is exclusive.  In
		// this case, the file doesn't actually contain any point keys equal to
		// `key`. We next to keep searching for a file that actually contains
		// point keys ≥ key.
		//
		// Additionally, this prevents loading untruncated range deletions from
		// a table which can't possibly contain the target key and is required
		// for correctness by mergingIter.SeekGE (see the comment in that
		// function).
		if m.LargestPointKey.IsExclusiveSentinel() && l.cmp(m.LargestPointKey.UserKey, key) == 0 {
			m = l.files.Next()
			continue
		}

		// This file contains point keys ≥ `key`. Break and return it.
		break
	}
	return m
}

func (l *levelIter) findFileLT(key []byte, flags base.SeekLTFlags) *fileMetadata {
	// Find the last file whose smallest key is < ikey.

	// Ordinarily we seek the LevelIterator using SeekLT.
	//
	// When lazy combined iteration is enabled, there's a complication. The
	// level iterator is responsible for watching for files containing range
	// keys and triggering the switch to combined iteration when such a file is
	// observed. If a range deletion was observed in a higher level causing the
	// merging iterator to seek the level to the range deletion's start key, we
	// need to check whether all of the files between the old position and the
	// new position contain any range keys.
	//
	// In this scenario, we don't seek the LevelIterator and instead we Prev it,
	// one file at a time, checking each for range keys.
	prevInsteadOfSeek := flags.RelativeSeek() && l.combinedIterState != nil && !l.combinedIterState.initialized

	var m *fileMetadata
	if prevInsteadOfSeek {
		m = l.iterFile
	} else {
		m = l.files.SeekLT(l.cmp, key)
	}
	// The below loop has a bit of an unusual organization. There are several
	// conditions under which we need to Prev to a previous file. If none of
	// those conditions are met, the file in `m` is okay to return. The loop
	// body is structured with a series of if statements, each of which may
	// continue the loop to the previous file. If none of the statements are
	// met, the end of the loop body is a break.
	for m != nil {
		if m.HasRangeKeys {
			l.maybeTriggerCombinedIteration(m, -1)

			// Some files may only contain range keys, which we can skip.
			// NB: HasPointKeys=true if the file contains any points or range
			// deletions (which delete points).
			if !m.HasPointKeys {
				m = l.files.Prev()
				continue
			}
		}

		// This file has point keys.
		//
		// However, there are a couple reasons why `m` may not be positioned <
		// `key` yet:
		//
		// 1. If SeekLT(key) landed on a file containing range keys, the file
		//    may contain range keys < `key` but no point keys < `key`.
		// 2. When preving instead of seeking, we must check to see whether
		//    we've preved sufficiently far, or we need to prev again.
		//
		// If the file does not contain point keys < `key`, prev to continue
		// looking for a file that does.
		if (m.HasRangeKeys || prevInsteadOfSeek) && l.cmp(m.SmallestPointKey.UserKey, key) >= 0 {
			m = l.files.Prev()
			continue
		}

		// This file contains point keys < `key`. Break and return it.
		break
	}
	return m
}

// Init the iteration bounds for the current table. Returns -1 if the table
// lies fully before the lower bound, +1 if the table lies fully after the
// upper bound, and 0 if the table overlaps the iteration bounds.
func (l *levelIter) initTableBounds(f *fileMetadata) int {
	l.tableOpts.LowerBound = l.lower
	if l.tableOpts.LowerBound != nil {
		if l.cmp(f.LargestPointKey.UserKey, l.tableOpts.LowerBound) < 0 {
			// The largest key in the sstable is smaller than the lower bound.
			return -1
		}
		if l.cmp(l.tableOpts.LowerBound, f.SmallestPointKey.UserKey) <= 0 {
			// The lower bound is smaller or equal to the smallest key in the
			// table. Iteration within the table does not need to check the lower
			// bound.
			l.tableOpts.LowerBound = nil
		}
	}
	l.tableOpts.UpperBound = l.upper
	if l.tableOpts.UpperBound != nil {
		if l.cmp(f.SmallestPointKey.UserKey, l.tableOpts.UpperBound) >= 0 {
			// The smallest key in the sstable is greater than or equal to the upper
			// bound.
			return 1
		}
		if l.cmp(l.tableOpts.UpperBound, f.LargestPointKey.UserKey) > 0 {
			// The upper bound is greater than the largest key in the
			// table. Iteration within the table does not need to check the upper
			// bound. NB: tableOpts.UpperBound is exclusive and f.LargestPointKey is
			// inclusive.
			l.tableOpts.UpperBound = nil
		}
	}
	return 0
}

type loadFileReturnIndicator int8

const (
	noFileLoaded loadFileReturnIndicator = iota
	fileAlreadyLoaded
	newFileLoaded
)

func (l *levelIter) loadFile(file *fileMetadata, dir int) loadFileReturnIndicator {
	l.smallestBoundary = nil
	l.largestBoundary = nil
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}
	if l.iterFile == file {
		if l.err != nil {
			return noFileLoaded
		}
		if l.iter != nil {
			// We don't bother comparing the file bounds with the iteration bounds when we have
			// an already open iterator. It is possible that the iter may not be relevant given the
			// current iteration bounds, but it knows those bounds, so it will enforce them.
			if l.rangeDelIterPtr != nil {
				*l.rangeDelIterPtr = l.rangeDelIterCopy
			}

			// There are a few reasons we might not have triggered combined
			// iteration yet, even though we already had `file` open.
			// 1. If the bounds changed, we might have previously avoided
			//    switching to combined iteration because the bounds excluded
			//    the range keys contained in this file.
			// 2. If an existing iterator was reconfigured to iterate over range
			//    keys (eg, using SetOptions), then we wouldn't have triggered
			//    the switch to combined iteration yet.
			l.maybeTriggerCombinedIteration(file, dir)
			return fileAlreadyLoaded
		}
		// We were already at file, but don't have an iterator, probably because the file was
		// beyond the iteration bounds. It may still be, but it is also possible that the bounds
		// have changed. We handle that below.
	}

	// Close both iter and rangeDelIterPtr. While mergingIter knows about
	// rangeDelIterPtr, it can't call Close() on it because it does not know
	// when the levelIter will switch it. Note that levelIter.Close() can be
	// called multiple times.
	if err := l.Close(); err != nil {
		return noFileLoaded
	}

	for {
		l.iterFile = file
		if file == nil {
			return noFileLoaded
		}

		l.maybeTriggerCombinedIteration(file, dir)
		if !file.HasPointKeys {
			switch dir {
			case +1:
				file = l.files.Next()
				continue
			case -1:
				file = l.files.Prev()
				continue
			}
		}

		switch l.initTableBounds(file) {
		case -1:
			// The largest key in the sstable is smaller than the lower bound.
			if dir < 0 {
				return noFileLoaded
			}
			file = l.files.Next()
			continue
		case +1:
			// The smallest key in the sstable is greater than or equal to the upper
			// bound.
			if dir > 0 {
				return noFileLoaded
			}
			file = l.files.Prev()
			continue
		}

		var rangeDelIter keyspan.FragmentIterator
		var iter internalIterator
		iter, rangeDelIter, l.err = l.newIters(l.ctx, l.iterFile, &l.tableOpts, l.internalOpts)
		l.iter = iter
		if l.err != nil {
			return noFileLoaded
		}
		if rangeDelIter != nil {
			if fi, ok := iter.(filteredIter); ok {
				l.filteredIter = fi
			} else {
				l.filteredIter = nil
			}
		} else {
			l.filteredIter = nil
		}
		if l.rangeDelIterPtr != nil {
			*l.rangeDelIterPtr = rangeDelIter
			l.rangeDelIterCopy = rangeDelIter
		} else if rangeDelIter != nil {
			rangeDelIter.Close()
		}
		if l.boundaryContext != nil {
			l.boundaryContext.smallestUserKey = file.Smallest.UserKey
			l.boundaryContext.largestUserKey = file.Largest.UserKey
			l.boundaryContext.isLargestUserKeyExclusive = file.Largest.IsExclusiveSentinel()
		}
		return newFileLoaded
	}
}

// In race builds we verify that the keys returned by levelIter lie within
// [lower,upper).
func (l *levelIter) verify(key *InternalKey, val base.LazyValue) (*InternalKey, base.LazyValue) {
	// Note that invariants.Enabled is a compile time constant, which means the
	// block of code will be compiled out of normal builds making this method
	// eligible for inlining. Do not change this to use a variable.
	if invariants.Enabled && !l.disableInvariants && key != nil {
		// We allow returning a boundary key that is outside of the lower/upper
		// bounds as such keys are always range tombstones which will be skipped by
		// the Iterator.
		if l.lower != nil && key != l.smallestBoundary && l.cmp(key.UserKey, l.lower) < 0 {
			l.logger.Fatalf("levelIter %s: lower bound violation: %s < %s\n%s", l.level, key, l.lower, debug.Stack())
		}
		if l.upper != nil && key != l.largestBoundary && l.cmp(key.UserKey, l.upper) > 0 {
			l.logger.Fatalf("levelIter %s: upper bound violation: %s > %s\n%s", l.level, key, l.upper, debug.Stack())
		}
	}
	return key, val
}

func (l *levelIter) SeekGE(key []byte, flags base.SeekGEFlags) (*InternalKey, base.LazyValue) {
	l.err = nil // clear cached iteration error
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}
	// NB: the top-level Iterator has already adjusted key based on
	// IterOptions.LowerBound.
	loadFileIndicator := l.loadFile(l.findFileGE(key, flags), +1)
	if loadFileIndicator == noFileLoaded {
		return nil, base.LazyValue{}
	}
	if loadFileIndicator == newFileLoaded {
		// File changed, so l.iter has changed, and that iterator is not
		// positioned appropriately.
		flags = flags.DisableTrySeekUsingNext()
	}
	if ikey, val := l.iter.SeekGE(key, flags); ikey != nil {
		return l.verify(ikey, val)
	}
	return l.verify(l.skipEmptyFileForward())
}

func (l *levelIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	l.err = nil // clear cached iteration error
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}

	// NB: the top-level Iterator has already adjusted key based on
	// IterOptions.LowerBound.
	loadFileIndicator := l.loadFile(l.findFileGE(key, flags), +1)
	if loadFileIndicator == noFileLoaded {
		return nil, base.LazyValue{}
	}
	if loadFileIndicator == newFileLoaded {
		// File changed, so l.iter has changed, and that iterator is not
		// positioned appropriately.
		flags = flags.DisableTrySeekUsingNext()
	}
	if key, val := l.iter.SeekPrefixGE(prefix, key, flags); key != nil {
		return l.verify(key, val)
	}
	// When SeekPrefixGE returns nil, we have not necessarily reached the end of
	// the sstable. All we know is that a key with prefix does not exist in the
	// current sstable. We do know that the key lies within the bounds of the
	// table as findFileGE found the table where key <= meta.Largest. We return
	// the table's bound with isIgnorableBoundaryKey set.
	if l.rangeDelIterPtr != nil && *l.rangeDelIterPtr != nil {
		if l.tableOpts.UpperBound != nil {
			l.syntheticBoundary.UserKey = l.tableOpts.UpperBound
			l.syntheticBoundary.Trailer = InternalKeyRangeDeleteSentinel
			l.largestBoundary = &l.syntheticBoundary
			if l.boundaryContext != nil {
				l.boundaryContext.isSyntheticIterBoundsKey = true
				l.boundaryContext.isIgnorableBoundaryKey = false
			}
			return l.verify(l.largestBoundary, base.LazyValue{})
		}
		// Return the file's largest bound, ensuring this file stays open until
		// the mergingIter advances beyond the file's bounds. We set
		// isIgnorableBoundaryKey to signal that the actual key returned should
		// be ignored, and does not represent a real key in the database.
		l.largestBoundary = &l.iterFile.LargestPointKey
		if l.boundaryContext != nil {
			l.boundaryContext.isSyntheticIterBoundsKey = false
			l.boundaryContext.isIgnorableBoundaryKey = true
		}
		return l.verify(l.largestBoundary, base.LazyValue{})
	}
	// It is possible that we are here because bloom filter matching failed. In
	// that case it is likely that all keys matching the prefix are wholly
	// within the current file and cannot be in the subsequent file. In that
	// case we don't want to go to the next file, since loading and seeking in
	// there has some cost. Additionally, for sparse key spaces, loading the
	// next file will defeat the optimization for the next SeekPrefixGE that is
	// called with flags.TrySeekUsingNext(), since for sparse key spaces it is
	// likely that the next key will also be contained in the current file.
	var n int
	if l.split != nil {
		// If the split function is specified, calculate the prefix length accordingly.
		n = l.split(l.iterFile.LargestPointKey.UserKey)
	} else {
		// If the split function is not specified, the entire key is used as the
		// prefix. This case can occur when getIter uses SeekPrefixGE.
		n = len(l.iterFile.LargestPointKey.UserKey)
	}
	if l.cmp(prefix, l.iterFile.LargestPointKey.UserKey[:n]) < 0 {
		return nil, base.LazyValue{}
	}
	return l.verify(l.skipEmptyFileForward())
}

func (l *levelIter) SeekLT(key []byte, flags base.SeekLTFlags) (*InternalKey, base.LazyValue) {
	l.err = nil // clear cached iteration error
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}

	// NB: the top-level Iterator has already adjusted key based on
	// IterOptions.UpperBound.
	if l.loadFile(l.findFileLT(key, flags), -1) == noFileLoaded {
		return nil, base.LazyValue{}
	}
	if key, val := l.iter.SeekLT(key, flags); key != nil {
		return l.verify(key, val)
	}
	return l.verify(l.skipEmptyFileBackward())
}

func (l *levelIter) First() (*InternalKey, base.LazyValue) {
	l.err = nil // clear cached iteration error
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}

	// NB: the top-level Iterator will call SeekGE if IterOptions.LowerBound is
	// set.
	if l.loadFile(l.files.First(), +1) == noFileLoaded {
		return nil, base.LazyValue{}
	}
	if key, val := l.iter.First(); key != nil {
		return l.verify(key, val)
	}
	return l.verify(l.skipEmptyFileForward())
}

func (l *levelIter) Last() (*InternalKey, base.LazyValue) {
	l.err = nil // clear cached iteration error
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}

	// NB: the top-level Iterator will call SeekLT if IterOptions.UpperBound is
	// set.
	if l.loadFile(l.files.Last(), -1) == noFileLoaded {
		return nil, base.LazyValue{}
	}
	if key, val := l.iter.Last(); key != nil {
		return l.verify(key, val)
	}
	return l.verify(l.skipEmptyFileBackward())
}

func (l *levelIter) Next() (*InternalKey, base.LazyValue) {
	if l.err != nil || l.iter == nil {
		return nil, base.LazyValue{}
	}
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}

	switch {
	case l.largestBoundary != nil:
		if l.tableOpts.UpperBound != nil {
			// The UpperBound was within this file, so don't load the next
			// file. We leave the largestBoundary unchanged so that subsequent
			// calls to Next() stay at this file. If a Seek/First/Last call is
			// made and this file continues to be relevant, loadFile() will
			// set the largestBoundary to nil.
			if l.rangeDelIterPtr != nil {
				*l.rangeDelIterPtr = nil
			}
			return nil, base.LazyValue{}
		}
		// We're stepping past the boundary key, so now we can load the next file.
		if l.loadFile(l.files.Next(), +1) != noFileLoaded {
			if key, val := l.iter.First(); key != nil {
				return l.verify(key, val)
			}
			return l.verify(l.skipEmptyFileForward())
		}
		return nil, base.LazyValue{}

	default:
		// Reset the smallest boundary since we're moving away from it.
		l.smallestBoundary = nil
		if key, val := l.iter.Next(); key != nil {
			return l.verify(key, val)
		}
	}
	return l.verify(l.skipEmptyFileForward())
}

func (l *levelIter) NextPrefix(succKey []byte) (*InternalKey, base.LazyValue) {
	if l.err != nil || l.iter == nil {
		return nil, base.LazyValue{}
	}
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}

	switch {
	case l.largestBoundary != nil:
		if l.tableOpts.UpperBound != nil {
			// The UpperBound was within this file, so don't load the next
			// file. We leave the largestBoundary unchanged so that subsequent
			// calls to Next() stay at this file. If a Seek/First/Last call is
			// made and this file continues to be relevant, loadFile() will
			// set the largestBoundary to nil.
			if l.rangeDelIterPtr != nil {
				*l.rangeDelIterPtr = nil
			}
			return nil, base.LazyValue{}
		}
		// We're stepping past the boundary key, so we need to load a later
		// file.

	default:
		// Reset the smallest boundary since we're moving away from it.
		l.smallestBoundary = nil

		if key, val := l.iter.NextPrefix(succKey); key != nil {
			return l.verify(key, val)
		}
		// Fall through to seeking.
	}

	// Seek the manifest level iterator using TrySeekUsingNext=true and
	// RelativeSeek=true so that we take advantage of the knowledge that
	// `succKey` can only be contained in later files.
	metadataSeekFlags := base.SeekGEFlagsNone.EnableTrySeekUsingNext().EnableRelativeSeek()
	if l.loadFile(l.findFileGE(succKey, metadataSeekFlags), +1) != noFileLoaded {
		// NB: The SeekGE on the file's iterator must not set TrySeekUsingNext,
		// because l.iter is unpositioned.
		if key, val := l.iter.SeekGE(succKey, base.SeekGEFlagsNone); key != nil {
			return l.verify(key, val)
		}
		return l.verify(l.skipEmptyFileForward())
	}
	return nil, base.LazyValue{}
}

func (l *levelIter) Prev() (*InternalKey, base.LazyValue) {
	if l.err != nil || l.iter == nil {
		return nil, base.LazyValue{}
	}
	if l.boundaryContext != nil {
		l.boundaryContext.isSyntheticIterBoundsKey = false
		l.boundaryContext.isIgnorableBoundaryKey = false
	}

	switch {
	case l.smallestBoundary != nil:
		if l.tableOpts.LowerBound != nil {
			// The LowerBound was within this file, so don't load the previous
			// file. We leave the smallestBoundary unchanged so that
			// subsequent calls to Prev() stay at this file. If a
			// Seek/First/Last call is made and this file continues to be
			// relevant, loadFile() will set the smallestBoundary to nil.
			if l.rangeDelIterPtr != nil {
				*l.rangeDelIterPtr = nil
			}
			return nil, base.LazyValue{}
		}
		// We're stepping past the boundary key, so now we can load the prev file.
		if l.loadFile(l.files.Prev(), -1) != noFileLoaded {
			if key, val := l.iter.Last(); key != nil {
				return l.verify(key, val)
			}
			return l.verify(l.skipEmptyFileBackward())
		}
		return nil, base.LazyValue{}

	default:
		// Reset the largest boundary since we're moving away from it.
		l.largestBoundary = nil
		if key, val := l.iter.Prev(); key != nil {
			return l.verify(key, val)
		}
	}
	return l.verify(l.skipEmptyFileBackward())
}

func (l *levelIter) skipEmptyFileForward() (*InternalKey, base.LazyValue) {
	var key *InternalKey
	var val base.LazyValue
	// The first iteration of this loop starts with an already exhausted
	// l.iter. The reason for the exhaustion is either that we iterated to the
	// end of the sstable, or our iteration was terminated early due to the
	// presence of an upper-bound or the use of SeekPrefixGE. If
	// l.rangeDelIterPtr is non-nil, we may need to pretend the iterator is
	// not exhausted to allow for the merging to finish consuming the
	// l.rangeDelIterPtr before levelIter switches the rangeDelIter from
	// under it. This pretense is done by either generating a synthetic
	// boundary key or returning the largest key of the file, depending on the
	// exhaustion reason.

	// Subsequent iterations will examine consecutive files such that the first
	// file that does not have an exhausted iterator causes the code to return
	// that key, else the behavior described above if there is a corresponding
	// rangeDelIterPtr.
	for ; key == nil; key, val = l.iter.First() {
		if l.rangeDelIterPtr != nil {
			// We're being used as part of a mergingIter and we've exhausted the
			// current sstable. If an upper bound is present and the upper bound lies
			// within the current sstable, then we will have reached the upper bound
			// rather than the end of the sstable. We need to return a synthetic
			// boundary key so that mergingIter can use the range tombstone iterator
			// until the other levels have reached this boundary.
			//
			// It is safe to set the boundary key to the UpperBound user key
			// with the RANGEDEL sentinel since it is the smallest InternalKey
			// that matches the exclusive upper bound, and does not represent
			// a real key.
			if l.tableOpts.UpperBound != nil {
				if *l.rangeDelIterPtr != nil {
					l.syntheticBoundary.UserKey = l.tableOpts.UpperBound
					l.syntheticBoundary.Trailer = InternalKeyRangeDeleteSentinel
					l.largestBoundary = &l.syntheticBoundary
					if l.boundaryContext != nil {
						l.boundaryContext.isSyntheticIterBoundsKey = true
					}
					return l.largestBoundary, base.LazyValue{}
				}
				// Else there are no range deletions in this sstable. This
				// helps with performance when many levels are populated with
				// sstables and most don't have any actual keys within the
				// bounds.
				return nil, base.LazyValue{}
			}
			// If the boundary is a range deletion tombstone, return that key.
			if l.iterFile.LargestPointKey.Kind() == InternalKeyKindRangeDelete {
				l.largestBoundary = &l.iterFile.LargestPointKey
				if l.boundaryContext != nil {
					l.boundaryContext.isIgnorableBoundaryKey = true
				}
				return l.largestBoundary, base.LazyValue{}
			}
			// If the last point iterator positioning op might've skipped keys,
			// it's possible the file's range deletions are still relevant to
			// other levels. Return the largest boundary as a special ignorable
			// marker to avoid advancing to the next file.
			//
			// The sstable iterator cannot guarantee that keys were skipped. A
			// SeekGE that lands on a index separator k only knows that the
			// block at the index entry contains keys ≤ k. We can't know whether
			// there were actually keys between the seek key and the index
			// separator key. If the block is then excluded due to block
			// property filters, the iterator does not know whether keys were
			// actually skipped by the block's exclusion.
			//
			// Since MaybeFilteredKeys cannot guarantee that keys were skipped,
			// it's possible l.iterFile.Largest was already returned. Returning
			// l.iterFile.Largest again is a violation of the strict
			// monotonicity normally provided. The mergingIter's heap can
			// tolerate this repeat key and in this case will keep the level at
			// the top of the heap and immediately skip the entry, advancing to
			// the next file.
			if *l.rangeDelIterPtr != nil && l.filteredIter != nil &&
				l.filteredIter.MaybeFilteredKeys() {
				l.largestBoundary = &l.iterFile.Largest
				if l.boundaryContext != nil {
					l.boundaryContext.isIgnorableBoundaryKey = true
				}
				return l.largestBoundary, base.LazyValue{}
			}
		}

		// Current file was exhausted. Move to the next file.
		if l.loadFile(l.files.Next(), +1) == noFileLoaded {
			return nil, base.LazyValue{}
		}
	}
	return key, val
}

func (l *levelIter) skipEmptyFileBackward() (*InternalKey, base.LazyValue) {
	var key *InternalKey
	var val base.LazyValue
	// The first iteration of this loop starts with an already exhausted
	// l.iter. The reason for the exhaustion is either that we iterated to the
	// end of the sstable, or our iteration was terminated early due to the
	// presence of a lower-bound. If l.rangeDelIterPtr is non-nil, we may need
	// to pretend the iterator is not exhausted to allow for the merging to
	// finish consuming the l.rangeDelIterPtr before levelIter switches the
	// rangeDelIter from under it. This pretense is done by either generating
	// a synthetic boundary key or returning the smallest key of the file,
	// depending on the exhaustion reason.

	// Subsequent iterations will examine consecutive files such that the first
	// file that does not have an exhausted iterator causes the code to return
	// that key, else the behavior described above if there is a corresponding
	// rangeDelIterPtr.
	for ; key == nil; key, val = l.iter.Last() {
		if l.rangeDelIterPtr != nil {
			// We're being used as part of a mergingIter and we've exhausted the
			// current sstable. If a lower bound is present and the lower bound lies
			// within the current sstable, then we will have reached the lower bound
			// rather than the beginning of the sstable. We need to return a
			// synthetic boundary key so that mergingIter can use the range tombstone
			// iterator until the other levels have reached this boundary.
			//
			// It is safe to set the boundary key to the LowerBound user key
			// with the RANGEDEL sentinel since it is the smallest InternalKey
			// that is within the inclusive lower bound, and does not
			// represent a real key.
			if l.tableOpts.LowerBound != nil {
				if *l.rangeDelIterPtr != nil {
					l.syntheticBoundary.UserKey = l.tableOpts.LowerBound
					l.syntheticBoundary.Trailer = InternalKeyRangeDeleteSentinel
					l.smallestBoundary = &l.syntheticBoundary
					if l.boundaryContext != nil {
						l.boundaryContext.isSyntheticIterBoundsKey = true
					}
					return l.smallestBoundary, base.LazyValue{}
				}
				// Else there are no range deletions in this sstable. This
				// helps with performance when many levels are populated with
				// sstables and most don't have any actual keys within the
				// bounds.
				return nil, base.LazyValue{}
			}
			// If the boundary is a range deletion tombstone, return that key.
			if l.iterFile.SmallestPointKey.Kind() == InternalKeyKindRangeDelete {
				l.smallestBoundary = &l.iterFile.SmallestPointKey
				if l.boundaryContext != nil {
					l.boundaryContext.isIgnorableBoundaryKey = true
				}
				return l.smallestBoundary, base.LazyValue{}
			}
			// If the last point iterator positioning op skipped keys, it's
			// possible the file's range deletions are still relevant to other
			// levels. Return the smallest boundary as a special ignorable key
			// to avoid advancing to the next file.
			//
			// The sstable iterator cannot guarantee that keys were skipped.  A
			// SeekGE that lands on a index separator k only knows that the
			// block at the index entry contains keys ≤ k. We can't know whether
			// there were actually keys between the seek key and the index
			// separator key. If the block is then excluded due to block
			// property filters, the iterator does not know whether keys were
			// actually skipped by the block's exclusion.
			//
			// Since MaybeFilteredKeys cannot guarantee that keys were skipped,
			// it's possible l.iterFile.Smallest was already returned. Returning
			// l.iterFile.Smallest again is a violation of the strict
			// monotonicity normally provided. The mergingIter's heap can
			// tolerate this repeat key and in this case will keep the level at
			// the top of the heap and immediately skip the entry, advancing to
			// the next file.
			if *l.rangeDelIterPtr != nil && l.filteredIter != nil && l.filteredIter.MaybeFilteredKeys() {
				l.smallestBoundary = &l.iterFile.Smallest
				if l.boundaryContext != nil {
					l.boundaryContext.isIgnorableBoundaryKey = true
				}
				return l.smallestBoundary, base.LazyValue{}
			}
		}

		// Current file was exhausted. Move to the previous file.
		if l.loadFile(l.files.Prev(), -1) == noFileLoaded {
			return nil, base.LazyValue{}
		}
	}
	return key, val
}

func (l *levelIter) Error() error {
	if l.err != nil || l.iter == nil {
		return l.err
	}
	return l.iter.Error()
}

func (l *levelIter) Close() error {
	if l.iter != nil {
		l.err = l.iter.Close()
		l.iter = nil
	}
	if l.rangeDelIterPtr != nil {
		if t := l.rangeDelIterCopy; t != nil {
			l.err = firstError(l.err, t.Close())
		}
		*l.rangeDelIterPtr = nil
		l.rangeDelIterCopy = nil
	}
	return l.err
}

func (l *levelIter) SetBounds(lower, upper []byte) {
	l.lower = lower
	l.upper = upper

	if l.iter == nil {
		return
	}

	// Update tableOpts.{Lower,Upper}Bound in case the new boundaries fall within
	// the boundaries of the current table.
	if l.initTableBounds(l.iterFile) != 0 {
		// The table does not overlap the bounds. Close() will set levelIter.err if
		// an error occurs.
		_ = l.Close()
		return
	}

	l.iter.SetBounds(l.tableOpts.LowerBound, l.tableOpts.UpperBound)
}

func (l *levelIter) String() string {
	if l.iterFile != nil {
		return fmt.Sprintf("%s: fileNum=%s", l.level, l.iter.String())
	}
	return fmt.Sprintf("%s: fileNum=<nil>", l.level)
}

var _ internalIterator = &levelIter{}
