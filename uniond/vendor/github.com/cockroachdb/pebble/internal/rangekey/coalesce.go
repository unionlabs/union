// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package rangekey

import (
	"bytes"
	"math"
	"sort"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
)

// UserIteratorConfig holds state for constructing the range key iterator stack
// for user iteration. The range key iterator must merge range key spans across
// the levels of the LSM. This merging is performed by a keyspan.MergingIter
// on-the-fly. The UserIteratorConfig implements keyspan.Transformer, evaluating
// range-key semantics and shadowing, so the spans returned by a MergingIter are
// fully resolved.
//
// The MergingIter is wrapped by a BoundedIter, which elides spans that are
// outside the iterator bounds (or the current prefix's bounds, during prefix
// iteration mode).
//
// To provide determinisim during iteration, the BoundedIter is wrapped by a
// DefragmentingIter that defragments abutting spans with identical
// user-observable state.
//
// At the top-level an InterleavingIter interleaves range keys with point keys
// and performs truncation to iterator bounds.
//
// Below is an abbreviated diagram illustrating the mechanics of a SeekGE.
//
//	               InterleavingIter.SeekGE
//	                       │
//	            DefragmentingIter.SeekGE
//	                       │
//	               BoundedIter.SeekGE
//	                       │
//	      ╭────────────────┴───────────────╮
//	      │                                ├── defragmentBwd*
//	MergingIter.SeekGE                     │
//	      │                                ╰── defragmentFwd
//	      ╰─╶╶ per level╶╶ ─╮
//	                        │
//	                        │
//	                        ├── <?>.SeekLT
//	                        │
//	                        ╰── <?>.Next
type UserIteratorConfig struct {
	snapshot     uint64
	comparer     *base.Comparer
	miter        keyspan.MergingIter
	biter        keyspan.BoundedIter
	diter        keyspan.DefragmentingIter
	liters       [manifest.NumLevels]keyspan.LevelIter
	litersUsed   int
	internalKeys bool
	bufs         *Buffers
}

// Buffers holds various buffers used for range key iteration. They're exposed
// so that they may be pooled and reused between iterators.
type Buffers struct {
	merging       keyspan.MergingBuffers
	defragmenting keyspan.DefragmentingBuffers
	sortBuf       keyspan.KeysBySuffix
}

// PrepareForReuse discards any excessively large buffers.
func (bufs *Buffers) PrepareForReuse() {
	bufs.merging.PrepareForReuse()
	bufs.defragmenting.PrepareForReuse()
}

// Init initializes the range key iterator stack for user iteration. The
// resulting fragment iterator applies range key semantics, defragments spans
// according to their user-observable state and, if !internalKeys, removes all
// Keys other than RangeKeySets describing the current state of range keys. The
// resulting spans contain Keys sorted by suffix (unless internalKeys is true,
// in which case they remain sorted by trailer descending).
//
// The snapshot sequence number parameter determines which keys are visible. Any
// keys not visible at the provided snapshot are ignored.
func (ui *UserIteratorConfig) Init(
	comparer *base.Comparer,
	snapshot uint64,
	lower, upper []byte,
	hasPrefix *bool,
	prefix *[]byte,
	internalKeys bool,
	bufs *Buffers,
	iters ...keyspan.FragmentIterator,
) keyspan.FragmentIterator {
	ui.snapshot = snapshot
	ui.comparer = comparer
	ui.internalKeys = internalKeys
	ui.miter.Init(comparer.Compare, ui, &bufs.merging, iters...)
	ui.biter.Init(comparer.Compare, comparer.Split, &ui.miter, lower, upper, hasPrefix, prefix)
	if internalKeys {
		ui.diter.Init(comparer, &ui.biter, keyspan.DefragmentInternal, keyspan.StaticDefragmentReducer, &bufs.defragmenting)
	} else {
		ui.diter.Init(comparer, &ui.biter, ui, keyspan.StaticDefragmentReducer, &bufs.defragmenting)
	}
	ui.litersUsed = 0
	ui.bufs = bufs
	return &ui.diter
}

