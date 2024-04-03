// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sharedcache

import (
	"context"
	"fmt"
	"io"
	"math/bits"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/prometheus/client_golang/prometheus"
)

// Exported to enable exporting from package pebble to enable
// exporting metrics with below buckets in CRDB.
var (
	IOBuckets           = prometheus.ExponentialBucketsRange(float64(time.Millisecond*1), float64(10*time.Second), 50)
	ChannelWriteBuckets = prometheus.ExponentialBucketsRange(float64(time.Microsecond*1), float64(10*time.Second), 50)
)

// Cache is a persistent cache backed by a local filesystem. It is intended
// to cache data that is in slower shared storage (e.g. S3), hence the
// package name 'sharedcache'.
type Cache struct {
	shards       []shard
	writeWorkers writeWorkers

	bm                blockMath
	shardingBlockSize int64

	logger  base.Logger
	metrics internalMetrics
}

// Metrics is a struct containing metrics exported by the secondary cache.
// TODO(josh): Reconsider the set of metrics exported by the secondary cache
// before we release the secondary cache to users. We choose to export many metrics
// right now, so we learn a lot from the benchmarking we are doing over the 23.2
// cycle.
type Metrics struct {
	// The number of sstable bytes stored in the cache.
	Size int64
	// The count of cache blocks in the cache (not sstable blocks).
	Count int64

	// The number of calls to ReadAt.
	TotalReads int64
	// The number of calls to ReadAt that require reading data from 2+ shards.
	MultiShardReads int64
	// The number of calls to ReadAt that require reading data from 2+ cache blocks.
	MultiBlockReads int64
	// The number of calls to ReadAt where all data returned was read from the cache.
	ReadsWithFullHit int64
	// The number of calls to ReadAt where some data returned was read from the cache.
	ReadsWithPartialHit int64
	// The number of calls to ReadAt where no data returned was read from the cache.
	ReadsWithNoHit int64

	// The number of times a cache block was evicted from the cache.
	Evictions int64
	// The number of times writing a cache block to the cache failed.
	WriteBackFailures int64

	// The latency of calls to get some data from the cache.
	GetLatency prometheus.Histogram
	// The latency of reads of a single cache block from disk.
	DiskReadLatency prometheus.Histogram
	// The latency of writing data to write back to the cache to a channel.
	// Generally should be low, but if the channel is full, could be high.
	QueuePutLatency prometheus.Histogram
	// The latency of calls to put some data read from block storage into the cache.
	PutLatency prometheus.Histogram
	// The latency of writes of a single cache block to disk.
	DiskWriteLatency prometheus.Histogram
}

// See docs at Metrics.
type internalMetrics struct {
	count atomic.Int64

	totalReads          atomic.Int64
	multiShardReads     atomic.Int64
	multiBlockReads     atomic.Int64
	readsWithFullHit    atomic.Int64
	readsWithPartialHit atomic.Int64
	readsWithNoHit      atomic.Int64

	evictions         atomic.Int64
	writeBackFailures atomic.Int64

	getLatency       prometheus.Histogram
	diskReadLatency  prometheus.Histogram
	queuePutLatency  prometheus.Histogram
	putLatency       prometheus.Histogram
	diskWriteLatency prometheus.Histogram
}

const (
	// writeWorkersPerShard is used to establish the number of worker goroutines
	// that perform writes to the cache.
	writeWorkersPerShard = 4
	// writeTaskPerWorker is used to establish how many tasks can be queued up
	// until we have to block.
	writeTasksPerWorker = 4
)

