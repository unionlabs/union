// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package manifest

import (
	"bytes"
	"fmt"
	"sort"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"unicode"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
)

// Compare exports the base.Compare type.
type Compare = base.Compare

// InternalKey exports the base.InternalKey type.
type InternalKey = base.InternalKey

// TableInfo contains the common information for table related events.
type TableInfo struct {
	// FileNum is the internal DB identifier for the table.
	FileNum base.FileNum
	// Size is the size of the file in bytes.
	Size uint64
	// Smallest is the smallest internal key in the table.
	Smallest InternalKey
	// Largest is the largest internal key in the table.
	Largest InternalKey
	// SmallestSeqNum is the smallest sequence number in the table.
	SmallestSeqNum uint64
	// LargestSeqNum is the largest sequence number in the table.
	LargestSeqNum uint64
}

// TableStats contains statistics on a table used for compaction heuristics,
// and export via Metrics.
type TableStats struct {
	// The total number of entries in the table.
	NumEntries uint64
	// The number of point and range deletion entries in the table.
	NumDeletions uint64
	// NumRangeKeySets is the total number of range key sets in the table.
	//
	// NB: If there's a chance that the sstable contains any range key sets,
	// then NumRangeKeySets must be > 0.
	NumRangeKeySets uint64
	// Estimate of the total disk space that may be dropped by this table's
	// point deletions by compacting them.
	PointDeletionsBytesEstimate uint64
	// Estimate of the total disk space that may be dropped by this table's
	// range deletions by compacting them. This estimate is at data-block
	// granularity and is not updated if compactions beneath the table reduce
	// the amount of reclaimable disk space. It also does not account for
	// overlapping data in L0 and ignores L0 sublevels, but the error that
	// introduces is expected to be small.
	//
	// Tables in the bottommost level of the LSM may have a nonzero estimate if
	// snapshots or move compactions prevented the elision of their range
	// tombstones. A table in the bottommost level that was ingested into L6
	// will have a zero estimate, because the file's sequence numbers indicate
	// that the tombstone cannot drop any data contained within the file itself.
	RangeDeletionsBytesEstimate uint64
	// Total size of value blocks and value index block.
	ValueBlocksSize uint64
}

// boundType represents the type of key (point or range) present as the smallest
// and largest keys.
type boundType uint8

const (
	boundTypePointKey boundType = iota + 1
	boundTypeRangeKey
)

// CompactionState is the compaction state of a file.
//
// The following shows the valid state transitions:
//
//	NotCompacting --> Compacting --> Compacted
//	      ^               |
//	      |               |
//	      +-------<-------+
//
// Input files to a compaction transition to Compacting when a compaction is
// picked. A file that has finished compacting typically transitions into the
// Compacted state, at which point it is effectively obsolete ("zombied") and
// will eventually be removed from the LSM. A file that has been move-compacted
// will transition from Compacting back into the NotCompacting state, signaling
// that the file may be selected for a subsequent compaction. A failed
// compaction will result in all input tables transitioning from Compacting to
// NotCompacting.
//
// This state is in-memory only. It is not persisted to the manifest.
type CompactionState uint8

// CompactionStates.
const (
	CompactionStateNotCompacting CompactionState = iota
	CompactionStateCompacting
	CompactionStateCompacted
)

// String implements fmt.Stringer.
func (s CompactionState) String() string {
	switch s {
	case CompactionStateNotCompacting:
		return "NotCompacting"
	case CompactionStateCompacting:
		return "Compacting"
	case CompactionStateCompacted:
		return "Compacted"
	default:
		panic(fmt.Sprintf("pebble: unknown compaction state %d", s))
	}
}

// FileMetadata is maintained for leveled-ssts, i.e., they belong to a level of
// some version. FileMetadata does not contain the actual level of the sst,
// since such leveled-ssts can move across levels in different versions, while
// sharing the same FileMetadata. There are two kinds of leveled-ssts, physical
// and virtual. Underlying both leveled-ssts is a backing-sst, for which the
// only state is FileBacking. A backing-sst is level-less. It is possible for a
// backing-sst to be referred to by a physical sst in one version and by one or
// more virtual ssts in one or more versions. A backing-sst becomes obsolete
// and can be deleted once it is no longer required by any physical or virtual
// sst in any version.
//
// We maintain some invariants:
//
//  1. Each physical and virtual sst will have a unique FileMetadata.FileNum,
//     and there will be exactly one FileMetadata associated with the FileNum.
//
//  2. Within a version, a backing-sst is either only referred to by one
//     physical sst or one or more virtual ssts.
//
//  3. Once a backing-sst is referred to by a virtual sst in the latest version,
//     it cannot go back to being referred to by a physical sst in any future
//     version.
//
// Once a physical sst is no longer needed by any version, we will no longer
// maintain the file metadata associated with it. We will still maintain the
// FileBacking associated with the physical sst if the backing sst is required
// by any virtual ssts in any version.
type FileMetadata struct {
	// AllowedSeeks is used to determine if a file should be picked for
	// a read triggered compaction. It is decremented when read sampling
	// in pebble.Iterator after every after every positioning operation
	// that returns a user key (eg. Next, Prev, SeekGE, SeekLT, etc).
	AllowedSeeks atomic.Int64

	// statsValid indicates if stats have been loaded for the table. The
	// TableStats structure is populated only if valid is true.
	statsValid atomic.Bool

	// FileBacking is the state which backs either a physical or virtual
	// sstables.
	FileBacking *FileBacking

	// InitAllowedSeeks is the inital value of allowed seeks. This is used
	// to re-set allowed seeks on a file once it hits 0.
	InitAllowedSeeks int64
	// FileNum is the file number.
	//
	// INVARIANT: when !FileMetadata.Virtual, FileNum == FileBacking.DiskFileNum.
	FileNum base.FileNum
	// Size is the size of the file, in bytes. Size is an approximate value for
	// virtual sstables.
	//
	// INVARIANTS:
	// - When !FileMetadata.Virtual, Size == FileBacking.Size.
	// - Size should be non-zero. Size 0 virtual sstables must not be created.
	Size uint64
	// File creation time in seconds since the epoch (1970-01-01 00:00:00
	// UTC). For ingested sstables, this corresponds to the time the file was
	// ingested. For virtual sstables, this corresponds to the wall clock time
	// when the FileMetadata for the virtual sstable was first created.
	CreationTime int64
	// Lower and upper bounds for the smallest and largest sequence numbers in
	// the table, across both point and range keys. For physical sstables, these
	// values are tight bounds. For virtual sstables, there is no guarantee that
	// there will be keys with SmallestSeqNum or LargestSeqNum within virtual
	// sstable bounds.
	SmallestSeqNum uint64
	LargestSeqNum  uint64
	// SmallestPointKey and LargestPointKey are the inclusive bounds for the
	// internal point keys stored in the table. This includes RANGEDELs, which
	// alter point keys.
	// NB: these field should be set using ExtendPointKeyBounds. They are left
	// exported for reads as an optimization.
	SmallestPointKey InternalKey
	LargestPointKey  InternalKey
	// SmallestRangeKey and LargestRangeKey are the inclusive bounds for the
	// internal range keys stored in the table.
	// NB: these field should be set using ExtendRangeKeyBounds. They are left
	// exported for reads as an optimization.
	SmallestRangeKey InternalKey
	LargestRangeKey  InternalKey
	// Smallest and Largest are the inclusive bounds for the internal keys stored
	// in the table, across both point and range keys.
	// NB: these fields are derived from their point and range key equivalents,
	// and are updated via the MaybeExtend{Point,Range}KeyBounds methods.
	Smallest InternalKey
	Largest  InternalKey
	// Stats describe table statistics. Protected by DB.mu.
	//
	// For virtual sstables, set stats upon virtual sstable creation as
	// asynchronous computation of stats is not currently supported.
	//
	// TODO(bananabrick): To support manifest replay for virtual sstables, we
	// probably need to compute virtual sstable stats asynchronously. Otherwise,
	// we'd have to write virtual sstable stats to the version edit.
	Stats TableStats

	// For L0 files only. Protected by DB.mu. Used to generate L0 sublevels and
	// pick L0 compactions. Only accurate for the most recent Version.
	SubLevel         int
	L0Index          int
	minIntervalIndex int
	maxIntervalIndex int

	// NB: the alignment of this struct is 8 bytes. We pack all the bools to
	// ensure an optimal packing.

	// IsIntraL0Compacting is set to True if this file is part of an intra-L0
	// compaction. When it's true, IsCompacting must also return true. If
	// Compacting is true and IsIntraL0Compacting is false for an L0 file, the
	// file must be part of a compaction to Lbase.
	IsIntraL0Compacting bool
	CompactionState     CompactionState
	// True if compaction of this file has been explicitly requested.
	// Previously, RocksDB and earlier versions of Pebble allowed this
	// flag to be set by a user table property collector. Some earlier
	// versions of Pebble respected this flag, while other more recent
	// versions ignored this flag.
	//
	// More recently this flag has been repurposed to facilitate the
	// compaction of 'atomic compaction units'. Files marked for
	// compaction are compacted in a rewrite compaction at the lowest
	// possible compaction priority.
	//
	// NB: A count of files marked for compaction is maintained on
	// Version, and compaction picking reads cached annotations
	// determined by this field.
	//
	// Protected by DB.mu.
	MarkedForCompaction bool
	// HasPointKeys tracks whether the table contains point keys (including
	// RANGEDELs). If a table contains only range deletions, HasPointsKeys is
	// still true.
	HasPointKeys bool
	// HasRangeKeys tracks whether the table contains any range keys.
	HasRangeKeys bool
	// smallestSet and largestSet track whether the overall bounds have been set.
	boundsSet bool
	// boundTypeSmallest and boundTypeLargest provide an indication as to which
	// key type (point or range) corresponds to the smallest and largest overall
	// table bounds.
	boundTypeSmallest, boundTypeLargest boundType
	// Virtual is true if the FileMetadata belongs to a virtual sstable.
	Virtual bool
}

