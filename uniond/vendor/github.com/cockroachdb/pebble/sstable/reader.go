// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"bytes"
	"context"
	"encoding/binary"
	"io"
	"os"
	"sort"
	"time"

	"github.com/cespare/xxhash/v2"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/bytealloc"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/cockroachdb/pebble/internal/crc"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/private"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/objiotracing"
)

var errCorruptIndexEntry = base.CorruptionErrorf("pebble/table: corrupt index entry")
var errReaderClosed = errors.New("pebble/table: reader is closed")

// decodeBlockHandle returns the block handle encoded at the start of src, as
// well as the number of bytes it occupies. It returns zero if given invalid
// input. A block handle for a data block or a first/lower level index block
// should not be decoded using decodeBlockHandle since the caller may validate
// that the number of bytes decoded is equal to the length of src, which will
// be false if the properties are not decoded. In those cases the caller
// should use decodeBlockHandleWithProperties.
func decodeBlockHandle(src []byte) (BlockHandle, int) {
	offset, n := binary.Uvarint(src)
	length, m := binary.Uvarint(src[n:])
	if n == 0 || m == 0 {
		return BlockHandle{}, 0
	}
	return BlockHandle{offset, length}, n + m
}

// decodeBlockHandleWithProperties returns the block handle and properties
// encoded in src. src needs to be exactly the length that was encoded. This
// method must be used for data block and first/lower level index blocks. The
// properties in the block handle point to the bytes in src.
func decodeBlockHandleWithProperties(src []byte) (BlockHandleWithProperties, error) {
	bh, n := decodeBlockHandle(src)
	if n == 0 {
		return BlockHandleWithProperties{}, errors.Errorf("invalid BlockHandle")
	}
	return BlockHandleWithProperties{
		BlockHandle: bh,
		Props:       src[n:],
	}, nil
}

func encodeBlockHandle(dst []byte, b BlockHandle) int {
	n := binary.PutUvarint(dst, b.Offset)
	m := binary.PutUvarint(dst[n:], b.Length)
	return n + m
}

func encodeBlockHandleWithProperties(dst []byte, b BlockHandleWithProperties) []byte {
	n := encodeBlockHandle(dst, b.BlockHandle)
	dst = append(dst[:n], b.Props...)
	return dst
}

// block is a []byte that holds a sequence of key/value pairs plus an index
// over those pairs.
type block []byte

type loadBlockResult int8

const (
	loadBlockOK loadBlockResult = iota
	// Could be due to error or because no block left to load.
	loadBlockFailed
	loadBlockIrrelevant
)

type blockTransform func([]byte) ([]byte, error)

// ReaderOption provide an interface to do work on Reader while it is being
// opened.
type ReaderOption interface {
	// readerApply is called on the reader during opening in order to set internal
	// parameters.
	readerApply(*Reader)
}

// Comparers is a map from comparer name to comparer. It is used for debugging
// tools which may be used on multiple databases configured with different
// comparers. Comparers implements the OpenOption interface and can be passed
// as a parameter to NewReader.
type Comparers map[string]*Comparer

func (c Comparers) readerApply(r *Reader) {
	if r.Compare != nil || r.Properties.ComparerName == "" {
		return
	}
	if comparer, ok := c[r.Properties.ComparerName]; ok {
		r.Compare = comparer.Compare
		r.FormatKey = comparer.FormatKey
		r.Split = comparer.Split
	}
}

// Mergers is a map from merger name to merger. It is used for debugging tools
// which may be used on multiple databases configured with different
// mergers. Mergers implements the OpenOption interface and can be passed as
// a parameter to NewReader.
type Mergers map[string]*Merger

func (m Mergers) readerApply(r *Reader) {
	if r.mergerOK || r.Properties.MergerName == "" {
		return
	}
	_, r.mergerOK = m[r.Properties.MergerName]
}

// cacheOpts is a Reader open option for specifying the cache ID and sstable file
// number. If not specified, a unique cache ID will be used.
type cacheOpts struct {
	cacheID uint64
	fileNum base.DiskFileNum
}

// Marker function to indicate the option should be applied before reading the
// sstable properties and, in the write path, before writing the default
// sstable properties.
func (c *cacheOpts) preApply() {}

func (c *cacheOpts) readerApply(r *Reader) {
	if r.cacheID == 0 {
		r.cacheID = c.cacheID
	}
	if r.fileNum.FileNum() == 0 {
		r.fileNum = c.fileNum
	}
}

func (c *cacheOpts) writerApply(w *Writer) {
	if w.cacheID == 0 {
		w.cacheID = c.cacheID
	}
	if w.fileNum.FileNum() == 0 {
		w.fileNum = c.fileNum
	}
}

// rawTombstonesOpt is a Reader open option for specifying that range
// tombstones returned by Reader.NewRangeDelIter() should not be
// fragmented. Used by debug tools to get a raw view of the tombstones
// contained in an sstable.
type rawTombstonesOpt struct{}

func (rawTombstonesOpt) preApply() {}

func (rawTombstonesOpt) readerApply(r *Reader) {
	r.rawTombstones = true
}

func init() {
	private.SSTableCacheOpts = func(cacheID uint64, fileNum base.DiskFileNum) interface{} {
		return &cacheOpts{cacheID, fileNum}
	}
	private.SSTableRawTombstonesOpt = rawTombstonesOpt{}
}

