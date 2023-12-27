// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan // import "github.com/cockroachdb/pebble/internal/keyspan"

import (
	"bytes"
	"fmt"
	"sort"
	"strconv"
	"strings"
	"unicode"

	"github.com/cockroachdb/pebble/internal/base"
)

// Span represents a set of keys over a span of user key space. All of the keys
// within a Span are applied across the span's key span indicated by Start and
// End. Each internal key applied over the user key span appears as a separate
// Key, with its own kind and sequence number. Optionally, each Key may also
// have a Suffix and/or Value.
//
// Note that the start user key is inclusive and the end user key is exclusive.
//
// Currently the only supported key kinds are:
//
//	RANGEDEL, RANGEKEYSET, RANGEKEYUNSET, RANGEKEYDEL.
type Span struct {
	// Start and End encode the user key range of all the contained items, with
	// an inclusive start key and exclusive end key. Both Start and End must be
	// non-nil, or both nil if representing an invalid Span.
	Start, End []byte
	// Keys holds the set of keys applied over the [Start, End) user key range.
	// Keys is sorted by (SeqNum, Kind) descending, unless otherwise specified
	// by the context. If SeqNum and Kind are equal, the order of Keys is
	// undefined. Keys may be empty, even if Start and End are non-nil.
	//
	// Keys are a decoded representation of the internal keys stored in batches
	// or sstable blocks. A single internal key in a range key block may produce
	// several decoded Keys.
	Keys      []Key
	KeysOrder KeysOrder
}

// KeysOrder describes the ordering of Keys within a Span.
type KeysOrder int8

const (
	// ByTrailerDesc indicates a Span's keys are sorted by Trailer descending.
	// This is the default ordering, and the ordering used during physical
	// storage.
	ByTrailerDesc KeysOrder = iota
	// BySuffixAsc indicates a Span's keys are sorted by Suffix ascending. This
	// ordering is used during user iteration of range keys.
	BySuffixAsc
)

// Key represents a single key applied over a span of user keys. A Key is
// contained by a Span which specifies the span of user keys over which the Key
// is applied.
type Key struct {
	// Trailer contains the key kind and sequence number.
	Trailer uint64
	// Suffix holds an optional suffix associated with the key. This is only
	// non-nil for RANGEKEYSET and RANGEKEYUNSET keys.
	Suffix []byte
	// Value holds a logical value associated with the Key. It is NOT the
	// internal value stored in a range key or range deletion block.  This is
	// only non-nil for RANGEKEYSET keys.
	Value []byte
}

// SeqNum returns the sequence number component of the key.
func (k Key) SeqNum() uint64 {
	return k.Trailer >> 8
}

// VisibleAt returns true if the provided key is visible at the provided
// snapshot sequence number. It interprets batch sequence numbers as always
// visible, because non-visible batch span keys are filtered when they're
// fragmented.
func (k Key) VisibleAt(snapshot uint64) bool {
	seq := k.SeqNum()
	return seq < snapshot || seq&base.InternalKeySeqNumBatch != 0
}

// Kind returns the kind component of the key.
func (k Key) Kind() base.InternalKeyKind {
	return base.InternalKeyKind(k.Trailer & 0xff)
}

// Equal returns true if this Key is equal to the given key. Two keys are said
// to be equal if the two Keys have equal trailers, suffix and value. Suffix
// comparison uses the provided base.Compare func. Value comparison is bytewise.
func (k Key) Equal(equal base.Equal, b Key) bool {
	return k.Trailer == b.Trailer &&
		equal(k.Suffix, b.Suffix) &&
		bytes.Equal(k.Value, b.Value)
}

// Valid returns true if the span is defined.
func (s *Span) Valid() bool {
	return s.Start != nil && s.End != nil
}

// Empty returns true if the span does not contain any keys. An empty span may
// still be Valid. A non-empty span must be Valid.
//
// An Empty span may be produced by Visible, or be produced by iterators in
// order to surface the gaps between keys.
func (s *Span) Empty() bool {
	return s == nil || len(s.Keys) == 0
}

// SmallestKey returns the smallest internal key defined by the span's keys.
// It requires the Span's keys be in ByTrailerDesc order. It panics if the span
// contains no keys or its keys are sorted in a different order.
func (s *Span) SmallestKey() base.InternalKey {
	if len(s.Keys) == 0 {
		panic("pebble: Span contains no keys")
	} else if s.KeysOrder != ByTrailerDesc {
		panic("pebble: span's keys unexpectedly not in trailer order")
	}
	// The first key has the highest (sequence number,kind) tuple.
	return base.InternalKey{
		UserKey: s.Start,
		Trailer: s.Keys[0].Trailer,
	}
}

