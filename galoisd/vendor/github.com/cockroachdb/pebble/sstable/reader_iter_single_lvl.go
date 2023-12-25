// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"context"
	"fmt"
	"unsafe"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/objiotracing"
)

// singleLevelIterator iterates over an entire table of data. To seek for a given
// key, it first looks in the index for the block that contains that key, and then
// looks inside that block.
type singleLevelIterator struct {
	ctx context.Context
	cmp Compare
	// Global lower/upper bound for the iterator.
	lower []byte
	upper []byte
	bpfs  *BlockPropertiesFilterer
	// Per-block lower/upper bound. Nil if the bound does not apply to the block
	// because we determined the block lies completely within the bound.
	blockLower []byte
	blockUpper []byte
	reader     *Reader
	// vState will be set iff the iterator is constructed for virtual sstable
	// iteration.
	vState *virtualState
	// endKeyInclusive is set to force the iterator to treat the upper field as
	// inclusive while iterating instead of exclusive.
	endKeyInclusive bool
	index           blockIter
	data            blockIter
	dataRH          objstorage.ReadHandle
	dataRHPrealloc  objstorageprovider.PreallocatedReadHandle
	// dataBH refers to the last data block that the iterator considered
	// loading. It may not actually have loaded the block, due to an error or
	// because it was considered irrelevant.
	dataBH   BlockHandle
	vbReader *valueBlockReader
	// vbRH is the read handle for value blocks, which are in a different
	// part of the sstable than data blocks.
	vbRH         objstorage.ReadHandle
	vbRHPrealloc objstorageprovider.PreallocatedReadHandle
	err          error
	closeHook    func(i Iterator) error
	stats        *base.InternalIteratorStats
	bufferPool   *BufferPool

	// boundsCmp and positionedUsingLatestBounds are for optimizing iteration
	// that uses multiple adjacent bounds. The seek after setting a new bound
	// can use the fact that the iterator is either within the previous bounds
	// or exactly one key before or after the bounds. If the new bounds is
	// after/before the previous bounds, and we are already positioned at a
	// block that is relevant for the new bounds, we can try to first position
	// using Next/Prev (repeatedly) instead of doing a more expensive seek.
	//
	// When there are wide files at higher levels that match the bounds
	// but don't have any data for the bound, we will already be
	// positioned at the key beyond the bounds and won't need to do much
	// work -- given that most data is in L6, such files are likely to
	// dominate the performance of the mergingIter, and may be the main
	// benefit of this performance optimization (of course it also helps
	// when the file that has the data has successive seeks that stay in
	// the same block).
	//
	// Specifically, boundsCmp captures the relationship between the previous
	// and current bounds, if the iterator had been positioned after setting
	// the previous bounds. If it was not positioned, i.e., Seek/First/Last
	// were not called, we don't know where it is positioned and cannot
	// optimize.
	//
	// Example: Bounds moving forward, and iterator exhausted in forward direction.
	//      bounds = [f, h), ^ shows block iterator position
	//  file contents [ a  b  c  d  e  f  g  h  i  j  k ]
	//                                       ^
	//  new bounds = [j, k). Since positionedUsingLatestBounds=true, boundsCmp is
	//  set to +1. SeekGE(j) can use next (the optimization also requires that j
	//  is within the block, but that is not for correctness, but to limit the
	//  optimization to when it will actually be an optimization).
	//
	// Example: Bounds moving forward.
	//      bounds = [f, h), ^ shows block iterator position
	//  file contents [ a  b  c  d  e  f  g  h  i  j  k ]
	//                                 ^
	//  new bounds = [j, k). Since positionedUsingLatestBounds=true, boundsCmp is
	//  set to +1. SeekGE(j) can use next.
	//
	// Example: Bounds moving forward, but iterator not positioned using previous
	//  bounds.
	//      bounds = [f, h), ^ shows block iterator position
	//  file contents [ a  b  c  d  e  f  g  h  i  j  k ]
	//                                             ^
	//  new bounds = [i, j). Iterator is at j since it was never positioned using
	//  [f, h). So positionedUsingLatestBounds=false, and boundsCmp is set to 0.
	//  SeekGE(i) will not use next.
	//
	// Example: Bounds moving forward and sparse file
	//      bounds = [f, h), ^ shows block iterator position
	//  file contents [ a z ]
	//                    ^
	//  new bounds = [j, k). Since positionedUsingLatestBounds=true, boundsCmp is
	//  set to +1. SeekGE(j) notices that the iterator is already past j and does
	//  not need to do anything.
	//
	// Similar examples can be constructed for backward iteration.
	//
	// This notion of exactly one key before or after the bounds is not quite
	// true when block properties are used to ignore blocks. In that case we
	// can't stop precisely at the first block that is past the bounds since
	// we are using the index entries to enforce the bounds.
	//
	// e.g. 3 blocks with keys [b, c]  [f, g], [i, j, k] with index entries d,
	// h, l. And let the lower bound be k, and we are reverse iterating. If
	// the block [i, j, k] is ignored due to the block interval annotations we
	// do need to move the index to block [f, g] since the index entry for the
	// [i, j, k] block is l which is not less than the lower bound of k. So we
	// have passed the entries i, j.
	//
	// This behavior is harmless since the block property filters are fixed
	// for the lifetime of the iterator so i, j are irrelevant. In addition,
	// the current code will not load the [f, g] block, so the seek
	// optimization that attempts to use Next/Prev do not apply anyway.
	boundsCmp                   int
	positionedUsingLatestBounds bool

	// exhaustedBounds represents whether the iterator is exhausted for
	// iteration by reaching the upper or lower bound. +1 when exhausted
	// the upper bound, -1 when exhausted the lower bound, and 0 when
	// neither. exhaustedBounds is also used for the TrySeekUsingNext
	// optimization in twoLevelIterator and singleLevelIterator. Care should be
	// taken in setting this in twoLevelIterator before calling into
	// singleLevelIterator, given that these two iterators share this field.
	exhaustedBounds int8

	// maybeFilteredKeysSingleLevel indicates whether the last iterator
	// positioning operation may have skipped any data blocks due to
	// block-property filters when positioning the index.
	maybeFilteredKeysSingleLevel bool

	// useFilter specifies whether the filter block in this sstable, if present,
	// should be used for prefix seeks or not. In some cases it is beneficial
	// to skip a filter block even if it exists (eg. if probability of a match
	// is high).
	useFilter              bool
	lastBloomFilterMatched bool

	hideObsoletePoints bool
}

