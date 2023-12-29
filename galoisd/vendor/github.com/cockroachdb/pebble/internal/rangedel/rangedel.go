// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package rangedel

import (
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
)

// Encode takes a Span containing only range deletions. It invokes the provided
// closure with the encoded internal keys that represent the Span's state. The
// keys and values passed to emit are only valid until the closure returns.  If
// emit returns an error, Encode stops and returns the error.
func Encode(s *keyspan.Span, emit func(k base.InternalKey, v []byte) error) error {
	for _, k := range s.Keys {
		if k.Kind() != base.InternalKeyKindRangeDelete {
			return base.CorruptionErrorf("pebble: rangedel.Encode cannot encode %s key", k.Kind())
		}
		ik := base.InternalKey{
			UserKey: s.Start,
			Trailer: k.Trailer,
		}
		if err := emit(ik, s.End); err != nil {
			return err
		}
	}
	return nil
}

// Decode takes an internal key pair encoding a range deletion and returns a
// decoded keyspan containing the key. If keysDst is provided, the key will be
// appended to keysDst, avoiding an allocation.
func Decode(ik base.InternalKey, v []byte, keysDst []keyspan.Key) keyspan.Span {
	return keyspan.Span{
		Start: ik.UserKey,
		End:   v,
		Keys: append(keysDst, keyspan.Key{
			Trailer: ik.Trailer,
		}),
	}
}
