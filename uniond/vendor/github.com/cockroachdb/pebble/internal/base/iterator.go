// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import (
	"fmt"
	"time"

	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/redact"
)

// InternalIterator iterates over a DB's key/value pairs in key order. Unlike
// the Iterator interface, the returned keys are InternalKeys composed of the
// user-key, a sequence number and a key kind. In forward iteration, key/value
// pairs for identical user-keys are returned in descending sequence order. In
// reverse iteration, key/value pairs for identical user-keys are returned in
// ascending sequence order.
//
// InternalIterators provide 5 absolute positioning methods and 2 relative
// positioning methods. The absolute positioning methods are:
//
// - SeekGE
// - SeekPrefixGE
// - SeekLT
// - First
// - Last
//
// The relative positioning methods are:
//
// - Next
// - Prev
//
// The relative positioning methods can be used in conjunction with any of the
// absolute positioning methods with one exception: SeekPrefixGE does not
// support reverse iteration via Prev. It is undefined to call relative
// positioning methods without ever calling an absolute positioning method.
//
// InternalIterators can optionally implement a prefix iteration mode. This
// mode is entered by calling SeekPrefixGE and exited by any other absolute
// positioning method (SeekGE, SeekLT, First, Last). When in prefix iteration
// mode, a call to Next will advance to the next key which has the same
// "prefix" as the one supplied to SeekPrefixGE. Note that "prefix" in this
// context is not a strict byte prefix, but defined by byte equality for the
// result of the Comparer.Split method. An InternalIterator is not required to
// support prefix iteration mode, and can implement SeekPrefixGE by forwarding
// to SeekGE. When the iteration prefix is exhausted, it is not valid to call
// Next on an internal iterator that's already returned (nil,nilv) or a key
// beyond the prefix.
//
// Bounds, [lower, upper), can be set on iterators, either using the SetBounds()
// function in the interface, or in implementation specific ways during iterator
// creation. The forward positioning routines (SeekGE, First, and Next) only
// check the upper bound. The reverse positioning routines (SeekLT, Last, and
// Prev) only check the lower bound. It is up to the caller to ensure that the
// forward positioning routines respect the lower bound and the reverse
// positioning routines respect the upper bound (i.e. calling SeekGE instead of
// First if there is a lower bound, and SeekLT instead of Last if there is an
// upper bound). This imposition is done in order to elevate that enforcement to
// the caller (generally pebble.Iterator or pebble.mergingIter) rather than
// having it duplicated in every InternalIterator implementation.
//
// Additionally, the caller needs to ensure that SeekGE/SeekPrefixGE are not
// called with a key > the upper bound, and SeekLT is not called with a key <
// the lower bound. InternalIterator implementations are required to respect
// the iterator bounds, never returning records outside of the bounds with one
// exception: an iterator may generate synthetic RANGEDEL marker records. See
// levelIter.syntheticBoundary for the sole existing example of this behavior.
// Specifically, levelIter can return synthetic keys whose user key is equal to
// the lower/upper bound.
//
// The bounds provided to an internal iterator must remain valid until a
// subsequent call to SetBounds has returned. This requirement exists so that
// iterator implementations may compare old and new bounds to apply low-level
// optimizations. The pebble.Iterator satisfies this requirement by maintaining
// two bound buffers and switching between them.
//
// An iterator must be closed after use, but it is not necessary to read an
// iterator until exhaustion.
//
// An iterator is not goroutine-safe, but it is safe to use multiple iterators
// concurrently, either in separate goroutines or switching between the
// iterators in a single goroutine.
//
// It is also safe to use an iterator concurrently with modifying its
// underlying DB, if that DB permits modification. However, the resultant
// key/value pairs are not guaranteed to be a consistent snapshot of that DB
// at a particular point in time.
//
// InternalIterators accumulate errors encountered during operation, exposing
// them through the Error method. All of the absolute positioning methods
// reset any accumulated error before positioning. Relative positioning
// methods return without advancing if the iterator has accumulated an error.
//
// nilv == shorthand for LazyValue{}, which represents a nil value.
type InternalIterator interface {
	// SeekGE moves the iterator to the first key/value pair whose key is greater
	// than or equal to the given key. Returns the key and value if the iterator
	// is pointing at a valid entry, and (nil, nilv) otherwise. Note that SeekGE
	// only checks the upper bound. It is up to the caller to ensure that key
	// is greater than or equal to the lower bound.
	SeekGE(key []byte, flags SeekGEFlags) (*InternalKey, LazyValue)

	// SeekPrefixGE moves the iterator to the first key/value pair whose key is
	// greater than or equal to the given key. Returns the key and value if the
	// iterator is pointing at a valid entry, and (nil, nilv) otherwise. Note that
	// SeekPrefixGE only checks the upper bound. It is up to the caller to ensure
	// that key is greater than or equal to the lower bound.
	//
	// The prefix argument is used by some InternalIterator implementations (e.g.
	// sstable.Reader) to avoid expensive operations. A user-defined Split
	// function must be supplied to the Comparer for the DB. The supplied prefix
	// will be the prefix of the given key returned by that Split function. If
	// the iterator is able to determine that no key with the prefix exists, it
	// can return (nil,nilv). Unlike SeekGE, this is not an indication that
	// iteration is exhausted.
	//
	// Note that the iterator may return keys not matching the prefix. It is up
	// to the caller to check if the prefix matches.
	//
	// Calling SeekPrefixGE places the receiver into prefix iteration mode. Once
	// in this mode, reverse iteration may not be supported and will return an
	// error. Note that pebble/Iterator.SeekPrefixGE has this same restriction on
	// not supporting reverse iteration in prefix iteration mode until a
	// different positioning routine (SeekGE, SeekLT, First or Last) switches the
	// iterator out of prefix iteration.
	SeekPrefixGE(prefix, key []byte, flags SeekGEFlags) (*InternalKey, LazyValue)

	// SeekLT moves the iterator to the last key/value pair whose key is less
	// than the given key. Returns the key and value if the iterator is pointing
	// at a valid entry, and (nil, nilv) otherwise. Note that SeekLT only checks
	// the lower bound. It is up to the caller to ensure that key is less than
	// the upper bound.
	SeekLT(key []byte, flags SeekLTFlags) (*InternalKey, LazyValue)

	// First moves the iterator the the first key/value pair. Returns the key and
	// value if the iterator is pointing at a valid entry, and (nil, nilv)
	// otherwise. Note that First only checks the upper bound. It is up to the
	// caller to ensure that First() is not called when there is a lower bound,
	// and instead call SeekGE(lower).
	First() (*InternalKey, LazyValue)

	// Last moves the iterator the the last key/value pair. Returns the key and
	// value if the iterator is pointing at a valid entry, and (nil, nilv)
	// otherwise. Note that Last only checks the lower bound. It is up to the
	// caller to ensure that Last() is not called when there is an upper bound,
	// and instead call SeekLT(upper).
	Last() (*InternalKey, LazyValue)

	// Next moves the iterator to the next key/value pair. Returns the key and
	// value if the iterator is pointing at a valid entry, and (nil, nilv)
	// otherwise. Note that Next only checks the upper bound. It is up to the
	// caller to ensure that key is greater than or equal to the lower bound.
	//
	// It is valid to call Next when the iterator is positioned before the first
	// key/value pair due to either a prior call to SeekLT or Prev which returned
	// (nil, nilv). It is not allowed to call Next when the previous call to SeekGE,
	// SeekPrefixGE or Next returned (nil, nilv).
	Next() (*InternalKey, LazyValue)

	// NextPrefix moves the iterator to the next key/value pair with a different
	// prefix than the key at the current iterator position. Returns the key and
	// value if the iterator is pointing at a valid entry, and (nil, nil)
	// otherwise. Note that NextPrefix only checks the upper bound. It is up to
	// the caller to ensure that key is greater than or equal to the lower
	// bound.
	//
	// NextPrefix is passed the immediate successor to the current prefix key. A
	// valid implementation of NextPrefix is to call SeekGE with succKey.
	//
	// It is not allowed to call NextPrefix when the previous call was a reverse
	// positioning operation or a call to a forward positioning method that
	// returned (nil, nilv). It is also not allowed to call NextPrefix when the
	// iterator is in prefix iteration mode.
	NextPrefix(succKey []byte) (*InternalKey, LazyValue)

	// Prev moves the iterator to the previous key/value pair. Returns the key
	// and value if the iterator is pointing at a valid entry, and (nil, nilv)
	// otherwise. Note that Prev only checks the lower bound. It is up to the
	// caller to ensure that key is less than the upper bound.
	//
	// It is valid to call Prev when the iterator is positioned after the last
	// key/value pair due to either a prior call to SeekGE or Next which returned
	// (nil, nilv). It is not allowed to call Prev when the previous call to SeekLT
	// or Prev returned (nil, nilv).
	Prev() (*InternalKey, LazyValue)

	// Error returns any accumulated error. It may not include errors returned
	// to the client when calling LazyValue.Value().
	Error() error

	// Close closes the iterator and returns any accumulated error. Exhausting
	// all the key/value pairs in a table is not considered to be an error.
	// It is valid to call Close multiple times. Other methods should not be
	// called after the iterator has been closed.
	Close() error

	// SetBounds sets the lower and upper bounds for the iterator. Note that the
	// result of Next and Prev will be undefined until the iterator has been
	// repositioned with SeekGE, SeekPrefixGE, SeekLT, First, or Last.
	//
	// The bounds provided must remain valid until a subsequent call to
	// SetBounds has returned. This requirement exists so that iterator
	// implementations may compare old and new bounds to apply low-level
	// optimizations.
	SetBounds(lower, upper []byte)

	fmt.Stringer
}