// CommonReader abstracts functionality over a Reader or a VirtualReader. This
// can be used by code which doesn't care to distinguish between a reader and a
// virtual reader.
type CommonReader interface {
	NewRawRangeKeyIter() (keyspan.FragmentIterator, error)
	NewRawRangeDelIter() (keyspan.FragmentIterator, error)
	NewIterWithBlockPropertyFiltersAndContextEtc(
		ctx context.Context, lower, upper []byte,
		filterer *BlockPropertiesFilterer,
		hideObsoletePoints, useFilterBlock bool,
		stats *base.InternalIteratorStats,
		rp ReaderProvider,
	) (Iterator, error)
	NewCompactionIter(
		bytesIterated *uint64,
		rp ReaderProvider,
		bufferPool *BufferPool,
	) (Iterator, error)
	EstimateDiskUsage(start, end []byte) (uint64, error)
	CommonProperties() *CommonProperties
}

// Reader is a table reader.
type Reader struct {
	readable          objstorage.Readable
	cacheID           uint64
	fileNum           base.DiskFileNum
	err               error
	indexBH           BlockHandle
	filterBH          BlockHandle
	rangeDelBH        BlockHandle
	rangeKeyBH        BlockHandle
	rangeDelTransform blockTransform
	valueBIH          valueBlocksIndexHandle
	propertiesBH      BlockHandle
	metaIndexBH       BlockHandle
	footerBH          BlockHandle
	opts              ReaderOptions
	Compare           Compare
	FormatKey         base.FormatKey
	Split             Split
	tableFilter       *tableFilterReader
	// Keep types that are not multiples of 8 bytes at the end and with
	// decreasing size.
	Properties    Properties
	tableFormat   TableFormat
	rawTombstones bool
	mergerOK      bool
	checksumType  ChecksumType
	// metaBufferPool is a buffer pool used exclusively when opening a table and
	// loading its meta blocks. metaBufferPoolAlloc is used to batch-allocate
	// the BufferPool.pool slice as a part of the Reader allocation. It's
	// capacity 3 to accommodate the meta block (1), and both the compressed
	// properties block (1) and decompressed properties block (1)
	// simultaneously.
	metaBufferPool      BufferPool
	metaBufferPoolAlloc [3]allocedBuffer
}

// Close implements DB.Close, as documented in the pebble package.
func (r *Reader) Close() error {
	r.opts.Cache.Unref()

	if r.readable != nil {
		r.err = firstError(r.err, r.readable.Close())
		r.readable = nil
	}

	if r.err != nil {
		return r.err
	}
	// Make any future calls to Get, NewIter or Close return an error.
	r.err = errReaderClosed
	return nil
}

// NewIterWithBlockPropertyFilters returns an iterator for the contents of the
// table. If an error occurs, NewIterWithBlockPropertyFilters cleans up after
// itself and returns a nil iterator.
func (r *Reader) NewIterWithBlockPropertyFilters(
	lower, upper []byte,
	filterer *BlockPropertiesFilterer,
	useFilterBlock bool,
	stats *base.InternalIteratorStats,
	rp ReaderProvider,
) (Iterator, error) {
	return r.newIterWithBlockPropertyFiltersAndContext(
		context.Background(),
		lower, upper, filterer, false, useFilterBlock, stats, rp, nil,
	)
}

// NewIterWithBlockPropertyFiltersAndContextEtc is similar to
// NewIterWithBlockPropertyFilters and additionally accepts a context for
// tracing.
//
// If hideObsoletePoints, the callee assumes that filterer already includes
// obsoleteKeyBlockPropertyFilter. The caller can satisfy this contract by
// first calling TryAddBlockPropertyFilterForHideObsoletePoints.
func (r *Reader) NewIterWithBlockPropertyFiltersAndContextEtc(
	ctx context.Context,
	lower, upper []byte,
	filterer *BlockPropertiesFilterer,
	hideObsoletePoints, useFilterBlock bool,
	stats *base.InternalIteratorStats,
	rp ReaderProvider,
) (Iterator, error) {
	return r.newIterWithBlockPropertyFiltersAndContext(
		ctx, lower, upper, filterer, hideObsoletePoints, useFilterBlock, stats, rp, nil,
	)
}

// TryAddBlockPropertyFilterForHideObsoletePoints is expected to be called
// before the call to NewIterWithBlockPropertyFiltersAndContextEtc, to get the
// value of hideObsoletePoints and potentially add a block property filter.
func (r *Reader) TryAddBlockPropertyFilterForHideObsoletePoints(
	snapshotForHideObsoletePoints uint64,
	fileLargestSeqNum uint64,
	pointKeyFilters []BlockPropertyFilter,
) (hideObsoletePoints bool, filters []BlockPropertyFilter) {
	hideObsoletePoints = r.tableFormat >= TableFormatPebblev4 &&
		snapshotForHideObsoletePoints > fileLargestSeqNum
	if hideObsoletePoints {
		pointKeyFilters = append(pointKeyFilters, obsoleteKeyBlockPropertyFilter{})
	}
	return hideObsoletePoints, pointKeyFilters
}

