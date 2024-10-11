// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"context"
	"fmt"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/objiotracing"
)

type twoLevelIterator struct {
	singleLevelIterator
	// maybeFilteredKeysSingleLevel indicates whether the last iterator
	// positioning operation may have skipped any index blocks due to
	// block-property filters when positioning the top-level-index.
	maybeFilteredKeysTwoLevel bool
	topLevelIndex             blockIter
}

// twoLevelIterator implements the base.InternalIterator interface.
var _ base.InternalIterator = (*twoLevelIterator)(nil)

// loadIndex loads the index block at the current top level index position and
// leaves i.index unpositioned. If unsuccessful, it gets i.err to any error
// encountered, which may be nil if we have simply exhausted the entire table.
// This is used for two level indexes.
func (i *twoLevelIterator) loadIndex(dir int8) loadBlockResult {
	// Ensure the index data block iterators are invalidated even if loading of
	// the index fails.
	i.data.invalidate()
	i.index.invalidate()
	if !i.topLevelIndex.valid() {
		i.index.offset = 0
		i.index.restarts = 0
		return loadBlockFailed
	}
	v := i.topLevelIndex.value()
	bhp, err := decodeBlockHandleWithProperties(v.InPlaceValue())
	if err != nil {
		i.err = base.CorruptionErrorf("pebble/table: corrupt top level index entry")
		return loadBlockFailed
	}
	if i.bpfs != nil {
		intersects, err := i.bpfs.intersects(bhp.Props)
		if err != nil {
			i.err = errCorruptIndexEntry
			return loadBlockFailed
		}
		if intersects == blockMaybeExcluded {
			intersects = i.resolveMaybeExcluded(dir)
		}
		if intersects == blockExcluded {
			i.maybeFilteredKeysTwoLevel = true
			return loadBlockIrrelevant
		}
		// blockIntersects
	}
	ctx := objiotracing.WithBlockType(i.ctx, objiotracing.MetadataBlock)
	indexBlock, err := i.reader.readBlock(ctx, bhp.BlockHandle, nil /* transform */, nil /* readHandle */, i.stats, i.bufferPool)
	if err != nil {
		i.err = err
		return loadBlockFailed
	}
	if i.err = i.index.initHandle(i.cmp, indexBlock, i.reader.Properties.GlobalSeqNum, false); i.err == nil {
		return loadBlockOK
	}
	return loadBlockFailed
}

// resolveMaybeExcluded is invoked when the block-property filterer has found
// that an index block is excluded according to its properties but only if its
// bounds fall within the filter's current bounds. This function consults the
// apprioriate bound, depending on the iteration direction, and returns either
// `blockIntersects` or
// `blockMaybeExcluded`.
func (i *twoLevelIterator) resolveMaybeExcluded(dir int8) intersectsResult {
	// This iterator is configured with a bound-limited block property filter.
	// The bpf determined this entire index block could be excluded from
	// iteration based on the property encoded in the block handle. However, we
	// still need to determine if the index block is wholly contained within the
	// filter's key bounds.
	//
	// External guarantees ensure all its data blocks' keys are ≥ the filter's
	// lower bound during forward iteration, and that all its data blocks' keys
	// are < the filter's upper bound during backward iteration. We only need to
	// determine if the opposite bound is also met.
	//
	// The index separator in topLevelIndex.Key() provides an inclusive
	// upper-bound for the index block's keys, guaranteeing that all its keys
	// are ≤ topLevelIndex.Key(). For forward iteration, this is all we need.
	if dir > 0 {
		// Forward iteration.
		if i.bpfs.boundLimitedFilter.KeyIsWithinUpperBound(i.topLevelIndex.Key().UserKey) {
			return blockExcluded
		}
		return blockIntersects
	}

	// Reverse iteration.
	//
	// Because we're iterating in the reverse direction, we don't yet have
	// enough context available to determine if the block is wholly contained
	// within its bounds. This case arises only during backward iteration,
	// because of the way the index is structured.
	//
	// Consider a bound-limited bpf limited to the bounds [b,d), loading the
	// block with separator `c`. During reverse iteration, the guarantee that
	// all the block's keys are < `d` is externally provided, but no guarantee
	// is made on the bpf's lower bound. The separator `c` only provides an
	// inclusive upper bound on the block's keys, indicating that the
	// corresponding block handle points to a block containing only keys ≤ `c`.
	//
	// To establish a lower bound, we step the top-level index backwards to read
	// the previous block's separator, which provides an inclusive lower bound
	// on the original index block's keys. Afterwards, we step forward to
	// restore our top-level index position.
	if peekKey, _ := i.topLevelIndex.Prev(); peekKey == nil {
		// The original block points to the first index block of this table. If
		// we knew the lower bound for the entire table, it could provide a
		// lower bound, but the code refactoring necessary to read it doesn't
		// seem worth the payoff. We fall through to loading the block.
	} else if i.bpfs.boundLimitedFilter.KeyIsWithinLowerBound(peekKey.UserKey) {
		// The lower-bound on the original index block falls within the filter's
		// bounds, and we can skip the block (after restoring our current
		// top-level index position).
		_, _ = i.topLevelIndex.Next()
		return blockExcluded
	}
	_, _ = i.topLevelIndex.Next()
	return blockIntersects
}

