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
	"bytes"
	"encoding/binary"
	"fmt"
	"log"
	"math"
	"math/rand"
	"os"
	"path/filepath"
	"sync"
	"sync/atomic"
	"time"

	humanize "github.com/dustin/go-humanize"
	"github.com/spf13/cobra"

	"github.com/dgraph-io/badger/v4"
	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/pb"
	"github.com/dgraph-io/badger/v4/y"
	"github.com/dgraph-io/ristretto/z"
)

var writeBenchCmd = &cobra.Command{
	Use:   "write",
	Short: "Writes random data to Badger to benchmark write speed.",
	Long: `
	 This command writes random data to Badger to benchmark write speed. Useful for testing and
	 performance analysis.
	 `,
	RunE: writeBench,
}

var (
	wo = struct {
		keySz      int
		valSz      int
		numKeys    float64
		syncWrites bool
		force      bool
		sorted     bool
		showLogs   bool

		valueThreshold   int64
		numVersions      int
		vlogMaxEntries   uint32
		loadBloomsOnOpen bool
		detectConflicts  bool
		zstdComp         bool
		showDir          bool
		ttlDuration      string
		encryptionKey    string
		showKeysCount    bool
		blockCacheSize   int64
		indexCacheSize   int64

		dropAllPeriod    string
		dropPrefixPeriod string
		gcPeriod         string
		gcDiscardRatio   float64
	}{}

	sizeWritten    atomic.Uint64
	gcSuccess      atomic.Uint64
	sstCount       uint32
	vlogCount      uint32
	files          []string
	entriesWritten atomic.Uint64
)

const (
	mil float64 = 1e6
)

func init() {
	benchCmd.AddCommand(writeBenchCmd)
	writeBenchCmd.Flags().IntVarP(&wo.keySz, "key-size", "k", 32, "Size of key")
	writeBenchCmd.Flags().IntVar(&wo.valSz, "val-size", 128, "Size of value")
	writeBenchCmd.Flags().Float64VarP(&wo.numKeys, "keys-mil", "m", 10.0,
		"Number of keys to add in millions")
	writeBenchCmd.Flags().BoolVar(&wo.syncWrites, "sync", false,
		"If true, sync writes to disk.")
	writeBenchCmd.Flags().BoolVarP(&wo.force, "force-compact", "f", true,
		"Force compact level 0 on close.")
	writeBenchCmd.Flags().BoolVarP(&wo.sorted, "sorted", "s", false, "Write keys in sorted order.")
	writeBenchCmd.Flags().BoolVarP(&wo.showLogs, "verbose", "v", false, "Show Badger logs.")
	writeBenchCmd.Flags().Int64VarP(&wo.valueThreshold, "value-th", "t", 1<<10, "Value threshold")
	writeBenchCmd.Flags().IntVarP(&wo.numVersions, "num-version", "n", 1, "Number of versions to keep")
	writeBenchCmd.Flags().Int64Var(&wo.blockCacheSize, "block-cache-mb", 256,
		"Size of block cache in MB")
	writeBenchCmd.Flags().Int64Var(&wo.indexCacheSize, "index-cache-mb", 0,
		"Size of index cache in MB.")
	writeBenchCmd.Flags().Uint32Var(&wo.vlogMaxEntries, "vlog-maxe", 1000000, "Value log Max Entries")
	writeBenchCmd.Flags().StringVarP(&wo.encryptionKey, "encryption-key", "e", "",
		"If it is true, badger will encrypt all the data stored on the disk.")
	writeBenchCmd.Flags().BoolVar(&wo.loadBloomsOnOpen, "load-blooms", true,
		"Load Bloom filter on DB open.")
	writeBenchCmd.Flags().BoolVar(&wo.detectConflicts, "conficts", false,
		"If true, it badger will detect the conflicts")
	writeBenchCmd.Flags().BoolVar(&wo.zstdComp, "zstd", false,
		"If true, badger will use ZSTD mode. Otherwise, use default.")
	writeBenchCmd.Flags().BoolVar(&wo.showDir, "show-dir", false,
		"If true, the report will include the directory contents")
	writeBenchCmd.Flags().StringVar(&wo.dropAllPeriod, "dropall", "0s",
		"If set, run dropAll periodically over given duration.")
	writeBenchCmd.Flags().StringVar(&wo.dropPrefixPeriod, "drop-prefix", "0s",
		"If set, drop random prefixes periodically over given duration.")
	writeBenchCmd.Flags().StringVar(&wo.ttlDuration, "entry-ttl", "0s",
		"TTL duration in seconds for the entries, 0 means without TTL")
	writeBenchCmd.Flags().StringVarP(&wo.gcPeriod, "gc-every", "g", "0s", "GC Period.")
	writeBenchCmd.Flags().Float64VarP(&wo.gcDiscardRatio, "gc-ratio", "r", 0.5, "GC discard ratio.")
	writeBenchCmd.Flags().BoolVar(&wo.showKeysCount, "show-keys", false,
		"If true, the report will include the keys statistics")
}