// singleLevelIterator implements the base.InternalIterator interface.
var _ base.InternalIterator = (*singleLevelIterator)(nil)

// init initializes a singleLevelIterator for reading from the table. It is
// synonmous with Reader.NewIter, but allows for reusing of the iterator
// between different Readers.
//
// Note that lower, upper passed into init has nothing to do with virtual sstable
// bounds. If the virtualState passed in is not nil, then virtual sstable bounds
// will be enforced.
func (i *singleLevelIterator) init(
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
	indexH, err := r.readIndex(ctx, stats)
	if err != nil {
		return err
	}
	if v != nil {
		i.vState = v
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
	err = i.index.initHandle(i.cmp, indexH, r.Properties.GlobalSeqNum, false)
	if err != nil {
		// blockIter.Close releases indexH and always returns a nil error
		_ = i.index.Close()
		return err
	}
	i.dataRH = objstorageprovider.UsePreallocatedReadHandle(ctx, r.readable, &i.dataRHPrealloc)
	if r.tableFormat >= TableFormatPebblev3 {
		if r.Properties.NumValueBlocks > 0 {
			// NB: we cannot avoid this ~248 byte allocation, since valueBlockReader
			// can outlive the singleLevelIterator due to be being embedded in a
			// LazyValue. This consumes ~2% in microbenchmark CPU profiles, but we
			// should only optimize this if it shows up as significant in end-to-end
			// CockroachDB benchmarks, since it is tricky to do so. One possibility
			// is that if many sstable iterators only get positioned at latest
			// versions of keys, and therefore never expose a LazyValue that is
			// separated to their callers, they can put this valueBlockReader into a
			// sync.Pool.
			i.vbReader = &valueBlockReader{
				ctx:    ctx,
				bpOpen: i,
				rp:     rp,
				vbih:   r.valueBIH,
				stats:  stats,
			}
			i.data.lazyValueHandling.vbr = i.vbReader
			i.vbRH = objstorageprovider.UsePreallocatedReadHandle(ctx, r.readable, &i.vbRHPrealloc)
		}
		i.data.lazyValueHandling.hasValuePrefix = true
	}
	return nil
}

// Helper function to check if keys returned from iterator are within global and virtual bounds.
func (i *singleLevelIterator) maybeVerifyKey(
	iKey *InternalKey, val base.LazyValue,
) (*InternalKey, base.LazyValue) {
	// maybeVerify key is only used for virtual sstable iterators.
	if invariants.Enabled && i.vState != nil && iKey != nil {
		key := iKey.UserKey

		uc, vuc := i.cmp(key, i.upper), i.cmp(key, i.vState.upper.UserKey)
		lc, vlc := i.cmp(key, i.lower), i.cmp(key, i.vState.lower.UserKey)

		if (i.vState.upper.IsExclusiveSentinel() && vuc == 0) || (!i.endKeyInclusive && uc == 0) || uc > 0 || vuc > 0 || lc < 0 || vlc < 0 {
			panic(fmt.Sprintf("key: %s out of bounds of singleLevelIterator", key))
		}
	}
	return iKey, val
}

// setupForCompaction sets up the singleLevelIterator for use with compactionIter.
// Currently, it skips readahead ramp-up. It should be called after init is called.
func (i *singleLevelIterator) setupForCompaction() {
	i.dataRH.SetupForCompaction()
	if i.vbRH != nil {
		i.vbRH.SetupForCompaction()
	}
}

func (i *singleLevelIterator) resetForReuse() singleLevelIterator {
	return singleLevelIterator{
		index: i.index.resetForReuse(),
		data:  i.data.resetForReuse(),
	}
}

func (i *singleLevelIterator) initBounds() {
	// Trim the iteration bounds for the current block. We don't have to check
	// the bounds on each iteration if the block is entirely contained within the
	// iteration bounds.
	i.blockLower = i.lower
	if i.blockLower != nil {
		key, _ := i.data.First()
		if key != nil && i.cmp(i.blockLower, key.UserKey) < 0 {
			// The lower-bound is less than the first key in the block. No need
			// to check the lower-bound again for this block.
			i.blockLower = nil
		}
	}
	i.blockUpper = i.upper
	if i.blockUpper != nil && i.cmp(i.blockUpper, i.index.Key().UserKey) > 0 {
		// The upper-bound is greater than the index key which itself is greater
		// than or equal to every key in the block. No need to check the
		// upper-bound again for this block. Even if blockUpper is inclusive
		// because of upper being inclusive, we can still safely set blockUpper
		// to nil here.
		//
		// TODO(bananabrick): We could also set blockUpper to nil for the >=
		// case, if blockUpper is inclusive.
		i.blockUpper = nil
	}
}

// Deterministic disabling of the bounds-based optimization that avoids seeking.
// Uses the iterator pointer, since we want diversity in iterator behavior for
// the same SetBounds call. Used for tests.
func disableBoundsOpt(bound []byte, ptr uintptr) bool {
	// Fibonacci hash https://probablydance.com/2018/06/16/fibonacci-hashing-the-optimization-that-the-world-forgot-or-a-better-alternative-to-integer-modulo/
	simpleHash := (11400714819323198485 * uint64(ptr)) >> 63
	return bound[len(bound)-1]&byte(1) == 0 && simpleHash == 0
}

// ensureBoundsOptDeterminism provides a facility for disabling of the bounds
// optimizations performed by disableBoundsOpt for tests that require
// deterministic iterator behavior. Some unit tests examine internal iterator
// state and require this behavior to be deterministic.
var ensureBoundsOptDeterminism bool

// SetBounds implements internalIterator.SetBounds, as documented in the pebble
// package. Note that the upper field is exclusive.
func (i *singleLevelIterator) SetBounds(lower, upper []byte) {
	i.boundsCmp = 0
	if i.vState != nil {
		// If the reader is constructed for a virtual sstable, then we must
		// constrain the bounds of the reader. For physical sstables, the bounds
		// can be wider than the actual sstable's bounds because we won't
		// accidentally expose additional keys as there are no additional keys.
		i.endKeyInclusive, lower, upper = i.vState.constrainBounds(
			lower, upper, false,
		)
	} else {
		// TODO(bananabrick): Figure out the logic here to enable the boundsCmp
		// optimization for virtual sstables.
		if i.positionedUsingLatestBounds {
			if i.upper != nil && lower != nil && i.cmp(i.upper, lower) <= 0 {
				i.boundsCmp = +1
				if invariants.Enabled && !ensureBoundsOptDeterminism &&
					disableBoundsOpt(lower, uintptr(unsafe.Pointer(i))) {
					i.boundsCmp = 0
				}
			} else if i.lower != nil && upper != nil && i.cmp(upper, i.lower) <= 0 {
				i.boundsCmp = -1
				if invariants.Enabled && !ensureBoundsOptDeterminism &&
					disableBoundsOpt(upper, uintptr(unsafe.Pointer(i))) {
					i.boundsCmp = 0
				}
			}
		}
	}

	i.positionedUsingLatestBounds = false
	i.lower = lower
	i.upper = upper
	i.blockLower = nil
	i.blockUpper = nil
}

// loadBlock loads the block at the current index position and leaves i.data
// unpositioned. If unsuccessful, it sets i.err to any error encountered, which
// may be nil if we have simply exhausted the entire table.
func (i *singleLevelIterator) loadBlock(dir int8) loadBlockResult {
	if !i.index.valid() {
		// Ensure the data block iterator is invalidated even if loading of the block
		// fails.
		i.data.invalidate()
		return loadBlockFailed
	}
	// Load the next block.
	v := i.index.value()
	bhp, err := decodeBlockHandleWithProperties(v.InPlaceValue())
	if i.dataBH == bhp.BlockHandle && i.data.valid() {
		// We're already at the data block we want to load. Reset bounds in case
		// they changed since the last seek, but don't reload the block from cache
		// or disk.
		//
		// It's safe to leave i.data in its original state here, as all callers to
		// loadBlock make an absolute positioning call (i.e. a seek, first, or last)
		// to `i.data` right after loadBlock returns loadBlockOK.
		i.initBounds()
		return loadBlockOK
	}
	// Ensure the data block iterator is invalidated even if loading of the block
	// fails.
	i.data.invalidate()
	i.dataBH = bhp.BlockHandle
	if err != nil {
		i.err = errCorruptIndexEntry
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
			i.maybeFilteredKeysSingleLevel = true
			return loadBlockIrrelevant
		}
		// blockIntersects
	}
	ctx := objiotracing.WithBlockType(i.ctx, objiotracing.DataBlock)
	block, err := i.reader.readBlock(ctx, i.dataBH, nil /* transform */, i.dataRH, i.stats, i.bufferPool)
	if err != nil {
		i.err = err
		return loadBlockFailed
	}
	i.err = i.data.initHandle(i.cmp, block, i.reader.Properties.GlobalSeqNum, i.hideObsoletePoints)
	if i.err != nil {
		// The block is partially loaded, and we don't want it to appear valid.
		i.data.invalidate()
		return loadBlockFailed
	}
	i.initBounds()
	return loadBlockOK
}

