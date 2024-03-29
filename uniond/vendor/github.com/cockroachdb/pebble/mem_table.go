// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"fmt"
	"os"
	"sync"
	"sync/atomic"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/arenaskl"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manual"
	"github.com/cockroachdb/pebble/internal/rangedel"
	"github.com/cockroachdb/pebble/internal/rangekey"
)

func memTableEntrySize(keyBytes, valueBytes int) uint64 {
	return arenaskl.MaxNodeSize(uint32(keyBytes)+8, uint32(valueBytes))
}

// memTableEmptySize is the amount of allocated space in the arena when the
// memtable is empty.
var memTableEmptySize = func() uint32 {
	var pointSkl arenaskl.Skiplist
	var rangeDelSkl arenaskl.Skiplist
	var rangeKeySkl arenaskl.Skiplist
	arena := arenaskl.NewArena(make([]byte, 16<<10 /* 16 KB */))
	pointSkl.Reset(arena, bytes.Compare)
	rangeDelSkl.Reset(arena, bytes.Compare)
	rangeKeySkl.Reset(arena, bytes.Compare)
	return arena.Size()
}()

// A memTable implements an in-memory layer of the LSM. A memTable is mutable,
// but append-only. Records are added, but never removed. Deletion is supported
// via tombstones, but it is up to higher level code (see Iterator) to support
// processing those tombstones.
//
// A memTable is implemented on top of a lock-free arena-backed skiplist. An
// arena is a fixed size contiguous chunk of memory (see
// Options.MemTableSize). A memTable's memory consumption is thus fixed at the
// time of creation (with the exception of the cached fragmented range
// tombstones). The arena-backed skiplist provides both forward and reverse
// links which makes forward and reverse iteration the same speed.
//
// A batch is "applied" to a memTable in a two step process: prepare(batch) ->
// apply(batch). memTable.prepare() is not thread-safe and must be called with
// external synchronization. Preparation reserves space in the memTable for the
// batch. Note that we pessimistically compute how much space a batch will
// consume in the memTable (see memTableEntrySize and
// Batch.memTableSize). Preparation is an O(1) operation. Applying a batch to
// the memTable can be performed concurrently with other apply
// operations. Applying a batch is an O(n logm) operation where N is the number
// of records in the batch and M is the number of records in the memtable. The
// commitPipeline serializes batch preparation, and allows batch application to
// proceed concurrently.
//
// It is safe to call get, apply, newIter, and newRangeDelIter concurrently.
type memTable struct {
	cmp         Compare
	formatKey   base.FormatKey
	equal       Equal
	arenaBuf    []byte
	skl         arenaskl.Skiplist
	rangeDelSkl arenaskl.Skiplist
	rangeKeySkl arenaskl.Skiplist
	// reserved tracks the amount of space used by the memtable, both by actual
	// data stored in the memtable as well as inflight batch commit
	// operations. This value is incremented pessimistically by prepare() in
	// order to account for the space needed by a batch.
	reserved uint32
	// writerRefs tracks the write references on the memtable. The two sources of
	// writer references are the memtable being on DB.mu.mem.queue and from
	// inflight mutations that have reserved space in the memtable but not yet
	// applied. The memtable cannot be flushed to disk until the writer refs
	// drops to zero.
	writerRefs atomic.Int32
	tombstones keySpanCache
	rangeKeys  keySpanCache
	// The current logSeqNum at the time the memtable was created. This is
	// guaranteed to be less than or equal to any seqnum stored in the memtable.
	logSeqNum                    uint64
	releaseAccountingReservation func()
}

func (m *memTable) free() {
	if m != nil {
		m.releaseAccountingReservation()
		manual.Free(m.arenaBuf)
		m.arenaBuf = nil
	}
}

// memTableOptions holds configuration used when creating a memTable. All of
// the fields are optional and will be filled with defaults if not specified
// which is used by tests.
type memTableOptions struct {
	*Options
	arenaBuf                     []byte
	size                         int
	logSeqNum                    uint64
	releaseAccountingReservation func()
}

func checkMemTable(obj interface{}) {
	m := obj.(*memTable)
	if m.arenaBuf != nil {
		fmt.Fprintf(os.Stderr, "%p: memTable buffer was not freed\n", m.arenaBuf)
		os.Exit(1)
	}
}

// newMemTable returns a new MemTable of the specified size. If size is zero,
// Options.MemTableSize is used instead.
func newMemTable(opts memTableOptions) *memTable {
	opts.Options = opts.Options.EnsureDefaults()
	m := new(memTable)
	m.init(opts)
	return m
}

