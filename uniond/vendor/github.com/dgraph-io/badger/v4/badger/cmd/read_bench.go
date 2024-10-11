/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package cmd

import (
	"context"
	"fmt"
	"math"
	"math/rand"
	"sync/atomic"
	"time"

	humanize "github.com/dustin/go-humanize"
	"github.com/pkg/errors"
	"github.com/spf13/cobra"

	"github.com/dgraph-io/badger/v4"
	"github.com/dgraph-io/badger/v4/pb"
	"github.com/dgraph-io/badger/v4/y"
	"github.com/dgraph-io/ristretto/z"
)

var readBenchCmd = &cobra.Command{
	Use:   "read",
	Short: "Read data from Badger randomly to benchmark read speed.",
	Long: `
This command reads data from existing Badger database randomly using multiple go routines.`,
	RunE: readBench,
}

var (
	sizeRead    atomic.Uint64 // will store size read till now
	entriesRead atomic.Uint64 // will store entries read till now
	startTime   time.Time     // start time of read benchmarking

	ro = struct {
		blockCacheSize int64
		indexCacheSize int64

		sampleSize int
		keysOnly   bool
		readOnly   bool
		fullScan   bool
	}{}
)

func init() {
	benchCmd.AddCommand(readBenchCmd)
	readBenchCmd.Flags().IntVarP(
		&numGoroutines, "goroutines", "g", 16, "Number of goroutines to run for reading.")
	readBenchCmd.Flags().StringVarP(
		&duration, "duration", "d", "1m", "How long to run the benchmark.")
	readBenchCmd.Flags().IntVar(
		&ro.sampleSize, "sample-size", 1000000, "Keys sample size to be used for random lookup.")
	readBenchCmd.Flags().BoolVar(
		&ro.keysOnly, "keys-only", false, "If false, values will also be read.")
	readBenchCmd.Flags().BoolVar(
		&ro.readOnly, "read-only", true, "If true, DB will be opened in read only mode.")
	readBenchCmd.Flags().BoolVar(
		&ro.fullScan, "full-scan", false, "If true, full db will be scanned using iterators.")
	readBenchCmd.Flags().Int64Var(&ro.blockCacheSize, "block-cache", 256, "Max size of block cache in MB")
	readBenchCmd.Flags().Int64Var(&ro.indexCacheSize, "index-cache", 0, "Max size of index cache in MB")
}

// Scan the whole database using the iterators
func fullScanDB(db *badger.DB) {
	txn := db.NewTransactionAt(math.MaxUint64, false)
	defer txn.Discard()

	startTime = time.Now()
	// Print the stats
	c := z.NewCloser(0)
	c.AddRunning(1)
	go printStats(c)

	it := txn.NewIterator(badger.DefaultIteratorOptions)
	defer it.Close()
	for it.Rewind(); it.Valid(); it.Next() {
		i := it.Item()
		entriesRead.Add(1)
		sizeRead.Add(uint64(i.EstimatedSize()))
	}
}

func readBench(cmd *cobra.Command, args []string) error {
	rand.Seed(time.Now().Unix())

	dur, err := time.ParseDuration(duration)
	if err != nil {
		return y.Wrapf(err, "unable to parse duration")
	}
	y.AssertTrue(numGoroutines > 0)
	opt := badger.DefaultOptions(sstDir).
		WithValueDir(vlogDir).
		WithReadOnly(ro.readOnly).
		WithBlockCacheSize(ro.blockCacheSize << 20).
		WithIndexCacheSize(ro.indexCacheSize << 20)
	fmt.Printf("Opening badger with options = %+v\n", opt)
	db, err := badger.OpenManaged(opt)
	if err != nil {
		return y.Wrapf(err, "unable to open DB")
	}
	defer db.Close()

	fmt.Println("*********************************************************")
	fmt.Println("Starting to benchmark Reads")
	fmt.Println("*********************************************************")

	// if fullScan is true then do a complete scan of the db and return
	if ro.fullScan {
		fullScanDB(db)
		return nil
	}
	readTest(db, dur)
	return nil
}

func printStats(c *z.Closer) {
	defer c.Done()

	t := time.NewTicker(time.Second)
	defer t.Stop()
	for {
		select {
		case <-c.HasBeenClosed():
			return
		case <-t.C:
			dur := time.Since(startTime)
			sz := sizeRead.Load()
			entries := entriesRead.Load()
			bytesRate := sz / uint64(dur.Seconds())
			entriesRate := entries / uint64(dur.Seconds())
			fmt.Printf("Time elapsed: %s, bytes read: %s, speed: %s/sec, "+
				"entries read: %d, speed: %d/sec\n", y.FixedDuration(time.Since(startTime)),
				humanize.IBytes(sz), humanize.IBytes(bytesRate), entries, entriesRate)
		}
	}
}

func readKeys(db *badger.DB, c *z.Closer, keys [][]byte) {
	defer c.Done()
	r := rand.New(rand.NewSource(time.Now().Unix()))
	for {
		select {
		case <-c.HasBeenClosed():
			return
		default:
			key := keys[r.Int31n(int32(len(keys)))]
			sizeRead.Add(lookupForKey(db, key))
			entriesRead.Add(1)
		}
	}
}

func lookupForKey(db *badger.DB, key []byte) (sz uint64) {
	err := db.View(func(txn *badger.Txn) error {
		iopt := badger.DefaultIteratorOptions
		iopt.AllVersions = true
		iopt.PrefetchValues = false
		it := txn.NewKeyIterator(key, iopt)
		defer it.Close()

		cnt := 0
		for it.Seek(key); it.Valid(); it.Next() {
			itm := it.Item()
			sz += uint64(itm.EstimatedSize())
			cnt++
			if cnt == 10 {
				break
			}
		}
		return nil
	})
	y.Check(err)
	return
}

// getSampleKeys uses stream framework internally, to get keys in random order.
func getSampleKeys(db *badger.DB, sampleSize int) ([][]byte, error) {
	var keys [][]byte
	count := 0
	stream := db.NewStreamAt(math.MaxUint64)

	// overide stream.KeyToList as we only want keys. Also
	// we can take only first version for the key.
	stream.KeyToList = func(key []byte, itr *badger.Iterator) (*pb.KVList, error) {
		l := &pb.KVList{}
		// Since stream framework copies the item's key while calling
		// KeyToList, we can directly append key to list.
		l.Kv = append(l.Kv, &pb.KV{Key: key})
		return l, nil
	}

	errStop := errors.Errorf("Stop iterating")
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	stream.Send = func(buf *z.Buffer) error {
		if count >= ro.sampleSize {
			return nil
		}
		err := buf.SliceIterate(func(s []byte) error {
			var kv pb.KV
			if err := kv.Unmarshal(s); err != nil {
				return err
			}
			keys = append(keys, kv.Key)
			count++
			if count >= sampleSize {
				cancel()
				return errStop
			}
			return nil
		})
		if err == errStop || err == nil {
			return nil
		}
		return err
	}

	if err := stream.Orchestrate(ctx); err != nil && err != context.Canceled {
		return nil, err
	}

	// Shuffle keys before returning to minimise locality
	// of keys coming from stream framework.
	rand.Shuffle(len(keys), func(i, j int) {
		keys[i], keys[j] = keys[j], keys[i]
	})

	return keys, nil
}
