/*
 * Copyright 2018 Dgraph Labs, Inc. and Contributors
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

package badger

import (
	"bytes"
	"context"
	"encoding/binary"
	"flag"
	"fmt"
	"log"
	"math"
	"math/rand"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/pb"
	"github.com/dgraph-io/badger/v4/table"
	"github.com/dgraph-io/badger/v4/y"
	"github.com/dgraph-io/ristretto/z"
)

func TestTruncateVlogWithClose(t *testing.T) {
	key := func(i int) []byte {
		return []byte(fmt.Sprintf("%d%10d", i, i))
	}
	data := func(l int) []byte {
		m := make([]byte, l)
		_, err := rand.Read(m)
		require.NoError(t, err)
		return m
	}

	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	opt := getTestOptions(dir)
	opt.SyncWrites = true
	opt.ValueThreshold = 1 // Force all reads from value log.

	db, err := Open(opt)
	require.NoError(t, err)

	err = db.Update(func(txn *Txn) error {
		return txn.SetEntry(NewEntry(key(0), data(4055)))
	})
	require.NoError(t, err)

	// Close the DB.
	require.NoError(t, db.Close())
	// We start value logs at 1.
	require.NoError(t, os.Truncate(filepath.Join(dir, "000001.vlog"), 4090))

	// Reopen and write some new data.
	db, err = Open(opt)
	require.NoError(t, err)
	for i := 0; i < 32; i++ {
		err := db.Update(func(txn *Txn) error {
			return txn.SetEntry(NewEntry(key(i), data(10)))
		})
		require.NoError(t, err)
	}

	// Read it back to ensure that we can read it now.
	for i := 0; i < 32; i++ {
		err := db.View(func(txn *Txn) error {
			item, err := txn.Get(key(i))
			require.NoError(t, err)
			val := getItemValue(t, item)
			require.Equal(t, 10, len(val))
			return nil
		})
		require.NoError(t, err)
	}
	require.NoError(t, db.Close())

	// Reopen and read the data again.
	db, err = Open(opt)
	require.NoError(t, err)
	for i := 0; i < 32; i++ {
		err := db.View(func(txn *Txn) error {
			item, err := txn.Get(key(i))
			require.NoError(t, err, "key: %s", key(i))
			val := getItemValue(t, item)
			require.Equal(t, 10, len(val))
			return nil
		})
		require.NoError(t, err)
	}
	require.NoError(t, db.Close())
}

var manual = flag.Bool("manual", false, "Set when manually running some tests.")

// Badger dir to be used for performing db.Open benchmark.
var benchDir = flag.String("benchdir", "", "Set when running db.Open benchmark")

// The following 3 TruncateVlogNoClose tests should be run one after another.
// None of these close the DB, simulating a crash. They should be run with a
// script, which truncates the value log to 4090, lining up with the end of the
// first entry in the txn. At <4090, it would cause the entry to be truncated
// immediately, at >4090, same thing.
func TestTruncateVlogNoClose(t *testing.T) {
	if !*manual {
		t.Skip("Skipping test meant to be run manually.")
		return
	}
	dir := "p"
	opts := getTestOptions(dir)
	opts.SyncWrites = true

	kv, err := Open(opts)
	require.NoError(t, err)
	key := func(i int) string {
		return fmt.Sprintf("%d%10d", i, i)
	}
	data := fmt.Sprintf("%4055d", 1)
	err = kv.Update(func(txn *Txn) error {
		return txn.SetEntry(NewEntry([]byte(key(0)), []byte(data)))
	})
	require.NoError(t, err)
}
func TestTruncateVlogNoClose2(t *testing.T) {
	if !*manual {
		t.Skip("Skipping test meant to be run manually.")
		return
	}
	dir := "p"
	opts := getTestOptions(dir)
	opts.SyncWrites = true

	kv, err := Open(opts)
	require.NoError(t, err)
	key := func(i int) string {
		return fmt.Sprintf("%d%10d", i, i)
	}
	data := fmt.Sprintf("%10d", 1)
	for i := 32; i < 64; i++ {
		err := kv.Update(func(txn *Txn) error {
			return txn.SetEntry(NewEntry([]byte(key(i)), []byte(data)))
		})
		require.NoError(t, err)
	}
	for i := 32; i < 64; i++ {
		require.NoError(t, kv.View(func(txn *Txn) error {
			item, err := txn.Get([]byte(key(i)))
			require.NoError(t, err)
			val := getItemValue(t, item)
			require.NotNil(t, val)
			require.True(t, len(val) > 0)
			return nil
		}))
	}
}
func TestTruncateVlogNoClose3(t *testing.T) {
	if !*manual {
		t.Skip("Skipping test meant to be run manually.")
		return
	}
	fmt.Print("Running")
	dir := "p"
	opts := getTestOptions(dir)
	opts.SyncWrites = true

	kv, err := Open(opts)
	require.NoError(t, err)
	key := func(i int) string {
		return fmt.Sprintf("%d%10d", i, i)
	}
	for i := 32; i < 64; i++ {
		require.NoError(t, kv.View(func(txn *Txn) error {
			item, err := txn.Get([]byte(key(i)))
			require.NoError(t, err)
			val := getItemValue(t, item)
			require.NotNil(t, val)
			require.True(t, len(val) > 0)
			return nil
		}))
	}
}

func TestBigKeyValuePairs(t *testing.T) {
	// This test takes too much memory. So, run separately.
	if !*manual {
		t.Skip("Skipping test meant to be run manually.")
		return
	}

	// Passing an empty directory since it will be filled by runBadgerTest.
	opts := DefaultOptions("").
		WithBaseTableSize(1 << 20).
		WithValueLogMaxEntries(64)
	runBadgerTest(t, &opts, func(t *testing.T, db *DB) {
		bigK := make([]byte, 65001)
		bigV := make([]byte, db.opt.ValueLogFileSize+1)
		small := make([]byte, 65000)

		txn := db.NewTransaction(true)
		require.Regexp(t, regexp.MustCompile("Key.*exceeded"), txn.SetEntry(NewEntry(bigK, small)))
		require.Regexp(t, regexp.MustCompile("Value.*exceeded"),
			txn.SetEntry(NewEntry(small, bigV)))

		require.NoError(t, txn.SetEntry(NewEntry(small, small)))
		require.Regexp(t, regexp.MustCompile("Key.*exceeded"), txn.SetEntry(NewEntry(bigK, bigV)))

		require.NoError(t, db.View(func(txn *Txn) error {
			_, err := txn.Get(small)
			require.Equal(t, ErrKeyNotFound, err)
			return nil
		}))

		// Now run a longer test, which involves value log GC.
		data := fmt.Sprintf("%100d", 1)
		key := func(i int) string {
			return fmt.Sprintf("%65000d", i)
		}

		saveByKey := func(key string, value []byte) error {
			return db.Update(func(txn *Txn) error {
				return txn.SetEntry(NewEntry([]byte(key), value))
			})
		}

		getByKey := func(key string) error {
			return db.View(func(txn *Txn) error {
				item, err := txn.Get([]byte(key))
				if err != nil {
					return err
				}
				return item.Value(func(val []byte) error {
					if len(val) == 0 {
						log.Fatalf("key not found %q", len(key))
					}
					return nil
				})
			})
		}

		for i := 0; i < 32; i++ {
			if i < 30 {
				require.NoError(t, saveByKey(key(i), []byte(data)))
			} else {
				require.NoError(t, saveByKey(key(i), []byte(fmt.Sprintf("%100d", i))))
			}
		}

		for j := 0; j < 5; j++ {
			for i := 0; i < 32; i++ {
				if i < 30 {
					require.NoError(t, saveByKey(key(i), []byte(data)))
				} else {
					require.NoError(t, saveByKey(key(i), []byte(fmt.Sprintf("%100d", i))))
				}
			}
		}

		for i := 0; i < 32; i++ {
			require.NoError(t, getByKey(key(i)))
		}

		var loops int
		var err error
		for err == nil {
			err = db.RunValueLogGC(0.5)
			require.NotRegexp(t, regexp.MustCompile("truncate"), err)
			loops++
		}
		t.Logf("Ran value log GC %d times. Last error: %v\n", loops, err)
	})
}

// The following test checks for issue #585.
func TestPushValueLogLimit(t *testing.T) {
	// This test takes too much memory. So, run separately.
	if !*manual {
		t.Skip("Skipping test meant to be run manually.")
		return
	}

	// Passing an empty directory since it will be filled by runBadgerTest.
	opt := DefaultOptions("").
		WithValueLogMaxEntries(64).
		WithValueLogFileSize(2<<30 - 1)
	runBadgerTest(t, &opt, func(t *testing.T, db *DB) {
		data := []byte(fmt.Sprintf("%30d", 1))
		key := func(i int) string {
			return fmt.Sprintf("%100d", i)
		}

		for i := 0; i < 32; i++ {
			if i == 4 {
				v := make([]byte, math.MaxInt32)
				err := db.Update(func(txn *Txn) error {
					return txn.SetEntry(NewEntry([]byte(key(i)), v))
				})
				require.NoError(t, err)
			} else {
				err := db.Update(func(txn *Txn) error {
					return txn.SetEntry(NewEntry([]byte(key(i)), data))
				})
				require.NoError(t, err)
			}
		}

		for i := 0; i < 32; i++ {
			err := db.View(func(txn *Txn) error {
				item, err := txn.Get([]byte(key(i)))
				require.NoError(t, err, "Getting key: %s", key(i))
				err = item.Value(func(v []byte) error {
					_ = v
					return nil
				})
				require.NoError(t, err, "Getting value: %s", key(i))
				return nil
			})
			require.NoError(t, err)
		}
	})
}

// The following benchmark test is supposed to be run against a badger directory with some data.
// Use badger fill to create data if it doesn't exist.
func BenchmarkDBOpen(b *testing.B) {
	if *benchDir == "" {
		b.Skip("Please set -benchdir to badger directory")
	}
	dir := *benchDir
	// Passing an empty directory since it will be filled by runBadgerTest.
	opt := DefaultOptions(dir).
		WithReadOnly(true)
	for i := 0; i < b.N; i++ {
		db, err := Open(opt)
		require.NoError(b, err)
		require.NoError(b, db.Close())
	}
}

// Test for values of size uint32.
func TestBigValues(t *testing.T) {
	if !*manual {
		t.Skip("Skipping test meant to be run manually.")
		return
	}
	opts := DefaultOptions("").
		WithValueThreshold(1 << 20).
		WithValueLogMaxEntries(100)
	test := func(t *testing.T, db *DB) {
		keyCount := 1000

		data := bytes.Repeat([]byte("a"), (1 << 20)) // Valuesize 1 MB.
		key := func(i int) string {
			return fmt.Sprintf("%65000d", i)
		}

		saveByKey := func(key string, value []byte) error {
			return db.Update(func(txn *Txn) error {
				return txn.SetEntry(NewEntry([]byte(key), value))
			})
		}

		getByKey := func(key string) error {
			return db.View(func(txn *Txn) error {
				item, err := txn.Get([]byte(key))
				if err != nil {
					return err
				}
				return item.Value(func(val []byte) error {
					if len(val) == 0 || len(val) != len(data) || !bytes.Equal(val, data) {
						log.Fatalf("key not found %q", len(key))
					}
					return nil
				})
			})
		}

		for i := 0; i < keyCount; i++ {
			require.NoError(t, saveByKey(key(i), data))
		}

		for i := 0; i < keyCount; i++ {
			require.NoError(t, getByKey(key(i)))
		}
	}
	t.Run("disk mode", func(t *testing.T) {
		runBadgerTest(t, &opts, func(t *testing.T, db *DB) {
			test(t, db)
		})
	})
	t.Run("InMemory mode", func(t *testing.T) {
		opts.InMemory = true
		opts.Dir = ""
		opts.ValueDir = ""
		db, err := Open(opts)
		require.NoError(t, err)
		test(t, db)
		require.NoError(t, db.Close())
	})
}

// This test is for compaction file picking testing. We are creating db with two levels. We have 10
// tables on level 3 and 3 tables on level 2. Tables on level 2 have overlap with 2, 4, 3 tables on
// level 3.
func TestCompactionFilePicking(t *testing.T) {
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	db, err := Open(DefaultOptions(dir))
	require.NoError(t, err, "error while opening db")
	defer func() {
		require.NoError(t, db.Close())
	}()

	l3 := db.lc.levels[3]
	for i := 1; i <= 10; i++ {
		// Each table has difference of 1 between smallest and largest key.
		tab := createTableWithRange(t, db, 2*i-1, 2*i)
		addToManifest(t, db, tab, 3)
		require.NoError(t, l3.replaceTables([]*table.Table{}, []*table.Table{tab}))
	}

	l2 := db.lc.levels[2]
	// First table has keys 1 and 4.
	tab := createTableWithRange(t, db, 1, 4)
	addToManifest(t, db, tab, 2)
	require.NoError(t, l2.replaceTables([]*table.Table{}, []*table.Table{tab}))

	// Second table has keys 5 and 12.
	tab = createTableWithRange(t, db, 5, 12)
	addToManifest(t, db, tab, 2)
	require.NoError(t, l2.replaceTables([]*table.Table{}, []*table.Table{tab}))

	// Third table has keys 13 and 18.
	tab = createTableWithRange(t, db, 13, 18)
	addToManifest(t, db, tab, 2)
	require.NoError(t, l2.replaceTables([]*table.Table{}, []*table.Table{tab}))

	cdef := &compactDef{
		thisLevel: db.lc.levels[2],
		nextLevel: db.lc.levels[3],
	}

	tables := db.lc.levels[2].tables
	db.lc.sortByHeuristic(tables, cdef)

	var expKey [8]byte
	// First table should be with smallest and biggest keys as 1 and 4 which
	// has the lowest version.
	binary.BigEndian.PutUint64(expKey[:], uint64(1))
	require.Equal(t, expKey[:], y.ParseKey(tables[0].Smallest()))
	binary.BigEndian.PutUint64(expKey[:], uint64(4))
	require.Equal(t, expKey[:], y.ParseKey(tables[0].Biggest()))

	// Second table should be with smallest and biggest keys as 13 and 18
	// which has the second lowest version.
	binary.BigEndian.PutUint64(expKey[:], uint64(13))
	require.Equal(t, expKey[:], y.ParseKey(tables[2].Smallest()))
	binary.BigEndian.PutUint64(expKey[:], uint64(18))
	require.Equal(t, expKey[:], y.ParseKey(tables[2].Biggest()))

	// Third table should be with smallest and biggest keys as 5 and 12 which
	// has the maximum version.
	binary.BigEndian.PutUint64(expKey[:], uint64(5))
	require.Equal(t, expKey[:], y.ParseKey(tables[1].Smallest()))
	binary.BigEndian.PutUint64(expKey[:], uint64(12))
	require.Equal(t, expKey[:], y.ParseKey(tables[1].Biggest()))
}

// addToManifest function is used in TestCompactionFilePicking. It adds table to db manifest.
func addToManifest(t *testing.T, db *DB, tab *table.Table, level uint32) {
	change := &pb.ManifestChange{
		Id:          tab.ID(),
		Op:          pb.ManifestChange_CREATE,
		Level:       level,
		Compression: uint32(tab.CompressionType()),
	}
	require.NoError(t, db.manifest.addChanges([]*pb.ManifestChange{change}),
		"unable to add to manifest")
}

// createTableWithRange function is used in TestCompactionFilePicking. It creates
// a table with key starting from start and ending with end.
func createTableWithRange(t *testing.T, db *DB, start, end int) *table.Table {
	bopts := buildTableOptions(db)
	b := table.NewTableBuilder(bopts)
	defer b.Close()
	nums := []int{start, end}
	for _, i := range nums {
		key := make([]byte, 8)
		binary.BigEndian.PutUint64(key[:], uint64(i))
		key = y.KeyWithTs(key, uint64(0))
		val := y.ValueStruct{Value: []byte(fmt.Sprintf("%d", i))}
		b.Add(key, val, 0)
	}

	fileID := db.lc.reserveFileID()
	tab, err := table.CreateTable(table.NewFilename(fileID, db.opt.Dir), b)
	require.NoError(t, err)
	return tab
}

func TestReadSameVlog(t *testing.T) {
	key := func(i int) []byte {
		return []byte(fmt.Sprintf("%d%10d", i, i))
	}
	testReadingSameKey := func(t *testing.T, db *DB) {
		// Forcing to read all values from vlog.
		for i := 0; i < 50; i++ {
			err := db.Update(func(txn *Txn) error {
				return txn.Set(key(i), key(i))
			})
			require.NoError(t, err)
		}
		// reading it again several times
		for i := 0; i < 50; i++ {
			for j := 0; j < 10; j++ {
				err := db.View(func(txn *Txn) error {
					item, err := txn.Get(key(i))
					require.NoError(t, err)
					require.Equal(t, key(i), getItemValue(t, item))
					return nil
				})
				require.NoError(t, err)
			}
		}
	}

	t.Run("Test Read Again Plain Text", func(t *testing.T) {
		opt := getTestOptions("")
		// Forcing to read from vlog
		opt.ValueThreshold = 1
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			testReadingSameKey(t, db)
		})

	})

	t.Run("Test Read Again Encryption", func(t *testing.T) {
		opt := getTestOptions("")
		opt.ValueThreshold = 1
		// Generate encryption key.
		eKey := make([]byte, 32)
		_, err := rand.Read(eKey)
		require.NoError(t, err)
		opt.EncryptionKey = eKey
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			testReadingSameKey(t, db)
		})
	})
}

// The test ensures we don't lose data when badger is opened with KeepL0InMemory and GC is being
// done.
func TestL0GCBug(t *testing.T) {
	t.Skipf("TestL0GCBug is DISABLED. TODO(ibrahim): Do we need this?")

	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	// Do not change any of the options below unless it's necessary.
	opts := getTestOptions(dir)
	opts.NumLevelZeroTables = 50
	opts.NumLevelZeroTablesStall = 51
	opts.ValueLogMaxEntries = 2
	opts.ValueThreshold = 2
	// Setting LoadingMode to mmap seems to cause segmentation fault while closing DB.

	db1, err := Open(opts)
	require.NoError(t, err)
	key := func(i int) []byte {
		return []byte(fmt.Sprintf("%10d", i))
	}
	val := []byte{1, 1, 1, 1, 1, 1, 1, 1}
	// Insert 100 entries. This will create about 50*3 vlog files and 6 SST files.
	for i := 0; i < 3; i++ {
		for j := 0; j < 100; j++ {
			err = db1.Update(func(txn *Txn) error {
				return txn.SetEntry(NewEntry(key(j), val))
			})
			require.NoError(t, err)
		}
	}
	// Run value log GC multiple times. This would ensure at least
	// one value log file is garbage collected.
	success := 0
	for i := 0; i < 10; i++ {
		err := db1.RunValueLogGC(0.01)
		if err == nil {
			success++
		}
		if err != nil && err != ErrNoRewrite {
			t.Fatalf(err.Error())
		}
	}
	// Ensure alteast one GC call was successful.
	require.NotZero(t, success)
	// CheckKeys reads all the keys previously stored.
	checkKeys := func(db *DB) {
		for i := 0; i < 100; i++ {
			err := db.View(func(txn *Txn) error {
				item, err := txn.Get(key(i))
				require.NoError(t, err)
				val1 := getItemValue(t, item)
				require.Equal(t, val, val1)
				return nil
			})
			require.NoError(t, err)
		}
	}

	checkKeys(db1)
	// Simulate a crash by not closing db1 but releasing the locks.
	if db1.dirLockGuard != nil {
		require.NoError(t, db1.dirLockGuard.release())
		db1.dirLockGuard = nil
	}
	if db1.valueDirGuard != nil {
		require.NoError(t, db1.valueDirGuard.release())
		db1.valueDirGuard = nil
	}
	require.NoError(t, db1.Close())

	db2, err := Open(opts)
	require.NoError(t, err)

	// Ensure we still have all the keys.
	checkKeys(db2)
	require.NoError(t, db2.Close())
}

// Regression test for https://github.com/dgraph-io/badger/issues/1126
//
// The test has 3 steps
// Step 1 - Create badger data. It is necessary that the value size is
//
//	greater than valuethreshold. The value log file size after
//	this step is around 170 bytes.
//
// Step 2 - Re-open the same badger and simulate a crash. The value log file
//
//	size after this crash is around 2 GB (we increase the file size to mmap it).
//
// Step 3 - Re-open the same badger. We should be able to read all the data
//
//	inserted in the first step.
func TestWindowsDataLoss(t *testing.T) {
	if runtime.GOOS != "windows" {
		t.Skip("The test is only for Windows.")
	}

	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	opt := DefaultOptions(dir).WithSyncWrites(true)
	opt.ValueThreshold = 32

	db, err := Open(opt)
	require.NoError(t, err)
	keyCount := 20
	var keyList [][]byte // Stores all the keys generated.
	for i := 0; i < keyCount; i++ {
		// It is important that we create different transactions for each request.
		err := db.Update(func(txn *Txn) error {
			key := []byte(fmt.Sprintf("%d", i))
			v := []byte("barValuebarValuebarValuebarValuebarValue")
			require.Greater(t, len(v), db.valueThreshold())

			//32 bytes length and now it's not working
			err := txn.Set(key, v)
			require.NoError(t, err)
			keyList = append(keyList, key)
			return nil
		})
		require.NoError(t, err)
	}
	require.NoError(t, db.Close())

	db, err = Open(opt)
	require.NoError(t, err)
	// Return after reading one entry. We're simulating a crash.
	// Simulate a crash by not closing db but releasing the locks.
	if db.dirLockGuard != nil {
		require.NoError(t, db.dirLockGuard.release())
	}
	if db.valueDirGuard != nil {
		require.NoError(t, db.valueDirGuard.release())
	}
	// Don't use vlog.Close here. We don't want to fix the file size. Only un-mmap
	// the data so that we can truncate the file durning the next vlog.Open.
	require.NoError(t, z.Munmap(db.vlog.filesMap[db.vlog.maxFid].Data))
	for _, f := range db.vlog.filesMap {
		require.NoError(t, f.Fd.Close())
	}
	require.NoError(t, db.registry.Close())
	require.NoError(t, db.manifest.close())
	require.NoError(t, db.lc.close())

	db, err = Open(opt)
	require.NoError(t, err)
	defer db.Close()

	txn := db.NewTransaction(false)
	defer txn.Discard()
	it := txn.NewIterator(DefaultIteratorOptions)
	defer it.Close()

	var result [][]byte // stores all the keys read from the db.
	for it.Rewind(); it.Valid(); it.Next() {
		item := it.Item()
		k := item.Key()
		err := item.Value(func(v []byte) error {
			_ = v
			return nil
		})
		require.NoError(t, err)
		result = append(result, k)
	}
	require.ElementsMatch(t, keyList, result)
}

func TestDropPrefixWithNoData(t *testing.T) {
	runBadgerTest(t, nil, func(t *testing.T, db *DB) {
		val := []byte("value")
		require.NoError(t, db.Update(func(txn *Txn) error {
			require.NoError(t, txn.Set([]byte("aaa"), val))
			require.NoError(t, txn.Set([]byte("aab"), val))
			require.NoError(t, txn.Set([]byte("aba"), val))
			require.NoError(t, txn.Set([]byte("aca"), val))
			return nil
		}))

		// If we drop prefix, we flush the memtables and create a new mutable memtable. Hence, the
		// nextMemFid increases by 1. But if there does not exist any data for the prefixes, we
		// don't do that.
		memFid := db.nextMemFid
		prefixes := [][]byte{[]byte("bbb")}
		require.NoError(t, db.DropPrefix(prefixes...))
		require.Equal(t, memFid, db.nextMemFid)
		prefixes = [][]byte{[]byte("aba"), []byte("bbb")}
		require.NoError(t, db.DropPrefix(prefixes...))
		require.Equal(t, memFid+1, db.nextMemFid)
	})
}

func TestDropAllDropPrefix(t *testing.T) {
	key := func(i int) []byte {
		return []byte(fmt.Sprintf("%10d", i))
	}
	val := func(i int) []byte {
		return []byte(fmt.Sprintf("%128d", i))
	}
	runBadgerTest(t, nil, func(t *testing.T, db *DB) {
		wb := db.NewWriteBatch()
		defer wb.Cancel()

		N := 50000

		for i := 0; i < N; i++ {
			require.NoError(t, wb.Set(key(i), val(i)))
		}
		require.NoError(t, wb.Flush())

		var wg sync.WaitGroup
		wg.Add(3)
		go func() {
			defer wg.Done()
			err := db.DropPrefix([]byte("000"))
			for err == ErrBlockedWrites {
				err = db.DropPrefix([]byte("000"))
				time.Sleep(time.Millisecond * 500)
			}
			require.NoError(t, err)
		}()
		go func() {
			defer wg.Done()
			err := db.DropPrefix([]byte("111"))
			for err == ErrBlockedWrites {
				err = db.DropPrefix([]byte("111"))
				time.Sleep(time.Millisecond * 500)
			}
			require.NoError(t, err)
		}()
		go func() {
			time.Sleep(time.Millisecond) // Let drop prefix run first.
			defer wg.Done()
			err := db.DropAll()
			for err == ErrBlockedWrites {
				err = db.DropAll()
				time.Sleep(time.Millisecond * 300)
			}
			require.NoError(t, err)
		}()
		wg.Wait()
	})
}

func TestIsClosed(t *testing.T) {
	test := func(inMemory bool) {
		opt := DefaultOptions("")
		if inMemory {
			opt.InMemory = true
		} else {
			dir, err := os.MkdirTemp("", "badger-test")
			require.NoError(t, err)
			defer removeDir(dir)

			opt.Dir = dir
			opt.ValueDir = dir
		}

		db, err := Open(opt)
		require.NoError(t, err)
		require.False(t, db.IsClosed())
		require.NoError(t, db.Close())
		require.True(t, db.IsClosed())
	}

	t.Run("normal", func(t *testing.T) {
		test(false)
	})
	t.Run("in-memory", func(t *testing.T) {
		test(true)
	})

}

// This test is failing currently because we're returning version+1 from MaxVersion()
func TestMaxVersion(t *testing.T) {
	N := 10000
	key := func(i int) []byte {
		return []byte(fmt.Sprintf("%d%10d", i, i))
	}
	t.Run("normal", func(t *testing.T) {
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			// This will create commits from 1 to N.
			for i := 0; i < N; i++ {
				txnSet(t, db, key(i), nil, 0)
			}
			ver := db.MaxVersion()
			require.Equal(t, N, int(ver))
		})
	})
	t.Run("multiple versions", func(t *testing.T) {
		dir, err := os.MkdirTemp("", "badger-test")
		require.NoError(t, err)
		defer removeDir(dir)

		opt := getTestOptions(dir)
		opt.NumVersionsToKeep = 100
		db, err := OpenManaged(opt)
		require.NoError(t, err)

		wb := db.NewManagedWriteBatch()
		defer wb.Cancel()

		k := make([]byte, 100)
		rand.Read(k)
		// Create multiple version of the same key.
		for i := 1; i <= N; i++ {
			require.NoError(t, wb.SetEntryAt(&Entry{Key: k}, uint64(i)))
		}
		require.NoError(t, wb.Flush())

		ver := db.MaxVersion()
		require.Equal(t, N, int(ver))

		require.NoError(t, db.Close())
	})
	t.Run("Managed mode", func(t *testing.T) {
		dir, err := os.MkdirTemp("", "badger-test")
		require.NoError(t, err)
		defer removeDir(dir)

		opt := getTestOptions(dir)
		db, err := OpenManaged(opt)
		require.NoError(t, err)

		wb := db.NewManagedWriteBatch()
		defer wb.Cancel()

		// This will create commits from 1 to N.
		for i := 1; i <= N; i++ {
			require.NoError(t, wb.SetEntryAt(&Entry{Key: []byte(fmt.Sprintf("%d", i))}, uint64(i)))
		}
		require.NoError(t, wb.Flush())

		ver := db.MaxVersion()
		require.NoError(t, err)
		require.Equal(t, N, int(ver))

		require.NoError(t, db.Close())
	})
}

func TestTxnReadTs(t *testing.T) {
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	opt := DefaultOptions(dir)
	db, err := Open(opt)
	require.NoError(t, err)
	require.Equal(t, 0, int(db.orc.readTs()))

	txnSet(t, db, []byte("foo"), nil, 0)
	require.Equal(t, 1, int(db.orc.readTs()))
	require.NoError(t, db.Close())
	require.Equal(t, 1, int(db.orc.readTs()))

	db, err = Open(opt)
	require.NoError(t, err)
	require.Equal(t, 1, int(db.orc.readTs()))
}

// This tests failed for stream writer with jemalloc and compression enabled.
func TestKeyCount(t *testing.T) {
	if !*manual {
		t.Skip("Skipping test meant to be run manually.")
		return
	}

	writeSorted := func(db *DB, num uint64) {
		valSz := 128
		value := make([]byte, valSz)
		y.Check2(rand.Read(value))
		es := 8 + valSz // key size is 8 bytes and value size is valSz

		writer := db.NewStreamWriter()
		require.NoError(t, writer.Prepare())

		wg := &sync.WaitGroup{}
		writeCh := make(chan *pb.KVList, 3)
		writeRange := func(start, end uint64, streamId uint32) {
			// end is not included.
			defer wg.Done()
			kvs := &pb.KVList{}
			var sz int
			for i := start; i < end; i++ {
				key := make([]byte, 8)
				binary.BigEndian.PutUint64(key, i)
				kvs.Kv = append(kvs.Kv, &pb.KV{
					Key:      key,
					Value:    value,
					Version:  1,
					StreamId: streamId,
				})

				sz += es

				if sz >= 4<<20 { // 4 MB
					writeCh <- kvs
					kvs = &pb.KVList{}
					sz = 0
				}
			}
			writeCh <- kvs
		}

		// Let's create some streams.
		width := num / 16
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

		write := func(kvs *pb.KVList) error {
			buf := z.NewBuffer(1<<20, "test")
			defer func() { require.NoError(t, buf.Release()) }()

			for _, kv := range kvs.Kv {
				KVToBuffer(kv, buf)
			}
			require.NoError(t, writer.Write(buf))
			return nil
		}

		for kvs := range writeCh {
			require.NoError(t, write(kvs))
		}
		require.NoError(t, writer.Flush())
	}

	N := uint64(10 * 1e6) // 10 million entries
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)
	opt := DefaultOptions(dir).
		WithBlockCacheSize(100 << 20).
		WithCompression(options.ZSTD)

	db, err := Open(opt)
	y.Check(err)
	defer db.Close()
	writeSorted(db, N)
	require.NoError(t, db.Close())
	t.Logf("Writing DONE\n")

	// Read the db
	db2, err := Open(DefaultOptions(dir))
	y.Check(err)
	defer db.Close()
	lastKey := -1
	count := 0

	streams := make(map[uint32]int)
	stream := db2.NewStream()
	stream.Send = func(buf *z.Buffer) error {
		list, err := BufferToKVList(buf)
		if err != nil {
			return err
		}
		for _, kv := range list.Kv {
			last := streams[kv.StreamId]
			key := binary.BigEndian.Uint64(kv.Key)
			// The following should happen as we're writing sorted data.
			if last > 0 {
				require.Equalf(t, last+1, int(key), "Expected key: %d, Found Key: %d", lastKey+1, int(key))
			}
			streams[kv.StreamId] = int(key)
		}
		count += len(list.Kv)
		return nil
	}
	require.NoError(t, stream.Orchestrate(context.Background()))
	require.Equal(t, N, uint64(count))
}

func TestAssertValueLogIsNotWrittenToOnStartup(t *testing.T) {
	opt := DefaultOptions("").WithValueLogFileSize(1 << 20).WithValueThreshold(1 << 4)

	dir, err := os.MkdirTemp(".", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	openDb := func(readonly bool) *DB {
		opts := &opt
		opts.Dir = dir
		opts.ValueDir = dir
		if readonly {
			opts.ReadOnly = true
		}

		if opts.InMemory {
			opts.Dir = ""
			opts.ValueDir = ""
		}
		db, err := Open(*opts)
		require.NoError(t, err)

		return db
	}

	key := func(i int) string {
		return fmt.Sprintf("key%100d", i)
	}

	assertOnLoadDb := func(db *DB) uint32 {
		data := []byte(fmt.Sprintf("value%100d", 1))
		for i := 0; i < 20; i++ {
			err := db.Update(func(txn *Txn) error {
				return txn.SetEntry(NewEntry([]byte(key(i)), data))
			})
			require.NoError(t, err)
		}
		return db.vlog.maxFid
	}

	latestVLogFileSize := func(db *DB, vLogId uint32) uint32 {
		return db.vlog.filesMap[vLogId].size.Load()
	}

	assertOnReadDb := func(db *DB) {
		for i := 0; i < 20; i++ {
			err := db.View(func(txn *Txn) error {
				item, err := txn.Get([]byte(key(i)))
				require.NoError(t, err, "Getting key: %s", key(i))
				err = item.Value(func(v []byte) error {
					_ = v
					return nil
				})
				require.NoError(t, err, "Getting value for the key: %s", key(i))
				return nil
			})
			require.NoError(t, err)
		}
	}

	db := openDb(false)
	vLogFileSize := latestVLogFileSize(db, assertOnLoadDb(db))
	assertOnReadDb(db)

	require.NoError(t, db.Sync())
	require.NoError(t, db.Close())

	db = openDb(true)
	defer func() {
		require.NoError(t, db.Close())
	}()

	assertOnReadDb(db)
	require.Equal(t, latestVLogFileSize(db, db.vlog.maxFid), vLogFileSize)
}