// Open opens a cache. If there is no existing cache at fsDir, a new one
// is created.
func Open(
	fs vfs.FS,
	logger base.Logger,
	fsDir string,
	blockSize int,
	// shardingBlockSize is the size of a shard block. The cache is split into contiguous
	// shardingBlockSize units. The units are distributed across multiple independent shards
	// of the cache, via a hash(offset) modulo num shards operation. The cache replacement
	// policies operate at the level of shard, not whole cache. This is done to reduce lock
	// contention.
	shardingBlockSize int64,
	sizeBytes int64,
	numShards int,
) (*Cache, error) {
	if minSize := shardingBlockSize * int64(numShards); sizeBytes < minSize {
		// Up the size so that we have one block per shard. In practice, this should
		// only happen in tests.
		sizeBytes = minSize
	}

	c := &Cache{
		logger:            logger,
		bm:                makeBlockMath(blockSize),
		shardingBlockSize: shardingBlockSize,
	}
	c.shards = make([]shard, numShards)
	blocksPerShard := sizeBytes / int64(numShards) / int64(blockSize)
	for i := range c.shards {
		if err := c.shards[i].init(c, fs, fsDir, i, blocksPerShard, blockSize, shardingBlockSize); err != nil {
			return nil, err
		}
	}

	c.writeWorkers.Start(c, numShards*writeWorkersPerShard)

	c.metrics.getLatency = prometheus.NewHistogram(prometheus.HistogramOpts{Buckets: IOBuckets})
	c.metrics.diskReadLatency = prometheus.NewHistogram(prometheus.HistogramOpts{Buckets: IOBuckets})
	c.metrics.putLatency = prometheus.NewHistogram(prometheus.HistogramOpts{Buckets: IOBuckets})
	c.metrics.diskWriteLatency = prometheus.NewHistogram(prometheus.HistogramOpts{Buckets: IOBuckets})

	// Measures a channel write, so lower min.
	c.metrics.queuePutLatency = prometheus.NewHistogram(prometheus.HistogramOpts{Buckets: ChannelWriteBuckets})

	return c, nil
}

// Close closes the cache. Methods such as ReadAt should not be called after Close is
// called.
func (c *Cache) Close() error {
	c.writeWorkers.Stop()

	var retErr error
	for i := range c.shards {
		if err := c.shards[i].close(); err != nil && retErr == nil {
			retErr = err
		}
	}
	c.shards = nil
	return retErr
}

// Metrics return metrics for the cache. Callers should not mutate
// the returned histograms, which are pointer types.
func (c *Cache) Metrics() Metrics {
	return Metrics{
		Count:               c.metrics.count.Load(),
		Size:                c.metrics.count.Load() * int64(c.bm.BlockSize()),
		TotalReads:          c.metrics.totalReads.Load(),
		MultiShardReads:     c.metrics.multiShardReads.Load(),
		MultiBlockReads:     c.metrics.multiBlockReads.Load(),
		ReadsWithFullHit:    c.metrics.readsWithFullHit.Load(),
		ReadsWithPartialHit: c.metrics.readsWithPartialHit.Load(),
		ReadsWithNoHit:      c.metrics.readsWithNoHit.Load(),
		Evictions:           c.metrics.evictions.Load(),
		WriteBackFailures:   c.metrics.writeBackFailures.Load(),
		GetLatency:          c.metrics.getLatency,
		DiskReadLatency:     c.metrics.diskReadLatency,
		QueuePutLatency:     c.metrics.queuePutLatency,
		PutLatency:          c.metrics.putLatency,
		DiskWriteLatency:    c.metrics.diskWriteLatency,
	}
}

// ReadFlags contains options for Cache.ReadAt.
type ReadFlags struct {
	// ReadOnly instructs ReadAt to not write any new data into the cache; it is
	// used when the data is unlikely to be used again.
	ReadOnly bool
}