// PhysicalFileMeta is used by functions which want a guarantee that their input
// belongs to a physical sst and not a virtual sst.
//
// NB: This type should only be constructed by calling
// FileMetadata.PhysicalMeta.
type PhysicalFileMeta struct {
	*FileMetadata
}

// VirtualFileMeta is used by functions which want a guarantee that their input
// belongs to a virtual sst and not a physical sst.
//
// A VirtualFileMeta inherits all the same fields as a FileMetadata. These
// fields have additional invariants imposed on them, and/or slightly varying
// meanings:
//   - Smallest and Largest (and their counterparts
//     {Smallest, Largest}{Point,Range}Key) remain tight bounds that represent a
//     key at that exact bound. We make the effort to determine the next smallest
//     or largest key in an sstable after virtualizing it, to maintain this
//     tightness. If the largest is a sentinel key (IsExclusiveSentinel()), it
//     could mean that a rangedel or range key ends at that user key, or has been
//     truncated to that user key.
//   - One invariant is that if a rangedel or range key is truncated on its
//     upper bound, the virtual sstable *must* have a rangedel or range key
//     sentinel key as its upper bound. This is because truncation yields
//     an exclusive upper bound for the rangedel/rangekey, and if there are
//     any points at that exclusive upper bound within the same virtual
//     sstable, those could get uncovered by this truncation. We enforce this
//     invariant in calls to keyspan.Truncate.
//   - Size is an estimate of the size of the virtualized portion of this sstable.
//     The underlying file's size is stored in FileBacking.Size, though it could
//     also be estimated or could correspond to just the referenced portion of
//     a file (eg. if the file originated on another node).
//   - Size must be > 0.
//   - SmallestSeqNum and LargestSeqNum are loose bounds for virtual sstables.
//     This means that all keys in the virtual sstable must have seqnums within
//     [SmallestSeqNum, LargestSeqNum], however there's no guarantee that there's
//     a key with a seqnum at either of the bounds. Calculating tight seqnum
//     bounds would be too expensive and deliver little value.
//
// NB: This type should only be constructed by calling FileMetadata.VirtualMeta.
type VirtualFileMeta struct {
	*FileMetadata
}

// PhysicalMeta should be the only source of creating the PhysicalFileMeta
// wrapper type.
func (m *FileMetadata) PhysicalMeta() PhysicalFileMeta {
	if m.Virtual {
		panic("pebble: file metadata does not belong to a physical sstable")
	}
	return PhysicalFileMeta{
		m,
	}
}

// VirtualMeta should be the only source of creating the VirtualFileMeta wrapper
// type.
func (m *FileMetadata) VirtualMeta() VirtualFileMeta {
	if !m.Virtual {
		panic("pebble: file metadata does not belong to a virtual sstable")
	}
	return VirtualFileMeta{
		m,
	}
}

// FileBacking either backs a single physical sstable, or one or more virtual
// sstables.
//
// See the comment above the FileMetadata type for sstable terminology.
type FileBacking struct {
	// Reference count for the backing file on disk: incremented when a
	// physical or virtual sstable which is backed by the FileBacking is
	// added to a version and decremented when the version is unreferenced.
	// We ref count in order to determine when it is safe to delete a
	// backing sst file from disk. The backing file is obsolete when the
	// reference count falls to zero.
	refs atomic.Int32
	// latestVersionRefs are the references to the FileBacking in the
	// latest version. This reference can be through a single physical
	// sstable in the latest version, or one or more virtual sstables in the
	// latest version.
	//
	// INVARIANT: latestVersionRefs <= refs.
	latestVersionRefs atomic.Int32
	// VirtualizedSize is set iff the backing sst is only referred to by
	// virtual ssts in the latest version. VirtualizedSize is the sum of the
	// virtual sstable sizes of all of the virtual sstables in the latest
	// version which are backed by the physical sstable. When a virtual
	// sstable is removed from the latest version, we will decrement the
	// VirtualizedSize. During compaction picking, we'll compensate a
	// virtual sstable file size by
	// (FileBacking.Size - FileBacking.VirtualizedSize) / latestVersionRefs.
	// The intuition is that if FileBacking.Size - FileBacking.VirtualizedSize
	// is high, then the space amplification due to virtual sstables is
	// high, and we should pick the virtual sstable with a higher priority.
	//
	// TODO(bananabrick): Compensate the virtual sstable file size using
	// the VirtualizedSize during compaction picking and test.
	VirtualizedSize atomic.Uint64
	DiskFileNum     base.DiskFileNum
	Size            uint64
}

// InitPhysicalBacking allocates and sets the FileBacking which is required by a
// physical sstable FileMetadata.
//
// Ensure that the state required by FileBacking, such as the FileNum, is
// already set on the FileMetadata before InitPhysicalBacking is called.
// Calling InitPhysicalBacking only after the relevant state has been set in the
// FileMetadata is not necessary in tests which don't rely on FileBacking.
func (m *FileMetadata) InitPhysicalBacking() {
	if m.Virtual {
		panic("pebble: virtual sstables should use a pre-existing FileBacking")
	}
	if m.FileBacking == nil {
		m.FileBacking = &FileBacking{Size: m.Size, DiskFileNum: m.FileNum.DiskFileNum()}
	}
}