func (r *Reader) newIterWithBlockPropertyFiltersAndContext(
	ctx context.Context,
	lower, upper []byte,
	filterer *BlockPropertiesFilterer,
	hideObsoletePoints bool,
	useFilterBlock bool,
	stats *base.InternalIteratorStats,
	rp ReaderProvider,
	v *virtualState,
) (Iterator, error) {
	// NB: pebble.tableCache wraps the returned iterator with one which performs
	// reference counting on the Reader, preventing the Reader from being closed
	// until the final iterator closes.
	if r.Properties.IndexType == twoLevelIndex {
		i := twoLevelIterPool.Get().(*twoLevelIterator)
		err := i.init(ctx, r, v, lower, upper, filterer, useFilterBlock, hideObsoletePoints, stats, rp, nil /* bufferPool */)
		if err != nil {
			return nil, err
		}
		return i, nil
	}

	i := singleLevelIterPool.Get().(*singleLevelIterator)
	err := i.init(ctx, r, v, lower, upper, filterer, useFilterBlock, hideObsoletePoints, stats, rp, nil /* bufferPool */)
	if err != nil {
		return nil, err
	}
	return i, nil
}

// NewIter returns an iterator for the contents of the table. If an error
// occurs, NewIter cleans up after itself and returns a nil iterator. NewIter
// must only be used when the Reader is guaranteed to outlive any LazyValues
// returned from the iter.
func (r *Reader) NewIter(lower, upper []byte) (Iterator, error) {
	return r.NewIterWithBlockPropertyFilters(
		lower, upper, nil, true /* useFilterBlock */, nil, /* stats */
		TrivialReaderProvider{Reader: r})
}

// NewCompactionIter returns an iterator similar to NewIter but it also increments
// the number of bytes iterated. If an error occurs, NewCompactionIter cleans up
// after itself and returns a nil iterator.
func (r *Reader) NewCompactionIter(
	bytesIterated *uint64, rp ReaderProvider, bufferPool *BufferPool,
) (Iterator, error) {
	return r.newCompactionIter(bytesIterated, rp, nil, bufferPool)
}

func (r *Reader) newCompactionIter(
	bytesIterated *uint64, rp ReaderProvider, v *virtualState, bufferPool *BufferPool,
) (Iterator, error) {
	if r.Properties.IndexType == twoLevelIndex {
		i := twoLevelIterPool.Get().(*twoLevelIterator)
		err := i.init(
			context.Background(),
			r, v, nil /* lower */, nil /* upper */, nil,
			false /* useFilter */, v != nil && v.isForeign, /* hideObsoletePoints */
			nil /* stats */, rp, bufferPool,
		)
		if err != nil {
			return nil, err
		}
		i.setupForCompaction()
		return &twoLevelCompactionIterator{
			twoLevelIterator: i,
			bytesIterated:    bytesIterated,
		}, nil
	}
	i := singleLevelIterPool.Get().(*singleLevelIterator)
	err := i.init(
		context.Background(), r, v, nil /* lower */, nil, /* upper */
		nil, false /* useFilter */, v != nil && v.isForeign, /* hideObsoletePoints */
		nil /* stats */, rp, bufferPool,
	)
	if err != nil {
		return nil, err
	}
	i.setupForCompaction()
	return &compactionIterator{
		singleLevelIterator: i,
		bytesIterated:       bytesIterated,
	}, nil
}

// NewRawRangeDelIter returns an internal iterator for the contents of the
// range-del block for the table. Returns nil if the table does not contain
// any range deletions.
//
// TODO(sumeer): plumb context.Context since this path is relevant in the user-facing
// iterator. Add WithContext methods since the existing ones are public.
func (r *Reader) NewRawRangeDelIter() (keyspan.FragmentIterator, error) {
	if r.rangeDelBH.Length == 0 {
		return nil, nil
	}
	h, err := r.readRangeDel(nil /* stats */)
	if err != nil {
		return nil, err
	}
	i := &fragmentBlockIter{elideSameSeqnum: true}
	if err := i.blockIter.initHandle(r.Compare, h, r.Properties.GlobalSeqNum, false); err != nil {
		return nil, err
	}
	return i, nil
}

// NewRawRangeKeyIter returns an internal iterator for the contents of the
// range-key block for the table. Returns nil if the table does not contain any
// range keys.
//
// TODO(sumeer): plumb context.Context since this path is relevant in the user-facing
// iterator. Add WithContext methods since the existing ones are public.
func (r *Reader) NewRawRangeKeyIter() (keyspan.FragmentIterator, error) {
	if r.rangeKeyBH.Length == 0 {
		return nil, nil
	}
	h, err := r.readRangeKey(nil /* stats */)
	if err != nil {
		return nil, err
	}
	i := rangeKeyFragmentBlockIterPool.Get().(*rangeKeyFragmentBlockIter)
	if err := i.blockIter.initHandle(r.Compare, h, r.Properties.GlobalSeqNum, false); err != nil {
		return nil, err
	}
	return i, nil
}

type rangeKeyFragmentBlockIter struct {
	fragmentBlockIter
}

func (i *rangeKeyFragmentBlockIter) Close() error {
	err := i.fragmentBlockIter.Close()
	i.fragmentBlockIter = i.fragmentBlockIter.resetForReuse()
	rangeKeyFragmentBlockIterPool.Put(i)
	return err
}

func (r *Reader) readIndex(
	ctx context.Context, stats *base.InternalIteratorStats,
) (bufferHandle, error) {
	ctx = objiotracing.WithBlockType(ctx, objiotracing.MetadataBlock)
	return r.readBlock(ctx, r.indexBH, nil, nil, stats, nil /* buffer pool */)
}

func (r *Reader) readFilter(
	ctx context.Context, stats *base.InternalIteratorStats,
) (bufferHandle, error) {
	ctx = objiotracing.WithBlockType(ctx, objiotracing.FilterBlock)
	return r.readBlock(ctx, r.filterBH, nil /* transform */, nil /* readHandle */, stats, nil /* buffer pool */)
}

