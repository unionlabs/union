// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/sstable"
)

// constructRangeKeyIter constructs the range-key iterator stack, populating
// i.rangeKey.rangeKeyIter with the resulting iterator.
func (i *Iterator) constructRangeKeyIter() {
	i.rangeKey.rangeKeyIter = i.rangeKey.iterConfig.Init(
		&i.comparer, i.seqNum, i.opts.LowerBound, i.opts.UpperBound,
		&i.hasPrefix, &i.prefixOrFullSeekKey, false /* internalKeys */, &i.rangeKey.rangeKeyBuffers.internal)

	// If there's an indexed batch with range keys, include it.
	if i.batch != nil {
		if i.batch.index == nil {
			i.rangeKey.iterConfig.AddLevel(newErrorKeyspanIter(ErrNotIndexed))
		} else {
			// Only include the batch's range key iterator if it has any keys.
			// NB: This can force reconstruction of the rangekey iterator stack
			// in SetOptions if subsequently range keys are added. See
			// SetOptions.
			if i.batch.countRangeKeys > 0 {
				i.batch.initRangeKeyIter(&i.opts, &i.batchRangeKeyIter, i.batchSeqNum)
				i.rangeKey.iterConfig.AddLevel(&i.batchRangeKeyIter)
			}
		}
	}

	// Next are the flushables: memtables and large batches.
	if i.readState != nil {
		for j := len(i.readState.memtables) - 1; j >= 0; j-- {
			mem := i.readState.memtables[j]
			// We only need to read from memtables which contain sequence numbers older
			// than seqNum.
			if logSeqNum := mem.logSeqNum; logSeqNum >= i.seqNum {
				continue
			}
			if rki := mem.newRangeKeyIter(&i.opts); rki != nil {
				i.rangeKey.iterConfig.AddLevel(rki)
			}
		}
	}

	current := i.version
	if current == nil {
		current = i.readState.current
	}
	// Next are the file levels: L0 sub-levels followed by lower levels.
	//
	// Add file-specific iterators for L0 files containing range keys. This is less
	// efficient than using levelIters for sublevels of L0 files containing
	// range keys, but range keys are expected to be sparse anyway, reducing the
	// cost benefit of maintaining a separate L0Sublevels instance for range key
	// files and then using it here.
	//
	// NB: We iterate L0's files in reverse order. They're sorted by
	// LargestSeqNum ascending, and we need to add them to the merging iterator
	// in LargestSeqNum descending to preserve the merging iterator's invariants
	// around Key Trailer order.
	iter := current.RangeKeyLevels[0].Iter()
	for f := iter.Last(); f != nil; f = iter.Prev() {
		spanIter, err := i.newIterRangeKey(f, i.opts.SpanIterOptions())
		if err != nil {
			i.rangeKey.iterConfig.AddLevel(&errorKeyspanIter{err: err})
			continue
		}
		i.rangeKey.iterConfig.AddLevel(spanIter)
	}

	// Add level iterators for the non-empty non-L0 levels.
	for level := 1; level < len(current.RangeKeyLevels); level++ {
		if current.RangeKeyLevels[level].Empty() {
			continue
		}
		li := i.rangeKey.iterConfig.NewLevelIter()
		spanIterOpts := i.opts.SpanIterOptions()
		li.Init(spanIterOpts, i.cmp, i.newIterRangeKey, current.RangeKeyLevels[level].Iter(),
			manifest.Level(level), manifest.KeyTypeRange)
		i.rangeKey.iterConfig.AddLevel(li)
	}
}

