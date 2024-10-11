// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package pebble provides an ordered key/value store.
package pebble // import "github.com/cockroachdb/pebble"

import (
	"context"
	"fmt"
	"io"
	"os"
	"strconv"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/arenaskl"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invalidating"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/internal/manual"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/rangekey"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/pebble/vfs/atomicfs"
	"github.com/cockroachdb/tokenbucket"
	"github.com/prometheus/client_golang/prometheus"
)

const (
	// minTableCacheSize is the minimum size of the table cache, for a single db.
	minTableCacheSize = 64

	// numNonTableCacheFiles is an approximation for the number of files
	// that we don't use for table caches, for a given db.
	numNonTableCacheFiles = 10
)

var (
	// ErrNotFound is returned when a get operation does not find the requested
	// key.
	ErrNotFound = base.ErrNotFound
	// ErrClosed is panicked when an operation is performed on a closed snapshot or
	// DB. Use errors.Is(err, ErrClosed) to check for this error.
	ErrClosed = errors.New("pebble: closed")
	// ErrReadOnly is returned when a write operation is performed on a read-only
	// database.
	ErrReadOnly = errors.New("pebble: read-only")
	// errNoSplit indicates that the user is trying to perform a range key
	// operation but the configured Comparer does not provide a Split
	// implementation.
	errNoSplit = errors.New("pebble: Comparer.Split required for range key operations")
)

// Reader is a readable key/value store.
//
// It is safe to call Get and NewIter from concurrent goroutines.
type Reader interface {
	// Get gets the value for the given key. It returns ErrNotFound if the DB
	// does not contain the key.
	//
	// The caller should not modify the contents of the returned slice, but it is
	// safe to modify the contents of the argument after Get returns. The
	// returned slice will remain valid until the returned Closer is closed. On
	// success, the caller MUST call closer.Close() or a memory leak will occur.
	Get(key []byte) (value []byte, closer io.Closer, err error)

	// NewIter returns an iterator that is unpositioned (Iterator.Valid() will
	// return false). The iterator can be positioned via a call to SeekGE,
	// SeekLT, First or Last.
	NewIter(o *IterOptions) (*Iterator, error)

	// Close closes the Reader. It may or may not close any underlying io.Reader
	// or io.Writer, depending on how the DB was created.
	//
	// It is not safe to close a DB until all outstanding iterators are closed.
	// It is valid to call Close multiple times. Other methods should not be
	// called after the DB has been closed.
	Close() error
}

// Writer is a writable key/value store.
//
// Goroutine safety is dependent on the specific implementation.
type Writer interface {
	// Apply the operations contained in the batch to the DB.
	//
	// It is safe to modify the contents of the arguments after Apply returns.
	Apply(batch *Batch, o *WriteOptions) error

	// Delete deletes the value for the given key. Deletes are blind all will
	// succeed even if the given key does not exist.
	//
	// It is safe to modify the contents of the arguments after Delete returns.
	Delete(key []byte, o *WriteOptions) error

	// DeleteSized behaves identically to Delete, but takes an additional
	// argument indicating the size of the value being deleted. DeleteSized
	// should be preferred when the caller has the expectation that there exists
	// a single internal KV pair for the key (eg, the key has not been
	// overwritten recently), and the caller knows the size of its value.
	//
	// DeleteSized will record the value size within the tombstone and use it to
	// inform compaction-picking heuristics which strive to reduce space
	// amplification in the LSM. This "calling your shot" mechanic allows the
	// storage engine to more accurately estimate and reduce space
	// amplification.
	//
	// It is safe to modify the contents of the arguments after DeleteSized
	// returns.
	DeleteSized(key []byte, valueSize uint32, _ *WriteOptions) error

	// SingleDelete is similar to Delete in that it deletes the value for the given key. Like Delete,
	// it is a blind operation that will succeed even if the given key does not exist.
	//
	// WARNING: Undefined (non-deterministic) behavior will result if a key is overwritten and
	// then deleted using SingleDelete. The record may appear deleted immediately, but be
	// resurrected at a later time after compactions have been performed. Or the record may
	// be deleted permanently. A Delete operation lays down a "tombstone" which shadows all
	// previous versions of a key. The SingleDelete operation is akin to "anti-matter" and will
	// only delete the most recently written version for a key. These different semantics allow
	// the DB to avoid propagating a SingleDelete operation during a compaction as soon as the
	// corresponding Set operation is encountered. These semantics require extreme care to handle
	// properly. Only use if you have a workload where the performance gain is critical and you
	// can guarantee that a record is written once and then deleted once.
	//
	// SingleDelete is internally transformed into a Delete if the most recent record for a key is either
	// a Merge or Delete record.
	//
	// It is safe to modify the contents of the arguments after SingleDelete returns.
	SingleDelete(key []byte, o *WriteOptions) error

	// DeleteRange deletes all of the point keys (and values) in the range
	// [start,end) (inclusive on start, exclusive on end). DeleteRange does NOT
	// delete overlapping range keys (eg, keys set via RangeKeySet).
	//
	// It is safe to modify the contents of the arguments after DeleteRange
	// returns.
	DeleteRange(start, end []byte, o *WriteOptions) error

	// LogData adds the specified to the batch. The data will be written to the
	// WAL, but not added to memtables or sstables. Log data is never indexed,
	// which makes it useful for testing WAL performance.
	//
	// It is safe to modify the contents of the argument after LogData returns.
	LogData(data []byte, opts *WriteOptions) error

	// Merge merges the value for the given key. The details of the merge are
	// dependent upon the configured merge operation.
	//
	// It is safe to modify the contents of the arguments after Merge returns.
	Merge(key, value []byte, o *WriteOptions) error

	// Set sets the value for the given key. It overwrites any previous value
	// for that key; a DB is not a multi-map.
	//
	// It is safe to modify the contents of the arguments after Set returns.
	Set(key, value []byte, o *WriteOptions) error

	// RangeKeySet sets a range key mapping the key range [start, end) at the MVCC
	// timestamp suffix to value. The suffix is optional. If any portion of the key
	// range [start, end) is already set by a range key with the same suffix value,
	// RangeKeySet overrides it.
	//
	// It is safe to modify the contents of the arguments after RangeKeySet returns.
	RangeKeySet(start, end, suffix, value []byte, opts *WriteOptions) error

	// RangeKeyUnset removes a range key mapping the key range [start, end) at the
	// MVCC timestamp suffix. The suffix may be omitted to remove an unsuffixed
	// range key. RangeKeyUnset only removes portions of range keys that fall within
	// the [start, end) key span, and only range keys with suffixes that exactly
	// match the unset suffix.
	//
	// It is safe to modify the contents of the arguments after RangeKeyUnset
	// returns.
	RangeKeyUnset(start, end, suffix []byte, opts *WriteOptions) error

	// RangeKeyDelete deletes all of the range keys in the range [start,end)
	// (inclusive on start, exclusive on end). It does not delete point keys (for
	// that use DeleteRange). RangeKeyDelete removes all range keys within the
	// bounds, including those with or without suffixes.
	//
	// It is safe to modify the contents of the arguments after RangeKeyDelete
	// returns.
	RangeKeyDelete(start, end []byte, opts *WriteOptions) error
}

// CPUWorkHandle represents a handle used by the CPUWorkPermissionGranter API.
type CPUWorkHandle interface {
	// Permitted indicates whether Pebble can use additional CPU resources.
	Permitted() bool
}

// CPUWorkPermissionGranter is used to request permission to opportunistically
// use additional CPUs to speed up internal background work.
type CPUWorkPermissionGranter interface {
	// GetPermission returns a handle regardless of whether permission is granted
	// or not. In the latter case, the handle is only useful for recording
	// the CPU time actually spent on this calling goroutine.
	GetPermission(time.Duration) CPUWorkHandle
	// CPUWorkDone must be called regardless of whether CPUWorkHandle.Permitted
	// returns true or false.
	CPUWorkDone(CPUWorkHandle)
}

// Use a default implementation for the CPU work granter to avoid excessive nil
// checks in the code.
type defaultCPUWorkHandle struct{}

func (d defaultCPUWorkHandle) Permitted() bool {
	return false
}

type defaultCPUWorkGranter struct{}

func (d defaultCPUWorkGranter) GetPermission(_ time.Duration) CPUWorkHandle {
	return defaultCPUWorkHandle{}
}

func (d defaultCPUWorkGranter) CPUWorkDone(_ CPUWorkHandle) {}

// DB provides a concurrent, persistent ordered key/value store.
//
// A DB's basic operations (Get, Set, Delete) should be self-explanatory. Get
// and Delete will return ErrNotFound if the requested key is not in the store.
// Callers are free to ignore this error.
//
// A DB also allows for iterating over the key/value pairs in key order. If d
// is a DB, the code below prints all key/value pairs whose keys are 'greater
// than or equal to' k:
//
//	iter := d.NewIter(readOptions)
//	for iter.SeekGE(k); iter.Valid(); iter.Next() {
//		fmt.Printf("key=%q value=%q\n", iter.Key(), iter.Value())
//	}
//	return iter.Close()
//
// The Options struct holds the optional parameters for the DB, including a
// Comparer to define a 'less than' relationship over keys. It is always valid
// to pass a nil *Options, which means to use the default parameter values. Any
// zero field of a non-nil *Options also means to use the default value for
// that parameter. Thus, the code below uses a custom Comparer, but the default
// values for every other parameter:
//
//	db := pebble.Open(&Options{
//		Comparer: myComparer,
//	})
type DB struct {
	// The count and size of referenced memtables. This includes memtables
	// present in DB.mu.mem.queue, as well as memtables that have been flushed
	// but are still referenced by an inuse readState, as well as up to one
	// memTable waiting to be reused and stored in d.memTableRecycle.
	memTableCount    atomic.Int64
	memTableReserved atomic.Int64 // number of bytes reserved in the cache for memtables
	// memTableRecycle holds a pointer to an obsolete memtable. The next
	// memtable allocation will reuse this memtable if it has not already been
	// recycled.
	memTableRecycle atomic.Pointer[memTable]

	// The size of the current log file (i.e. db.mu.log.queue[len(queue)-1].
	logSize atomic.Uint64

	// The number of bytes available on disk.
	diskAvailBytes atomic.Uint64

	cacheID        uint64
	dirname        string
	walDirname     string
	opts           *Options
	cmp            Compare
	equal          Equal
	merge          Merge
	split          Split
	abbreviatedKey AbbreviatedKey
	// The threshold for determining when a batch is "large" and will skip being
	// inserted into a memtable.
	largeBatchThreshold uint64
	// The current OPTIONS file number.
	optionsFileNum base.DiskFileNum
	// The on-disk size of the current OPTIONS file.
	optionsFileSize uint64

	// objProvider is used to access and manage SSTs.
	objProvider objstorage.Provider

	fileLock *Lock
	dataDir  vfs.File
	walDir   vfs.File

	tableCache           *tableCacheContainer
	newIters             tableNewIters
	tableNewRangeKeyIter keyspan.TableNewSpanIter

	commit *commitPipeline

	// readState provides access to the state needed for reading without needing
	// to acquire DB.mu.
	readState struct {
		sync.RWMutex
		val *readState
	}
	// logRecycler holds a set of log file numbers that are available for
	// reuse. Writing to a recycled log file is faster than to a new log file on
	// some common filesystems (xfs, and ext3/4) due to avoiding metadata
	// updates.
	logRecycler logRecycler

	closed   *atomic.Value
	closedCh chan struct{}

	cleanupManager *cleanupManager

	// During an iterator close, we may asynchronously schedule read compactions.
	// We want to wait for those goroutines to finish, before closing the DB.
	// compactionShedulers.Wait() should not be called while the DB.mu is held.
	compactionSchedulers sync.WaitGroup

	// The main mutex protecting internal DB state. This mutex encompasses many
	// fields because those fields need to be accessed and updated atomically. In
	// particular, the current version, log.*, mem.*, and snapshot list need to
	// be accessed and updated atomically during compaction.
	//
	// Care is taken to avoid holding DB.mu during IO operations. Accomplishing
	// this sometimes requires releasing DB.mu in a method that was called with
	// it held. See versionSet.logAndApply() and DB.makeRoomForWrite() for
	// examples. This is a common pattern, so be careful about expectations that
	// DB.mu will be held continuously across a set of calls.
	mu struct {
		sync.Mutex

		formatVers struct {
			// vers is the database's current format major version.
			// Backwards-incompatible features are gated behind new
			// format major versions and not enabled until a database's
			// version is ratcheted upwards.
			//
			// Although this is under the `mu` prefix, readers may read vers
			// atomically without holding d.mu. Writers must only write to this
			// value through finalizeFormatVersUpgrade which requires d.mu is
			// held.
			vers atomic.Uint64
			// marker is the atomic marker for the format major version.
			// When a database's version is ratcheted upwards, the
			// marker is moved in order to atomically record the new
			// version.
			marker *atomicfs.Marker
			// ratcheting when set to true indicates that the database is
			// currently in the process of ratcheting the format major version
			// to vers + 1. As a part of ratcheting the format major version,
			// migrations may drop and re-acquire the mutex.
			ratcheting bool
		}

		// The ID of the next job. Job IDs are passed to event listener
		// notifications and act as a mechanism for tying together the events and
		// log messages for a single job such as a flush, compaction, or file
		// ingestion. Job IDs are not serialized to disk or used for correctness.
		nextJobID int

		// The collection of immutable versions and state about the log and visible
		// sequence numbers. Use the pointer here to ensure the atomic fields in
		// version set are aligned properly.
		versions *versionSet

		log struct {
			// The queue of logs, containing both flushed and unflushed logs. The
			// flushed logs will be a prefix, the unflushed logs a suffix. The
			// delimeter between flushed and unflushed logs is
			// versionSet.minUnflushedLogNum.
			queue []fileInfo
			// The number of input bytes to the log. This is the raw size of the
			// batches written to the WAL, without the overhead of the record
			// envelopes. Requires DB.mu to be held when read or written.
			bytesIn uint64
			// The LogWriter is protected by commitPipeline.mu. This allows log
			// writes to be performed without holding DB.mu, but requires both
			// commitPipeline.mu and DB.mu to be held when rotating the WAL/memtable
			// (i.e. makeRoomForWrite).
			*record.LogWriter
			// Can be nil.
			metrics struct {
				fsyncLatency prometheus.Histogram
				record.LogWriterMetrics
			}
			registerLogWriterForTesting func(w *record.LogWriter)
		}

		mem struct {
			// The current mutable memTable.
			mutable *memTable
			// Queue of flushables (the mutable memtable is at end). Elements are
			// added to the end of the slice and removed from the beginning. Once an
			// index is set it is never modified making a fixed slice immutable and
			// safe for concurrent reads.
			queue flushableList
			// nextSize is the size of the next memtable. The memtable size starts at
			// min(256KB,Options.MemTableSize) and doubles each time a new memtable
			// is allocated up to Options.MemTableSize. This reduces the memory
			// footprint of memtables when lots of DB instances are used concurrently
			// in test environments.
			nextSize uint64
		}

		compact struct {
			// Condition variable used to signal when a flush or compaction has
			// completed. Used by the write-stall mechanism to wait for the stall
			// condition to clear. See DB.makeRoomForWrite().
			cond sync.Cond
			// True when a flush is in progress.
			flushing bool
			// The number of ongoing compactions.
			compactingCount int
			// The list of deletion hints, suggesting ranges for delete-only
			// compactions.
			deletionHints []deleteCompactionHint
			// The list of manual compactions. The next manual compaction to perform
			// is at the start of the list. New entries are added to the end.
			manual []*manualCompaction
			// inProgress is the set of in-progress flushes and compactions.
			// It's used in the calculation of some metrics and to initialize L0
			// sublevels' state. Some of the compactions contained within this
			// map may have already committed an edit to the version but are
			// lingering performing cleanup, like deleting obsolete files.
			inProgress map[*compaction]struct{}

			// rescheduleReadCompaction indicates to an iterator that a read compaction
			// should be scheduled.
			rescheduleReadCompaction bool

			// readCompactions is a readCompactionQueue which keeps track of the
			// compactions which we might have to perform.
			readCompactions readCompactionQueue

			// The cumulative duration of all completed compactions since Open.
			// Does not include flushes.
			duration time.Duration
			// Flush throughput metric.
			flushWriteThroughput ThroughputMetric
			// The idle start time for the flush "loop", i.e., when the flushing
			// bool above transitions to false.
			noOngoingFlushStartTime time.Time
		}

		// Non-zero when file cleaning is disabled. The disabled count acts as a
		// reference count to prohibit file cleaning. See
		// DB.{disable,Enable}FileDeletions().
		disableFileDeletions int

		snapshots struct {
			// The list of active snapshots.
			snapshotList

			// The cumulative count and size of snapshot-pinned keys written to
			// sstables.
			cumulativePinnedCount uint64
			cumulativePinnedSize  uint64
		}

		tableStats struct {
			// Condition variable used to signal the completion of a
			// job to collect table stats.
			cond sync.Cond
			// True when a stat collection operation is in progress.
			loading bool
			// True if stat collection has loaded statistics for all tables
			// other than those listed explicitly in pending. This flag starts
			// as false when a database is opened and flips to true once stat
			// collection has caught up.
			loadedInitial bool
			// A slice of files for which stats have not been computed.
			// Compactions, ingests, flushes append files to be processed. An
			// active stat collection goroutine clears the list and processes
			// them.
			pending []manifest.NewFileEntry
		}

		tableValidation struct {
			// cond is a condition variable used to signal the completion of a
			// job to validate one or more sstables.
			cond sync.Cond
			// pending is a slice of metadata for sstables waiting to be
			// validated. Only physical sstables should be added to the pending
			// queue.
			pending []newFileEntry
			// validating is set to true when validation is running.
			validating bool
		}
	}

	// Normally equal to time.Now() but may be overridden in tests.
	timeNow func() time.Time
	// the time at database Open; may be used to compute metrics like effective
	// compaction concurrency
	openedAt time.Time
}

