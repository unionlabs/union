// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"encoding/binary"
	"fmt"
	"io"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/cockroachdb/pebble/internal/arenaskl"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

type testCommitEnv struct {
	logSeqNum     atomic.Uint64
	visibleSeqNum atomic.Uint64
	writeCount    atomic.Uint64
	applyBuf      struct {
		sync.Mutex
		buf []uint64
	}
	queueSemChan chan struct{}
}

func (e *testCommitEnv) env() commitEnv {
	return commitEnv{
		logSeqNum:     &e.logSeqNum,
		visibleSeqNum: &e.visibleSeqNum,
		apply:         e.apply,
		write:         e.write,
	}
}

func (e *testCommitEnv) apply(b *Batch, mem *memTable) error {
	e.applyBuf.Lock()
	e.applyBuf.buf = append(e.applyBuf.buf, b.SeqNum())
	e.applyBuf.Unlock()
	return nil
}

func (e *testCommitEnv) write(b *Batch, wg *sync.WaitGroup, _ *error) (*memTable, error) {
	e.writeCount.Add(1)
	if wg != nil {
		wg.Done()
		<-e.queueSemChan
	}
	return nil, nil
}

func TestCommitQueue(t *testing.T) {
	var q commitQueue
	var batches [16]Batch
	for i := range batches {
		q.enqueue(&batches[i])
	}
	if b := q.dequeueApplied(); b != nil {
		t.Fatalf("unexpectedly dequeued batch: %p", b)
	}
	batches[1].applied.Store(true)
	if b := q.dequeueApplied(); b != nil {
		t.Fatalf("unexpectedly dequeued batch: %p", b)
	}
	for i := range batches {
		batches[i].applied.Store(true)
		if b := q.dequeueApplied(); b != &batches[i] {
			t.Fatalf("%d: expected batch %p, but found %p", i, &batches[i], b)
		}
	}
	if b := q.dequeueApplied(); b != nil {
		t.Fatalf("unexpectedly dequeued batch: %p", b)
	}
}

func TestCommitPipeline(t *testing.T) {
	var e testCommitEnv
	p := newCommitPipeline(e.env())

	n := 10000
	if invariants.RaceEnabled {
		// Under race builds we have to limit the concurrency or we hit the
		// following error:
		//
		//   race: limit on 8128 simultaneously alive goroutines is exceeded, dying
		n = 1000
	}

	var wg sync.WaitGroup
	wg.Add(n)
	for i := 0; i < n; i++ {
		go func(i int) {
			defer wg.Done()
			var b Batch
			_ = b.Set([]byte(fmt.Sprint(i)), nil, nil)
			_ = p.Commit(&b, false, false)
		}(i)
	}
	wg.Wait()

	if s := e.writeCount.Load(); uint64(n) != s {
		t.Fatalf("expected %d written batches, but found %d", n, s)
	}
	if n != len(e.applyBuf.buf) {
		t.Fatalf("expected %d written batches, but found %d",
			n, len(e.applyBuf.buf))
	}
	if s := e.logSeqNum.Load(); uint64(n) != s {
		t.Fatalf("expected %d, but found %d", n, s)
	}
	if s := e.visibleSeqNum.Load(); uint64(n) != s {
		t.Fatalf("expected %d, but found %d", n, s)
	}
}

func TestCommitPipelineSync(t *testing.T) {
	n := 10000
	if invariants.RaceEnabled {
		// Under race builds we have to limit the concurrency or we hit the
		// following error:
		//
		//   race: limit on 8128 simultaneously alive goroutines is exceeded, dying
		n = 1000
	}

	for _, noSyncWait := range []bool{false, true} {
		t.Run(fmt.Sprintf("no-sync-wait=%t", noSyncWait), func(t *testing.T) {
			var e testCommitEnv
			p := newCommitPipeline(e.env())
			e.queueSemChan = p.logSyncQSem

			var wg sync.WaitGroup
			wg.Add(n)
			for i := 0; i < n; i++ {
				go func(i int) {
					defer wg.Done()
					var b Batch
					require.NoError(t, b.Set([]byte(fmt.Sprint(i)), nil, nil))
					require.NoError(t, p.Commit(&b, true, noSyncWait))
					if noSyncWait {
						require.NoError(t, b.SyncWait())
					}
				}(i)
			}
			wg.Wait()
			if s := e.writeCount.Load(); uint64(n) != s {
				t.Fatalf("expected %d written batches, but found %d", n, s)
			}
			if n != len(e.applyBuf.buf) {
				t.Fatalf("expected %d written batches, but found %d",
					n, len(e.applyBuf.buf))
			}
			if s := e.logSeqNum.Load(); uint64(n) != s {
				t.Fatalf("expected %d, but found %d", n, s)
			}
			if s := e.visibleSeqNum.Load(); uint64(n) != s {
				t.Fatalf("expected %d, but found %d", n, s)
			}
		})
	}
}