// InitProviderBacking creates a new FileBacking for a file backed by
// an objstorage.Provider.
func (m *FileMetadata) InitProviderBacking(fileNum base.DiskFileNum) {
	if !m.Virtual {
		panic("pebble: provider-backed sstables must be virtual")
	}
	if m.FileBacking == nil {
		m.FileBacking = &FileBacking{DiskFileNum: fileNum}
	}
}

// ValidateVirtual should be called once the FileMetadata for a virtual sstable
// is created to verify that the fields of the virtual sstable are sound.
func (m *FileMetadata) ValidateVirtual(createdFrom *FileMetadata) {
	if !m.Virtual {
		panic("pebble: invalid virtual sstable")
	}

	if createdFrom.SmallestSeqNum != m.SmallestSeqNum {
		panic("pebble: invalid smallest sequence number for virtual sstable")
	}

	if createdFrom.LargestSeqNum != m.LargestSeqNum {
		panic("pebble: invalid largest sequence number for virtual sstable")
	}

	if createdFrom.FileBacking != nil && createdFrom.FileBacking != m.FileBacking {
		panic("pebble: invalid physical sstable state for virtual sstable")
	}

	if m.Size == 0 {
		panic("pebble: virtual sstable size must be set upon creation")
	}
}

// Refs returns the refcount of backing sstable.
func (m *FileMetadata) Refs() int32 {
	return m.FileBacking.refs.Load()
}

// Ref increments the ref count associated with the backing sstable.
func (m *FileMetadata) Ref() {
	m.FileBacking.refs.Add(1)
}

// Unref decrements the ref count associated with the backing sstable.
func (m *FileMetadata) Unref() int32 {
	v := m.FileBacking.refs.Add(-1)
	if invariants.Enabled && v < 0 {
		panic("pebble: invalid FileMetadata refcounting")
	}
	return v
}

// LatestRef increments the latest ref count associated with the backing
// sstable.
func (m *FileMetadata) LatestRef() {
	m.FileBacking.latestVersionRefs.Add(1)

	if m.Virtual {
		m.FileBacking.VirtualizedSize.Add(m.Size)
	}
}

// LatestUnref decrements the latest ref count associated with the backing
// sstable.
func (m *FileMetadata) LatestUnref() int32 {
	if m.Virtual {
		m.FileBacking.VirtualizedSize.Add(-m.Size)
	}

	v := m.FileBacking.latestVersionRefs.Add(-1)
	if invariants.Enabled && v < 0 {
		panic("pebble: invalid FileMetadata latest refcounting")
	}
	return v
}

// LatestRefs returns the latest ref count associated with the backing sstable.
func (m *FileMetadata) LatestRefs() int32 {
	return m.FileBacking.latestVersionRefs.Load()
}

// SetCompactionState transitions this file's compaction state to the given
// state. Protected by DB.mu.
func (m *FileMetadata) SetCompactionState(to CompactionState) {
	if invariants.Enabled {
		transitionErr := func() error {
			return errors.Newf("pebble: invalid compaction state transition: %s -> %s", m.CompactionState, to)
		}
		switch m.CompactionState {
		case CompactionStateNotCompacting:
			if to != CompactionStateCompacting {
				panic(transitionErr())
			}
		case CompactionStateCompacting:
			if to != CompactionStateCompacted && to != CompactionStateNotCompacting {
				panic(transitionErr())
			}
		case CompactionStateCompacted:
			panic(transitionErr())
		default:
			panic(fmt.Sprintf("pebble: unknown compaction state: %d", m.CompactionState))
		}
	}
	m.CompactionState = to
}

// IsCompacting returns true if this file's compaction state is
// CompactionStateCompacting. Protected by DB.mu.
func (m *FileMetadata) IsCompacting() bool {
	return m.CompactionState == CompactionStateCompacting
}

// StatsValid returns true if the table stats have been populated. If StatValid
// returns true, the Stats field may be read (with or without holding the
// database mutex).
func (m *FileMetadata) StatsValid() bool {
	return m.statsValid.Load()
}

// StatsMarkValid marks the TableStats as valid. The caller must hold DB.mu
// while populating TableStats and calling StatsMarkValud. Once stats are
// populated, they must not be mutated.
func (m *FileMetadata) StatsMarkValid() {
	m.statsValid.Store(true)
}

// ExtendPointKeyBounds attempts to extend the lower and upper point key bounds
// and overall table bounds with the given smallest and largest keys. The
// smallest and largest bounds may not be extended if the table already has a
// bound that is smaller or larger, respectively. The receiver is returned.
// NB: calling this method should be preferred to manually setting the bounds by
// manipulating the fields directly, to maintain certain invariants.
func (m *FileMetadata) ExtendPointKeyBounds(
	cmp Compare, smallest, largest InternalKey,
) *FileMetadata {
	// Update the point key bounds.
	if !m.HasPointKeys {
		m.SmallestPointKey, m.LargestPointKey = smallest, largest
		m.HasPointKeys = true
	} else {
		if base.InternalCompare(cmp, smallest, m.SmallestPointKey) < 0 {
			m.SmallestPointKey = smallest
		}
		if base.InternalCompare(cmp, largest, m.LargestPointKey) > 0 {
			m.LargestPointKey = largest
		}
	}
	// Update the overall bounds.
	m.extendOverallBounds(cmp, m.SmallestPointKey, m.LargestPointKey, boundTypePointKey)
	return m
}

// ExtendRangeKeyBounds attempts to extend the lower and upper range key bounds
// and overall table bounds with the given smallest and largest keys. The
// smallest and largest bounds may not be extended if the table already has a
// bound that is smaller or larger, respectively. The receiver is returned.
// NB: calling this method should be preferred to manually setting the bounds by
// manipulating the fields directly, to maintain certain invariants.
func (m *FileMetadata) ExtendRangeKeyBounds(
	cmp Compare, smallest, largest InternalKey,
) *FileMetadata {
	// Update the range key bounds.
	if !m.HasRangeKeys {
		m.SmallestRangeKey, m.LargestRangeKey = smallest, largest
		m.HasRangeKeys = true
	} else {
		if base.InternalCompare(cmp, smallest, m.SmallestRangeKey) < 0 {
			m.SmallestRangeKey = smallest
		}
		if base.InternalCompare(cmp, largest, m.LargestRangeKey) > 0 {
			m.LargestRangeKey = largest
		}
	}
	// Update the overall bounds.
	m.extendOverallBounds(cmp, m.SmallestRangeKey, m.LargestRangeKey, boundTypeRangeKey)
	return m
}

// extendOverallBounds attempts to extend the overall table lower and upper
// bounds. The given bounds may not be used if a lower or upper bound already
// exists that is smaller or larger than the given keys, respectively. The given
// boundType will be used if the bounds are updated.
func (m *FileMetadata) extendOverallBounds(
	cmp Compare, smallest, largest InternalKey, bTyp boundType,
) {
	if !m.boundsSet {
		m.Smallest, m.Largest = smallest, largest
		m.boundsSet = true
		m.boundTypeSmallest, m.boundTypeLargest = bTyp, bTyp
	} else {
		if base.InternalCompare(cmp, smallest, m.Smallest) < 0 {
			m.Smallest = smallest
			m.boundTypeSmallest = bTyp
		}
		if base.InternalCompare(cmp, largest, m.Largest) > 0 {
			m.Largest = largest
			m.boundTypeLargest = bTyp
		}
	}
}