var _ Reader = (*DB)(nil)
var _ Writer = (*DB)(nil)

// TestOnlyWaitForCleaning MUST only be used in tests.
func (d *DB) TestOnlyWaitForCleaning() {
	d.cleanupManager.Wait()
}

// Get gets the value for the given key. It returns ErrNotFound if the DB does
// not contain the key.
//
// The caller should not modify the contents of the returned slice, but it is
// safe to modify the contents of the argument after Get returns. The returned
// slice will remain valid until the returned Closer is closed. On success, the
// caller MUST call closer.Close() or a memory leak will occur.
func (d *DB) Get(key []byte) ([]byte, io.Closer, error) {
	return d.getInternal(key, nil /* batch */, nil /* snapshot */)
}

type getIterAlloc struct {
	dbi    Iterator
	keyBuf []byte
	get    getIter
}

var getIterAllocPool = sync.Pool{
	New: func() interface{} {
		return &getIterAlloc{}
	},
}

func (d *DB) getInternal(key []byte, b *Batch, s *Snapshot) ([]byte, io.Closer, error) {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}

	// Grab and reference the current readState. This prevents the underlying
	// files in the associated version from being deleted if there is a current
	// compaction. The readState is unref'd by Iterator.Close().
	readState := d.loadReadState()

	// Determine the seqnum to read at after grabbing the read state (current and
	// memtables) above.
	var seqNum uint64
	if s != nil {
		seqNum = s.seqNum
	} else {
		seqNum = d.mu.versions.visibleSeqNum.Load()
	}

	buf := getIterAllocPool.Get().(*getIterAlloc)

	get := &buf.get
	*get = getIter{
		logger:   d.opts.Logger,
		comparer: d.opts.Comparer,
		newIters: d.newIters,
		snapshot: seqNum,
		key:      key,
		batch:    b,
		mem:      readState.memtables,
		l0:       readState.current.L0SublevelFiles,
		version:  readState.current,
	}

	// Strip off memtables which cannot possibly contain the seqNum being read
	// at.
	for len(get.mem) > 0 {
		n := len(get.mem)
		if logSeqNum := get.mem[n-1].logSeqNum; logSeqNum < seqNum {
			break
		}
		get.mem = get.mem[:n-1]
	}

	i := &buf.dbi
	pointIter := get
	*i = Iterator{
		ctx:          context.Background(),
		getIterAlloc: buf,
		iter:         pointIter,
		pointIter:    pointIter,
		merge:        d.merge,
		comparer:     *d.opts.Comparer,
		readState:    readState,
		keyBuf:       buf.keyBuf,
	}

	if !i.First() {
		err := i.Close()
		if err != nil {
			return nil, nil, err
		}
		return nil, nil, ErrNotFound
	}
	return i.Value(), i, nil
}