// ReadAt performs a read form an object, attempting to use cached data when
// possible.
func (c *Cache) ReadAt(
	ctx context.Context,
	fileNum base.DiskFileNum,
	p []byte,
	ofs int64,
	objReader remote.ObjectReader,
	objSize int64,
	flags ReadFlags,
) error {
	c.metrics.totalReads.Add(1)
	if ofs >= objSize {
		if invariants.Enabled {
			panic(fmt.Sprintf("invalid ReadAt offset %v %v", ofs, objSize))
		}
		return io.EOF
	}
	// TODO(radu): for compaction reads, we may not want to read from the cache at
	// all.
	{
		start := time.Now()
		n, err := c.get(fileNum, p, ofs)
		c.metrics.getLatency.Observe(float64(time.Since(start)))
		if err != nil {
			return err
		}
		if n == len(p) {
			// Everything was in cache!
			c.metrics.readsWithFullHit.Add(1)
			return nil
		}
		if n == 0 {
			c.metrics.readsWithNoHit.Add(1)
		} else {
			c.metrics.readsWithPartialHit.Add(1)
		}

		// Note this. The below code does not need the original ofs, as with the earlier
		// reading from the cache done, the relevant offset is ofs + int64(n). Same with p.
		ofs += int64(n)
		p = p[n:]

		if invariants.Enabled {
			if n != 0 && c.bm.Remainder(ofs) != 0 {
				panic(fmt.Sprintf("after non-zero read from cache, ofs is not block-aligned: %v %v", ofs, n))
			}
		}
	}

	if flags.ReadOnly {
		return objReader.ReadAt(ctx, p, ofs)
	}

	// We must do reads with offset & size that are multiples of the block size. Else
	// later cache hits may return incorrect zeroed results from the cache.
	firstBlockInd := c.bm.Block(ofs)
	adjustedOfs := c.bm.BlockOffset(firstBlockInd)

	// Take the length of what is left to read plus the length of the adjustment of
	// the offset plus the size of a block minus one and divide by the size of a block
	// to get the number of blocks to read from the object.
	sizeOfOffAdjustment := int(ofs - adjustedOfs)
	adjustedLen := int(c.bm.RoundUp(int64(len(p) + sizeOfOffAdjustment)))
	adjustedP := make([]byte, adjustedLen)

	// Read the rest from the object. We may need to cap the length to avoid past EOF reads.
	eofCap := int64(adjustedLen)
	if adjustedOfs+eofCap > objSize {
		eofCap = objSize - adjustedOfs
	}
	if err := objReader.ReadAt(ctx, adjustedP[:eofCap], adjustedOfs); err != nil {
		return err
	}
	copy(p, adjustedP[sizeOfOffAdjustment:])

	start := time.Now()
	c.writeWorkers.QueueWrite(fileNum, adjustedP, adjustedOfs)
	c.metrics.queuePutLatency.Observe(float64(time.Since(start)))

	return nil
}

// get attempts to read the requested data from the cache, if it is already
// there.
//
// If all data is available, returns n = len(p).
//
// If data is partially available, a prefix of the data is read; returns n < len(p)
// and no error. If no prefix is available, returns n = 0 and no error.
func (c *Cache) get(fileNum base.DiskFileNum, p []byte, ofs int64) (n int, _ error) {
	// The data extent might cross shard boundaries, hence the loop. In the hot
	// path, max two iterations of this loop will be executed, since reads are sized
	// in units of sstable block size.
	var multiShard bool
	for {
		shard := c.getShard(fileNum, ofs+int64(n))
		cappedLen := len(p[n:])
		if toBoundary := int(c.shardingBlockSize - ((ofs + int64(n)) % c.shardingBlockSize)); cappedLen > toBoundary {
			cappedLen = toBoundary
		}
		numRead, err := shard.get(fileNum, p[n:n+cappedLen], ofs+int64(n))
		if err != nil {
			return n, err
		}
		n += numRead
		if numRead < cappedLen {
			// We only read a prefix from this shard.
			return n, nil
		}
		if n == len(p) {
			// We are done.
			return n, nil
		}
		// Data extent crosses shard boundary, continue with next shard.
		if !multiShard {
			c.metrics.multiShardReads.Add(1)
			multiShard = true
		}
	}
}