// readBlockForVBR implements the blockProviderWhenOpen interface for use by
// the valueBlockReader.
func (i *singleLevelIterator) readBlockForVBR(
	ctx context.Context, h BlockHandle, stats *base.InternalIteratorStats,
) (bufferHandle, error) {
	ctx = objiotracing.WithBlockType(ctx, objiotracing.ValueBlock)
	return i.reader.readBlock(ctx, h, nil, i.vbRH, stats, i.bufferPool)
}

// resolveMaybeExcluded is invoked when the block-property filterer has found
// that a block is excluded according to its properties but only if its bounds
// fall within the filter's current bounds.  This function consults the
// apprioriate bound, depending on the iteration direction, and returns either
// `blockIntersects` or `blockMaybeExcluded`.
func (i *singleLevelIterator) resolveMaybeExcluded(dir int8) intersectsResult {
	// TODO(jackson): We could first try comparing to top-level index block's
	// key, and if within bounds avoid per-data block key comparisons.

	// This iterator is configured with a bound-limited block property
	// filter. The bpf determined this block could be excluded from
	// iteration based on the property encoded in the block handle.
	// However, we still need to determine if the block is wholly
	// contained within the filter's key bounds.
	//
	// External guarantees ensure all the block's keys are ≥ the
	// filter's lower bound during forward iteration, and that all the
	// block's keys are < the filter's upper bound during backward
	// iteration. We only need to determine if the opposite bound is
	// also met.
	//
	// The index separator in index.Key() provides an inclusive
	// upper-bound for the data block's keys, guaranteeing that all its
	// keys are ≤ index.Key(). For forward iteration, this is all we
	// need.
	if dir > 0 {
		// Forward iteration.
		if i.bpfs.boundLimitedFilter.KeyIsWithinUpperBound(i.index.Key().UserKey) {
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
	// To establish a lower bound, we step the index backwards to read the
	// previous block's separator, which provides an inclusive lower bound on
	// the original block's keys. Afterwards, we step forward to restore our
	// index position.
	if peekKey, _ := i.index.Prev(); peekKey == nil {
		// The original block points to the first block of this index block. If
		// there's a two-level index, it could potentially provide a lower
		// bound, but the code refactoring necessary to read it doesn't seem
		// worth the payoff. We fall through to loading the block.
	} else if i.bpfs.boundLimitedFilter.KeyIsWithinLowerBound(peekKey.UserKey) {
		// The lower-bound on the original block falls within the filter's
		// bounds, and we can skip the block (after restoring our current index
		// position).
		_, _ = i.index.Next()
		return blockExcluded
	}
	_, _ = i.index.Next()
	return blockIntersects
}

func (i *singleLevelIterator) initBoundsForAlreadyLoadedBlock() {
	if i.data.getFirstUserKey() == nil {
		panic("initBoundsForAlreadyLoadedBlock must not be called on empty or corrupted block")
	}
	i.blockLower = i.lower
	if i.blockLower != nil {
		firstUserKey := i.data.getFirstUserKey()
		if firstUserKey != nil && i.cmp(i.blockLower, firstUserKey) < 0 {
			// The lower-bound is less than the first key in the block. No need
			// to check the lower-bound again for this block.
			i.blockLower = nil
		}
	}
	i.blockUpper = i.upper
	if i.blockUpper != nil && i.cmp(i.blockUpper, i.index.Key().UserKey) > 0 {
		// The upper-bound is greater than the index key which itself is greater
		// than or equal to every key in the block. No need to check the
		// upper-bound again for this block.
		i.blockUpper = nil
	}
}

// The number of times to call Next/Prev in a block before giving up and seeking.
// The value of 4 is arbitrary.
// TODO(sumeer): experiment with dynamic adjustment based on the history of
// seeks for a particular iterator.
const numStepsBeforeSeek = 4

func (i *singleLevelIterator) trySeekGEUsingNextWithinBlock(
	key []byte,
) (k *InternalKey, v base.LazyValue, done bool) {
	k, v = i.data.Key(), i.data.value()
	for j := 0; j < numStepsBeforeSeek; j++ {
		curKeyCmp := i.cmp(k.UserKey, key)
		if curKeyCmp >= 0 {
			if i.blockUpper != nil {
				cmp := i.cmp(k.UserKey, i.blockUpper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
					return nil, base.LazyValue{}, true
				}
			}
			return k, v, true
		}
		k, v = i.data.Next()
		if k == nil {
			break
		}
	}
	return k, v, false
}

func (i *singleLevelIterator) trySeekLTUsingPrevWithinBlock(
	key []byte,
) (k *InternalKey, v base.LazyValue, done bool) {
	k, v = i.data.Key(), i.data.value()
	for j := 0; j < numStepsBeforeSeek; j++ {
		curKeyCmp := i.cmp(k.UserKey, key)
		if curKeyCmp < 0 {
			if i.blockLower != nil && i.cmp(k.UserKey, i.blockLower) < 0 {
				i.exhaustedBounds = -1
				return nil, base.LazyValue{}, true
			}
			return k, v, true
		}
		k, v = i.data.Prev()
		if k == nil {
			break
		}
	}
	return k, v, false
}

func (i *singleLevelIterator) recordOffset() uint64 {
	offset := i.dataBH.Offset
	if i.data.valid() {
		// - i.dataBH.Length/len(i.data.data) is the compression ratio. If
		//   uncompressed, this is 1.
		// - i.data.nextOffset is the uncompressed position of the current record
		//   in the block.
		// - i.dataBH.Offset is the offset of the block in the sstable before
		//   decompression.
		offset += (uint64(i.data.nextOffset) * i.dataBH.Length) / uint64(len(i.data.data))
	} else {
		// Last entry in the block must increment bytes iterated by the size of the block trailer
		// and restart points.
		offset += i.dataBH.Length + blockTrailerLen
	}
	return offset
}

// SeekGE implements internalIterator.SeekGE, as documented in the pebble
// package. Note that SeekGE only checks the upper bound. It is up to the
// caller to ensure that key is greater than or equal to the lower bound.
func (i *singleLevelIterator) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	if i.vState != nil {
		// Callers of SeekGE don't know about virtual sstable bounds, so we may
		// have to internally restrict the bounds.
		//
		// TODO(bananabrick): We can optimize this check away for the level iter
		// if necessary.
		if i.cmp(key, i.lower) < 0 {
			key = i.lower
		}
	}

	if flags.TrySeekUsingNext() {
		// The i.exhaustedBounds comparison indicates that the upper bound was
		// reached. The i.data.isDataInvalidated() indicates that the sstable was
		// exhausted.
		if (i.exhaustedBounds == +1 || i.data.isDataInvalidated()) && i.err == nil {
			// Already exhausted, so return nil.
			return nil, base.LazyValue{}
		}
		if i.err != nil {
			// The current iterator position cannot be used.
			flags = flags.DisableTrySeekUsingNext()
		}
		// INVARIANT: flags.TrySeekUsingNext() => i.err == nil &&
		// !i.exhaustedBounds==+1 && !i.data.isDataInvalidated(). That is,
		// data-exhausted and bounds-exhausted, as defined earlier, are both
		// false. Ths makes it safe to clear out i.exhaustedBounds and i.err
		// before calling into seekGEHelper.
	}

	i.exhaustedBounds = 0
	i.err = nil // clear cached iteration error
	boundsCmp := i.boundsCmp
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0
	i.positionedUsingLatestBounds = true
	return i.seekGEHelper(key, boundsCmp, flags)
}

// seekGEHelper contains the common functionality for SeekGE and SeekPrefixGE.
func (i *singleLevelIterator) seekGEHelper(
	key []byte, boundsCmp int, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	// Invariant: trySeekUsingNext => !i.data.isDataInvalidated() && i.exhaustedBounds != +1

	// SeekGE performs various step-instead-of-seeking optimizations: eg enabled
	// by trySeekUsingNext, or by monotonically increasing bounds (i.boundsCmp).
	// Care must be taken to ensure that when performing these optimizations and
	// the iterator becomes exhausted, i.maybeFilteredKeys is set appropriately.
	// Consider a previous SeekGE that filtered keys from k until the current
	// iterator position.
	//
	// If the previous SeekGE exhausted the iterator, it's possible keys greater
	// than or equal to the current search key were filtered. We must not reuse
	// the current iterator position without remembering the previous value of
	// maybeFilteredKeys.

	var dontSeekWithinBlock bool
	if !i.data.isDataInvalidated() && !i.index.isDataInvalidated() && i.data.valid() && i.index.valid() &&
		boundsCmp > 0 && i.cmp(key, i.index.Key().UserKey) <= 0 {
		// Fast-path: The bounds have moved forward and this SeekGE is
		// respecting the lower bound (guaranteed by Iterator). We know that
		// the iterator must already be positioned within or just outside the
		// previous bounds. Therefore it cannot be positioned at a block (or
		// the position within that block) that is ahead of the seek position.
		// However it can be positioned at an earlier block. This fast-path to
		// use Next() on the block is only applied when we are already at the
		// block that the slow-path (the else-clause) would load -- this is
		// the motivation for the i.cmp(key, i.index.Key().UserKey) <= 0
		// predicate.
		i.initBoundsForAlreadyLoadedBlock()
		ikey, val, done := i.trySeekGEUsingNextWithinBlock(key)
		if done {
			return ikey, val
		}
		if ikey == nil {
			// Done with this block.
			dontSeekWithinBlock = true
		}
	} else {
		// Cannot use bounds monotonicity. But may be able to optimize if
		// caller claimed externally known invariant represented by
		// flags.TrySeekUsingNext().
		if flags.TrySeekUsingNext() {
			// seekPrefixGE or SeekGE has already ensured
			// !i.data.isDataInvalidated() && i.exhaustedBounds != +1
			currKey := i.data.Key()
			value := i.data.value()
			less := i.cmp(currKey.UserKey, key) < 0
			// We could be more sophisticated and confirm that the seek
			// position is within the current block before applying this
			// optimization. But there may be some benefit even if it is in
			// the next block, since we can avoid seeking i.index.
			for j := 0; less && j < numStepsBeforeSeek; j++ {
				currKey, value = i.Next()
				if currKey == nil {
					return nil, base.LazyValue{}
				}
				less = i.cmp(currKey.UserKey, key) < 0
			}
			if !less {
				if i.blockUpper != nil {
					cmp := i.cmp(currKey.UserKey, i.blockUpper)
					if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
						i.exhaustedBounds = +1
						return nil, base.LazyValue{}
					}
				}
				return currKey, value
			}
		}

		// Slow-path.
		// Since we're re-seeking the iterator, the previous value of
		// maybeFilteredKeysSingleLevel is irrelevant. If we filter out blocks
		// during seeking, loadBlock will set it to true.
		i.maybeFilteredKeysSingleLevel = false

		var ikey *InternalKey
		if ikey, _ = i.index.SeekGE(key, flags.DisableTrySeekUsingNext()); ikey == nil {
			// The target key is greater than any key in the index block.
			// Invalidate the block iterator so that a subsequent call to Prev()
			// will return the last key in the table.
			i.data.invalidate()
			return nil, base.LazyValue{}
		}
		result := i.loadBlock(+1)
		if result == loadBlockFailed {
			return nil, base.LazyValue{}
		}
		if result == loadBlockIrrelevant {
			// Enforce the upper bound here since don't want to bother moving
			// to the next block if upper bound is already exceeded. Note that
			// the next block starts with keys >= ikey.UserKey since even
			// though this is the block separator, the same user key can span
			// multiple blocks. If upper is exclusive we use >= below, else
			// we use >.
			if i.upper != nil {
				cmp := i.cmp(ikey.UserKey, i.upper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
					return nil, base.LazyValue{}
				}
			}
			// Want to skip to the next block.
			dontSeekWithinBlock = true
		}
	}
	if !dontSeekWithinBlock {
		if ikey, val := i.data.SeekGE(key, flags.DisableTrySeekUsingNext()); ikey != nil {
			if i.blockUpper != nil {
				cmp := i.cmp(ikey.UserKey, i.blockUpper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
					return nil, base.LazyValue{}
				}
			}
			return ikey, val
		}
	}
	return i.skipForward()
}

