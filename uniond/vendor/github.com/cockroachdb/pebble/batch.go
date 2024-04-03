// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"context"
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"sort"
	"sync"
	"sync/atomic"
	"time"
	"unsafe"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/batchskl"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/private"
	"github.com/cockroachdb/pebble/internal/rangedel"
	"github.com/cockroachdb/pebble/internal/rangekey"
	"github.com/cockroachdb/pebble/internal/rawalloc"
)

const (
	batchCountOffset     = 8
	batchHeaderLen       = 12
	batchInitialSize     = 1 << 10 // 1 KB
	batchMaxRetainedSize = 1 << 20 // 1 MB
	invalidBatchCount    = 1<<32 - 1
	maxVarintLen32       = 5
)

// ErrNotIndexed means that a read operation on a batch failed because the
// batch is not indexed and thus doesn't support reads.
var ErrNotIndexed = errors.New("pebble: batch not indexed")

// ErrInvalidBatch indicates that a batch is invalid or otherwise corrupted.
var ErrInvalidBatch = base.MarkCorruptionError(errors.New("pebble: invalid batch"))

// ErrBatchTooLarge indicates that a batch is invalid or otherwise corrupted.
var ErrBatchTooLarge = base.MarkCorruptionError(errors.Newf("pebble: batch too large: >= %s", humanize.Bytes.Uint64(maxBatchSize)))

// DeferredBatchOp represents a batch operation (eg. set, merge, delete) that is
// being inserted into the batch. Indexing is not performed on the specified key
// until Finish is called, hence the name deferred. This struct lets the caller
// copy or encode keys/values directly into the batch representation instead of
// copying into an intermediary buffer then having pebble.Batch copy off of it.
type DeferredBatchOp struct {
	index *batchskl.Skiplist

	// Key and Value point to parts of the binary batch representation where
	// keys and values should be encoded/copied into. len(Key) and len(Value)
	// bytes must be copied into these slices respectively before calling
	// Finish(). Changing where these slices point to is not allowed.
	Key, Value []byte
	offset     uint32
}

// Finish completes the addition of this batch operation, and adds it to the
// index if necessary. Must be called once (and exactly once) keys/values
// have been filled into Key and Value. Not calling Finish or not
// copying/encoding keys will result in an incomplete index, and calling Finish
// twice may result in a panic.
func (d DeferredBatchOp) Finish() error {
	if d.index != nil {
		if err := d.index.Add(d.offset); err != nil {
			return err
		}
	}
	return nil
}

// A Batch is a sequence of Sets, Merges, Deletes, DeleteRanges, RangeKeySets,
// RangeKeyUnsets, and/or RangeKeyDeletes that are applied atomically. Batch
// implements the Reader interface, but only an indexed batch supports reading
// (without error) via Get or NewIter. A non-indexed batch will return
// ErrNotIndexed when read from. A batch is not safe for concurrent use, and
// consumers should use a batch per goroutine or provide their own
// synchronization.
//
// # Indexing
//
// Batches can be optionally indexed (see DB.NewIndexedBatch). An indexed batch
// allows iteration via an Iterator (see Batch.NewIter). The iterator provides
// a merged view of the operations in the batch and the underlying
// database. This is implemented by treating the batch as an additional layer
// in the LSM where every entry in the batch is considered newer than any entry
// in the underlying database (batch entries have the InternalKeySeqNumBatch
// bit set). By treating the batch as an additional layer in the LSM, iteration
// supports all batch operations (i.e. Set, Merge, Delete, DeleteRange,
// RangeKeySet, RangeKeyUnset, RangeKeyDelete) with minimal effort.
//
// The same key can be operated on multiple times in a batch, though only the
// latest operation will be visible. For example, Put("a", "b"), Delete("a")
// will cause the key "a" to not be visible in the batch. Put("a", "b"),
// Put("a", "c") will cause a read of "a" to return the value "c".
//
// The batch index is implemented via an skiplist (internal/batchskl). While
// the skiplist implementation is very fast, inserting into an indexed batch is
// significantly slower than inserting into a non-indexed batch. Only use an
// indexed batch if you require reading from it.
//
// # Atomic commit
//
// The operations in a batch are persisted by calling Batch.Commit which is
// equivalent to calling DB.Apply(batch). A batch is committed atomically by
// writing the internal batch representation to the WAL, adding all of the
// batch operations to the memtable associated with the WAL, and then
// incrementing the visible sequence number so that subsequent reads can see
// the effects of the batch operations. If WriteOptions.Sync is true, a call to
// Batch.Commit will guarantee that the batch is persisted to disk before
// returning. See commitPipeline for more on the implementation details.
//
// # Large batches
//
// The size of a batch is limited only by available memory (be aware that
// indexed batches require considerably additional memory for the skiplist
// structure). A given WAL file has a single memtable associated with it (this
// restriction could be removed, but doing so is onerous and complex). And a
// memtable has a fixed size due to the underlying fixed size arena. Note that
// this differs from RocksDB where a memtable can grow arbitrarily large using
// a list of arena chunks. In RocksDB this is accomplished by storing pointers
// in the arena memory, but that isn't possible in Go.
//
// During Batch.Commit, a batch which is larger than a threshold (>
// MemTableSize/2) is wrapped in a flushableBatch and inserted into the queue
// of memtables. A flushableBatch forces WAL to be rotated, but that happens
// anyways when the memtable becomes full so this does not cause significant
// WAL churn. Because the flushableBatch is readable as another layer in the
// LSM, Batch.Commit returns as soon as the flushableBatch has been added to
// the queue of memtables.
//
// Internally, a flushableBatch provides Iterator support by sorting the batch
// contents (the batch is sorted once, when it is added to the memtable
// queue). Sorting the batch contents and insertion of the contents into a
// memtable have the same big-O time, but the constant factor dominates
// here. Sorting is significantly faster and uses significantly less memory.
//
// # Internal representation
//
// The internal batch representation is a contiguous byte buffer with a fixed
// 12-byte header, followed by a series of records.
//
//	+-------------+------------+--- ... ---+
//	| SeqNum (8B) | Count (4B) |  Entries  |
//	+-------------+------------+--- ... ---+
//
// Each record has a 1-byte kind tag prefix, followed by 1 or 2 length prefixed
// strings (varstring):
//
//	+-----------+-----------------+-------------------+
//	| Kind (1B) | Key (varstring) | Value (varstring) |
//	+-----------+-----------------+-------------------+
//
// A varstring is a varint32 followed by N bytes of data. The Kind tags are
// exactly those specified by InternalKeyKind. The following table shows the
// format for records of each kind:
//
//	InternalKeyKindDelete         varstring
//	InternalKeyKindLogData        varstring
//	InternalKeyKindIngestSST      varstring
//	InternalKeyKindSet            varstring varstring
//	InternalKeyKindMerge          varstring varstring
//	InternalKeyKindRangeDelete    varstring varstring
//	InternalKeyKindRangeKeySet    varstring varstring
//	InternalKeyKindRangeKeyUnset  varstring varstring
//	InternalKeyKindRangeKeyDelete varstring varstring
//
// The intuitive understanding here are that the arguments to Delete, Set,
// Merge, DeleteRange and RangeKeyDelete are encoded into the batch. The
// RangeKeySet and RangeKeyUnset operations are slightly more complicated,
// encoding their end key, suffix and value [in the case of RangeKeySet] within
// the Value varstring. For more information on the value encoding for
// RangeKeySet and RangeKeyUnset, see the internal/rangekey package.
//
// The internal batch representation is the on disk format for a batch in the
// WAL, and thus stable. New record kinds may be added, but the existing ones
// will not be modified.
type Batch struct {
	batchInternal
	applied atomic.Bool
}

// batchInternal contains the set of fields within Batch that are non-atomic and
// capable of being reset using a *b = batchInternal{} struct copy.
type batchInternal struct {
	// Data is the wire format of a batch's log entry:
	//   - 8 bytes for a sequence number of the first batch element,
	//     or zeroes if the batch has not yet been applied,
	//   - 4 bytes for the count: the number of elements in the batch,
	//     or "\xff\xff\xff\xff" if the batch is invalid,
	//   - count elements, being:
	//     - one byte for the kind
	//     - the varint-string user key,
	//     - the varint-string value (if kind != delete).
	// The sequence number and count are stored in little-endian order.
	//
	// The data field can be (but is not guaranteed to be) nil for new
	// batches. Large batches will set the data field to nil when committed as
	// the data has been moved to a flushableBatch and inserted into the queue of
	// memtables.
	data           []byte
	cmp            Compare
	formatKey      base.FormatKey
	abbreviatedKey AbbreviatedKey

	// An upper bound on required space to add this batch to a memtable.
	// Note that although batches are limited to 4 GiB in size, that limit
	// applies to len(data), not the memtable size. The upper bound on the
	// size of a memtable node is larger than the overhead of the batch's log
	// encoding, so memTableSize is larger than len(data) and may overflow a
	// uint32.
	memTableSize uint64

	// The db to which the batch will be committed. Do not change this field
	// after the batch has been created as it might invalidate internal state.
	// Batch.memTableSize is only refreshed if Batch.db is set. Setting db to
	// nil once it has been set implies that the Batch has encountered an error.
	db *DB

	// The count of records in the batch. This count will be stored in the batch
	// data whenever Repr() is called.
	count uint64

	// The count of range deletions in the batch. Updated every time a range
	// deletion is added.
	countRangeDels uint64

	// The count of range key sets, unsets and deletes in the batch. Updated
	// every time a RANGEKEYSET, RANGEKEYUNSET or RANGEKEYDEL key is added.
	countRangeKeys uint64

	// A deferredOp struct, stored in the Batch so that a pointer can be returned
	// from the *Deferred() methods rather than a value.
	deferredOp DeferredBatchOp

	// An optional skiplist keyed by offset into data of the entry.
	index         *batchskl.Skiplist
	rangeDelIndex *batchskl.Skiplist
	rangeKeyIndex *batchskl.Skiplist

	// Fragmented range deletion tombstones. Cached the first time a range
	// deletion iterator is requested. The cache is invalidated whenever a new
	// range deletion is added to the batch. This cache can only be used when
	// opening an iterator to read at a batch sequence number >=
	// tombstonesSeqNum. This is the case for all new iterators created over a
	// batch but it's not the case for all cloned iterators.
	tombstones       []keyspan.Span
	tombstonesSeqNum uint64

	// Fragmented range key spans. Cached the first time a range key iterator is
	// requested. The cache is invalidated whenever a new range key
	// (RangeKey{Set,Unset,Del}) is added to the batch. This cache can only be
	// used when opening an iterator to read at a batch sequence number >=
	// tombstonesSeqNum. This is the case for all new iterators created over a
	// batch but it's not the case for all cloned iterators.
	rangeKeys       []keyspan.Span
	rangeKeysSeqNum uint64

	// The flushableBatch wrapper if the batch is too large to fit in the
	// memtable.
	flushable *flushableBatch

	// minimumFormatMajorVersion indicates the format major version required in
	// order to commit this batch. If an operation requires a particular format
	// major version, it ratchets the batch's minimumFormatMajorVersion. When
	// the batch is committed, this is validated against the database's current
	// format major version.
	minimumFormatMajorVersion FormatMajorVersion

	// Synchronous Apply uses the commit WaitGroup for both publishing the
	// seqnum and waiting for the WAL fsync (if needed). Asynchronous
	// ApplyNoSyncWait, which implies WriteOptions.Sync is true, uses the commit
	// WaitGroup for publishing the seqnum and the fsyncWait WaitGroup for
	// waiting for the WAL fsync.
	//
	// TODO(sumeer): if we find that ApplyNoSyncWait in conjunction with
	// SyncWait is causing higher memory usage because of the time duration
	// between when the sync is already done, and a goroutine calls SyncWait
	// (followed by Batch.Close), we could separate out {fsyncWait, commitErr}
	// into a separate struct that is allocated separately (using another
	// sync.Pool), and only that struct needs to outlive Batch.Close (which
	// could then be called immediately after ApplyNoSyncWait). commitStats
	// will also need to be in this separate struct.
	commit    sync.WaitGroup
	fsyncWait sync.WaitGroup

	commitStats BatchCommitStats

	commitErr error

	// Position bools together to reduce the sizeof the struct.

	// ingestedSSTBatch indicates that the batch contains one or more key kinds
	// of InternalKeyKindIngestSST. If the batch contains key kinds of IngestSST
	// then it will only contain key kinds of IngestSST.
	ingestedSSTBatch bool

	// committing is set to true when a batch begins to commit. It's used to
	// ensure the batch is not mutated concurrently. It is not an atomic
	// deliberately, so as to avoid the overhead on batch mutations. This is
	// okay, because under correct usage this field will never be accessed
	// concurrently. It's only under incorrect usage the memory accesses of this
	// variable may violate memory safety. Since we don't use atomics here,
	// false negatives are possible.
	committing bool
}