func (r *Reader) readRangeDel(stats *base.InternalIteratorStats) (bufferHandle, error) {
	ctx := objiotracing.WithBlockType(context.Background(), objiotracing.MetadataBlock)
	return r.readBlock(ctx, r.rangeDelBH, r.rangeDelTransform, nil /* readHandle */, stats, nil /* buffer pool */)
}

func (r *Reader) readRangeKey(stats *base.InternalIteratorStats) (bufferHandle, error) {
	ctx := objiotracing.WithBlockType(context.Background(), objiotracing.MetadataBlock)
	return r.readBlock(ctx, r.rangeKeyBH, nil /* transform */, nil /* readHandle */, stats, nil /* buffer pool */)
}

func checkChecksum(
	checksumType ChecksumType, b []byte, bh BlockHandle, fileNum base.FileNum,
) error {
	expectedChecksum := binary.LittleEndian.Uint32(b[bh.Length+1:])
	var computedChecksum uint32
	switch checksumType {
	case ChecksumTypeCRC32c:
		computedChecksum = crc.New(b[:bh.Length+1]).Value()
	case ChecksumTypeXXHash64:
		computedChecksum = uint32(xxhash.Sum64(b[:bh.Length+1]))
	default:
		return errors.Errorf("unsupported checksum type: %d", checksumType)
	}

	if expectedChecksum != computedChecksum {
		return base.CorruptionErrorf(
			"pebble/table: invalid table %s (checksum mismatch at %d/%d)",
			errors.Safe(fileNum), errors.Safe(bh.Offset), errors.Safe(bh.Length))
	}
	return nil
}

type cacheValueOrBuf struct {
	// buf.Valid() returns true if backed by a BufferPool.
	buf Buf
	// v is non-nil if backed by the block cache.
	v *cache.Value
}

func (b cacheValueOrBuf) get() []byte {
	if b.buf.Valid() {
		return b.buf.p.pool[b.buf.i].b
	}
	return b.v.Buf()
}

func (b cacheValueOrBuf) release() {
	if b.buf.Valid() {
		b.buf.Release()
	} else {
		cache.Free(b.v)
	}
}

func (b cacheValueOrBuf) truncate(n int) {
	if b.buf.Valid() {
		b.buf.p.pool[b.buf.i].b = b.buf.p.pool[b.buf.i].b[:n]
	} else {
		b.v.Truncate(n)
	}
}

func (r *Reader) readBlock(
	ctx context.Context,
	bh BlockHandle,
	transform blockTransform,
	readHandle objstorage.ReadHandle,
	stats *base.InternalIteratorStats,
	bufferPool *BufferPool,
) (handle bufferHandle, _ error) {
	if h := r.opts.Cache.Get(r.cacheID, r.fileNum, bh.Offset); h.Get() != nil {
		// Cache hit.
		if readHandle != nil {
			readHandle.RecordCacheHit(ctx, int64(bh.Offset), int64(bh.Length+blockTrailerLen))
		}
		if stats != nil {
			stats.BlockBytes += bh.Length
			stats.BlockBytesInCache += bh.Length
		}
		// This block is already in the cache; return a handle to existing vlaue
		// in the cache.
		return bufferHandle{h: h}, nil
	}

	// Cache miss.

	if sema := r.opts.LoadBlockSema; sema != nil {
		if err := sema.Acquire(ctx, 1); err != nil {
			// An error here can only come from the context.
			return bufferHandle{}, err
		}
		defer sema.Release(1)
	}

	var compressed cacheValueOrBuf
	if bufferPool != nil {
		compressed = cacheValueOrBuf{
			buf: bufferPool.Alloc(int(bh.Length + blockTrailerLen)),
		}
	} else {
		compressed = cacheValueOrBuf{
			v: cache.Alloc(int(bh.Length + blockTrailerLen)),
		}
	}

	readStartTime := time.Now()
	var err error
	if readHandle != nil {
		err = readHandle.ReadAt(ctx, compressed.get(), int64(bh.Offset))
	} else {
		err = r.readable.ReadAt(ctx, compressed.get(), int64(bh.Offset))
	}
	readDuration := time.Since(readStartTime)
	// TODO(sumeer): should the threshold be configurable.
	const slowReadTracingThreshold = 5 * time.Millisecond
	// The invariants.Enabled path is for deterministic testing.
	if invariants.Enabled {
		readDuration = slowReadTracingThreshold
	}
	// Call IsTracingEnabled to avoid the allocations of boxing integers into an
	// interface{}, unless necessary.
	if readDuration >= slowReadTracingThreshold && r.opts.LoggerAndTracer.IsTracingEnabled(ctx) {
		r.opts.LoggerAndTracer.Eventf(ctx, "reading %d bytes took %s",
			int(bh.Length+blockTrailerLen), readDuration.String())
	}
	if stats != nil {
		stats.BlockBytes += bh.Length
		stats.BlockReadDuration += readDuration
	}
	if err != nil {
		compressed.release()
		return bufferHandle{}, err
	}
	if err := checkChecksum(r.checksumType, compressed.get(), bh, r.fileNum.FileNum()); err != nil {
		compressed.release()
		return bufferHandle{}, err
	}

	typ := blockType(compressed.get()[bh.Length])
	compressed.truncate(int(bh.Length))

	var decompressed cacheValueOrBuf
	if typ == noCompressionBlockType {
		decompressed = compressed
	} else {
		// Decode the length of the decompressed value.
		decodedLen, prefixLen, err := decompressedLen(typ, compressed.get())
		if err != nil {
			compressed.release()
			return bufferHandle{}, err
		}

		if bufferPool != nil {
			decompressed = cacheValueOrBuf{buf: bufferPool.Alloc(decodedLen)}
		} else {
			decompressed = cacheValueOrBuf{v: cache.Alloc(decodedLen)}
		}
		if _, err := decompressInto(typ, compressed.get()[prefixLen:], decompressed.get()); err != nil {
			compressed.release()
			return bufferHandle{}, err
		}
		compressed.release()
	}

	if transform != nil {
		// Transforming blocks is very rare, so the extra copy of the
		// transformed data is not problematic.
		tmpTransformed, err := transform(decompressed.get())
		if err != nil {
			decompressed.release()
			return bufferHandle{}, err
		}

		var transformed cacheValueOrBuf
		if bufferPool != nil {
			transformed = cacheValueOrBuf{buf: bufferPool.Alloc(len(tmpTransformed))}
		} else {
			transformed = cacheValueOrBuf{v: cache.Alloc(len(tmpTransformed))}
		}
		copy(transformed.get(), tmpTransformed)
		decompressed.release()
		decompressed = transformed
	}

	if decompressed.buf.Valid() {
		return bufferHandle{b: decompressed.buf}, nil
	}
	h := r.opts.Cache.Set(r.cacheID, r.fileNum, bh.Offset, decompressed.v)
	return bufferHandle{h: h}, nil
}

