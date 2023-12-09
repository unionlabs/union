// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import "github.com/cockroachdb/pebble/internal/base"

// InternalIteratorShim is a temporary iterator type used as a shim between
// keyspan.MergingIter and base.InternalIterator. It's used temporarily for
// range deletions during compactions, allowing range deletions to be
// interleaved by a compaction input iterator.
//
// TODO(jackson): This type should be removed, and the usages converted to using
// an InterleavingIterator type that interleaves keyspan.Spans from a
// keyspan.FragmentIterator with point keys.
type InternalIteratorShim struct {
	miter   MergingIter
	mbufs   MergingBuffers
	span    *Span
	iterKey base.InternalKey
}

// Assert that InternalIteratorShim implements InternalIterator.
var _ base.InternalIterator = &InternalIteratorShim{}

// Init initializes the internal iterator shim to merge the provided fragment
// iterators.
func (i *InternalIteratorShim) Init(cmp base.Compare, iters ...FragmentIterator) {
	i.miter.Init(cmp, noopTransform, &i.mbufs, iters...)
}

// Span returns the span containing the full set of keys over the key span at
// the current iterator position.
func (i *InternalIteratorShim) Span() *Span {
	return i.span
}

// SeekGE implements (base.InternalIterator).SeekGE.
func (i *InternalIteratorShim) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

// SeekPrefixGE implements (base.InternalIterator).SeekPrefixGE.
func (i *InternalIteratorShim) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

// SeekLT implements (base.InternalIterator).SeekLT.
func (i *InternalIteratorShim) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

// First implements (base.InternalIterator).First.
func (i *InternalIteratorShim) First() (*base.InternalKey, base.LazyValue) {
	i.span = i.miter.First()
	for i.span != nil && i.span.Empty() {
		i.span = i.miter.Next()
	}
	if i.span == nil {
		return nil, base.LazyValue{}
	}
	i.iterKey = base.InternalKey{UserKey: i.span.Start, Trailer: i.span.Keys[0].Trailer}
	return &i.iterKey, base.MakeInPlaceValue(i.span.End)
}

// Last implements (base.InternalIterator).Last.
func (i *InternalIteratorShim) Last() (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

// Next implements (base.InternalIterator).Next.
func (i *InternalIteratorShim) Next() (*base.InternalKey, base.LazyValue) {
	i.span = i.miter.Next()
	for i.span != nil && i.span.Empty() {
		i.span = i.miter.Next()
	}
	if i.span == nil {
		return nil, base.LazyValue{}
	}
	i.iterKey = base.InternalKey{UserKey: i.span.Start, Trailer: i.span.Keys[0].Trailer}
	return &i.iterKey, base.MakeInPlaceValue(i.span.End)
}

// NextPrefix implements (base.InternalIterator).NextPrefix.
func (i *InternalIteratorShim) NextPrefix([]byte) (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

// Prev implements (base.InternalIterator).Prev.
func (i *InternalIteratorShim) Prev() (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

// Error implements (base.InternalIterator).Error.
func (i *InternalIteratorShim) Error() error {
	return i.miter.Error()
}

// Close implements (base.InternalIterator).Close.
func (i *InternalIteratorShim) Close() error {
	return i.miter.Close()
}

// SetBounds implements (base.InternalIterator).SetBounds.
func (i *InternalIteratorShim) SetBounds(lower, upper []byte) {
}

// String implements fmt.Stringer.
func (i *InternalIteratorShim) String() string {
	return i.miter.String()
}