// BatchCommitStats exposes stats related to committing a batch.
//
// NB: there is no Pebble internal tracing (using LoggerAndTracer) of slow
// batch commits. The caller can use these stats to do their own tracing as
// needed.
type BatchCommitStats struct {
	// TotalDuration is the time spent in DB.{Apply,ApplyNoSyncWait} or
	// Batch.Commit, plus the time waiting in Batch.SyncWait. If there is a gap
	// between calling ApplyNoSyncWait and calling SyncWait, that gap could
	// include some duration in which real work was being done for the commit
	// and will not be included here. This missing time is considered acceptable
	// since the goal of these stats is to understand user-facing latency.
	//
	// TotalDuration includes time spent in various queues both inside Pebble
	// and outside Pebble (I/O queues, goroutine scheduler queue, mutex wait
	// etc.). For some of these queues (which we consider important) the wait
	// times are included below -- these expose low-level implementation detail
	// and are meant for expert diagnosis and subject to change. There may be
	// unaccounted time after subtracting those values from TotalDuration.
	TotalDuration time.Duration
	// SemaphoreWaitDuration is the wait time for semaphores in
	// commitPipeline.Commit.
	SemaphoreWaitDuration time.Duration
	// WALQueueWaitDuration is the wait time for allocating memory blocks in the
	// LogWriter (due to the LogWriter not writing fast enough). At the moment
	// this is duration is always zero because a single WAL will allow
	// allocating memory blocks up to the entire memtable size. In the future,
	// we may pipeline WALs and bound the WAL queued blocks separately, so this
	// field is preserved for that possibility.
	WALQueueWaitDuration time.Duration
	// MemTableWriteStallDuration is the wait caused by a write stall due to too
	// many memtables (due to not flushing fast enough).
	MemTableWriteStallDuration time.Duration
	// L0ReadAmpWriteStallDuration is the wait caused by a write stall due to
	// high read amplification in L0 (due to not compacting fast enough out of
	// L0).
	L0ReadAmpWriteStallDuration time.Duration
	// WALRotationDuration is the wait time for WAL rotation, which includes
	// syncing and closing the old WAL and creating (or reusing) a new one.
	WALRotationDuration time.Duration
	// CommitWaitDuration is the wait for publishing the seqnum plus the
	// duration for the WAL sync (if requested). The former should be tiny and
	// one can assume that this is all due to the WAL sync.
	CommitWaitDuration time.Duration
}

var _ Reader = (*Batch)(nil)
var _ Writer = (*Batch)(nil)

var batchPool = sync.Pool{
	New: func() interface{} {
		return &Batch{}
	},
}

type indexedBatch struct {
	batch Batch
	index batchskl.Skiplist
}

var indexedBatchPool = sync.Pool{
	New: func() interface{} {
		return &indexedBatch{}
	},
}

func newBatch(db *DB) *Batch {
	b := batchPool.Get().(*Batch)
	b.db = db
	return b
}

func newBatchWithSize(db *DB, size int) *Batch {
	b := newBatch(db)
	if cap(b.data) < size {
		b.data = rawalloc.New(0, size)
	}
	return b
}

func newIndexedBatch(db *DB, comparer *Comparer) *Batch {
	i := indexedBatchPool.Get().(*indexedBatch)
	i.batch.cmp = comparer.Compare
	i.batch.formatKey = comparer.FormatKey
	i.batch.abbreviatedKey = comparer.AbbreviatedKey
	i.batch.db = db
	i.batch.index = &i.index
	i.batch.index.Init(&i.batch.data, i.batch.cmp, i.batch.abbreviatedKey)
	return &i.batch
}

func newIndexedBatchWithSize(db *DB, comparer *Comparer, size int) *Batch {
	b := newIndexedBatch(db, comparer)
	if cap(b.data) < size {
		b.data = rawalloc.New(0, size)
	}
	return b
}

// nextSeqNum returns the batch "sequence number" that will be given to the next
// key written to the batch. During iteration keys within an indexed batch are
// given a sequence number consisting of their offset within the batch combined
// with the base.InternalKeySeqNumBatch bit. These sequence numbers are only
// used during iteration, and the keys are assigned ordinary sequence numbers
// when the batch is committed.
func (b *Batch) nextSeqNum() uint64 {
	return uint64(len(b.data)) | base.InternalKeySeqNumBatch
}

func (b *Batch) release() {
	if b.db == nil {
		// The batch was not created using newBatch or newIndexedBatch, or an error
		// was encountered. We don't try to reuse batches that encountered an error
		// because they might be stuck somewhere in the system and attempting to
		// reuse such batches is a recipe for onerous debugging sessions. Instead,
		// let the GC do its job.
		return
	}
	b.db = nil

	// NB: This is ugly (it would be cleaner if we could just assign a Batch{}),
	// but necessary so that we can use atomic.StoreUint32 for the Batch.applied
	// field. Without using an atomic to clear that field the Go race detector
	// complains.
	b.Reset()
	b.cmp = nil
	b.formatKey = nil
	b.abbreviatedKey = nil

	if b.index == nil {
		batchPool.Put(b)
	} else {
		b.index, b.rangeDelIndex, b.rangeKeyIndex = nil, nil, nil
		indexedBatchPool.Put((*indexedBatch)(unsafe.Pointer(b)))
	}
}

func (b *Batch) refreshMemTableSize() error {
	b.memTableSize = 0
	if len(b.data) < batchHeaderLen {
		return nil
	}

	b.countRangeDels = 0
	b.countRangeKeys = 0
	b.minimumFormatMajorVersion = 0
	for r := b.Reader(); ; {
		kind, key, value, ok, err := r.Next()
		if !ok {
			if err != nil {
				return err
			}
			break
		}
		switch kind {
		case InternalKeyKindRangeDelete:
			b.countRangeDels++
		case InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete:
			b.countRangeKeys++
		case InternalKeyKindDeleteSized:
			if b.minimumFormatMajorVersion < FormatDeleteSizedAndObsolete {
				b.minimumFormatMajorVersion = FormatDeleteSizedAndObsolete
			}
		case InternalKeyKindIngestSST:
			if b.minimumFormatMajorVersion < FormatFlushableIngest {
				b.minimumFormatMajorVersion = FormatFlushableIngest
			}
			// This key kind doesn't contribute to the memtable size.
			continue
		}
		b.memTableSize += memTableEntrySize(len(key), len(value))
	}
	if b.countRangeKeys > 0 && b.minimumFormatMajorVersion < FormatRangeKeys {
		b.minimumFormatMajorVersion = FormatRangeKeys
	}
	return nil
}

// Apply the operations contained in the batch to the receiver batch.
//
// It is safe to modify the contents of the arguments after Apply returns.
func (b *Batch) Apply(batch *Batch, _ *WriteOptions) error {
	if b.ingestedSSTBatch {
		panic("pebble: invalid batch application")
	}
	if len(batch.data) == 0 {
		return nil
	}
	if len(batch.data) < batchHeaderLen {
		return ErrInvalidBatch
	}

	offset := len(b.data)
	if offset == 0 {
		b.init(offset)
		offset = batchHeaderLen
	}
	b.data = append(b.data, batch.data[batchHeaderLen:]...)

	b.setCount(b.Count() + batch.Count())

	if b.db != nil || b.index != nil {
		// Only iterate over the new entries if we need to track memTableSize or in
		// order to update the index.
		for iter := BatchReader(b.data[offset:]); len(iter) > 0; {
			offset := uintptr(unsafe.Pointer(&iter[0])) - uintptr(unsafe.Pointer(&b.data[0]))
			kind, key, value, ok, err := iter.Next()
			if !ok {
				if err != nil {
					return err
				}
				break
			}
			switch kind {
			case InternalKeyKindRangeDelete:
				b.countRangeDels++
			case InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete:
				b.countRangeKeys++
			case InternalKeyKindIngestSST:
				panic("pebble: invalid key kind for batch")
			}
			if b.index != nil {
				var err error
				switch kind {
				case InternalKeyKindRangeDelete:
					b.tombstones = nil
					b.tombstonesSeqNum = 0
					if b.rangeDelIndex == nil {
						b.rangeDelIndex = batchskl.NewSkiplist(&b.data, b.cmp, b.abbreviatedKey)
					}
					err = b.rangeDelIndex.Add(uint32(offset))
				case InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete:
					b.rangeKeys = nil
					b.rangeKeysSeqNum = 0
					if b.rangeKeyIndex == nil {
						b.rangeKeyIndex = batchskl.NewSkiplist(&b.data, b.cmp, b.abbreviatedKey)
					}
					err = b.rangeKeyIndex.Add(uint32(offset))
				default:
					err = b.index.Add(uint32(offset))
				}
				if err != nil {
					return err
				}
			}
			b.memTableSize += memTableEntrySize(len(key), len(value))
		}
	}
	return nil
}