// Note that lower, upper passed into init has nothing to do with virtual sstable
// bounds. If the virtualState passed in is not nil, then virtual sstable bounds
// will be enforced.
func (i *twoLevelIterator) init(
	ctx context.Context,
	r *Reader,
	v *virtualState,
	lower, upper []byte,
	filterer *BlockPropertiesFilterer,
	useFilter, hideObsoletePoints bool,
	stats *base.InternalIteratorStats,
	rp ReaderProvider,
	bufferPool *BufferPool,
) error {
	if r.err != nil {
		return r.err
	}
	topLevelIndexH, err := r.readIndex(ctx, stats)
	if err != nil {
		return err
	}
	if v != nil {
		i.vState = v
		// Note that upper is exclusive here.
		i.endKeyInclusive, lower, upper = v.constrainBounds(lower, upper, false /* endInclusive */)
	}

	i.ctx = ctx
	i.lower = lower
	i.upper = upper
	i.bpfs = filterer
	i.useFilter = useFilter
	i.reader = r
	i.cmp = r.Compare
	i.stats = stats
	i.hideObsoletePoints = hideObsoletePoints
	i.bufferPool = bufferPool
	err = i.topLevelIndex.initHandle(i.cmp, topLevelIndexH, r.Properties.GlobalSeqNum, false)
	if err != nil {
		// blockIter.Close releases topLevelIndexH and always returns a nil error
		_ = i.topLevelIndex.Close()
		return err
	}
	i.dataRH = objstorageprovider.UsePreallocatedReadHandle(ctx, r.readable, &i.dataRHPrealloc)
	if r.tableFormat >= TableFormatPebblev3 {
		if r.Properties.NumValueBlocks > 0 {
			i.vbReader = &valueBlockReader{
				ctx:    ctx,
				bpOpen: i,
				rp:     rp,
				vbih:   r.valueBIH,
				stats:  stats,
			}
			i.data.lazyValueHandling.vbr = i.vbReader
			i.vbRH = r.readable.NewReadHandle(ctx)
		}
		i.data.lazyValueHandling.hasValuePrefix = true
	}
	return nil
}

func (i *twoLevelIterator) String() string {
	if i.vState != nil {
		return i.vState.fileNum.String()
	}
	return i.reader.fileNum.String()
}

// MaybeFilteredKeys may be called when an iterator is exhausted to indicate
// whether or not the last positioning method may have skipped any keys due to
// block-property filters.
func (i *twoLevelIterator) MaybeFilteredKeys() bool {
	// While reading sstables with two-level indexes, knowledge of whether we've
	// filtered keys is tracked separately for each index level. The
	// seek-using-next optimizations have different criteria. We can only reset
	// maybeFilteredKeys back to false during a seek when NOT using the
	// fast-path that uses the current iterator position.
	//
	// If either level might have filtered keys to arrive at the current
	// iterator position, return MaybeFilteredKeys=true.
	return i.maybeFilteredKeysTwoLevel || i.maybeFilteredKeysSingleLevel
}