func (r *Reader) transformRangeDelV1(b []byte) ([]byte, error) {
	// Convert v1 (RocksDB format) range-del blocks to v2 blocks on the fly. The
	// v1 format range-del blocks have unfragmented and unsorted range
	// tombstones. We need properly fragmented and sorted range tombstones in
	// order to serve from them directly.
	iter := &blockIter{}
	if err := iter.init(r.Compare, b, r.Properties.GlobalSeqNum, false); err != nil {
		return nil, err
	}
	var tombstones []keyspan.Span
	for key, value := iter.First(); key != nil; key, value = iter.Next() {
		t := keyspan.Span{
			Start: key.UserKey,
			End:   value.InPlaceValue(),
			Keys:  []keyspan.Key{{Trailer: key.Trailer}},
		}
		tombstones = append(tombstones, t)
	}
	keyspan.Sort(r.Compare, tombstones)

	// Fragment the tombstones, outputting them directly to a block writer.
	rangeDelBlock := blockWriter{
		restartInterval: 1,
	}
	frag := keyspan.Fragmenter{
		Cmp:    r.Compare,
		Format: r.FormatKey,
		Emit: func(s keyspan.Span) {
			for _, k := range s.Keys {
				startIK := InternalKey{UserKey: s.Start, Trailer: k.Trailer}
				rangeDelBlock.add(startIK, s.End)
			}
		},
	}
	for i := range tombstones {
		frag.Add(tombstones[i])
	}
	frag.Finish()

	// Return the contents of the constructed v2 format range-del block.
	return rangeDelBlock.finish(), nil
}

func (r *Reader) readMetaindex(metaindexBH BlockHandle) error {
	// We use a BufferPool when reading metaindex blocks in order to avoid
	// populating the block cache with these blocks. In heavy-write workloads,
	// especially with high compaction concurrency, new tables may be created
	// frequently. Populating the block cache with these metaindex blocks adds
	// additional contention on the block cache mutexes (see #1997).
	// Additionally, these blocks are exceedingly unlikely to be read again
	// while they're still in the block cache except in misconfigurations with
	// excessive sstables counts or a table cache that's far too small.
	r.metaBufferPool.initPreallocated(r.metaBufferPoolAlloc[:0])
	// When we're finished, release the buffers we've allocated back to memory
	// allocator. We don't expect to use metaBufferPool again.
	defer r.metaBufferPool.Release()

	b, err := r.readBlock(
		context.Background(), metaindexBH, nil /* transform */, nil /* readHandle */, nil /* stats */, &r.metaBufferPool)
	if err != nil {
		return err
	}
	data := b.Get()
	defer b.Release()

	if uint64(len(data)) != metaindexBH.Length {
		return base.CorruptionErrorf("pebble/table: unexpected metaindex block size: %d vs %d",
			errors.Safe(len(data)), errors.Safe(metaindexBH.Length))
	}

	i, err := newRawBlockIter(bytes.Compare, data)
	if err != nil {
		return err
	}

	meta := map[string]BlockHandle{}
	for valid := i.First(); valid; valid = i.Next() {
		value := i.Value()
		if bytes.Equal(i.Key().UserKey, []byte(metaValueIndexName)) {
			vbih, n, err := decodeValueBlocksIndexHandle(i.Value())
			if err != nil {
				return err
			}
			if n == 0 || n != len(value) {
				return base.CorruptionErrorf("pebble/table: invalid table (bad value blocks index handle)")
			}
			r.valueBIH = vbih
		} else {
			bh, n := decodeBlockHandle(value)
			if n == 0 || n != len(value) {
				return base.CorruptionErrorf("pebble/table: invalid table (bad block handle)")
			}
			meta[string(i.Key().UserKey)] = bh
		}
	}
	if err := i.Close(); err != nil {
		return err
	}

	if bh, ok := meta[metaPropertiesName]; ok {
		b, err = r.readBlock(
			context.Background(), bh, nil /* transform */, nil /* readHandle */, nil /* stats */, nil /* buffer pool */)
		if err != nil {
			return err
		}
		r.propertiesBH = bh
		err := r.Properties.load(b.Get(), bh.Offset, r.opts.DeniedUserProperties)
		b.Release()
		if err != nil {
			return err
		}
	}

	if bh, ok := meta[metaRangeDelV2Name]; ok {
		r.rangeDelBH = bh
	} else if bh, ok := meta[metaRangeDelName]; ok {
		r.rangeDelBH = bh
		if !r.rawTombstones {
			r.rangeDelTransform = r.transformRangeDelV1
		}
	}

	if bh, ok := meta[metaRangeKeyName]; ok {
		r.rangeKeyBH = bh
	}

	for name, fp := range r.opts.Filters {
		types := []struct {
			ftype  FilterType
			prefix string
		}{
			{TableFilter, "fullfilter."},
		}
		var done bool
		for _, t := range types {
			if bh, ok := meta[t.prefix+name]; ok {
				r.filterBH = bh

				switch t.ftype {
				case TableFilter:
					r.tableFilter = newTableFilterReader(fp)
				default:
					return base.CorruptionErrorf("unknown filter type: %v", errors.Safe(t.ftype))
				}

				done = true
				break
			}
		}
		if done {
			break
		}
	}
	return nil
}

