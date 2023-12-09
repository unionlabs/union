// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package rangekey provides functionality for working with range keys.
package rangekey

import (
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/rangekey"
	"github.com/cockroachdb/pebble/sstable"
)

// Fragmenter exports the keyspan.Fragmenter type.
type Fragmenter = keyspan.Fragmenter

// Key exports the keyspan.Key type.
type Key = keyspan.Key

// Span exports the keyspan.Span type.
type Span = keyspan.Span

// IsRangeKey returns if this InternalKey is a range key. Alias for
// rangekey.IsRangeKey.
func IsRangeKey(ik sstable.InternalKey) bool {
	return rangekey.IsRangeKey(ik.Kind())
}

// Decode decodes an InternalKey into a keyspan.Span, if it is a range key. If
// keysDst is provided, keys will be appended to keysDst to reduce allocations.
func Decode(ik sstable.InternalKey, val []byte, keysDst []keyspan.Key) (Span, error) {
	return rangekey.Decode(ik, val, keysDst)
}