// set attempts to write the requested data to the cache. Both ofs & len(p) must
// be multiples of the block size.
//
// If all of p is not written to the shard, set returns a non-nil error.
func (c *Cache) set(fileNum base.DiskFileNum, p []byte, ofs int64) error {
	if invariants.Enabled {
		if c.bm.Remainder(ofs) != 0 || c.bm.Remainder(int64(len(p))) != 0 {
			panic(fmt.Sprintf("set with ofs & len not multiples of block size: %v %v", ofs, len(p)))
		}
	}

	// The data extent might cross shard boundaries, hence the loop. In the hot
	// path, max two iterations of this loop will be executed, since reads are sized
	// in units of sstable block size.
	n := 0
	for {
		shard := c.getShard(fileNum, ofs+int64(n))
		cappedLen := len(p[n:])
		if toBoundary := int(c.shardingBlockSize - ((ofs + int64(n)) % c.shardingBlockSize)); cappedLen > toBoundary {
			cappedLen = toBoundary
		}
		err := shard.set(fileNum, p[n:n+cappedLen], ofs+int64(n))
		if err != nil {
			return err
		}
		// set returns an error if cappedLen bytes aren't written to the shard.
		n += cappedLen
		if n == len(p) {
			// We are done.
			return nil
		}
		// Data extent crosses shard boundary, continue with next shard.
	}
}

func (c *Cache) getShard(fileNum base.DiskFileNum, ofs int64) *shard {
	const prime64 = 1099511628211
	hash := uint64(fileNum.FileNum())*prime64 + uint64(ofs/c.shardingBlockSize)
	// TODO(josh): Instance change ops are often run in production. Such an operation
	// updates len(c.shards); see openSharedCache. As a result, the behavior of this
	// function changes, and the cache empties out at restart time. We may want a better
	// story here eventually.
	return &c.shards[hash%uint64(len(c.shards))]
}

type shard struct {
	cache             *Cache
	file              vfs.File
	sizeInBlocks      int64
	bm                blockMath
	shardingBlockSize int64
	mu                struct {
		sync.Mutex
		// TODO(josh): None of these datastructures are space-efficient.
		// Focusing on correctness to start.
		where  whereMap
		blocks []cacheBlockState
		// Head of LRU list (doubly-linked circular).
		lruHead cacheBlockIndex
		// Head of free list (singly-linked chain).
		freeHead cacheBlockIndex
	}
}

type cacheBlockState struct {
	lock    lockState
	logical logicalBlockID

	// next is the next block in the LRU or free list (or invalidBlockIndex if it
	// is the last block in the free list).
	next cacheBlockIndex

	// prev is the previous block in the LRU list. It is not used when the block
	// is in the free list.
	prev cacheBlockIndex
}

// Maps a logical block in an SST to an index of the cache block with the
// file contents (to the "cache block index").
type whereMap map[logicalBlockID]cacheBlockIndex

type logicalBlockID struct {
	filenum       base.DiskFileNum
	cacheBlockIdx cacheBlockIndex
}

type lockState int64

const (
	unlocked lockState = 0
	// >0 lockState tracks the number of distinct readers of some cache block / logical block
	// which is in the secondary cache. It is used to ensure that a cache block is not evicted
	// and overwritten, while there are active readers.
	readLockTakenInc = 1
	// -1 lockState indicates that some cache block is currently being populated with data from
	// blob storage. It is used to ensure that a cache block is not read or evicted again, while
	// it is being populated.
	writeLockTaken = -1
)

func (s *shard) init(
	cache *Cache,
	fs vfs.FS,
	fsDir string,
	shardIdx int,
	sizeInBlocks int64,
	blockSize int,
	shardingBlockSize int64,
) error {
	*s = shard{
		cache:        cache,
		sizeInBlocks: sizeInBlocks,
	}
	if blockSize < 1024 || shardingBlockSize%int64(blockSize) != 0 {
		return errors.Newf("invalid block size %d (must divide %d)", blockSize, shardingBlockSize)
	}
	s.bm = makeBlockMath(blockSize)
	s.shardingBlockSize = shardingBlockSize
	file, err := fs.OpenReadWrite(fs.PathJoin(fsDir, fmt.Sprintf("SHARED-CACHE-%03d", shardIdx)))
	if err != nil {
		return err
	}
	// TODO(radu): truncate file if necessary (especially important if we restart
	// with more shards).
	if err := file.Preallocate(0, int64(blockSize)*sizeInBlocks); err != nil {
		return err
	}
	s.file = file

	// TODO(josh): Right now, the secondary cache is not persistent. All existing
	// cache contents will be over-written, since all metadata is only stored in
	// memory.
	s.mu.where = make(whereMap)
	s.mu.blocks = make([]cacheBlockState, sizeInBlocks)
	s.mu.lruHead = invalidBlockIndex
	s.mu.freeHead = invalidBlockIndex
	for i := range s.mu.blocks {
		s.freePush(cacheBlockIndex(i))
	}

	return nil
}

