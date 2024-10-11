// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"fmt"
	"log"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/randvar"
	"github.com/spf13/cobra"
	"golang.org/x/exp/rand"
)

var syncConfig struct {
	batch   *randvar.Flag
	walOnly bool
	values  *randvar.BytesFlag
}

var syncCmd = &cobra.Command{
	Use:   "sync <dir>",
	Short: "run the sync benchmark",
	Long:  ``,
	Args:  cobra.ExactArgs(1),
	Run:   runSync,
}

func init() {
	syncConfig.batch = randvar.NewFlag("5")
	syncCmd.Flags().Var(
		syncConfig.batch, "batch",
		"batch size distribution [{zipf,uniform}:]min[-max]")
	syncCmd.Flags().BoolVar(
		&syncConfig.walOnly, "wal-only", false, "write data only to the WAL")
	syncConfig.values = randvar.NewBytesFlag("uniform:60-80/1.0")
	syncCmd.Flags().Var(
		syncConfig.values, "values",
		"value size distribution [{zipf,uniform}:]min[-max][/<target-compression>]")
}

func runSync(cmd *cobra.Command, args []string) {
	reg := newHistogramRegistry()
	var bytes atomic.Uint64
	var lastBytes uint64

	opts := pebble.Sync
	if disableWAL {
		opts = pebble.NoSync
	}

	batchDist := syncConfig.batch

	runTest(args[0], test{
		init: func(d DB, wg *sync.WaitGroup) {
			limiter := maxOpsPerSec.newRateLimiter()

			wg.Add(concurrency)
			for i := 0; i < concurrency; i++ {
				latency := reg.Register("ops")
				go func() {
					defer wg.Done()

					rand := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))
					var raw []byte
					var buf []byte
					var block []byte
					for {
						wait(limiter)

						start := time.Now()
						b := d.NewBatch()
						var n uint64
						count := int(batchDist.Uint64(rand))
						for j := 0; j < count; j++ {
							block = syncConfig.values.Bytes(rand, block)

							if syncConfig.walOnly {
								if err := b.LogData(block, nil); err != nil {
									log.Fatal(err)
								}
							} else {
								raw = encodeUint32Ascending(raw[:0], rand.Uint32())
								key := mvccEncode(buf[:0], raw, 0, 0)
								buf = key[:0]
								if err := b.Set(key, block, nil); err != nil {
									log.Fatal(err)
								}
							}
							n += uint64(len(block))
						}
						if err := b.Commit(opts); err != nil {
							log.Fatal(err)
						}
						latency.Record(time.Since(start))
						bytes.Add(n)
					}
				}()
			}
		},

		tick: func(elapsed time.Duration, i int) {
			if i%20 == 0 {
				fmt.Println("_elapsed____ops/sec___mb/sec__p50(ms)__p95(ms)__p99(ms)_pMax(ms)")
			}
			reg.Tick(func(tick histogramTick) {
				h := tick.Hist
				n := bytes.Load()
				fmt.Printf("%8s %10.1f %8.1f %8.1f %8.1f %8.1f %8.1f\n",
					time.Duration(elapsed.Seconds()+0.5)*time.Second,
					float64(h.TotalCount())/tick.Elapsed.Seconds(),
					float64(n-lastBytes)/(1024.0*1024.0)/tick.Elapsed.Seconds(),
					time.Duration(h.ValueAtQuantile(50)).Seconds()*1000,
					time.Duration(h.ValueAtQuantile(95)).Seconds()*1000,
					time.Duration(h.ValueAtQuantile(99)).Seconds()*1000,
					time.Duration(h.ValueAtQuantile(100)).Seconds()*1000,
				)
				lastBytes = n
			})
		},

		done: func(elapsed time.Duration) {
			fmt.Println("\n_elapsed___ops(total)_ops/sec(cum)_mb/sec(cum)__avg(ms)__p50(ms)__p95(ms)__p99(ms)_pMax(ms)")
			reg.Tick(func(tick histogramTick) {
				h := tick.Cumulative
				fmt.Printf("%7.1fs %12d %12.1f %11.1f %8.1f %8.1f %8.1f %8.1f %8.1f\n\n",
					elapsed.Seconds(), h.TotalCount(),
					float64(h.TotalCount())/elapsed.Seconds(),
					float64(bytes.Load()/(1024.0*1024.0))/elapsed.Seconds(),
					time.Duration(h.Mean()).Seconds()*1000,
					time.Duration(h.ValueAtQuantile(50)).Seconds()*1000,
					time.Duration(h.ValueAtQuantile(95)).Seconds()*1000,
					time.Duration(h.ValueAtQuantile(99)).Seconds()*1000,
					time.Duration(h.ValueAtQuantile(100)).Seconds()*1000)
			})
		},
	})
}