// Layout returns the layout (block organization) for an sstable.
func (r *Reader) Layout() (*Layout, error) {
	if r.err != nil {
		return nil, r.err
	}

	l := &Layout{
		Data:       make([]BlockHandleWithProperties, 0, r.Properties.NumDataBlocks),
		Filter:     r.filterBH,
		RangeDel:   r.rangeDelBH,
		RangeKey:   r.rangeKeyBH,
		ValueIndex: r.valueBIH.h,
		Properties: r.propertiesBH,
		MetaIndex:  r.metaIndexBH,
		Footer:     r.footerBH,
		Format:     r.tableFormat,
	}

	indexH, err := r.readIndex(context.Background(), nil)
	if err != nil {
		return nil, err
	}
	defer indexH.Release()

	var alloc bytealloc.A

	if r.Properties.IndexPartitions == 0 {
		l.Index = append(l.Index, r.indexBH)
		iter, _ := newBlockIter(r.Compare, indexH.Get())
		for key, value := iter.First(); key != nil; key, value = iter.Next() {
			dataBH, err := decodeBlockHandleWithProperties(value.InPlaceValue())
			if err != nil {
				return nil, errCorruptIndexEntry
			}
			if len(dataBH.Props) > 0 {
				alloc, dataBH.Props = alloc.Copy(dataBH.Props)
			}
			l.Data = append(l.Data, dataBH)
		}
	} else {
		l.TopIndex = r.indexBH
		topIter, _ := newBlockIter(r.Compare, indexH.Get())
		iter := &blockIter{}
		for key, value := topIter.First(); key != nil; key, value = topIter.Next() {
			indexBH, err := decodeBlockHandleWithProperties(value.InPlaceValue())
			if err != nil {
				return nil, errCorruptIndexEntry
			}
			l.Index = append(l.Index, indexBH.BlockHandle)

			subIndex, err := r.readBlock(context.Background(), indexBH.BlockHandle,
				nil /* transform */, nil /* readHandle */, nil /* stats */, nil /* buffer pool */)
			if err != nil {
				return nil, err
			}
			if err := iter.init(r.Compare, subIndex.Get(), 0, /* globalSeqNum */
				false /* hideObsoletePoints */); err != nil {
				return nil, err
			}
			for key, value := iter.First(); key != nil; key, value = iter.Next() {
				dataBH, err := decodeBlockHandleWithProperties(value.InPlaceValue())
				if len(dataBH.Props) > 0 {
					alloc, dataBH.Props = alloc.Copy(dataBH.Props)
				}
				if err != nil {
					return nil, errCorruptIndexEntry
				}
				l.Data = append(l.Data, dataBH)
			}
			subIndex.Release()
			*iter = iter.resetForReuse()
		}
	}
	if r.valueBIH.h.Length != 0 {
		vbiH, err := r.readBlock(context.Background(), r.valueBIH.h, nil, nil, nil, nil /* buffer pool */)
		if err != nil {
			return nil, err
		}
		defer vbiH.Release()
		vbiBlock := vbiH.Get()
		indexEntryLen := int(r.valueBIH.blockNumByteLength + r.valueBIH.blockOffsetByteLength +
			r.valueBIH.blockLengthByteLength)
		i := 0
		for len(vbiBlock) != 0 {
			if len(vbiBlock) < indexEntryLen {
				return nil, errors.Errorf(
					"remaining value index block %d does not contain a full entry of length %d",
					len(vbiBlock), indexEntryLen)
			}
			n := int(r.valueBIH.blockNumByteLength)
			bn := int(littleEndianGet(vbiBlock, n))
			if bn != i {
				return nil, errors.Errorf("unexpected block num %d, expected %d",
					bn, i)
			}
			i++
			vbiBlock = vbiBlock[n:]
			n = int(r.valueBIH.blockOffsetByteLength)
			blockOffset := littleEndianGet(vbiBlock, n)
			vbiBlock = vbiBlock[n:]
			n = int(r.valueBIH.blockLengthByteLength)
			blockLen := littleEndianGet(vbiBlock, n)
			vbiBlock = vbiBlock[n:]
			l.ValueBlock = append(l.ValueBlock, BlockHandle{Offset: blockOffset, Length: blockLen})
		}
	}

	return l, nil
}

