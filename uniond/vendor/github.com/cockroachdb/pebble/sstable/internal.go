// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import "github.com/cockroachdb/pebble/internal/base"

// InternalKeyKind exports the base.InternalKeyKind type.
type InternalKeyKind = base.InternalKeyKind

// SeekGEFlags exports base.SeekGEFlags.
type SeekGEFlags = base.SeekGEFlags

// SeekLTFlags exports base.SeekLTFlags.
type SeekLTFlags = base.SeekLTFlags

// These constants are part of the file format, and should not be changed.
const (
	InternalKeyKindDelete          = base.InternalKeyKindDelete
	InternalKeyKindSet             = base.InternalKeyKindSet
	InternalKeyKindMerge           = base.InternalKeyKindMerge
	InternalKeyKindLogData         = base.InternalKeyKindLogData
	InternalKeyKindSingleDelete    = base.InternalKeyKindSingleDelete
	InternalKeyKindRangeDelete     = base.InternalKeyKindRangeDelete
	InternalKeyKindSetWithDelete   = base.InternalKeyKindSetWithDelete
	InternalKeyKindDeleteSized     = base.InternalKeyKindDeleteSized
	InternalKeyKindMax             = base.InternalKeyKindMax
	InternalKeyKindInvalid         = base.InternalKeyKindInvalid
	InternalKeySeqNumBatch         = base.InternalKeySeqNumBatch
	InternalKeySeqNumMax           = base.InternalKeySeqNumMax
	InternalKeyRangeDeleteSentinel = base.InternalKeyRangeDeleteSentinel
)

// InternalKey exports the base.InternalKey type.
type InternalKey = base.InternalKey