func TestCommitPipelineAllocateSeqNum(t *testing.T) {
	var e testCommitEnv
	p := newCommitPipeline(e.env())

	const n = 10
	var wg sync.WaitGroup
	wg.Add(n)
	var prepareCount atomic.Uint64
	var applyCount atomic.Uint64
	for i := 1; i <= n; i++ {
		go func(i int) {
			defer wg.Done()
			p.AllocateSeqNum(i, func(_ uint64) {
				prepareCount.Add(1)
			}, func(seqNum uint64) {
				applyCount.Add(1)
			})
		}(i)
	}
	wg.Wait()

	if s := prepareCount.Load(); n != s {
		t.Fatalf("expected %d prepares, but found %d", n, s)
	}
	if s := applyCount.Load(); n != s {
		t.Fatalf("expected %d applies, but found %d", n, s)
	}
	// AllocateSeqNum always returns a non-zero sequence number causing the
	// values we see to be offset from 1.
	const total = 1 + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10
	if s := e.logSeqNum.Load(); total != s {
		t.Fatalf("expected %d, but found %d", total, s)
	}
	if s := e.visibleSeqNum.Load(); total != s {
		t.Fatalf("expected %d, but found %d", total, s)
	}
}

type syncDelayFile struct {
	vfs.File
	done chan struct{}
}

func (f *syncDelayFile) Sync() error {
	<-f.done
	return nil
}

func TestCommitPipelineWALClose(t *testing.T) {
	// This test stresses the edge case of N goroutines blocked in the
	// commitPipeline waiting for the log to sync when we concurrently decide to
	// rotate and close the log.

	mem := vfs.NewMem()
	f, err := mem.Create("test-wal")
	require.NoError(t, err)

	// syncDelayFile will block on the done channel befor returning from Sync
	// call.
	sf := &syncDelayFile{
		File: f,
		done: make(chan struct{}),
	}

	// A basic commitEnv which writes to a WAL.
	var wal *record.LogWriter
	var walDone sync.WaitGroup
	testEnv := commitEnv{
		logSeqNum:     new(atomic.Uint64),
		visibleSeqNum: new(atomic.Uint64),
		apply: func(b *Batch, mem *memTable) error {
			// At this point, we've called SyncRecord but the sync is blocked.
			walDone.Done()
			return nil
		},
		write: func(b *Batch, syncWG *sync.WaitGroup, syncErr *error) (*memTable, error) {
			_, err := wal.SyncRecord(b.data, syncWG, syncErr)
			return nil, err
		},
	}
	p := newCommitPipeline(testEnv)
	wal = record.NewLogWriter(sf, 0 /* logNum */, record.LogWriterConfig{
		WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{}),
		QueueSemChan:    p.logSyncQSem,
	})

	// Launch N (commitConcurrency) goroutines which each create a batch and
	// commit it with sync==true. Because of the syncDelayFile, none of these
	// operations can complete until syncDelayFile.done is closed.
	errCh := make(chan error, cap(p.commitQueueSem))
	walDone.Add(cap(errCh))
	for i := 0; i < cap(errCh); i++ {
		go func(i int) {
			b := &Batch{}
			if err := b.LogData([]byte("foo"), nil); err != nil {
				errCh <- err
				return
			}
			errCh <- p.Commit(b, true /* sync */, false)
		}(i)
	}

	// Wait for all of the WAL writes to queue up. This ensures we don't violate
	// the concurrency requirements of LogWriter, and also ensures all of the WAL
	// writes are queued.
	walDone.Wait()
	close(sf.done)

	// Close the WAL. A "queue is full" panic means that something is broken.
	require.NoError(t, wal.Close())
	for i := 0; i < cap(errCh); i++ {
		require.NoError(t, <-errCh)
	}
}

func BenchmarkCommitPipeline(b *testing.B) {
	for _, noSyncWait := range []bool{false, true} {
		for _, parallelism := range []int{1, 2, 4, 8, 16, 32, 64, 128} {
			b.Run(fmt.Sprintf("no-sync-wait=%t/parallel=%d", noSyncWait, parallelism),
				func(b *testing.B) {
					b.SetParallelism(parallelism)
					mem := newMemTable(memTableOptions{})
					var wal *record.LogWriter
					nullCommitEnv := commitEnv{
						logSeqNum:     new(atomic.Uint64),
						visibleSeqNum: new(atomic.Uint64),
						apply: func(b *Batch, mem *memTable) error {
							err := mem.apply(b, b.SeqNum())
							if err != nil {
								return err
							}
							mem.writerUnref()
							return nil
						},
						write: func(b *Batch, syncWG *sync.WaitGroup, syncErr *error) (*memTable, error) {
							for {
								err := mem.prepare(b)
								if err == arenaskl.ErrArenaFull {
									mem = newMemTable(memTableOptions{})
									continue
								}
								if err != nil {
									return nil, err
								}
								break
							}

							_, err := wal.SyncRecord(b.data, syncWG, syncErr)
							return mem, err
						},
					}
					p := newCommitPipeline(nullCommitEnv)
					wal = record.NewLogWriter(io.Discard, 0, /* logNum */
						record.LogWriterConfig{
							WALFsyncLatency: prometheus.NewHistogram(prometheus.HistogramOpts{}),
							QueueSemChan:    p.logSyncQSem,
						})
					const keySize = 8
					b.SetBytes(2 * keySize)
					b.ResetTimer()

					b.RunParallel(func(pb *testing.PB) {
						rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))
						buf := make([]byte, keySize)

						for pb.Next() {
							batch := newBatch(nil)
							binary.BigEndian.PutUint64(buf, rng.Uint64())
							batch.Set(buf, buf, nil)
							if err := p.Commit(batch, true /* sync */, noSyncWait); err != nil {
								b.Fatal(err)
							}
							if noSyncWait {
								if err := batch.SyncWait(); err != nil {
									b.Fatal(err)
								}
							}
							batch.release()
						}
					})
				})
		}
	}
}
