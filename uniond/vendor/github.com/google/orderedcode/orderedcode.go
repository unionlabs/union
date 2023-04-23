// Copyright 2015 Google Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/*
Package orderedcode provides a byte encoding of a sequence of typed items.
The resulting bytes can be lexicographically compared to yield the same
ordering as item-wise comparison on the original sequences.

More precisely, suppose:
  - A is the encoding of the sequence of items [A_1..A_n],
  - B is the encoding of the sequence of items [B_1..B_n],
  - For each i, A_i and B_i have the same type.
Then comparing A versus B lexicographically is the same as comparing the
vectors [A_1..A_n] and [B_1..B_n] lexicographically.

Furthermore, if i < j then [A_1..A_i]'s encoding is a prefix of [A_1..A_j]'s
encoding.

The order-maintaining and prefix properties described above are useful for
generating keys for databases like Bigtable.

Call Append(buffer, item1, ..., itemN) to construct the encoded bytes. The
valid item types are:
  - string,
  - struct{}, which is an 'infinity' that sorts greater than any other string,
  - orderedcode.StringOrInfinity, which is a union type,
  - TrailingString,
  - float64,
  - int64,
  - uint64.

As a convenience, orderedcode.Infinity is a value of type struct{}. For
example, to encode a sequence of two strings, an 'infinity' and an uint64:
	buf, err := orderedcode.Append(
		nil, "foo", "bar", orderedcode.Infinity, uint64(42))
	if err != nil {
		return err
	}
	key := string(buf)

Alternatively, encoding can be done in multiple steps:
	var buf []byte
	// Ignore errors, for demonstration purposes.
	buf, _ = orderedcode.Append(buf, "foo")
	buf, _ = orderedcode.Append(buf, "bar")
	buf, _ = orderedcode.Append(buf, orderedcode.Infinity)
	buf, _ = orderedcode.Append(buf, uint64(42))
	key := string(buf)

Call Parse(encoded, &item1, ..., &itemN) to deconstruct an encoded string.
The valid argument types are the pointers to the valid encoding types. For
example:
	var (
		s1, s2    string
		infinity3 struct{}
		u4        uint64
	)
	remainingKey, err := orderedcode.Parse(key, &s1, &s2, &infinity3, &u4)

Alternatively:
	var (
		x1, x2, x3 orderedcode.StringOrInfinity
		u4         uint64
	)
	remainingKey, err := orderedcode.Parse(key, &x1, &x2, &x3, &u4)

A TrailingString is a string that, if present, must be the last item appended
or parsed. It is not mandatory to use a TrailingString; it is valid for the
last item to be a standard string or any other type listed above. A
TrailingString simply allows a more efficient encoding while retaining the
lexicographic order-maintaining property. If used, you cannot append a
TrailingString and parse the result as a standard string, or as a
StringOrInfinity. For example:
	key, err := orderedcode.Append(
		nil, "first", "middle", orderedcode.TrailingString("last"))
	if err != nil {
		return err
	}
	var (
		s1, s2 string
		t3     orderedcode.TrailingString
	)
	remainingKey, err := orderedcode.Parse(string(key), &s1, &s2, &t3)
	if err != nil {
		return err
	}
	fmt.Printf("trailing string: got s1=%q s2=%q t3=%q\n", s1, s2, t3)

The same sequence of types should be used for encoding and decoding (although
StringOrInfinity can substitute for either a string or a struct{}, but not for
a TrailingString). The wire format is not fully self-describing:
"\x00\x01\x04\x03\x02\x00\x01" is a valid encoding of both ["", "\x04\x03\x02"]
and [uint64(0), uint64(4), uint64(0x20001)]. Decoding into a pointer of the
wrong type may return corrupt data and no error.

Each item can optionally be encoded in decreasing order. If the i'th item is
and the lexicographic comparison of A and B comes down to A_i versus B_i, then
A < B will equal A_i > B_i.

To encode in decreasing order, wrap the item in an orderedcode.Decr value. To
decode, wrap the item pointer in an orderedcode.Decr. For example:
	key, err := orderedcode.Append(nil, "foo", orderedcode.Decr("bar"))
	if err != nil {
		return err
	}
	var s1, s2 string
	_, err := orderedcode.Parse(string(key), &s1, orderedcode.Decr(&s2))
	if err != nil {
		return err
	}
	fmt.Printf("round trip: got s1=%q s2=%q\n", s1, s2)