// SeekGE implements internalIterator.SeekGE, as documented in the pebble
// package. Note that SeekGE only checks the upper bound. It is up to the
// caller to ensure that key is greater than or equal to the lower bound.
func (i *twoLevelIterator) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	if i.vState != nil {
		// Callers of SeekGE don't know about virtual sstable bounds, so we may
		// have to internally restrict the bounds.
		//
		// TODO(bananabrick): We can optimize away this check for the level iter
		// if necessary.
		if i.cmp(key, i.lower) < 0 {
			key = i.lower
		}
	}

	err := i.err
	i.err = nil // clear cached iteration error

	// The twoLevelIterator could be already exhausted. Utilize that when
	// trySeekUsingNext is true. See the comment about data-exhausted, PGDE, and
	// bounds-exhausted near the top of the file.
	if flags.TrySeekUsingNext() &&
		(i.exhaustedBounds == +1 || (i.data.isDataInvalidated() && i.index.isDataInvalidated())) &&
		err == nil {
		// Already exhausted, so return nil.
		return nil, base.LazyValue{}
	}

	// SeekGE performs various step-instead-of-seeking optimizations: eg enabled
	// by trySeekUsingNext, or by monotonically increasing bounds (i.boundsCmp).
	// Care must be taken to ensure that when performing these optimizations and
	// the iterator becomes exhausted, i.maybeFilteredKeys is set appropriately.
	// Consider a previous SeekGE that filtered keys from k until the current
	// iterator position.
	//
	// If the previous SeekGE exhausted the iterator while seeking within the
	// two-level index, it's possible keys greater than or equal to the current
	// search key were filtered through skipped index blocks. We must not reuse
	// the position of the two-level index iterator without remembering the
	// previous value of maybeFilteredKeys.

	// We fall into the slow path if i.index.isDataInvalidated() even if the
	// top-level iterator is already positioned correctly and all other
	// conditions are met. An alternative structure could reuse topLevelIndex's
	// current position and reload the index block to which it points. Arguably,
	// an index block load is expensive and the index block may still be earlier
	// than the index block containing the sought key, resulting in a wasteful
	// block load.

	var dontSeekWithinSingleLevelIter bool
	if i.topLevelIndex.isDataInvalidated() || !i.topLevelIndex.valid() || i.index.isDataInvalidated() || err != nil ||
		(i.boundsCmp <= 0 && !flags.TrySeekUsingNext()) || i.cmp(key, i.topLevelIndex.Key().UserKey) > 0 {
		// Slow-path: need to position the topLevelIndex.

		// The previous exhausted state of singleLevelIterator is no longer
		// relevant, since we may be moving to a different index block.
		i.exhaustedBounds = 0
		i.maybeFilteredKeysTwoLevel = false
		flags = flags.DisableTrySeekUsingNext()
		var ikey *InternalKey
		if ikey, _ = i.topLevelIndex.SeekGE(key, flags); ikey == nil {
			i.data.invalidate()
			i.index.invalidate()
			return nil, base.LazyValue{}
		}

		result := i.loadIndex(+1)
		if result == loadBlockFailed {
			i.boundsCmp = 0
			return nil, base.LazyValue{}
		}
		if result == loadBlockIrrelevant {
			// Enforce the upper bound here since don't want to bother moving
			// to the next entry in the top level index if upper bound is
			// already exceeded. Note that the next entry starts with keys >=
			// ikey.UserKey since even though this is the block separator, the
			// same user key can span multiple index blocks. If upper is
			// exclusive we use >= below, else we use >.
			if i.upper != nil {
				cmp := i.cmp(ikey.UserKey, i.upper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
				}
			}
			// Fall through to skipForward.
			dontSeekWithinSingleLevelIter = true
			// Clear boundsCmp.
			//
			// In the typical cases where dontSeekWithinSingleLevelIter=false,
			// the singleLevelIterator.SeekGE call will clear boundsCmp.
			// However, in this case where dontSeekWithinSingleLevelIter=true,
			// we never seek on the single-level iterator. This call will fall
			// through to skipForward, which may improperly leave boundsCmp=+1
			// unless we clear it here.
			i.boundsCmp = 0
		}
	} else {
		// INVARIANT: err == nil.
		//
		// Else fast-path: There are two possible cases, from
		// (i.boundsCmp > 0 || flags.TrySeekUsingNext()):
		//
		// 1) The bounds have moved forward (i.boundsCmp > 0) and this SeekGE is
		// respecting the lower bound (guaranteed by Iterator). We know that the
		// iterator must already be positioned within or just outside the previous
		// bounds. Therefore, the topLevelIndex iter cannot be positioned at an
		// entry ahead of the seek position (though it can be positioned behind).
		// The !i.cmp(key, i.topLevelIndex.Key().UserKey) > 0 confirms that it is
		// not behind. Since it is not ahead and not behind it must be at the
		// right position.
		//
		// 2) This SeekGE will land on a key that is greater than the key we are
		// currently at (guaranteed by trySeekUsingNext), but since i.cmp(key,
		// i.topLevelIndex.Key().UserKey) <= 0, we are at the correct lower level
		// index block. No need to reset the state of singleLevelIterator.
		//
		// Note that cases 1 and 2 never overlap, and one of them must be true,
		// but we have some test code (TestIterRandomizedMaybeFilteredKeys) that
		// sets both to true, so we fix things here and then do an invariant
		// check.
		//
		// This invariant checking is important enough that we do not gate it
		// behind invariants.Enabled.
		if i.boundsCmp > 0 {
			// TODO(sumeer): fix TestIterRandomizedMaybeFilteredKeys so as to not
			// need this behavior.
			flags = flags.DisableTrySeekUsingNext()
		}
		if i.boundsCmp > 0 == flags.TrySeekUsingNext() {
			panic(fmt.Sprintf("inconsistency in optimization case 1 %t and case 2 %t",
				i.boundsCmp > 0, flags.TrySeekUsingNext()))
		}

		if !flags.TrySeekUsingNext() {
			// Case 1. Bounds have changed so the previous exhausted bounds state is
			// irrelevant.
			// WARNING-data-exhausted: this is safe to do only because the monotonic
			// bounds optimizations only work when !data-exhausted. If they also
			// worked with data-exhausted, we have made it unclear whether
			// data-exhausted is actually true. See the comment at the top of the
			// file.
			i.exhaustedBounds = 0
		}
		// Else flags.TrySeekUsingNext(). The i.exhaustedBounds is important to
		// preserve for singleLevelIterator, and twoLevelIterator.skipForward. See
		// bug https://github.com/cockroachdb/pebble/issues/2036.
	}

	if !dontSeekWithinSingleLevelIter {
		// Note that while trySeekUsingNext could be false here, singleLevelIterator
		// could do its own boundsCmp-based optimization to seek using next.
		if ikey, val := i.singleLevelIterator.SeekGE(key, flags); ikey != nil {
			return ikey, val
		}
	}
	return i.skipForward()
}

