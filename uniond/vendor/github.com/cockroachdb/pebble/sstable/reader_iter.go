// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"fmt"
	"os"
	"sync"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
)

// Iterator iterates over an entire table of data.
type Iterator interface {
	base.InternalIterator

	// NextPrefix implements (base.InternalIterator).NextPrefix.
	NextPrefix(succKey []byte) (*InternalKey, base.LazyValue)

	// MaybeFilteredKeys may be called when an iterator is exhausted to indicate
	// whether or not the last positioning method may have skipped any keys due
	// to block-property filters. This is used by the Pebble levelIter to
	// control when an iterator steps to the next sstable.
	//
	// MaybeFilteredKeys may always return false positives, that is it may
	// return true when no keys were filtered. It should only be called when the
	// iterator is exhausted. It must never return false negatives when the
	// iterator is exhausted.
	MaybeFilteredKeys() bool

	SetCloseHook(fn func(i Iterator) error)
}

// Iterator positioning optimizations and singleLevelIterator and
// twoLevelIterator:
//
// An iterator is absolute positioned using one of the Seek or First or Last
// calls. After absolute positioning, there can be relative positioning done
// by stepping using Prev or Next.
//
// We implement optimizations below where an absolute positioning call can in
// some cases use the current position to do less work. To understand these,
// we first define some terms. An iterator is bounds-exhausted if the bounds
// (upper of lower) have been reached. An iterator is data-exhausted if it has
// the reached the end of the data (forward or reverse) in the sstable. A
// singleLevelIterator only knows a local-data-exhausted property since when
// it is used as part of a twoLevelIterator, the twoLevelIterator can step to
// the next lower-level index block.
//
// The bounds-exhausted property is tracked by
// singleLevelIterator.exhaustedBounds being +1 (upper bound reached) or -1
// (lower bound reached). The same field is reused by twoLevelIterator. Either
// may notice the exhaustion of the bound and set it. Note that if
// singleLevelIterator sets this property, it is not a local property (since
// the bound has been reached regardless of whether this is in the context of
// the twoLevelIterator or not).
//
// The data-exhausted property is tracked in a more subtle manner. We define
// two predicates:
// - partial-local-data-exhausted (PLDE):
//   i.data.isDataInvalidated() || !i.data.valid()
// - partial-global-data-exhausted (PGDE):
//   i.index.isDataInvalidated() || !i.index.valid() || i.data.isDataInvalidated() ||
//   !i.data.valid()
//
// PLDE is defined for a singleLevelIterator. PGDE is defined for a
// twoLevelIterator. Oddly, in our code below the singleLevelIterator does not
// know when it is part of a twoLevelIterator so it does not know when its
// property is local or global.
//
// Now to define data-exhausted:
// - Prerequisite: we must know that the iterator has been positioned and
//   i.err is nil.
// - bounds-exhausted must not be true:
//   If bounds-exhausted is true, we have incomplete knowledge of
//   data-exhausted since PLDE or PGDE could be true because we could have
//   chosen not to load index block or data block and figured out that the
//   bound is exhausted (due to block property filters filtering out index and
//   data blocks and going past the bound on the top level index block). Note
//   that if we tried to separate out the BPF case from others we could
//   develop more knowledge here.
// - PGDE is true for twoLevelIterator. PLDE is true if it is a standalone
//   singleLevelIterator. !PLDE or !PGDE of course imply that data-exhausted
//   is not true.
//
// An implication of the above is that if we are going to somehow utilize
// knowledge of data-exhausted in an optimization, we must not forget the
// existing value of bounds-exhausted since by forgetting the latter we can
// erroneously think that data-exhausted is true. Bug #2036 was due to this
// forgetting.
//
// Now to the two categories of optimizations we currently have:
// - Monotonic bounds optimization that reuse prior iterator position when
//   doing seek: These only work with !data-exhausted. We could choose to make
//   these work with data-exhausted but have not bothered because in the
//   context of a DB if data-exhausted were true, the DB would move to the
//   next file in the level. Note that this behavior of moving to the next
//   file is not necessarily true for L0 files, so there could be some benefit
//   in the future in this optimization. See the WARNING-data-exhausted
//   comments if trying to optimize this in the future.
// - TrySeekUsingNext optimizations: these work regardless of exhaustion
//   state.
//
// Implementation detail: In the code PLDE only checks that
// i.data.isDataInvalidated(). This narrower check is safe, since this is a
// subset of the set expressed by the OR expression. Also, it is not a
// de-optimization since whenever we exhaust the iterator we explicitly call
// i.data.invalidate(). PGDE checks i.index.isDataInvalidated() &&
// i.data.isDataInvalidated(). Again, this narrower check is safe, and not a
// de-optimization since whenever we exhaust the iterator we explicitly call
// i.index.invalidate() and i.data.invalidate(). The && is questionable -- for
// now this is a bit of defensive code. We should seriously consider removing
// it, since defensive code suggests we are not confident about our invariants
// (and if we are not confident, we need more invariant assertions, not
// defensive code).
//
// TODO(sumeer): remove the aforementioned defensive code.