// SeekGEFlags holds flags that may configure the behavior of a forward seek.
// Not all flags are relevant to all iterators.
type SeekGEFlags uint8

const (
	seekGEFlagTrySeekUsingNext uint8 = iota
	seekGEFlagRelativeSeek
	seekGEFlagBatchJustRefreshed
)

// SeekGEFlagsNone is the default value of SeekGEFlags, with all flags disabled.
const SeekGEFlagsNone = SeekGEFlags(0)

// TrySeekUsingNext indicates whether a performance optimization was enabled
// by a caller, indicating the caller has not done any action to move this
// iterator beyond the first key that would be found if this iterator were to
// honestly do the intended seek. For example, say the caller did a
// SeekGE(k1...), followed by SeekGE(k2...) where k1 <= k2, without any
// intermediate positioning calls. The caller can safely specify true for this
// parameter in the second call. As another example, say the caller did do one
// call to Next between the two Seek calls, and k1 < k2. Again, the caller can
// safely specify a true value for this parameter. Note that a false value is
// always safe. The callee is free to ignore the true value if its
// implementation does not permit this optimization.
//
// We make the caller do this determination since a string comparison of k1, k2
// is not necessarily cheap, and there may be many iterators in the iterator
// stack. Doing it once at the root of the iterator stack is cheaper.
//
// This optimization could also be applied to SeekLT (where it would be
// trySeekUsingPrev). We currently only do it for SeekPrefixGE and SeekGE
// because this is where this optimization helps the performance of CockroachDB.
// The SeekLT cases in CockroachDB are typically accompanied with bounds that
// change between seek calls, and is optimized inside certain iterator
// implementations, like singleLevelIterator, without any extra parameter
// passing (though the same amortization of string comparisons could be done to
// improve that optimization, by making the root of the iterator stack do it).
func (s SeekGEFlags) TrySeekUsingNext() bool { return (s & (1 << seekGEFlagTrySeekUsingNext)) != 0 }