func (s *shard) close() error {
	defer func() {
		s.file = nil
	}()
	return s.file.Close()
}

// freePush pushes a block to the front of the free list.
func (s *shard) freePush(index cacheBlockIndex) {
	s.mu.blocks[index].next = s.mu.freeHead
	s.mu.freeHead = index
}

// freePop removes the block from the front of the free list. Must not be called
// if the list is empty (i.e. freeHead = invalidBlockIndex).
func (s *shard) freePop() cacheBlockIndex {
	index := s.mu.freeHead
	s.mu.freeHead = s.mu.blocks[index].next
	return index
}

// lruInsertFront inserts a block at the front of the LRU list.
func (s *shard) lruInsertFront(index cacheBlockIndex) {
	b := &s.mu.blocks[index]
	if s.mu.lruHead == invalidBlockIndex {
		b.next = index
		b.prev = index
	} else {
		b.next = s.mu.lruHead
		h := &s.mu.blocks[s.mu.lruHead]
		b.prev = h.prev
		s.mu.blocks[h.prev].next = index
		h.prev = index
	}
	s.mu.lruHead = index
}

func (s *shard) lruNext(index cacheBlockIndex) cacheBlockIndex {
	return s.mu.blocks[index].next
}

func (s *shard) lruPrev(index cacheBlockIndex) cacheBlockIndex {
	return s.mu.blocks[index].prev
}

// lruUnlink removes a block from the LRU list.
func (s *shard) lruUnlink(index cacheBlockIndex) {
	b := &s.mu.blocks[index]
	if b.next == index {
		s.mu.lruHead = invalidBlockIndex
	} else {
		s.mu.blocks[b.prev].next = b.next
		s.mu.blocks[b.next].prev = b.prev
		if s.mu.lruHead == index {
			s.mu.lruHead = b.next
		}
	}
	b.next, b.prev = invalidBlockIndex, invalidBlockIndex
}

