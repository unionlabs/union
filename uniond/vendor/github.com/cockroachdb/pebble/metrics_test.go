// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"fmt"
	"math/rand"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/redact"
	"github.com/stretchr/testify/require"
)

func exampleMetrics() Metrics {
	var m Metrics
	m.BlockCache.Size = 1
	m.BlockCache.Count = 2
	m.BlockCache.Hits = 3
	m.BlockCache.Misses = 4
	m.Compact.Count = 5
	m.Compact.DefaultCount = 27
	m.Compact.DeleteOnlyCount = 28
	m.Compact.ElisionOnlyCount = 29
	m.Compact.MoveCount = 30
	m.Compact.ReadCount = 31
	m.Compact.RewriteCount = 32
	m.Compact.MultiLevelCount = 33
	m.Compact.EstimatedDebt = 6
	m.Compact.InProgressBytes = 7
	m.Compact.NumInProgress = 2
	m.Flush.Count = 8
	m.Flush.AsIngestBytes = 34
	m.Flush.AsIngestTableCount = 35
	m.Flush.AsIngestCount = 36
	m.Filter.Hits = 9
	m.Filter.Misses = 10
	m.MemTable.Size = 11
	m.MemTable.Count = 12
	m.MemTable.ZombieSize = 13
	m.MemTable.ZombieCount = 14
	m.Snapshots.Count = 4
	m.Snapshots.EarliestSeqNum = 1024
	m.Table.ZombieSize = 15
	m.Table.BackingTableCount = 1
	m.Table.BackingTableSize = 2 << 20
	m.Table.ZombieCount = 16
	m.TableCache.Size = 17
	m.TableCache.Count = 18
	m.TableCache.Hits = 19
	m.TableCache.Misses = 20
	m.TableIters = 21
	m.WAL.Files = 22
	m.WAL.ObsoleteFiles = 23
	m.WAL.Size = 24
	m.WAL.BytesIn = 25
	m.WAL.BytesWritten = 26
	m.Ingest.Count = 27

	for i := range m.Levels {
		l := &m.Levels[i]
		base := uint64((i + 1) * 100)
		l.Sublevels = int32(i + 1)
		l.NumFiles = int64(base) + 1
		l.NumVirtualFiles = uint64(base) + 1
		l.VirtualSize = base + 3
		l.Size = int64(base) + 2
		l.Score = float64(base) + 3
		l.BytesIn = base + 4
		l.BytesIngested = base + 4
		l.BytesMoved = base + 6
		l.BytesRead = base + 7
		l.BytesCompacted = base + 8
		l.BytesFlushed = base + 9
		l.TablesCompacted = base + 10
		l.TablesFlushed = base + 11
		l.TablesIngested = base + 12
		l.TablesMoved = base + 13
		l.MultiLevel.BytesInTop = base + 4
		l.MultiLevel.BytesIn = base + 4
		l.MultiLevel.BytesRead = base + 4
	}
	return m
}