// SeekPrefixGE implements internalIterator.SeekPrefixGE, as documented in the
// pebble package. Note that SeekPrefixGE only checks the upper bound. It is up
// to the caller to ensure that key is greater than or equal to the lower bound.
func (i *singleLevelIterator) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	if i.vState != nil {
		// Callers of SeekPrefixGE aren't aware of virtual sstable bounds, so
		// we may have to internally restrict the bounds.
		//
		// TODO(bananabrick): We can optimize away this check for the level iter
		// if necessary.
		if i.cmp(key, i.lower) < 0 {
			key = i.lower
		}
	}
	return i.seekPrefixGE(prefix, key, flags, i.useFilter)
}

func (i *singleLevelIterator) seekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags, checkFilter bool,
) (k *InternalKey, value base.LazyValue) {
	// NOTE: prefix is only used for bloom filter checking and not later work in
	// this method. Hence, we can use the existing iterator position if the last
	// SeekPrefixGE did not fail bloom filter matching.

	err := i.err
	i.err = nil // clear cached iteration error
	if checkFilter && i.reader.tableFilter != nil {
		if !i.lastBloomFilterMatched {
			// Iterator is not positioned based on last seek.
			flags = flags.DisableTrySeekUsingNext()
		}
		i.lastBloomFilterMatched = false
		// Check prefix bloom filter.
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
	if flags.TrySeekUsingNext() {
		// The i.exhaustedBounds comparison indicates that the upper bound was
		// reached. The i.data.isDataInvalidated() indicates that the sstable was
		// exhausted.
		if (i.exhaustedBounds == +1 || i.data.isDataInvalidated()) && err == nil {
			// Already exhausted, so return nil.
			return nil, base.LazyValue{}
		}
		if err != nil {
			// The current iterator position cannot be used.
			flags = flags.DisableTrySeekUsingNext()
		}
		// INVARIANT: flags.TrySeekUsingNext() => err == nil &&
		// !i.exhaustedBounds==+1 && !i.data.isDataInvalidated(). That is,
		// data-exhausted and bounds-exhausted, as defined earlier, are both
		// false. Ths makes it safe to clear out i.exhaustedBounds and i.err
		// before calling into seekGEHelper.
	}
	// Bloom filter matches, or skipped, so this method will position the
	// iterator.
	i.exhaustedBounds = 0
	boundsCmp := i.boundsCmp
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0
	i.positionedUsingLatestBounds = true
	k, value = i.seekGEHelper(key, boundsCmp, flags)
	return i.maybeVerifyKey(k, value)
}

// virtualLast should only be called if i.vReader != nil.
func (i *singleLevelIterator) virtualLast() (*InternalKey, base.LazyValue) {
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
// caller to ensure that key is less than or equal to the upper bound.
func (i *singleLevelIterator) SeekLT(
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
			// Return the last key in the virtual sstable.
			return i.virtualLast()
		}
	}

	i.exhaustedBounds = 0
	i.err = nil // clear cached iteration error
	boundsCmp := i.boundsCmp
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0

	// Seeking operations perform various step-instead-of-seeking optimizations:
	// eg by considering monotonically increasing bounds (i.boundsCmp). Care
	// must be taken to ensure that when performing these optimizations and the
	// iterator becomes exhausted i.maybeFilteredKeysSingleLevel is set
	// appropriately.  Consider a previous SeekLT that filtered keys from k
	// until the current iterator position.
	//
	// If the previous SeekLT did exhausted the iterator, it's possible keys
	// less than the current search key were filtered. We must not reuse the
	// current iterator position without remembering the previous value of
	// maybeFilteredKeysSingleLevel.

	i.positionedUsingLatestBounds = true

	var dontSeekWithinBlock bool
	if !i.data.isDataInvalidated() && !i.index.isDataInvalidated() && i.data.valid() && i.index.valid() &&
		boundsCmp < 0 && i.cmp(i.data.getFirstUserKey(), key) < 0 {
		// Fast-path: The bounds have moved backward, and this SeekLT is
		// respecting the upper bound (guaranteed by Iterator). We know that
		// the iterator must already be positioned within or just outside the
		// previous bounds. Therefore it cannot be positioned at a block (or
		// the position within that block) that is behind the seek position.
		// However it can be positioned at a later block. This fast-path to
		// use Prev() on the block is only applied when we are already at the
		// block that can satisfy this seek -- this is the motivation for the
		// the i.cmp(i.data.firstKey.UserKey, key) < 0 predicate.
		i.initBoundsForAlreadyLoadedBlock()
		ikey, val, done := i.trySeekLTUsingPrevWithinBlock(key)
		if done {
			return ikey, val
		}
		if ikey == nil {
			// Done with this block.
			dontSeekWithinBlock = true
		}
	} else {
		// Slow-path.
		i.maybeFilteredKeysSingleLevel = false
		var ikey *InternalKey

		// NB: If a bound-limited block property filter is configured, it's
		// externally ensured that the filter is disabled (through returning
		// Intersects=false irrespective of the block props provided) during
		// seeks.
		if ikey, _ = i.index.SeekGE(key, base.SeekGEFlagsNone); ikey == nil {
			ikey, _ = i.index.Last()
			if ikey == nil {
				return nil, base.LazyValue{}
			}
		}
		// INVARIANT: ikey != nil.
		result := i.loadBlock(-1)
		if result == loadBlockFailed {
			return nil, base.LazyValue{}
		}
		if result == loadBlockIrrelevant {
			// Enforce the lower bound here since don't want to bother moving
			// to the previous block if lower bound is already exceeded. Note
			// that the previous block starts with keys <= ikey.UserKey since
			// even though this is the current block's separator, the same
			// user key can span multiple blocks.
			if i.lower != nil && i.cmp(ikey.UserKey, i.lower) < 0 {
				i.exhaustedBounds = -1
				return nil, base.LazyValue{}
			}
			// Want to skip to the previous block.
			dontSeekWithinBlock = true
		}
	}
	if !dontSeekWithinBlock {
		if ikey, val := i.data.SeekLT(key, flags); ikey != nil {
			if i.blockLower != nil && i.cmp(ikey.UserKey, i.blockLower) < 0 {
				i.exhaustedBounds = -1
				return nil, base.LazyValue{}
			}
			return ikey, val
		}
	}
	// The index contains separator keys which may lie between
	// user-keys. Consider the user-keys:
	//
	//   complete
	// ---- new block ---
	//   complexion
	//
	// If these two keys end one block and start the next, the index key may
	// be chosen as "compleu". The SeekGE in the index block will then point
	// us to the block containing "complexion". If this happens, we want the
	// last key from the previous data block.
	return i.maybeVerifyKey(i.skipBackward())
}