func (m *memTable) init(opts memTableOptions) {
	if opts.size == 0 {
		opts.size = int(opts.MemTableSize)
	}
	*m = memTable{
		cmp:                          opts.Comparer.Compare,
		formatKey:                    opts.Comparer.FormatKey,
		equal:                        opts.Comparer.Equal,
		arenaBuf:                     opts.arenaBuf,
		logSeqNum:                    opts.logSeqNum,
		releaseAccountingReservation: opts.releaseAccountingReservation,
	}
	m.writerRefs.Store(1)
	m.tombstones = keySpanCache{
		cmp:           m.cmp,
		formatKey:     m.formatKey,
		skl:           &m.rangeDelSkl,
		constructSpan: rangeDelConstructSpan,
	}
	m.rangeKeys = keySpanCache{
		cmp:           m.cmp,
		formatKey:     m.formatKey,
		skl:           &m.rangeKeySkl,
		constructSpan: rangekey.Decode,
	}

	if m.arenaBuf == nil {
		m.arenaBuf = make([]byte, opts.size)
	}

	arena := arenaskl.NewArena(m.arenaBuf)
	m.skl.Reset(arena, m.cmp)
	m.rangeDelSkl.Reset(arena, m.cmp)
	m.rangeKeySkl.Reset(arena, m.cmp)
	m.reserved = arena.Size()
}

func (m *memTable) writerRef() {
	switch v := m.writerRefs.Add(1); {
	case v <= 1:
		panic(fmt.Sprintf("pebble: inconsistent reference count: %d", v))
	}
}

// writerUnref drops a ref on the memtable. Returns true if this was the last ref.
func (m *memTable) writerUnref() (wasLastRef bool) {
	switch v := m.writerRefs.Add(-1); {
	case v < 0:
		panic(fmt.Sprintf("pebble: inconsistent reference count: %d", v))
	case v == 0:
		return true
	default:
		return false
	}
}

// readyForFlush is part of the flushable interface.
func (m *memTable) readyForFlush() bool {
	return m.writerRefs.Load() == 0
}

// Prepare reserves space for the batch in the memtable and references the
// memtable preventing it from being flushed until the batch is applied. Note
// that prepare is not thread-safe, while apply is. The caller must call
// writerUnref() after the batch has been applied.
func (m *memTable) prepare(batch *Batch) error {
	avail := m.availBytes()
	if batch.memTableSize > uint64(avail) {
		return arenaskl.ErrArenaFull
	}
	m.reserved += uint32(batch.memTableSize)

	m.writerRef()
	return nil
}

func (m *memTable) apply(batch *Batch, seqNum uint64) error {
	if seqNum < m.logSeqNum {
		return base.CorruptionErrorf("pebble: batch seqnum %d is less than memtable creation seqnum %d",
			errors.Safe(seqNum), errors.Safe(m.logSeqNum))
	}

	var ins arenaskl.Inserter
	var tombstoneCount, rangeKeyCount uint32
	startSeqNum := seqNum
	for r := batch.Reader(); ; seqNum++ {
		kind, ukey, value, ok, err := r.Next()
		if !ok {
			if err != nil {
				return err
			}
			break
		}
		ikey := base.MakeInternalKey(ukey, seqNum, kind)
		switch kind {
		case InternalKeyKindRangeDelete:
			err = m.rangeDelSkl.Add(ikey, value)
			tombstoneCount++
		case InternalKeyKindRangeKeySet, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeyDelete:
			err = m.rangeKeySkl.Add(ikey, value)
			rangeKeyCount++
		case InternalKeyKindLogData:
			// Don't increment seqNum for LogData, since these are not applied
			// to the memtable.
			seqNum--
		case InternalKeyKindIngestSST:
			panic("pebble: cannot apply ingested sstable key kind to memtable")
		default:
			err = ins.Add(&m.skl, ikey, value)
		}
		if err != nil {
			return err
		}
	}
	if seqNum != startSeqNum+uint64(batch.Count()) {
		return base.CorruptionErrorf("pebble: inconsistent batch count: %d vs %d",
			errors.Safe(seqNum), errors.Safe(startSeqNum+uint64(batch.Count())))
	}
	if tombstoneCount != 0 {
		m.tombstones.invalidate(tombstoneCount)
	}
	if rangeKeyCount != 0 {
		m.rangeKeys.invalidate(rangeKeyCount)
	}
	return nil
}

// newIter is part of the flushable interface. It returns an iterator that is
// unpositioned (Iterator.Valid() will return false). The iterator can be
// positioned via a call to SeekGE, SeekLT, First or Last.
func (m *memTable) newIter(o *IterOptions) internalIterator {
	return m.skl.NewIter(o.GetLowerBound(), o.GetUpperBound())
}

// newFlushIter is part of the flushable interface.
func (m *memTable) newFlushIter(o *IterOptions, bytesFlushed *uint64) internalIterator {
	return m.skl.NewFlushIter(bytesFlushed)
}

// newRangeDelIter is part of the flushable interface.
func (m *memTable) newRangeDelIter(*IterOptions) keyspan.FragmentIterator {
	tombstones := m.tombstones.get()
	if tombstones == nil {
		return nil
	}
	return keyspan.NewIter(m.cmp, tombstones)
}

// newRangeKeyIter is part of the flushable interface.
func (m *memTable) newRangeKeyIter(*IterOptions) keyspan.FragmentIterator {
	rangeKeys := m.rangeKeys.get()
	if rangeKeys == nil {
		return nil
	}
	return keyspan.NewIter(m.cmp, rangeKeys)
}

