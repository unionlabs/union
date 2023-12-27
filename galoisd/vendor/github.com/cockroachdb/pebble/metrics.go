// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"fmt"
	"math"
	"time"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/sharedcache"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/redact"
	"github.com/prometheus/client_golang/prometheus"
)

// CacheMetrics holds metrics for the block and table cache.
type CacheMetrics = cache.Metrics

// FilterMetrics holds metrics for the filter policy
type FilterMetrics = sstable.FilterMetrics

// ThroughputMetric is a cumulative throughput metric. See the detailed
// comment in base.
type ThroughputMetric = base.ThroughputMetric

// SecondaryCacheMetrics holds metrics for the persistent secondary cache
// that caches commonly accessed blocks from blob storage on a local
// file system.
type SecondaryCacheMetrics = sharedcache.Metrics

// LevelMetrics holds per-level metrics such as the number of files and total
// size of the files, and compaction related metrics.
type LevelMetrics struct {
	// The number of sublevels within the level. The sublevel count corresponds
	// to the read amplification for the level. An empty level will have a
	// sublevel count of 0, implying no read amplification. Only L0 will have
	// a sublevel count other than 0 or 1.
	Sublevels int32
	// The total number of files in the level.
	NumFiles int64
	// The total number of virtual sstables in the level.
	NumVirtualFiles uint64
	// The total size in bytes of the files in the level.
	Size int64
	// The total size of the virtual sstables in the level.
	VirtualSize uint64
	// The level's compaction score. This is the compensatedScoreRatio in the
	// candidateLevelInfo.
	Score float64
	// The number of incoming bytes from other levels read during
	// compactions. This excludes bytes moved and bytes ingested. For L0 this is
	// the bytes written to the WAL.
	BytesIn uint64
	// The number of bytes ingested. The sibling metric for tables is
	// TablesIngested.
	BytesIngested uint64
	// The number of bytes moved into the level by a "move" compaction. The
	// sibling metric for tables is TablesMoved.
	BytesMoved uint64
	// The number of bytes read for compactions at the level. This includes bytes
	// read from other levels (BytesIn), as well as bytes read for the level.
	BytesRead uint64
	// The number of bytes written during compactions. The sibling
	// metric for tables is TablesCompacted. This metric may be summed
	// with BytesFlushed to compute the total bytes written for the level.
	BytesCompacted uint64
	// The number of bytes written during flushes. The sibling
	// metrics for tables is TablesFlushed. This metric is always
	// zero for all levels other than L0.
	BytesFlushed uint64
	// The number of sstables compacted to this level.
	TablesCompacted uint64
	// The number of sstables flushed to this level.
	TablesFlushed uint64
	// The number of sstables ingested into the level.
	TablesIngested uint64
	// The number of sstables moved to this level by a "move" compaction.
	TablesMoved uint64

	MultiLevel struct {
		// BytesInTop are the total bytes in a multilevel compaction coming from the top level.
		BytesInTop uint64

		// BytesIn, exclusively for multiLevel compactions.
		BytesIn uint64

		// BytesRead, exclusively for multilevel compactions.
		BytesRead uint64
	}

	// Additional contains misc additional metrics that are not always printed.
	Additional struct {
		// The sum of Properties.ValueBlocksSize for all the sstables in this
		// level. Printed by LevelMetrics.format iff there is at least one level
		// with a non-zero value.
		ValueBlocksSize uint64
		// Cumulative metrics about bytes written to data blocks and value blocks,
		// via compactions (except move compactions) or flushes. Not printed by
		// LevelMetrics.format, but are available to sophisticated clients.
		BytesWrittenDataBlocks  uint64
		BytesWrittenValueBlocks uint64
	}
}