// First implements internalIterator.First, as documented in the pebble
// package. Note that First only checks the upper bound. It is up to the caller
// to ensure that key is greater than or equal to the lower bound (e.g. via a
// call to SeekGE(lower)).
func (i *singleLevelIterator) First() (*InternalKey, base.LazyValue) {
	// If the iterator was created on a virtual sstable, we will SeekGE to the
	// lower bound instead of using First, because First does not respect
	// bounds.
	if i.vState != nil {
		return i.SeekGE(i.lower, base.SeekGEFlagsNone)
	}

	if i.lower != nil {
		panic("singleLevelIterator.First() used despite lower bound")
	}
	i.positionedUsingLatestBounds = true
	i.maybeFilteredKeysSingleLevel = false

	return i.firstInternal()
}

// firstInternal is a helper used for absolute positioning in a single-level
// index file, or for positioning in the second-level index in a two-level
// index file. For the latter, one cannot make any claims about absolute
// positioning.
func (i *singleLevelIterator) firstInternal() (*InternalKey, base.LazyValue) {
	i.exhaustedBounds = 0
	i.err = nil // clear cached iteration error
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0

	var ikey *InternalKey
	if ikey, _ = i.index.First(); ikey == nil {
		i.data.invalidate()
		return nil, base.LazyValue{}
	}
	result := i.loadBlock(+1)
	if result == loadBlockFailed {
		return nil, base.LazyValue{}
	}
	if result == loadBlockOK {
		if ikey, val := i.data.First(); ikey != nil {
			if i.blockUpper != nil {
				cmp := i.cmp(ikey.UserKey, i.blockUpper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
					return nil, base.LazyValue{}
				}
			}
			return ikey, val
		}
		// Else fall through to skipForward.
	} else {
		// result == loadBlockIrrelevant. Enforce the upper bound here since
		// don't want to bother moving to the next block if upper bound is
		// already exceeded. Note that the next block starts with keys >=
		// ikey.UserKey since even though this is the block separator, the
		// same user key can span multiple blocks. If upper is exclusive we
		// use >= below, else we use >.
		if i.upper != nil {
			cmp := i.cmp(ikey.UserKey, i.upper)
			if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
				i.exhaustedBounds = +1
				return nil, base.LazyValue{}
			}
		}
		// Else fall through to skipForward.
	}

	return i.skipForward()
}