// SeekPrefixGE implements internalIterator.SeekPrefixGE, as documented in the
// pebble package. Note that SeekPrefixGE only checks the upper bound. It is up
// to the caller to ensure that key is greater than or equal to the lower bound.
func (i *twoLevelIterator) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	if i.vState != nil {
		// Callers of SeekGE don't know about virtual sstable bounds, so we may
		// have to internally restrict the bounds.
		//
		// TODO(bananabrick): We can optimize away this check for the level iter
		// if necessary.
		if i.cmp(key, i.lower) < 0 {
			key = i.lower
		}
	}

	// NOTE: prefix is only used for bloom filter checking and not later work in
	// this method. Hence, we can use the existing iterator position if the last
	// SeekPrefixGE did not fail bloom filter matching.

	err := i.err
	i.err = nil // clear cached iteration error

	// The twoLevelIterator could be already exhausted. Utilize that when
	// trySeekUsingNext is true. See the comment about data-exhausted, PGDE, and
	// bounds-exhausted near the top of the file.
	filterUsedAndDidNotMatch :=
		i.reader.tableFilter != nil && i.useFilter && !i.lastBloomFilterMatched
	if flags.TrySeekUsingNext() && !filterUsedAndDidNotMatch &&
		(i.exhaustedBounds == +1 || (i.data.isDataInvalidated() && i.index.isDataInvalidated())) &&
		err == nil {
		// Already exhausted, so return nil.
		return nil, base.LazyValue{}
	}

	// Check prefix bloom filter.
	if i.reader.tableFilter != nil && i.useFilter {
		if !i.lastBloomFilterMatched {
			// Iterator is not positioned based on last seek.
			flags = flags.DisableTrySeekUsingNext()
		}
		i.lastBloomFilterMatched = false
		var dataH bufferHandle
		dataH, i.err = i.reader.readFilter(i.ctx, i.stats)
		if i.err != nil {
			i.data.invalidate()
			return nil, base.LazyValue{}
		}
		mayContain := i.reader.tableFilter.mayContain(dataH.Get(), prefix)
		dataH.Release()
		if !mayContain {
			// This invalidation may not be necessary for correctness, and may
			// be a place to optimize later by reusing the already loaded
			// block. It was necessary in earlier versions of the code since
			// the caller was allowed to call Next when SeekPrefixGE returned
			// nil. This is no longer allowed.
			i.data.invalidate()
			return nil, base.LazyValue{}
		}
		i.lastBloomFilterMatched = true
	}

	// Bloom filter matches.

	// SeekPrefixGE performs various step-instead-of-seeking optimizations: eg
	// enabled by trySeekUsingNext, or by monotonically increasing bounds
	// (i.boundsCmp).  Care must be taken to ensure that when performing these
	// optimizations and the iterator becomes exhausted,
	// i.maybeFilteredKeysTwoLevel is set appropriately.  Consider a previous
	// SeekPrefixGE that filtered keys from k until the current iterator
	// position.
	//
	// If the previous SeekPrefixGE exhausted the iterator while seeking within
	// the two-level index, it's possible keys greater than or equal to the
	// current search key were filtered through skipped index blocks. We must
	// not reuse the position of the two-level index iterator without
	// remembering the previous value of maybeFilteredKeysTwoLevel.

	// We fall into the slow path if i.index.isDataInvalidated() even if the
	// top-level iterator is already positioned correctly and all other
	// conditions are met. An alternative structure could reuse topLevelIndex's
	// current position and reload the index block to which it points. Arguably,
	// an index block load is expensive and the index block may still be earlier
	// than the index block containing the sought key, resulting in a wasteful
	// block load.

	var dontSeekWithinSingleLevelIter bool
	if i.topLevelIndex.isDataInvalidated() || !i.topLevelIndex.valid() || i.index.isDataInvalidated() || err != nil ||
		(i.boundsCmp <= 0 && !flags.TrySeekUsingNext()) || i.cmp(key, i.topLevelIndex.Key().UserKey) > 0 {
		// Slow-path: need to position the topLevelIndex.

		// The previous exhausted state of singleLevelIterator is no longer
		// relevant, since we may be moving to a different index block.
		i.exhaustedBounds = 0
		i.maybeFilteredKeysTwoLevel = false
		flags = flags.DisableTrySeekUsingNext()
		var ikey *InternalKey
		if ikey, _ = i.topLevelIndex.SeekGE(key, flags); ikey == nil {
			i.data.invalidate()
			i.index.invalidate()
			return nil, base.LazyValue{}
		}

		result := i.loadIndex(+1)
		if result == loadBlockFailed {
			i.boundsCmp = 0
			return nil, base.LazyValue{}
		}
		if result == loadBlockIrrelevant {
			// Enforce the upper bound here since don't want to bother moving
			// to the next entry in the top level index if upper bound is
			// already exceeded. Note that the next entry starts with keys >=
			// ikey.UserKey since even though this is the block separator, the
			// same user key can span multiple index blocks. If upper is
			// exclusive we use >= below, else we use >.
			if i.upper != nil {
				cmp := i.cmp(ikey.UserKey, i.upper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
				}
			}
			// Fall through to skipForward.
			dontSeekWithinSingleLevelIter = true
			// Clear boundsCmp.
			//
			// In the typical cases where dontSeekWithinSingleLevelIter=false,
			// the singleLevelIterator.SeekPrefixGE call will clear boundsCmp.
			// However, in this case where dontSeekWithinSingleLevelIter=true,
			// we never seek on the single-level iterator. This call will fall
			// through to skipForward, which may improperly leave boundsCmp=+1
			// unless we clear it here.
			i.boundsCmp = 0
		}
	} else {
		// INVARIANT: err == nil.
		//
		// Else fast-path: There are two possible cases, from
		// (i.boundsCmp > 0 || flags.TrySeekUsingNext()):
		//
		// 1) The bounds have moved forward (i.boundsCmp > 0) and this
		// SeekPrefixGE is respecting the lower bound (guaranteed by Iterator). We
		// know that the iterator must already be positioned within or just
		// outside the previous bounds. Therefore, the topLevelIndex iter cannot
		// be positioned at an entry ahead of the seek position (though it can be
		// positioned behind). The !i.cmp(key, i.topLevelIndex.Key().UserKey) > 0
		// confirms that it is not behind. Since it is not ahead and not behind it
		// must be at the right position.
		//
		// 2) This SeekPrefixGE will land on a key that is greater than the key we
		// are currently at (guaranteed by trySeekUsingNext), but since i.cmp(key,
		// i.topLevelIndex.Key().UserKey) <= 0, we are at the correct lower level
		// index block. No need to reset the state of singleLevelIterator.
		//
		// Note that cases 1 and 2 never overlap, and one of them must be true.
		// This invariant checking is important enough that we do not gate it
		// behind invariants.Enabled.
		if i.boundsCmp > 0 == flags.TrySeekUsingNext() {
			panic(fmt.Sprintf("inconsistency in optimization case 1 %t and case 2 %t",
				i.boundsCmp > 0, flags.TrySeekUsingNext()))
		}

		if !flags.TrySeekUsingNext() {
			// Case 1. Bounds have changed so the previous exhausted bounds state is
			// irrelevant.
			// WARNING-data-exhausted: this is safe to do only because the monotonic
			// bounds optimizations only work when !data-exhausted. If they also
			// worked with data-exhausted, we have made it unclear whether
			// data-exhausted is actually true. See the comment at the top of the
			// file.
			i.exhaustedBounds = 0
		}
		// Else flags.TrySeekUsingNext(). The i.exhaustedBounds is important to
		// preserve for singleLevelIterator, and twoLevelIterator.skipForward. See
		// bug https://github.com/cockroachdb/pebble/issues/2036.
	}

	if !dontSeekWithinSingleLevelIter {
		if ikey, val := i.singleLevelIterator.seekPrefixGE(
			prefix, key, flags, false /* checkFilter */); ikey != nil {
			return ikey, val
		}
	}
	// NB: skipForward checks whether exhaustedBounds is already +1.
	return i.skipForward()
}