// AddLevel adds a new level to the bottom of the iterator stack. AddLevel
// must be called after Init and before any other method on the iterator.
func (ui *UserIteratorConfig) AddLevel(iter keyspan.FragmentIterator) {
	ui.miter.AddLevel(iter)
}

// NewLevelIter returns a pointer to a newly allocated or reused
// keyspan.LevelIter. The caller is responsible for calling Init() on this
// instance.
func (ui *UserIteratorConfig) NewLevelIter() *keyspan.LevelIter {
	if ui.litersUsed >= len(ui.liters) {
		return &keyspan.LevelIter{}
	}
	ui.litersUsed++
	return &ui.liters[ui.litersUsed-1]
}

// SetBounds propagates bounds to the iterator stack. The fragment iterator
// interface ordinarily doesn't enforce bounds, so this is exposed as an
// explicit method on the user iterator config.
func (ui *UserIteratorConfig) SetBounds(lower, upper []byte) {
	ui.biter.SetBounds(lower, upper)
}

// Transform implements the keyspan.Transformer interface for use with a
// keyspan.MergingIter. It transforms spans by resolving range keys at the
// provided snapshot sequence number. Shadowing of keys is resolved (eg, removal
// of unset keys, removal of keys overwritten by a set at the same suffix, etc)
// and then non-RangeKeySet keys are removed. The resulting transformed spans
// only contain RangeKeySets describing the state visible at the provided
// sequence number, and hold their Keys sorted by Suffix (except if internalKeys
// is true, then keys remain sorted by trailer.
func (ui *UserIteratorConfig) Transform(cmp base.Compare, s keyspan.Span, dst *keyspan.Span) error {
	// Apply shadowing of keys.
	dst.Start = s.Start
	dst.End = s.End
	ui.bufs.sortBuf = keyspan.KeysBySuffix{
		Cmp:  cmp,
		Keys: ui.bufs.sortBuf.Keys[:0],
	}
	if err := coalesce(ui.comparer.Equal, &ui.bufs.sortBuf, ui.snapshot, s.Keys); err != nil {
		return err
	}
	if ui.internalKeys {
		if s.KeysOrder != keyspan.ByTrailerDesc {
			panic("unexpected key ordering in UserIteratorTransform with internalKeys = true")
		}
		dst.Keys = ui.bufs.sortBuf.Keys
		keyspan.SortKeysByTrailer(&dst.Keys)
		return nil
	}
	// During user iteration over range keys, unsets and deletes don't matter. This
	// step helps logical defragmentation during iteration.
	keys := ui.bufs.sortBuf.Keys
	dst.Keys = dst.Keys[:0]
	for i := range keys {
		switch keys[i].Kind() {
		case base.InternalKeyKindRangeKeySet:
			if invariants.Enabled && len(dst.Keys) > 0 && cmp(dst.Keys[len(dst.Keys)-1].Suffix, keys[i].Suffix) > 0 {
				panic("pebble: keys unexpectedly not in ascending suffix order")
			}
			dst.Keys = append(dst.Keys, keys[i])
		case base.InternalKeyKindRangeKeyUnset:
			if invariants.Enabled && len(dst.Keys) > 0 && cmp(dst.Keys[len(dst.Keys)-1].Suffix, keys[i].Suffix) > 0 {
				panic("pebble: keys unexpectedly not in ascending suffix order")
			}
			// Skip.
			continue
		case base.InternalKeyKindRangeKeyDelete:
			// Skip.
			continue
		default:
			return base.CorruptionErrorf("pebble: unrecognized range key kind %s", keys[i].Kind())
		}
	}
	// coalesce results in dst.Keys being sorted by Suffix.
	dst.KeysOrder = keyspan.BySuffixAsc
	return nil
}