// Overlaps returns true if the file key range overlaps with the given range.
func (m *FileMetadata) Overlaps(cmp Compare, start []byte, end []byte, exclusiveEnd bool) bool {
	if c := cmp(m.Largest.UserKey, start); c < 0 || (c == 0 && m.Largest.IsExclusiveSentinel()) {
		// f is completely before the specified range; no overlap.
		return false
	}
	if c := cmp(m.Smallest.UserKey, end); c > 0 || (c == 0 && exclusiveEnd) {
		// f is completely after the specified range; no overlap.
		return false
	}
	return true
}

// ContainedWithinSpan returns true if the file key range completely overlaps with the
// given range ("end" is assumed to exclusive).
func (m *FileMetadata) ContainedWithinSpan(cmp Compare, start, end []byte) bool {
	lowerCmp, upperCmp := cmp(m.Smallest.UserKey, start), cmp(m.Largest.UserKey, end)
	return lowerCmp >= 0 && (upperCmp < 0 || (upperCmp == 0 && m.Largest.IsExclusiveSentinel()))
}

// ContainsKeyType returns whether or not the file contains keys of the provided
// type.
func (m *FileMetadata) ContainsKeyType(kt KeyType) bool {
	switch kt {
	case KeyTypePointAndRange:
		return true
	case KeyTypePoint:
		return m.HasPointKeys
	case KeyTypeRange:
		return m.HasRangeKeys
	default:
		panic("unrecognized key type")
	}
}

// SmallestBound returns the file's smallest bound of the key type. It returns a
// false second return value if the file does not contain any keys of the key
// type.
func (m *FileMetadata) SmallestBound(kt KeyType) (*InternalKey, bool) {
	switch kt {
	case KeyTypePointAndRange:
		return &m.Smallest, true
	case KeyTypePoint:
		return &m.SmallestPointKey, m.HasPointKeys
	case KeyTypeRange:
		return &m.SmallestRangeKey, m.HasRangeKeys
	default:
		panic("unrecognized key type")
	}
}

// LargestBound returns the file's largest bound of the key type. It returns a
// false second return value if the file does not contain any keys of the key
// type.
func (m *FileMetadata) LargestBound(kt KeyType) (*InternalKey, bool) {
	switch kt {
	case KeyTypePointAndRange:
		return &m.Largest, true
	case KeyTypePoint:
		return &m.LargestPointKey, m.HasPointKeys
	case KeyTypeRange:
		return &m.LargestRangeKey, m.HasRangeKeys
	default:
		panic("unrecognized key type")
	}
}

const (
	maskContainsPointKeys = 1 << 0
	maskSmallest          = 1 << 1
	maskLargest           = 1 << 2
)

// boundsMarker returns a marker byte whose bits encode the following
// information (in order from least significant bit):
// - if the table contains point keys
// - if the table's smallest key is a point key
// - if the table's largest key is a point key
func (m *FileMetadata) boundsMarker() (sentinel uint8, err error) {
	if m.HasPointKeys {
		sentinel |= maskContainsPointKeys
	}
	switch m.boundTypeSmallest {
	case boundTypePointKey:
		sentinel |= maskSmallest
	case boundTypeRangeKey:
		// No op - leave bit unset.
	default:
		return 0, base.CorruptionErrorf("file %s has neither point nor range key as smallest key", m.FileNum)
	}
	switch m.boundTypeLargest {
	case boundTypePointKey:
		sentinel |= maskLargest
	case boundTypeRangeKey:
		// No op - leave bit unset.
	default:
		return 0, base.CorruptionErrorf("file %s has neither point nor range key as largest key", m.FileNum)
	}
	return
}

// String implements fmt.Stringer, printing the file number and the overall
// table bounds.
func (m *FileMetadata) String() string {
	return fmt.Sprintf("%s:[%s-%s]", m.FileNum, m.Smallest, m.Largest)
}

// DebugString returns a verbose representation of FileMetadata, typically for
// use in tests and debugging, returning the file number and the point, range
// and overall bounds for the table.
func (m *FileMetadata) DebugString(format base.FormatKey, verbose bool) string {
	var b bytes.Buffer
	fmt.Fprintf(&b, "%s:[%s-%s]",
		m.FileNum, m.Smallest.Pretty(format), m.Largest.Pretty(format))
	if !verbose {
		return b.String()
	}
	fmt.Fprintf(&b, " seqnums:[%d-%d]", m.SmallestSeqNum, m.LargestSeqNum)
	if m.HasPointKeys {
		fmt.Fprintf(&b, " points:[%s-%s]",
			m.SmallestPointKey.Pretty(format), m.LargestPointKey.Pretty(format))
	}
	if m.HasRangeKeys {
		fmt.Fprintf(&b, " ranges:[%s-%s]",
			m.SmallestRangeKey.Pretty(format), m.LargestRangeKey.Pretty(format))
	}
	return b.String()
}

// ParseFileMetadataDebug parses a FileMetadata from its DebugString
// representation.
func ParseFileMetadataDebug(s string) (*FileMetadata, error) {
	// Split lines of the form:
	//  000000:[a#0,SET-z#0,SET] seqnums:[5-5] points:[...] ranges:[...]
	fields := strings.FieldsFunc(s, func(c rune) bool {
		switch c {
		case ':', '[', '-', ']':
			return true
		default:
			return unicode.IsSpace(c) // NB: also trim whitespace padding.
		}
	})
	if len(fields)%3 != 0 {
		return nil, errors.Newf("malformed input: %s", s)
	}
	m := &FileMetadata{}
	for len(fields) > 0 {
		prefix := fields[0]
		if prefix == "seqnums" {
			smallestSeqNum, err := strconv.ParseUint(fields[1], 10, 64)
			if err != nil {
				return m, errors.Newf("malformed input: %s: %s", s, err)
			}
			largestSeqNum, err := strconv.ParseUint(fields[2], 10, 64)
			if err != nil {
				return m, errors.Newf("malformed input: %s: %s", s, err)
			}
			m.SmallestSeqNum, m.LargestSeqNum = smallestSeqNum, largestSeqNum
			fields = fields[3:]
			continue
		}
		smallest := base.ParsePrettyInternalKey(fields[1])
		largest := base.ParsePrettyInternalKey(fields[2])
		switch prefix {
		case "points":
			m.SmallestPointKey, m.LargestPointKey = smallest, largest
			m.HasPointKeys = true
		case "ranges":
			m.SmallestRangeKey, m.LargestRangeKey = smallest, largest
			m.HasRangeKeys = true
		default:
			fileNum, err := strconv.ParseUint(prefix, 10, 64)
			if err != nil {
				return m, errors.Newf("malformed input: %s: %s", s, err)
			}
			m.FileNum = base.FileNum(fileNum)
			m.Smallest, m.Largest = smallest, largest
			m.boundsSet = true
		}
		fields = fields[3:]
	}
	// By default, when the parser sees just the overall bounds, we set the point
	// keys. This preserves backwards compatability with existing test cases that
	// specify only the overall bounds.
	if !m.HasPointKeys && !m.HasRangeKeys {
		m.SmallestPointKey, m.LargestPointKey = m.Smallest, m.Largest
		m.HasPointKeys = true
	}
	m.InitPhysicalBacking()
	return m, nil
}