// Last implements internalIterator.Last, as documented in the pebble
// package. Note that Last only checks the lower bound. It is up to the caller
// to ensure that key is less than the upper bound (e.g. via a call to
// SeekLT(upper))
func (i *singleLevelIterator) Last() (*InternalKey, base.LazyValue) {
	if i.vState != nil {
		return i.virtualLast()
	}

	if i.upper != nil {
		panic("singleLevelIterator.Last() used despite upper bound")
	}
	i.positionedUsingLatestBounds = true
	i.maybeFilteredKeysSingleLevel = false
	return i.lastInternal()
}

// lastInternal is a helper used for absolute positioning in a single-level
// index file, or for positioning in the second-level index in a two-level
// index file. For the latter, one cannot make any claims about absolute
// positioning.
func (i *singleLevelIterator) lastInternal() (*InternalKey, base.LazyValue) {
	i.exhaustedBounds = 0
	i.err = nil // clear cached iteration error
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0

	var ikey *InternalKey
	if ikey, _ = i.index.Last(); ikey == nil {
		i.data.invalidate()
		return nil, base.LazyValue{}
	}
	result := i.loadBlock(-1)
	if result == loadBlockFailed {
		return nil, base.LazyValue{}
	}
	if result == loadBlockOK {
		if ikey, val := i.data.Last(); ikey != nil {
			if i.blockLower != nil && i.cmp(ikey.UserKey, i.blockLower) < 0 {
				i.exhaustedBounds = -1
				return nil, base.LazyValue{}
			}
			return ikey, val
		}
		// Else fall through to skipBackward.
	} else {
		// result == loadBlockIrrelevant. Enforce the lower bound here since
		// don't want to bother moving to the previous block if lower bound is
		// already exceeded. Note that the previous block starts with keys <=
		// key.UserKey since even though this is the current block's
		// separator, the same user key can span multiple blocks.
		if i.lower != nil && i.cmp(ikey.UserKey, i.lower) < 0 {
			i.exhaustedBounds = -1
			return nil, base.LazyValue{}
		}
	}

	return i.skipBackward()
}

