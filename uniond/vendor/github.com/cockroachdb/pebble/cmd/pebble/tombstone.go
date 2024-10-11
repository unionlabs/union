// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/spf13/cobra"
)

func init() {
	// NB: the tombstone workload piggybacks off the existing flags and
	// configs for the queue and ycsb workloads.
	initQueue(tombstoneCmd)
	initYCSB(tombstoneCmd)
}

var tombstoneCmd = &cobra.Command{
	Use:   "tombstone <dir>",
	Short: "run the mixed-workload point tombstone benchmark",
	Long: `
Run a customizable YCSB workload, alongside a single-writer, fixed-sized queue
workload. This command is intended for evaluating compaction heuristics
surrounding point tombstones.

The queue workload writes a point tombstone with every operation. A compaction
strategy that does not account for point tombstones may accumulate many
uncompacted tombstones, causing steady growth of the disk space consumed by
the queue keyspace.

The --queue-values flag controls the distribution of the queue value sizes.
Larger values are more likely to exhibit problematic point tombstone behavior
on a database using a min-overlapping ratio heuristic because the compact
point tombstones may overlap many tables in the next level.

The --queue-size flag controls the fixed number of live keys in the queue. Low
queue sizes may not exercise problematic tombstone behavior if queue sets and
deletes get written to the same sstable. The large-valued sets can serve as a
counterweight to the point tombstones, narrowing the keyrange of the sstable
inflating its size relative to its overlap with the next level.
	`,
	Args: cobra.ExactArgs(1),
	RunE: runTombstoneCmd,
}

func runTombstoneCmd(cmd *cobra.Command, args []string) error {
	if wipe && ycsbConfig.prepopulatedKeys > 0 {
		return errors.New("--wipe and --prepopulated-keys both specified which is nonsensical")
	}

	weights, err := ycsbParseWorkload(ycsbConfig.workload)
	if err != nil {
		return err
	}

	keyDist, err := ycsbParseKeyDist(ycsbConfig.keys)
	if err != nil {
		return err
	}

	batchDist := ycsbConfig.batch
	scanDist := ycsbConfig.scans
	if err != nil {
		return err
	}

	valueDist := ycsbConfig.values
	y := newYcsb(weights, keyDist, batchDist, scanDist, valueDist)
	q, queueOps := queueTest()

	queueStart := []byte("queue-")
	queueEnd := append(append([]byte{}, queueStart...), 0xFF)

	var lastElapsed time.Duration
	var lastQueueOps int64

	var pdb pebbleDB
	runTest(args[0], test{
		init: func(d DB, wg *sync.WaitGroup) {
			pdb = d.(pebbleDB)
			y.init(d, wg)
			q.init(d, wg)
		},
		tick: func(elapsed time.Duration, i int) {
			if i%20 == 0 {
				fmt.Println("                                             queue                         ycsb")
				fmt.Println("________elapsed______queue_size__ops/sec(inst)___ops/sec(cum)__ops/sec(inst)___ops/sec(cum)")
			}

			curQueueOps := queueOps.Load()
			dur := elapsed - lastElapsed
			queueOpsPerSec := float64(curQueueOps-lastQueueOps) / dur.Seconds()
			queueCumOpsPerSec := float64(curQueueOps) / elapsed.Seconds()

			lastQueueOps = curQueueOps
			lastElapsed = elapsed

			var ycsbOpsPerSec, ycsbCumOpsPerSec float64
			y.reg.Tick(func(tick histogramTick) {
				h := tick.Hist
				ycsbOpsPerSec = float64(h.TotalCount()) / tick.Elapsed.Seconds()
				ycsbCumOpsPerSec = float64(tick.Cumulative.TotalCount()) / elapsed.Seconds()
			})

			queueSize, err := pdb.d.EstimateDiskUsage(queueStart, queueEnd)
			if err != nil {
				log.Fatal(err)
			}
			fmt.Printf("%15s %15s %14.1f %14.1f %14.1f %14.1f\n",
				time.Duration(elapsed.Seconds()+0.5)*time.Second,
				humanize.Bytes.Uint64(queueSize),
				queueOpsPerSec,
				queueCumOpsPerSec,
				ycsbOpsPerSec,
				ycsbCumOpsPerSec)
		},
		done: func(elapsed time.Duration) {
			fmt.Println("________elapsed______queue_size")
			queueSize, err := pdb.d.EstimateDiskUsage(queueStart, queueEnd)
			if err != nil {
				log.Fatal(err)
			}
			fmt.Printf("%15s %15s\n", elapsed.Truncate(time.Second), humanize.Bytes.Uint64(queueSize))
		},
	})
	return nil
}