// Validate validates the metadata for consistency with itself, returning an
// error if inconsistent.
func (m *FileMetadata) Validate(cmp Compare, formatKey base.FormatKey) error {
	// Combined range and point key validation.

	if !m.HasPointKeys && !m.HasRangeKeys {
		return base.CorruptionErrorf("file %s has neither point nor range keys",
			errors.Safe(m.FileNum))
	}
	if base.InternalCompare(cmp, m.Smallest, m.Largest) > 0 {
		return base.CorruptionErrorf("file %s has inconsistent bounds: %s vs %s",
			errors.Safe(m.FileNum), m.Smallest.Pretty(formatKey),
			m.Largest.Pretty(formatKey))
	}
	if m.SmallestSeqNum > m.LargestSeqNum {
		return base.CorruptionErrorf("file %s has inconsistent seqnum bounds: %d vs %d",
			errors.Safe(m.FileNum), m.SmallestSeqNum, m.LargestSeqNum)
	}

	// Point key validation.

	if m.HasPointKeys {
		if base.InternalCompare(cmp, m.SmallestPointKey, m.LargestPointKey) > 0 {
			return base.CorruptionErrorf("file %s has inconsistent point key bounds: %s vs %s",
				errors.Safe(m.FileNum), m.SmallestPointKey.Pretty(formatKey),
				m.LargestPointKey.Pretty(formatKey))
		}
		if base.InternalCompare(cmp, m.SmallestPointKey, m.Smallest) < 0 ||
			base.InternalCompare(cmp, m.LargestPointKey, m.Largest) > 0 {
			return base.CorruptionErrorf(
				"file %s has inconsistent point key bounds relative to overall bounds: "+
					"overall = [%s-%s], point keys = [%s-%s]",
				errors.Safe(m.FileNum),
				m.Smallest.Pretty(formatKey), m.Largest.Pretty(formatKey),
				m.SmallestPointKey.Pretty(formatKey), m.LargestPointKey.Pretty(formatKey),
			)
		}
	}

	// Range key validation.

	if m.HasRangeKeys {
		if base.InternalCompare(cmp, m.SmallestRangeKey, m.LargestRangeKey) > 0 {
			return base.CorruptionErrorf("file %s has inconsistent range key bounds: %s vs %s",
				errors.Safe(m.FileNum), m.SmallestRangeKey.Pretty(formatKey),
				m.LargestRangeKey.Pretty(formatKey))
		}
		if base.InternalCompare(cmp, m.SmallestRangeKey, m.Smallest) < 0 ||
			base.InternalCompare(cmp, m.LargestRangeKey, m.Largest) > 0 {
			return base.CorruptionErrorf(
				"file %s has inconsistent range key bounds relative to overall bounds: "+
					"overall = [%s-%s], range keys = [%s-%s]",
				errors.Safe(m.FileNum),
				m.Smallest.Pretty(formatKey), m.Largest.Pretty(formatKey),
				m.SmallestRangeKey.Pretty(formatKey), m.LargestRangeKey.Pretty(formatKey),
			)
		}
	}

	// Ensure that FileMetadata.Init was called.
	if m.FileBacking == nil {
		return base.CorruptionErrorf("file metadata FileBacking not set")
	}

	return nil
}

// TableInfo returns a subset of the FileMetadata state formatted as a
// TableInfo.
func (m *FileMetadata) TableInfo() TableInfo {
	return TableInfo{
		FileNum:        m.FileNum,
		Size:           m.Size,
		Smallest:       m.Smallest,
		Largest:        m.Largest,
		SmallestSeqNum: m.SmallestSeqNum,
		LargestSeqNum:  m.LargestSeqNum,
	}
}

func cmpUint64(a, b uint64) int {
	switch {
	case a < b:
		return -1
	case a > b:
		return +1
	default:
		return 0
	}
}

func (m *FileMetadata) cmpSeqNum(b *FileMetadata) int {
	// NB: This is the same ordering that RocksDB uses for L0 files.

	// Sort first by largest sequence number.
	if m.LargestSeqNum != b.LargestSeqNum {
		return cmpUint64(m.LargestSeqNum, b.LargestSeqNum)
	}
	// Then by smallest sequence number.
	if m.SmallestSeqNum != b.SmallestSeqNum {
		return cmpUint64(m.SmallestSeqNum, b.SmallestSeqNum)
	}
	// Break ties by file number.
	return cmpUint64(uint64(m.FileNum), uint64(b.FileNum))
}

func (m *FileMetadata) lessSeqNum(b *FileMetadata) bool {
	return m.cmpSeqNum(b) < 0
}

func (m *FileMetadata) cmpSmallestKey(b *FileMetadata, cmp Compare) int {
	return base.InternalCompare(cmp, m.Smallest, b.Smallest)
}

// KeyRange returns the minimum smallest and maximum largest internalKey for
// all the FileMetadata in iters.
func KeyRange(ucmp Compare, iters ...LevelIterator) (smallest, largest InternalKey) {
	first := true
	for _, iter := range iters {
		for meta := iter.First(); meta != nil; meta = iter.Next() {
			if first {
				first = false
				smallest, largest = meta.Smallest, meta.Largest
				continue
			}
			if base.InternalCompare(ucmp, smallest, meta.Smallest) >= 0 {
				smallest = meta.Smallest
			}
			if base.InternalCompare(ucmp, largest, meta.Largest) <= 0 {
				largest = meta.Largest
			}
		}
	}
	return smallest, largest
}

type bySeqNum []*FileMetadata

func (b bySeqNum) Len() int { return len(b) }
func (b bySeqNum) Less(i, j int) bool {
	return b[i].lessSeqNum(b[j])
}
func (b bySeqNum) Swap(i, j int) { b[i], b[j] = b[j], b[i] }

// SortBySeqNum sorts the specified files by increasing sequence number.
func SortBySeqNum(files []*FileMetadata) {
	sort.Sort(bySeqNum(files))
}

type bySmallest struct {
	files []*FileMetadata
	cmp   Compare
}

func (b bySmallest) Len() int { return len(b.files) }
func (b bySmallest) Less(i, j int) bool {
	return b.files[i].cmpSmallestKey(b.files[j], b.cmp) < 0
}
func (b bySmallest) Swap(i, j int) { b.files[i], b.files[j] = b.files[j], b.files[i] }

// SortBySmallest sorts the specified files by smallest key using the supplied
// comparison function to order user keys.
func SortBySmallest(files []*FileMetadata, cmp Compare) {
	sort.Sort(bySmallest{files, cmp})
}

func overlaps(iter LevelIterator, cmp Compare, start, end []byte, exclusiveEnd bool) LevelSlice {
	startIter := iter.Clone()
	{
		startIterFile := startIter.SeekGE(cmp, start)
		// SeekGE compares user keys. The user key `start` may be equal to the
		// f.Largest because f.Largest is a range deletion sentinel, indicating
		// that the user key `start` is NOT contained within the file f. If
		// that's the case, we can narrow the overlapping bounds to exclude the
		// file with the sentinel.
		if startIterFile != nil && startIterFile.Largest.IsExclusiveSentinel() &&
			cmp(startIterFile.Largest.UserKey, start) == 0 {
			startIterFile = startIter.Next()
		}
		_ = startIterFile // Ignore unused assignment.
	}

	endIter := iter.Clone()
	{
		endIterFile := endIter.SeekGE(cmp, end)

		if !exclusiveEnd {
			// endIter is now pointing at the *first* file with a largest key >= end.
			// If there are multiple files including the user key `end`, we want all
			// of them, so move forward.
			for endIterFile != nil && cmp(endIterFile.Largest.UserKey, end) == 0 {
				endIterFile = endIter.Next()
			}
		}

		// LevelSlice uses inclusive bounds, so if we seeked to the end sentinel
		// or nexted too far because Largest.UserKey equaled `end`, go back.
		//
		// Consider !exclusiveEnd and end = 'f', with the following file bounds:
		//
		//     [b,d] [e, f] [f, f] [g, h]
		//
		// the above for loop will Next until it arrives at [g, h]. We need to
		// observe that g > f, and Prev to the file with bounds [f, f].
		if endIterFile == nil {
			endIterFile = endIter.Prev()
		} else if c := cmp(endIterFile.Smallest.UserKey, end); c > 0 || c == 0 && exclusiveEnd {
			endIterFile = endIter.Prev()
		}
		_ = endIterFile // Ignore unused assignment.
	}
	return newBoundedLevelSlice(startIter.Clone().iter, &startIter.iter, &endIter.iter)
}