// ShouldDefragment implements the DefragmentMethod interface and configures a
// DefragmentingIter to defragment spans of range keys if their user-visible
// state is identical. This defragmenting method assumes the provided spans have
// already been transformed through (UserIterationConfig).Transform, so all
// RangeKeySets are user-visible sets and are already in Suffix order. This
// defragmenter checks for equality between set suffixes and values (ignoring
// sequence numbers). It's intended for use during user iteration, when the
// wrapped keyspan iterator is merging spans across all levels of the LSM.
func (ui *UserIteratorConfig) ShouldDefragment(equal base.Equal, a, b *keyspan.Span) bool {
	// This method is not called with internalKeys = true.
	if ui.internalKeys {
		panic("unexpected call to ShouldDefragment with internalKeys = true")
	}
	// This implementation must only be used on spans that have transformed by
	// ui.Transform. The transform applies shadowing, removes all keys besides
	// the resulting Sets and sorts the keys by suffix. Since shadowing has been
	// applied, each Set must set a unique suffix. If the two spans are
	// equivalent, they must have the same number of range key sets.
	if len(a.Keys) != len(b.Keys) || len(a.Keys) == 0 {
		return false
	}
	if a.KeysOrder != keyspan.BySuffixAsc || b.KeysOrder != keyspan.BySuffixAsc {
		panic("pebble: range key span's keys unexpectedly not in ascending suffix order")
	}

	ret := true
	for i := range a.Keys {
		if invariants.Enabled {
			if a.Keys[i].Kind() != base.InternalKeyKindRangeKeySet ||
				b.Keys[i].Kind() != base.InternalKeyKindRangeKeySet {
				panic("pebble: unexpected non-RangeKeySet during defragmentation")
			}
			if i > 0 && (ui.comparer.Compare(a.Keys[i].Suffix, a.Keys[i-1].Suffix) < 0 ||
				ui.comparer.Compare(b.Keys[i].Suffix, b.Keys[i-1].Suffix) < 0) {
				panic("pebble: range keys not ordered by suffix during defragmentation")
			}
		}
		if !equal(a.Keys[i].Suffix, b.Keys[i].Suffix) {
			ret = false
			break
		}
		if !bytes.Equal(a.Keys[i].Value, b.Keys[i].Value) {
			ret = false
			break
		}
	}
	return ret
}

// Coalesce imposes range key semantics and coalesces range keys with the same
// bounds. Coalesce drops any keys shadowed by more recent sets, unsets or
// deletes. Coalesce modifies the provided span's Keys slice, reslicing the
// slice to remove dropped keys.
//
// Coalescence has subtle behavior with respect to sequence numbers. Coalesce
// depends on a keyspan.Span's Keys being sorted in sequence number descending
// order. The first key has the largest sequence number. The returned coalesced
// span includes only the largest sequence number. All other sequence numbers
// are forgotten. When a compaction constructs output range keys from a
// coalesced span, it produces at most one RANGEKEYSET, one RANGEKEYUNSET and
// one RANGEKEYDEL. Each one of these keys adopt the largest sequence number.
//
// This has the potentially surprising effect of 'promoting' a key to a higher
// sequence number. This is okay, because:
//   - There are no other overlapping keys within the coalesced span of
//     sequence numbers (otherwise they would be in the compaction, due to
//     the LSM invariant).
//   - Range key sequence numbers are never compared to point key sequence
//     numbers. Range keys and point keys have parallel existences.
//   - Compactions only coalesce within snapshot stripes.
//
// Additionally, internal range keys at the same sequence number have subtle
// mechanics:
//   - RANGEKEYSETs shadow RANGEKEYUNSETs of the same suffix.
//   - RANGEKEYDELs only apply to keys at lower sequence numbers.
//
// This is required for ingestion. Ingested sstables are assigned a single
// sequence number for the file, at which all of the file's keys are visible.
// The RANGEKEYSET, RANGEKEYUNSET and RANGEKEYDEL key kinds are ordered such
// that among keys with equal sequence numbers (thus ordered by their kinds) the
// keys do not affect one another. Ingested sstables are expected to be
// consistent with respect to the set/unset suffixes: A given suffix should be
// set or unset but not both.
//
// The resulting dst Keys slice is sorted by Trailer.
func Coalesce(cmp base.Compare, eq base.Equal, keys []keyspan.Key, dst *[]keyspan.Key) error {
	// TODO(jackson): Currently, Coalesce doesn't actually perform the sequence
	// number promotion described in the comment above.
	keysBySuffix := keyspan.KeysBySuffix{
		Cmp:  cmp,
		Keys: (*dst)[:0],
	}
	if err := coalesce(eq, &keysBySuffix, math.MaxUint64, keys); err != nil {
		return err
	}
	// Update the span with the (potentially reduced) keys slice. coalesce left
	// the keys in *dst sorted by suffix. Re-sort them by trailer.
	*dst = keysBySuffix.Keys
	keyspan.SortKeysByTrailer(dst)
	return nil
}

