// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build !pebble_obj_io_tracing
// +build !pebble_obj_io_tracing

package objiotracing

import (
	"context"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/vfs"
)

// Enabled is used to short circuit tracing-related code in regular builds.
const Enabled = false

// Tracer manages the writing of object IO traces to files.
type Tracer struct{}

// Open creates a Tracer which generates trace files in the given directory.
// Each trace file contains a series of Events (as they are in memory).
func Open(fs vfs.FS, fsDir string) *Tracer {
	return nil
}

// Close the tracer, flushing any remaining events.
func (*Tracer) Close() {}

// WrapReadable wraps an objstorage.Readable with one that generates tracing
// events.
func (*Tracer) WrapReadable(
	ctx context.Context, r objstorage.Readable, fileNum base.DiskFileNum,
) objstorage.Readable {
	return r
}

// WrapWritable wraps an objstorage.Writable with one that generates tracing
// events.
func (t *Tracer) WrapWritable(
	ctx context.Context, w objstorage.Writable, fileNum base.DiskFileNum,
) objstorage.Writable {
	return w
}

// WithReason creates a context that has an associated Reason (which ends up in
// traces created under that context).
func WithReason(ctx context.Context, reason Reason) context.Context { return ctx }

// WithBlockType creates a context that has an associated BlockType (which ends up in
// traces created under that context).
func WithBlockType(ctx context.Context, blockType BlockType) context.Context { return ctx }

// WithLevel creates a context that has an associated level (which ends up in
// traces created under that context).
func WithLevel(ctx context.Context, level int) context.Context { return ctx }