// virtualLast should only be called if i.vReader != nil and i.endKeyInclusive
// is true.
func (i *twoLevelIterator) virtualLast() (*InternalKey, base.LazyValue) {
	if i.vState == nil {
		panic("pebble: invalid call to virtualLast")
	}

	// Seek to the first internal key.
	ikey, _ := i.SeekGE(i.upper, base.SeekGEFlagsNone)
	if i.endKeyInclusive {
		// Let's say the virtual sstable upper bound is c#1, with the keys c#3, c#2,
		// c#1, d, e, ... in the sstable. So, the last key in the virtual sstable is
		// c#1. We can perform SeekGE(i.upper) and then keep nexting until we find
		// the last key with userkey == i.upper.
		//
		// TODO(bananabrick): Think about how to improve this. If many internal keys
		// with the same user key at the upper bound then this could be slow, but
		// maybe the odds of having many internal keys with the same user key at the
		// upper bound are low.
		for ikey != nil && i.cmp(ikey.UserKey, i.upper) == 0 {
			ikey, _ = i.Next()
		}
		return i.Prev()
	}
	// We seeked to the first key >= i.upper.
	return i.Prev()
}

// SeekLT implements internalIterator.SeekLT, as documented in the pebble
// package. Note that SeekLT only checks the lower bound. It is up to the
// caller to ensure that key is less than the upper bound.
func (i *twoLevelIterator) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*InternalKey, base.LazyValue) {
	if i.vState != nil {
		// Might have to fix upper bound since virtual sstable bounds are not
		// known to callers of SeekLT.
		//
		// TODO(bananabrick): We can optimize away this check for the level iter
		// if necessary.
		cmp := i.cmp(key, i.upper)
		// key == i.upper is fine. We'll do the right thing and return the
		// first internal key with user key < key.
		if cmp > 0 {
			return i.virtualLast()
		}
	}

	i.exhaustedBounds = 0
	i.err = nil // clear cached iteration error
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0

	var result loadBlockResult
	var ikey *InternalKey
	// NB: Unlike SeekGE, we don't have a fast-path here since we don't know
	// whether the topLevelIndex is positioned after the position that would
	// be returned by doing i.topLevelIndex.SeekGE(). To know this we would
	// need to know the index key preceding the current one.
	// NB: If a bound-limited block property filter is configured, it's
	// externally ensured that the filter is disabled (through returning
	// Intersects=false irrespective of the block props provided) during seeks.
	i.maybeFilteredKeysTwoLevel = false
	if ikey, _ = i.topLevelIndex.SeekGE(key, base.SeekGEFlagsNone); ikey == nil {
		if ikey, _ = i.topLevelIndex.Last(); ikey == nil {
			i.data.invalidate()
			i.index.invalidate()
			return nil, base.LazyValue{}
		}

		result = i.loadIndex(-1)
		if result == loadBlockFailed {
			return nil, base.LazyValue{}
		}
		if result == loadBlockOK {
			if ikey, val := i.singleLevelIterator.lastInternal(); ikey != nil {
				return i.maybeVerifyKey(ikey, val)
			}
			// Fall through to skipBackward since the singleLevelIterator did
			// not have any blocks that satisfy the block interval
			// constraints, or the lower bound was reached.
		}
		// Else loadBlockIrrelevant, so fall through.
	} else {
		result = i.loadIndex(-1)
		if result == loadBlockFailed {
			return nil, base.LazyValue{}
		}
		if result == loadBlockOK {
			if ikey, val := i.singleLevelIterator.SeekLT(key, flags); ikey != nil {
				return i.maybeVerifyKey(ikey, val)
			}
			// Fall through to skipBackward since the singleLevelIterator did
			// not have any blocks that satisfy the block interval
			// constraint, or the lower bound was reached.
		}
		// Else loadBlockIrrelevant, so fall through.
	}
	if result == loadBlockIrrelevant {
		// Enforce the lower bound here since don't want to bother moving to
		// the previous entry in the top level index if lower bound is already
		// exceeded. Note that the previous entry starts with keys <=
		// ikey.UserKey since even though this is the current block's
		// separator, the same user key can span multiple index blocks.
		if i.lower != nil && i.cmp(ikey.UserKey, i.lower) < 0 {
			i.exhaustedBounds = -1
		}
	}
	// NB: skipBackward checks whether exhaustedBounds is already -1.
	return i.skipBackward()
}