// ValidateBlockChecksums validates the checksums for each block in the SSTable.
func (r *Reader) ValidateBlockChecksums() error {
	// Pre-compute the BlockHandles for the underlying file.
	l, err := r.Layout()
	if err != nil {
		return err
	}

	// Construct the set of blocks to check. Note that the footer is not checked
	// as it is not a block with a checksum.
	blocks := make([]BlockHandle, len(l.Data))
	for i := range l.Data {
		blocks[i] = l.Data[i].BlockHandle
	}
	blocks = append(blocks, l.Index...)
	blocks = append(blocks, l.TopIndex, l.Filter, l.RangeDel, l.RangeKey, l.Properties, l.MetaIndex)

	// Sorting by offset ensures we are performing a sequential scan of the
	// file.
	sort.Slice(blocks, func(i, j int) bool {
		return blocks[i].Offset < blocks[j].Offset
	})

	// Check all blocks sequentially. Make use of read-ahead, given we are
	// scanning the entire file from start to end.
	rh := r.readable.NewReadHandle(context.TODO())
	defer rh.Close()

	for _, bh := range blocks {
		// Certain blocks may not be present, in which case we skip them.
		if bh.Length == 0 {
			continue
		}

		// Read the block, which validates the checksum.
		h, err := r.readBlock(context.Background(), bh, nil, rh, nil, nil /* buffer pool */)
		if err != nil {
			return err
		}
		h.Release()
	}

	return nil
}

// CommonProperties implemented the CommonReader interface.
func (r *Reader) CommonProperties() *CommonProperties {
	return &r.Properties.CommonProperties
}

// EstimateDiskUsage returns the total size of data blocks overlapping the range
// `[start, end]`. Even if a data block partially overlaps, or we cannot
// determine overlap due to abbreviated index keys, the full data block size is
// included in the estimation.
//
// This function does not account for any metablock space usage. Assumes there
// is at least partial overlap, i.e., `[start, end]` falls neither completely
// before nor completely after the file's range.
//
// Only blocks containing point keys are considered. Range deletion and range
// key blocks are not considered.
//
// TODO(ajkr): account for metablock space usage. Perhaps look at the fraction of
// data blocks overlapped and add that same fraction of the metadata blocks to the
// estimate.
func (r *Reader) EstimateDiskUsage(start, end []byte) (uint64, error) {
	if r.err != nil {
		return 0, r.err
	}

	indexH, err := r.readIndex(context.Background(), nil)
	if err != nil {
		return 0, err
	}
	defer indexH.Release()

	// Iterators over the bottom-level index blocks containing start and end.
	// These may be different in case of partitioned index but will both point
	// to the same blockIter over the single index in the unpartitioned case.
	var startIdxIter, endIdxIter *blockIter
	if r.Properties.IndexPartitions == 0 {
		iter, err := newBlockIter(r.Compare, indexH.Get())
		if err != nil {
			return 0, err
		}
		startIdxIter = iter
		endIdxIter = iter
	} else {
		topIter, err := newBlockIter(r.Compare, indexH.Get())
		if err != nil {
			return 0, err
		}

		key, val := topIter.SeekGE(start, base.SeekGEFlagsNone)
		if key == nil {
			// The range falls completely after this file, or an error occurred.
			return 0, topIter.Error()
		}
		startIdxBH, err := decodeBlockHandleWithProperties(val.InPlaceValue())
		if err != nil {
			return 0, errCorruptIndexEntry
		}
		startIdxBlock, err := r.readBlock(context.Background(), startIdxBH.BlockHandle,
			nil /* transform */, nil /* readHandle */, nil /* stats */, nil /* buffer pool */)
		if err != nil {
			return 0, err
		}
		defer startIdxBlock.Release()
		startIdxIter, err = newBlockIter(r.Compare, startIdxBlock.Get())
		if err != nil {
			return 0, err
		}

		key, val = topIter.SeekGE(end, base.SeekGEFlagsNone)
		if key == nil {
			if err := topIter.Error(); err != nil {
				return 0, err
			}
		} else {
			endIdxBH, err := decodeBlockHandleWithProperties(val.InPlaceValue())
			if err != nil {
				return 0, errCorruptIndexEntry
			}
			endIdxBlock, err := r.readBlock(context.Background(),
				endIdxBH.BlockHandle, nil /* transform */, nil /* readHandle */, nil /* stats */, nil /* buffer pool */)
			if err != nil {
				return 0, err
			}
			defer endIdxBlock.Release()
			endIdxIter, err = newBlockIter(r.Compare, endIdxBlock.Get())
			if err != nil {
				return 0, err
			}
		}
	}
	// startIdxIter should not be nil at this point, while endIdxIter can be if the
	// range spans past the end of the file.

	key, val := startIdxIter.SeekGE(start, base.SeekGEFlagsNone)
	if key == nil {
		// The range falls completely after this file, or an error occurred.
		return 0, startIdxIter.Error()
	}
	startBH, err := decodeBlockHandleWithProperties(val.InPlaceValue())
	if err != nil {
		return 0, errCorruptIndexEntry
	}

	includeInterpolatedValueBlocksSize := func(dataBlockSize uint64) uint64 {
		// INVARIANT: r.Properties.DataSize > 0 since startIdxIter is not nil.
		// Linearly interpolate what is stored in value blocks.
		//
		// TODO(sumeer): if we need more accuracy, without loading any data blocks
		// (which contain the value handles, and which may also be insufficient if
		// the values are in separate files), we will need to accumulate the
		// logical size of the key-value pairs and store the cumulative value for
		// each data block in the index block entry. This increases the size of
		// the BlockHandle, so wait until this becomes necessary.
		return dataBlockSize +
			uint64((float64(dataBlockSize)/float64(r.Properties.DataSize))*
				float64(r.Properties.ValueBlocksSize))
	}
	if endIdxIter == nil {
		// The range spans beyond this file. Include data blocks through the last.
		return includeInterpolatedValueBlocksSize(r.Properties.DataSize - startBH.Offset), nil
	}
	key, val = endIdxIter.SeekGE(end, base.SeekGEFlagsNone)
	if key == nil {
		if err := endIdxIter.Error(); err != nil {
			return 0, err
		}
		// The range spans beyond this file. Include data blocks through the last.
		return includeInterpolatedValueBlocksSize(r.Properties.DataSize - startBH.Offset), nil
	}
	endBH, err := decodeBlockHandleWithProperties(val.InPlaceValue())
	if err != nil {
		return 0, errCorruptIndexEntry
	}
	return includeInterpolatedValueBlocksSize(
		endBH.Offset + endBH.Length + blockTrailerLen - startBH.Offset), nil
}