// RelativeSeek is set when in the course of a forward positioning operation, a
// higher-level iterator seeks a lower-level iterator to a larger key than the
// one at the current iterator position.
//
// Concretely, this occurs when the merging iterator observes a range deletion
// covering the key at a level's current position, and the merging iterator
// seeks the level to the range deletion's end key. During lazy-combined
// iteration, this flag signals to the level iterator that the seek is NOT an
// absolute-positioning operation from the perspective of the pebble.Iterator,
// and the level iterator must look for range keys in tables between the current
// iterator position and the new seeked position.
func (s SeekGEFlags) RelativeSeek() bool { return (s & (1 << seekGEFlagRelativeSeek)) != 0 }

// BatchJustRefreshed is set by Seek[Prefix]GE when an iterator's view of an
// indexed batch was just refreshed. It serves as a signal to the batch iterator
// to ignore the TrySeekUsingNext optimization, because the external knowledge
// imparted by the TrySeekUsingNext flag does not apply to the batch iterator's
// position. See (pebble.Iterator).batchJustRefreshed.
func (s SeekGEFlags) BatchJustRefreshed() bool { return (s & (1 << seekGEFlagBatchJustRefreshed)) != 0 }

// EnableTrySeekUsingNext returns the provided flags with the
// try-seek-using-next optimization enabled. See TrySeekUsingNext for an
// explanation of this optimization.
func (s SeekGEFlags) EnableTrySeekUsingNext() SeekGEFlags {
	return s | (1 << seekGEFlagTrySeekUsingNext)
}

