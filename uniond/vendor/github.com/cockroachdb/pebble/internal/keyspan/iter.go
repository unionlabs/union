// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/manifest"
)

// FragmentIterator defines an iterator interface over spans. The spans
// surfaced by a FragmentIterator must be non-overlapping. This is achieved by
// fragmenting spans at overlap points (see Fragmenter).
//
// A Span returned by a FragmentIterator is only valid until the next
// positioning method. Some implementations (eg, keyspan.Iter) may provide
// longer lifetimes but implementations need only guarantee stability until the
// next positioning method.
type FragmentIterator interface {
	// SeekGE moves the iterator to the first span covering a key greater than
	// or equal to the given key. This is equivalent to seeking to the first
	// span with an end key greater than the given key.
	SeekGE(key []byte) *Span

	// SeekLT moves the iterator to the last span covering a key less than the
	// given key. This is equivalent to seeking to the last span with a start
	// key less than the given key.
	SeekLT(key []byte) *Span

	// First moves the iterator to the first span.
	First() *Span

	// Last moves the iterator to the last span.
	Last() *Span

	// Next moves the iterator to the next span.
	//
	// It is valid to call Next when the iterator is positioned before the first
	// key/value pair due to either a prior call to SeekLT or Prev which
	// returned an invalid span. It is not allowed to call Next when the
	// previous call to SeekGE, SeekPrefixGE or Next returned an invalid span.
	Next() *Span

	// Prev moves the iterator to the previous span.
	//
	// It is valid to call Prev when the iterator is positioned after the last
	// key/value pair due to either a prior call to SeekGE or Next which
	// returned an invalid span. It is not allowed to call Prev when the
	// previous call to SeekLT or Prev returned an invalid span.
	Prev() *Span

	// Error returns any accumulated error.
	//
	// TODO(jackson): Lift errors into return values on the positioning methods.
	Error() error

	// Close closes the iterator and returns any accumulated error. Exhausting
	// the iterator is not considered to be an error. It is valid to call Close
	// multiple times. Other methods should not be called after the iterator has
	// been closed.
	Close() error
}

// TableNewSpanIter creates a new iterator for range key spans for the given
// file.
type TableNewSpanIter func(file *manifest.FileMetadata, iterOptions SpanIterOptions) (FragmentIterator, error)

// SpanIterOptions is a subset of IterOptions that are necessary to instantiate
// per-sstable span iterators.
type SpanIterOptions struct {
	// RangeKeyFilters can be used to avoid scanning tables and blocks in tables
	// when iterating over range keys.
	RangeKeyFilters []base.BlockPropertyFilter
}

// Iter is an iterator over a set of fragmented spans.
type Iter struct {
	cmp   base.Compare
	spans []Span
	index int
}

// Iter implements the FragmentIterator interface.
var _ FragmentIterator = (*Iter)(nil)

// NewIter returns a new iterator over a set of fragmented spans.
func NewIter(cmp base.Compare, spans []Span) *Iter {
	i := &Iter{}
	i.Init(cmp, spans)
	return i
}

// Count returns the number of spans contained by Iter.
func (i *Iter) Count() int {
	return len(i.spans)
}

// Init initializes an Iter with the provided spans.
func (i *Iter) Init(cmp base.Compare, spans []Span) {
	*i = Iter{
		cmp:   cmp,
		spans: spans,
		index: -1,
	}
}

// SeekGE implements FragmentIterator.SeekGE.
func (i *Iter) SeekGE(key []byte) *Span {
	// NB: manually inlined sort.Search is ~5% faster.
	//
	// Define f(j) = false iff the span i.spans[j] is strictly before `key`
	// (equivalently, i.spans[j].End ≤ key.)
	//
	// Define f(-1) == false and f(n) == true.
	// Invariant: f(index-1) == false, f(upper) == true.
	i.index = 0
	upper := len(i.spans)
	for i.index < upper {
		h := int(uint(i.index+upper) >> 1) // avoid overflow when computing h
		// i.index ≤ h < upper
		if i.cmp(key, i.spans[h].End) >= 0 {
			i.index = h + 1 // preserves f(i-1) == false
		} else {
			upper = h // preserves f(j) == true
		}
	}

	// i.index == upper, f(i.index-1) == false, and f(upper) (= f(i.index)) ==
	// true => answer is i.index.
	if i.index >= len(i.spans) {
		return nil
	}
	return &i.spans[i.index]
}

// SeekLT implements FragmentIterator.SeekLT.
func (i *Iter) SeekLT(key []byte) *Span {
	// NB: manually inlined sort.Search is ~5% faster.
	//
	// Define f(-1) == false and f(n) == true.
	// Invariant: f(index-1) == false, f(upper) == true.
	i.index = 0
	upper := len(i.spans)
	for i.index < upper {
		h := int(uint(i.index+upper) >> 1) // avoid overflow when computing h
		// i.index ≤ h < upper
		if i.cmp(key, i.spans[h].Start) > 0 {
			i.index = h + 1 // preserves f(i-1) == false
		} else {
			upper = h // preserves f(j) == true
		}
	}
	// i.index == upper, f(i.index-1) == false, and f(upper) (= f(i.index)) ==
	// true => answer is i.index.

	// Since keys are strictly increasing, if i.index > 0 then i.index-1 will be
	// the largest whose key is < the key sought.
	i.index--
	if i.index < 0 {
		return nil
	}
	return &i.spans[i.index]
}

// First implements FragmentIterator.First.
func (i *Iter) First() *Span {
	if len(i.spans) == 0 {
		return nil
	}
	i.index = 0
	return &i.spans[i.index]
}

// Last implements FragmentIterator.Last.
func (i *Iter) Last() *Span {
	if len(i.spans) == 0 {
		return nil
	}
	i.index = len(i.spans) - 1
	return &i.spans[i.index]
}

// Next implements FragmentIterator.Next.
func (i *Iter) Next() *Span {
	if i.index >= len(i.spans) {
		return nil
	}
	i.index++
	if i.index >= len(i.spans) {
		return nil
	}
	return &i.spans[i.index]
}

// Prev implements FragmentIterator.Prev.
func (i *Iter) Prev() *Span {
	if i.index < 0 {
		return nil
	}
	i.index--
	if i.index < 0 {
		return nil
	}
	return &i.spans[i.index]
}

// Error implements FragmentIterator.Error.
func (i *Iter) Error() error {
	return nil
}

// Close implements FragmentIterator.Close.
func (i *Iter) Close() error {
	return nil
}

func (i *Iter) String() string {
	return "fragmented-spans"
}