// NumLevels is the number of levels a Version contains.
const NumLevels = 7

// NewVersion constructs a new Version with the provided files. It requires
// the provided files are already well-ordered. It's intended for testing.
func NewVersion(
	cmp Compare, formatKey base.FormatKey, flushSplitBytes int64, files [NumLevels][]*FileMetadata,
) *Version {
	var v Version
	for l := range files {
		// NB: We specifically insert `files` into the B-Tree in the order
		// they appear within `files`. Some tests depend on this behavior in
		// order to test consistency checking, etc. Once we've constructed the
		// initial B-Tree, we swap out the btreeCmp for the correct one.
		// TODO(jackson): Adjust or remove the tests and remove this.
		v.Levels[l].tree, _ = makeBTree(btreeCmpSpecificOrder(files[l]), files[l])
		v.Levels[l].level = l
		if l == 0 {
			v.Levels[l].tree.cmp = btreeCmpSeqNum
		} else {
			v.Levels[l].tree.cmp = btreeCmpSmallestKey(cmp)
		}
		for _, f := range files[l] {
			v.Levels[l].totalSize += f.Size
		}
	}
	if err := v.InitL0Sublevels(cmp, formatKey, flushSplitBytes); err != nil {
		panic(err)
	}
	return &v
}

// Version is a collection of file metadata for on-disk tables at various
// levels. In-memory DBs are written to level-0 tables, and compactions
// migrate data from level N to level N+1. The tables map internal keys (which
// are a user key, a delete or set bit, and a sequence number) to user values.
//
// The tables at level 0 are sorted by largest sequence number. Due to file
// ingestion, there may be overlap in the ranges of sequence numbers contain in
// level 0 sstables. In particular, it is valid for one level 0 sstable to have
// the seqnum range [1,100] while an adjacent sstable has the seqnum range
// [50,50]. This occurs when the [50,50] table was ingested and given a global
// seqnum. The ingestion code will have ensured that the [50,50] sstable will
// not have any keys that overlap with the [1,100] in the seqnum range
// [1,49]. The range of internal keys [fileMetadata.smallest,
// fileMetadata.largest] in each level 0 table may overlap.
//
// The tables at any non-0 level are sorted by their internal key range and any
// two tables at the same non-0 level do not overlap.
//
// The internal key ranges of two tables at different levels X and Y may
// overlap, for any X != Y.
//
// Finally, for every internal key in a table at level X, there is no internal
// key in a higher level table that has both the same user key and a higher
// sequence number.
type Version struct {
	refs atomic.Int32

	// The level 0 sstables are organized in a series of sublevels. Similar to
	// the seqnum invariant in normal levels, there is no internal key in a
	// higher level table that has both the same user key and a higher sequence
	// number. Within a sublevel, tables are sorted by their internal key range
	// and any two tables at the same sublevel do not overlap. Unlike the normal
	// levels, sublevel n contains older tables (lower sequence numbers) than
	// sublevel n+1.
	//
	// The L0Sublevels struct is mostly used for compaction picking. As most
	// internal data structures in it are only necessary for compaction picking
	// and not for iterator creation, the reference to L0Sublevels is nil'd
	// after this version becomes the non-newest version, to reduce memory
	// usage.
	//
	// L0Sublevels.Levels contains L0 files ordered by sublevels. All the files
	// in Levels[0] are in L0Sublevels.Levels. L0SublevelFiles is also set to
	// a reference to that slice, as that slice is necessary for iterator
	// creation and needs to outlast L0Sublevels.
	L0Sublevels     *L0Sublevels
	L0SublevelFiles []LevelSlice

	Levels [NumLevels]LevelMetadata

	// RangeKeyLevels holds a subset of the same files as Levels that contain range
	// keys (i.e. fileMeta.HasRangeKeys == true). The memory amplification of this
	// duplication should be minimal, as range keys are expected to be rare.
	RangeKeyLevels [NumLevels]LevelMetadata

	// The callback to invoke when the last reference to a version is
	// removed. Will be called with list.mu held.
	Deleted func(obsolete []*FileBacking)

	// Stats holds aggregated stats about the version maintained from
	// version to version.
	Stats struct {
		// MarkedForCompaction records the count of files marked for
		// compaction within the version.
		MarkedForCompaction int
	}

	// The list the version is linked into.
	list *VersionList

	// The next/prev link for the versionList doubly-linked list of versions.
	prev, next *Version
}

// String implements fmt.Stringer, printing the FileMetadata for each level in
// the Version.
func (v *Version) String() string {
	return v.string(base.DefaultFormatter, false)
}

// DebugString returns an alternative format to String() which includes sequence
// number and kind information for the sstable boundaries.
func (v *Version) DebugString(format base.FormatKey) string {
	return v.string(format, true)
}

func describeSublevels(format base.FormatKey, verbose bool, sublevels []LevelSlice) string {
	var buf bytes.Buffer
	for sublevel := len(sublevels) - 1; sublevel >= 0; sublevel-- {
		fmt.Fprintf(&buf, "0.%d:\n", sublevel)
		sublevels[sublevel].Each(func(f *FileMetadata) {
			fmt.Fprintf(&buf, "  %s\n", f.DebugString(format, verbose))
		})
	}
	return buf.String()
}

func (v *Version) string(format base.FormatKey, verbose bool) string {
	var buf bytes.Buffer
	if len(v.L0SublevelFiles) > 0 {
		fmt.Fprintf(&buf, "%s", describeSublevels(format, verbose, v.L0SublevelFiles))
	}
	for level := 1; level < NumLevels; level++ {
		if v.Levels[level].Empty() {
			continue
		}
		fmt.Fprintf(&buf, "%d:\n", level)
		iter := v.Levels[level].Iter()
		for f := iter.First(); f != nil; f = iter.Next() {
			fmt.Fprintf(&buf, "  %s\n", f.DebugString(format, verbose))
		}
	}
	return buf.String()
}

// ParseVersionDebug parses a Version from its DebugString output.
func ParseVersionDebug(
	cmp Compare, formatKey base.FormatKey, flushSplitBytes int64, s string,
) (*Version, error) {
	var level int
	var files [NumLevels][]*FileMetadata
	for _, l := range strings.Split(s, "\n") {
		l = strings.TrimSpace(l)

		switch l[:2] {
		case "0.", "0:", "1:", "2:", "3:", "4:", "5:", "6:":
			var err error
			level, err = strconv.Atoi(l[:1])
			if err != nil {
				return nil, err
			}
		default:
			m, err := ParseFileMetadataDebug(l)
			if err != nil {
				return nil, err
			}
			// If we only parsed overall bounds, default to setting the point bounds.
			if !m.HasPointKeys && !m.HasRangeKeys {
				m.SmallestPointKey, m.LargestPointKey = m.Smallest, m.Largest
				m.HasPointKeys = true
			}
			files[level] = append(files[level], m)
		}
	}
	// Reverse the order of L0 files. This ensures we construct the same
	// sublevels. (They're printed from higher sublevel to lower, which means in
	// a partial order that represents newest to oldest).
	for i := 0; i < len(files[0])/2; i++ {
		files[0][i], files[0][len(files[0])-i-1] = files[0][len(files[0])-i-1], files[0][i]
	}
	return NewVersion(cmp, formatKey, flushSplitBytes, files), nil
}