func writeRandom(db *badger.DB, num uint64) error {
	value := make([]byte, wo.valSz)
	y.Check2(rand.Read(value))

	es := uint64(wo.keySz + wo.valSz) // entry size is keySz + valSz
	batch := db.NewManagedWriteBatch()

	ttlPeriod, errParse := time.ParseDuration(wo.ttlDuration)
	y.Check(errParse)

	for i := uint64(1); i <= num; i++ {
		key := make([]byte, wo.keySz)
		y.Check2(rand.Read(key))

		vsz := rand.Intn(wo.valSz) + 1
		e := badger.NewEntry(key, value[:vsz])

		if ttlPeriod != 0 {
			e.WithTTL(ttlPeriod)
		}
		err := batch.SetEntryAt(e, 1)
		for err == badger.ErrBlockedWrites {
			time.Sleep(time.Second)
			batch = db.NewManagedWriteBatch()
			err = batch.SetEntryAt(e, 1)
		}
		if err != nil {
			panic(err)
		}

		entriesWritten.Add(1)
		sizeWritten.Add(es)
	}
	return batch.Flush()
}

func readTest(db *badger.DB, dur time.Duration) {
	now := time.Now()
	keys, err := getSampleKeys(db, ro.sampleSize)
	if err != nil {
		panic(err)
	}
	fmt.Println("*********************************************************")
	fmt.Printf("Total Sampled Keys: %d, read in time: %s\n", len(keys), time.Since(now))
	fmt.Println("*********************************************************")

	if len(keys) == 0 {
		fmt.Println("DB is empty, hence returning")
		return
	}
	c := z.NewCloser(0)
	readStartTime := time.Now()
	for i := 0; i < numGoroutines; i++ {
		c.AddRunning(1)
		go readKeys(db, c, keys)
	}

	// also start printing stats
	c.AddRunning(1)
	go printReadStats(c, readStartTime)
	<-time.After(dur)
	c.SignalAndWait()
}

func writeSorted(db *badger.DB, num uint64) error {
	value := make([]byte, wo.valSz)
	y.Check2(rand.Read(value))
	es := 8 + wo.valSz // key size is 8 bytes and value size is valSz

	writer := db.NewStreamWriter()
	if err := writer.Prepare(); err != nil {
		return err
	}

	wg := &sync.WaitGroup{}
	writeCh := make(chan *z.Buffer, 3)
	writeRange := func(start, end uint64, streamId uint32) {
		// end is not included.
		defer wg.Done()
		kvBuf := z.NewBuffer(5<<20, "Benchmark.WriteSorted")
		var sz int
		for i := start; i < end; i++ {
			key := make([]byte, 8)
			binary.BigEndian.PutUint64(key, i)
			kv := &pb.KV{
				Key:      key,
				Value:    value,
				Version:  1,
				StreamId: streamId,
			}
			badger.KVToBuffer(kv, kvBuf)

			sz += es
			entriesWritten.Add(1)
			sizeWritten.Add(uint64(es))

			if sz >= 4<<20 { // 4 MB
				writeCh <- kvBuf
				kvBuf = z.NewBuffer(1<<20, "Benchmark.WriteSorted")
				sz = 0
			}
		}
		writeCh <- kvBuf
	}

	// Let's create some streams.
	width := num / 4
	streamID := uint32(0)
	for start := uint64(0); start < num; start += width {
		end := start + width
		if end > num {
			end = num
		}
		streamID++
		wg.Add(1)
		go writeRange(start, end, streamID)
	}
	go func() {
		wg.Wait()
		close(writeCh)
	}()
	log.Printf("Max StreamId used: %d. Width: %d\n", streamID, width)
	for kvs := range writeCh {
		if err := writer.Write(kvs); err != nil {
			panic(err)
		}
		y.Check(kvs.Release())
	}
	log.Println("DONE streaming. Flushing...")
	return writer.Flush()
}