// Add updates the counter metrics for the level.
func (m *LevelMetrics) Add(u *LevelMetrics) {
	m.NumFiles += u.NumFiles
	m.NumVirtualFiles += u.NumVirtualFiles
	m.VirtualSize += u.VirtualSize
	m.Size += u.Size
	m.BytesIn += u.BytesIn
	m.BytesIngested += u.BytesIngested
	m.BytesMoved += u.BytesMoved
	m.BytesRead += u.BytesRead
	m.BytesCompacted += u.BytesCompacted
	m.BytesFlushed += u.BytesFlushed
	m.TablesCompacted += u.TablesCompacted
	m.TablesFlushed += u.TablesFlushed
	m.TablesIngested += u.TablesIngested
	m.TablesMoved += u.TablesMoved
	m.MultiLevel.BytesInTop += u.MultiLevel.BytesInTop
	m.MultiLevel.BytesRead += u.MultiLevel.BytesRead
	m.MultiLevel.BytesIn += u.MultiLevel.BytesIn
	m.Additional.BytesWrittenDataBlocks += u.Additional.BytesWrittenDataBlocks
	m.Additional.BytesWrittenValueBlocks += u.Additional.BytesWrittenValueBlocks
	m.Additional.ValueBlocksSize += u.Additional.ValueBlocksSize
}

// WriteAmp computes the write amplification for compactions at this
// level. Computed as (BytesFlushed + BytesCompacted) / BytesIn.
func (m *LevelMetrics) WriteAmp() float64 {
	if m.BytesIn == 0 {
		return 0
	}
	return float64(m.BytesFlushed+m.BytesCompacted) / float64(m.BytesIn)
}

// Metrics holds metrics for various subsystems of the DB such as the Cache,
// Compactions, WAL, and per-Level metrics.
//
// TODO(peter): The testing of these metrics is relatively weak. There should
// be testing that performs various operations on a DB and verifies that the
// metrics reflect those operations.
type Metrics struct {
	BlockCache CacheMetrics

	Compact struct {
		// The total number of compactions, and per-compaction type counts.
		Count             int64
		DefaultCount      int64
		DeleteOnlyCount   int64
		ElisionOnlyCount  int64
		MoveCount         int64
		ReadCount         int64
		RewriteCount      int64
		MultiLevelCount   int64
		CounterLevelCount int64
		// An estimate of the number of bytes that need to be compacted for the LSM
		// to reach a stable state.
		EstimatedDebt uint64
		// Number of bytes present in sstables being written by in-progress
		// compactions. This value will be zero if there are no in-progress
		// compactions.
		InProgressBytes int64
		// Number of compactions that are in-progress.
		NumInProgress int64
		// MarkedFiles is a count of files that are marked for
		// compaction. Such files are compacted in a rewrite compaction
		// when no other compactions are picked.
		MarkedFiles int
		// Duration records the cumulative duration of all compactions since the
		// database was opened.
		Duration time.Duration
	}

	Ingest struct {
		// The total number of ingestions
		Count uint64
	}

	Flush struct {
		// The total number of flushes.
		Count           int64
		WriteThroughput ThroughputMetric
		// Number of flushes that are in-progress. In the current implementation
		// this will always be zero or one.
		NumInProgress int64
		// AsIngestCount is a monotonically increasing counter of flush operations
		// handling ingested tables.
		AsIngestCount uint64
		// AsIngestCount is a monotonically increasing counter of tables ingested as
		// flushables.
		AsIngestTableCount uint64
		// AsIngestBytes is a monotonically increasing counter of the bytes flushed
		// for flushables that originated as ingestion operations.
		AsIngestBytes uint64
	}

	Filter FilterMetrics

	Levels [numLevels]LevelMetrics

	MemTable struct {
		// The number of bytes allocated by memtables and large (flushable)
		// batches.
		Size uint64
		// The count of memtables.
		Count int64
		// The number of bytes present in zombie memtables which are no longer
		// referenced by the current DB state. An unbounded number of memtables
		// may be zombie if they're still in use by an iterator. One additional
		// memtable may be zombie if it's no longer in use and waiting to be
		// recycled.
		ZombieSize uint64
		// The count of zombie memtables.
		ZombieCount int64
	}

	Keys struct {
		// The approximate count of internal range key set keys in the database.
		RangeKeySetsCount uint64
		// The approximate count of internal tombstones (DEL, SINGLEDEL and
		// RANGEDEL key kinds) within the database.
		TombstoneCount uint64
		// A cumulative total number of missized DELSIZED keys encountered by
		// compactions since the database was opened.
		MissizedTombstonesCount uint64
	}

	Snapshots struct {
		// The number of currently open snapshots.
		Count int
		// The sequence number of the earliest, currently open snapshot.
		EarliestSeqNum uint64
		// A running tally of keys written to sstables during flushes or
		// compactions that would've been elided if it weren't for open
		// snapshots.
		PinnedKeys uint64
		// A running cumulative sum of the size of keys and values written to
		// sstables during flushes or compactions that would've been elided if
		// it weren't for open snapshots.
		PinnedSize uint64
	}

	Table struct {
		// The number of bytes present in obsolete tables which are no longer
		// referenced by the current DB state or any open iterators.
		ObsoleteSize uint64
		// The count of obsolete tables.
		ObsoleteCount int64
		// The number of bytes present in zombie tables which are no longer
		// referenced by the current DB state but are still in use by an iterator.
		ZombieSize uint64
		// The count of zombie tables.
		ZombieCount int64
		// The count of the backing sstables.
		BackingTableCount uint64
		// The sum of the sizes of the all of the backing sstables.
		BackingTableSize uint64
	}

	TableCache CacheMetrics

	// Count of the number of open sstable iterators.
	TableIters int64
	// Uptime is the total time since this DB was opened.
	Uptime time.Duration

	WAL struct {
		// Number of live WAL files.
		Files int64
		// Number of obsolete WAL files.
		ObsoleteFiles int64
		// Physical size of the obsolete WAL files.
		ObsoletePhysicalSize uint64
		// Size of the live data in the WAL files. Note that with WAL file
		// recycling this is less than the actual on-disk size of the WAL files.
		Size uint64
		// Physical size of the WAL files on-disk. With WAL file recycling,
		// this is greater than the live data in WAL files.
		PhysicalSize uint64
		// Number of logical bytes written to the WAL.
		BytesIn uint64
		// Number of bytes written to the WAL.
		BytesWritten uint64
	}

	LogWriter struct {
		FsyncLatency prometheus.Histogram
		record.LogWriterMetrics
	}

	SecondaryCacheMetrics SecondaryCacheMetrics

	private struct {
		optionsFileSize  uint64
		manifestFileSize uint64
	}
}