// First implements internalIterator.First, as documented in the pebble
// package. Note that First only checks the upper bound. It is up to the caller
// to ensure that key is greater than or equal to the lower bound (e.g. via a
// call to SeekGE(lower)).
func (i *twoLevelIterator) First() (*InternalKey, base.LazyValue) {
	// If the iterator was created on a virtual sstable, we will SeekGE to the
	// lower bound instead of using First, because First does not respect
	// bounds.
	if i.vState != nil {
		return i.SeekGE(i.lower, base.SeekGEFlagsNone)
	}

	if i.lower != nil {
		panic("twoLevelIterator.First() used despite lower bound")
	}
	i.exhaustedBounds = 0
	i.maybeFilteredKeysTwoLevel = false
	i.err = nil // clear cached iteration error
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0

	var ikey *InternalKey
	if ikey, _ = i.topLevelIndex.First(); ikey == nil {
		return nil, base.LazyValue{}
	}

	result := i.loadIndex(+1)
	if result == loadBlockFailed {
		return nil, base.LazyValue{}
	}
	if result == loadBlockOK {
		if ikey, val := i.singleLevelIterator.First(); ikey != nil {
			return ikey, val
		}
		// Else fall through to skipForward.
	} else {
		// result == loadBlockIrrelevant. Enforce the upper bound here since
		// don't want to bother moving to the next entry in the top level
		// index if upper bound is already exceeded. Note that the next entry
		// starts with keys >= ikey.UserKey since even though this is the
		// block separator, the same user key can span multiple index blocks.
		// If upper is exclusive we use >= below, else we use >.
		if i.upper != nil {
			cmp := i.cmp(ikey.UserKey, i.upper)
			if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
				i.exhaustedBounds = +1
			}
		}
	}
	// NB: skipForward checks whether exhaustedBounds is already +1.
	return i.skipForward()
}

// Last implements internalIterator.Last, as documented in the pebble
// package. Note that Last only checks the lower bound. It is up to the caller
// to ensure that key is less than the upper bound (e.g. via a call to
// SeekLT(upper))
func (i *twoLevelIterator) Last() (*InternalKey, base.LazyValue) {
	if i.vState != nil {
		if i.endKeyInclusive {
			return i.virtualLast()
		}
		return i.SeekLT(i.upper, base.SeekLTFlagsNone)
	}

	if i.upper != nil {
		panic("twoLevelIterator.Last() used despite upper bound")
	}
	i.exhaustedBounds = 0
	i.maybeFilteredKeysTwoLevel = false
	i.err = nil // clear cached iteration error
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0

	var ikey *InternalKey
	if ikey, _ = i.topLevelIndex.Last(); ikey == nil {
		return nil, base.LazyValue{}
	}

	result := i.loadIndex(-1)
	if result == loadBlockFailed {
		return nil, base.LazyValue{}
	}
	if result == loadBlockOK {
		if ikey, val := i.singleLevelIterator.Last(); ikey != nil {
			return ikey, val
		}
		// Else fall through to skipBackward.
	} else {
		// result == loadBlockIrrelevant. Enforce the lower bound here
		// since don't want to bother moving to the previous entry in the
		// top level index if lower bound is already exceeded. Note that
		// the previous entry starts with keys <= ikey.UserKey since even
		// though this is the current block's separator, the same user key
		// can span multiple index blocks.
		if i.lower != nil && i.cmp(ikey.UserKey, i.lower) < 0 {
			i.exhaustedBounds = -1
		}
	}
	// NB: skipBackward checks whether exhaustedBounds is already -1.
	return i.skipBackward()
}