func writeBench(cmd *cobra.Command, args []string) error {
	opt := badger.DefaultOptions(sstDir).
		WithValueDir(vlogDir).
		WithSyncWrites(wo.syncWrites).
		WithCompactL0OnClose(wo.force).
		WithValueThreshold(wo.valueThreshold).
		WithNumVersionsToKeep(wo.numVersions).
		WithBlockCacheSize(wo.blockCacheSize << 20).
		WithIndexCacheSize(wo.indexCacheSize << 20).
		WithValueLogMaxEntries(wo.vlogMaxEntries).
		WithEncryptionKey([]byte(wo.encryptionKey)).
		WithDetectConflicts(wo.detectConflicts).
		WithLoggingLevel(badger.INFO)
	if wo.zstdComp {
		opt = opt.WithCompression(options.ZSTD)
	}

	if !wo.showLogs {
		opt = opt.WithLogger(nil)
	}

	fmt.Printf("Opening badger with options = %+v\n", opt)
	db, err := badger.OpenManaged(opt)
	if err != nil {
		return err
	}
	defer func() {
		start := time.Now()
		err := db.Close()
		log.Printf("DB.Close. Error: %v. Time taken to close: %s", err, time.Since(start))
	}()

	fmt.Println("*********************************************************")
	fmt.Println("Starting to benchmark Writes")
	fmt.Println("*********************************************************")

	startTime = time.Now()
	num := uint64(wo.numKeys * mil)
	c := z.NewCloser(4)
	go reportStats(c, db)
	go dropAll(c, db)
	go dropPrefix(c, db)
	go runGC(c, db)

	if wo.sorted {
		err = writeSorted(db, num)
	} else {
		err = writeRandom(db, num)
	}

	c.SignalAndWait()
	fmt.Println(db.LevelsToString())
	return err
}

func showKeysStats(db *badger.DB) {
	var (
		internalKeyCount uint32
		invalidKeyCount  uint32
		validKeyCount    uint32
	)

	txn := db.NewTransactionAt(math.MaxUint64, false)
	defer txn.Discard()

	iopt := badger.DefaultIteratorOptions
	iopt.AllVersions = true
	iopt.InternalAccess = true
	it := txn.NewIterator(iopt)
	defer it.Close()

	for it.Rewind(); it.Valid(); it.Next() {
		i := it.Item()
		if bytes.HasPrefix(i.Key(), []byte("!badger!")) {
			internalKeyCount++
		}
		if i.IsDeletedOrExpired() {
			invalidKeyCount++
		} else {
			validKeyCount++
		}
	}
	fmt.Printf("Valid Keys: %d Invalid Keys: %d Internal Keys: %d\n",
		validKeyCount, invalidKeyCount, internalKeyCount)
}