// Range key masking
//
// Pebble iterators may be configured such that range keys with suffixes mask
// point keys with lower suffixes. The intended use is implementing a MVCC
// delete range operation using range keys, when suffixes are MVCC timestamps.
//
// To enable masking, the user populates the IterOptions's RangeKeyMasking
// field. The Suffix field configures which range keys act as masks. The
// intended use is to hold a MVCC read timestamp. When implementing a MVCC
// delete range operation, only range keys that are visible at the read
// timestamp should be visible. If a range key has a suffix ≤
// RangeKeyMasking.Suffix, it acts as a mask.
//
// Range key masking is facilitated by the keyspan.InterleavingIter. The
// interleaving iterator interleaves range keys and point keys during combined
// iteration. During user iteration, the interleaving iterator is configured
// with a keyspan.SpanMask, implemented by the rangeKeyMasking struct below.
// The SpanMask interface defines two methods: SpanChanged and SkipPoint.
//
// SpanChanged is used to keep the current mask up-to-date. Whenever the point
// iterator has stepped into or out of the bounds of a range key, the
// interleaving iterator invokes SpanChanged passing the current covering range
// key. The below rangeKeyMasking implementation scans the range keys looking
// for the range key with the largest suffix that's still ≤ the suffix supplied
// to IterOptions.RangeKeyMasking.Suffix (the "read timestamp"). If it finds a
// range key that meets the condition, the range key should act as a mask. The
// span and the relevant range key's suffix are saved.
//
// The above ensures that `rangeKeyMasking.maskActiveSuffix` always contains the
// current masking suffix such that any point keys with lower suffixes should be
// skipped.
//
// There are two ways in which masked point keys are skipped.
//
//   1. Interleaving iterator SkipPoint
//
// Whenever the interleaving iterator encounters a point key that falls within
// the bounds of a range key, it invokes SkipPoint. The interleaving iterator
// guarantees that the SpanChanged method described above has already been
// invoked with the covering range key. The below rangeKeyMasking implementation
// of SkipPoint splits the key into prefix and suffix, compares the suffix to
// the `maskActiveSuffix` updated by SpanChanged and returns true if
// suffix(point) < maskActiveSuffix.
//
// The SkipPoint logic is sufficient to ensure that the Pebble iterator filters
// out all masked point keys. However, it requires the iterator read each masked
// point key. For broad range keys that mask many points, this may be expensive.
//
//   2. Block property filter
//
// For more efficient handling of braad range keys that mask many points, the
// IterOptions.RangeKeyMasking field has an optional Filter option. This Filter
// field takes a superset of the block-property filter interface, adding a
// method to dynamically configure the filter's filtering criteria.
//
// To make use of the Filter option, the user is required to define and
// configure a block-property collector that collects a property containing at
// least the maximum suffix of a key within a block.
//
// When the SpanChanged method described above is invoked, rangeKeyMasking also
// reconfigures the user-provided filter. It invokes a SetSuffix method,
// providing the `maskActiveSuffix`, requesting that from now on the
// block-property filter return Intersects()=false for any properties indicating
// that a block contains exclusively keys with suffixes greater than the
// provided suffix.
//
// Note that unlike other block-property filters, the filter used for masking
// must not apply across the entire keyspace. It must only filter blocks that
// lie within the bounds of the range key that set the mask suffix. To
// accommodate this, rangeKeyMasking implements a special interface:
// sstable.BoundLimitedBlockPropertyFilter. This interface extends the block
// property filter interface with two new methods: KeyIsWithinLowerBound and
// KeyIsWithinUpperBound. The rangeKeyMasking type wraps the user-provided block
// property filter, implementing these two methods and overriding Intersects to
// always return true if there is no active mask.
//
// The logic to ensure that a mask block-property filter is only applied within
// the bounds of the masking range key is subtle. The interleaving iterator
// guarantees that it never invokes SpanChanged until the point iterator is
// positioned within the range key. During forward iteration, this guarantees
// that any block that a sstable reader might attempt to load contains only keys
// greater than or equal to the range key's lower bound. During backward
// iteration, it provides the analagous guarantee on the range key's upper
// bound.
//
// The above ensures that an sstable reader only needs to verify that a block
// that it skips meets the opposite bound. This is where the
// KeyIsWithinLowerBound and KeyIsWithinUpperBound methods are used. When an
// sstable iterator is configured with a BoundLimitedBlockPropertyFilter, it
// checks for intersection with the block-property filter before every block
// load, like ordinary block-property filters. However, if the bound-limited
// block property filter indicates that it does NOT intersect, the filter's
// relevant KeyIsWithin{Lower,Upper}Bound method is queried, using a block
// index separator as the bound. If the method indicates that the provided index
// separator does not fall within the range key bounds, the no-intersection
// result is ignored, and the block is read.

