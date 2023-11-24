// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"strconv"
	"unicode/utf8"
)

// Compare returns -1, 0, or +1 depending on whether a is 'less than', 'equal
// to' or 'greater than' b. The two arguments can only be 'equal' if their
// contents are exactly equal. Furthermore, the empty slice must be 'less than'
// any non-empty slice. Compare is used to compare user keys, such as those
// passed as arguments to the various DB methods, as well as those returned
// from Separator, Successor, and Split.
type Compare func(a, b []byte) int

// Equal returns true if a and b are equivalent. For a given Compare,
// Equal(a,b) must return true iff Compare(a,b) returns zero, that is,
// Equal is a (potentially faster) specialization of Compare.
type Equal func(a, b []byte) bool

// AbbreviatedKey returns a fixed length prefix of a user key such that AbbreviatedKey(a)
// < AbbreviatedKey(b) iff a < b and AbbreviatedKey(a) > AbbreviatedKey(b) iff a > b. If
// AbbreviatedKey(a) == AbbreviatedKey(b) an additional comparison is required to
// determine if the two keys are actually equal.
//
// This helps optimize indexed batch comparisons for cache locality. If a Split
// function is specified, AbbreviatedKey usually returns the first eight bytes
// of the user key prefix in the order that gives the correct ordering.
type AbbreviatedKey func(key []byte) uint64

// FormatKey returns a formatter for the user key.
type FormatKey func(key []byte) fmt.Formatter

// FormatValue returns a formatter for the user value. The key is also
// specified for the value formatter in order to support value formatting that
// is dependent on the key.
type FormatValue func(key, value []byte) fmt.Formatter

// Separator is used to construct SSTable index blocks. A trivial implementation
// is `return a`, but appending fewer bytes leads to smaller SSTables.
//
// Given keys a, b for which Compare(a, b) < 0, Separator returns a key k such
// that:
//
// 1. Compare(a, k) <= 0, and
// 2. Compare(k, b) < 0.
//
// As a special case, b may be nil in which case the second condition is dropped.
//
// For example, if dst, a and b are the []byte equivalents of the strings
// "aqua", "black" and "blue", then the result may be "aquablb".
// Similarly, if the arguments were "aqua", "green" and "", then the result
// may be "aquah".
type Separator func(dst, a, b []byte) []byte

// Successor returns a shortened key given a key a, such that Compare(k, a) >=
// 0. A simple implementation may return a unchanged. The dst parameter may be
// used to store the returned key, though it is valid to pass nil. The returned
// key must be valid to pass to Compare.
type Successor func(dst, a []byte) []byte

// ImmediateSuccessor is invoked with a prefix key ([Split(a) == len(a)]) and
// returns the smallest key that is larger than the given prefix a.
// ImmediateSuccessor must return a prefix key k such that:
//
//	Split(k) == len(k) and Compare(k, a) > 0
//
// and there exists no representable k2 such that:
//
//	Split(k2) == len(k2) and Compare(k2, a) > 0 and Compare(k2, k) < 0
//
// As an example, an implementation built on the natural byte ordering using
// bytes.Compare could append a `\0` to `a`.
//
// The dst parameter may be used to store the returned key, though it is valid
// to pass nil. The returned key must be valid to pass to Compare.
type ImmediateSuccessor func(dst, a []byte) []byte

// Split returns the length of the prefix of the user key that corresponds to
// the key portion of an MVCC encoding scheme to enable the use of prefix bloom
// filters.
//
// The method will only ever be called with valid MVCC keys, that is, keys that
// the user could potentially store in the database. Pebble does not know which
// keys are MVCC keys and which are not, and may call Split on both MVCC keys
// and non-MVCC keys.
//
// A trivial MVCC scheme is one in which Split() returns len(a). This
// corresponds to assigning a constant version to each key in the database. For
// performance reasons, it is preferable to use a `nil` split in this case.
//
// The returned prefix must have the following properties:
//
//  1. The prefix must be a byte prefix:
//
//     bytes.HasPrefix(a, prefix(a))
//
//  2. A key consisting of just a prefix must sort before all other keys with
//     that prefix:
//
//     Compare(prefix(a), a) < 0 if len(suffix(a)) > 0
//
//  3. Prefixes must be used to order keys before suffixes:
//
//     If Compare(a, b) <= 0, then Compare(prefix(a), prefix(b)) <= 0
//
//  4. Suffixes themselves must be valid keys and comparable, respecting the same
//     ordering as within a key.
//
//     If Compare(prefix(a), prefix(b)) == 0, then Compare(suffix(a), suffix(b)) == Compare(a, b)
type Split func(a []byte) int

