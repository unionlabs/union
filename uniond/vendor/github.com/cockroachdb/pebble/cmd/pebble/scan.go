// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"fmt"
	"log"
	"math"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/randvar"
	"github.com/spf13/cobra"
	"golang.org/x/exp/rand"
)

var scanConfig struct {
	reverse bool
	rows    *randvar.Flag
	values  *randvar.BytesFlag
}

var scanCmd = &cobra.Command{
	Use:   "scan <dir>",
	Short: "run the scan benchmark",
	Long:  ``,
	Args:  cobra.ExactArgs(1),
	Run:   runScan,
}

func init() {
	scanCmd.Flags().BoolVarP(
		&scanConfig.reverse, "reverse", "r", false, "reverse scan")
	scanConfig.rows = randvar.NewFlag("100")
	scanCmd.Flags().Var(
		scanConfig.rows, "rows", "number of rows to scan in each operation")
	scanConfig.values = randvar.NewBytesFlag("8")
	scanCmd.Flags().Var(
		scanConfig.values, "values",
		"value size distribution [{zipf,uniform}:]min[-max][/<target-compression>]")
}

func runScan(cmd *cobra.Command, args []string) {
	var (
		bytes       atomic.Int64
		scanned     atomic.Int64
		lastBytes   int64
		lastScanned int64
		lastElapsed time.Duration
	)

	opts := pebble.Sync
	if disableWAL {
		opts = pebble.NoSync
	}

	rowDist := scanConfig.rows

	runTest(args[0], test{
		init: func(d DB, wg *sync.WaitGroup) {
			const count = 100000
			const batch = 1000

			rng := rand.New(rand.NewSource(1449168817))
			keys := make([][]byte, count)

			for i := 0; i < count; {
				b := d.NewBatch()
				var value []byte
				for end := i + batch; i < end; i++ {
					keys[i] = mvccEncode(nil, encodeUint32Ascending([]byte("key-"), uint32(i)), uint64(i+1), 0)
					value = scanConfig.values.Bytes(rng, value)
					if err := b.Set(keys[i], value, nil); err != nil {
						log.Fatal(err)
					}
				}
				if err := b.Commit(opts); err != nil {
					log.Fatal(err)
				}
			}

			if err := d.Flush(); err != nil {
				log.Fatal(err)
			}

			limiter := maxOpsPerSec.newRateLimiter()

			wg.Add(concurrency)
			for i := 0; i < concurrency; i++ {
				go func(i int) {
					defer wg.Done()

					rng := rand.New(rand.NewSource(uint64(i)))
					startKeyBuf := append(make([]byte, 0, 64), []byte("key-")...)
					endKeyBuf := append(make([]byte, 0, 64), []byte("key-")...)
					minTS := encodeUint64Ascending(nil, math.MaxUint64)

					for {
						wait(limiter)

						rows := int(rowDist.Uint64(rng))
						startIdx := rng.Int31n(int32(len(keys) - rows))
						startKey := encodeUint32Ascending(startKeyBuf[:4], uint32(startIdx))
						endKey := encodeUint32Ascending(endKeyBuf[:4], uint32(startIdx+int32(rows)))

						var count int
						var nbytes int64
						if scanConfig.reverse {
							count, nbytes = mvccReverseScan(d, startKey, endKey, minTS)
						} else {
							count, nbytes = mvccForwardScan(d, startKey, endKey, minTS)
						}

						if count != rows {
							log.Fatalf("scanned %d, expected %d\n", count, rows)
						}

						bytes.Add(nbytes)
						scanned.Add(int64(count))
					}
				}(i)
			}
		},

		tick: func(elapsed time.Duration, i int) {
			if i%20 == 0 {
				fmt.Println("_elapsed_______rows/sec_______MB/sec_______ns/row")
			}

			curBytes := bytes.Load()
			curScanned := scanned.Load()
			dur := elapsed - lastElapsed
			fmt.Printf("%8s %14.1f %12.1f %12.1f\n",
				time.Duration(elapsed.Seconds()+0.5)*time.Second,
				float64(curScanned-lastScanned)/dur.Seconds(),
				float64(curBytes-lastBytes)/(dur.Seconds()*(1<<20)),
				float64(dur)/float64(curScanned-lastScanned),
			)
			lastBytes = curBytes
			lastScanned = curScanned
			lastElapsed = elapsed
		},

		done: func(elapsed time.Duration) {
			curBytes := bytes.Load()
			curScanned := scanned.Load()
			fmt.Println("\n_elapsed___ops/sec(cum)__MB/sec(cum)__ns/row(avg)")
			fmt.Printf("%7.1fs %14.1f %12.1f %12.1f\n\n",
				elapsed.Seconds(),
				float64(curScanned)/elapsed.Seconds(),
				float64(curBytes)/(elapsed.Seconds()*(1<<20)),
				float64(elapsed)/float64(curScanned),
			)
		},
	})
}
