// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
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

var queueConfig struct {
	size   int
	values *randvar.BytesFlag
}

func initQueue(cmd *cobra.Command) {
	cmd.Flags().IntVar(
		&queueConfig.size, "queue-size", 256,
		"size of the queue to maintain")
	queueConfig.values = randvar.NewBytesFlag("16384")
	cmd.Flags().Var(
		queueConfig.values, "queue-values",
		"queue value size distribution [{zipf,uniform}:]min[-max][/<target-compression>]")
}

func queueTest() (test, *atomic.Int64) {
	ops := new(atomic.Int64) // atomic
	var (
		lastOps     int64
		lastElapsed time.Duration
	)
	return test{
		init: func(d DB, wg *sync.WaitGroup) {
			var (
				value []byte
				rng   = rand.New(rand.NewSource(1449168817))
				queue = make([][]byte, queueConfig.size)
			)
			for i := 0; i < queueConfig.size; i++ {
				b := d.NewBatch()
				queue[i] = mvccEncode(nil, encodeUint32Ascending([]byte("queue-"), uint32(i)), uint64(i+1), 0)
				value = queueConfig.values.Bytes(rng, value)
				b.Set(queue[i], value, pebble.NoSync)
				if err := b.Commit(pebble.NoSync); err != nil {
					log.Fatal(err)
				}
			}
			if err := d.Flush(); err != nil {
				log.Fatal(err)
			}

			limiter := maxOpsPerSec.newRateLimiter()
			wg.Add(1)
			go func() {
				defer wg.Done()

				for i := queueConfig.size; ; i++ {
					idx := i % queueConfig.size

					// Delete the head.
					b := d.NewBatch()
					if err := b.Delete(queue[idx], pebble.Sync); err != nil {
						log.Fatal(err)
					}
					if err := b.Commit(pebble.Sync); err != nil {
						log.Fatal(err)
					}
					_ = b.Close()
					wait(limiter)

					// Append to the tail.
					b = d.NewBatch()
					queue[idx] = mvccEncode(queue[idx][:0], encodeUint32Ascending([]byte("queue-"), uint32(i)), uint64(i+1), 0)
					value = queueConfig.values.Bytes(rng, value)
					b.Set(queue[idx], value, nil)
					if err := b.Commit(pebble.Sync); err != nil {
						log.Fatal(err)
					}
					_ = b.Close()
					wait(limiter)
					ops.Add(1)
				}
			}()
		},
		tick: func(elapsed time.Duration, i int) {
			if i%20 == 0 {
				fmt.Println("Queue___elapsed_______ops/sec")
			}

			curOps := ops.Load()
			dur := elapsed - lastElapsed
			fmt.Printf("%15s %13.1f\n",
				time.Duration(elapsed.Seconds()+0.5)*time.Second,
				float64(curOps-lastOps)/dur.Seconds(),
			)
			lastOps = curOps
			lastElapsed = elapsed
		},
		done: func(elapsed time.Duration) {
			curOps := ops.Load()
			fmt.Println("\nQueue___elapsed___ops/sec(cum)")
			fmt.Printf("%13.1fs %14.1f\n\n",
				elapsed.Seconds(),
				float64(curOps)/elapsed.Seconds())
		},
	}, ops
}