var (
	// FsyncLatencyBuckets are prometheus histogram buckets suitable for a histogram
	// that records latencies for fsyncs.
	FsyncLatencyBuckets = append(
		prometheus.LinearBuckets(0.0, float64(time.Microsecond*100), 50),
		prometheus.ExponentialBucketsRange(float64(time.Millisecond*5), float64(10*time.Second), 50)...,
	)

	// SecondaryCacheIOBuckets exported to enable exporting from package pebble to
	// enable exporting metrics with below buckets in CRDB.
	SecondaryCacheIOBuckets = sharedcache.IOBuckets
	// SecondaryCacheChannelWriteBuckets exported to enable exporting from package
	// pebble to enable exporting metrics with below buckets in CRDB.
	SecondaryCacheChannelWriteBuckets = sharedcache.ChannelWriteBuckets
)

// DiskSpaceUsage returns the total disk space used by the database in bytes,
// including live and obsolete files.
func (m *Metrics) DiskSpaceUsage() uint64 {
	var usageBytes uint64
	usageBytes += m.WAL.PhysicalSize
	usageBytes += m.WAL.ObsoletePhysicalSize
	for _, lm := range m.Levels {
		usageBytes += uint64(lm.Size)
	}
	usageBytes += m.Table.ObsoleteSize
	usageBytes += m.Table.ZombieSize
	usageBytes += m.private.optionsFileSize
	usageBytes += m.private.manifestFileSize
	usageBytes += uint64(m.Compact.InProgressBytes)
	return usageBytes
}

// NumVirtual is the number of virtual sstables in the latest version
// summed over every level in the lsm.
func (m *Metrics) NumVirtual() uint64 {
	var n uint64
	for _, level := range m.Levels {
		n += level.NumVirtualFiles
	}
	return n
}

// VirtualSize is the sum of the sizes of the virtual sstables in the
// latest version. BackingTableSize - VirtualSize gives an estimate for
// the space amplification caused by not compacting virtual sstables.
func (m *Metrics) VirtualSize() uint64 {
	var size uint64
	for _, level := range m.Levels {
		size += level.VirtualSize
	}
	return size
}

// ReadAmp returns the current read amplification of the database.
// It's computed as the number of sublevels in L0 + the number of non-empty
// levels below L0.
func (m *Metrics) ReadAmp() int {
	var ramp int32
	for _, l := range m.Levels {
		ramp += l.Sublevels
	}
	return int(ramp)
}