// LargestKey returns the largest internal key defined by the span's keys. The
// returned key will always be a "sentinel key" at the end boundary. The
// "sentinel key" models the exclusive end boundary by returning an InternalKey
// with the maximal sequence number, ensuring all InternalKeys with the same
// user key sort after the sentinel key.
//
// It requires the Span's keys be in ByTrailerDesc order. It panics if the span
// contains no keys or its keys are sorted in a different order.
func (s *Span) LargestKey() base.InternalKey {
	if len(s.Keys) == 0 {
		panic("pebble: Span contains no keys")
	} else if s.KeysOrder != ByTrailerDesc {
		panic("pebble: span's keys unexpectedly not in trailer order")
	}
	// The last key has the lowest (sequence number,kind) tuple.
	kind := s.Keys[len(s.Keys)-1].Kind()
	return base.MakeExclusiveSentinelKey(kind, s.End)
}

// SmallestSeqNum returns the smallest sequence number of a key contained within
// the span. It requires the Span's keys be in ByTrailerDesc order. It panics if
// the span contains no keys or its keys are sorted in a different order.
func (s *Span) SmallestSeqNum() uint64 {
	if len(s.Keys) == 0 {
		panic("pebble: Span contains no keys")
	} else if s.KeysOrder != ByTrailerDesc {
		panic("pebble: span's keys unexpectedly not in trailer order")
	}

	return s.Keys[len(s.Keys)-1].SeqNum()
}

// LargestSeqNum returns the largest sequence number of a key contained within
// the span. It requires the Span's keys be in ByTrailerDesc order. It panics if
// the span contains no keys or its keys are sorted in a different order.
func (s *Span) LargestSeqNum() uint64 {
	if len(s.Keys) == 0 {
		panic("pebble: Span contains no keys")
	} else if s.KeysOrder != ByTrailerDesc {
		panic("pebble: span's keys unexpectedly not in trailer order")
	}
	return s.Keys[0].SeqNum()
}

// TODO(jackson): Replace most of the calls to Visible with more targeted calls
// that avoid the need to construct a new Span.

// Visible returns a span with the subset of keys visible at the provided
// sequence number. It requires the Span's keys be in ByTrailerDesc order. It
// panics if the span's keys are sorted in a different order.
//
// Visible may incur an allocation, so callers should prefer targeted,
// non-allocating methods when possible.
func (s Span) Visible(snapshot uint64) Span {
	if s.KeysOrder != ByTrailerDesc {
		panic("pebble: span's keys unexpectedly not in trailer order")
	}

	ret := Span{Start: s.Start, End: s.End}
	if len(s.Keys) == 0 {
		return ret
	}

	// Keys from indexed batches may force an allocation. The Keys slice is
	// ordered by sequence number, so ordinarily we can return the trailing
	// subslice containing keys with sequence numbers less than `seqNum`.
	//
	// However, batch keys are special. Only visible batch keys are included
	// when an Iterator's batch spans are fragmented. They must always be
	// visible.
	//
	// Batch keys can create a sandwich of visible batch keys at the beginning
	// of the slice and visible committed keys at the end of the slice, forcing
	// us to allocate a new slice and copy the contents.
	//
	// Care is taking to only incur an allocation only when batch keys and
	// visible keys actually sandwich non-visible keys.

	// lastBatchIdx and lastNonVisibleIdx are set to the last index of a batch
	// key and a non-visible key respectively.
	lastBatchIdx := -1
	lastNonVisibleIdx := -1
	for i := range s.Keys {
		if seqNum := s.Keys[i].SeqNum(); seqNum&base.InternalKeySeqNumBatch != 0 {
			// Batch key. Always visible.
			lastBatchIdx = i
		} else if seqNum >= snapshot {
			// This key is not visible.
			lastNonVisibleIdx = i
		}
	}

	// In the following comments: b = batch, h = hidden, v = visible (committed).
	switch {
	case lastNonVisibleIdx == -1:
		// All keys are visible.
		//
		// [b b b], [v v v] and [b b b v v v]
		ret.Keys = s.Keys
	case lastBatchIdx == -1:
		// There are no batch keys, so we can return the continuous subslice
		// starting after the last non-visible Key.
		//
		// h h h [v v v]
		ret.Keys = s.Keys[lastNonVisibleIdx+1:]
	case lastNonVisibleIdx == len(s.Keys)-1:
		// While we have a batch key and non-visible keys, there are no
		// committed visible keys. The 'sandwich' is missing the bottom layer,
		// so we can return the continuous sublice at the beginning.
		//
		// [b b b] h h h
		ret.Keys = s.Keys[0 : lastBatchIdx+1]
	default:
		// This is the problematic sandwich case. Allocate a new slice, copying
		// the batch keys and the visible keys into it.
		//
		// [b b b] h h h [v v v]
		ret.Keys = make([]Key, (lastBatchIdx+1)+(len(s.Keys)-lastNonVisibleIdx-1))
		copy(ret.Keys, s.Keys[:lastBatchIdx+1])
		copy(ret.Keys[lastBatchIdx+1:], s.Keys[lastNonVisibleIdx+1:])
	}
	return ret
}