func coalesce(
	equal base.Equal, keysBySuffix *keyspan.KeysBySuffix, snapshot uint64, keys []keyspan.Key,
) error {
	// First, enforce visibility and RangeKeyDelete mechanics. We only need to
	// consider the prefix of keys before and including the first
	// RangeKeyDelete. We also must skip any keys that aren't visible at the
	// provided snapshot sequence number.
	//
	// NB: Within a given sequence number, keys are ordered as:
	//   RangeKeySet > RangeKeyUnset > RangeKeyDelete
	// This is significant, because this ensures that a Set or Unset sharing a
	// sequence number with a Delete do not shadow each other.
	deleteIdx := -1
	for i := range keys {
		if invariants.Enabled && i > 0 && keys[i].Trailer > keys[i-1].Trailer {
			panic("pebble: invariant violation: span keys unordered")
		}
		if !keys[i].VisibleAt(snapshot) {
			continue
		}
		// Once a RangeKeyDelete is observed, we know it shadows all subsequent
		// keys and we can break early. We don't add the RangeKeyDelete key to
		// keysBySuffix.keys yet, because we don't want a suffix-less key
		// that appeared earlier in the slice to elide it. It'll be added back
		// in at the end.
		if keys[i].Kind() == base.InternalKeyKindRangeKeyDelete {
			deleteIdx = i
			break
		}
		keysBySuffix.Keys = append(keysBySuffix.Keys, keys[i])
	}

	// Sort the accumulated keys by suffix. There may be duplicates within a
	// suffix, in which case the one with a larger trailer survives.
	//
	// We use a stable sort so that the first key with a given suffix is the one
	// that with the highest Trailer (because the input `keys` was sorted by
	// trailer descending).
	sort.Stable(keysBySuffix)

	// Grab a handle of the full sorted slice, before reslicing
	// keysBySuffix.keys to accumulate the final coalesced keys.
	sorted := keysBySuffix.Keys
	keysBySuffix.Keys = keysBySuffix.Keys[:0]

	var (
		// prevSuffix is updated on each iteration of the below loop, and
		// compared by the subsequent iteration to determine whether adjacent
		// keys are defined at the same suffix.
		prevSuffix []byte
		// shadowing is set to true once any Key is shadowed by another key.
		// When it's set to true—or after the loop if no keys are shadowed—the
		// keysBySuffix.keys slice is resliced to contain the prefix of
		// unshadowed keys. This avoids copying them incrementally in the common
		// case of no shadowing.
		shadowing bool
	)
	for i := range sorted {
		if i > 0 && equal(prevSuffix, sorted[i].Suffix) {
			// Skip; this key is shadowed by the predecessor that had a larger
			// Trailer. If this is the first shadowed key, set shadowing=true
			// and reslice keysBySuffix.keys to hold the entire unshadowed
			// prefix.
			if !shadowing {
				keysBySuffix.Keys = keysBySuffix.Keys[:i]
				shadowing = true
			}
			continue
		}
		prevSuffix = sorted[i].Suffix
		if shadowing {
			keysBySuffix.Keys = append(keysBySuffix.Keys, sorted[i])
		}
	}
	// If there was no shadowing, keysBySuffix.keys is untouched. We can simply
	// set it to the existing `sorted` slice (also backed by keysBySuffix.keys).
	if !shadowing {
		keysBySuffix.Keys = sorted
	}
	// If the original input `keys` slice contained a RangeKeyDelete, add it.
	if deleteIdx >= 0 {
		keysBySuffix.Keys = append(keysBySuffix.Keys, keys[deleteIdx])
	}
	return nil
}