// Get gets the value for the given key. It returns ErrNotFound if the Batch
// does not contain the key.
//
// The caller should not modify the contents of the returned slice, but it is
// safe to modify the contents of the argument after Get returns. The returned
// slice will remain valid until the returned Closer is closed. On success, the
// caller MUST call closer.Close() or a memory leak will occur.
func (b *Batch) Get(key []byte) ([]byte, io.Closer, error) {
	if b.index == nil {
		return nil, nil, ErrNotIndexed
	}
	return b.db.getInternal(key, b, nil /* snapshot */)
}

func (b *Batch) prepareDeferredKeyValueRecord(keyLen, valueLen int, kind InternalKeyKind) {
	if b.committing {
		panic("pebble: batch already committing")
	}
	if len(b.data) == 0 {
		b.init(keyLen + valueLen + 2*binary.MaxVarintLen64 + batchHeaderLen)
	}
	b.count++
	b.memTableSize += memTableEntrySize(keyLen, valueLen)

	pos := len(b.data)
	b.deferredOp.offset = uint32(pos)
	b.grow(1 + 2*maxVarintLen32 + keyLen + valueLen)
	b.data[pos] = byte(kind)
	pos++

	{
		// TODO(peter): Manually inlined version binary.PutUvarint(). This is 20%
		// faster on BenchmarkBatchSet on go1.13. Remove if go1.14 or future
		// versions show this to not be a performance win.
		x := uint32(keyLen)
		for x >= 0x80 {
			b.data[pos] = byte(x) | 0x80
			x >>= 7
			pos++
		}
		b.data[pos] = byte(x)
		pos++
	}

	b.deferredOp.Key = b.data[pos : pos+keyLen]
	pos += keyLen

	{
		// TODO(peter): Manually inlined version binary.PutUvarint(). This is 20%
		// faster on BenchmarkBatchSet on go1.13. Remove if go1.14 or future
		// versions show this to not be a performance win.
		x := uint32(valueLen)
		for x >= 0x80 {
			b.data[pos] = byte(x) | 0x80
			x >>= 7
			pos++
		}
		b.data[pos] = byte(x)
		pos++
	}

	b.deferredOp.Value = b.data[pos : pos+valueLen]
	// Shrink data since varints may be shorter than the upper bound.
	b.data = b.data[:pos+valueLen]
}

func (b *Batch) prepareDeferredKeyRecord(keyLen int, kind InternalKeyKind) {
	if b.committing {
		panic("pebble: batch already committing")
	}
	if len(b.data) == 0 {
		b.init(keyLen + binary.MaxVarintLen64 + batchHeaderLen)
	}
	b.count++
	b.memTableSize += memTableEntrySize(keyLen, 0)

	pos := len(b.data)
	b.deferredOp.offset = uint32(pos)
	b.grow(1 + maxVarintLen32 + keyLen)
	b.data[pos] = byte(kind)
	pos++

	{
		// TODO(peter): Manually inlined version binary.PutUvarint(). Remove if
		// go1.13 or future versions show this to not be a performance win. See
		// BenchmarkBatchSet.
		x := uint32(keyLen)
		for x >= 0x80 {
			b.data[pos] = byte(x) | 0x80
			x >>= 7
			pos++
		}
		b.data[pos] = byte(x)
		pos++
	}

	b.deferredOp.Key = b.data[pos : pos+keyLen]
	b.deferredOp.Value = nil

	// Shrink data since varint may be shorter than the upper bound.
	b.data = b.data[:pos+keyLen]
}