// Next implements internalIterator.Next, as documented in the pebble
// package.
// Note: twoLevelCompactionIterator.Next mirrors the implementation of
// twoLevelIterator.Next due to performance. Keep the two in sync.
func (i *twoLevelIterator) Next() (*InternalKey, base.LazyValue) {
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0
	i.maybeFilteredKeysTwoLevel = false
	if i.err != nil {
		return nil, base.LazyValue{}
	}
	if key, val := i.singleLevelIterator.Next(); key != nil {
		return key, val
	}
	return i.skipForward()
}

// NextPrefix implements (base.InternalIterator).NextPrefix.
func (i *twoLevelIterator) NextPrefix(succKey []byte) (*InternalKey, base.LazyValue) {
	if i.exhaustedBounds == +1 {
		panic("Next called even though exhausted upper bound")
	}
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0
	i.maybeFilteredKeysTwoLevel = false
	if i.err != nil {
		return nil, base.LazyValue{}
	}
	if key, val := i.singleLevelIterator.NextPrefix(succKey); key != nil {
		return key, val
	}
	// key == nil
	if i.err != nil {
		return nil, base.LazyValue{}
	}

	// Did not find prefix in the existing second-level index block. This is the
	// slow-path where we seek the iterator.
	var ikey *InternalKey
	if ikey, _ = i.topLevelIndex.SeekGE(succKey, base.SeekGEFlagsNone); ikey == nil {
		i.data.invalidate()
		i.index.invalidate()
		return nil, base.LazyValue{}
	}
	result := i.loadIndex(+1)
	if result == loadBlockFailed {
		return nil, base.LazyValue{}
	}
	if result == loadBlockIrrelevant {
		// Enforce the upper bound here since don't want to bother moving to the
		// next entry in the top level index if upper bound is already exceeded.
		// Note that the next entry starts with keys >= ikey.UserKey since even
		// though this is the block separator, the same user key can span multiple
		// index blocks. If upper is exclusive we use >= below, else we use >.
		if i.upper != nil {
			cmp := i.cmp(ikey.UserKey, i.upper)
			if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
				i.exhaustedBounds = +1
			}
		}
	} else if key, val := i.singleLevelIterator.SeekGE(succKey, base.SeekGEFlagsNone); key != nil {
		return i.maybeVerifyKey(key, val)
	}
	return i.skipForward()
}

// Prev implements internalIterator.Prev, as documented in the pebble
// package.
func (i *twoLevelIterator) Prev() (*InternalKey, base.LazyValue) {
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0
	i.maybeFilteredKeysTwoLevel = false
	if i.err != nil {
		return nil, base.LazyValue{}
	}
	if key, val := i.singleLevelIterator.Prev(); key != nil {
		return key, val
	}
	return i.skipBackward()
}

func (i *twoLevelIterator) skipForward() (*InternalKey, base.LazyValue) {
	for {
		if i.err != nil || i.exhaustedBounds > 0 {
			return nil, base.LazyValue{}
		}
		i.exhaustedBounds = 0
		var ikey *InternalKey
		if ikey, _ = i.topLevelIndex.Next(); ikey == nil {
			i.data.invalidate()
			i.index.invalidate()
			return nil, base.LazyValue{}
		}
		result := i.loadIndex(+1)
		if result == loadBlockFailed {
			return nil, base.LazyValue{}
		}
		if result == loadBlockOK {
			if ikey, val := i.singleLevelIterator.firstInternal(); ikey != nil {
				return i.maybeVerifyKey(ikey, val)
			}
			// Next iteration will return if singleLevelIterator set
			// exhaustedBounds = +1.
		} else {
			// result == loadBlockIrrelevant. Enforce the upper bound here
			// since don't want to bother moving to the next entry in the top
			// level index if upper bound is already exceeded. Note that the
			// next entry starts with keys >= ikey.UserKey since even though
			// this is the block separator, the same user key can span
			// multiple index blocks. If upper is exclusive we use >=
			// below, else we use >.
			if i.upper != nil {
				cmp := i.cmp(ikey.UserKey, i.upper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
					// Next iteration will return.
				}
			}
		}
	}
}