// DisableTrySeekUsingNext returns the provided flags with the
// try-seek-using-next optimization disabled.
func (s SeekGEFlags) DisableTrySeekUsingNext() SeekGEFlags {
	return s &^ (1 << seekGEFlagTrySeekUsingNext)
}

// EnableRelativeSeek returns the provided flags with the relative-seek flag
// enabled. See RelativeSeek for an explanation of this flag's use.
func (s SeekGEFlags) EnableRelativeSeek() SeekGEFlags {
	return s | (1 << seekGEFlagRelativeSeek)
}

// DisableRelativeSeek returns the provided flags with the relative-seek flag
// disabled.
func (s SeekGEFlags) DisableRelativeSeek() SeekGEFlags {
	return s &^ (1 << seekGEFlagRelativeSeek)
}

// EnableBatchJustRefreshed returns the provided flags with the
// batch-just-refreshed bit set. See BatchJustRefreshed for an explanation of
// this flag.
func (s SeekGEFlags) EnableBatchJustRefreshed() SeekGEFlags {
	return s | (1 << seekGEFlagBatchJustRefreshed)
}

// DisableBatchJustRefreshed returns the provided flags with the
// batch-just-refreshed bit unset.
func (s SeekGEFlags) DisableBatchJustRefreshed() SeekGEFlags {
	return s &^ (1 << seekGEFlagBatchJustRefreshed)
}

// SeekLTFlags holds flags that may configure the behavior of a reverse seek.
// Not all flags are relevant to all iterators.
type SeekLTFlags uint8

const (
	seekLTFlagRelativeSeek uint8 = iota
)

// SeekLTFlagsNone is the default value of SeekLTFlags, with all flags disabled.
const SeekLTFlagsNone = SeekLTFlags(0)

// RelativeSeek is set when in the course of a reverse positioning operation, a
// higher-level iterator seeks a lower-level iterator to a smaller key than the
// one at the current iterator position.
//
// Concretely, this occurs when the merging iterator observes a range deletion
// covering the key at a level's current position, and the merging iterator
// seeks the level to the range deletion's start key. During lazy-combined
// iteration, this flag signals to the level iterator that the seek is NOT an
// absolute-positioning operation from the perspective of the pebble.Iterator,
// and the level iterator must look for range keys in tables between the current
// iterator position and the new seeked position.
func (s SeekLTFlags) RelativeSeek() bool { return s&(1<<seekLTFlagRelativeSeek) != 0 }

// EnableRelativeSeek returns the provided flags with the relative-seek flag
// enabled. See RelativeSeek for an explanation of this flag's use.
func (s SeekLTFlags) EnableRelativeSeek() SeekLTFlags {
	return s | (1 << seekLTFlagRelativeSeek)
}

// DisableRelativeSeek returns the provided flags with the relative-seek flag
// disabled.
func (s SeekLTFlags) DisableRelativeSeek() SeekLTFlags {
	return s &^ (1 << seekLTFlagRelativeSeek)
}