type rangeKeyMasking struct {
	cmp    base.Compare
	split  base.Split
	filter BlockPropertyFilterMask
	// maskActiveSuffix holds the suffix of a range key currently acting as a
	// mask, hiding point keys with suffixes greater than it. maskActiveSuffix
	// is only ever non-nil if IterOptions.RangeKeyMasking.Suffix is non-nil.
	// maskActiveSuffix is updated whenever the iterator passes over a new range
	// key. The maskActiveSuffix should only be used if maskSpan is non-nil.
	//
	// See SpanChanged.
	maskActiveSuffix []byte
	// maskSpan holds the span from which the active mask suffix was extracted.
	// The span is used for bounds comparisons, to ensure that a range-key mask
	// is not applied beyond the bounds of the range key.
	maskSpan *keyspan.Span
	parent   *Iterator
}

func (m *rangeKeyMasking) init(parent *Iterator, cmp base.Compare, split base.Split) {
	m.cmp = cmp
	m.split = split
	if parent.opts.RangeKeyMasking.Filter != nil {
		m.filter = parent.opts.RangeKeyMasking.Filter()
	}
	m.parent = parent
}

// SpanChanged implements the keyspan.SpanMask interface, used during range key
// iteration.
func (m *rangeKeyMasking) SpanChanged(s *keyspan.Span) {
	if s == nil && m.maskSpan == nil {
		return
	}
	m.maskSpan = nil
	m.maskActiveSuffix = m.maskActiveSuffix[:0]

	// Find the smallest suffix of a range key contained within the Span,
	// excluding suffixes less than m.opts.RangeKeyMasking.Suffix.
	if s != nil {
		m.parent.rangeKey.stale = true
		if m.parent.opts.RangeKeyMasking.Suffix != nil {
			for j := range s.Keys {
				if s.Keys[j].Suffix == nil {
					continue
				}
				if m.cmp(s.Keys[j].Suffix, m.parent.opts.RangeKeyMasking.Suffix) < 0 {
					continue
				}
				if len(m.maskActiveSuffix) == 0 || m.cmp(m.maskActiveSuffix, s.Keys[j].Suffix) > 0 {
					m.maskSpan = s
					m.maskActiveSuffix = append(m.maskActiveSuffix[:0], s.Keys[j].Suffix...)
				}
			}
		}
	}

	if m.maskSpan != nil && m.parent.opts.RangeKeyMasking.Filter != nil {
		// Update the  block-property filter to filter point keys with suffixes
		// greater than m.maskActiveSuffix.
		err := m.filter.SetSuffix(m.maskActiveSuffix)
		if err != nil {
			m.parent.err = err
		}
	}
	// If no span is active, we leave the inner block-property filter configured
	// with its existing suffix. That's okay, because Intersects calls are first
	// evaluated by iteratorRangeKeyState.Intersects, which considers all blocks
	// as intersecting if there's no active mask.
}