// Total returns the sum of the per-level metrics and WAL metrics.
func (m *Metrics) Total() LevelMetrics {
	var total LevelMetrics
	for level := 0; level < numLevels; level++ {
		l := &m.Levels[level]
		total.Add(l)
		total.Sublevels += l.Sublevels
	}
	// Compute total bytes-in as the bytes written to the WAL + bytes ingested.
	total.BytesIn = m.WAL.BytesWritten + total.BytesIngested
	// Add the total bytes-in to the total bytes-flushed. This is to account for
	// the bytes written to the log and bytes written externally and then
	// ingested.
	total.BytesFlushed += total.BytesIn
	return total
}

// String pretty-prints the metrics as below:
//
//	      |                             |       |       |   ingested   |     moved    |    written   |       |    amp
//	level | tables  size val-bl vtables | score |   in  | tables  size | tables  size | tables  size |  read |   r   w
//	------+-----------------------------+-------+-------+--------------+--------------+--------------+-------+---------
//	    0 |   101   102B     0B       0 | 103.0 |  104B |   112   104B |   113   106B |   221   217B |  107B |   1  2.1
//	    1 |   201   202B     0B       0 | 203.0 |  204B |   212   204B |   213   206B |   421   417B |  207B |   2  2.0
//	    2 |   301   302B     0B       0 | 303.0 |  304B |   312   304B |   313   306B |   621   617B |  307B |   3  2.0
//	    3 |   401   402B     0B       0 | 403.0 |  404B |   412   404B |   413   406B |   821   817B |  407B |   4  2.0
//	    4 |   501   502B     0B       0 | 503.0 |  504B |   512   504B |   513   506B |  1.0K  1017B |  507B |   5  2.0
//	    5 |   601   602B     0B       0 | 603.0 |  604B |   612   604B |   613   606B |  1.2K  1.2KB |  607B |   6  2.0
//	    6 |   701   702B     0B       0 |     - |  704B |   712   704B |   713   706B |  1.4K  1.4KB |  707B |   7  2.0
//	total |  2.8K  2.7KB     0B       0 |     - | 2.8KB |  2.9K  2.8KB |  2.9K  2.8KB |  5.7K  8.4KB | 2.8KB |  28  3.0
//	-------------------------------------------------------------------------------------------------------------------
//	WAL: 22 files (24B)  in: 25B  written: 26B (4% overhead)
//	Flushes: 8
//	Compactions: 5  estimated debt: 6B  in progress: 2 (7B)
//	default: 27  delete: 28  elision: 29  move: 30  read: 31  rewrite: 32  multi-level: 33
//	MemTables: 12 (11B)  zombie: 14 (13B)
//	Zombie tables: 16 (15B)
//	Backing tables: 0 (0B)
//	Block cache: 2 entries (1B)  hit rate: 42.9%
//	Table cache: 18 entries (17B)  hit rate: 48.7%
//	Secondary cache: 40 entries (40B)  hit rate: 49.9%
//	Snapshots: 4  earliest seq num: 1024
//	Table iters: 21
//	Filter utility: 47.4%
//	Ingestions: 27  as flushable: 36 (34B in 35 tables)
func (m *Metrics) String() string {
	return redact.StringWithoutMarkers(m)
}

var _ redact.SafeFormatter = &Metrics{}