// Next implements internalIterator.Next, as documented in the pebble
// package.
// Note: compactionIterator.Next mirrors the implementation of Iterator.Next
// due to performance. Keep the two in sync.
func (i *singleLevelIterator) Next() (*InternalKey, base.LazyValue) {
	if i.exhaustedBounds == +1 {
		panic("Next called even though exhausted upper bound")
	}
	i.exhaustedBounds = 0
	i.maybeFilteredKeysSingleLevel = false
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0

	if i.err != nil {
		return nil, base.LazyValue{}
	}
	if key, val := i.data.Next(); key != nil {
		if i.blockUpper != nil {
			cmp := i.cmp(key.UserKey, i.blockUpper)
			if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
				i.exhaustedBounds = +1
				return nil, base.LazyValue{}
			}
		}
		return key, val
	}
	return i.skipForward()
}

// NextPrefix implements (base.InternalIterator).NextPrefix.
func (i *singleLevelIterator) NextPrefix(succKey []byte) (*InternalKey, base.LazyValue) {
	if i.exhaustedBounds == +1 {
		panic("NextPrefix called even though exhausted upper bound")
	}
	i.exhaustedBounds = 0
	i.maybeFilteredKeysSingleLevel = false
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0
	if i.err != nil {
		return nil, base.LazyValue{}
	}
	if key, val := i.data.NextPrefix(succKey); key != nil {
		if i.blockUpper != nil {
			cmp := i.cmp(key.UserKey, i.blockUpper)
			if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
				i.exhaustedBounds = +1
				return nil, base.LazyValue{}
			}
		}
		return key, val
	}
	// Did not find prefix in the existing data block. This is the slow-path
	// where we effectively seek the iterator.
	var ikey *InternalKey
	// The key is likely to be in the next data block, so try one step.
	if ikey, _ = i.index.Next(); ikey == nil {
		// The target key is greater than any key in the index block.
		// Invalidate the block iterator so that a subsequent call to Prev()
		// will return the last key in the table.
		i.data.invalidate()
		return nil, base.LazyValue{}
	}
	if i.cmp(succKey, ikey.UserKey) > 0 {
		// Not in the next data block, so seek the index.
		if ikey, _ = i.index.SeekGE(succKey, base.SeekGEFlagsNone); ikey == nil {
			// The target key is greater than any key in the index block.
			// Invalidate the block iterator so that a subsequent call to Prev()
			// will return the last key in the table.
			i.data.invalidate()
			return nil, base.LazyValue{}
		}
	}
	result := i.loadBlock(+1)
	if result == loadBlockFailed {
		return nil, base.LazyValue{}
	}
	if result == loadBlockIrrelevant {
		// Enforce the upper bound here since don't want to bother moving
		// to the next block if upper bound is already exceeded. Note that
		// the next block starts with keys >= ikey.UserKey since even
		// though this is the block separator, the same user key can span
		// multiple blocks. If upper is exclusive we use >= below, else we use
		// >.
		if i.upper != nil {
			cmp := i.cmp(ikey.UserKey, i.upper)
			if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
				i.exhaustedBounds = +1
				return nil, base.LazyValue{}
			}
		}
	} else if key, val := i.data.SeekGE(succKey, base.SeekGEFlagsNone); key != nil {
		if i.blockUpper != nil {
			cmp := i.cmp(key.UserKey, i.blockUpper)
			if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
				i.exhaustedBounds = +1
				return nil, base.LazyValue{}
			}
		}
		return i.maybeVerifyKey(key, val)
	}

	return i.skipForward()
}