// VisibleAt returns true if the span contains a key visible at the provided
// snapshot. Keys with sequence numbers with the batch bit set are treated as
// always visible.
//
// VisibleAt requires the Span's keys be in ByTrailerDesc order. It panics if
// the span's keys are sorted in a different order.
func (s *Span) VisibleAt(snapshot uint64) bool {
	if s.KeysOrder != ByTrailerDesc {
		panic("pebble: span's keys unexpectedly not in trailer order")
	}
	if len(s.Keys) == 0 {
		return false
	} else if first := s.Keys[0].SeqNum(); first&base.InternalKeySeqNumBatch != 0 {
		// Only visible batch keys are included when an Iterator's batch spans
		// are fragmented. They must always be visible.
		return true
	} else {
		// Otherwise we check the last key. Since keys are ordered decreasing in
		// sequence number, the last key has the lowest sequence number of any
		// of the span's keys. If any of the keys are visible, the last key must
		// be visible. Or put differently: if the last key is not visible, then
		// no key is visible.
		return s.Keys[len(s.Keys)-1].SeqNum() < snapshot
	}
}

// ShallowClone returns the span with a Keys slice owned by the span itself.
// None of the key byte slices are cloned (see Span.DeepClone).
func (s *Span) ShallowClone() Span {
	c := Span{
		Start:     s.Start,
		End:       s.End,
		Keys:      make([]Key, len(s.Keys)),
		KeysOrder: s.KeysOrder,
	}
	copy(c.Keys, s.Keys)
	return c
}

// DeepClone clones the span, creating copies of all contained slices. DeepClone
// is intended for non-production code paths like tests, the level checker, etc
// because it is allocation heavy.
func (s *Span) DeepClone() Span {
	c := Span{
		Start:     make([]byte, len(s.Start)),
		End:       make([]byte, len(s.End)),
		Keys:      make([]Key, len(s.Keys)),
		KeysOrder: s.KeysOrder,
	}
	copy(c.Start, s.Start)
	copy(c.End, s.End)
	for i := range s.Keys {
		c.Keys[i].Trailer = s.Keys[i].Trailer
		if len(s.Keys[i].Suffix) > 0 {
			c.Keys[i].Suffix = make([]byte, len(s.Keys[i].Suffix))
			copy(c.Keys[i].Suffix, s.Keys[i].Suffix)
		}
		if len(s.Keys[i].Value) > 0 {
			c.Keys[i].Value = make([]byte, len(s.Keys[i].Value))
			copy(c.Keys[i].Value, s.Keys[i].Value)
		}
	}
	return c
}

// Contains returns true if the specified key resides within the span's bounds.
func (s *Span) Contains(cmp base.Compare, key []byte) bool {
	return cmp(s.Start, key) <= 0 && cmp(key, s.End) < 0
}

// Covers returns true if the span covers keys at seqNum.
//
// Covers requires the Span's keys be in ByTrailerDesc order. It panics if the
// span's keys are sorted in a different order.
func (s Span) Covers(seqNum uint64) bool {
	if s.KeysOrder != ByTrailerDesc {
		panic("pebble: span's keys unexpectedly not in trailer order")
	}
	return !s.Empty() && s.Keys[0].SeqNum() > seqNum
}