// SafeFormat implements redact.SafeFormatter.
func (m *Metrics) SafeFormat(w redact.SafePrinter, _ rune) {
	// NB: Pebble does not make any assumptions as to which Go primitive types
	// have been registered as safe with redact.RegisterSafeType and does not
	// register any types itself. Some of the calls to `redact.Safe`, etc are
	// superfluous in the context of CockroachDB, which registers all the Go
	// numeric types as safe.

	// TODO(jackson): There are a few places where we use redact.SafeValue
	// instead of redact.RedactableString. This is necessary because of a bug
	// whereby formatting a redact.RedactableString argument does not respect
	// width specifiers. When the issue is fixed, we can convert these to
	// RedactableStrings. https://github.com/cockroachdb/redact/issues/17

	multiExists := m.Compact.MultiLevelCount > 0
	appendIfMulti := func(line redact.SafeString) {
		if multiExists {
			w.SafeString(line)
		}
	}
	newline := func() {
		w.SafeString("\n")
	}

	w.SafeString("      |                             |       |       |   ingested   |     moved    |    written   |       |    amp")
	appendIfMulti("   |     multilevel")
	newline()
	w.SafeString("level | tables  size val-bl vtables | score |   in  | tables  size | tables  size | tables  size |  read |   r   w")
	appendIfMulti("  |    top   in  read")
	newline()
	w.SafeString("------+-----------------------------+-------+-------+--------------+--------------+--------------+-------+---------")
	appendIfMulti("-+------------------")
	newline()

	// formatRow prints out a row of the table.
	formatRow := func(m *LevelMetrics, score float64) {
		scoreStr := "-"
		if !math.IsNaN(score) {
			// Try to keep the string no longer than 5 characters.
			switch {
			case score < 99.995:
				scoreStr = fmt.Sprintf("%.2f", score)
			case score < 999.95:
				scoreStr = fmt.Sprintf("%.1f", score)
			default:
				scoreStr = fmt.Sprintf("%.0f", score)
			}
		}
		var wampStr string
		if wamp := m.WriteAmp(); wamp > 99.5 {
			wampStr = fmt.Sprintf("%.0f", wamp)
		} else {
			wampStr = fmt.Sprintf("%.1f", wamp)
		}

		w.Printf("| %5s %6s %6s %7s | %5s | %5s | %5s %6s | %5s %6s | %5s %6s | %5s | %3d %4s",
			humanize.Count.Int64(m.NumFiles),
			humanize.Bytes.Int64(m.Size),
			humanize.Bytes.Uint64(m.Additional.ValueBlocksSize),
			humanize.Count.Uint64(m.NumVirtualFiles),
			redact.Safe(scoreStr),
			humanize.Bytes.Uint64(m.BytesIn),
			humanize.Count.Uint64(m.TablesIngested),
			humanize.Bytes.Uint64(m.BytesIngested),
			humanize.Count.Uint64(m.TablesMoved),
			humanize.Bytes.Uint64(m.BytesMoved),
			humanize.Count.Uint64(m.TablesFlushed+m.TablesCompacted),
			humanize.Bytes.Uint64(m.BytesFlushed+m.BytesCompacted),
			humanize.Bytes.Uint64(m.BytesRead),
			redact.Safe(m.Sublevels),
			redact.Safe(wampStr))

		if multiExists {
			w.Printf(" | %5s %5s %5s",
				humanize.Bytes.Uint64(m.MultiLevel.BytesInTop),
				humanize.Bytes.Uint64(m.MultiLevel.BytesIn),
				humanize.Bytes.Uint64(m.MultiLevel.BytesRead))
		}
		newline()
	}

	var total LevelMetrics
	for level := 0; level < numLevels; level++ {
		l := &m.Levels[level]
		w.Printf("%5d ", redact.Safe(level))

		// Format the score.
		score := math.NaN()
		if level < numLevels-1 {
			score = l.Score
		}
		formatRow(l, score)
		total.Add(l)
		total.Sublevels += l.Sublevels
	}
	// Compute total bytes-in as the bytes written to the WAL + bytes ingested.
	total.BytesIn = m.WAL.BytesWritten + total.BytesIngested
	// Add the total bytes-in to the total bytes-flushed. This is to account for
	// the bytes written to the log and bytes written externally and then
	// ingested.
	total.BytesFlushed += total.BytesIn
	w.SafeString("total ")
	formatRow(&total, math.NaN())

	w.SafeString("-------------------------------------------------------------------------------------------------------------------")
	appendIfMulti("--------------------")
	newline()
	w.Printf("WAL: %d files (%s)  in: %s  written: %s (%.0f%% overhead)\n",
		redact.Safe(m.WAL.Files),
		humanize.Bytes.Uint64(m.WAL.Size),
		humanize.Bytes.Uint64(m.WAL.BytesIn),
		humanize.Bytes.Uint64(m.WAL.BytesWritten),
		redact.Safe(percent(int64(m.WAL.BytesWritten)-int64(m.WAL.BytesIn), int64(m.WAL.BytesIn))))

	w.Printf("Flushes: %d\n", redact.Safe(m.Flush.Count))

	w.Printf("Compactions: %d  estimated debt: %s  in progress: %d (%s)\n",
		redact.Safe(m.Compact.Count),
		humanize.Bytes.Uint64(m.Compact.EstimatedDebt),
		redact.Safe(m.Compact.NumInProgress),
		humanize.Bytes.Int64(m.Compact.InProgressBytes))

	w.Printf("             default: %d  delete: %d  elision: %d  move: %d  read: %d  rewrite: %d  multi-level: %d\n",
		redact.Safe(m.Compact.DefaultCount),
		redact.Safe(m.Compact.DeleteOnlyCount),
		redact.Safe(m.Compact.ElisionOnlyCount),
		redact.Safe(m.Compact.MoveCount),
		redact.Safe(m.Compact.ReadCount),
		redact.Safe(m.Compact.RewriteCount),
		redact.Safe(m.Compact.MultiLevelCount))

	w.Printf("MemTables: %d (%s)  zombie: %d (%s)\n",
		redact.Safe(m.MemTable.Count),
		humanize.Bytes.Uint64(m.MemTable.Size),
		redact.Safe(m.MemTable.ZombieCount),
		humanize.Bytes.Uint64(m.MemTable.ZombieSize))

	w.Printf("Zombie tables: %d (%s)\n",
		redact.Safe(m.Table.ZombieCount),
		humanize.Bytes.Uint64(m.Table.ZombieSize))

	w.Printf("Backing tables: %d (%s)\n",
		redact.Safe(m.Table.BackingTableCount),
		humanize.Bytes.Uint64(m.Table.BackingTableSize))
	w.Printf("Virtual tables: %d (%s)\n",
		redact.Safe(m.NumVirtual()),
		humanize.Bytes.Uint64(m.VirtualSize()))

	formatCacheMetrics := func(m *CacheMetrics, name redact.SafeString) {
		w.Printf("%s: %s entries (%s)  hit rate: %.1f%%\n",
			name,
			humanize.Count.Int64(m.Count),
			humanize.Bytes.Int64(m.Size),
			redact.Safe(hitRate(m.Hits, m.Misses)))
	}
	formatCacheMetrics(&m.BlockCache, "Block cache")
	formatCacheMetrics(&m.TableCache, "Table cache")

	formatSharedCacheMetrics := func(w redact.SafePrinter, m *SecondaryCacheMetrics, name redact.SafeString) {
		w.Printf("%s: %s entries (%s)  hit rate: %.1f%%\n",
			name,
			humanize.Count.Int64(m.Count),
			humanize.Bytes.Int64(m.Size),
			redact.Safe(hitRate(m.ReadsWithFullHit, m.ReadsWithPartialHit+m.ReadsWithNoHit)))
	}
	formatSharedCacheMetrics(w, &m.SecondaryCacheMetrics, "Secondary cache")

	w.Printf("Snapshots: %d  earliest seq num: %d\n",
		redact.Safe(m.Snapshots.Count),
		redact.Safe(m.Snapshots.EarliestSeqNum))

	w.Printf("Table iters: %d\n", redact.Safe(m.TableIters))
	w.Printf("Filter utility: %.1f%%\n", redact.Safe(hitRate(m.Filter.Hits, m.Filter.Misses)))
	w.Printf("Ingestions: %d  as flushable: %d (%s in %d tables)\n",
		redact.Safe(m.Ingest.Count),
		redact.Safe(m.Flush.AsIngestCount),
		humanize.Bytes.Uint64(m.Flush.AsIngestBytes),
		redact.Safe(m.Flush.AsIngestTableCount))
}

func hitRate(hits, misses int64) float64 {
	return percent(hits, hits+misses)
}

func percent(numerator, denominator int64) float64 {
	if denominator == 0 {
		return 0
	}
	return 100 * float64(numerator) / float64(denominator)
}

// StringForTests is identical to m.String() on 64-bit platforms. It is used to
// provide a platform-independent result for tests.
func (m *Metrics) StringForTests() string {
	mCopy := *m
	if math.MaxInt == math.MaxInt32 {
		// This is the difference in Sizeof(sstable.Reader{})) between 64 and 32 bit
		// platforms.
		const tableCacheSizeAdjustment = 212
		mCopy.TableCache.Size += mCopy.TableCache.Count * tableCacheSizeAdjustment
	}
	return redact.StringWithoutMarkers(&mCopy)
}