// Prev implements internalIterator.Prev, as documented in the pebble
// package.
func (i *singleLevelIterator) Prev() (*InternalKey, base.LazyValue) {
	if i.exhaustedBounds == -1 {
		panic("Prev called even though exhausted lower bound")
	}
	i.exhaustedBounds = 0
	i.maybeFilteredKeysSingleLevel = false
	// Seek optimization only applies until iterator is first positioned after SetBounds.
	i.boundsCmp = 0

	if i.err != nil {
		return nil, base.LazyValue{}
	}
	if key, val := i.data.Prev(); key != nil {
		if i.blockLower != nil && i.cmp(key.UserKey, i.blockLower) < 0 {
			i.exhaustedBounds = -1
			return nil, base.LazyValue{}
		}
		return key, val
	}
	return i.skipBackward()
}

func (i *singleLevelIterator) skipForward() (*InternalKey, base.LazyValue) {
	for {
		var key *InternalKey
		if key, _ = i.index.Next(); key == nil {
			i.data.invalidate()
			break
		}
		result := i.loadBlock(+1)
		if result != loadBlockOK {
			if i.err != nil {
				break
			}
			if result == loadBlockFailed {
				// We checked that i.index was at a valid entry, so
				// loadBlockFailed could not have happened due to to i.index
				// being exhausted, and must be due to an error.
				panic("loadBlock should not have failed with no error")
			}
			// result == loadBlockIrrelevant. Enforce the upper bound here
			// since don't want to bother moving to the next block if upper
			// bound is already exceeded. Note that the next block starts with
			// keys >= key.UserKey since even though this is the block
			// separator, the same user key can span multiple blocks. If upper
			// is exclusive we use >= below, else we use >.
			if i.upper != nil {
				cmp := i.cmp(key.UserKey, i.upper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
					return nil, base.LazyValue{}
				}
			}
			continue
		}
		if key, val := i.data.First(); key != nil {
			if i.blockUpper != nil {
				cmp := i.cmp(key.UserKey, i.blockUpper)
				if (!i.endKeyInclusive && cmp >= 0) || cmp > 0 {
					i.exhaustedBounds = +1
					return nil, base.LazyValue{}
				}
			}
			return i.maybeVerifyKey(key, val)
		}
	}
	return nil, base.LazyValue{}
}

func (i *singleLevelIterator) skipBackward() (*InternalKey, base.LazyValue) {
	for {
		var key *InternalKey
		if key, _ = i.index.Prev(); key == nil {
			i.data.invalidate()
			break
		}
		result := i.loadBlock(-1)
		if result != loadBlockOK {
			if i.err != nil {
				break
			}
			if result == loadBlockFailed {
				// We checked that i.index was at a valid entry, so
				// loadBlockFailed could not have happened due to to i.index
				// being exhausted, and must be due to an error.
				panic("loadBlock should not have failed with no error")
			}
			// result == loadBlockIrrelevant. Enforce the lower bound here
			// since don't want to bother moving to the previous block if lower
			// bound is already exceeded. Note that the previous block starts with
			// keys <= key.UserKey since even though this is the current block's
			// separator, the same user key can span multiple blocks.
			if i.lower != nil && i.cmp(key.UserKey, i.lower) < 0 {
				i.exhaustedBounds = -1
				return nil, base.LazyValue{}
			}
			continue
		}
		key, val := i.data.Last()
		if key == nil {
			return nil, base.LazyValue{}
		}
		if i.blockLower != nil && i.cmp(key.UserKey, i.blockLower) < 0 {
			i.exhaustedBounds = -1
			return nil, base.LazyValue{}
		}
		return i.maybeVerifyKey(key, val)
	}
	return nil, base.LazyValue{}
}

// Error implements internalIterator.Error, as documented in the pebble
// package.
func (i *singleLevelIterator) Error() error {
	if err := i.data.Error(); err != nil {
		return err
	}
	return i.err
}

// MaybeFilteredKeys may be called when an iterator is exhausted to indicate
// whether or not the last positioning method may have skipped any keys due to
// block-property filters.
func (i *singleLevelIterator) MaybeFilteredKeys() bool {
	return i.maybeFilteredKeysSingleLevel
}

// SetCloseHook sets a function that will be called when the iterator is
// closed.
func (i *singleLevelIterator) SetCloseHook(fn func(i Iterator) error) {
	i.closeHook = fn
}

func firstError(err0, err1 error) error {
	if err0 != nil {
		return err0
	}
	return err1
}

// Close implements internalIterator.Close, as documented in the pebble
// package.
func (i *singleLevelIterator) Close() error {
	var err error
	if i.closeHook != nil {
		err = firstError(err, i.closeHook(i))
	}
	err = firstError(err, i.data.Close())
	err = firstError(err, i.index.Close())
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
	*i = i.resetForReuse()
	singleLevelIterPool.Put(i)
	return err
}

func (i *singleLevelIterator) String() string {
	if i.vState != nil {
		return i.vState.fileNum.String()
	}
	return i.reader.fileNum.String()
}