// TableFormat returns the format version for the table.
func (r *Reader) TableFormat() (TableFormat, error) {
	if r.err != nil {
		return TableFormatUnspecified, r.err
	}
	return r.tableFormat, nil
}

// NewReader returns a new table reader for the file. Closing the reader will
// close the file.
func NewReader(f objstorage.Readable, o ReaderOptions, extraOpts ...ReaderOption) (*Reader, error) {
	o = o.ensureDefaults()
	r := &Reader{
		readable: f,
		opts:     o,
	}
	if r.opts.Cache == nil {
		r.opts.Cache = cache.New(0)
	} else {
		r.opts.Cache.Ref()
	}

	if f == nil {
		r.err = errors.New("pebble/table: nil file")
		return nil, r.Close()
	}

	// Note that the extra options are applied twice. First here for pre-apply
	// options, and then below for post-apply options. Pre and post refer to
	// before and after reading the metaindex and properties.
	type preApply interface{ preApply() }
	for _, opt := range extraOpts {
		if _, ok := opt.(preApply); ok {
			opt.readerApply(r)
		}
	}
	if r.cacheID == 0 {
		r.cacheID = r.opts.Cache.NewID()
	}

	footer, err := readFooter(f)
	if err != nil {
		r.err = err
		return nil, r.Close()
	}
	r.checksumType = footer.checksum
	r.tableFormat = footer.format
	// Read the metaindex.
	if err := r.readMetaindex(footer.metaindexBH); err != nil {
		r.err = err
		return nil, r.Close()
	}
	r.indexBH = footer.indexBH
	r.metaIndexBH = footer.metaindexBH
	r.footerBH = footer.footerBH

	if r.Properties.ComparerName == "" || o.Comparer.Name == r.Properties.ComparerName {
		r.Compare = o.Comparer.Compare
		r.FormatKey = o.Comparer.FormatKey
		r.Split = o.Comparer.Split
	}

	if o.MergerName == r.Properties.MergerName {
		r.mergerOK = true
	}

	// Apply the extra options again now that the comparer and merger names are
	// known.
	for _, opt := range extraOpts {
		if _, ok := opt.(preApply); !ok {
			opt.readerApply(r)
		}
	}

	if r.Compare == nil {
		r.err = errors.Errorf("pebble/table: %d: unknown comparer %s",
			errors.Safe(r.fileNum), errors.Safe(r.Properties.ComparerName))
	}
	if !r.mergerOK {
		if name := r.Properties.MergerName; name != "" && name != "nullptr" {
			r.err = errors.Errorf("pebble/table: %d: unknown merger %s",
				errors.Safe(r.fileNum), errors.Safe(r.Properties.MergerName))
		}
	}
	if r.err != nil {
		return nil, r.Close()
	}

	return r, nil
}

// ReadableFile describes the smallest subset of vfs.File that is required for
// reading SSTs.
type ReadableFile interface {
	io.ReaderAt
	io.Closer
	Stat() (os.FileInfo, error)
}

// NewSimpleReadable wraps a ReadableFile in a objstorage.Readable
// implementation (which does not support read-ahead)
func NewSimpleReadable(r ReadableFile) (objstorage.Readable, error) {
	info, err := r.Stat()
	if err != nil {
		return nil, err
	}
	res := &simpleReadable{
		f:    r,
		size: info.Size(),
	}
	res.rh = objstorage.MakeNoopReadHandle(res)
	return res, nil
}

// simpleReadable wraps a ReadableFile to implement objstorage.Readable.
type simpleReadable struct {
	f    ReadableFile
	size int64
	rh   objstorage.NoopReadHandle
}

var _ objstorage.Readable = (*simpleReadable)(nil)

// ReadAt is part of the objstorage.Readable interface.
func (s *simpleReadable) ReadAt(_ context.Context, p []byte, off int64) error {
	n, err := s.f.ReadAt(p, off)
	if invariants.Enabled && err == nil && n != len(p) {
		panic("short read")
	}
	return err
}

// Close is part of the objstorage.Readable interface.
func (s *simpleReadable) Close() error {
	return s.f.Close()
}

// Size is part of the objstorage.Readable interface.
func (s *simpleReadable) Size() int64 {
	return s.size
}

// NewReaddHandle is part of the objstorage.Readable interface.
func (s *simpleReadable) NewReadHandle(_ context.Context) objstorage.ReadHandle {
	return &s.rh
}
