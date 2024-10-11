// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import "encoding/binary"

const (
	maxVarintSize = 8 // encoded size in bytes
	maxVarint     = (1 << 62) - 1
)

// consumeVarint parses a variable-length integer, reporting its length.
// It returns a negative length upon an error.
//
// https://www.rfc-editor.org/rfc/rfc9000.html#section-16
func consumeVarint(b []byte) (v uint64, n int) {
	if len(b) < 1 {
		return 0, -1
	}
	b0 := b[0] & 0x3f
	switch b[0] >> 6 {
	case 0:
		return uint64(b0), 1
	case 1:
		if len(b) < 2 {
			return 0, -1
		}
		return uint64(b0)<<8 | uint64(b[1]), 2
	case 2:
		if len(b) < 4 {
			return 0, -1
		}
		return uint64(b0)<<24 | uint64(b[1])<<16 | uint64(b[2])<<8 | uint64(b[3]), 4
	case 3:
		if len(b) < 8 {
			return 0, -1
		}
		return uint64(b0)<<56 | uint64(b[1])<<48 | uint64(b[2])<<40 | uint64(b[3])<<32 | uint64(b[4])<<24 | uint64(b[5])<<16 | uint64(b[6])<<8 | uint64(b[7]), 8
	}
	return 0, -1
}

// consumeVarint64 parses a variable-length integer as an int64.
func consumeVarintInt64(b []byte) (v int64, n int) {
	u, n := consumeVarint(b)
	// QUIC varints are 62-bits large, so this conversion can never overflow.
	return int64(u), n
}

// appendVarint appends a variable-length integer to b.
//
// https://www.rfc-editor.org/rfc/rfc9000.html#section-16
func appendVarint(b []byte, v uint64) []byte {
	switch {
	case v <= 63:
		return append(b, byte(v))
	case v <= 16383:
		return append(b, (1<<6)|byte(v>>8), byte(v))
	case v <= 1073741823:
		return append(b, (2<<6)|byte(v>>24), byte(v>>16), byte(v>>8), byte(v))
	case v <= 4611686018427387903:
		return append(b, (3<<6)|byte(v>>56), byte(v>>48), byte(v>>40), byte(v>>32), byte(v>>24), byte(v>>16), byte(v>>8), byte(v))
	default:
		panic("varint too large")
	}
}

// sizeVarint returns the size of the variable-length integer encoding of f.
func sizeVarint(v uint64) int {
	switch {
	case v <= 63:
		return 1
	case v <= 16383:
		return 2
	case v <= 1073741823:
		return 4
	case v <= 4611686018427387903:
		return 8
	default:
		panic("varint too large")
	}
}

// consumeUint32 parses a 32-bit fixed-length, big-endian integer, reporting its length.
// It returns a negative length upon an error.
func consumeUint32(b []byte) (uint32, int) {
	if len(b) < 4 {
		return 0, -1
	}
	return binary.BigEndian.Uint32(b), 4
}

// consumeUint64 parses a 64-bit fixed-length, big-endian integer, reporting its length.
// It returns a negative length upon an error.
func consumeUint64(b []byte) (uint64, int) {
	if len(b) < 8 {
		return 0, -1
	}
	return binary.BigEndian.Uint64(b), 8
}

// consumeUint8Bytes parses a sequence of bytes prefixed with an 8-bit length,
// reporting the total number of bytes consumed.
// It returns a negative length upon an error.
func consumeUint8Bytes(b []byte) ([]byte, int) {
	if len(b) < 1 {
		return nil, -1
	}
	size := int(b[0])
	const n = 1
	if size > len(b[n:]) {
		return nil, -1
	}
	return b[n:][:size], size + n
}

// appendUint8Bytes appends a sequence of bytes prefixed by an 8-bit length.
func appendUint8Bytes(b, v []byte) []byte {
	if len(v) > 0xff {
		panic("uint8-prefixed bytes too large")
	}
	b = append(b, uint8(len(v)))
	b = append(b, v...)
	return b
}

// consumeVarintBytes parses a sequence of bytes preceded by a variable-length integer length,
// reporting the total number of bytes consumed.
// It returns a negative length upon an error.
func consumeVarintBytes(b []byte) ([]byte, int) {
	size, n := consumeVarint(b)
	if n < 0 {
		return nil, -1
	}
	if size > uint64(len(b[n:])) {
		return nil, -1
	}
	return b[n:][:size], int(size) + n
}

// appendVarintBytes appends a sequence of bytes prefixed by a variable-length integer length.
func appendVarintBytes(b, v []byte) []byte {
	b = appendVarint(b, uint64(len(v)))
	b = append(b, v...)
	return b
}