func reportStats(c *z.Closer, db *badger.DB) {
	defer c.Done()

	t := time.NewTicker(time.Second)
	defer t.Stop()

	var count int
	for {
		select {
		case <-c.HasBeenClosed():
			return
		case <-t.C:
			count++
			if wo.showKeysCount {
				showKeysStats(db)
			}
			// fetch directory contents
			if wo.showDir {
				err := filepath.Walk(sstDir, func(path string, info os.FileInfo, err error) error {
					fileSize := humanize.IBytes(uint64(info.Size()))
					files = append(files, "[Content] "+path+" "+fileSize)
					if filepath.Ext(path) == ".vlog" {
						vlogCount++
					}
					if filepath.Ext(path) == ".sst" {
						sstCount++
					}
					return nil
				})
				if err != nil {
					log.Printf("Error while fetching directory. %v.", err)
				} else {
					fmt.Printf("[Content] Number of files:%d\n", len(files))
					for _, file := range files {
						fmt.Println(file)
					}
					fmt.Printf("SST Count: %d vlog Count: %d\n", sstCount, vlogCount)
				}
			}

			dur := time.Since(startTime)
			sz := sizeWritten.Load()
			entries := entriesWritten.Load()
			bytesRate := sz / uint64(dur.Seconds())
			entriesRate := entries / uint64(dur.Seconds())
			fmt.Printf("[WRITE] Time elapsed: %s, bytes written: %s, speed: %s/sec, "+
				"entries written: %d, speed: %d/sec, jemalloc: %s\n",
				y.FixedDuration(time.Since(startTime)),
				humanize.IBytes(sz), humanize.IBytes(bytesRate), entries, entriesRate,
				humanize.IBytes(uint64(z.NumAllocBytes())))

			if count%10 == 0 {
				fmt.Println(db.LevelsToString())
			}
		}
	}
}

func runGC(c *z.Closer, db *badger.DB) {
	defer c.Done()
	period, err := time.ParseDuration(wo.gcPeriod)
	y.Check(err)
	if period == 0 {
		return
	}

	t := time.NewTicker(period)
	defer t.Stop()
	for {
		select {
		case <-c.HasBeenClosed():
			return
		case <-t.C:
			if err := db.RunValueLogGC(wo.gcDiscardRatio); err == nil {
				gcSuccess.Add(1)
			} else {
				log.Printf("[GC] Failed due to following err %v", err)
			}
		}
	}
}

func dropAll(c *z.Closer, db *badger.DB) {
	defer c.Done()
	dropPeriod, err := time.ParseDuration(wo.dropAllPeriod)
	y.Check(err)
	if dropPeriod == 0 {
		return
	}

	t := time.NewTicker(dropPeriod)
	defer t.Stop()
	for {
		select {
		case <-c.HasBeenClosed():
			return
		case <-t.C:
			fmt.Println("[DropAll] Started")
			err := db.DropAll()
			for err == badger.ErrBlockedWrites {
				err = db.DropAll()
				time.Sleep(time.Millisecond * 300)
			}

			if err != nil {
				fmt.Println("[DropAll] Failed")
			} else {
				fmt.Println("[DropAll] Successful")
			}
		}
	}
}

func dropPrefix(c *z.Closer, db *badger.DB) {
	defer c.Done()
	dropPeriod, err := time.ParseDuration(wo.dropPrefixPeriod)
	y.Check(err)
	if dropPeriod == 0 {
		return
	}

	t := time.NewTicker(dropPeriod)
	defer t.Stop()
	for {
		select {
		case <-c.HasBeenClosed():
			return
		case <-t.C:
			fmt.Println("[DropPrefix] Started")
			prefix := make([]byte, 1+int(float64(wo.keySz)*0.1))
			y.Check2(rand.Read(prefix))
			err = db.DropPrefix(prefix)

			if err != nil {
				panic(err)
			} else {
				fmt.Println("[DropPrefix] Successful")
			}
		}
	}
}

func printReadStats(c *z.Closer, startTime time.Time) {
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
			fmt.Printf("[READ] Time elapsed: %s, bytes read: %s, speed: %s/sec, "+
				"entries read: %d, speed: %d/sec\n", y.FixedDuration(time.Since(startTime)),
				humanize.IBytes(sz), humanize.IBytes(bytesRate), entries, entriesRate)
		}
	}
}