// SkipPoint implements the keyspan.SpanMask interface, used during range key
// iteration. Whenever a point key is covered by a non-empty Span, the
// interleaving iterator invokes SkipPoint. This function is responsible for
// performing range key masking.
//
// If a non-nil IterOptions.RangeKeyMasking.Suffix is set, range key masking is
// enabled. Masking hides point keys, transparently skipping over the keys.
// Whether or not a point key is masked is determined by comparing the point
// key's suffix, the overlapping span's keys' suffixes, and the user-configured
// IterOption's RangeKeyMasking.Suffix. When configured with a masking threshold
// _t_, and there exists a span with suffix _r_ covering a point key with suffix
// _p_, and
//
//	_t_ ≤ _r_ < _p_
//
// then the point key is elided. Consider the following rendering, where using
// integer suffixes with higher integers sort before suffixes with lower
// integers, (for example @7 ≤ @6 < @5):
//
//	     ^
//	  @9 |        •―――――――――――――――○ [e,m)@9
//	s  8 |                      • l@8
//	u  7 |------------------------------------ @7 RangeKeyMasking.Suffix
//	f  6 |      [h,q)@6 •―――――――――――――――――○            (threshold)
//	f  5 |              • h@5
//	f  4 |                          • n@4
//	i  3 |          •―――――――――――○ [f,l)@3
//	x  2 |  • b@2
//	   1 |
//	   0 |___________________________________
//	      a b c d e f g h i j k l m n o p q
//
// An iterator scanning the entire keyspace with the masking threshold set to @7
// will observe point keys b@2 and l@8. The span keys [h,q)@6 and [f,l)@3 serve
// as masks, because cmp(@6,@7) ≥ 0 and cmp(@3,@7) ≥ 0. The span key [e,m)@9
// does not serve as a mask, because cmp(@9,@7) < 0.
//
// Although point l@8 falls within the user key bounds of [e,m)@9, [e,m)@9 is
// non-masking due to its suffix. The point key l@8 also falls within the user
// key bounds of [h,q)@6, but since cmp(@6,@8) ≥ 0, l@8 is unmasked.
//
// Invariant: The userKey is within the user key bounds of the span most
// recently provided to `SpanChanged`.
func (m *rangeKeyMasking) SkipPoint(userKey []byte) bool {
	m.parent.stats.RangeKeyStats.ContainedPoints++
	if m.maskSpan == nil {
		// No range key is currently acting as a mask, so don't skip.
		return false
	}
	// Range key masking is enabled and the current span includes a range key
	// that is being used as a mask. (NB: SpanChanged already verified that the
	// range key's suffix is ≥ RangeKeyMasking.Suffix).
	//
	// This point key falls within the bounds of the range key (guaranteed by
	// the InterleavingIter). Skip the point key if the range key's suffix is
	// greater than the point key's suffix.
	pointSuffix := userKey[m.split(userKey):]
	if len(pointSuffix) > 0 && m.cmp(m.maskActiveSuffix, pointSuffix) < 0 {
		m.parent.stats.RangeKeyStats.SkippedPoints++
		return true
	}
	return false
}

// The iteratorRangeKeyState type implements the sstable package's
// BoundLimitedBlockPropertyFilter interface in order to use block property
// filters for range key masking. The iteratorRangeKeyState implementation wraps
// the block-property filter provided in Options.RangeKeyMasking.Filter.
//
// Using a block-property filter for range-key masking requires limiting the
// filter's effect to the bounds of the range key currently acting as a mask.
// Consider the range key [a,m)@10, and an iterator positioned just before the
// below block, bounded by index separators `c` and `z`:
//
//	          c                          z
//	   x      |  c@9 c@5 c@1 d@7 e@4 y@4 | ...
//	iter pos
//
// The next block cannot be skipped, despite the range key suffix @10 is greater
// than all the block's keys' suffixes, because it contains a key (y@4) outside
// the bounds of the range key.
//
// This extended BoundLimitedBlockPropertyFilter interface adds two new methods,
// KeyIsWithinLowerBound and KeyIsWithinUpperBound, for testing whether a
// particular block is within bounds.
//
// The iteratorRangeKeyState implements these new methods by first checking if
// the iterator is currently positioned within a range key. If not, the provided
// key is considered out-of-bounds. If the iterator is positioned within a range
// key, it compares the corresponding range key bound.
var _ sstable.BoundLimitedBlockPropertyFilter = (*rangeKeyMasking)(nil)