// Comparer defines a total ordering over the space of []byte keys: a 'less
// than' relationship.
type Comparer struct {
	Compare            Compare
	Equal              Equal
	AbbreviatedKey     AbbreviatedKey
	FormatKey          FormatKey
	FormatValue        FormatValue
	Separator          Separator
	Split              Split
	Successor          Successor
	ImmediateSuccessor ImmediateSuccessor

	// Name is the name of the comparer.
	//
	// The Level-DB on-disk format stores the comparer name, and opening a
	// database with a different comparer from the one it was created with
	// will result in an error.
	Name string
}

// DefaultFormatter is the default implementation of user key formatting:
// non-ASCII data is formatted as escaped hexadecimal values.
var DefaultFormatter = func(key []byte) fmt.Formatter {
	return FormatBytes(key)
}

// DefaultComparer is the default implementation of the Comparer interface.
// It uses the natural ordering, consistent with bytes.Compare.
var DefaultComparer = &Comparer{
	Compare: bytes.Compare,
	Equal:   bytes.Equal,

	AbbreviatedKey: func(key []byte) uint64 {
		if len(key) >= 8 {
			return binary.BigEndian.Uint64(key)
		}
		var v uint64
		for _, b := range key {
			v <<= 8
			v |= uint64(b)
		}
		return v << uint(8*(8-len(key)))
	},

	FormatKey: DefaultFormatter,

	Separator: func(dst, a, b []byte) []byte {
		i, n := SharedPrefixLen(a, b), len(dst)
		dst = append(dst, a...)

		min := len(a)
		if min > len(b) {
			min = len(b)
		}
		if i >= min {
			// Do not shorten if one string is a prefix of the other.
			return dst
		}

		if a[i] >= b[i] {
			// b is smaller than a or a is already the shortest possible.
			return dst
		}

		if i < len(b)-1 || a[i]+1 < b[i] {
			i += n
			dst[i]++
			return dst[:i+1]
		}

		i += n + 1
		for ; i < len(dst); i++ {
			if dst[i] != 0xff {
				dst[i]++
				return dst[:i+1]
			}
		}
		return dst
	},

	Successor: func(dst, a []byte) (ret []byte) {
		for i := 0; i < len(a); i++ {
			if a[i] != 0xff {
				dst = append(dst, a[:i+1]...)
				dst[len(dst)-1]++
				return dst
			}
		}
		// a is a run of 0xffs, leave it alone.
		return append(dst, a...)
	},

	ImmediateSuccessor: func(dst, a []byte) (ret []byte) {
		return append(append(dst, a...), 0x00)
	},

	// This name is part of the C++ Level-DB implementation's default file
	// format, and should not be changed.
	Name: "leveldb.BytewiseComparator",
}

// SharedPrefixLen returns the largest i such that a[:i] equals b[:i].
// This function can be useful in implementing the Comparer interface.
func SharedPrefixLen(a, b []byte) int {
	i, n := 0, len(a)
	if n > len(b) {
		n = len(b)
	}
	asUint64 := func(c []byte, i int) uint64 {
		return binary.LittleEndian.Uint64(c[i:])
	}
	for i < n-7 && asUint64(a, i) == asUint64(b, i) {
		i += 8
	}
	for i < n && a[i] == b[i] {
		i++
	}
	return i
}

// FormatBytes formats a byte slice using hexadecimal escapes for non-ASCII
// data.
type FormatBytes []byte

const lowerhex = "0123456789abcdef"

// Format implements the fmt.Formatter interface.
func (p FormatBytes) Format(s fmt.State, c rune) {
	buf := make([]byte, 0, len(p))
	for _, b := range p {
		if b < utf8.RuneSelf && strconv.IsPrint(rune(b)) {
			buf = append(buf, b)
			continue
		}
		buf = append(buf, `\x`...)
		buf = append(buf, lowerhex[b>>4])
		buf = append(buf, lowerhex[b&0xF])
	}
	s.Write(buf)
}