func (i *twoLevelIterator) skipBackward() (*InternalKey, base.LazyValue) {
	for {
		if i.err != nil || i.exhaustedBounds < 0 {
			return nil, base.LazyValue{}
		}
		i.exhaustedBounds = 0
		var ikey *InternalKey
		if ikey, _ = i.topLevelIndex.Prev(); ikey == nil {
			i.data.invalidate()
			i.index.invalidate()
			return nil, base.LazyValue{}
		}
		result := i.loadIndex(-1)
		if result == loadBlockFailed {
			return nil, base.LazyValue{}
		}
		if result == loadBlockOK {
			if ikey, val := i.singleLevelIterator.lastInternal(); ikey != nil {
				return i.maybeVerifyKey(ikey, val)
			}
			// Next iteration will return if singleLevelIterator set
			// exhaustedBounds = -1.
		} else {
			// result == loadBlockIrrelevant. Enforce the lower bound here
			// since don't want to bother moving to the previous entry in the
			// top level index if lower bound is already exceeded. Note that
			// the previous entry starts with keys <= ikey.UserKey since even
			// though this is the current block's separator, the same user key
			// can span multiple index blocks.
			if i.lower != nil && i.cmp(ikey.UserKey, i.lower) < 0 {
				i.exhaustedBounds = -1
				// Next iteration will return.
			}
		}
	}
}

// Close implements internalIterator.Close, as documented in the pebble
// package.
func (i *twoLevelIterator) Close() error {
	var err error
	if i.closeHook != nil {
		err = firstError(err, i.closeHook(i))
	}
	err = firstError(err, i.data.Close())
	err = firstError(err, i.index.Close())
	err = firstError(err, i.topLevelIndex.Close())
	if i.dataRH != nil {
		err = firstError(err, i.dataRH.Close())
		i.dataRH = nil
	}
	err = firstError(err, i.err)
	if i.bpfs != nil {
		releaseBlockPropertiesFilterer(i.bpfs)
	}
	if i.vbReader != nil {
		i.vbReader.close()
	}
	if i.vbRH != nil {
		err = firstError(err, i.vbRH.Close())
		i.vbRH = nil
	}
	*i = twoLevelIterator{
		singleLevelIterator: i.singleLevelIterator.resetForReuse(),
		topLevelIndex:       i.topLevelIndex.resetForReuse(),
	}
	twoLevelIterPool.Put(i)
	return err
}

// Note: twoLevelCompactionIterator and compactionIterator are very similar but
// were separated due to performance.
type twoLevelCompactionIterator struct {
	*twoLevelIterator
	bytesIterated *uint64
	prevOffset    uint64
}

// twoLevelCompactionIterator implements the base.InternalIterator interface.
var _ base.InternalIterator = (*twoLevelCompactionIterator)(nil)

func (i *twoLevelCompactionIterator) Close() error {
	return i.twoLevelIterator.Close()
}

func (i *twoLevelCompactionIterator) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	panic("pebble: SeekGE unimplemented")
}

func (i *twoLevelCompactionIterator) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("pebble: SeekPrefixGE unimplemented")
}

func (i *twoLevelCompactionIterator) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*InternalKey, base.LazyValue) {
	panic("pebble: SeekLT unimplemented")
}

func (i *twoLevelCompactionIterator) First() (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	return i.skipForward(i.twoLevelIterator.First())
}

func (i *twoLevelCompactionIterator) Last() (*InternalKey, base.LazyValue) {
	panic("pebble: Last unimplemented")
}

// Note: twoLevelCompactionIterator.Next mirrors the implementation of
// twoLevelIterator.Next due to performance. Keep the two in sync.
func (i *twoLevelCompactionIterator) Next() (*InternalKey, base.LazyValue) {
	if i.err != nil {
		return nil, base.LazyValue{}
	}
	return i.skipForward(i.singleLevelIterator.Next())
}

func (i *twoLevelCompactionIterator) NextPrefix(succKey []byte) (*InternalKey, base.LazyValue) {
	panic("pebble: NextPrefix unimplemented")
}

func (i *twoLevelCompactionIterator) Prev() (*InternalKey, base.LazyValue) {
	panic("pebble: Prev unimplemented")
}

func (i *twoLevelCompactionIterator) String() string {
	if i.vState != nil {
		return i.vState.fileNum.String()
	}
	return i.reader.fileNum.String()
}

func (i *twoLevelCompactionIterator) skipForward(
	key *InternalKey, val base.LazyValue,
) (*InternalKey, base.LazyValue) {
	if key == nil {
		for {
			if key, _ := i.topLevelIndex.Next(); key == nil {
				break
			}
			result := i.loadIndex(+1)
			if result != loadBlockOK {
				if i.err != nil {
					break
				}
				switch result {
				case loadBlockFailed:
					// We checked that i.index was at a valid entry, so
					// loadBlockFailed could not have happened due to to i.index
					// being exhausted, and must be due to an error.
					panic("loadBlock should not have failed with no error")
				case loadBlockIrrelevant:
					panic("compactionIter should not be using block intervals for skipping")
				default:
					panic(fmt.Sprintf("unexpected case %d", result))
				}
			}
			// result == loadBlockOK
			if key, val = i.singleLevelIterator.First(); key != nil {
				break
			}
		}
	}

	curOffset := i.recordOffset()
	*i.bytesIterated += uint64(curOffset - i.prevOffset)
	i.prevOffset = curOffset

	if i.vState != nil && key != nil {
		cmp := i.cmp(key.UserKey, i.vState.upper.UserKey)
		if cmp > 0 || (i.vState.upper.IsExclusiveSentinel() && cmp == 0) {
			return nil, base.LazyValue{}
		}
	}

	return key, val
}