// Name implements the limitedBlockPropertyFilter interface defined in the
// sstable package by passing through to the user-defined block property filter.
func (m *rangeKeyMasking) Name() string {
	return m.filter.Name()
}

// Intersects implements the limitedBlockPropertyFilter interface defined in the
// sstable package by passing the intersection decision to the user-provided
// block property filter only if a range key is covering the current iterator
// position.
func (m *rangeKeyMasking) Intersects(prop []byte) (bool, error) {
	if m.maskSpan == nil {
		// No span is actively masking.
		return true, nil
	}
	return m.filter.Intersects(prop)
}

// KeyIsWithinLowerBound implements the limitedBlockPropertyFilter interface
// defined in the sstable package. It's used to restrict the masking block
// property filter to only applying within the bounds of the active range key.
func (m *rangeKeyMasking) KeyIsWithinLowerBound(key []byte) bool {
	// Invariant: m.maskSpan != nil
	//
	// The provided `key` is an inclusive lower bound of the block we're
	// considering skipping.
	return m.cmp(m.maskSpan.Start, key) <= 0
}

// KeyIsWithinUpperBound implements the limitedBlockPropertyFilter interface
// defined in the sstable package. It's used to restrict the masking block
// property filter to only applying within the bounds of the active range key.
func (m *rangeKeyMasking) KeyIsWithinUpperBound(key []byte) bool {
	// Invariant: m.maskSpan != nil
	//
	// The provided `key` is an *inclusive* upper bound of the block we're
	// considering skipping, so the range key's end must be strictly greater
	// than the block bound for the block to be within bounds.
	return m.cmp(m.maskSpan.End, key) > 0
}

// lazyCombinedIter implements the internalIterator interface, wrapping a
// pointIter. It requires the pointIter's the levelIters be configured with
// pointers to its combinedIterState. When the levelIter observes a file
// containing a range key, the lazyCombinedIter constructs the combined
// range+point key iterator stack and switches to it.
type lazyCombinedIter struct {
	// parent holds a pointer to the root *pebble.Iterator containing this
	// iterator. It's used to mutate the internalIterator in use when switching
	// to combined iteration.
	parent            *Iterator
	pointIter         internalIterator
	combinedIterState combinedIterState
}

// combinedIterState encapsulates the current state of combined iteration.
// Various low-level iterators (mergingIter, leveliter) hold pointers to the
// *pebble.Iterator's combinedIterState. This allows them to check whether or
// not they must monitor for files containing range keys (!initialized), or not.
//
// When !initialized, low-level iterators watch for files containing range keys.
// When one is discovered, they set triggered=true and key to the smallest
// (forward direction) or largest (reverse direction) range key that's been
// observed.
type combinedIterState struct {
	// key holds the smallest (forward direction) or largest (backward
	// direction) user key from a range key bound discovered during the iterator
	// operation that triggered the switch to combined iteration.
	//
	// Slices stored here must be stable. This is possible because callers pass
	// a Smallest/Largest bound from a fileMetadata, which are immutable. A key
	// slice's bytes must not be overwritten.
	key         []byte
	triggered   bool
	initialized bool
}

// Assert that *lazyCombinedIter implements internalIterator.
var _ internalIterator = (*lazyCombinedIter)(nil)

