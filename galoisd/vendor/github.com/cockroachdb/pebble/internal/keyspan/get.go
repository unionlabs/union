// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import "github.com/cockroachdb/pebble/internal/base"

// Get returns the newest span that contains the target key. If no span
// contains the target key, an empty span is returned. The snapshot
// parameter controls the visibility of spans (only spans older than the
// snapshot sequence number are visible). The iterator must contain
// fragmented spans: no span may overlap another.
func Get(cmp base.Compare, iter FragmentIterator, key []byte) *Span {
	// NB: We use SeekLT in order to land on the proper span for a search
	// key that resides in the middle of a span. Consider the scenario:
	//
	//     a---e
	//         e---i
	//
	// The spans are indexed by their start keys `a` and `e`. If the
	// search key is `c` we want to land on the span [a,e). If we were
	// to use SeekGE then the search key `c` would land on the span
	// [e,i) and we'd have to backtrack. The one complexity here is what
	// happens for the search key `e`. In that case SeekLT will land us
	// on the span [a,e) and we'll have to move forward.
	iterSpan := iter.SeekLT(key)
	if iterSpan == nil {
		iterSpan = iter.Next()
		if iterSpan == nil {
			// The iterator is empty.
			return nil
		}
		if cmp(key, iterSpan.Start) < 0 {
			// The search key lies before the first span.
			return nil
		}
	}

	// Invariant: key > iterSpan.Start
	if cmp(key, iterSpan.End) >= 0 {
		// The current span lies before the search key. Advance the iterator
		// once to potentially land on a key with a start key exactly equal to
		// key. (See the comment at the beginning of this function.)
		iterSpan = iter.Next()
		if iterSpan == nil || cmp(key, iterSpan.Start) < 0 {
			// We've run out of spans or we've moved on to a span which
			// starts after our search key.
			return nil
		}
	}
	return iterSpan
}