// containsRangeKeys is part of the flushable interface.
func (m *memTable) containsRangeKeys() bool {
	return m.rangeKeys.count.Load() > 0
}

func (m *memTable) availBytes() uint32 {
	a := m.skl.Arena()
	if m.writerRefs.Load() == 1 {
		// If there are no other concurrent apply operations, we can update the
		// reserved bytes setting to accurately reflect how many bytes of been
		// allocated vs the over-estimation present in memTableEntrySize.
		m.reserved = a.Size()
	}
	return a.Capacity() - m.reserved
}

// inuseBytes is part of the flushable interface.
func (m *memTable) inuseBytes() uint64 {
	return uint64(m.skl.Size() - memTableEmptySize)
}

// totalBytes is part of the flushable interface.
func (m *memTable) totalBytes() uint64 {
	return uint64(m.skl.Arena().Capacity())
}

// empty returns whether the MemTable has no key/value pairs.
func (m *memTable) empty() bool {
	return m.skl.Size() == memTableEmptySize
}

// A keySpanFrags holds a set of fragmented keyspan.Spans with a particular key
// kind at a particular moment for a memtable.
//
// When a new span of a particular kind is added to the memtable, it may overlap
// with other spans of the same kind. Instead of performing the fragmentation
// whenever an iterator requires it, fragments are cached within a keySpanCache
// type. The keySpanCache uses keySpanFrags to hold the cached fragmented spans.
//
// The count of keys (and keys of any given kind) in a memtable only
// monotonically increases. The count of key spans of a particular kind is used
// as a stand-in for a 'sequence number'. A keySpanFrags represents the
// fragmented state of the memtable's keys of a given kind at the moment while
// there existed `count` keys of that kind in the memtable.
//
// It's currently only used to contain fragmented range deletion tombstones.
type keySpanFrags struct {
	count uint32
	once  sync.Once
	spans []keyspan.Span
}

type constructSpan func(ik base.InternalKey, v []byte, keysDst []keyspan.Key) (keyspan.Span, error)

func rangeDelConstructSpan(
	ik base.InternalKey, v []byte, keysDst []keyspan.Key,
) (keyspan.Span, error) {
	return rangedel.Decode(ik, v, keysDst), nil
}

// get retrieves the fragmented spans, populating them if necessary. Note that
// the populated span fragments may be built from more than f.count memTable
// spans, but that is ok for correctness. All we're requiring is that the
// memTable contains at least f.count keys of the configured kind. This
// situation can occur if there are multiple concurrent additions of the key
// kind and a concurrent reader. The reader can load a keySpanFrags and populate
// it even though is has been invalidated (i.e. replaced with a newer
// keySpanFrags).
func (f *keySpanFrags) get(
	skl *arenaskl.Skiplist, cmp Compare, formatKey base.FormatKey, constructSpan constructSpan,
) []keyspan.Span {
	f.once.Do(func() {
		frag := &keyspan.Fragmenter{
			Cmp:    cmp,
			Format: formatKey,
			Emit: func(fragmented keyspan.Span) {
				f.spans = append(f.spans, fragmented)
			},
		}
		it := skl.NewIter(nil, nil)
		var keysDst []keyspan.Key
		for key, val := it.First(); key != nil; key, val = it.Next() {
			s, err := constructSpan(*key, val.InPlaceValue(), keysDst)
			if err != nil {
				panic(err)
			}
			frag.Add(s)
			keysDst = s.Keys[len(s.Keys):]
		}
		frag.Finish()
	})
	return f.spans
}

// A keySpanCache is used to cache a set of fragmented spans. The cache is
// invalidated whenever a key of the same kind is added to a memTable, and
// populated when empty when a span iterator of that key kind is created.
type keySpanCache struct {
	count         atomic.Uint32
	frags         atomic.Pointer[keySpanFrags]
	cmp           Compare
	formatKey     base.FormatKey
	constructSpan constructSpan
	skl           *arenaskl.Skiplist
}

// Invalidate the current set of cached spans, indicating the number of
// spans that were added.
func (c *keySpanCache) invalidate(count uint32) {
	newCount := c.count.Add(count)
	var frags *keySpanFrags

	for {
		oldFrags := c.frags.Load()
		if oldFrags != nil && oldFrags.count >= newCount {
			// Someone else invalidated the cache before us and their invalidation
			// subsumes ours.
			break
		}
		if frags == nil {
			frags = &keySpanFrags{count: newCount}
		}
		if c.frags.CompareAndSwap(oldFrags, frags) {
			// We successfully invalidated the cache.
			break
		}
		// Someone else invalidated the cache. Loop and try again.
	}
}

func (c *keySpanCache) get() []keyspan.Span {
	frags := c.frags.Load()
	if frags == nil {
		return nil
	}
	return frags.get(c.skl, c.cmp, c.formatKey, c.constructSpan)
}