// initCombinedIteration is invoked after a pointIter positioning operation
// resulted in i.combinedIterState.triggered=true.
//
// The `dir` parameter is `+1` or `-1` indicating forward iteration or backward
// iteration respectively.
//
// The `pointKey` and `pointValue` parameters provide the new point key-value
// pair that the iterator was just positioned to. The combined iterator should
// be seeded with this point key-value pair and return the smaller (forward
// iteration) or largest (backward iteration) of the two.
//
// The `seekKey` parameter is non-nil only if the iterator operation that
// triggered the switch to combined iteration was a SeekGE, SeekPrefixGE or
// SeekLT. It provides the seek key supplied and is used to seek the range-key
// iterator using the same key. This is necessary for SeekGE/SeekPrefixGE
// operations that land in the middle of a range key and must truncate to the
// user-provided seek key.
func (i *lazyCombinedIter) initCombinedIteration(
	dir int8, pointKey *InternalKey, pointValue base.LazyValue, seekKey []byte,
) (*InternalKey, base.LazyValue) {
	// Invariant: i.parent.rangeKey is nil.
	// Invariant: !i.combinedIterState.initialized.
	if invariants.Enabled {
		if i.combinedIterState.initialized {
			panic("pebble: combined iterator already initialized")
		}
		if i.parent.rangeKey != nil {
			panic("pebble: iterator already has a range-key iterator stack")
		}
	}

	// We need to determine the key to seek the range key iterator to. If
	// seekKey is not nil, the user-initiated operation that triggered the
	// switch to combined iteration was itself a seek, and we can use that key.
	// Otherwise, a First/Last or relative positioning operation triggered the
	// switch to combined iteration.
	//
	// The levelIter that observed a file containing range keys populated
	// combinedIterState.key with the smallest (forward) or largest (backward)
	// range key it observed. If multiple levelIters observed files with range
	// keys during the same operation on the mergingIter, combinedIterState.key
	// is the smallest [during forward iteration; largest in reverse iteration]
	// such key.
	if seekKey == nil {
		// Use the levelIter-populated key.
		seekKey = i.combinedIterState.key

		// We may need to adjust the levelIter-populated seek key to the
		// surfaced point key. If the key observed is beyond [in the iteration
		// direction] the current point key, there may still exist a range key
		// at an earlier key. Consider the following example:
		//
		//   L5:  000003:[bar.DEL.5, foo.RANGEKEYSET.9]
		//   L6:  000001:[bar.SET.2] 000002:[bax.RANGEKEYSET.8]
		//
		// A call to First() seeks the levels to files L5.000003 and L6.000001.
		// The L5 levelIter observes that L5.000003 contains the range key with
		// start key `foo`, and triggers a switch to combined iteration, setting
		// `combinedIterState.key` = `foo`.
		//
		// The L6 levelIter did not observe the true first range key
		// (bax.RANGEKEYSET.8), because it appears in a later sstable. When the
		// combined iterator is initialized, the range key iterator must be
		// seeked to a key that will find `bax`. To accomplish this, we seek the
		// key instead to `bar`. It is guaranteed that no range key exists
		// earlier than `bar`, otherwise a levelIter would've observed it and
		// set `combinedIterState.key` to its start key.
		if pointKey != nil {
			if dir == +1 && i.parent.cmp(i.combinedIterState.key, pointKey.UserKey) > 0 {
				seekKey = pointKey.UserKey
			} else if dir == -1 && i.parent.cmp(seekKey, pointKey.UserKey) < 0 {
				seekKey = pointKey.UserKey
			}
		}
	}

	// An operation on the point iterator observed a file containing range keys,
	// so we must switch to combined interleaving iteration. First, construct
	// the range key iterator stack. It must not exist, otherwise we'd already
	// be performing combined iteration.
	i.parent.rangeKey = iterRangeKeyStateAllocPool.Get().(*iteratorRangeKeyState)
	i.parent.rangeKey.init(i.parent.comparer.Compare, i.parent.comparer.Split, &i.parent.opts)
	i.parent.constructRangeKeyIter()

	// Initialize the Iterator's interleaving iterator.
	i.parent.rangeKey.iiter.Init(
		&i.parent.comparer, i.parent.pointIter, i.parent.rangeKey.rangeKeyIter,
		keyspan.InterleavingIterOpts{
			Mask:       &i.parent.rangeKeyMasking,
			LowerBound: i.parent.opts.LowerBound,
			UpperBound: i.parent.opts.UpperBound,
		})

	// Set the parent's primary iterator to point to the combined, interleaving
	// iterator that's now initialized with our current state.
	i.parent.iter = &i.parent.rangeKey.iiter
	i.combinedIterState.initialized = true
	i.combinedIterState.key = nil

	// All future iterator operations will go directly through the combined
	// iterator.
	//
	// Initialize the interleaving iterator. We pass the point key-value pair so
	// that the interleaving iterator knows where the point iterator is
	// positioned. Additionally, we pass the seek key to which the range-key
	// iterator should be seeked in order to initialize its position.
	//
	// In the forward direction (invert for backwards), the seek key is a key
	// guaranteed to find the smallest range key that's greater than the last
	// key the iterator returned. The range key may be less than pointKey, in
	// which case the range key will be interleaved next instead of the point
	// key.
	if dir == +1 {
		var prefix []byte
		if i.parent.hasPrefix {
			prefix = i.parent.prefixOrFullSeekKey
		}
		return i.parent.rangeKey.iiter.InitSeekGE(prefix, seekKey, pointKey, pointValue)
	}
	return i.parent.rangeKey.iiter.InitSeekLT(seekKey, pointKey, pointValue)
}