func TestMetrics(t *testing.T) {
	if runtime.GOARCH == "386" {
		t.Skip("skipped on 32-bit due to slightly varied output")
	}
	opts := &Options{
		Comparer:              testkeys.Comparer,
		FormatMajorVersion:    FormatNewest,
		FS:                    vfs.NewMem(),
		L0CompactionThreshold: 8,
	}
	opts.Experimental.EnableValueBlocks = func() bool { return true }
	opts.Levels = append(opts.Levels, LevelOptions{TargetFileSize: 50})

	// Prevent foreground flushes and compactions from triggering asynchronous
	// follow-up compactions. This avoids asynchronously-scheduled work from
	// interfering with the expected metrics output and reduces test flakiness.
	opts.DisableAutomaticCompactions = true

	// Increase the threshold for memtable stalls to allow for more flushable
	// ingests.
	opts.MemTableStopWritesThreshold = 4

	d, err := Open("", opts)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, d.Close())
	}()

	iters := make(map[string]*Iterator)
	defer func() {
		for _, i := range iters {
			require.NoError(t, i.Close())
		}
	}()

	datadriven.RunTest(t, "testdata/metrics", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "example":
			m := exampleMetrics()
			res := m.String()

			// Nothing in the metrics should be redacted.
			redacted := string(redact.Sprintf("%s", &m).Redact())
			if redacted != res {
				td.Fatalf(t, "redacted metrics don't match\nunredacted:\n%s\nredacted:%s\n", res, redacted)
			}
			return res

		case "batch":
			b := d.NewBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			b.Commit(nil)
			return ""

		case "build":
			if err := runBuildCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			return ""

		case "compact":
			if err := runCompactCmd(td, d); err != nil {
				return err.Error()
			}

			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "delay-flush":
			d.mu.Lock()
			defer d.mu.Unlock()
			switch td.Input {
			case "enable":
				d.mu.compact.flushing = true
			case "disable":
				d.mu.compact.flushing = false
			default:
				return fmt.Sprintf("unknown directive %q (expected 'enable'/'disable')", td.Input)
			}
			return ""

		case "flush":
			if err := d.Flush(); err != nil {
				return err.Error()
			}

			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "ingest":
			if err := runIngestCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			return ""

		case "lsm":
			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "ingest-and-excise":
			if err := runIngestAndExciseCmd(td, d, d.opts.FS); err != nil {
				return err.Error()
			}
			return ""

		case "iter-close":
			if len(td.CmdArgs) != 1 {
				return "iter-close <name>"
			}
			name := td.CmdArgs[0].String()
			if iter := iters[name]; iter != nil {
				if err := iter.Close(); err != nil {
					return err.Error()
				}
				delete(iters, name)
			} else {
				return fmt.Sprintf("%s: not found", name)
			}

			// The deletion of obsolete files happens asynchronously when an iterator
			// is closed. Wait for the obsolete tables to be deleted.
			d.cleanupManager.Wait()
			return ""

		case "iter-new":
			if len(td.CmdArgs) != 1 {
				return "iter-new <name>"
			}
			name := td.CmdArgs[0].String()
			if iter := iters[name]; iter != nil {
				if err := iter.Close(); err != nil {
					return err.Error()
				}
			}
			iter, _ := d.NewIter(nil)
			// Some iterators (eg. levelIter) do not instantiate the underlying
			// iterator until the first positioning call. Position the iterator
			// so that levelIters will have loaded an sstable.
			iter.First()
			iters[name] = iter
			return ""

		case "metrics":
			// The asynchronous loading of table stats can change metrics, so
			// wait for all the tables' stats to be loaded.
			d.mu.Lock()
			d.waitTableStats()
			d.mu.Unlock()

			return d.Metrics().StringForTests()

		case "metrics-value":
			// metrics-value confirms the value of a given metric. Note that there
			// are some metrics which aren't deterministic and behave differently
			// for invariant/non-invariant builds. An example of this is cache
			// hit rates. Under invariant builds, the excising code will try
			// to create iterators and confirm that the virtual sstable bounds
			// are accurate. Reads on these iterators will change the cache hit
			// rates.
			lines := strings.Split(td.Input, "\n")
			m := d.Metrics()
			// TODO(bananabrick): Use reflection to pull the values associated
			// with the metrics fields.
			var buf bytes.Buffer
			for i := range lines {
				line := lines[i]
				if line == "num-backing" {
					buf.WriteString(fmt.Sprintf("%d\n", m.Table.BackingTableCount))
				} else if line == "backing-size" {
					buf.WriteString(fmt.Sprintf("%s\n", humanize.Bytes.Uint64(m.Table.BackingTableSize)))
				} else if line == "virtual-size" {
					buf.WriteString(fmt.Sprintf("%s\n", humanize.Bytes.Uint64(m.VirtualSize())))
				} else if strings.HasPrefix(line, "num-virtual") {
					splits := strings.Split(line, " ")
					if len(splits) == 1 {
						buf.WriteString(fmt.Sprintf("%d\n", m.NumVirtual()))
						continue
					}
					// Level is specified.
					l, err := strconv.Atoi(splits[1])
					if err != nil {
						panic(err)
					}
					if l >= numLevels {
						panic(fmt.Sprintf("invalid level %d", l))
					}
					buf.WriteString(fmt.Sprintf("%d\n", m.Levels[l].NumVirtualFiles))
				} else {
					panic(fmt.Sprintf("invalid field: %s", line))
				}
			}
			return buf.String()

		case "disk-usage":
			return humanize.Bytes.Uint64(d.Metrics().DiskSpaceUsage()).String()

		case "additional-metrics":
			// The asynchronous loading of table stats can change metrics, so
			// wait for all the tables' stats to be loaded.
			d.mu.Lock()
			d.waitTableStats()
			d.mu.Unlock()

			m := d.Metrics()
			var b strings.Builder
			fmt.Fprintf(&b, "block bytes written:\n")
			fmt.Fprintf(&b, " __level___data-block__value-block\n")
			for i := range m.Levels {
				fmt.Fprintf(&b, "%7d ", i)
				fmt.Fprintf(&b, "%12s %12s\n",
					humanize.Bytes.Uint64(m.Levels[i].Additional.BytesWrittenDataBlocks),
					humanize.Bytes.Uint64(m.Levels[i].Additional.BytesWrittenValueBlocks))
			}
			return b.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestMetricsWAmpDisableWAL(t *testing.T) {
	d, err := Open("", &Options{FS: vfs.NewMem(), DisableWAL: true})
	require.NoError(t, err)
	ks := testkeys.Alpha(2)
	wo := WriteOptions{Sync: false}
	for i := 0; i < 5; i++ {
		v := []byte(strconv.Itoa(i))
		for j := int64(0); j < ks.Count(); j++ {
			require.NoError(t, d.Set(testkeys.Key(ks, j), v, &wo))
		}
		require.NoError(t, d.Flush())
		require.NoError(t, d.Compact([]byte("a"), []byte("z"), false /* parallelize */))
	}
	m := d.Metrics()
	tot := m.Total()
	require.Greater(t, tot.WriteAmp(), 1.0)
	require.NoError(t, d.Close())
}

// TestMetricsWALBytesWrittenMonotonicity tests that the
// Metrics.WAL.BytesWritten metric is always nondecreasing.
// It's a regression test for issue #3505.
func TestMetricsWALBytesWrittenMonotonicity(t *testing.T) {
	fs := vfs.NewMem()
	d, err := Open("", &Options{
		FS: fs,
		// Use a tiny memtable size so that we get frequent flushes. While a
		// flush is in-progress or completing, the WAL bytes written should
		// remain nondecreasing.
		MemTableSize: 1 << 20, /* 20 KiB */
	})
	require.NoError(t, err)

	stopCh := make(chan struct{})

	ks := testkeys.Alpha(3)
	var wg sync.WaitGroup
	const concurrentWriters = 5
	wg.Add(concurrentWriters)
	for w := 0; w < concurrentWriters; w++ {
		go func() {
			defer wg.Done()
			data := make([]byte, 1<<10) // 1 KiB
			rng := rand.New(rand.NewSource(time.Now().UnixNano()))
			_, err := rng.Read(data)
			require.NoError(t, err)

			buf := make([]byte, ks.MaxLen())
			for i := 0; ; i++ {
				select {
				case <-stopCh:
					return
				default:
				}
				n := testkeys.WriteKey(buf, ks, int64(i)%ks.Count())
				require.NoError(t, d.Set(buf[:n], data, NoSync))
			}
		}()
	}

	func() {
		defer func() { close(stopCh) }()
		abort := time.After(time.Second)
		var prevWALBytesWritten uint64
		for {
			select {
			case <-abort:
				return
			default:
			}

			m := d.Metrics()
			if m.WAL.BytesWritten < prevWALBytesWritten {
				t.Fatalf("WAL bytes written decreased: %d -> %d", prevWALBytesWritten, m.WAL.BytesWritten)
			}
			prevWALBytesWritten = m.WAL.BytesWritten
		}
	}()
	wg.Wait()
}