Each item's ordering is independent from other items, but the same ordering
should be used to encode and decode the i'th item.
*/
package orderedcode // import "github.com/google/orderedcode"

import (
	"errors"
	"fmt"
	"math"
)

func invert(b []byte) {
	for i := range b {
		b[i] ^= 0xff
	}
}

// Infinity is an encodable value that sorts greater than any other string.
var Infinity struct{}

// StringOrInfinity is a union type. If Infinity is true, String must be "",
// and the value represents an 'infinity' that is greater than any other
// string. Otherwise, the value is the String string.
type StringOrInfinity struct {
	String   string
	Infinity bool
}

// TrailingString is a string that, if present, must be the last item appended
// or parsed.
type TrailingString string

// Decr wraps a value so that it is encoded or decoded in decreasing order.
func Decr(val interface{}) interface{} {
	return decr{val}
}

type decr struct {
	val interface{}
}

// Append appends the encoded representations of items to buf. Items can have
// different underlying types, but each item must have type T or be the value
// Decr(somethingOfTypeT), for T in the set: string, struct{}, StringOrInfinity,
// TrailingString, float64, int64 or uint64.
func Append(buf []byte, items ...interface{}) ([]byte, error) {
	for _, item := range items {
		n := len(buf)
		d, dOK := item.(decr)
		if dOK {
			item = d.val
		}

		switch x := item.(type) {
		case string:
			buf = appendString(buf, x)
		case struct{}:
			buf = appendInfinity(buf)
		case StringOrInfinity:
			if x.Infinity {
				if x.String != "" {
					return nil, errors.New("orderedcode: StringOrInfinity has non-zero String and non-zero Infinity")
				}
				buf = appendInfinity(buf)
			} else {
				buf = appendString(buf, x.String)
			}
		case TrailingString:
			buf = append(buf, string(x)...)
		case float64:
			buf = appendFloat64(buf, x)
		case int64:
			buf = appendInt64(buf, x)
		case uint64:
			buf = appendUint64(buf, x)
		default:
			return nil, fmt.Errorf("orderedcode: cannot append an item of type %T", item)
		}

		if dOK {
			invert(buf[n:])
		}
	}
	return buf, nil
}

// The wire format for strings or infinity is:
//   - \x00\x01 terminates the string.
//   - \x00 bytes are escaped as \x00\xff.
//   - \xff bytes are escaped as \xff\x00.
//   - \xff\xff encodes 'infinity'.
//   - All other bytes are literals.
const (
	term  = "\x00\x01"
	lit00 = "\x00\xff"
	litff = "\xff\x00"
	inf   = "\xff\xff"
)

func appendString(s []byte, x string) []byte {
	last := 0
	for i := 0; i < len(x); i++ {
		switch x[i] {
		case 0x00:
			s = append(s, x[last:i]...)
			s = append(s, lit00...)
			last = i + 1
		case 0xff:
			s = append(s, x[last:i]...)
			s = append(s, litff...)
			last = i + 1
		}
	}
	s = append(s, x[last:]...)
	s = append(s, term...)
	return s
}

func appendInfinity(s []byte) []byte {
	return append(s, inf...)
}

// The wire format for a float64 value x is, for positive x, the encoding of
// the 64 bits (in IEEE 754 format) re-interpreted as an int64. For negative
// x, we keep the sign bit and invert all other bits. Negative zero is a
// special case that encodes the same as positive zero.

func appendFloat64(s []byte, x float64) []byte {
	i := int64(math.Float64bits(x))
	if i < 0 {
		i = math.MinInt64 - i
	}
	return appendInt64(s, i)
}

// The wire format for an int64 value x is, for non-negative x, n leading 1
// bits, followed by a 0 bit, followed by n-1 bytes. That entire slice, after
// masking off the leading 1 bits, is the big-endian representation of x.
// n is the smallest positive integer that can represent x this way.
//
// The encoding of a negative x is the inversion of the encoding for ^x.
// Thus, the encoded form's leading bit is a sign bit: it is 0 for negative x
// and 1 for non-negative x.
//
// For example:
//   - 0x23   encodes as 10 100011
//     n=0, the remainder after masking is 0x23.
//   - 0x10e  encodes as 110 00001  00001110
//     n=1, the remainder after masking is 0x10e.
//   - -0x10f encodes as 001 11110  11110001
//     This is the inverse of the encoding of 0x10e.
// There are many more examples in orderedcode_test.go.