// InternalIteratorStats contains miscellaneous stats produced by
// InternalIterators that are part of the InternalIterator tree. Not every
// field is relevant for an InternalIterator implementation. The field values
// are aggregated as one goes up the InternalIterator tree.
type InternalIteratorStats struct {
	// Bytes in the loaded blocks. If the block was compressed, this is the
	// compressed bytes. Currently, only the index blocks, data blocks
	// containing points, and filter blocks are included.
	BlockBytes uint64
	// Subset of BlockBytes that were in the block cache.
	BlockBytesInCache uint64
	// BlockReadDuration accumulates the duration spent fetching blocks
	// due to block cache misses.
	// TODO(sumeer): this currently excludes the time spent in Reader creation,
	// and in reading the rangedel and rangekey blocks. Fix that.
	BlockReadDuration time.Duration
	// The following can repeatedly count the same points if they are iterated
	// over multiple times. Additionally, they may count a point twice when
	// switching directions. The latter could be improved if needed.

	// Bytes in keys that were iterated over. Currently, only point keys are
	// included.
	KeyBytes uint64
	// Bytes in values that were iterated over. Currently, only point values are
	// included. For separated values, this is the size of the handle.
	ValueBytes uint64
	// The count of points iterated over.
	PointCount uint64
	// Points that were iterated over that were covered by range tombstones. It
	// can be useful for discovering instances of
	// https://github.com/cockroachdb/pebble/issues/1070.
	PointsCoveredByRangeTombstones uint64

	// Stats related to points in value blocks encountered during iteration.
	// These are useful to understand outliers, since typical user facing
	// iteration should tend to only look at the latest point, and hence have
	// the following stats close to 0.
	SeparatedPointValue struct {
		// Count is a count of points that were in value blocks. This is not a
		// subset of PointCount: PointCount is produced by mergingIter and if
		// positioned once, and successful in returning a point, will have a
		// PointCount of 1, regardless of how many sstables (and memtables etc.)
		// in the heap got positioned. The count here includes every sstable
		// iterator that got positioned in the heap.
		Count uint64
		// ValueBytes represent the total byte length of the values (in value
		// blocks) of the points corresponding to Count.
		ValueBytes uint64
		// ValueBytesFetched is the total byte length of the values (in value
		// blocks) that were retrieved.
		ValueBytesFetched uint64
	}
}

// Merge merges the stats in from into the given stats.
func (s *InternalIteratorStats) Merge(from InternalIteratorStats) {
	s.BlockBytes += from.BlockBytes
	s.BlockBytesInCache += from.BlockBytesInCache
	s.BlockReadDuration += from.BlockReadDuration
	s.KeyBytes += from.KeyBytes
	s.ValueBytes += from.ValueBytes
	s.PointCount += from.PointCount
	s.PointsCoveredByRangeTombstones += from.PointsCoveredByRangeTombstones
	s.SeparatedPointValue.Count += from.SeparatedPointValue.Count
	s.SeparatedPointValue.ValueBytes += from.SeparatedPointValue.ValueBytes
	s.SeparatedPointValue.ValueBytesFetched += from.SeparatedPointValue.ValueBytesFetched
}

func (s *InternalIteratorStats) String() string {
	return redact.StringWithoutMarkers(s)
}

// SafeFormat implements the redact.SafeFormatter interface.
func (s *InternalIteratorStats) SafeFormat(p redact.SafePrinter, verb rune) {
	p.Printf("blocks: %s cached",
		humanize.Bytes.Uint64(s.BlockBytesInCache),
	)
	if s.BlockBytes != s.BlockBytesInCache || s.BlockReadDuration != 0 {
		p.Printf(", %s not cached (read time: %s)",
			humanize.Bytes.Uint64(s.BlockBytes-s.BlockBytesInCache),
			humanize.FormattedString(s.BlockReadDuration.String()),
		)
	}
	p.Printf("; points: %s", humanize.Count.Uint64(s.PointCount))

	if s.PointsCoveredByRangeTombstones != 0 {
		p.Printf("(%s tombstoned)", humanize.Count.Uint64(s.PointsCoveredByRangeTombstones))
	}
	p.Printf(" (%s keys, %s values)",
		humanize.Bytes.Uint64(s.KeyBytes),
		humanize.Bytes.Uint64(s.ValueBytes),
	)
	if s.SeparatedPointValue.Count != 0 {
		p.Printf("; separated: %s (%s, %s fetched)",
			humanize.Count.Uint64(s.SeparatedPointValue.Count),
			humanize.Bytes.Uint64(s.SeparatedPointValue.ValueBytes),
			humanize.Bytes.Uint64(s.SeparatedPointValue.ValueBytesFetched))
	}
}