var singleLevelIterPool = sync.Pool{
	New: func() interface{} {
		i := &singleLevelIterator{}
		// Note: this is a no-op if invariants are disabled or race is enabled.
		invariants.SetFinalizer(i, checkSingleLevelIterator)
		return i
	},
}

var twoLevelIterPool = sync.Pool{
	New: func() interface{} {
		i := &twoLevelIterator{}
		// Note: this is a no-op if invariants are disabled or race is enabled.
		invariants.SetFinalizer(i, checkTwoLevelIterator)
		return i
	},
}

// TODO(jackson): rangedel fragmentBlockIters can't be pooled because of some
// code paths that double Close the iters. Fix the double close and pool the
// *fragmentBlockIter type directly.

var rangeKeyFragmentBlockIterPool = sync.Pool{
	New: func() interface{} {
		i := &rangeKeyFragmentBlockIter{}
		// Note: this is a no-op if invariants are disabled or race is enabled.
		invariants.SetFinalizer(i, checkRangeKeyFragmentBlockIterator)
		return i
	},
}

func checkSingleLevelIterator(obj interface{}) {
	i := obj.(*singleLevelIterator)
	if p := i.data.handle.Get(); p != nil {
		fmt.Fprintf(os.Stderr, "singleLevelIterator.data.handle is not nil: %p\n", p)
		os.Exit(1)
	}
	if p := i.index.handle.Get(); p != nil {
		fmt.Fprintf(os.Stderr, "singleLevelIterator.index.handle is not nil: %p\n", p)
		os.Exit(1)
	}
}

func checkTwoLevelIterator(obj interface{}) {
	i := obj.(*twoLevelIterator)
	if p := i.data.handle.Get(); p != nil {
		fmt.Fprintf(os.Stderr, "singleLevelIterator.data.handle is not nil: %p\n", p)
		os.Exit(1)
	}
	if p := i.index.handle.Get(); p != nil {
		fmt.Fprintf(os.Stderr, "singleLevelIterator.index.handle is not nil: %p\n", p)
		os.Exit(1)
	}
}

func checkRangeKeyFragmentBlockIterator(obj interface{}) {
	i := obj.(*rangeKeyFragmentBlockIter)
	if p := i.blockIter.handle.Get(); p != nil {
		fmt.Fprintf(os.Stderr, "fragmentBlockIter.blockIter.handle is not nil: %p\n", p)
		os.Exit(1)
	}
}

// compactionIterator is similar to Iterator but it increments the number of
// bytes that have been iterated through.
type compactionIterator struct {
	*singleLevelIterator
	bytesIterated *uint64
	prevOffset    uint64
}

// compactionIterator implements the base.InternalIterator interface.
var _ base.InternalIterator = (*compactionIterator)(nil)

func (i *compactionIterator) String() string {
	if i.vState != nil {
		return i.vState.fileNum.String()
	}
	return i.reader.fileNum.String()
}

func (i *compactionIterator) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	panic("pebble: SeekGE unimplemented")
}

func (i *compactionIterator) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("pebble: SeekPrefixGE unimplemented")
}

func (i *compactionIterator) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*InternalKey, base.LazyValue) {
	panic("pebble: SeekLT unimplemented")
}

func (i *compactionIterator) First() (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	return i.skipForward(i.singleLevelIterator.First())
}

func (i *compactionIterator) Last() (*InternalKey, base.LazyValue) {
	panic("pebble: Last unimplemented")
}

// Note: compactionIterator.Next mirrors the implementation of Iterator.Next
// due to performance. Keep the two in sync.
func (i *compactionIterator) Next() (*InternalKey, base.LazyValue) {
	if i.err != nil {
		return nil, base.LazyValue{}
	}
	return i.skipForward(i.data.Next())
}

func (i *compactionIterator) NextPrefix(succKey []byte) (*InternalKey, base.LazyValue) {
	panic("pebble: NextPrefix unimplemented")
}

func (i *compactionIterator) Prev() (*InternalKey, base.LazyValue) {
	panic("pebble: Prev unimplemented")
}

func (i *compactionIterator) skipForward(
	key *InternalKey, val base.LazyValue,
) (*InternalKey, base.LazyValue) {
	if key == nil {
		for {
			if key, _ := i.index.Next(); key == nil {
				break
			}
			result := i.loadBlock(+1)
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
			if key, val = i.data.First(); key != nil {
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
