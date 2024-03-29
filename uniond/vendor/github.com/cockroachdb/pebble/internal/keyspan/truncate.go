// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import "github.com/cockroachdb/pebble/internal/base"

// Truncate creates a new iterator where every span in the supplied iterator is
// truncated to be contained within the range [lower, upper). If start and end
// are specified, filter out any spans that are completely outside those bounds.
func Truncate(
	cmp base.Compare,
	iter FragmentIterator,
	lower, upper []byte,
	start, end *base.InternalKey,
	panicOnUpperTruncate bool,
) FragmentIterator {
	return Filter(iter, func(in *Span, out *Span) (keep bool) {
		out.Start, out.End = in.Start, in.End
		out.Keys = append(out.Keys[:0], in.Keys...)

		// Ignore this span if it lies completely outside start, end. Note that
		// end endInclusive indicated whether end is inclusive.
		//
		// The comparison between s.End and start is by user key only, as
		// the span is exclusive at s.End, so comparing by user keys
		// is sufficient.
		if start != nil && cmp(in.End, start.UserKey) <= 0 {
			return false
		}
		if end != nil {
			v := cmp(in.Start, end.UserKey)
			switch {
			case v > 0:
				// Wholly outside the end bound. Skip it.
				return false
			case v == 0:
				// This span begins at the same user key as `end`. Whether or
				// not any of the keys contained within the span are relevant is
				// dependent on Trailers. Any keys contained within the span
				// with trailers larger than end cover the small sliver of
				// keyspace between [k#inf, k#<end-seqnum>]. Since keys are
				// sorted descending by Trailer within the span, we need to find
				// the prefix of keys with larger trailers.
				for i := range in.Keys {
					if in.Keys[i].Trailer < end.Trailer {
						out.Keys = out.Keys[:i]
						break
					}
				}
			default:
				// Wholly within the end bound. Keep it.
			}
		}

		var truncated bool
		// Truncate the bounds to lower and upper.
		if cmp(in.Start, lower) < 0 {
			out.Start = lower
		}
		if cmp(in.End, upper) > 0 {
			truncated = true
			out.End = upper
		}

		if panicOnUpperTruncate && truncated {
			panic("pebble: upper bound should not be truncated")
		}

		return !out.Empty() && cmp(out.Start, out.End) < 0
	}, cmp)
}