func (i *lazyCombinedIter) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.SeekGE(key, flags)
	}
	k, v := i.pointIter.SeekGE(key, flags)
	if i.combinedIterState.triggered {
		return i.initCombinedIteration(+1, k, v, key)
	}
	return k, v
}

func (i *lazyCombinedIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.SeekPrefixGE(prefix, key, flags)
	}
	k, v := i.pointIter.SeekPrefixGE(prefix, key, flags)
	if i.combinedIterState.triggered {
		return i.initCombinedIteration(+1, k, v, key)
	}
	return k, v
}

func (i *lazyCombinedIter) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*InternalKey, base.LazyValue) {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.SeekLT(key, flags)
	}
	k, v := i.pointIter.SeekLT(key, flags)
	if i.combinedIterState.triggered {
		return i.initCombinedIteration(-1, k, v, key)
	}
	return k, v
}

func (i *lazyCombinedIter) First() (*InternalKey, base.LazyValue) {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.First()
	}
	k, v := i.pointIter.First()
	if i.combinedIterState.triggered {
		return i.initCombinedIteration(+1, k, v, nil)
	}
	return k, v
}

func (i *lazyCombinedIter) Last() (*InternalKey, base.LazyValue) {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.Last()
	}
	k, v := i.pointIter.Last()
	if i.combinedIterState.triggered {
		return i.initCombinedIteration(-1, k, v, nil)
	}
	return k, v
}

func (i *lazyCombinedIter) Next() (*InternalKey, base.LazyValue) {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.Next()
	}
	k, v := i.pointIter.Next()
	if i.combinedIterState.triggered {
		return i.initCombinedIteration(+1, k, v, nil)
	}
	return k, v
}

func (i *lazyCombinedIter) NextPrefix(succKey []byte) (*InternalKey, base.LazyValue) {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.NextPrefix(succKey)
	}
	k, v := i.pointIter.NextPrefix(succKey)
	if i.combinedIterState.triggered {
		return i.initCombinedIteration(+1, k, v, nil)
	}
	return k, v
}

func (i *lazyCombinedIter) Prev() (*InternalKey, base.LazyValue) {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.Prev()
	}
	k, v := i.pointIter.Prev()
	if i.combinedIterState.triggered {
		return i.initCombinedIteration(-1, k, v, nil)
	}
	return k, v
}

func (i *lazyCombinedIter) Error() error {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.Error()
	}
	return i.pointIter.Error()
}

func (i *lazyCombinedIter) Close() error {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.Close()
	}
	return i.pointIter.Close()
}

func (i *lazyCombinedIter) SetBounds(lower, upper []byte) {
	if i.combinedIterState.initialized {
		i.parent.rangeKey.iiter.SetBounds(lower, upper)
		return
	}
	i.pointIter.SetBounds(lower, upper)
}

func (i *lazyCombinedIter) String() string {
	if i.combinedIterState.initialized {
		return i.parent.rangeKey.iiter.String()
	}
	return i.pointIter.String()
}
