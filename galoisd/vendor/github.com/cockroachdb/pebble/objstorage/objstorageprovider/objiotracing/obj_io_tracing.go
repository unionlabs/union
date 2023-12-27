// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objiotracing

import "github.com/cockroachdb/pebble/internal/base"

// OpType indicates the type of operation.
type OpType uint8

// OpType values.
const (
	ReadOp OpType = iota
	WriteOp
	// RecordCacheHitOp happens when a read is satisfied from the block cache. See
	// objstorage.ReadHandle.RecordCacheHit().
	RecordCacheHitOp
	// SetupForCompactionOp is a "meta operation" that configures a read handle
	// for large sequential reads. See objstorage.ReadHandle.SetupForCompaction().
	SetupForCompactionOp
)

// Reason indicates the higher-level context of the operation.
type Reason uint8

// Reason values.
const (
	UnknownReason Reason = iota
	ForFlush
	ForCompaction
	ForIngestion
	// TODO(radu): add ForUserFacing.
)

// BlockType indicates the type of data block relevant to an operation.
type BlockType uint8

// BlockType values.
const (
	UnknownBlock BlockType = iota
	DataBlock
	ValueBlock
	FilterBlock
	MetadataBlock
)

// Event is the on-disk format of a tracing event. It is exported here so that
// trace processing tools can use it by importing this package.
type Event struct {
	// Event start time as a Unix time (see time.Time.StartUnixNano()).
	// Note that recorded events are not necessarily ordered by time - this is
	// because separate event "streams" use local buffers (for performance).
	StartUnixNano int64
	Op            OpType
	Reason        Reason
	BlockType     BlockType
	// LSM level plus one (with 0 indicating unknown level).
	LevelPlusOne uint8
	// Hardcoded padding so that struct layout doesn't depend on architecture.
	_       uint32
	FileNum base.FileNum
	// HandleID is a unique identifier corresponding to an objstorage.ReadHandle;
	// only set for read operations performed through a ReadHandle.
	HandleID uint64
	Offset   int64
	Size     int64
}
