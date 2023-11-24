// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

// FilterFunc defines a transform from the input Span into the output Span. The
// function returns true if the Span should be returned by the iterator, and
// false if the Span should be skipped. The FilterFunc is permitted to mutate
// the output Span, for example, to elice certain keys, or update the Span's
// bounds if so desired. The output Span's Keys slice may be reused to reduce
// allocations.
type FilterFunc func(in *Span, out *Span) (keep bool)

// filteringIter is a FragmentIterator that uses a FilterFunc to select which
// Spans from the input iterator are returned in the output.
//
// A note on Span lifetimes: as the FilterFunc reuses a Span with a mutable
// slice of Keys to reduce allocations, Spans returned by this iterator are only
// valid until the next relative or absolute positioning method is called.
type filteringIter struct {
	iter     FragmentIterator
	filterFn FilterFunc

	// span is a mutable Span passed to the filterFn. The filterFn is free to
	// mutate this Span. The slice of Keys in the Span is reused with every call
	// to the filterFn.
	span Span
}

var _ FragmentIterator = (*filteringIter)(nil)

// Filter returns a new filteringIter that will filter the Spans from the
// provided child iterator using the provided FilterFunc.
func Filter(iter FragmentIterator, filter FilterFunc) FragmentIterator {
	return &filteringIter{iter: iter, filterFn: filter}
}

// SeekGE implements FragmentIterator.
func (i *filteringIter) SeekGE(key []byte) *Span {
	return i.filter(i.iter.SeekGE(key), +1)
}

// SeekLT implements FragmentIterator.
func (i *filteringIter) SeekLT(key []byte) *Span {
	return i.filter(i.iter.SeekLT(key), -1)
}

// First implements FragmentIterator.
func (i *filteringIter) First() *Span {
	return i.filter(i.iter.First(), +1)
}

// Last implements FragmentIterator.
func (i *filteringIter) Last() *Span {
	return i.filter(i.iter.Last(), -1)
}

// Next implements FragmentIterator.
func (i *filteringIter) Next() *Span {
	return i.filter(i.iter.Next(), +1)
}

// Prev implements FragmentIterator.
func (i *filteringIter) Prev() *Span {
	return i.filter(i.iter.Prev(), -1)
}

// Error implements FragmentIterator.
func (i *filteringIter) Error() error {
	return i.iter.Error()
}

// Close implements FragmentIterator.
func (i *filteringIter) Close() error {
	return i.iter.Close()
}

// filter uses the filterFn (if configured) to filter and possibly mutate the
// given Span. If the current Span is to be skipped, the iterator continues
// iterating in the given direction until it lands on a Span that should be
// returned, or the iterator becomes invalid.
func (i *filteringIter) filter(span *Span, dir int8) *Span {
	if i.filterFn == nil {
		return span
	}
	for i.Error() == nil && span != nil {
		if keep := i.filterFn(span, &i.span); keep {
			return &i.span
		}
		if dir == +1 {
			span = i.iter.Next()
		} else {
			span = i.iter.Prev()
		}
	}
	return span
}
