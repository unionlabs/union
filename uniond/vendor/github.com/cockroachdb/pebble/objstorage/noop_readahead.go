// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorage

import "context"

// NoopReadHandle can be used by Readable implementations that don't
// support read-ahead.
type NoopReadHandle struct {
	readable Readable
}

// MakeNoopReadHandle initializes a NoopReadHandle.
func MakeNoopReadHandle(r Readable) NoopReadHandle {
	return NoopReadHandle{readable: r}
}

var _ ReadHandle = (*NoopReadHandle)(nil)

// ReadAt is part of the ReadHandle interface.
func (h *NoopReadHandle) ReadAt(ctx context.Context, p []byte, off int64) error {
	return h.readable.ReadAt(ctx, p, off)
}

// Close is part of the ReadHandle interface.
func (*NoopReadHandle) Close() error { return nil }

// SetupForCompaction is part of the ReadHandle interface.
func (*NoopReadHandle) SetupForCompaction() {}

// RecordCacheHit is part of the ReadHandle interface.
func (*NoopReadHandle) RecordCacheHit(_ context.Context, offset, size int64) {}