// Set sets the value for the given key. It overwrites any previous value
// for that key; a DB is not a multi-map.
//
// It is safe to modify the contents of the arguments after Set returns.
func (d *DB) Set(key, value []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.Set(key, value, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// Delete deletes the value for the given key. Deletes are blind all will
// succeed even if the given key does not exist.
//
// It is safe to modify the contents of the arguments after Delete returns.
func (d *DB) Delete(key []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.Delete(key, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
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
func (d *DB) DeleteSized(key []byte, valueSize uint32, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.DeleteSized(key, valueSize, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// SingleDelete adds an action to the batch that single deletes the entry for key.
// See Writer.SingleDelete for more details on the semantics of SingleDelete.
//
// It is safe to modify the contents of the arguments after SingleDelete returns.
func (d *DB) SingleDelete(key []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.SingleDelete(key, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// DeleteRange deletes all of the keys (and values) in the range [start,end)
// (inclusive on start, exclusive on end).
//
// It is safe to modify the contents of the arguments after DeleteRange
// returns.
func (d *DB) DeleteRange(start, end []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.DeleteRange(start, end, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// Merge adds an action to the DB that merges the value at key with the new
// value. The details of the merge are dependent upon the configured merge
// operator.
//
// It is safe to modify the contents of the arguments after Merge returns.
func (d *DB) Merge(key, value []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.Merge(key, value, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// LogData adds the specified to the batch. The data will be written to the
// WAL, but not added to memtables or sstables. Log data is never indexed,
// which makes it useful for testing WAL performance.
//
// It is safe to modify the contents of the argument after LogData returns.
func (d *DB) LogData(data []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.LogData(data, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// RangeKeySet sets a range key mapping the key range [start, end) at the MVCC
// timestamp suffix to value. The suffix is optional. If any portion of the key
// range [start, end) is already set by a range key with the same suffix value,
// RangeKeySet overrides it.
//
// It is safe to modify the contents of the arguments after RangeKeySet returns.
func (d *DB) RangeKeySet(start, end, suffix, value []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.RangeKeySet(start, end, suffix, value, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// RangeKeyUnset removes a range key mapping the key range [start, end) at the
// MVCC timestamp suffix. The suffix may be omitted to remove an unsuffixed
// range key. RangeKeyUnset only removes portions of range keys that fall within
// the [start, end) key span, and only range keys with suffixes that exactly
// match the unset suffix.
//
// It is safe to modify the contents of the arguments after RangeKeyUnset
// returns.
func (d *DB) RangeKeyUnset(start, end, suffix []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.RangeKeyUnset(start, end, suffix, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// RangeKeyDelete deletes all of the range keys in the range [start,end)
// (inclusive on start, exclusive on end). It does not delete point keys (for
// that use DeleteRange). RangeKeyDelete removes all range keys within the
// bounds, including those with or without suffixes.
//
// It is safe to modify the contents of the arguments after RangeKeyDelete
// returns.
func (d *DB) RangeKeyDelete(start, end []byte, opts *WriteOptions) error {
	b := newBatch(d)
	_ = b.RangeKeyDelete(start, end, opts)
	if err := d.Apply(b, opts); err != nil {
		return err
	}
	// Only release the batch on success.
	b.release()
	return nil
}

// Apply the operations contained in the batch to the DB. If the batch is large
// the contents of the batch may be retained by the database. If that occurs
// the batch contents will be cleared preventing the caller from attempting to
// reuse them.
//
// It is safe to modify the contents of the arguments after Apply returns.
func (d *DB) Apply(batch *Batch, opts *WriteOptions) error {
	return d.applyInternal(batch, opts, false)
}

// ApplyNoSyncWait must only be used when opts.Sync is true and the caller
// does not want to wait for the WAL fsync to happen. The method will return
// once the mutation is applied to the memtable and is visible (note that a
// mutation is visible before the WAL sync even in the wait case, so we have
// not weakened the durability semantics). The caller must call Batch.SyncWait
// to wait for the WAL fsync. The caller must not Close the batch without
// first calling Batch.SyncWait.
//
// RECOMMENDATION: Prefer using Apply unless you really understand why you
// need ApplyNoSyncWait.
// EXPERIMENTAL: API/feature subject to change. Do not yet use outside
// CockroachDB.
func (d *DB) ApplyNoSyncWait(batch *Batch, opts *WriteOptions) error {
	if !opts.Sync {
		return errors.Errorf("cannot request asynchonous apply when WriteOptions.Sync is false")
	}
	return d.applyInternal(batch, opts, true)
}

// REQUIRES: noSyncWait => opts.Sync
func (d *DB) applyInternal(batch *Batch, opts *WriteOptions, noSyncWait bool) error {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	if batch.committing {
		panic("pebble: batch already committing")
	}
	if batch.applied.Load() {
		panic("pebble: batch already applied")
	}
	if d.opts.ReadOnly {
		return ErrReadOnly
	}
	if batch.db != nil && batch.db != d {
		panic(fmt.Sprintf("pebble: batch db mismatch: %p != %p", batch.db, d))
	}

	sync := opts.GetSync()
	if sync && d.opts.DisableWAL {
		return errors.New("pebble: WAL disabled")
	}

	if batch.minimumFormatMajorVersion != FormatMostCompatible {
		if fmv := d.FormatMajorVersion(); fmv < batch.minimumFormatMajorVersion {
			panic(fmt.Sprintf(
				"pebble: batch requires at least format major version %d (current: %d)",
				batch.minimumFormatMajorVersion, fmv,
			))
		}
	}

	if batch.countRangeKeys > 0 {
		if d.split == nil {
			return errNoSplit
		}
		// TODO(jackson): Assert that all range key operands are suffixless.
	}
	batch.committing = true

	if batch.db == nil {
		if err := batch.refreshMemTableSize(); err != nil {
			return err
		}
	}
	if batch.memTableSize >= d.largeBatchThreshold {
		var err error
		batch.flushable, err = newFlushableBatch(batch, d.opts.Comparer)
		if err != nil {
			return err
		}
	}
	if err := d.commit.Commit(batch, sync, noSyncWait); err != nil {
		// There isn't much we can do on an error here. The commit pipeline will be
		// horked at this point.
		d.opts.Logger.Fatalf("pebble: fatal commit error: %v", err)
	}
	// If this is a large batch, we need to clear the batch contents as the
	// flushable batch may still be present in the flushables queue.
	//
	// TODO(peter): Currently large batches are written to the WAL. We could
	// skip the WAL write and instead wait for the large batch to be flushed to
	// an sstable. For a 100 MB batch, this might actually be faster. For a 1
	// GB batch this is almost certainly faster.
	if batch.flushable != nil {
		batch.data = nil
	}
	return nil
}

func (d *DB) commitApply(b *Batch, mem *memTable) error {
	if b.flushable != nil {
		// This is a large batch which was already added to the immutable queue.
		return nil
	}
	err := mem.apply(b, b.SeqNum())
	if err != nil {
		return err
	}

	// If the batch contains range tombstones and the database is configured
	// to flush range deletions, schedule a delayed flush so that disk space
	// may be reclaimed without additional writes or an explicit flush.
	if b.countRangeDels > 0 && d.opts.FlushDelayDeleteRange > 0 {
		d.mu.Lock()
		d.maybeScheduleDelayedFlush(mem, d.opts.FlushDelayDeleteRange)
		d.mu.Unlock()
	}

	// If the batch contains range keys and the database is configured to flush
	// range keys, schedule a delayed flush so that the range keys are cleared
	// from the memtable.
	if b.countRangeKeys > 0 && d.opts.FlushDelayRangeKey > 0 {
		d.mu.Lock()
		d.maybeScheduleDelayedFlush(mem, d.opts.FlushDelayRangeKey)
		d.mu.Unlock()
	}

	if mem.writerUnref() {
		d.mu.Lock()
		d.maybeScheduleFlush()
		d.mu.Unlock()
	}
	return nil
}

func (d *DB) commitWrite(b *Batch, syncWG *sync.WaitGroup, syncErr *error) (*memTable, error) {
	var size int64
	repr := b.Repr()

	if b.flushable != nil {
		// We have a large batch. Such batches are special in that they don't get
		// added to the memtable, and are instead inserted into the queue of
		// memtables. The call to makeRoomForWrite with this batch will force the
		// current memtable to be flushed. We want the large batch to be part of
		// the same log, so we add it to the WAL here, rather than after the call
		// to makeRoomForWrite().
		//
		// Set the sequence number since it was not set to the correct value earlier
		// (see comment in newFlushableBatch()).
		b.flushable.setSeqNum(b.SeqNum())
		if !d.opts.DisableWAL {
			var err error
			size, err = d.mu.log.SyncRecord(repr, syncWG, syncErr)
			if err != nil {
				panic(err)
			}
		}
	}

	d.mu.Lock()

	var err error
	if !b.ingestedSSTBatch {
		// Batches which contain keys of kind InternalKeyKindIngestSST will
		// never be applied to the memtable, so we don't need to make room for
		// write. For the other cases, switch out the memtable if there was not
		// enough room to store the batch.
		err = d.makeRoomForWrite(b)
	}

	if err == nil && !d.opts.DisableWAL {
		d.mu.log.bytesIn += uint64(len(repr))
	}

	// Grab a reference to the memtable while holding DB.mu. Note that for
	// non-flushable batches (b.flushable == nil) makeRoomForWrite() added a
	// reference to the memtable which will prevent it from being flushed until
	// we unreference it. This reference is dropped in DB.commitApply().
	mem := d.mu.mem.mutable

	d.mu.Unlock()
	if err != nil {
		return nil, err
	}

	if d.opts.DisableWAL {
		return mem, nil
	}

	if b.flushable == nil {
		size, err = d.mu.log.SyncRecord(repr, syncWG, syncErr)
		if err != nil {
			panic(err)
		}
	}

	d.logSize.Store(uint64(size))
	return mem, err
}

type iterAlloc struct {
	dbi                 Iterator
	keyBuf              []byte
	boundsBuf           [2][]byte
	prefixOrFullSeekKey []byte
	merging             mergingIter
	mlevels             [3 + numLevels]mergingIterLevel
	levels              [3 + numLevels]levelIter
	levelsPositioned    [3 + numLevels]bool
}

var iterAllocPool = sync.Pool{
	New: func() interface{} {
		return &iterAlloc{}
	},
}

// snapshotIterOpts denotes snapshot-related iterator options when calling
// newIter. These are the possible cases for a snapshotIterOpts:
//   - No snapshot: All fields are zero values.
//   - Classic snapshot: Only `seqNum` is set. The latest readState will be used
//     and the specified seqNum will be used as the snapshot seqNum.
//   - EventuallyFileOnlySnapshot (EFOS) behaving as a classic snapshot. Only
//     the `seqNum` is set. The latest readState will be used
//     and the specified seqNum will be used as the snapshot seqNum.
//   - EFOS in file-only state: Only `seqNum` and `vers` are set. All the
//     relevant SSTs are referenced by the *version.
type snapshotIterOpts struct {
	seqNum uint64
	vers   *version
}

// newIter constructs a new iterator, merging in batch iterators as an extra
// level.
func (d *DB) newIter(
	ctx context.Context, batch *Batch, sOpts snapshotIterOpts, o *IterOptions,
) *Iterator {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	seqNum := sOpts.seqNum
	if o.rangeKeys() {
		if d.FormatMajorVersion() < FormatRangeKeys {
			panic(fmt.Sprintf(
				"pebble: range keys require at least format major version %d (current: %d)",
				FormatRangeKeys, d.FormatMajorVersion(),
			))
		}
	}
	if o != nil && o.RangeKeyMasking.Suffix != nil && o.KeyTypes != IterKeyTypePointsAndRanges {
		panic("pebble: range key masking requires IterKeyTypePointsAndRanges")
	}
	if (batch != nil || seqNum != 0) && (o != nil && o.OnlyReadGuaranteedDurable) {
		// We could add support for OnlyReadGuaranteedDurable on snapshots if
		// there was a need: this would require checking that the sequence number
		// of the snapshot has been flushed, by comparing with
		// DB.mem.queue[0].logSeqNum.
		panic("OnlyReadGuaranteedDurable is not supported for batches or snapshots")
	}
	// Grab and reference the current readState. This prevents the underlying
	// files in the associated version from being deleted if there is a current
	// compaction. The readState is unref'd by Iterator.Close().
	var readState *readState
	if sOpts.vers == nil {
		// NB: loadReadState() calls readState.ref().
		readState = d.loadReadState()
	} else {
		// s.vers != nil
		sOpts.vers.Ref()
	}

	// Determine the seqnum to read at after grabbing the read state (current and
	// memtables) above.
	if seqNum == 0 {
		seqNum = d.mu.versions.visibleSeqNum.Load()
	}

	// Bundle various structures under a single umbrella in order to allocate
	// them together.
	buf := iterAllocPool.Get().(*iterAlloc)
	dbi := &buf.dbi
	*dbi = Iterator{
		ctx:                 ctx,
		alloc:               buf,
		merge:               d.merge,
		comparer:            *d.opts.Comparer,
		readState:           readState,
		version:             sOpts.vers,
		keyBuf:              buf.keyBuf,
		prefixOrFullSeekKey: buf.prefixOrFullSeekKey,
		boundsBuf:           buf.boundsBuf,
		batch:               batch,
		newIters:            d.newIters,
		newIterRangeKey:     d.tableNewRangeKeyIter,
		seqNum:              seqNum,
	}
	if o != nil {
		dbi.opts = *o
		dbi.processBounds(o.LowerBound, o.UpperBound)
	}
	dbi.opts.logger = d.opts.Logger
	if d.opts.private.disableLazyCombinedIteration {
		dbi.opts.disableLazyCombinedIteration = true
	}
	if batch != nil {
		dbi.batchSeqNum = dbi.batch.nextSeqNum()
	}
	return finishInitializingIter(ctx, buf)
}

// finishInitializingIter is a helper for doing the non-trivial initialization
// of an Iterator. It's invoked to perform the initial initialization of an
// Iterator during NewIter or Clone, and to perform reinitialization due to a
// change in IterOptions by a call to Iterator.SetOptions.
func finishInitializingIter(ctx context.Context, buf *iterAlloc) *Iterator {
	// Short-hand.
	dbi := &buf.dbi
	var memtables flushableList
	if dbi.readState != nil {
		memtables = dbi.readState.memtables
	}
	if dbi.opts.OnlyReadGuaranteedDurable {
		memtables = nil
	} else {
		// We only need to read from memtables which contain sequence numbers older
		// than seqNum. Trim off newer memtables.
		for i := len(memtables) - 1; i >= 0; i-- {
			if logSeqNum := memtables[i].logSeqNum; logSeqNum < dbi.seqNum {
				break
			}
			memtables = memtables[:i]
		}
	}

	if dbi.opts.pointKeys() {
		// Construct the point iterator, initializing dbi.pointIter to point to
		// dbi.merging. If this is called during a SetOptions call and this
		// Iterator has already initialized dbi.merging, constructPointIter is a
		// noop and an initialized pointIter already exists in dbi.pointIter.
		dbi.constructPointIter(ctx, memtables, buf)
		dbi.iter = dbi.pointIter
	} else {
		dbi.iter = emptyIter
	}

	if dbi.opts.rangeKeys() {
		dbi.rangeKeyMasking.init(dbi, dbi.comparer.Compare, dbi.comparer.Split)

		// When iterating over both point and range keys, don't create the
		// range-key iterator stack immediately if we can avoid it. This
		// optimization takes advantage of the expected sparseness of range
		// keys, and configures the point-key iterator to dynamically switch to
		// combined iteration when it observes a file containing range keys.
		//
		// Lazy combined iteration is not possible if a batch or a memtable
		// contains any range keys.
		useLazyCombinedIteration := dbi.rangeKey == nil &&
			dbi.opts.KeyTypes == IterKeyTypePointsAndRanges &&
			(dbi.batch == nil || dbi.batch.countRangeKeys == 0) &&
			!dbi.opts.disableLazyCombinedIteration
		if useLazyCombinedIteration {
			// The user requested combined iteration, and there's no indexed
			// batch currently containing range keys that would prevent lazy
			// combined iteration. Check the memtables to see if they contain
			// any range keys.
			for i := range memtables {
				if memtables[i].containsRangeKeys() {
					useLazyCombinedIteration = false
					break
				}
			}
		}

		if useLazyCombinedIteration {
			dbi.lazyCombinedIter = lazyCombinedIter{
				parent:    dbi,
				pointIter: dbi.pointIter,
				combinedIterState: combinedIterState{
					initialized: false,
				},
			}
			dbi.iter = &dbi.lazyCombinedIter
			dbi.iter = invalidating.MaybeWrapIfInvariants(dbi.iter)
		} else {
			dbi.lazyCombinedIter.combinedIterState = combinedIterState{
				initialized: true,
			}
			if dbi.rangeKey == nil {
				dbi.rangeKey = iterRangeKeyStateAllocPool.Get().(*iteratorRangeKeyState)
				dbi.rangeKey.init(dbi.comparer.Compare, dbi.comparer.Split, &dbi.opts)
				dbi.constructRangeKeyIter()
			} else {
				dbi.rangeKey.iterConfig.SetBounds(dbi.opts.LowerBound, dbi.opts.UpperBound)
			}

			// Wrap the point iterator (currently dbi.iter) with an interleaving
			// iterator that interleaves range keys pulled from
			// dbi.rangeKey.rangeKeyIter.
			//
			// NB: The interleaving iterator is always reinitialized, even if
			// dbi already had an initialized range key iterator, in case the point
			// iterator changed or the range key masking suffix changed.
			dbi.rangeKey.iiter.Init(&dbi.comparer, dbi.iter, dbi.rangeKey.rangeKeyIter,
				keyspan.InterleavingIterOpts{
					Mask:       &dbi.rangeKeyMasking,
					LowerBound: dbi.opts.LowerBound,
					UpperBound: dbi.opts.UpperBound,
				})
			dbi.iter = &dbi.rangeKey.iiter
		}
	} else {
		// !dbi.opts.rangeKeys()
		//
		// Reset the combined iterator state. The initialized=true ensures the
		// iterator doesn't unnecessarily try to switch to combined iteration.
		dbi.lazyCombinedIter.combinedIterState = combinedIterState{initialized: true}
	}
	return dbi
}

// ScanInternal scans all internal keys within the specified bounds, truncating
// any rangedels and rangekeys to those bounds if they span past them. For use
// when an external user needs to be aware of all internal keys that make up a
// key range.
//
// Keys deleted by range deletions must not be returned or exposed by this
// method, while the range deletion deleting that key must be exposed using
// visitRangeDel. Keys that would be masked by range key masking (if an
// appropriate prefix were set) should be exposed, alongside the range key
// that would have masked it. This method also collapses all point keys into
// one InternalKey; so only one internal key at most per user key is returned
// to visitPointKey.
//
// If visitSharedFile is not nil, ScanInternal iterates in skip-shared iteration
// mode. In this iteration mode, sstables in levels L5 and L6 are skipped, and
// their metadatas truncated to [lower, upper) and passed into visitSharedFile.
// ErrInvalidSkipSharedIteration is returned if visitSharedFile is not nil and an
// sstable in L5 or L6 is found that is not in shared storage according to
// provider.IsShared, or an sstable in those levels contains a newer key than the
// snapshot sequence number (only applicable for snapshot.ScanInternal). Examples
// of when this could happen could be if Pebble started writing sstables before a
// creator ID was set (as creator IDs are necessary to enable shared storage)
// resulting in some lower level SSTs being on non-shared storage. Skip-shared
// iteration is invalid in those cases.
func (d *DB) ScanInternal(
	ctx context.Context,
	lower, upper []byte,
	visitPointKey func(key *InternalKey, value LazyValue, iterInfo IteratorLevel) error,
	visitRangeDel func(start, end []byte, seqNum uint64) error,
	visitRangeKey func(start, end []byte, keys []rangekey.Key) error,
	visitSharedFile func(sst *SharedSSTMeta) error,
) error {
	scanInternalOpts := &scanInternalOptions{
		visitPointKey:    visitPointKey,
		visitRangeDel:    visitRangeDel,
		visitRangeKey:    visitRangeKey,
		visitSharedFile:  visitSharedFile,
		skipSharedLevels: visitSharedFile != nil,
		IterOptions: IterOptions{
			KeyTypes:   IterKeyTypePointsAndRanges,
			LowerBound: lower,
			UpperBound: upper,
		},
	}
	iter := d.newInternalIter(snapshotIterOpts{} /* snapshot */, scanInternalOpts)
	defer iter.close()
	return scanInternalImpl(ctx, lower, upper, iter, scanInternalOpts)
}

// newInternalIter constructs and returns a new scanInternalIterator on this db.
// If o.skipSharedLevels is true, levels below sharedLevelsStart are *not* added
// to the internal iterator.
//
// TODO(bilal): This method has a lot of similarities with db.newIter as well as
// finishInitializingIter. Both pairs of methods should be refactored to reduce
// this duplication.
func (d *DB) newInternalIter(sOpts snapshotIterOpts, o *scanInternalOptions) *scanInternalIterator {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	// Grab and reference the current readState. This prevents the underlying
	// files in the associated version from being deleted if there is a current
	// compaction. The readState is unref'd by Iterator.Close().
	var readState *readState
	if sOpts.vers == nil {
		readState = d.loadReadState()
	}
	if sOpts.vers != nil {
		sOpts.vers.Ref()
	}

	// Determine the seqnum to read at after grabbing the read state (current and
	// memtables) above.
	seqNum := sOpts.seqNum
	if seqNum == 0 {
		seqNum = d.mu.versions.visibleSeqNum.Load()
	}

	// Bundle various structures under a single umbrella in order to allocate
	// them together.
	buf := iterAllocPool.Get().(*iterAlloc)
	dbi := &scanInternalIterator{
		db:              d,
		comparer:        d.opts.Comparer,
		merge:           d.opts.Merger.Merge,
		readState:       readState,
		version:         sOpts.vers,
		alloc:           buf,
		newIters:        d.newIters,
		newIterRangeKey: d.tableNewRangeKeyIter,
		seqNum:          seqNum,
		mergingIter:     &buf.merging,
	}
	if o != nil {
		dbi.opts = *o
	}
	dbi.opts.logger = d.opts.Logger
	if d.opts.private.disableLazyCombinedIteration {
		dbi.opts.disableLazyCombinedIteration = true
	}
	return finishInitializingInternalIter(buf, dbi)
}

func finishInitializingInternalIter(buf *iterAlloc, i *scanInternalIterator) *scanInternalIterator {
	// Short-hand.
	var memtables flushableList
	if i.readState != nil {
		memtables = i.readState.memtables
	}
	// We only need to read from memtables which contain sequence numbers older
	// than seqNum. Trim off newer memtables.
	for j := len(memtables) - 1; j >= 0; j-- {
		if logSeqNum := memtables[j].logSeqNum; logSeqNum < i.seqNum {
			break
		}
		memtables = memtables[:j]
	}
	i.initializeBoundBufs(i.opts.LowerBound, i.opts.UpperBound)

	i.constructPointIter(memtables, buf)

	// For internal iterators, we skip the lazy combined iteration optimization
	// entirely, and create the range key iterator stack directly.
	i.rangeKey = iterRangeKeyStateAllocPool.Get().(*iteratorRangeKeyState)
	i.rangeKey.init(i.comparer.Compare, i.comparer.Split, &i.opts.IterOptions)
	i.constructRangeKeyIter()

	// Wrap the point iterator (currently i.iter) with an interleaving
	// iterator that interleaves range keys pulled from
	// i.rangeKey.rangeKeyIter.
	i.rangeKey.iiter.Init(i.comparer, i.iter, i.rangeKey.rangeKeyIter,
		keyspan.InterleavingIterOpts{
			LowerBound: i.opts.LowerBound,
			UpperBound: i.opts.UpperBound,
		})
	i.iter = &i.rangeKey.iiter

	return i
}

func (i *Iterator) constructPointIter(
	ctx context.Context, memtables flushableList, buf *iterAlloc,
) {
	if i.pointIter != nil {
		// Already have one.
		return
	}
	internalOpts := internalIterOpts{stats: &i.stats.InternalStats}
	if i.opts.RangeKeyMasking.Filter != nil {
		internalOpts.boundLimitedFilter = &i.rangeKeyMasking
	}

	// Merging levels and levels from iterAlloc.
	mlevels := buf.mlevels[:0]
	levels := buf.levels[:0]

	// We compute the number of levels needed ahead of time and reallocate a slice if
	// the array from the iterAlloc isn't large enough. Doing this allocation once
	// should improve the performance.
	numMergingLevels := 0
	numLevelIters := 0
	if i.batch != nil {
		numMergingLevels++
	}
	numMergingLevels += len(memtables)

	current := i.version
	if current == nil {
		current = i.readState.current
	}
	numMergingLevels += len(current.L0SublevelFiles)
	numLevelIters += len(current.L0SublevelFiles)
	for level := 1; level < len(current.Levels); level++ {
		if current.Levels[level].Empty() {
			continue
		}
		numMergingLevels++
		numLevelIters++
	}

	if numMergingLevels > cap(mlevels) {
		mlevels = make([]mergingIterLevel, 0, numMergingLevels)
	}
	if numLevelIters > cap(levels) {
		levels = make([]levelIter, 0, numLevelIters)
	}

	// Top-level is the batch, if any.
	if i.batch != nil {
		if i.batch.index == nil {
			// This isn't an indexed batch. Include an error iterator so that
			// the resulting iterator correctly surfaces ErrIndexed.
			mlevels = append(mlevels, mergingIterLevel{
				iter:         newErrorIter(ErrNotIndexed),
				rangeDelIter: newErrorKeyspanIter(ErrNotIndexed),
			})
		} else {
			i.batch.initInternalIter(&i.opts, &i.batchPointIter)
			i.batch.initRangeDelIter(&i.opts, &i.batchRangeDelIter, i.batchSeqNum)
			// Only include the batch's rangedel iterator if it's non-empty.
			// This requires some subtle logic in the case a rangedel is later
			// written to the batch and the view of the batch is refreshed
			// during a call to SetOptions—in this case, we need to reconstruct
			// the point iterator to add the batch rangedel iterator.
			var rangeDelIter keyspan.FragmentIterator
			if i.batchRangeDelIter.Count() > 0 {
				rangeDelIter = &i.batchRangeDelIter
			}
			mlevels = append(mlevels, mergingIterLevel{
				iter:         &i.batchPointIter,
				rangeDelIter: rangeDelIter,
			})
		}
	}

	// Next are the memtables.
	for j := len(memtables) - 1; j >= 0; j-- {
		mem := memtables[j]
		mlevels = append(mlevels, mergingIterLevel{
			iter:         mem.newIter(&i.opts),
			rangeDelIter: mem.newRangeDelIter(&i.opts),
		})
	}

	// Next are the file levels: L0 sub-levels followed by lower levels.
	mlevelsIndex := len(mlevels)
	levelsIndex := len(levels)
	mlevels = mlevels[:numMergingLevels]
	levels = levels[:numLevelIters]
	i.opts.snapshotForHideObsoletePoints = buf.dbi.seqNum
	addLevelIterForFiles := func(files manifest.LevelIterator, level manifest.Level) {
		li := &levels[levelsIndex]

		li.init(ctx, i.opts, &i.comparer, i.newIters, files, level, internalOpts)
		li.initRangeDel(&mlevels[mlevelsIndex].rangeDelIter)
		li.initBoundaryContext(&mlevels[mlevelsIndex].levelIterBoundaryContext)
		li.initCombinedIterState(&i.lazyCombinedIter.combinedIterState)
		mlevels[mlevelsIndex].levelIter = li
		mlevels[mlevelsIndex].iter = invalidating.MaybeWrapIfInvariants(li)

		levelsIndex++
		mlevelsIndex++
	}

	// Add level iterators for the L0 sublevels, iterating from newest to
	// oldest.
	for i := len(current.L0SublevelFiles) - 1; i >= 0; i-- {
		addLevelIterForFiles(current.L0SublevelFiles[i].Iter(), manifest.L0Sublevel(i))
	}

	// Add level iterators for the non-empty non-L0 levels.
	for level := 1; level < len(current.Levels); level++ {
		if current.Levels[level].Empty() {
			continue
		}
		addLevelIterForFiles(current.Levels[level].Iter(), manifest.Level(level))
	}
	buf.merging.init(&i.opts, &i.stats.InternalStats, i.comparer.Compare, i.comparer.Split, mlevels...)
	if len(mlevels) <= cap(buf.levelsPositioned) {
		buf.merging.levelsPositioned = buf.levelsPositioned[:len(mlevels)]
	}
	buf.merging.snapshot = i.seqNum
	buf.merging.batchSnapshot = i.batchSeqNum
	buf.merging.combinedIterState = &i.lazyCombinedIter.combinedIterState
	i.pointIter = invalidating.MaybeWrapIfInvariants(&buf.merging)
	i.merging = &buf.merging
}

// NewBatch returns a new empty write-only batch. Any reads on the batch will
// return an error. If the batch is committed it will be applied to the DB.
func (d *DB) NewBatch() *Batch {
	return newBatch(d)
}

// NewBatchWithSize is mostly identical to NewBatch, but it will allocate the
// the specified memory space for the internal slice in advance.
func (d *DB) NewBatchWithSize(size int) *Batch {
	return newBatchWithSize(d, size)
}

// NewIndexedBatch returns a new empty read-write batch. Any reads on the batch
// will read from both the batch and the DB. If the batch is committed it will
// be applied to the DB. An indexed batch is slower that a non-indexed batch
// for insert operations. If you do not need to perform reads on the batch, use
// NewBatch instead.
func (d *DB) NewIndexedBatch() *Batch {
	return newIndexedBatch(d, d.opts.Comparer)
}

// NewIndexedBatchWithSize is mostly identical to NewIndexedBatch, but it will
// allocate the the specified memory space for the internal slice in advance.
func (d *DB) NewIndexedBatchWithSize(size int) *Batch {
	return newIndexedBatchWithSize(d, d.opts.Comparer, size)
}

// NewIter returns an iterator that is unpositioned (Iterator.Valid() will
// return false). The iterator can be positioned via a call to SeekGE, SeekLT,
// First or Last. The iterator provides a point-in-time view of the current DB
// state. This view is maintained by preventing file deletions and preventing
// memtables referenced by the iterator from being deleted. Using an iterator
// to maintain a long-lived point-in-time view of the DB state can lead to an
// apparent memory and disk usage leak. Use snapshots (see NewSnapshot) for
// point-in-time snapshots which avoids these problems.
func (d *DB) NewIter(o *IterOptions) (*Iterator, error) {
	return d.NewIterWithContext(context.Background(), o)
}

// NewIterWithContext is like NewIter, and additionally accepts a context for
// tracing.
func (d *DB) NewIterWithContext(ctx context.Context, o *IterOptions) (*Iterator, error) {
	return d.newIter(ctx, nil /* batch */, snapshotIterOpts{}, o), nil
}

// NewSnapshot returns a point-in-time view of the current DB state. Iterators
// created with this handle will all observe a stable snapshot of the current
// DB state. The caller must call Snapshot.Close() when the snapshot is no
// longer needed. Snapshots are not persisted across DB restarts (close ->
// open). Unlike the implicit snapshot maintained by an iterator, a snapshot
// will not prevent memtables from being released or sstables from being
// deleted. Instead, a snapshot prevents deletion of sequence numbers
// referenced by the snapshot.
func (d *DB) NewSnapshot() *Snapshot {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}

	d.mu.Lock()
	s := &Snapshot{
		db:     d,
		seqNum: d.mu.versions.visibleSeqNum.Load(),
	}
	d.mu.snapshots.pushBack(s)
	d.mu.Unlock()
	return s
}

// NewEventuallyFileOnlySnapshot returns a point-in-time view of the current DB
// state, similar to NewSnapshot, but with consistency constrained to the
// provided set of key ranges. See the comment at EventuallyFileOnlySnapshot for
// its semantics.
func (d *DB) NewEventuallyFileOnlySnapshot(keyRanges []KeyRange) *EventuallyFileOnlySnapshot {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}

	internalKeyRanges := make([]internalKeyRange, len(keyRanges))
	for i := range keyRanges {
		if i > 0 && d.cmp(keyRanges[i-1].End, keyRanges[i].Start) > 0 {
			panic("pebble: key ranges for eventually-file-only-snapshot not in order")
		}
		internalKeyRanges[i] = internalKeyRange{
			smallest: base.MakeInternalKey(keyRanges[i].Start, InternalKeySeqNumMax, InternalKeyKindMax),
			largest:  base.MakeExclusiveSentinelKey(InternalKeyKindRangeDelete, keyRanges[i].End),
		}
	}

	return d.makeEventuallyFileOnlySnapshot(keyRanges, internalKeyRanges)
}

// Close closes the DB.
//
// It is not safe to close a DB until all outstanding iterators are closed
// or to call Close concurrently with any other DB method. It is not valid
// to call any of a DB's methods after the DB has been closed.
func (d *DB) Close() error {
	// Lock the commit pipeline for the duration of Close. This prevents a race
	// with makeRoomForWrite. Rotating the WAL in makeRoomForWrite requires
	// dropping d.mu several times for I/O. If Close only holds d.mu, an
	// in-progress WAL rotation may re-acquire d.mu only once the database is
	// closed.
	//
	// Additionally, locking the commit pipeline makes it more likely that
	// (illegal) concurrent writes will observe d.closed.Load() != nil, creating
	// more understable panics if the database is improperly used concurrently
	// during Close.
	d.commit.mu.Lock()
	defer d.commit.mu.Unlock()
	d.mu.Lock()
	defer d.mu.Unlock()
	if err := d.closed.Load(); err != nil {
		panic(err)
	}

	// Clear the finalizer that is used to check that an unreferenced DB has been
	// closed. We're closing the DB here, so the check performed by that
	// finalizer isn't necessary.
	//
	// Note: this is a no-op if invariants are disabled or race is enabled.
	invariants.SetFinalizer(d.closed, nil)

	d.closed.Store(errors.WithStack(ErrClosed))
	close(d.closedCh)

	defer d.opts.Cache.Unref()

	for d.mu.compact.compactingCount > 0 || d.mu.compact.flushing {
		d.mu.compact.cond.Wait()
	}
	for d.mu.tableStats.loading {
		d.mu.tableStats.cond.Wait()
	}
	for d.mu.tableValidation.validating {
		d.mu.tableValidation.cond.Wait()
	}

	var err error
	if n := len(d.mu.compact.inProgress); n > 0 {
		err = errors.Errorf("pebble: %d unexpected in-progress compactions", errors.Safe(n))
	}
	err = firstError(err, d.mu.formatVers.marker.Close())
	err = firstError(err, d.tableCache.close())
	if !d.opts.ReadOnly {
		err = firstError(err, d.mu.log.Close())
	} else if d.mu.log.LogWriter != nil {
		panic("pebble: log-writer should be nil in read-only mode")
	}
	err = firstError(err, d.fileLock.Close())

	// Note that versionSet.close() only closes the MANIFEST. The versions list
	// is still valid for the checks below.
	err = firstError(err, d.mu.versions.close())

	err = firstError(err, d.dataDir.Close())
	if d.dataDir != d.walDir {
		err = firstError(err, d.walDir.Close())
	}

	d.readState.val.unrefLocked()

	current := d.mu.versions.currentVersion()
	for v := d.mu.versions.versions.Front(); true; v = v.Next() {
		refs := v.Refs()
		if v == current {
			if refs != 1 {
				err = firstError(err, errors.Errorf("leaked iterators: current\n%s", v))
			}
			break
		}
		if refs != 0 {
			err = firstError(err, errors.Errorf("leaked iterators:\n%s", v))
		}
	}

	for _, mem := range d.mu.mem.queue {
		// Usually, we'd want to delete the files returned by readerUnref. But
		// in this case, even if we're unreferencing the flushables, the
		// flushables aren't obsolete. They will be reconstructed during WAL
		// replay.
		mem.readerUnrefLocked(false)
	}
	// If there's an unused, recycled memtable, we need to release its memory.
	if obsoleteMemTable := d.memTableRecycle.Swap(nil); obsoleteMemTable != nil {
		d.freeMemTable(obsoleteMemTable)
	}
	if reserved := d.memTableReserved.Load(); reserved != 0 {
		err = firstError(err, errors.Errorf("leaked memtable reservation: %d", errors.Safe(reserved)))
	}

	// Since we called d.readState.val.unrefLocked() above, we are expected to
	// manually schedule deletion of obsolete files.
	if len(d.mu.versions.obsoleteTables) > 0 {
		d.deleteObsoleteFiles(d.mu.nextJobID)
	}

	d.mu.Unlock()
	d.compactionSchedulers.Wait()

	// Wait for all cleaning jobs to finish.
	d.cleanupManager.Close()

	// Sanity check metrics.
	if invariants.Enabled {
		m := d.Metrics()
		if m.Compact.NumInProgress > 0 || m.Compact.InProgressBytes > 0 {
			d.mu.Lock()
			panic(fmt.Sprintf("invalid metrics on close:\n%s", m))
		}
	}

	d.mu.Lock()

	// As a sanity check, ensure that there are no zombie tables. A non-zero count
	// hints at a reference count leak.
	if ztbls := len(d.mu.versions.zombieTables); ztbls > 0 {
		err = firstError(err, errors.Errorf("non-zero zombie file count: %d", ztbls))
	}

	err = firstError(err, d.objProvider.Close())

	// If the options include a closer to 'close' the filesystem, close it.
	if d.opts.private.fsCloser != nil {
		d.opts.private.fsCloser.Close()
	}

	// Return an error if the user failed to close all open snapshots.
	if v := d.mu.snapshots.count(); v > 0 {
		err = firstError(err, errors.Errorf("leaked snapshots: %d open snapshots on DB %p", v, d))
	}

	return err
}

// Compact the specified range of keys in the database.
func (d *DB) Compact(start, end []byte, parallelize bool) error {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	if d.opts.ReadOnly {
		return ErrReadOnly
	}
	if d.cmp(start, end) >= 0 {
		return errors.Errorf("Compact start %s is not less than end %s",
			d.opts.Comparer.FormatKey(start), d.opts.Comparer.FormatKey(end))
	}
	iStart := base.MakeInternalKey(start, InternalKeySeqNumMax, InternalKeyKindMax)
	iEnd := base.MakeInternalKey(end, 0, 0)
	m := (&fileMetadata{}).ExtendPointKeyBounds(d.cmp, iStart, iEnd)
	meta := []*fileMetadata{m}

	d.mu.Lock()
	maxLevelWithFiles := 1
	cur := d.mu.versions.currentVersion()
	for level := 0; level < numLevels; level++ {
		overlaps := cur.Overlaps(level, d.cmp, start, end, iEnd.IsExclusiveSentinel())
		if !overlaps.Empty() {
			maxLevelWithFiles = level + 1
		}
	}

	keyRanges := make([]internalKeyRange, len(meta))
	for i := range meta {
		keyRanges[i] = internalKeyRange{smallest: m.Smallest, largest: m.Largest}
	}
	// Determine if any memtable overlaps with the compaction range. We wait for
	// any such overlap to flush (initiating a flush if necessary).
	mem, err := func() (*flushableEntry, error) {
		// Check to see if any files overlap with any of the memtables. The queue
		// is ordered from oldest to newest with the mutable memtable being the
		// last element in the slice. We want to wait for the newest table that
		// overlaps.
		for i := len(d.mu.mem.queue) - 1; i >= 0; i-- {
			mem := d.mu.mem.queue[i]
			if ingestMemtableOverlaps(d.cmp, mem, keyRanges) {
				var err error
				if mem.flushable == d.mu.mem.mutable {
					// We have to hold both commitPipeline.mu and DB.mu when calling
					// makeRoomForWrite(). Lock order requirements elsewhere force us to
					// unlock DB.mu in order to grab commitPipeline.mu first.
					d.mu.Unlock()
					d.commit.mu.Lock()
					d.mu.Lock()
					defer d.commit.mu.Unlock()
					if mem.flushable == d.mu.mem.mutable {
						// Only flush if the active memtable is unchanged.
						err = d.makeRoomForWrite(nil)
					}
				}
				mem.flushForced = true
				d.maybeScheduleFlush()
				return mem, err
			}
		}
		return nil, nil
	}()

	d.mu.Unlock()

	if err != nil {
		return err
	}
	if mem != nil {
		<-mem.flushed
	}

	for level := 0; level < maxLevelWithFiles; {
		if err := d.manualCompact(
			iStart.UserKey, iEnd.UserKey, level, parallelize); err != nil {
			return err
		}
		level++
		if level == numLevels-1 {
			// A manual compaction of the bottommost level occurred.
			// There is no next level to try and compact.
			break
		}
	}
	return nil
}

func (d *DB) manualCompact(start, end []byte, level int, parallelize bool) error {
	d.mu.Lock()
	curr := d.mu.versions.currentVersion()
	files := curr.Overlaps(level, d.cmp, start, end, false)
	if files.Empty() {
		d.mu.Unlock()
		return nil
	}

	var compactions []*manualCompaction
	if parallelize {
		compactions = append(compactions, d.splitManualCompaction(start, end, level)...)
	} else {
		compactions = append(compactions, &manualCompaction{
			level: level,
			done:  make(chan error, 1),
			start: start,
			end:   end,
		})
	}
	d.mu.compact.manual = append(d.mu.compact.manual, compactions...)
	d.maybeScheduleCompaction()
	d.mu.Unlock()

	// Each of the channels is guaranteed to be eventually sent to once. After a
	// compaction is possibly picked in d.maybeScheduleCompaction(), either the
	// compaction is dropped, executed after being scheduled, or retried later.
	// Assuming eventual progress when a compaction is retried, all outcomes send
	// a value to the done channel. Since the channels are buffered, it is not
	// necessary to read from each channel, and so we can exit early in the event
	// of an error.
	for _, compaction := range compactions {
		if err := <-compaction.done; err != nil {
			return err
		}
	}
	return nil
}

// splitManualCompaction splits a manual compaction over [start,end] on level
// such that the resulting compactions have no key overlap.
func (d *DB) splitManualCompaction(
	start, end []byte, level int,
) (splitCompactions []*manualCompaction) {
	curr := d.mu.versions.currentVersion()
	endLevel := level + 1
	baseLevel := d.mu.versions.picker.getBaseLevel()
	if level == 0 {
		endLevel = baseLevel
	}
	keyRanges := calculateInuseKeyRanges(curr, d.cmp, level, endLevel, start, end)
	for _, keyRange := range keyRanges {
		splitCompactions = append(splitCompactions, &manualCompaction{
			level: level,
			done:  make(chan error, 1),
			start: keyRange.Start,
			end:   keyRange.End,
			split: true,
		})
	}
	return splitCompactions
}

// DownloadSpan is a key range passed to the Download method.
type DownloadSpan struct {
	StartKey []byte
	// EndKey is exclusive.
	EndKey []byte
}

// Download ensures that the LSM does not use any external sstables for the
// given key ranges. It does so by performing appropriate compactions so that
// all external data becomes available locally.
//
// Note that calling this method does not imply that all other compactions stop;
// it simply informs Pebble of a list of spans for which external data should be
// downloaded with high priority.
//
// The method returns once no external sstasbles overlap the given spans, the
// context is canceled, or an error is hit.
//
// TODO(radu): consider passing a priority/impact knob to express how important
// the download is (versus live traffic performance, LSM health).
func (d *DB) Download(ctx context.Context, spans []DownloadSpan) error {
	return errors.Errorf("not implemented")
}

// Flush the memtable to stable storage.
func (d *DB) Flush() error {
	flushDone, err := d.AsyncFlush()
	if err != nil {
		return err
	}
	<-flushDone
	return nil
}

// AsyncFlush asynchronously flushes the memtable to stable storage.
//
// If no error is returned, the caller can receive from the returned channel in
// order to wait for the flush to complete.
func (d *DB) AsyncFlush() (<-chan struct{}, error) {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	if d.opts.ReadOnly {
		return nil, ErrReadOnly
	}

	d.commit.mu.Lock()
	defer d.commit.mu.Unlock()
	d.mu.Lock()
	defer d.mu.Unlock()
	flushed := d.mu.mem.queue[len(d.mu.mem.queue)-1].flushed
	err := d.makeRoomForWrite(nil)
	if err != nil {
		return nil, err
	}
	return flushed, nil
}

// Metrics returns metrics about the database.
func (d *DB) Metrics() *Metrics {
	metrics := &Metrics{}
	recycledLogsCount, recycledLogSize := d.logRecycler.stats()

	d.mu.Lock()
	vers := d.mu.versions.currentVersion()
	*metrics = d.mu.versions.metrics
	metrics.Compact.EstimatedDebt = d.mu.versions.picker.estimatedCompactionDebt(0)
	metrics.Compact.InProgressBytes = d.mu.versions.atomicInProgressBytes.Load()
	metrics.Compact.NumInProgress = int64(d.mu.compact.compactingCount)
	metrics.Compact.MarkedFiles = vers.Stats.MarkedForCompaction
	metrics.Compact.Duration = d.mu.compact.duration
	for c := range d.mu.compact.inProgress {
		if c.kind != compactionKindFlush {
			metrics.Compact.Duration += d.timeNow().Sub(c.beganAt)
		}
	}

	for _, m := range d.mu.mem.queue {
		metrics.MemTable.Size += m.totalBytes()
	}
	metrics.Snapshots.Count = d.mu.snapshots.count()
	if metrics.Snapshots.Count > 0 {
		metrics.Snapshots.EarliestSeqNum = d.mu.snapshots.earliest()
	}
	metrics.Snapshots.PinnedKeys = d.mu.snapshots.cumulativePinnedCount
	metrics.Snapshots.PinnedSize = d.mu.snapshots.cumulativePinnedSize
	metrics.MemTable.Count = int64(len(d.mu.mem.queue))
	metrics.MemTable.ZombieCount = d.memTableCount.Load() - metrics.MemTable.Count
	metrics.MemTable.ZombieSize = uint64(d.memTableReserved.Load()) - metrics.MemTable.Size
	metrics.WAL.ObsoleteFiles = int64(recycledLogsCount)
	metrics.WAL.ObsoletePhysicalSize = recycledLogSize
	metrics.WAL.Size = d.logSize.Load()
	// The current WAL size (d.atomic.logSize) is the current logical size,
	// which may be less than the WAL's physical size if it was recycled.
	// The file sizes in d.mu.log.queue are updated to the physical size
	// during WAL rotation. Use the larger of the two for the current WAL. All
	// the previous WALs's fileSizes in d.mu.log.queue are already updated.
	metrics.WAL.PhysicalSize = metrics.WAL.Size
	if len(d.mu.log.queue) > 0 && metrics.WAL.PhysicalSize < d.mu.log.queue[len(d.mu.log.queue)-1].fileSize {
		metrics.WAL.PhysicalSize = d.mu.log.queue[len(d.mu.log.queue)-1].fileSize
	}
	for i, n := 0, len(d.mu.log.queue)-1; i < n; i++ {
		metrics.WAL.PhysicalSize += d.mu.log.queue[i].fileSize
	}

	metrics.WAL.BytesIn = d.mu.log.bytesIn // protected by d.mu
	for i, n := 0, len(d.mu.mem.queue)-1; i < n; i++ {
		metrics.WAL.Size += d.mu.mem.queue[i].logSize
	}
	metrics.WAL.BytesWritten = metrics.Levels[0].BytesIn + metrics.WAL.Size
	if p := d.mu.versions.picker; p != nil {
		compactions := d.getInProgressCompactionInfoLocked(nil)
		for level, score := range p.getScores(compactions) {
			metrics.Levels[level].Score = score
		}
	}
	metrics.Table.ZombieCount = int64(len(d.mu.versions.zombieTables))
	for _, size := range d.mu.versions.zombieTables {
		metrics.Table.ZombieSize += size
	}
	metrics.private.optionsFileSize = d.optionsFileSize

	// TODO(jackson): Consider making these metrics optional.
	metrics.Keys.RangeKeySetsCount = countRangeKeySetFragments(vers)
	metrics.Keys.TombstoneCount = countTombstones(vers)

	d.mu.versions.logLock()
	metrics.private.manifestFileSize = uint64(d.mu.versions.manifest.Size())
	metrics.Table.BackingTableCount = uint64(len(d.mu.versions.backingState.fileBackingMap))
	metrics.Table.BackingTableSize = d.mu.versions.backingState.fileBackingSize
	if invariants.Enabled {
		var totalSize uint64
		for _, backing := range d.mu.versions.backingState.fileBackingMap {
			totalSize += backing.Size
		}
		if totalSize != metrics.Table.BackingTableSize {
			panic("pebble: invalid backing table size accounting")
		}
	}
	d.mu.versions.logUnlock()

	metrics.LogWriter.FsyncLatency = d.mu.log.metrics.fsyncLatency
	if err := metrics.LogWriter.Merge(&d.mu.log.metrics.LogWriterMetrics); err != nil {
		d.opts.Logger.Infof("metrics error: %s", err)
	}
	metrics.Flush.WriteThroughput = d.mu.compact.flushWriteThroughput
	if d.mu.compact.flushing {
		metrics.Flush.NumInProgress = 1
	}
	for i := 0; i < numLevels; i++ {
		metrics.Levels[i].Additional.ValueBlocksSize = valueBlocksSizeForLevel(vers, i)
	}

	d.mu.Unlock()

	metrics.BlockCache = d.opts.Cache.Metrics()
	metrics.TableCache, metrics.Filter = d.tableCache.metrics()
	metrics.TableIters = int64(d.tableCache.iterCount())

	metrics.SecondaryCacheMetrics = d.objProvider.Metrics()

	metrics.Uptime = d.timeNow().Sub(d.openedAt)

	return metrics
}

// sstablesOptions hold the optional parameters to retrieve TableInfo for all sstables.
type sstablesOptions struct {
	// set to true will return the sstable properties in TableInfo
	withProperties bool

	// if set, return sstables that overlap the key range (end-exclusive)
	start []byte
	end   []byte

	withApproximateSpanBytes bool
}

// SSTablesOption set optional parameter used by `DB.SSTables`.
type SSTablesOption func(*sstablesOptions)

// WithProperties enable return sstable properties in each TableInfo.
//
// NOTE: if most of the sstable properties need to be read from disk,
// this options may make method `SSTables` quite slow.
func WithProperties() SSTablesOption {
	return func(opt *sstablesOptions) {
		opt.withProperties = true
	}
}

// WithKeyRangeFilter ensures returned sstables overlap start and end (end-exclusive)
// if start and end are both nil these properties have no effect.
func WithKeyRangeFilter(start, end []byte) SSTablesOption {
	return func(opt *sstablesOptions) {
		opt.end = end
		opt.start = start
	}
}

// WithApproximateSpanBytes enables capturing the approximate number of bytes that
// overlap the provided key span for each sstable.
// NOTE: this option can only be used with WithKeyRangeFilter and WithProperties
// provided.
func WithApproximateSpanBytes() SSTablesOption {
	return func(opt *sstablesOptions) {
		opt.withApproximateSpanBytes = true
	}
}

// BackingType denotes the type of storage backing a given sstable.
type BackingType int

const (
	// BackingTypeLocal denotes an sstable stored on local disk according to the
	// objprovider. This file is completely owned by us.
	BackingTypeLocal BackingType = iota
	// BackingTypeShared denotes an sstable stored on shared storage, created
	// by this Pebble instance and possibly shared by other Pebble instances.
	// These types of files have lifecycle managed by Pebble.
	BackingTypeShared
	// BackingTypeSharedForeign denotes an sstable stored on shared storage,
	// created by a Pebble instance other than this one. These types of files have
	// lifecycle managed by Pebble.
	BackingTypeSharedForeign
	// BackingTypeExternal denotes an sstable stored on external storage,
	// not owned by any Pebble instance and with no refcounting/cleanup methods
	// or lifecycle management. An example of an external file is a file restored
	// from a backup.
	BackingTypeExternal
)

// SSTableInfo export manifest.TableInfo with sstable.Properties alongside
// other file backing info.
type SSTableInfo struct {
	manifest.TableInfo
	// Virtual indicates whether the sstable is virtual.
	Virtual bool
	// BackingSSTNum is the file number associated with backing sstable which
	// backs the sstable associated with this SSTableInfo. If Virtual is false,
	// then BackingSSTNum == FileNum.
	BackingSSTNum base.FileNum
	// BackingType is the type of storage backing this sstable.
	BackingType BackingType
	// Locator is the remote.Locator backing this sstable, if the backing type is
	// not BackingTypeLocal.
	Locator remote.Locator

	// Properties is the sstable properties of this table. If Virtual is true,
	// then the Properties are associated with the backing sst.
	Properties *sstable.Properties
}

// SSTables retrieves the current sstables. The returned slice is indexed by
// level and each level is indexed by the position of the sstable within the
// level. Note that this information may be out of date due to concurrent
// flushes and compactions.
func (d *DB) SSTables(opts ...SSTablesOption) ([][]SSTableInfo, error) {
	opt := &sstablesOptions{}
	for _, fn := range opts {
		fn(opt)
	}

	if opt.withApproximateSpanBytes && !opt.withProperties {
		return nil, errors.Errorf("Cannot use WithApproximateSpanBytes without WithProperties option.")
	}
	if opt.withApproximateSpanBytes && (opt.start == nil || opt.end == nil) {
		return nil, errors.Errorf("Cannot use WithApproximateSpanBytes without WithKeyRangeFilter option.")
	}

	// Grab and reference the current readState.
	readState := d.loadReadState()
	defer readState.unref()

	// TODO(peter): This is somewhat expensive, especially on a large
	// database. It might be worthwhile to unify TableInfo and FileMetadata and
	// then we could simply return current.Files. Note that RocksDB is doing
	// something similar to the current code, so perhaps it isn't too bad.
	srcLevels := readState.current.Levels
	var totalTables int
	for i := range srcLevels {
		totalTables += srcLevels[i].Len()
	}

	destTables := make([]SSTableInfo, totalTables)
	destLevels := make([][]SSTableInfo, len(srcLevels))
	for i := range destLevels {
		iter := srcLevels[i].Iter()
		j := 0
		for m := iter.First(); m != nil; m = iter.Next() {
			if opt.start != nil && opt.end != nil && !m.Overlaps(d.opts.Comparer.Compare, opt.start, opt.end, true /* exclusive end */) {
				continue
			}
			destTables[j] = SSTableInfo{TableInfo: m.TableInfo()}
			if opt.withProperties {
				p, err := d.tableCache.getTableProperties(
					m,
				)
				if err != nil {
					return nil, err
				}
				destTables[j].Properties = p
			}
			destTables[j].Virtual = m.Virtual
			destTables[j].BackingSSTNum = m.FileBacking.DiskFileNum.FileNum()
			objMeta, err := d.objProvider.Lookup(fileTypeTable, m.FileBacking.DiskFileNum)
			if err != nil {
				return nil, err
			}
			if objMeta.IsRemote() {
				if objMeta.IsShared() {
					if d.objProvider.IsSharedForeign(objMeta) {
						destTables[j].BackingType = BackingTypeSharedForeign
					} else {
						destTables[j].BackingType = BackingTypeShared
					}
				} else {
					destTables[j].BackingType = BackingTypeExternal
				}
				destTables[j].Locator = objMeta.Remote.Locator
			} else {
				destTables[j].BackingType = BackingTypeLocal
			}

			if opt.withApproximateSpanBytes {
				var spanBytes uint64
				if m.ContainedWithinSpan(d.opts.Comparer.Compare, opt.start, opt.end) {
					spanBytes = m.Size
				} else {
					size, err := d.tableCache.estimateSize(m, opt.start, opt.end)
					if err != nil {
						return nil, err
					}
					spanBytes = size
				}
				propertiesCopy := *destTables[j].Properties

				// Deep copy user properties so approximate span bytes can be added.
				propertiesCopy.UserProperties = make(map[string]string, len(destTables[j].Properties.UserProperties)+1)
				for k, v := range destTables[j].Properties.UserProperties {
					propertiesCopy.UserProperties[k] = v
				}
				propertiesCopy.UserProperties["approximate-span-bytes"] = strconv.FormatUint(spanBytes, 10)
				destTables[j].Properties = &propertiesCopy
			}
			j++
		}
		destLevels[i] = destTables[:j]
		destTables = destTables[j:]
	}

	return destLevels, nil
}

// EstimateDiskUsage returns the estimated filesystem space used in bytes for
// storing the range `[start, end]`. The estimation is computed as follows:
//
//   - For sstables fully contained in the range the whole file size is included.
//   - For sstables partially contained in the range the overlapping data block sizes
//     are included. Even if a data block partially overlaps, or we cannot determine
//     overlap due to abbreviated index keys, the full data block size is included in
//     the estimation. Note that unlike fully contained sstables, none of the
//     meta-block space is counted for partially overlapped files.
//   - For virtual sstables, we use the overlap between start, end and the virtual
//     sstable bounds to determine disk usage.
//   - There may also exist WAL entries for unflushed keys in this range. This
//     estimation currently excludes space used for the range in the WAL.
func (d *DB) EstimateDiskUsage(start, end []byte) (uint64, error) {
	bytes, _, _, err := d.EstimateDiskUsageByBackingType(start, end)
	return bytes, err
}

// EstimateDiskUsageByBackingType is like EstimateDiskUsage but additionally
// returns the subsets of that size in remote ane external files.
func (d *DB) EstimateDiskUsageByBackingType(
	start, end []byte,
) (totalSize, remoteSize, externalSize uint64, _ error) {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	if d.opts.Comparer.Compare(start, end) > 0 {
		return 0, 0, 0, errors.New("invalid key-range specified (start > end)")
	}

	// Grab and reference the current readState. This prevents the underlying
	// files in the associated version from being deleted if there is a concurrent
	// compaction.
	readState := d.loadReadState()
	defer readState.unref()

	for level, files := range readState.current.Levels {
		iter := files.Iter()
		if level > 0 {
			// We can only use `Overlaps` to restrict `files` at L1+ since at L0 it
			// expands the range iteratively until it has found a set of files that
			// do not overlap any other L0 files outside that set.
			overlaps := readState.current.Overlaps(level, d.opts.Comparer.Compare, start, end, false /* exclusiveEnd */)
			iter = overlaps.Iter()
		}
		for file := iter.First(); file != nil; file = iter.Next() {
			if d.opts.Comparer.Compare(start, file.Smallest.UserKey) <= 0 &&
				d.opts.Comparer.Compare(file.Largest.UserKey, end) <= 0 {
				// The range fully contains the file, so skip looking it up in
				// table cache/looking at its indexes, and add the full file size.
				meta, err := d.objProvider.Lookup(fileTypeTable, file.FileBacking.DiskFileNum)
				if err != nil {
					return 0, 0, 0, err
				}
				if meta.IsRemote() {
					remoteSize += file.Size
					if meta.Remote.CleanupMethod == objstorage.SharedNoCleanup {
						externalSize += file.Size
					}
				}
				totalSize += file.Size
			} else if d.opts.Comparer.Compare(file.Smallest.UserKey, end) <= 0 &&
				d.opts.Comparer.Compare(start, file.Largest.UserKey) <= 0 {
				var size uint64
				var err error
				if file.Virtual {
					err = d.tableCache.withVirtualReader(
						file.VirtualMeta(),
						func(r sstable.VirtualReader) (err error) {
							size, err = r.EstimateDiskUsage(start, end)
							return err
						},
					)
				} else {
					err = d.tableCache.withReader(
						file.PhysicalMeta(),
						func(r *sstable.Reader) (err error) {
							size, err = r.EstimateDiskUsage(start, end)
							return err
						},
					)
				}
				if err != nil {
					return 0, 0, 0, err
				}
				meta, err := d.objProvider.Lookup(fileTypeTable, file.FileBacking.DiskFileNum)
				if err != nil {
					return 0, 0, 0, err
				}
				if meta.IsRemote() {
					remoteSize += size
					if meta.Remote.CleanupMethod == objstorage.SharedNoCleanup {
						externalSize += size
					}
				}
				totalSize += size
			}
		}
	}
	return totalSize, remoteSize, externalSize, nil
}

func (d *DB) walPreallocateSize() int {
	// Set the WAL preallocate size to 110% of the memtable size. Note that there
	// is a bit of apples and oranges in units here as the memtabls size
	// corresponds to the memory usage of the memtable while the WAL size is the
	// size of the batches (plus overhead) stored in the WAL.
	//
	// TODO(peter): 110% of the memtable size is quite hefty for a block
	// size. This logic is taken from GetWalPreallocateBlockSize in
	// RocksDB. Could a smaller preallocation block size be used?
	size := d.opts.MemTableSize
	size = (size / 10) + size
	return int(size)
}

func (d *DB) newMemTable(logNum FileNum, logSeqNum uint64) (*memTable, *flushableEntry) {
	size := d.mu.mem.nextSize
	if d.mu.mem.nextSize < d.opts.MemTableSize {
		d.mu.mem.nextSize *= 2
		if d.mu.mem.nextSize > d.opts.MemTableSize {
			d.mu.mem.nextSize = d.opts.MemTableSize
		}
	}

	memtblOpts := memTableOptions{
		Options:   d.opts,
		logSeqNum: logSeqNum,
	}

	// Before attempting to allocate a new memtable, check if there's one
	// available for recycling in memTableRecycle. Large contiguous allocations
	// can be costly as fragmentation makes it more difficult to find a large
	// contiguous free space. We've observed 64MB allocations taking 10ms+.
	//
	// To reduce these costly allocations, up to 1 obsolete memtable is stashed
	// in `d.memTableRecycle` to allow a future memtable rotation to reuse
	// existing memory.
	var mem *memTable
	mem = d.memTableRecycle.Swap(nil)
	if mem != nil && uint64(len(mem.arenaBuf)) != size {
		d.freeMemTable(mem)
		mem = nil
	}
	if mem != nil {
		// Carry through the existing buffer and memory reservation.
		memtblOpts.arenaBuf = mem.arenaBuf
		memtblOpts.releaseAccountingReservation = mem.releaseAccountingReservation
	} else {
		mem = new(memTable)
		memtblOpts.arenaBuf = manual.New(int(size))
		memtblOpts.releaseAccountingReservation = d.opts.Cache.Reserve(int(size))
		d.memTableCount.Add(1)
		d.memTableReserved.Add(int64(size))

		// Note: this is a no-op if invariants are disabled or race is enabled.
		invariants.SetFinalizer(mem, checkMemTable)
	}
	mem.init(memtblOpts)

	entry := d.newFlushableEntry(mem, logNum, logSeqNum)
	entry.releaseMemAccounting = func() {
		// If the user leaks iterators, we may be releasing the memtable after
		// the DB is already closed. In this case, we want to just release the
		// memory because DB.Close won't come along to free it for us.
		if err := d.closed.Load(); err != nil {
			d.freeMemTable(mem)
			return
		}

		// The next memtable allocation might be able to reuse this memtable.
		// Stash it on d.memTableRecycle.
		if unusedMem := d.memTableRecycle.Swap(mem); unusedMem != nil {
			// There was already a memtable waiting to be recycled. We're now
			// responsible for freeing it.
			d.freeMemTable(unusedMem)
		}
	}
	return mem, entry
}

func (d *DB) freeMemTable(m *memTable) {
	d.memTableCount.Add(-1)
	d.memTableReserved.Add(-int64(len(m.arenaBuf)))
	m.free()
}

func (d *DB) newFlushableEntry(f flushable, logNum FileNum, logSeqNum uint64) *flushableEntry {
	fe := &flushableEntry{
		flushable:      f,
		flushed:        make(chan struct{}),
		logNum:         logNum,
		logSeqNum:      logSeqNum,
		deleteFn:       d.mu.versions.addObsolete,
		deleteFnLocked: d.mu.versions.addObsoleteLocked,
	}
	fe.readerRefs.Store(1)
	return fe
}

// makeRoomForWrite ensures that the memtable has room to hold the contents of
// Batch. It reserves the space in the memtable and adds a reference to the
// memtable. The caller must later ensure that the memtable is unreferenced. If
// the memtable is full, or a nil Batch is provided, the current memtable is
// rotated (marked as immutable) and a new mutable memtable is allocated. This
// memtable rotation also causes a log rotation.
//
// Both DB.mu and commitPipeline.mu must be held by the caller. Note that DB.mu
// may be released and reacquired.
func (d *DB) makeRoomForWrite(b *Batch) error {
	if b != nil && b.ingestedSSTBatch {
		panic("pebble: invalid function call")
	}

	force := b == nil || b.flushable != nil
	stalled := false
	for {
		if b != nil && b.flushable == nil {
			err := d.mu.mem.mutable.prepare(b)
			if err != arenaskl.ErrArenaFull {
				if stalled {
					d.opts.EventListener.WriteStallEnd()
				}
				return err
			}
		} else if !force {
			if stalled {
				d.opts.EventListener.WriteStallEnd()
			}
			return nil
		}
		// force || err == ErrArenaFull, so we need to rotate the current memtable.
		{
			var size uint64
			for i := range d.mu.mem.queue {
				size += d.mu.mem.queue[i].totalBytes()
			}
			if size >= uint64(d.opts.MemTableStopWritesThreshold)*d.opts.MemTableSize {
				// We have filled up the current memtable, but already queued memtables
				// are still flushing, so we wait.
				if !stalled {
					stalled = true
					d.opts.EventListener.WriteStallBegin(WriteStallBeginInfo{
						Reason: "memtable count limit reached",
					})
				}
				now := time.Now()
				d.mu.compact.cond.Wait()
				if b != nil {
					b.commitStats.MemTableWriteStallDuration += time.Since(now)
				}
				continue
			}
		}
		l0ReadAmp := d.mu.versions.currentVersion().L0Sublevels.ReadAmplification()
		if l0ReadAmp >= d.opts.L0StopWritesThreshold {
			// There are too many level-0 files, so we wait.
			if !stalled {
				stalled = true
				d.opts.EventListener.WriteStallBegin(WriteStallBeginInfo{
					Reason: "L0 file count limit exceeded",
				})
			}
			now := time.Now()
			d.mu.compact.cond.Wait()
			if b != nil {
				b.commitStats.L0ReadAmpWriteStallDuration += time.Since(now)
			}
			continue
		}

		var newLogNum base.FileNum
		var prevLogSize uint64
		if !d.opts.DisableWAL {
			now := time.Now()
			newLogNum, prevLogSize = d.rotateWAL()
			if b != nil {
				b.commitStats.WALRotationDuration += time.Since(now)
			}
		}

		immMem := d.mu.mem.mutable
		imm := d.mu.mem.queue[len(d.mu.mem.queue)-1]
		imm.logSize = prevLogSize
		imm.flushForced = imm.flushForced || (b == nil)

		// If we are manually flushing and we used less than half of the bytes in
		// the memtable, don't increase the size for the next memtable. This
		// reduces memtable memory pressure when an application is frequently
		// manually flushing.
		if (b == nil) && uint64(immMem.availBytes()) > immMem.totalBytes()/2 {
			d.mu.mem.nextSize = immMem.totalBytes()
		}

		if b != nil && b.flushable != nil {
			// The batch is too large to fit in the memtable so add it directly to
			// the immutable queue. The flushable batch is associated with the same
			// log as the immutable memtable, but logically occurs after it in
			// seqnum space. We ensure while flushing that the flushable batch
			// is flushed along with the previous memtable in the flushable
			// queue. See the top level comment in DB.flush1 to learn how this
			// is ensured.
			//
			// See DB.commitWrite for the special handling of log writes for large
			// batches. In particular, the large batch has already written to
			// imm.logNum.
			entry := d.newFlushableEntry(b.flushable, imm.logNum, b.SeqNum())
			// The large batch is by definition large. Reserve space from the cache
			// for it until it is flushed.
			entry.releaseMemAccounting = d.opts.Cache.Reserve(int(b.flushable.totalBytes()))
			d.mu.mem.queue = append(d.mu.mem.queue, entry)
		}

		var logSeqNum uint64
		if b != nil {
			logSeqNum = b.SeqNum()
			if b.flushable != nil {
				logSeqNum += uint64(b.Count())
			}
		} else {
			logSeqNum = d.mu.versions.logSeqNum.Load()
		}
		d.rotateMemtable(newLogNum, logSeqNum, immMem)
		force = false
	}
}

// Both DB.mu and commitPipeline.mu must be held by the caller.
func (d *DB) rotateMemtable(newLogNum FileNum, logSeqNum uint64, prev *memTable) {
	// Create a new memtable, scheduling the previous one for flushing. We do
	// this even if the previous memtable was empty because the DB.Flush
	// mechanism is dependent on being able to wait for the empty memtable to
	// flush. We can't just mark the empty memtable as flushed here because we
	// also have to wait for all previous immutable tables to
	// flush. Additionally, the memtable is tied to particular WAL file and we
	// want to go through the flush path in order to recycle that WAL file.
	//
	// NB: newLogNum corresponds to the WAL that contains mutations that are
	// present in the new memtable. When immutable memtables are flushed to
	// disk, a VersionEdit will be created telling the manifest the minimum
	// unflushed log number (which will be the next one in d.mu.mem.mutable
	// that was not flushed).
	//
	// NB: prev should be the current mutable memtable.
	var entry *flushableEntry
	d.mu.mem.mutable, entry = d.newMemTable(newLogNum, logSeqNum)
	d.mu.mem.queue = append(d.mu.mem.queue, entry)
	// d.logSize tracks the log size of the WAL file corresponding to the most
	// recent flushable. The log size of the previous mutable memtable no longer
	// applies to the current mutable memtable.
	//
	// It's tempting to perform this update in rotateWAL, but that would not be
	// atomic with the enqueue of the new flushable. A call to DB.Metrics()
	// could acquire DB.mu after the WAL has been rotated but before the new
	// memtable has been appended; this would result in omitting the log size of
	// the most recent flushable.
	d.logSize.Store(0)
	d.updateReadStateLocked(nil)
	if prev.writerUnref() {
		d.maybeScheduleFlush()
	}
}

// rotateWAL creates a new write-ahead log, possibly recycling a previous WAL's
// files. It returns the file number assigned to the new WAL, and the size of
// the previous WAL file.
//
// Both DB.mu and commitPipeline.mu must be held by the caller. Note that DB.mu
// may be released and reacquired.
func (d *DB) rotateWAL() (newLogNum FileNum, prevLogSize uint64) {
	if d.opts.DisableWAL {
		panic("pebble: invalid function call")
	}

	jobID := d.mu.nextJobID
	d.mu.nextJobID++
	newLogNum = d.mu.versions.getNextFileNum()

	prevLogSize = uint64(d.mu.log.Size())

	// The previous log may have grown past its original physical
	// size. Update its file size in the queue so we have a proper
	// accounting of its file size.
	if d.mu.log.queue[len(d.mu.log.queue)-1].fileSize < prevLogSize {
		d.mu.log.queue[len(d.mu.log.queue)-1].fileSize = prevLogSize
	}
	d.mu.Unlock()

	var err error
	// Close the previous log first. This writes an EOF trailer
	// signifying the end of the file and syncs it to disk. We must
	// close the previous log before linking the new log file,
	// otherwise a crash could leave both logs with unclean tails, and
	// Open will treat the previous log as corrupt.
	err = d.mu.log.LogWriter.Close()
	metrics := d.mu.log.LogWriter.Metrics()
	d.mu.Lock()
	if err := d.mu.log.metrics.Merge(metrics); err != nil {
		d.opts.Logger.Infof("metrics error: %s", err)
	}
	d.mu.Unlock()

	newLogName := base.MakeFilepath(d.opts.FS, d.walDirname, fileTypeLog, newLogNum.DiskFileNum())

	// Try to use a recycled log file. Recycling log files is an important
	// performance optimization as it is faster to sync a file that has
	// already been written, than one which is being written for the first
	// time. This is due to the need to sync file metadata when a file is
	// being written for the first time. Note this is true even if file
	// preallocation is performed (e.g. fallocate).
	var recycleLog fileInfo
	var recycleOK bool
	var newLogFile vfs.File
	if err == nil {
		recycleLog, recycleOK = d.logRecycler.peek()
		if recycleOK {
			recycleLogName := base.MakeFilepath(d.opts.FS, d.walDirname, fileTypeLog, recycleLog.fileNum)
			newLogFile, err = d.opts.FS.ReuseForWrite(recycleLogName, newLogName)
			base.MustExist(d.opts.FS, newLogName, d.opts.Logger, err)
		} else {
			newLogFile, err = d.opts.FS.Create(newLogName)
			base.MustExist(d.opts.FS, newLogName, d.opts.Logger, err)
		}
	}

	var newLogSize uint64
	if err == nil && recycleOK {
		// Figure out the recycled WAL size. This Stat is necessary
		// because ReuseForWrite's contract allows for removing the
		// old file and creating a new one. We don't know whether the
		// WAL was actually recycled.
		// TODO(jackson): Adding a boolean to the ReuseForWrite return
		// value indicating whether or not the file was actually
		// reused would allow us to skip the stat and use
		// recycleLog.fileSize.
		var finfo os.FileInfo
		finfo, err = newLogFile.Stat()
		if err == nil {
			newLogSize = uint64(finfo.Size())
		}
	}

	if err == nil {
		// TODO(peter): RocksDB delays sync of the parent directory until the
		// first time the log is synced. Is that worthwhile?
		err = d.walDir.Sync()
	}

	if err != nil && newLogFile != nil {
		newLogFile.Close()
	} else if err == nil {
		newLogFile = vfs.NewSyncingFile(newLogFile, vfs.SyncingFileOptions{
			NoSyncOnClose:   d.opts.NoSyncOnClose,
			BytesPerSync:    d.opts.WALBytesPerSync,
			PreallocateSize: d.walPreallocateSize(),
		})
	}

	if recycleOK {
		err = firstError(err, d.logRecycler.pop(recycleLog.fileNum.FileNum()))
	}

	d.opts.EventListener.WALCreated(WALCreateInfo{
		JobID:           jobID,
		Path:            newLogName,
		FileNum:         newLogNum,
		RecycledFileNum: recycleLog.fileNum.FileNum(),
		Err:             err,
	})

	d.mu.Lock()

	d.mu.versions.metrics.WAL.Files++

	if err != nil {
		// TODO(peter): avoid chewing through file numbers in a tight loop if there
		// is an error here.
		//
		// What to do here? Stumbling on doesn't seem worthwhile. If we failed to
		// close the previous log it is possible we lost a write.
		panic(err)
	}

	d.mu.log.queue = append(d.mu.log.queue, fileInfo{fileNum: newLogNum.DiskFileNum(), fileSize: newLogSize})
	d.mu.log.LogWriter = record.NewLogWriter(newLogFile, newLogNum, record.LogWriterConfig{
		WALFsyncLatency:    d.mu.log.metrics.fsyncLatency,
		WALMinSyncInterval: d.opts.WALMinSyncInterval,
		QueueSemChan:       d.commit.logSyncQSem,
	})
	if d.mu.log.registerLogWriterForTesting != nil {
		d.mu.log.registerLogWriterForTesting(d.mu.log.LogWriter)
	}

	return
}

func (d *DB) getEarliestUnflushedSeqNumLocked() uint64 {
	seqNum := InternalKeySeqNumMax
	for i := range d.mu.mem.queue {
		logSeqNum := d.mu.mem.queue[i].logSeqNum
		if seqNum > logSeqNum {
			seqNum = logSeqNum
		}
	}
	return seqNum
}

func (d *DB) getInProgressCompactionInfoLocked(finishing *compaction) (rv []compactionInfo) {
	for c := range d.mu.compact.inProgress {
		if len(c.flushing) == 0 && (finishing == nil || c != finishing) {
			info := compactionInfo{
				versionEditApplied: c.versionEditApplied,
				inputs:             c.inputs,
				smallest:           c.smallest,
				largest:            c.largest,
				outputLevel:        -1,
			}
			if c.outputLevel != nil {
				info.outputLevel = c.outputLevel.level
			}
			rv = append(rv, info)
		}
	}
	return
}

func inProgressL0Compactions(inProgress []compactionInfo) []manifest.L0Compaction {
	var compactions []manifest.L0Compaction
	for _, info := range inProgress {
		// Skip in-progress compactions that have already committed; the L0
		// sublevels initialization code requires the set of in-progress
		// compactions to be consistent with the current version. Compactions
		// with versionEditApplied=true are already applied to the current
		// version and but are performing cleanup without the database mutex.
		if info.versionEditApplied {
			continue
		}
		l0 := false
		for _, cl := range info.inputs {
			l0 = l0 || cl.level == 0
		}
		if !l0 {
			continue
		}
		compactions = append(compactions, manifest.L0Compaction{
			Smallest:  info.smallest,
			Largest:   info.largest,
			IsIntraL0: info.outputLevel == 0,
		})
	}
	return compactions
}

// firstError returns the first non-nil error of err0 and err1, or nil if both
// are nil.
func firstError(err0, err1 error) error {
	if err0 != nil {
		return err0
	}
	return err1
}

// SetCreatorID sets the CreatorID which is needed in order to use shared objects.
// Remote object usage is disabled until this method is called the first time.
// Once set, the Creator ID is persisted and cannot change.
//
// Does nothing if SharedStorage was not set in the options when the DB was
// opened or if the DB is in read-only mode.
func (d *DB) SetCreatorID(creatorID uint64) error {
	if d.opts.Experimental.RemoteStorage == nil || d.opts.ReadOnly {
		return nil
	}
	return d.objProvider.SetCreatorID(objstorage.CreatorID(creatorID))
}

// KeyStatistics keeps track of the number of keys that have been pinned by a
// snapshot as well as counts of the different key kinds in the lsm.
//
// One way of using the accumulated stats, when we only have sets and dels,
// and say the counts are represented as del_count, set_count,
// del_latest_count, set_latest_count, snapshot_pinned_count.
//
//   - del_latest_count + set_latest_count is the set of unique user keys
//     (unique).
//
//   - set_latest_count is the set of live unique user keys (live_unique).
//
//   - Garbage is del_count + set_count - live_unique.
//
//   - If everything were in the LSM, del_count+set_count-snapshot_pinned_count
//     would also be the set of unique user keys (note that
//     snapshot_pinned_count is counting something different -- see comment below).
//     But snapshot_pinned_count only counts keys in the LSM so the excess here
//     must be keys in memtables.
type KeyStatistics struct {
	// TODO(sumeer): the SnapshotPinned* are incorrect in that these older
	// versions can be in a different level. Either fix the accounting or
	// rename these fields.

	// SnapshotPinnedKeys represents obsolete keys that cannot be elided during
	// a compaction, because they are required by an open snapshot.
	SnapshotPinnedKeys int
	// SnapshotPinnedKeysBytes is the total number of bytes of all snapshot
	// pinned keys.
	SnapshotPinnedKeysBytes uint64
	// KindsCount is the count for each kind of key. It includes point keys,
	// range deletes and range keys.
	KindsCount [InternalKeyKindMax + 1]int
	// LatestKindsCount is the count for each kind of key when it is the latest
	// kind for a user key. It is only populated for point keys.
	LatestKindsCount [InternalKeyKindMax + 1]int
}

// LSMKeyStatistics is used by DB.ScanStatistics.
type LSMKeyStatistics struct {
	Accumulated KeyStatistics
	// Levels contains statistics only for point keys. Range deletions and range keys will
	// appear in Accumulated but not Levels.
	Levels [numLevels]KeyStatistics
	// BytesRead represents the logical, pre-compression size of keys and values read
	BytesRead uint64
}

// ScanStatisticsOptions is used by DB.ScanStatistics.
type ScanStatisticsOptions struct {
	// LimitBytesPerSecond indicates the number of bytes that are able to be read
	// per second using ScanInternal.
	// A value of 0 indicates that there is no limit set.
	LimitBytesPerSecond int64
}

// ScanStatistics returns the count of different key kinds within the lsm for a
// key span [lower, upper) as well as the number of snapshot keys.
func (d *DB) ScanStatistics(
	ctx context.Context, lower, upper []byte, opts ScanStatisticsOptions,
) (LSMKeyStatistics, error) {
	stats := LSMKeyStatistics{}
	var prevKey InternalKey
	var rateLimitFunc func(key *InternalKey, val LazyValue) error
	tb := tokenbucket.TokenBucket{}

	if opts.LimitBytesPerSecond != 0 {
		// Each "token" roughly corresponds to a byte that was read.
		tb.Init(tokenbucket.TokensPerSecond(opts.LimitBytesPerSecond), tokenbucket.Tokens(1024))
		rateLimitFunc = func(key *InternalKey, val LazyValue) error {
			return tb.WaitCtx(ctx, tokenbucket.Tokens(key.Size()+val.Len()))
		}
	}

	scanInternalOpts := &scanInternalOptions{
		visitPointKey: func(key *InternalKey, value LazyValue, iterInfo IteratorLevel) error {
			// If the previous key is equal to the current point key, the current key was
			// pinned by a snapshot.
			size := uint64(key.Size())
			kind := key.Kind()
			sameKey := d.equal(prevKey.UserKey, key.UserKey)
			if iterInfo.Kind == IteratorLevelLSM && sameKey {
				stats.Levels[iterInfo.Level].SnapshotPinnedKeys++
				stats.Levels[iterInfo.Level].SnapshotPinnedKeysBytes += size
				stats.Accumulated.SnapshotPinnedKeys++
				stats.Accumulated.SnapshotPinnedKeysBytes += size
			}
			if iterInfo.Kind == IteratorLevelLSM {
				stats.Levels[iterInfo.Level].KindsCount[kind]++
			}
			if !sameKey {
				if iterInfo.Kind == IteratorLevelLSM {
					stats.Levels[iterInfo.Level].LatestKindsCount[kind]++
				}
				stats.Accumulated.LatestKindsCount[kind]++
			}

			stats.Accumulated.KindsCount[kind]++
			prevKey.CopyFrom(*key)
			stats.BytesRead += uint64(key.Size() + value.Len())
			return nil
		},
		visitRangeDel: func(start, end []byte, seqNum uint64) error {
			stats.Accumulated.KindsCount[InternalKeyKindRangeDelete]++
			stats.BytesRead += uint64(len(start) + len(end))
			return nil
		},
		visitRangeKey: func(start, end []byte, keys []rangekey.Key) error {
			stats.BytesRead += uint64(len(start) + len(end))
			for _, key := range keys {
				stats.Accumulated.KindsCount[key.Kind()]++
				stats.BytesRead += uint64(len(key.Value) + len(key.Suffix))
			}
			return nil
		},
		includeObsoleteKeys: true,
		IterOptions: IterOptions{
			KeyTypes:   IterKeyTypePointsAndRanges,
			LowerBound: lower,
			UpperBound: upper,
		},
		rateLimitFunc: rateLimitFunc,
	}
	iter := d.newInternalIter(snapshotIterOpts{}, scanInternalOpts)
	defer iter.close()

	err := scanInternalImpl(ctx, lower, upper, iter, scanInternalOpts)

	if err != nil {
		return LSMKeyStatistics{}, err
	}

	return stats, nil
}

// ObjProvider returns the objstorage.Provider for this database. Meant to be
// used for internal purposes only.
func (d *DB) ObjProvider() objstorage.Provider {
	return d.objProvider
}

func (d *DB) checkVirtualBounds(m *fileMetadata) {
	if !invariants.Enabled {
		return
	}

	if m.HasPointKeys {
		pointIter, rangeDelIter, err := d.newIters(context.TODO(), m, nil, internalIterOpts{})
		if err != nil {
			panic(errors.Wrap(err, "pebble: error creating point iterator"))
		}

		defer pointIter.Close()
		if rangeDelIter != nil {
			defer rangeDelIter.Close()
		}

		pointKey, _ := pointIter.First()
		var rangeDel *keyspan.Span
		if rangeDelIter != nil {
			rangeDel = rangeDelIter.First()
		}

		// Check that the lower bound is tight.
		if (rangeDel == nil || d.cmp(rangeDel.SmallestKey().UserKey, m.SmallestPointKey.UserKey) != 0) &&
			(pointKey == nil || d.cmp(pointKey.UserKey, m.SmallestPointKey.UserKey) != 0) {
			panic(errors.Newf("pebble: virtual sstable %s lower point key bound is not tight", m.FileNum))
		}

		pointKey, _ = pointIter.Last()
		rangeDel = nil
		if rangeDelIter != nil {
			rangeDel = rangeDelIter.Last()
		}

		// Check that the upper bound is tight.
		if (rangeDel == nil || d.cmp(rangeDel.LargestKey().UserKey, m.LargestPointKey.UserKey) != 0) &&
			(pointKey == nil || d.cmp(pointKey.UserKey, m.LargestPointKey.UserKey) != 0) {
			panic(errors.Newf("pebble: virtual sstable %s upper point key bound is not tight", m.FileNum))
		}

		// Check that iterator keys are within bounds.
		for key, _ := pointIter.First(); key != nil; key, _ = pointIter.Next() {
			if d.cmp(key.UserKey, m.SmallestPointKey.UserKey) < 0 || d.cmp(key.UserKey, m.LargestPointKey.UserKey) > 0 {
				panic(errors.Newf("pebble: virtual sstable %s point key %s is not within bounds", m.FileNum, key.UserKey))
			}
		}

		if rangeDelIter != nil {
			for key := rangeDelIter.First(); key != nil; key = rangeDelIter.Next() {
				if d.cmp(key.SmallestKey().UserKey, m.SmallestPointKey.UserKey) < 0 {
					panic(errors.Newf("pebble: virtual sstable %s point key %s is not within bounds", m.FileNum, key.SmallestKey().UserKey))
				}

				if d.cmp(key.LargestKey().UserKey, m.LargestPointKey.UserKey) > 0 {
					panic(errors.Newf("pebble: virtual sstable %s point key %s is not within bounds", m.FileNum, key.LargestKey().UserKey))
				}
			}
		}
	}

	if !m.HasRangeKeys {
		return
	}

	rangeKeyIter, err := d.tableNewRangeKeyIter(m, keyspan.SpanIterOptions{})
	defer rangeKeyIter.Close()

	if err != nil {
		panic(errors.Wrap(err, "pebble: error creating range key iterator"))
	}

	// Check that the lower bound is tight.
	if d.cmp(rangeKeyIter.First().SmallestKey().UserKey, m.SmallestRangeKey.UserKey) != 0 {
		panic(errors.Newf("pebble: virtual sstable %s lower range key bound is not tight", m.FileNum))
	}

	// Check that upper bound is tight.
	if d.cmp(rangeKeyIter.Last().LargestKey().UserKey, m.LargestRangeKey.UserKey) != 0 {
		panic(errors.Newf("pebble: virtual sstable %s upper range key bound is not tight", m.FileNum))
	}

	for key := rangeKeyIter.First(); key != nil; key = rangeKeyIter.Next() {
		if d.cmp(key.SmallestKey().UserKey, m.SmallestRangeKey.UserKey) < 0 {
			panic(errors.Newf("pebble: virtual sstable %s point key %s is not within bounds", m.FileNum, key.SmallestKey().UserKey))
		}
		if d.cmp(key.LargestKey().UserKey, m.LargestRangeKey.UserKey) > 0 {
			panic(errors.Newf("pebble: virtual sstable %s point key %s is not within bounds", m.FileNum, key.LargestKey().UserKey))
		}
	}
}