// Refs returns the number of references to the version.
func (v *Version) Refs() int32 {
	return v.refs.Load()
}

// Ref increments the version refcount.
func (v *Version) Ref() {
	v.refs.Add(1)
}

// Unref decrements the version refcount. If the last reference to the version
// was removed, the version is removed from the list of versions and the
// Deleted callback is invoked. Requires that the VersionList mutex is NOT
// locked.
func (v *Version) Unref() {
	if v.refs.Add(-1) == 0 {
		l := v.list
		l.mu.Lock()
		l.Remove(v)
		v.Deleted(v.unrefFiles())
		l.mu.Unlock()
	}
}

// UnrefLocked decrements the version refcount. If the last reference to the
// version was removed, the version is removed from the list of versions and
// the Deleted callback is invoked. Requires that the VersionList mutex is
// already locked.
func (v *Version) UnrefLocked() {
	if v.refs.Add(-1) == 0 {
		v.list.Remove(v)
		v.Deleted(v.unrefFiles())
	}
}

func (v *Version) unrefFiles() []*FileBacking {
	var obsolete []*FileBacking
	for _, lm := range v.Levels {
		obsolete = append(obsolete, lm.release()...)
	}
	for _, lm := range v.RangeKeyLevels {
		obsolete = append(obsolete, lm.release()...)
	}
	return obsolete
}

// Next returns the next version in the list of versions.
func (v *Version) Next() *Version {
	return v.next
}

// InitL0Sublevels initializes the L0Sublevels
func (v *Version) InitL0Sublevels(
	cmp Compare, formatKey base.FormatKey, flushSplitBytes int64,
) error {
	var err error
	v.L0Sublevels, err = NewL0Sublevels(&v.Levels[0], cmp, formatKey, flushSplitBytes)
	if err == nil && v.L0Sublevels != nil {
		v.L0SublevelFiles = v.L0Sublevels.Levels
	}
	return err
}

// Contains returns a boolean indicating whether the provided file exists in
// the version at the given level. If level is non-zero then Contains binary
// searches among the files. If level is zero, Contains scans the entire
// level.
func (v *Version) Contains(level int, cmp Compare, m *FileMetadata) bool {
	iter := v.Levels[level].Iter()
	if level > 0 {
		overlaps := v.Overlaps(level, cmp, m.Smallest.UserKey, m.Largest.UserKey,
			m.Largest.IsExclusiveSentinel())
		iter = overlaps.Iter()
	}
	for f := iter.First(); f != nil; f = iter.Next() {
		if f == m {
			return true
		}
	}
	return false
}

// Overlaps returns all elements of v.files[level] whose user key range
// intersects the given range. If level is non-zero then the user key ranges of
// v.files[level] are assumed to not overlap (although they may touch). If level
// is zero then that assumption cannot be made, and the [start, end] range is
// expanded to the union of those matching ranges so far and the computation is
// repeated until [start, end] stabilizes.
// The returned files are a subsequence of the input files, i.e., the ordering
// is not changed.
func (v *Version) Overlaps(
	level int, cmp Compare, start, end []byte, exclusiveEnd bool,
) LevelSlice {
	if level == 0 {
		// Indices that have been selected as overlapping.
		l0 := v.Levels[level]
		l0Iter := l0.Iter()
		selectedIndices := make([]bool, l0.Len())
		numSelected := 0
		var slice LevelSlice
		for {
			restart := false
			for i, meta := 0, l0Iter.First(); meta != nil; i, meta = i+1, l0Iter.Next() {
				selected := selectedIndices[i]
				if selected {
					continue
				}
				if !meta.Overlaps(cmp, start, end, exclusiveEnd) {
					// meta is completely outside the specified range; skip it.
					continue
				}
				// Overlaps.
				selectedIndices[i] = true
				numSelected++

				smallest := meta.Smallest.UserKey
				largest := meta.Largest.UserKey
				// Since level == 0, check if the newly added fileMetadata has
				// expanded the range. We expand the range immediately for files
				// we have remaining to check in this loop. All already checked
				// and unselected files will need to be rechecked via the
				// restart below.
				if cmp(smallest, start) < 0 {
					start = smallest
					restart = true
				}
				if v := cmp(largest, end); v > 0 {
					end = largest
					exclusiveEnd = meta.Largest.IsExclusiveSentinel()
					restart = true
				} else if v == 0 && exclusiveEnd && !meta.Largest.IsExclusiveSentinel() {
					// Only update the exclusivity of our existing `end`
					// bound.
					exclusiveEnd = false
					restart = true
				}
			}

			if !restart {
				// Construct a B-Tree containing only the matching items.
				var tr btree
				tr.cmp = v.Levels[level].tree.cmp
				for i, meta := 0, l0Iter.First(); meta != nil; i, meta = i+1, l0Iter.Next() {
					if selectedIndices[i] {
						err := tr.Insert(meta)
						if err != nil {
							panic(err)
						}
					}
				}
				slice = newLevelSlice(tr.Iter())
				// TODO(jackson): Avoid the oddity of constructing and
				// immediately releasing a B-Tree. Make LevelSlice an
				// interface?
				tr.Release()
				break
			}
			// Continue looping to retry the files that were not selected.
		}
		return slice
	}

	return overlaps(v.Levels[level].Iter(), cmp, start, end, exclusiveEnd)
}

// CheckOrdering checks that the files are consistent with respect to
// increasing file numbers (for level 0 files) and increasing and non-
// overlapping internal key ranges (for level non-0 files).
func (v *Version) CheckOrdering(
	cmp Compare, format base.FormatKey, order OrderingInvariants,
) error {
	for sublevel := len(v.L0SublevelFiles) - 1; sublevel >= 0; sublevel-- {
		sublevelIter := v.L0SublevelFiles[sublevel].Iter()
		// Sublevels have NEVER allowed split user keys, so we can pass
		// ProhibitSplitUserKeys.
		if err := CheckOrdering(cmp, format, L0Sublevel(sublevel), sublevelIter, ProhibitSplitUserKeys); err != nil {
			return base.CorruptionErrorf("%s\n%s", err, v.DebugString(format))
		}
	}

	for level, lm := range v.Levels {
		if err := CheckOrdering(cmp, format, Level(level), lm.Iter(), order); err != nil {
			return base.CorruptionErrorf("%s\n%s", err, v.DebugString(format))
		}
	}
	return nil
}

// VersionList holds a list of versions. The versions are ordered from oldest
// to newest.
type VersionList struct {
	mu   *sync.Mutex
	root Version
}

// Init initializes the version list.
func (l *VersionList) Init(mu *sync.Mutex) {
	l.mu = mu
	l.root.next = &l.root
	l.root.prev = &l.root
}

// Empty returns true if the list is empty, and false otherwise.
func (l *VersionList) Empty() bool {
	return l.root.next == &l.root
}

// Front returns the oldest version in the list. Note that this version is only
// valid if Empty() returns true.
func (l *VersionList) Front() *Version {
	return l.root.next
}

// Back returns the newest version in the list. Note that this version is only
// valid if Empty() returns true.
func (l *VersionList) Back() *Version {
	return l.root.prev
}