func appendInt64(s []byte, x int64) []byte {
	// Fast-path those values of x that encode to a single byte.
	if x >= -64 && x < 64 {
		return append(s, uint8(x)^0x80)
	}
	// If x is negative, invert it, and correct for this at the end.
	neg := x < 0
	if neg {
		x = ^x
	}
	// x is now non-negative, and so its encoding starts with a 1: the sign bit.
	n := 1
	// buf is 8 bytes for x's big-endian representation plus 2 bytes for leading 1 bits.
	var buf [10]byte
	// Fill buf from back-to-front.
	i := 9
	for x > 0 {
		buf[i] = byte(x)
		n, i, x = n+1, i-1, x>>8
	}
	// Check if we need a full byte of leading 1 bits. 7 is 8 - 1; the 8 is the
	// number of bits in a byte, and the 1 is because lengthening the encoding
	// by one byte requires incrementing n.
	leadingFFByte := n > 7
	if leadingFFByte {
		n -= 7
	}
	// If we can squash the leading 1 bits together with x's most significant byte,
	// then we can save one byte.
	//
	// We need to adjust 8-n by -1 for the separating 0 bit, but also by
	// +1 because we are trying to get away with one fewer leading 1 bits.
	// The two adjustments cancel each other out.
	if buf[i+1] < 1<<uint(8-n) {
		n--
		i++
	}
	// Or in the leading 1 bits, invert if necessary, and return.
	buf[i] |= msb[n]
	if leadingFFByte {
		i--
		buf[i] = 0xff
	}
	if neg {
		invert(buf[i:])
	}
	return append(s, buf[i:]...)
}

// msb[i] is a byte whose first i bits (in most significant bit order) are 1
// and all other bits are 0.
var msb = [8]byte{
	0x00, 0x80, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc, 0xfe,
}

// The wire format for a uint64 value x is 1 byte (with value n) followed by n
// bytes being x's big-endian representation (dropping any leading zeroes).

func appendUint64(s []byte, x uint64) []byte {
	// buf is 8 bytes for value plus 1 byte for length.
	var buf [9]byte
	// Fill buf from back-to-front.
	i := 8
	for x > 0 {
		buf[i] = byte(x)
		i, x = i-1, x>>8
	}
	// The front-most byte is n, the number of bytes in the big-endian representation.
	buf[i] = byte(8 - i)
	return append(s, buf[i:]...)
}

// For decreasing order, the encoded bytes are the bitwise-not of the regular
// encoding. Bitwise-not of a byte is equivalent to bitwise-xor with 0xff, and
// bitwise-xor with 0x00 is a no-op.
const (
	increasing byte = 0x00
	decreasing byte = 0xff
)

// errCorrupt is returned from Parse if the input cannot be decoded into the
// requested types.
var errCorrupt = errors.New("orderedcode: corrupt input")

// Parse parses the next len(items) of their respective types and returns any
// remaining encoded data. Items can have different underlying types, but each
// item must have type *T or be the value Decr(somethingOfTypeStarT), for T in
// the set: string, struct{}, StringOrInfinity, TrailingString, float64, int64
// or uint64.
func Parse(encoded string, items ...interface{}) (remaining string, err error) {
	for _, item := range items {
		dir := increasing
		if d, dOK := item.(decr); dOK {
			dir, item = decreasing, d.val
		}
		switch x := item.(type) {
		case *string:
			encoded, err = parseString(x, encoded, dir)
		case *struct{}:
			encoded, err = parseInfinity(encoded, dir)
		case *StringOrInfinity:
			if rem, err1 := parseInfinity(encoded, dir); err1 == nil {
				*x = StringOrInfinity{Infinity: true}
				encoded = rem
			} else {
				var s string
				encoded, err = parseString(&s, encoded, dir)
				if err == nil {
					*x = StringOrInfinity{String: s}
				}
			}
		case *TrailingString:
			if dir == increasing {
				*x, encoded = TrailingString(encoded), ""
			} else {
				b := []byte(encoded)
				invert(b)
				*x, encoded = TrailingString(b), ""
			}
		case *float64:
			encoded, err = parseFloat64(x, encoded, dir)
		case *int64:
			encoded, err = parseInt64(x, encoded, dir)
		case *uint64:
			encoded, err = parseUint64(x, encoded, dir)
		default:
			return "", fmt.Errorf("orderedcode: cannot parse an item of type %T", item)
		}
		if err != nil {
			return "", err
		}
	}
	return encoded, nil
}