// get attempts to read the requested data from the shard. The data must not
// cross a shard boundary.
//
// If all data is available, returns n = len(p).
//
// If data is partially available, a prefix of the data is read; returns n < len(p)
// and no error. If no prefix is available, returns n = 0 and no error.
//
// TODO(josh): Today, if there are two cache blocks needed to satisfy a read, and the
// first block is not in the cache and the second one is, we will read both from
// blob storage. We should fix this. This is not an unlikely scenario if we are doing
// a reverse scan, since those iterate over sstable blocks in reverse order and due to
// cache block aligned reads will have read the suffix of the sstable block that will
// be needed next.
func (s *shard) get(fileNum base.DiskFileNum, p []byte, ofs int64) (n int, _ error) {
	if invariants.Enabled {
		if ofs/s.shardingBlockSize != (ofs+int64(len(p))-1)/s.shardingBlockSize {
			panic(fmt.Sprintf("get crosses shard boundary: %v %v", ofs, len(p)))
		}
		s.assertShardStateIsConsistent()
	}

	// The data extent might cross cache block boundaries, hence the loop. In the hot
	// path, max two iterations of this loop will be executed, since reads are sized
	// in units of sstable block size.
	var multiBlock bool
	for {
		k := logicalBlockID{
			filenum:       fileNum,
			cacheBlockIdx: s.bm.Block(ofs + int64(n)),
		}
		s.mu.Lock()
		cacheBlockIdx, ok := s.mu.where[k]
		// TODO(josh): Multiple reads within the same few milliseconds (anything that is smaller
		// than blob storage read latency) that miss on the same logical block ID will not necessarily
		// be rare. We may want to do only one read, with the later readers blocking on the first read
		// completing. This could be implemented either here or in the primary block cache. See
		// https://github.com/cockroachdb/pebble/pull/2586 for additional discussion.
		if !ok {
			s.mu.Unlock()
			return n, nil
		}
		if s.mu.blocks[cacheBlockIdx].lock == writeLockTaken {
			// In practice, if we have two reads of the same SST block in close succession, we
			// would expect the second to hit in the in-memory block cache. So it's not worth
			// optimizing this case here.
			s.mu.Unlock()
			return n, nil
		}
		s.mu.blocks[cacheBlockIdx].lock += readLockTakenInc
		// Move to front of the LRU list.
		s.lruUnlink(cacheBlockIdx)
		s.lruInsertFront(cacheBlockIdx)
		s.mu.Unlock()

		readAt := s.bm.BlockOffset(cacheBlockIdx)
		readSize := s.bm.BlockSize()
		if n == 0 { // if first read
			rem := s.bm.Remainder(ofs)
			readAt += rem
			readSize -= int(rem)
		}

		if len(p[n:]) <= readSize {
			start := time.Now()
			numRead, err := s.file.ReadAt(p[n:], readAt)
			s.cache.metrics.diskReadLatency.Observe(float64(time.Since(start)))
			s.dropReadLock(cacheBlockIdx)
			return n + numRead, err
		}
		start := time.Now()
		numRead, err := s.file.ReadAt(p[n:n+readSize], readAt)
		s.cache.metrics.diskReadLatency.Observe(float64(time.Since(start)))
		s.dropReadLock(cacheBlockIdx)
		if err != nil {
			return 0, err
		}

		// Note that numRead == readSize, since we checked for an error above.
		n += numRead

		if !multiBlock {
			s.cache.metrics.multiBlockReads.Add(1)
			multiBlock = true
		}
	}
}