// PushBack adds a new version to the back of the list. This new version
// becomes the "newest" version in the list.
func (l *VersionList) PushBack(v *Version) {
	if v.list != nil || v.prev != nil || v.next != nil {
		panic("pebble: version list is inconsistent")
	}
	v.prev = l.root.prev
	v.prev.next = v
	v.next = &l.root
	v.next.prev = v
	v.list = l
	// Let L0Sublevels on the second newest version get GC'd, as it is no longer
	// necessary. See the comment in Version.
	v.prev.L0Sublevels = nil
}

// Remove removes the specified version from the list.
func (l *VersionList) Remove(v *Version) {
	if v == &l.root {
		panic("pebble: cannot remove version list root node")
	}
	if v.list != l {
		panic("pebble: version list is inconsistent")
	}
	v.prev.next = v.next
	v.next.prev = v.prev
	v.next = nil // avoid memory leaks
	v.prev = nil // avoid memory leaks
	v.list = nil // avoid memory leaks
}

// OrderingInvariants dictates the file ordering invariants active.
type OrderingInvariants int8

const (
	// ProhibitSplitUserKeys indicates that adjacent files within a level cannot
	// contain the same user key.
	ProhibitSplitUserKeys OrderingInvariants = iota
	// AllowSplitUserKeys indicates that adjacent files within a level may
	// contain the same user key. This is only allowed by historical format
	// major versions.
	//
	// TODO(jackson): Remove.
	AllowSplitUserKeys
)

// CheckOrdering checks that the files are consistent with respect to
// seqnums (for level 0 files -- see detailed comment below) and increasing and non-
// overlapping internal key ranges (for non-level 0 files).
//
// The ordering field may be passed AllowSplitUserKeys to allow adjacent files that are both
// inclusive of the same user key. Pebble no longer creates version edits
// installing such files, and Pebble databases with sufficiently high format
// major version should no longer have any such files within their LSM.
// TODO(jackson): Remove AllowSplitUserKeys when we remove support for the
// earlier format major versions.
func CheckOrdering(
	cmp Compare, format base.FormatKey, level Level, files LevelIterator, ordering OrderingInvariants,
) error {
	// The invariants to check for L0 sublevels are the same as the ones to
	// check for all other levels. However, if L0 is not organized into
	// sublevels, or if all L0 files are being passed in, we do the legacy L0
	// checks, defined in the detailed comment below.
	if level == Level(0) {
		// We have 2 kinds of files:
		// - Files with exactly one sequence number: these could be either ingested files
		//   or flushed files. We cannot tell the difference between them based on FileMetadata,
		//   so our consistency checking here uses the weaker checks assuming it is a narrow
		//   flushed file. We cannot error on ingested files having sequence numbers coincident
		//   with flushed files as the seemingly ingested file could just be a flushed file
		//   with just one key in it which is a truncated range tombstone sharing sequence numbers
		//   with other files in the same flush.
		// - Files with multiple sequence numbers: these are necessarily flushed files.
		//
		// Three cases of overlapping sequence numbers:
		// Case 1:
		// An ingested file contained in the sequence numbers of the flushed file -- it must be
		// fully contained (not coincident with either end of the flushed file) since the memtable
		// must have been at [a, b-1] (where b > a) when the ingested file was assigned sequence
		// num b, and the memtable got a subsequent update that was given sequence num b+1, before
		// being flushed.
		//
		// So a sequence [1000, 1000] [1002, 1002] [1000, 2000] is invalid since the first and
		// third file are inconsistent with each other. So comparing adjacent files is insufficient
		// for consistency checking.
		//
		// Visually we have something like
		// x------y x-----------yx-------------y (flushed files where x, y are the endpoints)
		//     y       y  y        y             (y's represent ingested files)
		// And these are ordered in increasing order of y. Note that y's must be unique.
		//
		// Case 2:
		// A flushed file that did not overlap in keys with any file in any level, but does overlap
		// in the file key intervals. This file is placed in L0 since it overlaps in the file
		// key intervals but since it has no overlapping data, it is assigned a sequence number
		// of 0 in RocksDB. We handle this case for compatibility with RocksDB.
		//
		// Case 3:
		// A sequence of flushed files that overlap in sequence numbers with one another,
		// but do not overlap in keys inside the sstables. These files correspond to
		// partitioned flushes or the results of intra-L0 compactions of partitioned
		// flushes.
		//
		// Since these types of SSTables violate most other sequence number
		// overlap invariants, and handling this case is important for compatibility
		// with future versions of pebble, this method relaxes most L0 invariant
		// checks.

		var prev *FileMetadata
		for f := files.First(); f != nil; f, prev = files.Next(), f {
			if prev == nil {
				continue
			}
			// Validate that the sorting is sane.
			if prev.LargestSeqNum == 0 && f.LargestSeqNum == prev.LargestSeqNum {
				// Multiple files satisfying case 2 mentioned above.
			} else if !prev.lessSeqNum(f) {
				return base.CorruptionErrorf("L0 files %s and %s are not properly ordered: <#%d-#%d> vs <#%d-#%d>",
					errors.Safe(prev.FileNum), errors.Safe(f.FileNum),
					errors.Safe(prev.SmallestSeqNum), errors.Safe(prev.LargestSeqNum),
					errors.Safe(f.SmallestSeqNum), errors.Safe(f.LargestSeqNum))
			}
		}
	} else {
		var prev *FileMetadata
		for f := files.First(); f != nil; f, prev = files.Next(), f {
			if err := f.Validate(cmp, format); err != nil {
				return errors.Wrapf(err, "%s ", level)
			}
			if prev != nil {
				if prev.cmpSmallestKey(f, cmp) >= 0 {
					return base.CorruptionErrorf("%s files %s and %s are not properly ordered: [%s-%s] vs [%s-%s]",
						errors.Safe(level), errors.Safe(prev.FileNum), errors.Safe(f.FileNum),
						prev.Smallest.Pretty(format), prev.Largest.Pretty(format),
						f.Smallest.Pretty(format), f.Largest.Pretty(format))
				}

				// What's considered "overlapping" is dependent on the format
				// major version. If ordering=ProhibitSplitUserKeys, then both
				// files cannot contain keys with the same user keys. If the
				// bounds have the same user key, the previous file's boundary
				// must have a Trailer indicating that it's exclusive.
				switch ordering {
				case AllowSplitUserKeys:
					if base.InternalCompare(cmp, prev.Largest, f.Smallest) >= 0 {
						return base.CorruptionErrorf("%s files %s and %s have overlapping ranges: [%s-%s] vs [%s-%s]",
							errors.Safe(level), errors.Safe(prev.FileNum), errors.Safe(f.FileNum),
							prev.Smallest.Pretty(format), prev.Largest.Pretty(format),
							f.Smallest.Pretty(format), f.Largest.Pretty(format))
					}
				case ProhibitSplitUserKeys:
					if v := cmp(prev.Largest.UserKey, f.Smallest.UserKey); v > 0 || (v == 0 && !prev.Largest.IsExclusiveSentinel()) {
						return base.CorruptionErrorf("%s files %s and %s have overlapping ranges: [%s-%s] vs [%s-%s]",
							errors.Safe(level), errors.Safe(prev.FileNum), errors.Safe(f.FileNum),
							prev.Smallest.Pretty(format), prev.Largest.Pretty(format),
							f.Smallest.Pretty(format), f.Largest.Pretty(format))
					}
				default:
					panic("unreachable")
				}
			}
		}
	}
	return nil
}
