// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package crc implements the checksum algorithm used throughout pebble.
//
// The algorithm is CRC-32 with Castagnoli's polynomial, followed by a bit
// rotation and an additional delta. The additional processing is to lessen the
// probability of arbitrary key/value data coincidentally containing bytes that
// look like a checksum.
//
// To calculate the uint32 checksum of some data:
//
//	var u uint32 = crc.New(data).Value()
//
// In pebble, the uint32 value is then stored in little-endian format.
package crc // import "github.com/cockroachdb/pebble/internal/crc"

import "hash/crc32"

var table = crc32.MakeTable(crc32.Castagnoli)

// CRC is a small convenience wrapper for computing the CRC32 checksum used by
// pebble. This is the same algorithm as used by RocksDB.
type CRC uint32

// New returns the result of adding the bytes to the zero-value CRC.
func New(b []byte) CRC {
	return CRC(0).Update(b)
}

// Update returns the result of adding the bytes to the CRC.
func (c CRC) Update(b []byte) CRC {
	return CRC(crc32.Update(uint32(c), table, b))
}

// Value returns the cooked CRC value. The additional processing is to lessen
// the probability of arbitrary key/value data coincidentally containing bytes
// that look like a checksum.
func (c CRC) Value() uint32 {
	return uint32(c>>15|c<<17) + 0xa282ead8
}
