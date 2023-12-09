// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import "github.com/cockroachdb/pebble/internal/base"

// SeekLE seeks to the span that contains or is before the target key.
func SeekLE(cmp base.Compare, iter FragmentIterator, key []byte) *Span {
	// NB: We use SeekLT in order to land on the proper span for a search
	// key that resides in the middle of a span. Consider the scenario:
	//
	//     a---e
	//         e---i
	//
	// The spans are indexed by their start keys `a` and `e`. If the
	// search key is `c` we want to land on the span [a,e). If we were to
	// use SeekGE then the search key `c` would land on the span [e,i) and
	// we'd have to backtrack. The one complexity here is what happens for the
	// search key `e`. In that case SeekLT will land us on the span [a,e)
	// and we'll have to move forward.
	iterSpan := iter.SeekLT(key)

	if iterSpan == nil {
		// Advance the iterator once to see if the next span has a start key
		// equal to key.
		iterSpan = iter.Next()
		if iterSpan == nil || cmp(key, iterSpan.Start) < 0 {
			// The iterator is exhausted or we've hit the next span.
			return nil
		}
	} else {
		// Invariant: key > iterSpan.Start
		if cmp(key, iterSpan.End) >= 0 {
			// The current span lies entirely before the search key. Check to see if
			// the next span contains the search key. If it doesn't, we'll backup
			// and return to our earlier candidate.
			iterSpan = iter.Next()
			if iterSpan == nil || cmp(key, iterSpan.Start) < 0 {
				// The next span is past our search key or there is no next span. Go
				// back.
				iterSpan = iter.Prev()
			}
		}
	}
	return iterSpan
}