// AddInternalKey allows the caller to add an internal key of point key kinds to
// a batch. Passing in an internal key of kind RangeKey* or RangeDelete will
// result in a panic. Note that the seqnum in the internal key is effectively
// ignored, even though the Kind is preserved. This is because the batch format
// does not allow for a per-key seqnum to be specified, only a batch-wide one.
//
// Note that non-indexed keys (IngestKeyKind{LogData,IngestSST}) are not
// supported with this method as they require specialized logic.
func (b *Batch) AddInternalKey(key *base.InternalKey, value []byte, _ *WriteOptions) error {
	keyLen := len(key.UserKey)
	hasValue := false
	switch key.Kind() {
	case InternalKeyKindRangeDelete, InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete:
		panic("unexpected range delete or range key kind in AddInternalKey")
	case InternalKeyKindSingleDelete, InternalKeyKindDelete:
		b.prepareDeferredKeyRecord(len(key.UserKey), key.Kind())
	default:
		b.prepareDeferredKeyValueRecord(keyLen, len(value), key.Kind())
		hasValue = true
	}
	b.deferredOp.index = b.index
	copy(b.deferredOp.Key, key.UserKey)
	if hasValue {
		copy(b.deferredOp.Value, value)
	}
	// TODO(peter): Manually inline DeferredBatchOp.Finish(). Mid-stack inlining
	// in go1.13 will remove the need for this.
	if b.index != nil {
		if err := b.index.Add(b.deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

// Set adds an action to the batch that sets the key to map to the value.
//
// It is safe to modify the contents of the arguments after Set returns.
func (b *Batch) Set(key, value []byte, _ *WriteOptions) error {
	deferredOp := b.SetDeferred(len(key), len(value))
	copy(deferredOp.Key, key)
	copy(deferredOp.Value, value)
	// TODO(peter): Manually inline DeferredBatchOp.Finish(). Mid-stack inlining
	// in go1.13 will remove the need for this.
	if b.index != nil {
		if err := b.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

// SetDeferred is similar to Set in that it adds a set operation to the batch,
// except it only takes in key/value lengths instead of complete slices,
// letting the caller encode into those objects and then call Finish() on the
// returned object.
func (b *Batch) SetDeferred(keyLen, valueLen int) *DeferredBatchOp {
	b.prepareDeferredKeyValueRecord(keyLen, valueLen, InternalKeyKindSet)
	b.deferredOp.index = b.index
	return &b.deferredOp
}

// Merge adds an action to the batch that merges the value at key with the new
// value. The details of the merge are dependent upon the configured merge
// operator.
//
// It is safe to modify the contents of the arguments after Merge returns.
func (b *Batch) Merge(key, value []byte, _ *WriteOptions) error {
	deferredOp := b.MergeDeferred(len(key), len(value))
	copy(deferredOp.Key, key)
	copy(deferredOp.Value, value)
	// TODO(peter): Manually inline DeferredBatchOp.Finish(). Mid-stack inlining
	// in go1.13 will remove the need for this.
	if b.index != nil {
		if err := b.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

// MergeDeferred is similar to Merge in that it adds a merge operation to the
// batch, except it only takes in key/value lengths instead of complete slices,
// letting the caller encode into those objects and then call Finish() on the
// returned object.
func (b *Batch) MergeDeferred(keyLen, valueLen int) *DeferredBatchOp {
	b.prepareDeferredKeyValueRecord(keyLen, valueLen, InternalKeyKindMerge)
	b.deferredOp.index = b.index
	return &b.deferredOp
}

// Delete adds an action to the batch that deletes the entry for key.
//
// It is safe to modify the contents of the arguments after Delete returns.
func (b *Batch) Delete(key []byte, _ *WriteOptions) error {
	deferredOp := b.DeleteDeferred(len(key))
	copy(deferredOp.Key, key)
	// TODO(peter): Manually inline DeferredBatchOp.Finish(). Mid-stack inlining
	// in go1.13 will remove the need for this.
	if b.index != nil {
		if err := b.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

// DeleteDeferred is similar to Delete in that it adds a delete operation to
// the batch, except it only takes in key/value lengths instead of complete
// slices, letting the caller encode into those objects and then call Finish()
// on the returned object.
func (b *Batch) DeleteDeferred(keyLen int) *DeferredBatchOp {
	b.prepareDeferredKeyRecord(keyLen, InternalKeyKindDelete)
	b.deferredOp.index = b.index
	return &b.deferredOp
}

// DeleteSized behaves identically to Delete, but takes an additional
// argument indicating the size of the value being deleted. DeleteSized
// should be preferred when the caller has the expectation that there exists
// a single internal KV pair for the key (eg, the key has not been
// overwritten recently), and the caller knows the size of its value.
//
// DeleteSized will record the value size within the tombstone and use it to
// inform compaction-picking heuristics which strive to reduce space
// amplification in the LSM. This "calling your shot" mechanic allows the
// storage engine to more accurately estimate and reduce space amplification.
//
// It is safe to modify the contents of the arguments after DeleteSized
// returns.
func (b *Batch) DeleteSized(key []byte, deletedValueSize uint32, _ *WriteOptions) error {
	deferredOp := b.DeleteSizedDeferred(len(key), deletedValueSize)
	copy(b.deferredOp.Key, key)
	// TODO(peter): Manually inline DeferredBatchOp.Finish(). Check if in a
	// later Go release this is unnecessary.
	if b.index != nil {
		if err := b.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

// DeleteSizedDeferred is similar to DeleteSized in that it adds a sized delete
// operation to the batch, except it only takes in key length instead of a
// complete key slice, letting the caller encode into the DeferredBatchOp.Key
// slice and then call Finish() on the returned object.
func (b *Batch) DeleteSizedDeferred(keyLen int, deletedValueSize uint32) *DeferredBatchOp {
	if b.minimumFormatMajorVersion < FormatDeleteSizedAndObsolete {
		b.minimumFormatMajorVersion = FormatDeleteSizedAndObsolete
	}

	// Encode the sum of the key length and the value in the value.
	v := uint64(deletedValueSize) + uint64(keyLen)

	// Encode `v` as a varint.
	var buf [binary.MaxVarintLen64]byte
	n := 0
	{
		x := v
		for x >= 0x80 {
			buf[n] = byte(x) | 0x80
			x >>= 7
			n++
		}
		buf[n] = byte(x)
		n++
	}

	// NB: In batch entries and sstable entries, values are stored as
	// varstrings. Here, the value is itself a simple varint. This results in an
	// unnecessary double layer of encoding:
	//     varint(n) varint(deletedValueSize)
	// The first varint will always be 1-byte, since a varint-encoded uint64
	// will never exceed 128 bytes. This unnecessary extra byte and wrapping is
	// preserved to avoid special casing across the database, and in particular
	// in sstable block decoding which is performance sensitive.
	b.prepareDeferredKeyValueRecord(keyLen, n, InternalKeyKindDeleteSized)
	b.deferredOp.index = b.index
	copy(b.deferredOp.Value, buf[:n])
	return &b.deferredOp
}

// SingleDelete adds an action to the batch that single deletes the entry for key.
// See Writer.SingleDelete for more details on the semantics of SingleDelete.
//
// It is safe to modify the contents of the arguments after SingleDelete returns.
func (b *Batch) SingleDelete(key []byte, _ *WriteOptions) error {
	deferredOp := b.SingleDeleteDeferred(len(key))
	copy(deferredOp.Key, key)
	// TODO(peter): Manually inline DeferredBatchOp.Finish(). Mid-stack inlining
	// in go1.13 will remove the need for this.
	if b.index != nil {
		if err := b.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

// SingleDeleteDeferred is similar to SingleDelete in that it adds a single delete
// operation to the batch, except it only takes in key/value lengths instead of
// complete slices, letting the caller encode into those objects and then call
// Finish() on the returned object.
func (b *Batch) SingleDeleteDeferred(keyLen int) *DeferredBatchOp {
	b.prepareDeferredKeyRecord(keyLen, InternalKeyKindSingleDelete)
	b.deferredOp.index = b.index
	return &b.deferredOp
}

// DeleteRange deletes all of the point keys (and values) in the range
// [start,end) (inclusive on start, exclusive on end). DeleteRange does NOT
// delete overlapping range keys (eg, keys set via RangeKeySet).
//
// It is safe to modify the contents of the arguments after DeleteRange
// returns.
func (b *Batch) DeleteRange(start, end []byte, _ *WriteOptions) error {
	deferredOp := b.DeleteRangeDeferred(len(start), len(end))
	copy(deferredOp.Key, start)
	copy(deferredOp.Value, end)
	// TODO(peter): Manually inline DeferredBatchOp.Finish(). Mid-stack inlining
	// in go1.13 will remove the need for this.
	if deferredOp.index != nil {
		if err := deferredOp.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

// DeleteRangeDeferred is similar to DeleteRange in that it adds a delete range
// operation to the batch, except it only takes in key lengths instead of
// complete slices, letting the caller encode into those objects and then call
// Finish() on the returned object. Note that DeferredBatchOp.Key should be
// populated with the start key, and DeferredBatchOp.Value should be populated
// with the end key.
func (b *Batch) DeleteRangeDeferred(startLen, endLen int) *DeferredBatchOp {
	b.prepareDeferredKeyValueRecord(startLen, endLen, InternalKeyKindRangeDelete)
	b.countRangeDels++
	if b.index != nil {
		b.tombstones = nil
		b.tombstonesSeqNum = 0
		// Range deletions are rare, so we lazily allocate the index for them.
		if b.rangeDelIndex == nil {
			b.rangeDelIndex = batchskl.NewSkiplist(&b.data, b.cmp, b.abbreviatedKey)
		}
		b.deferredOp.index = b.rangeDelIndex
	}
	return &b.deferredOp
}

// RangeKeySet sets a range key mapping the key range [start, end) at the MVCC
// timestamp suffix to value. The suffix is optional. If any portion of the key
// range [start, end) is already set by a range key with the same suffix value,
// RangeKeySet overrides it.
//
// It is safe to modify the contents of the arguments after RangeKeySet returns.
func (b *Batch) RangeKeySet(start, end, suffix, value []byte, _ *WriteOptions) error {
	suffixValues := [1]rangekey.SuffixValue{{Suffix: suffix, Value: value}}
	internalValueLen := rangekey.EncodedSetValueLen(end, suffixValues[:])

	deferredOp := b.rangeKeySetDeferred(len(start), internalValueLen)
	copy(deferredOp.Key, start)
	n := rangekey.EncodeSetValue(deferredOp.Value, end, suffixValues[:])
	if n != internalValueLen {
		panic("unexpected internal value length mismatch")
	}

	// Manually inline DeferredBatchOp.Finish().
	if deferredOp.index != nil {
		if err := deferredOp.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

func (b *Batch) rangeKeySetDeferred(startLen, internalValueLen int) *DeferredBatchOp {
	b.prepareDeferredKeyValueRecord(startLen, internalValueLen, InternalKeyKindRangeKeySet)
	b.incrementRangeKeysCount()
	return &b.deferredOp
}

func (b *Batch) incrementRangeKeysCount() {
	b.countRangeKeys++
	if b.minimumFormatMajorVersion < FormatRangeKeys {
		b.minimumFormatMajorVersion = FormatRangeKeys
	}
	if b.index != nil {
		b.rangeKeys = nil
		b.rangeKeysSeqNum = 0
		// Range keys are rare, so we lazily allocate the index for them.
		if b.rangeKeyIndex == nil {
			b.rangeKeyIndex = batchskl.NewSkiplist(&b.data, b.cmp, b.abbreviatedKey)
		}
		b.deferredOp.index = b.rangeKeyIndex
	}
}

// RangeKeyUnset removes a range key mapping the key range [start, end) at the
// MVCC timestamp suffix. The suffix may be omitted to remove an unsuffixed
// range key. RangeKeyUnset only removes portions of range keys that fall within
// the [start, end) key span, and only range keys with suffixes that exactly
// match the unset suffix.
//
// It is safe to modify the contents of the arguments after RangeKeyUnset
// returns.
func (b *Batch) RangeKeyUnset(start, end, suffix []byte, _ *WriteOptions) error {
	suffixes := [1][]byte{suffix}
	internalValueLen := rangekey.EncodedUnsetValueLen(end, suffixes[:])

	deferredOp := b.rangeKeyUnsetDeferred(len(start), internalValueLen)
	copy(deferredOp.Key, start)
	n := rangekey.EncodeUnsetValue(deferredOp.Value, end, suffixes[:])
	if n != internalValueLen {
		panic("unexpected internal value length mismatch")
	}

	// Manually inline DeferredBatchOp.Finish()
	if deferredOp.index != nil {
		if err := deferredOp.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

func (b *Batch) rangeKeyUnsetDeferred(startLen, internalValueLen int) *DeferredBatchOp {
	b.prepareDeferredKeyValueRecord(startLen, internalValueLen, InternalKeyKindRangeKeyUnset)
	b.incrementRangeKeysCount()
	return &b.deferredOp
}

// RangeKeyDelete deletes all of the range keys in the range [start,end)
// (inclusive on start, exclusive on end). It does not delete point keys (for
// that use DeleteRange). RangeKeyDelete removes all range keys within the
// bounds, including those with or without suffixes.
//
// It is safe to modify the contents of the arguments after RangeKeyDelete
// returns.
func (b *Batch) RangeKeyDelete(start, end []byte, _ *WriteOptions) error {
	deferredOp := b.RangeKeyDeleteDeferred(len(start), len(end))
	copy(deferredOp.Key, start)
	copy(deferredOp.Value, end)
	// Manually inline DeferredBatchOp.Finish().
	if deferredOp.index != nil {
		if err := deferredOp.index.Add(deferredOp.offset); err != nil {
			return err
		}
	}
	return nil
}

// RangeKeyDeleteDeferred is similar to RangeKeyDelete in that it adds an
// operation to delete range keys to the batch, except it only takes in key
// lengths instead of complete slices, letting the caller encode into those
// objects and then call Finish() on the returned object. Note that
// DeferredBatchOp.Key should be populated with the start key, and
// DeferredBatchOp.Value should be populated with the end key.
func (b *Batch) RangeKeyDeleteDeferred(startLen, endLen int) *DeferredBatchOp {
	b.prepareDeferredKeyValueRecord(startLen, endLen, InternalKeyKindRangeKeyDelete)
	b.incrementRangeKeysCount()
	return &b.deferredOp
}

// LogData adds the specified to the batch. The data will be written to the
// WAL, but not added to memtables or sstables. Log data is never indexed,
// which makes it useful for testing WAL performance.
//
// It is safe to modify the contents of the argument after LogData returns.
func (b *Batch) LogData(data []byte, _ *WriteOptions) error {
	origCount, origMemTableSize := b.count, b.memTableSize
	b.prepareDeferredKeyRecord(len(data), InternalKeyKindLogData)
	copy(b.deferredOp.Key, data)
	// Since LogData only writes to the WAL and does not affect the memtable, we
	// restore b.count and b.memTableSize to their origin values. Note that
	// Batch.count only refers to records that are added to the memtable.
	b.count, b.memTableSize = origCount, origMemTableSize
	return nil
}

// IngestSST adds the FileNum for an sstable to the batch. The data will only be
// written to the WAL (not added to memtables or sstables).
func (b *Batch) ingestSST(fileNum base.FileNum) {
	if b.Empty() {
		b.ingestedSSTBatch = true
	} else if !b.ingestedSSTBatch {
		// Batch contains other key kinds.
		panic("pebble: invalid call to ingestSST")
	}

	origMemTableSize := b.memTableSize
	var buf [binary.MaxVarintLen64]byte
	length := binary.PutUvarint(buf[:], uint64(fileNum))
	b.prepareDeferredKeyRecord(length, InternalKeyKindIngestSST)
	copy(b.deferredOp.Key, buf[:length])
	// Since IngestSST writes only to the WAL and does not affect the memtable,
	// we restore b.memTableSize to its original value. Note that Batch.count
	// is not reset because for the InternalKeyKindIngestSST the count is the
	// number of sstable paths which have been added to the batch.
	b.memTableSize = origMemTableSize
	b.minimumFormatMajorVersion = FormatFlushableIngest
}

// Empty returns true if the batch is empty, and false otherwise.
func (b *Batch) Empty() bool {
	return len(b.data) <= batchHeaderLen
}

// Len returns the current size of the batch in bytes.
func (b *Batch) Len() int {
	if len(b.data) <= batchHeaderLen {
		return batchHeaderLen
	}
	return len(b.data)
}

// Repr returns the underlying batch representation. It is not safe to modify
// the contents. Reset() will not change the contents of the returned value,
// though any other mutation operation may do so.
func (b *Batch) Repr() []byte {
	if len(b.data) == 0 {
		b.init(batchHeaderLen)
	}
	binary.LittleEndian.PutUint32(b.countData(), b.Count())
	return b.data
}

// SetRepr sets the underlying batch representation. The batch takes ownership
// of the supplied slice. It is not safe to modify it afterwards until the
// Batch is no longer in use.
func (b *Batch) SetRepr(data []byte) error {
	if len(data) < batchHeaderLen {
		return base.CorruptionErrorf("invalid batch")
	}
	b.data = data
	b.count = uint64(binary.LittleEndian.Uint32(b.countData()))
	var err error
	if b.db != nil {
		// Only track memTableSize for batches that will be committed to the DB.
		err = b.refreshMemTableSize()
	}
	return err
}

// NewIter returns an iterator that is unpositioned (Iterator.Valid() will
// return false). The iterator can be positioned via a call to SeekGE,
// SeekPrefixGE, SeekLT, First or Last. Only indexed batches support iterators.
//
// The returned Iterator observes all of the Batch's existing mutations, but no
// later mutations. Its view can be refreshed via RefreshBatchSnapshot or
// SetOptions().
func (b *Batch) NewIter(o *IterOptions) (*Iterator, error) {
	return b.NewIterWithContext(context.Background(), o), nil
}

// NewIterWithContext is like NewIter, and additionally accepts a context for
// tracing.
func (b *Batch) NewIterWithContext(ctx context.Context, o *IterOptions) *Iterator {
	if b.index == nil {
		return &Iterator{err: ErrNotIndexed}
	}
	return b.db.newIter(ctx, b, snapshotIterOpts{}, o)
}

// newInternalIter creates a new internalIterator that iterates over the
// contents of the batch.
func (b *Batch) newInternalIter(o *IterOptions) *batchIter {
	iter := &batchIter{}
	b.initInternalIter(o, iter)
	return iter
}

func (b *Batch) initInternalIter(o *IterOptions, iter *batchIter) {
	*iter = batchIter{
		cmp:   b.cmp,
		batch: b,
		iter:  b.index.NewIter(o.GetLowerBound(), o.GetUpperBound()),
		// NB: We explicitly do not propagate the batch snapshot to the point
		// key iterator. Filtering point keys within the batch iterator can
		// cause pathological behavior where a batch iterator advances
		// significantly farther than necessary filtering many batch keys that
		// are not visible at the batch sequence number. Instead, the merging
		// iterator enforces bounds.
		//
		// For example, consider an engine that contains the committed keys
		// 'bar' and 'bax', with no keys between them. Consider a batch
		// containing keys 1,000 keys within the range [a,z]. All of the
		// batch keys were added to the batch after the iterator was
		// constructed, so they are not visible to the iterator. A call to
		// SeekGE('bax') would seek the LSM iterators and discover the key
		// 'bax'. It would also seek the batch iterator, landing on the key
		// 'baz' but discover it that it's not visible. The batch iterator would
		// next through the rest of the batch's keys, only to discover there are
		// no visible keys greater than or equal to 'bax'.
		//
		// Filtering these batch points within the merging iterator ensures that
		// the batch iterator never needs to iterate beyond 'baz', because it
		// already found a smaller, visible key 'bax'.
		snapshot: base.InternalKeySeqNumMax,
	}
}

func (b *Batch) newRangeDelIter(o *IterOptions, batchSnapshot uint64) *keyspan.Iter {
	// Construct an iterator even if rangeDelIndex is nil, because it is allowed
	// to refresh later, so we need the container to exist.
	iter := new(keyspan.Iter)
	b.initRangeDelIter(o, iter, batchSnapshot)
	return iter
}

func (b *Batch) initRangeDelIter(_ *IterOptions, iter *keyspan.Iter, batchSnapshot uint64) {
	if b.rangeDelIndex == nil {
		iter.Init(b.cmp, nil)
		return
	}

	// Fragment the range tombstones the first time a range deletion iterator is
	// requested. The cached tombstones are invalidated if another range
	// deletion tombstone is added to the batch. This cache is only guaranteed
	// to be correct if we're opening an iterator to read at a batch sequence
	// number at least as high as tombstonesSeqNum. The cache is guaranteed to
	// include all tombstones up to tombstonesSeqNum, and if any additional
	// tombstones were added after that sequence number the cache would've been
	// cleared.
	nextSeqNum := b.nextSeqNum()
	if b.tombstones != nil && b.tombstonesSeqNum <= batchSnapshot {
		iter.Init(b.cmp, b.tombstones)
		return
	}

	tombstones := make([]keyspan.Span, 0, b.countRangeDels)
	frag := &keyspan.Fragmenter{
		Cmp:    b.cmp,
		Format: b.formatKey,
		Emit: func(s keyspan.Span) {
			tombstones = append(tombstones, s)
		},
	}
	it := &batchIter{
		cmp:      b.cmp,
		batch:    b,
		iter:     b.rangeDelIndex.NewIter(nil, nil),
		snapshot: batchSnapshot,
	}
	fragmentRangeDels(frag, it, int(b.countRangeDels))
	iter.Init(b.cmp, tombstones)

	// If we just read all the tombstones in the batch (eg, batchSnapshot was
	// set to b.nextSeqNum()), then cache the tombstones so that a subsequent
	// call to initRangeDelIter may use them without refragmenting.
	if nextSeqNum == batchSnapshot {
		b.tombstones = tombstones
		b.tombstonesSeqNum = nextSeqNum
	}
}

func fragmentRangeDels(frag *keyspan.Fragmenter, it internalIterator, count int) {
	// The memory management here is a bit subtle. The keys and values returned
	// by the iterator are slices in Batch.data. Thus the fragmented tombstones
	// are slices within Batch.data. If additional entries are added to the
	// Batch, Batch.data may be reallocated. The references in the fragmented
	// tombstones will remain valid, pointing into the old Batch.data. GC for
	// the win.

	// Use a single []keyspan.Key buffer to avoid allocating many
	// individual []keyspan.Key slices with a single element each.
	keyBuf := make([]keyspan.Key, 0, count)
	for key, val := it.First(); key != nil; key, val = it.Next() {
		s := rangedel.Decode(*key, val.InPlaceValue(), keyBuf)
		keyBuf = s.Keys[len(s.Keys):]

		// Set a fixed capacity to avoid accidental overwriting.
		s.Keys = s.Keys[:len(s.Keys):len(s.Keys)]
		frag.Add(s)
	}
	frag.Finish()
}

func (b *Batch) newRangeKeyIter(o *IterOptions, batchSnapshot uint64) *keyspan.Iter {
	// Construct an iterator even if rangeKeyIndex is nil, because it is allowed
	// to refresh later, so we need the container to exist.
	iter := new(keyspan.Iter)
	b.initRangeKeyIter(o, iter, batchSnapshot)
	return iter
}

func (b *Batch) initRangeKeyIter(_ *IterOptions, iter *keyspan.Iter, batchSnapshot uint64) {
	if b.rangeKeyIndex == nil {
		iter.Init(b.cmp, nil)
		return
	}

	// Fragment the range keys the first time a range key iterator is requested.
	// The cached spans are invalidated if another range key is added to the
	// batch. This cache is only guaranteed to be correct if we're opening an
	// iterator to read at a batch sequence number at least as high as
	// rangeKeysSeqNum. The cache is guaranteed to include all range keys up to
	// rangeKeysSeqNum, and if any additional range keys were added after that
	// sequence number the cache would've been cleared.
	nextSeqNum := b.nextSeqNum()
	if b.rangeKeys != nil && b.rangeKeysSeqNum <= batchSnapshot {
		iter.Init(b.cmp, b.rangeKeys)
		return
	}

	rangeKeys := make([]keyspan.Span, 0, b.countRangeKeys)
	frag := &keyspan.Fragmenter{
		Cmp:    b.cmp,
		Format: b.formatKey,
		Emit: func(s keyspan.Span) {
			rangeKeys = append(rangeKeys, s)
		},
	}
	it := &batchIter{
		cmp:      b.cmp,
		batch:    b,
		iter:     b.rangeKeyIndex.NewIter(nil, nil),
		snapshot: batchSnapshot,
	}
	fragmentRangeKeys(frag, it, int(b.countRangeKeys))
	iter.Init(b.cmp, rangeKeys)

	// If we just read all the range keys in the batch (eg, batchSnapshot was
	// set to b.nextSeqNum()), then cache the range keys so that a subsequent
	// call to initRangeKeyIter may use them without refragmenting.
	if nextSeqNum == batchSnapshot {
		b.rangeKeys = rangeKeys
		b.rangeKeysSeqNum = nextSeqNum
	}
}

func fragmentRangeKeys(frag *keyspan.Fragmenter, it internalIterator, count int) error {
	// The memory management here is a bit subtle. The keys and values
	// returned by the iterator are slices in Batch.data. Thus the
	// fragmented key spans are slices within Batch.data. If additional
	// entries are added to the Batch, Batch.data may be reallocated. The
	// references in the fragmented keys will remain valid, pointing into
	// the old Batch.data. GC for the win.

	// Use a single []keyspan.Key buffer to avoid allocating many
	// individual []keyspan.Key slices with a single element each.
	keyBuf := make([]keyspan.Key, 0, count)
	for ik, val := it.First(); ik != nil; ik, val = it.Next() {
		s, err := rangekey.Decode(*ik, val.InPlaceValue(), keyBuf)
		if err != nil {
			return err
		}
		keyBuf = s.Keys[len(s.Keys):]

		// Set a fixed capacity to avoid accidental overwriting.
		s.Keys = s.Keys[:len(s.Keys):len(s.Keys)]
		frag.Add(s)
	}
	frag.Finish()
	return nil
}

// Commit applies the batch to its parent writer.
func (b *Batch) Commit(o *WriteOptions) error {
	return b.db.Apply(b, o)
}

// Close closes the batch without committing it.
func (b *Batch) Close() error {
	b.release()
	return nil
}

// Indexed returns true if the batch is indexed (i.e. supports read
// operations).
func (b *Batch) Indexed() bool {
	return b.index != nil
}

// init ensures that the batch data slice is initialized to meet the
// minimum required size and allocates space for the batch header.
func (b *Batch) init(size int) {
	n := batchInitialSize
	for n < size {
		n *= 2
	}
	if cap(b.data) < n {
		b.data = rawalloc.New(batchHeaderLen, n)
	}
	b.data = b.data[:batchHeaderLen]
	// Zero the sequence number in the header.
	for i := 0; i < len(b.data); i++ {
		b.data[i] = 0
	}
}

// Reset resets the batch for reuse. The underlying byte slice (that is
// returned by Repr()) may not be modified. It is only necessary to call this
// method if a batch is explicitly being reused. Close automatically takes are
// of releasing resources when appropriate for batches that are internally
// being reused.
func (b *Batch) Reset() {
	// Zero out the struct, retaining only the fields necessary for manual
	// reuse.
	b.batchInternal = batchInternal{
		data:           b.data,
		cmp:            b.cmp,
		formatKey:      b.formatKey,
		abbreviatedKey: b.abbreviatedKey,
		index:          b.index,
		db:             b.db,
	}
	b.applied.Store(false)
	if b.data != nil {
		if cap(b.data) > batchMaxRetainedSize {
			// If the capacity of the buffer is larger than our maximum
			// retention size, don't re-use it. Let it be GC-ed instead.
			// This prevents the memory from an unusually large batch from
			// being held on to indefinitely.
			b.data = nil
		} else {
			// Otherwise, reset the buffer for re-use.
			b.data = b.data[:batchHeaderLen]
			// Zero the sequence number in the header.
			for i := 0; i < len(b.data); i++ {
				b.data[i] = 0
			}
		}
	}
	if b.index != nil {
		b.index.Init(&b.data, b.cmp, b.abbreviatedKey)
	}
}

// seqNumData returns the 8 byte little-endian sequence number. Zero means that
// the batch has not yet been applied.
func (b *Batch) seqNumData() []byte {
	return b.data[:8]
}

// countData returns the 4 byte little-endian count data. "\xff\xff\xff\xff"
// means that the batch is invalid.
func (b *Batch) countData() []byte {
	return b.data[8:12]
}

func (b *Batch) grow(n int) {
	newSize := len(b.data) + n
	if uint64(newSize) >= maxBatchSize {
		panic(ErrBatchTooLarge)
	}
	if newSize > cap(b.data) {
		newCap := 2 * cap(b.data)
		for newCap < newSize {
			newCap *= 2
		}
		newData := rawalloc.New(len(b.data), newCap)
		copy(newData, b.data)
		b.data = newData
	}
	b.data = b.data[:newSize]
}

func (b *Batch) setSeqNum(seqNum uint64) {
	binary.LittleEndian.PutUint64(b.seqNumData(), seqNum)
}

// SeqNum returns the batch sequence number which is applied to the first
// record in the batch. The sequence number is incremented for each subsequent
// record. It returns zero if the batch is empty.
func (b *Batch) SeqNum() uint64 {
	if len(b.data) == 0 {
		b.init(batchHeaderLen)
	}
	return binary.LittleEndian.Uint64(b.seqNumData())
}

func (b *Batch) setCount(v uint32) {
	b.count = uint64(v)
}

// Count returns the count of memtable-modifying operations in this batch. All
// operations with the except of LogData increment this count. For IngestSSTs,
// count is only used to indicate the number of SSTs ingested in the record, the
// batch isn't applied to the memtable.
func (b *Batch) Count() uint32 {
	if b.count > math.MaxUint32 {
		panic(ErrInvalidBatch)
	}
	return uint32(b.count)
}

// Reader returns a BatchReader for the current batch contents. If the batch is
// mutated, the new entries will not be visible to the reader.
func (b *Batch) Reader() BatchReader {
	if len(b.data) == 0 {
		b.init(batchHeaderLen)
	}
	return b.data[batchHeaderLen:]
}

func batchDecodeStr(data []byte) (odata []byte, s []byte, ok bool) {
	// TODO(jackson): This will index out of bounds if there's no varint or an
	// invalid varint (eg, a single 0xff byte). Correcting will add a bit of
	// overhead. We could avoid that overhead whenever len(data) >=
	// binary.MaxVarint32?

	var v uint32
	var n int
	ptr := unsafe.Pointer(&data[0])
	if a := *((*uint8)(ptr)); a < 128 {
		v = uint32(a)
		n = 1
	} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
		v = uint32(b)<<7 | uint32(a)
		n = 2
	} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
		v = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		n = 3
	} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
		v = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		n = 4
	} else {
		d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
		v = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		n = 5
	}

	data = data[n:]
	if v > uint32(len(data)) {
		return nil, nil, false
	}
	return data[v:], data[:v], true
}

// SyncWait is to be used in conjunction with DB.ApplyNoSyncWait.
func (b *Batch) SyncWait() error {
	now := time.Now()
	b.fsyncWait.Wait()
	if b.commitErr != nil {
		b.db = nil // prevent batch reuse on error
	}
	waitDuration := time.Since(now)
	b.commitStats.CommitWaitDuration += waitDuration
	b.commitStats.TotalDuration += waitDuration
	return b.commitErr
}

// CommitStats returns stats related to committing the batch. Should be called
// after Batch.Commit, DB.Apply. If DB.ApplyNoSyncWait is used, should be
// called after Batch.SyncWait.
func (b *Batch) CommitStats() BatchCommitStats {
	return b.commitStats
}

// BatchReader iterates over the entries contained in a batch.
type BatchReader []byte

// ReadBatch constructs a BatchReader from a batch representation.  The
// header is not validated. ReadBatch returns a new batch reader and the
// count of entries contained within the batch.
func ReadBatch(repr []byte) (r BatchReader, count uint32) {
	if len(repr) <= batchHeaderLen {
		return nil, count
	}
	count = binary.LittleEndian.Uint32(repr[batchCountOffset:batchHeaderLen])
	return repr[batchHeaderLen:], count
}

// Next returns the next entry in this batch, if there is one. If the reader has
// reached the end of the batch, Next returns ok=false and a nil error. If the
// batch is corrupt and the next entry is illegible, Next returns ok=false and a
// non-nil error.
func (r *BatchReader) Next() (kind InternalKeyKind, ukey []byte, value []byte, ok bool, err error) {
	if len(*r) == 0 {
		return 0, nil, nil, false, nil
	}
	kind = InternalKeyKind((*r)[0])
	if kind > InternalKeyKindMax {
		return 0, nil, nil, false, errors.Wrapf(ErrInvalidBatch, "invalid key kind 0x%x", (*r)[0])
	}
	*r, ukey, ok = batchDecodeStr((*r)[1:])
	if !ok {
		return 0, nil, nil, false, errors.Wrapf(ErrInvalidBatch, "decoding user key")
	}
	switch kind {
	case InternalKeyKindSet, InternalKeyKindMerge, InternalKeyKindRangeDelete,
		InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete,
		InternalKeyKindDeleteSized:
		*r, value, ok = batchDecodeStr(*r)
		if !ok {
			return 0, nil, nil, false, errors.Wrapf(ErrInvalidBatch, "decoding %s value", kind)
		}
	}
	return kind, ukey, value, true, nil
}

// Note: batchIter mirrors the implementation of flushableBatchIter. Keep the
// two in sync.
type batchIter struct {
	cmp   Compare
	batch *Batch
	iter  batchskl.Iterator
	err   error
	// snapshot holds a batch "sequence number" at which the batch is being
	// read. This sequence number has the InternalKeySeqNumBatch bit set, so it
	// encodes an offset within the batch. Only batch entries earlier than the
	// offset are visible during iteration.
	snapshot uint64
}

// batchIter implements the base.InternalIterator interface.
var _ base.InternalIterator = (*batchIter)(nil)

func (i *batchIter) String() string {
	return "batch"
}

func (i *batchIter) SeekGE(key []byte, flags base.SeekGEFlags) (*InternalKey, base.LazyValue) {
	// Ignore TrySeekUsingNext if the view of the batch changed.
	if flags.TrySeekUsingNext() && flags.BatchJustRefreshed() {
		flags = flags.DisableTrySeekUsingNext()
	}

	i.err = nil // clear cached iteration error
	ikey := i.iter.SeekGE(key, flags)
	for ikey != nil && ikey.SeqNum() >= i.snapshot {
		ikey = i.iter.Next()
	}
	if ikey == nil {
		return nil, base.LazyValue{}
	}
	return ikey, base.MakeInPlaceValue(i.value())
}

func (i *batchIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	return i.SeekGE(key, flags)
}

func (i *batchIter) SeekLT(key []byte, flags base.SeekLTFlags) (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	ikey := i.iter.SeekLT(key)
	for ikey != nil && ikey.SeqNum() >= i.snapshot {
		ikey = i.iter.Prev()
	}
	if ikey == nil {
		return nil, base.LazyValue{}
	}
	return ikey, base.MakeInPlaceValue(i.value())
}

func (i *batchIter) First() (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	ikey := i.iter.First()
	for ikey != nil && ikey.SeqNum() >= i.snapshot {
		ikey = i.iter.Next()
	}
	if ikey == nil {
		return nil, base.LazyValue{}
	}
	return ikey, base.MakeInPlaceValue(i.value())
}

func (i *batchIter) Last() (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	ikey := i.iter.Last()
	for ikey != nil && ikey.SeqNum() >= i.snapshot {
		ikey = i.iter.Prev()
	}
	if ikey == nil {
		return nil, base.LazyValue{}
	}
	return ikey, base.MakeInPlaceValue(i.value())
}

func (i *batchIter) Next() (*InternalKey, base.LazyValue) {
	ikey := i.iter.Next()
	for ikey != nil && ikey.SeqNum() >= i.snapshot {
		ikey = i.iter.Next()
	}
	if ikey == nil {
		return nil, base.LazyValue{}
	}
	return ikey, base.MakeInPlaceValue(i.value())
}

func (i *batchIter) NextPrefix(succKey []byte) (*InternalKey, LazyValue) {
	// Because NextPrefix was invoked `succKey` must be ≥ the key at i's current
	// position. Seek the arena iterator using TrySeekUsingNext.
	ikey := i.iter.SeekGE(succKey, base.SeekGEFlagsNone.EnableTrySeekUsingNext())
	for ikey != nil && ikey.SeqNum() >= i.snapshot {
		ikey = i.iter.Next()
	}
	if ikey == nil {
		return nil, base.LazyValue{}
	}
	return ikey, base.MakeInPlaceValue(i.value())
}

func (i *batchIter) Prev() (*InternalKey, base.LazyValue) {
	ikey := i.iter.Prev()
	for ikey != nil && ikey.SeqNum() >= i.snapshot {
		ikey = i.iter.Prev()
	}
	if ikey == nil {
		return nil, base.LazyValue{}
	}
	return ikey, base.MakeInPlaceValue(i.value())
}

func (i *batchIter) value() []byte {
	offset, _, keyEnd := i.iter.KeyInfo()
	data := i.batch.data
	if len(data[offset:]) == 0 {
		i.err = base.CorruptionErrorf("corrupted batch")
		return nil
	}

	switch InternalKeyKind(data[offset]) {
	case InternalKeyKindSet, InternalKeyKindMerge, InternalKeyKindRangeDelete,
		InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete,
		InternalKeyKindDeleteSized:
		_, value, ok := batchDecodeStr(data[keyEnd:])
		if !ok {
			return nil
		}
		return value
	default:
		return nil
	}
}

func (i *batchIter) Error() error {
	return i.err
}

func (i *batchIter) Close() error {
	_ = i.iter.Close()
	return i.err
}

func (i *batchIter) SetBounds(lower, upper []byte) {
	i.iter.SetBounds(lower, upper)
}

type flushableBatchEntry struct {
	// offset is the byte offset of the record within the batch repr.
	offset uint32
	// index is the 0-based ordinal number of the record within the batch. Used
	// to compute the seqnum for the record.
	index uint32
	// key{Start,End} are the start and end byte offsets of the key within the
	// batch repr. Cached to avoid decoding the key length on every
	// comparison. The value is stored starting at keyEnd.
	keyStart uint32
	keyEnd   uint32
}

// flushableBatch wraps an existing batch and provides the interfaces needed
// for making the batch flushable (i.e. able to mimic a memtable).
type flushableBatch struct {
	cmp       Compare
	formatKey base.FormatKey
	data      []byte

	// The base sequence number for the entries in the batch. This is the same
	// value as Batch.seqNum() and is cached here for performance.
	seqNum uint64

	// A slice of offsets and indices for the entries in the batch. Used to
	// implement flushableBatchIter. Unlike the indexing on a normal batch, a
	// flushable batch is indexed such that batch entry i will be given the
	// sequence number flushableBatch.seqNum+i.
	//
	// Sorted in increasing order of key and decreasing order of offset (since
	// higher offsets correspond to higher sequence numbers).
	//
	// Does not include range deletion entries or range key entries.
	offsets []flushableBatchEntry

	// Fragmented range deletion tombstones.
	tombstones []keyspan.Span

	// Fragmented range keys.
	rangeKeys []keyspan.Span
}

var _ flushable = (*flushableBatch)(nil)

// newFlushableBatch creates a new batch that implements the flushable
// interface. This allows the batch to act like a memtable and be placed in the
// queue of flushable memtables. Note that the flushable batch takes ownership
// of the batch data.
func newFlushableBatch(batch *Batch, comparer *Comparer) (*flushableBatch, error) {
	b := &flushableBatch{
		data:      batch.data,
		cmp:       comparer.Compare,
		formatKey: comparer.FormatKey,
		offsets:   make([]flushableBatchEntry, 0, batch.Count()),
	}
	if b.data != nil {
		// Note that this sequence number is not correct when this batch has not
		// been applied since the sequence number has not been assigned yet. The
		// correct sequence number will be set later. But it is correct when the
		// batch is being replayed from the WAL.
		b.seqNum = batch.SeqNum()
	}
	var rangeDelOffsets []flushableBatchEntry
	var rangeKeyOffsets []flushableBatchEntry
	if len(b.data) > batchHeaderLen {
		// Non-empty batch.
		var index uint32
		for iter := BatchReader(b.data[batchHeaderLen:]); len(iter) > 0; index++ {
			offset := uintptr(unsafe.Pointer(&iter[0])) - uintptr(unsafe.Pointer(&b.data[0]))
			kind, key, _, ok, err := iter.Next()
			if !ok {
				if err != nil {
					return nil, err
				}
				break
			}
			entry := flushableBatchEntry{
				offset: uint32(offset),
				index:  uint32(index),
			}
			if keySize := uint32(len(key)); keySize == 0 {
				// Must add 2 to the offset. One byte encodes `kind` and the next
				// byte encodes `0`, which is the length of the key.
				entry.keyStart = uint32(offset) + 2
				entry.keyEnd = entry.keyStart
			} else {
				entry.keyStart = uint32(uintptr(unsafe.Pointer(&key[0])) -
					uintptr(unsafe.Pointer(&b.data[0])))
				entry.keyEnd = entry.keyStart + keySize
			}
			switch kind {
			case InternalKeyKindRangeDelete:
				rangeDelOffsets = append(rangeDelOffsets, entry)
			case InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete:
				rangeKeyOffsets = append(rangeKeyOffsets, entry)
			default:
				b.offsets = append(b.offsets, entry)
			}
		}
	}

	// Sort all of offsets, rangeDelOffsets and rangeKeyOffsets, using *batch's
	// sort.Interface implementation.
	pointOffsets := b.offsets
	sort.Sort(b)
	b.offsets = rangeDelOffsets
	sort.Sort(b)
	b.offsets = rangeKeyOffsets
	sort.Sort(b)
	b.offsets = pointOffsets

	if len(rangeDelOffsets) > 0 {
		frag := &keyspan.Fragmenter{
			Cmp:    b.cmp,
			Format: b.formatKey,
			Emit: func(s keyspan.Span) {
				b.tombstones = append(b.tombstones, s)
			},
		}
		it := &flushableBatchIter{
			batch:   b,
			data:    b.data,
			offsets: rangeDelOffsets,
			cmp:     b.cmp,
			index:   -1,
		}
		fragmentRangeDels(frag, it, len(rangeDelOffsets))
	}
	if len(rangeKeyOffsets) > 0 {
		frag := &keyspan.Fragmenter{
			Cmp:    b.cmp,
			Format: b.formatKey,
			Emit: func(s keyspan.Span) {
				b.rangeKeys = append(b.rangeKeys, s)
			},
		}
		it := &flushableBatchIter{
			batch:   b,
			data:    b.data,
			offsets: rangeKeyOffsets,
			cmp:     b.cmp,
			index:   -1,
		}
		fragmentRangeKeys(frag, it, len(rangeKeyOffsets))
	}
	return b, nil
}

func (b *flushableBatch) setSeqNum(seqNum uint64) {
	if b.seqNum != 0 {
		panic(fmt.Sprintf("pebble: flushableBatch.seqNum already set: %d", b.seqNum))
	}
	b.seqNum = seqNum
	for i := range b.tombstones {
		for j := range b.tombstones[i].Keys {
			b.tombstones[i].Keys[j].Trailer = base.MakeTrailer(
				b.tombstones[i].Keys[j].SeqNum()+seqNum,
				b.tombstones[i].Keys[j].Kind(),
			)
		}
	}
	for i := range b.rangeKeys {
		for j := range b.rangeKeys[i].Keys {
			b.rangeKeys[i].Keys[j].Trailer = base.MakeTrailer(
				b.rangeKeys[i].Keys[j].SeqNum()+seqNum,
				b.rangeKeys[i].Keys[j].Kind(),
			)
		}
	}
}

func (b *flushableBatch) Len() int {
	return len(b.offsets)
}

func (b *flushableBatch) Less(i, j int) bool {
	ei := &b.offsets[i]
	ej := &b.offsets[j]
	ki := b.data[ei.keyStart:ei.keyEnd]
	kj := b.data[ej.keyStart:ej.keyEnd]
	switch c := b.cmp(ki, kj); {
	case c < 0:
		return true
	case c > 0:
		return false
	default:
		return ei.offset > ej.offset
	}
}

func (b *flushableBatch) Swap(i, j int) {
	b.offsets[i], b.offsets[j] = b.offsets[j], b.offsets[i]
}

// newIter is part of the flushable interface.
func (b *flushableBatch) newIter(o *IterOptions) internalIterator {
	return &flushableBatchIter{
		batch:   b,
		data:    b.data,
		offsets: b.offsets,
		cmp:     b.cmp,
		index:   -1,
		lower:   o.GetLowerBound(),
		upper:   o.GetUpperBound(),
	}
}

// newFlushIter is part of the flushable interface.
func (b *flushableBatch) newFlushIter(o *IterOptions, bytesFlushed *uint64) internalIterator {
	return &flushFlushableBatchIter{
		flushableBatchIter: flushableBatchIter{
			batch:   b,
			data:    b.data,
			offsets: b.offsets,
			cmp:     b.cmp,
			index:   -1,
		},
		bytesIterated: bytesFlushed,
	}
}

// newRangeDelIter is part of the flushable interface.
func (b *flushableBatch) newRangeDelIter(o *IterOptions) keyspan.FragmentIterator {
	if len(b.tombstones) == 0 {
		return nil
	}
	return keyspan.NewIter(b.cmp, b.tombstones)
}

// newRangeKeyIter is part of the flushable interface.
func (b *flushableBatch) newRangeKeyIter(o *IterOptions) keyspan.FragmentIterator {
	if len(b.rangeKeys) == 0 {
		return nil
	}
	return keyspan.NewIter(b.cmp, b.rangeKeys)
}

// containsRangeKeys is part of the flushable interface.
func (b *flushableBatch) containsRangeKeys() bool { return len(b.rangeKeys) > 0 }

// inuseBytes is part of the flushable interface.
func (b *flushableBatch) inuseBytes() uint64 {
	return uint64(len(b.data) - batchHeaderLen)
}

// totalBytes is part of the flushable interface.
func (b *flushableBatch) totalBytes() uint64 {
	return uint64(cap(b.data))
}

// readyForFlush is part of the flushable interface.
func (b *flushableBatch) readyForFlush() bool {
	// A flushable batch is always ready for flush; it must be flushed together
	// with the previous memtable.
	return true
}

// Note: flushableBatchIter mirrors the implementation of batchIter. Keep the
// two in sync.
type flushableBatchIter struct {
	// Members to be initialized by creator.
	batch *flushableBatch
	// The bytes backing the batch. Always the same as batch.data?
	data []byte
	// The sorted entries. This is not always equal to batch.offsets.
	offsets []flushableBatchEntry
	cmp     Compare
	// Must be initialized to -1. It is the index into offsets that represents
	// the current iterator position.
	index int

	// For internal use by the implementation.
	key InternalKey
	err error

	// Optionally initialize to bounds of iteration, if any.
	lower []byte
	upper []byte
}

// flushableBatchIter implements the base.InternalIterator interface.
var _ base.InternalIterator = (*flushableBatchIter)(nil)

func (i *flushableBatchIter) String() string {
	return "flushable-batch"
}

// SeekGE implements internalIterator.SeekGE, as documented in the pebble
// package. Ignore flags.TrySeekUsingNext() since we don't expect this
// optimization to provide much benefit here at the moment.
func (i *flushableBatchIter) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	ikey := base.MakeSearchKey(key)
	i.index = sort.Search(len(i.offsets), func(j int) bool {
		return base.InternalCompare(i.cmp, ikey, i.getKey(j)) <= 0
	})
	if i.index >= len(i.offsets) {
		return nil, base.LazyValue{}
	}
	i.key = i.getKey(i.index)
	if i.upper != nil && i.cmp(i.key.UserKey, i.upper) >= 0 {
		i.index = len(i.offsets)
		return nil, base.LazyValue{}
	}
	return &i.key, i.value()
}

// SeekPrefixGE implements internalIterator.SeekPrefixGE, as documented in the
// pebble package.
func (i *flushableBatchIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	return i.SeekGE(key, flags)
}

// SeekLT implements internalIterator.SeekLT, as documented in the pebble
// package.
func (i *flushableBatchIter) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	ikey := base.MakeSearchKey(key)
	i.index = sort.Search(len(i.offsets), func(j int) bool {
		return base.InternalCompare(i.cmp, ikey, i.getKey(j)) <= 0
	})
	i.index--
	if i.index < 0 {
		return nil, base.LazyValue{}
	}
	i.key = i.getKey(i.index)
	if i.lower != nil && i.cmp(i.key.UserKey, i.lower) < 0 {
		i.index = -1
		return nil, base.LazyValue{}
	}
	return &i.key, i.value()
}

// First implements internalIterator.First, as documented in the pebble
// package.
func (i *flushableBatchIter) First() (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	if len(i.offsets) == 0 {
		return nil, base.LazyValue{}
	}
	i.index = 0
	i.key = i.getKey(i.index)
	if i.upper != nil && i.cmp(i.key.UserKey, i.upper) >= 0 {
		i.index = len(i.offsets)
		return nil, base.LazyValue{}
	}
	return &i.key, i.value()
}

// Last implements internalIterator.Last, as documented in the pebble
// package.
func (i *flushableBatchIter) Last() (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	if len(i.offsets) == 0 {
		return nil, base.LazyValue{}
	}
	i.index = len(i.offsets) - 1
	i.key = i.getKey(i.index)
	if i.lower != nil && i.cmp(i.key.UserKey, i.lower) < 0 {
		i.index = -1
		return nil, base.LazyValue{}
	}
	return &i.key, i.value()
}

// Note: flushFlushableBatchIter.Next mirrors the implementation of
// flushableBatchIter.Next due to performance. Keep the two in sync.
func (i *flushableBatchIter) Next() (*InternalKey, base.LazyValue) {
	if i.index == len(i.offsets) {
		return nil, base.LazyValue{}
	}
	i.index++
	if i.index == len(i.offsets) {
		return nil, base.LazyValue{}
	}
	i.key = i.getKey(i.index)
	if i.upper != nil && i.cmp(i.key.UserKey, i.upper) >= 0 {
		i.index = len(i.offsets)
		return nil, base.LazyValue{}
	}
	return &i.key, i.value()
}

func (i *flushableBatchIter) Prev() (*InternalKey, base.LazyValue) {
	if i.index < 0 {
		return nil, base.LazyValue{}
	}
	i.index--
	if i.index < 0 {
		return nil, base.LazyValue{}
	}
	i.key = i.getKey(i.index)
	if i.lower != nil && i.cmp(i.key.UserKey, i.lower) < 0 {
		i.index = -1
		return nil, base.LazyValue{}
	}
	return &i.key, i.value()
}

// Note: flushFlushableBatchIter.NextPrefix mirrors the implementation of
// flushableBatchIter.NextPrefix due to performance. Keep the two in sync.
func (i *flushableBatchIter) NextPrefix(succKey []byte) (*InternalKey, LazyValue) {
	return i.SeekGE(succKey, base.SeekGEFlagsNone.EnableTrySeekUsingNext())
}

func (i *flushableBatchIter) getKey(index int) InternalKey {
	e := &i.offsets[index]
	kind := InternalKeyKind(i.data[e.offset])
	key := i.data[e.keyStart:e.keyEnd]
	return base.MakeInternalKey(key, i.batch.seqNum+uint64(e.index), kind)
}

func (i *flushableBatchIter) value() base.LazyValue {
	p := i.data[i.offsets[i.index].offset:]
	if len(p) == 0 {
		i.err = base.CorruptionErrorf("corrupted batch")
		return base.LazyValue{}
	}
	kind := InternalKeyKind(p[0])
	if kind > InternalKeyKindMax {
		i.err = base.CorruptionErrorf("corrupted batch")
		return base.LazyValue{}
	}
	var value []byte
	var ok bool
	switch kind {
	case InternalKeyKindSet, InternalKeyKindMerge, InternalKeyKindRangeDelete,
		InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete,
		InternalKeyKindDeleteSized:
		keyEnd := i.offsets[i.index].keyEnd
		_, value, ok = batchDecodeStr(i.data[keyEnd:])
		if !ok {
			i.err = base.CorruptionErrorf("corrupted batch")
			return base.LazyValue{}
		}
	}
	return base.MakeInPlaceValue(value)
}

func (i *flushableBatchIter) Valid() bool {
	return i.index >= 0 && i.index < len(i.offsets)
}

func (i *flushableBatchIter) Error() error {
	return i.err
}

func (i *flushableBatchIter) Close() error {
	return i.err
}

func (i *flushableBatchIter) SetBounds(lower, upper []byte) {
	i.lower = lower
	i.upper = upper
}

// flushFlushableBatchIter is similar to flushableBatchIter but it keeps track
// of number of bytes iterated.
type flushFlushableBatchIter struct {
	flushableBatchIter
	bytesIterated *uint64
}

// flushFlushableBatchIter implements the base.InternalIterator interface.
var _ base.InternalIterator = (*flushFlushableBatchIter)(nil)

func (i *flushFlushableBatchIter) String() string {
	return "flushable-batch"
}

func (i *flushFlushableBatchIter) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*InternalKey, base.LazyValue) {
	panic("pebble: SeekGE unimplemented")
}

func (i *flushFlushableBatchIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("pebble: SeekPrefixGE unimplemented")
}

func (i *flushFlushableBatchIter) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*InternalKey, base.LazyValue) {
	panic("pebble: SeekLT unimplemented")
}

func (i *flushFlushableBatchIter) First() (*InternalKey, base.LazyValue) {
	i.err = nil // clear cached iteration error
	key, val := i.flushableBatchIter.First()
	if key == nil {
		return nil, base.LazyValue{}
	}
	entryBytes := i.offsets[i.index].keyEnd - i.offsets[i.index].offset
	*i.bytesIterated += uint64(entryBytes) + i.valueSize()
	return key, val
}

func (i *flushFlushableBatchIter) NextPrefix(succKey []byte) (*InternalKey, base.LazyValue) {
	panic("pebble: Prev unimplemented")
}

// Note: flushFlushableBatchIter.Next mirrors the implementation of
// flushableBatchIter.Next due to performance. Keep the two in sync.
func (i *flushFlushableBatchIter) Next() (*InternalKey, base.LazyValue) {
	if i.index == len(i.offsets) {
		return nil, base.LazyValue{}
	}
	i.index++
	if i.index == len(i.offsets) {
		return nil, base.LazyValue{}
	}
	i.key = i.getKey(i.index)
	entryBytes := i.offsets[i.index].keyEnd - i.offsets[i.index].offset
	*i.bytesIterated += uint64(entryBytes) + i.valueSize()
	return &i.key, i.value()
}

func (i flushFlushableBatchIter) Prev() (*InternalKey, base.LazyValue) {
	panic("pebble: Prev unimplemented")
}

func (i flushFlushableBatchIter) valueSize() uint64 {
	p := i.data[i.offsets[i.index].offset:]
	if len(p) == 0 {
		i.err = base.CorruptionErrorf("corrupted batch")
		return 0
	}
	kind := InternalKeyKind(p[0])
	if kind > InternalKeyKindMax {
		i.err = base.CorruptionErrorf("corrupted batch")
		return 0
	}
	var length uint64
	switch kind {
	case InternalKeyKindSet, InternalKeyKindMerge, InternalKeyKindRangeDelete:
		keyEnd := i.offsets[i.index].keyEnd
		v, n := binary.Uvarint(i.data[keyEnd:])
		if n <= 0 {
			i.err = base.CorruptionErrorf("corrupted batch")
			return 0
		}
		length = v + uint64(n)
	}
	return length
}

// batchSort returns iterators for the sorted contents of the batch. It is
// intended for testing use only. The batch.Sort dance is done to prevent
// exposing this method in the public pebble interface.
func batchSort(
	i interface{},
) (
	points internalIterator,
	rangeDels keyspan.FragmentIterator,
	rangeKeys keyspan.FragmentIterator,
) {
	b := i.(*Batch)
	if b.Indexed() {
		pointIter := b.newInternalIter(nil)
		rangeDelIter := b.newRangeDelIter(nil, math.MaxUint64)
		rangeKeyIter := b.newRangeKeyIter(nil, math.MaxUint64)
		return pointIter, rangeDelIter, rangeKeyIter
	}
	f, err := newFlushableBatch(b, b.db.opts.Comparer)
	if err != nil {
		panic(err)
	}
	return f.newIter(nil), f.newRangeDelIter(nil), f.newRangeKeyIter(nil)
}

func init() {
	private.BatchSort = batchSort
}