// set attempts to write the requested data to the shard. The data must not
// cross a shard boundary, and both ofs & len(p) must be multiples of the
// block size.
//
// If all of p is not written to the shard, set returns a non-nil error.
func (s *shard) set(fileNum base.DiskFileNum, p []byte, ofs int64) error {
	if invariants.Enabled {
		if ofs/s.shardingBlockSize != (ofs+int64(len(p))-1)/s.shardingBlockSize {
			panic(fmt.Sprintf("set crosses shard boundary: %v %v", ofs, len(p)))
		}
		if s.bm.Remainder(ofs) != 0 || s.bm.Remainder(int64(len(p))) != 0 {
			panic(fmt.Sprintf("set with ofs & len not multiples of block size: %v %v", ofs, len(p)))
		}
		s.assertShardStateIsConsistent()
	}

	// The data extent might cross cache block boundaries, hence the loop. In the hot
	// path, max two iterations of this loop will be executed, since reads are sized
	// in units of sstable block size.
	n := 0
	for {
		if n == len(p) {
			return nil
		}
		if invariants.Enabled {
			if n > len(p) {
				panic(fmt.Sprintf("set with n greater than len(p): %v %v", n, len(p)))
			}
		}

		// If the logical block is already in the cache, we should skip doing a set.
		k := logicalBlockID{
			filenum:       fileNum,
			cacheBlockIdx: s.bm.Block(ofs + int64(n)),
		}
		s.mu.Lock()
		if _, ok := s.mu.where[k]; ok {
			s.mu.Unlock()
			n += s.bm.BlockSize()
			continue
		}

		var cacheBlockIdx cacheBlockIndex
		if s.mu.freeHead == invalidBlockIndex {
			if invariants.Enabled && s.mu.lruHead == invalidBlockIndex {
				panic("both LRU and free lists empty")
			}

			// Find the last element in the LRU list which is not locked.
			for idx := s.lruPrev(s.mu.lruHead); ; idx = s.lruPrev(idx) {
				if lock := s.mu.blocks[idx].lock; lock == unlocked {
					cacheBlockIdx = idx
					break
				}
				if idx == s.mu.lruHead {
					// No unlocked block to evict.
					//
					// TODO(josh): We may want to block until a block frees up, instead of returning
					// an error here. But I think we can do that later on, e.g. after running some production
					// experiments.
					s.mu.Unlock()
					return errors.New("no block to evict so skipping write to cache")
				}
			}
			s.cache.metrics.evictions.Add(1)
			s.lruUnlink(cacheBlockIdx)
			delete(s.mu.where, s.mu.blocks[cacheBlockIdx].logical)
		} else {
			s.cache.metrics.count.Add(1)
			cacheBlockIdx = s.freePop()
		}

		s.lruInsertFront(cacheBlockIdx)
		s.mu.where[k] = cacheBlockIdx
		s.mu.blocks[cacheBlockIdx].logical = k
		s.mu.blocks[cacheBlockIdx].lock = writeLockTaken
		s.mu.Unlock()

		writeAt := s.bm.BlockOffset(cacheBlockIdx)

		writeSize := s.bm.BlockSize()
		if len(p[n:]) <= writeSize {
			writeSize = len(p[n:])
		}

		start := time.Now()
		_, err := s.file.WriteAt(p[n:n+writeSize], writeAt)
		s.cache.metrics.diskWriteLatency.Observe(float64(time.Since(start)))
		if err != nil {
			// Free the block.
			s.mu.Lock()
			defer s.mu.Unlock()

			delete(s.mu.where, k)
			s.lruUnlink(cacheBlockIdx)
			s.freePush(cacheBlockIdx)
			return err
		}
		s.dropWriteLock(cacheBlockIdx)
		n += writeSize
	}
}

// Doesn't inline currently. This might be okay, but something to keep in mind.
func (s *shard) dropReadLock(cacheBlockInd cacheBlockIndex) {
	s.mu.Lock()
	s.mu.blocks[cacheBlockInd].lock -= readLockTakenInc
	if invariants.Enabled && s.mu.blocks[cacheBlockInd].lock < 0 {
		panic(fmt.Sprintf("unexpected lock state %v in dropReadLock", s.mu.blocks[cacheBlockInd].lock))
	}
	s.mu.Unlock()
}

// Doesn't inline currently. This might be okay, but something to keep in mind.
func (s *shard) dropWriteLock(cacheBlockInd cacheBlockIndex) {
	s.mu.Lock()
	if invariants.Enabled && s.mu.blocks[cacheBlockInd].lock != writeLockTaken {
		panic(fmt.Sprintf("unexpected lock state %v in dropWriteLock", s.mu.blocks[cacheBlockInd].lock))
	}
	s.mu.blocks[cacheBlockInd].lock = unlocked
	s.mu.Unlock()
}

func (s *shard) assertShardStateIsConsistent() {
	s.mu.Lock()
	defer s.mu.Unlock()

	lruLen := 0
	if s.mu.lruHead != invalidBlockIndex {
		for b := s.mu.lruHead; ; {
			lruLen++
			if idx, ok := s.mu.where[s.mu.blocks[b].logical]; !ok || idx != b {
				panic("block in LRU list with no entry in where map")
			}
			b = s.lruNext(b)
			if b == s.mu.lruHead {
				break
			}
		}
	}
	if lruLen != len(s.mu.where) {
		panic(fmt.Sprintf("lru list len is %d but where map has %d entries", lruLen, len(s.mu.where)))
	}
	freeLen := 0
	for n := s.mu.freeHead; n != invalidBlockIndex; n = s.mu.blocks[n].next {
		freeLen++
	}

	if lruLen+freeLen != int(s.sizeInBlocks) {
		panic(fmt.Sprintf("%d lru blocks and %d free blocks don't add up to %d", lruLen, freeLen, s.sizeInBlocks))
	}
	for i := range s.mu.blocks {
		if state := s.mu.blocks[i].lock; state < writeLockTaken {
			panic(fmt.Sprintf("lock state %v is not allowed", state))
		}
	}
}