// CoversAt returns true if the span contains a key that is visible at the
// provided snapshot sequence number, and that key's sequence number is higher
// than seqNum.
//
// Keys with sequence numbers with the batch bit set are treated as always
// visible.
//
// CoversAt requires the Span's keys be in ByTrailerDesc order. It panics if the
// span's keys are sorted in a different order.
func (s *Span) CoversAt(snapshot, seqNum uint64) bool {
	if s.KeysOrder != ByTrailerDesc {
		panic("pebble: span's keys unexpectedly not in trailer order")
	}
	// NB: A key is visible at `snapshot` if its sequence number is strictly
	// less than `snapshot`. See base.Visible.
	for i := range s.Keys {
		if kseq := s.Keys[i].SeqNum(); kseq&base.InternalKeySeqNumBatch != 0 {
			// Only visible batch keys are included when an Iterator's batch spans
			// are fragmented. They must always be visible.
			return kseq > seqNum
		} else if kseq < snapshot {
			return kseq > seqNum
		}
	}
	return false
}

// String returns a string representation of the span.
func (s Span) String() string {
	return fmt.Sprint(prettySpan{Span: s, formatKey: base.DefaultFormatter})
}

// Pretty returns a formatter for the span.
func (s Span) Pretty(f base.FormatKey) fmt.Formatter {
	// TODO(jackson): Take a base.FormatValue to format Key.Value too.
	return prettySpan{s, f}
}

type prettySpan struct {
	Span
	formatKey base.FormatKey
}

func (s prettySpan) Format(fs fmt.State, c rune) {
	if !s.Valid() {
		fmt.Fprintf(fs, "<invalid>")
		return
	}
	fmt.Fprintf(fs, "%s-%s:{", s.formatKey(s.Start), s.formatKey(s.End))
	for i, k := range s.Keys {
		if i > 0 {
			fmt.Fprint(fs, " ")
		}
		fmt.Fprintf(fs, "(#%d,%s", k.SeqNum(), k.Kind())
		if len(k.Suffix) > 0 || len(k.Value) > 0 {
			fmt.Fprintf(fs, ",%s", k.Suffix)
		}
		if len(k.Value) > 0 {
			fmt.Fprintf(fs, ",%s", k.Value)
		}
		fmt.Fprint(fs, ")")
	}
	fmt.Fprintf(fs, "}")
}

// SortKeysByTrailer sorts a keys slice by trailer.
func SortKeysByTrailer(keys *[]Key) {
	// NB: keys is a pointer to a slice instead of a slice to avoid `sorted`
	// escaping to the heap.
	sorted := (*keysBySeqNumKind)(keys)
	sort.Sort(sorted)
}

// KeysBySuffix implements sort.Interface, sorting its member Keys slice to by
// Suffix in the order dictated by Cmp.
type KeysBySuffix struct {
	Cmp  base.Compare
	Keys []Key
}

func (s *KeysBySuffix) Len() int           { return len(s.Keys) }
func (s *KeysBySuffix) Less(i, j int) bool { return s.Cmp(s.Keys[i].Suffix, s.Keys[j].Suffix) < 0 }
func (s *KeysBySuffix) Swap(i, j int)      { s.Keys[i], s.Keys[j] = s.Keys[j], s.Keys[i] }

// ParseSpan parses the string representation of a Span. It's intended for
// tests. ParseSpan panics if passed a malformed span representation.
func ParseSpan(input string) Span {
	var s Span
	parts := strings.FieldsFunc(input, func(r rune) bool {
		switch r {
		case '-', ':', '{', '}':
			return true
		default:
			return unicode.IsSpace(r)
		}
	})
	s.Start, s.End = []byte(parts[0]), []byte(parts[1])

	// Each of the remaining parts represents a single Key.
	s.Keys = make([]Key, 0, len(parts)-2)
	for _, p := range parts[2:] {
		keyFields := strings.FieldsFunc(p, func(r rune) bool {
			switch r {
			case '#', ',', '(', ')':
				return true
			default:
				return unicode.IsSpace(r)
			}
		})

		var k Key
		// Parse the sequence number.
		seqNum, err := strconv.ParseUint(keyFields[0], 10, 64)
		if err != nil {
			panic(fmt.Sprintf("invalid sequence number: %q: %s", keyFields[0], err))
		}
		// Parse the key kind.
		kind := base.ParseKind(keyFields[1])
		k.Trailer = base.MakeTrailer(seqNum, kind)
		// Parse the optional suffix.
		if len(keyFields) >= 3 {
			k.Suffix = []byte(keyFields[2])
		}
		// Parse the optional value.
		if len(keyFields) >= 4 {
			k.Value = []byte(keyFields[3])
		}
		s.Keys = append(s.Keys, k)
	}
	return s
}