func parseString(dst *string, s string, dir byte) (string, error) {
	var (
		buf     []byte
		last, i int
	)
	for i < len(s) {
		switch v := s[i] ^ dir; v {
		case 0x00:
			if i+1 >= len(s) {
				return "", errCorrupt
			}
			switch s[i+1] ^ dir {
			case 0x01:
				// The terminator mark ends the string.
				if last == 0 && dir == increasing {
					// As an optimization, if no \x00 or \xff bytes were escaped,
					// and the result does not need inverting, then set *dst to a
					// sub-string of the original input.
					*dst = s[:i]
					return s[i+2:], nil
				}
				buf = append(buf, s[last:i]...)
				if dir != increasing {
					invert(buf)
				}
				*dst = string(buf)
				return s[i+2:], nil
			case 0xff:
				// Unescape the \x00.
				buf = append(buf, s[last:i]...)
				buf = append(buf, 0x00^dir)
				i += 2
				last = i
			default:
				return "", errCorrupt
			}
		case 0xff:
			if i+1 >= len(s) || s[i+1]^dir != 0x00 {
				return "", errCorrupt
			}
			// Unescape the \xff.
			buf = append(buf, s[last:i]...)
			buf = append(buf, 0xff^dir)
			i += 2
			last = i
		default:
			i++
		}
	}
	return "", errCorrupt
}

func parseInfinity(s string, dir byte) (string, error) {
	if len(s) < 2 {
		return "", errCorrupt
	}
	for i := 0; i < 2; i++ {
		if s[i]^dir != inf[i] {
			return "", errCorrupt
		}
	}
	return s[2:], nil
}

func parseFloat64(dst *float64, s string, dir byte) (string, error) {
	var i int64
	s, err := parseInt64(&i, s, dir)
	if err != nil {
		return "", err
	}
	if i < 0 {
		i = math.MinInt64 - i
	}
	*dst = math.Float64frombits(uint64(i))
	return s, nil
}

func parseInt64(dst *int64, s string, dir byte) (string, error) {
	if len(s) == 0 {
		return "", errCorrupt
	}
	// Fast-path any single-byte encoding.
	c := s[0] ^ dir
	if c >= 0x40 && c < 0xc0 {
		*dst = int64(int8(c ^ 0x80))
		return s[1:], nil
	}
	// Invert everything if the encoded value is negative.
	neg := c&0x80 == 0
	if neg {
		c, dir = ^c, ^dir
	}
	// Consume the leading 0xff full of 1 bits, if present.
	n := 0
	if c == 0xff {
		if len(s) == 1 {
			return "", errCorrupt
		}
		s = s[1:]
		c = s[0] ^ dir
		// The encoding of the largest int64 (1<<63-1) starts with "\xff\xc0".
		if c > 0xc0 {
			return "", errCorrupt
		}
		// The 7 (being 8 - 1) is for the same reason as in appendInt64.
		n = 7
	}
	// Count and mask off any remaining 1 bits.
	for mask := byte(0x80); c&mask != 0; mask >>= 1 {
		c &= ^mask
		n++
	}
	if len(s) < n {
		return "", errCorrupt
	}
	// Decode the big-endian, invert if necessary, and return.
	x := int64(c)
	for i := 1; i < n; i++ {
		c = s[i] ^ dir
		x = x<<8 | int64(c)
	}
	if neg {
		x = ^x
	}
	*dst = x
	return s[n:], nil
}

func parseUint64(dst *uint64, s string, dir byte) (string, error) {
	if len(s) == 0 {
		return "", errCorrupt
	}
	n := int(s[0] ^ dir)
	if n > 8 || len(s) < 1+n {
		return "", errCorrupt
	}
	x := uint64(0)
	for i := 0; i < n; i++ {
		x = x<<8 | uint64(s[1+i]^dir)
	}
	*dst = x
	return s[1+n:], nil
}