// cacheBlockIndex is the index of a blockSize-aligned cache block.
type cacheBlockIndex int64

// invalidBlockIndex is used for the head of a list when the list is empty.
const invalidBlockIndex cacheBlockIndex = -1

// blockMath is a helper type for performing conversions between offsets and
// block indexes.
type blockMath struct {
	blockSizeBits int8
}

func makeBlockMath(blockSize int) blockMath {
	bm := blockMath{
		blockSizeBits: int8(bits.Len64(uint64(blockSize)) - 1),
	}
	if blockSize != (1 << bm.blockSizeBits) {
		panic(fmt.Sprintf("blockSize %d is not a power of 2", blockSize))
	}
	return bm
}

func (bm blockMath) mask() int64 {
	return (1 << bm.blockSizeBits) - 1
}

// BlockSize returns the block size.
func (bm blockMath) BlockSize() int {
	return 1 << bm.blockSizeBits
}

// Block returns the block index containing the given offset.
func (bm blockMath) Block(offset int64) cacheBlockIndex {
	return cacheBlockIndex(offset >> bm.blockSizeBits)
}

// Remainder returns the offset relative to the start of the cache block.
func (bm blockMath) Remainder(offset int64) int64 {
	return offset & bm.mask()
}

// BlockOffset returns the object offset where the given block starts.
func (bm blockMath) BlockOffset(block cacheBlockIndex) int64 {
	return int64(block) << bm.blockSizeBits
}

// RoundUp rounds up the given value to the closest multiple of block size.
func (bm blockMath) RoundUp(x int64) int64 {
	return (x + bm.mask()) & ^(bm.mask())
}

type writeWorkers struct {
	doneCh        chan struct{}
	doneWaitGroup sync.WaitGroup

	numWorkers int
	tasksCh    chan writeTask
}

type writeTask struct {
	fileNum base.DiskFileNum
	p       []byte
	offset  int64
}

// Start starts the worker goroutines.
func (w *writeWorkers) Start(c *Cache, numWorkers int) {
	doneCh := make(chan struct{})
	tasksCh := make(chan writeTask, numWorkers*writeTasksPerWorker)

	w.numWorkers = numWorkers
	w.doneCh = doneCh
	w.tasksCh = tasksCh
	w.doneWaitGroup.Add(numWorkers)
	for i := 0; i < numWorkers; i++ {
		go func() {
			defer w.doneWaitGroup.Done()
			for {
				select {
				case <-doneCh:
					return
				case task, ok := <-tasksCh:
					if !ok {
						// The tasks channel was closed; this is used in testing code to
						// ensure all writes are completed.
						return
					}
					// TODO(radu): set() can perform multiple writes; perhaps each one
					// should be its own task.
					start := time.Now()
					err := c.set(task.fileNum, task.p, task.offset)
					c.metrics.putLatency.Observe(float64(time.Since(start)))
					if err != nil {
						c.metrics.writeBackFailures.Add(1)
						// TODO(radu): throttle logs.
						c.logger.Infof("writing back to cache after miss failed: %v", err)
					}
				}
			}
		}()
	}
}

// Stop waits for any in-progress writes to complete and stops the worker
// goroutines and waits for any in-pro. Any queued writes not yet started are
// discarded.
func (w *writeWorkers) Stop() {
	close(w.doneCh)
	w.doneCh = nil
	w.tasksCh = nil
	w.doneWaitGroup.Wait()
}

// QueueWrite adds a write task to the queue. Can block if the queue is full.
func (w *writeWorkers) QueueWrite(fileNum base.DiskFileNum, p []byte, offset int64) {
	w.tasksCh <- writeTask{
		fileNum: fileNum,
		p:       p,
		offset:  offset,
	}
}
