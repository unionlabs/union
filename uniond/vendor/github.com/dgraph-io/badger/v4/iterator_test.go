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
	"fmt"
	"math"
	"math/rand"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/table"
	"github.com/dgraph-io/badger/v4/y"
)

type tableMock struct {
	left, right []byte
}

func (tm *tableMock) Smallest() []byte             { return tm.left }
func (tm *tableMock) Biggest() []byte              { return tm.right }
func (tm *tableMock) DoesNotHave(hash uint32) bool { return false }
func (tm *tableMock) MaxVersion() uint64           { return math.MaxUint64 }

func TestPickTables(t *testing.T) {
	opt := DefaultIteratorOptions

	within := func(prefix, left, right []byte) {
		opt.Prefix = prefix
		// PickTable expects smallest and biggest to contain timestamps.
		tm := &tableMock{left: y.KeyWithTs(left, 1), right: y.KeyWithTs(right, 1)}
		require.True(t, opt.pickTable(tm), "within failed for %b %b %b\n", prefix, left, right)
	}
	outside := func(prefix, left, right string) {
		opt.Prefix = []byte(prefix)
		// PickTable expects smallest and biggest to contain timestamps.
		tm := &tableMock{left: y.KeyWithTs([]byte(left), 1), right: y.KeyWithTs([]byte(right), 1)}
		require.False(t, opt.pickTable(tm), "outside failed for %b %b %b", prefix, left, right)
	}
	within([]byte("abc"), []byte("ab"), []byte("ad"))
	within([]byte("abc"), []byte("abc"), []byte("ad"))
	within([]byte("abc"), []byte("abb123"), []byte("ad"))
	within([]byte("abc"), []byte("abc123"), []byte("abd234"))
	within([]byte("abc"), []byte("abc123"), []byte("abc456"))
	// Regression test for https://github.com/dgraph-io/badger/issues/992
	within([]byte{0, 0, 1}, []byte{0}, []byte{0, 0, 1})

	outside("abd", "abe", "ad")
	outside("abd", "ac", "ad")
	outside("abd", "b", "e")
	outside("abd", "a", "ab")
	outside("abd", "ab", "abc")
	outside("abd", "ab", "abc123")
}

func TestPickSortTables(t *testing.T) {
	type MockKeys struct {
		small string
		large string
	}
	genTables := func(mks ...MockKeys) []*table.Table {
		out := make([]*table.Table, 0)
		for _, mk := range mks {
			opts := table.Options{ChkMode: options.OnTableAndBlockRead}
			tbl := buildTable(t, [][]string{{mk.small, "some value"},
				{mk.large, "some value"}}, opts)
			defer func() { require.NoError(t, tbl.DecrRef()) }()
			out = append(out, tbl)
		}
		return out
	}
	tables := genTables(MockKeys{small: "a", large: "abc"},
		MockKeys{small: "abcd", large: "cde"},
		MockKeys{small: "cge", large: "chf"},
		MockKeys{small: "glr", large: "gyup"})
	opt := DefaultIteratorOptions
	opt.prefixIsKey = false
	opt.Prefix = []byte("c")
	filtered := opt.pickTables(tables)
	require.Equal(t, 2, len(filtered))
	// build table adds time stamp so removing tailing bytes.
	require.Equal(t, filtered[0].Smallest()[:4], []byte("abcd"))
	require.Equal(t, filtered[1].Smallest()[:3], []byte("cge"))
	tables = genTables(MockKeys{small: "a", large: "abc"},
		MockKeys{small: "abcd", large: "ade"},
		MockKeys{small: "cge", large: "chf"},
		MockKeys{small: "glr", large: "gyup"})
	filtered = opt.pickTables(tables)
	require.Equal(t, 1, len(filtered))
	require.Equal(t, filtered[0].Smallest()[:3], []byte("cge"))
	tables = genTables(MockKeys{small: "a", large: "abc"},
		MockKeys{small: "abcd", large: "ade"},
		MockKeys{small: "cge", large: "chf"},
		MockKeys{small: "ckr", large: "cyup"},
		MockKeys{small: "csfr", large: "gyup"})
	filtered = opt.pickTables(tables)
	require.Equal(t, 3, len(filtered))
	require.Equal(t, filtered[0].Smallest()[:3], []byte("cge"))
	require.Equal(t, filtered[1].Smallest()[:3], []byte("ckr"))
	require.Equal(t, filtered[2].Smallest()[:4], []byte("csfr"))

	opt.Prefix = []byte("aa")
	filtered = opt.pickTables(tables)
	require.Equal(t, y.ParseKey(filtered[0].Smallest()), []byte("a"))
	require.Equal(t, y.ParseKey(filtered[0].Biggest()), []byte("abc"))
}

func TestIterateSinceTs(t *testing.T) {
	bkey := func(i int) []byte {
		return []byte(fmt.Sprintf("%04d", i))
	}
	val := []byte("OK")
	n := 100000

	runBadgerTest(t, nil, func(t *testing.T, db *DB) {
		batch := db.NewWriteBatch()
		for i := 0; i < n; i++ {
			if (i % 10000) == 0 {
				t.Logf("Put i=%d\n", i)
			}
			require.NoError(t, batch.Set(bkey(i), val))
		}
		require.NoError(t, batch.Flush())

		maxVs := db.MaxVersion()
		sinceTs := maxVs - maxVs/10
		iopt := DefaultIteratorOptions
		iopt.SinceTs = sinceTs

		require.NoError(t, db.View(func(txn *Txn) error {
			it := txn.NewIterator(iopt)
			defer it.Close()

			for it.Rewind(); it.Valid(); it.Next() {
				i := it.Item()
				require.GreaterOrEqual(t, i.Version(), sinceTs)
			}
			return nil
		}))

	})
}

func TestIterateSinceTsWithPendingWrites(t *testing.T) {
	// The pending entries still have version=0. Even IteratorOptions.SinceTs is 0, the entries
	// should be visible.
	runBadgerTest(t, nil, func(t *testing.T, db *DB) {
		txn := db.NewTransaction(true)
		defer txn.Discard()
		require.NoError(t, txn.Set([]byte("key1"), []byte("value1")))
		require.NoError(t, txn.Set([]byte("key2"), []byte("value2")))
		itr := txn.NewIterator(DefaultIteratorOptions)
		defer itr.Close()
		count := 0
		for itr.Rewind(); itr.Valid(); itr.Next() {
			count++
		}
		require.Equal(t, 2, count)
	})
}

func TestIteratePrefix(t *testing.T) {
	if !*manual {
		t.Skip("Skipping test meant to be run manually.")
		return
	}
	testIteratorPrefix := func(t *testing.T, db *DB) {
		bkey := func(i int) []byte {
			return []byte(fmt.Sprintf("%04d", i))
		}
		val := []byte("OK")
		n := 10000

		batch := db.NewWriteBatch()
		for i := 0; i < n; i++ {
			if (i % 1000) == 0 {
				t.Logf("Put i=%d\n", i)
			}
			require.NoError(t, batch.Set(bkey(i), val))
		}
		require.NoError(t, batch.Flush())

		countKeys := func(prefix string) int {
			t.Logf("Testing with prefix: %s", prefix)
			var count int
			opt := DefaultIteratorOptions
			opt.Prefix = []byte(prefix)
			err := db.View(func(txn *Txn) error {
				itr := txn.NewIterator(opt)
				defer itr.Close()
				for itr.Rewind(); itr.Valid(); itr.Next() {
					item := itr.Item()
					err := item.Value(func(v []byte) error {
						require.Equal(t, val, v)
						return nil
					})
					require.NoError(t, err)
					require.True(t, bytes.HasPrefix(item.Key(), opt.Prefix))
					count++
				}
				return nil
			})
			require.NoError(t, err)
			return count
		}

		countOneKey := func(key []byte) int {
			var count int
			err := db.View(func(txn *Txn) error {
				itr := txn.NewKeyIterator(key, DefaultIteratorOptions)
				defer itr.Close()
				for itr.Rewind(); itr.Valid(); itr.Next() {
					item := itr.Item()
					err := item.Value(func(v []byte) error {
						require.Equal(t, val, v)
						return nil
					})
					require.NoError(t, err)
					require.Equal(t, key, item.Key())
					count++
				}
				return nil
			})
			require.NoError(t, err)
			return count
		}

		for i := 0; i <= 9; i++ {
			require.Equal(t, 1, countKeys(fmt.Sprintf("%d%d%d%d", i, i, i, i)))
			require.Equal(t, 10, countKeys(fmt.Sprintf("%d%d%d", i, i, i)))
			require.Equal(t, 100, countKeys(fmt.Sprintf("%d%d", i, i)))
			require.Equal(t, 1000, countKeys(fmt.Sprintf("%d", i)))
		}
		require.Equal(t, 10000, countKeys(""))

		t.Logf("Testing each key with key iterator")
		for i := 0; i < n; i++ {
			require.Equal(t, 1, countOneKey(bkey(i)))
		}
	}

	t.Run("With Default options", func(t *testing.T) {
		runBadgerTest(t, nil, func(t *testing.T, db *DB) {
			testIteratorPrefix(t, db)
		})
	})

	t.Run("With Block Offsets in Cache", func(t *testing.T) {
		opts := getTestOptions("")
		opts.IndexCacheSize = 100 << 20
		runBadgerTest(t, &opts, func(t *testing.T, db *DB) {
			testIteratorPrefix(t, db)
		})
	})

	t.Run("With Block Offsets and Blocks in Cache", func(t *testing.T) {
		opts := getTestOptions("")
		opts.BlockCacheSize = 100 << 20
		opts.IndexCacheSize = 100 << 20
		runBadgerTest(t, &opts, func(t *testing.T, db *DB) {
			testIteratorPrefix(t, db)
		})
	})

	t.Run("With Blocks in Cache", func(t *testing.T) {
		opts := getTestOptions("")
		opts.BlockCacheSize = 100 << 20
		runBadgerTest(t, &opts, func(t *testing.T, db *DB) {
			testIteratorPrefix(t, db)
		})
	})

}

// Sanity test to verify the iterator does not crash the db in readonly mode if data does not exist.
func TestIteratorReadOnlyWithNoData(t *testing.T) {
	dir, err := os.MkdirTemp(".", "badger-test")
	y.Check(err)
	defer removeDir(dir)
	opts := getTestOptions(dir)
	db, err := Open(opts)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	opts.ReadOnly = true
	db, err = Open(opts)
	require.NoError(t, err)
	defer func() {
		require.NoError(t, db.Close())
	}()

	require.NoError(t, db.View(func(txn *Txn) error {
		iopts := DefaultIteratorOptions
		iopts.Prefix = []byte("xxx")
		itr := txn.NewIterator(iopts)
		defer itr.Close()
		return nil
	}))
}

// go test -v -run=XXX -bench=BenchmarkIterate -benchtime=3s
// Benchmark with opt.Prefix set ===
// goos: linux
// goarch: amd64
// pkg: github.com/dgraph-io/badger
// BenchmarkIteratePrefixSingleKey/Key_lookups-4         	   10000	    365539 ns/op
// --- BENCH: BenchmarkIteratePrefixSingleKey/Key_lookups-4
//
//	iterator_test.go:147: Inner b.N: 1
//	iterator_test.go:147: Inner b.N: 100
//	iterator_test.go:147: Inner b.N: 10000
//
// --- BENCH: BenchmarkIteratePrefixSingleKey
//
//	iterator_test.go:143: LSM files: 79
//	iterator_test.go:145: Outer b.N: 1
//
// PASS
// ok  	github.com/dgraph-io/badger	41.586s
//
// Benchmark with NO opt.Prefix set ===
// goos: linux
// goarch: amd64
// pkg: github.com/dgraph-io/badger
// BenchmarkIteratePrefixSingleKey/Key_lookups-4         	   10000	    460924 ns/op
// --- BENCH: BenchmarkIteratePrefixSingleKey/Key_lookups-4
//
//	iterator_test.go:147: Inner b.N: 1
//	iterator_test.go:147: Inner b.N: 100
//	iterator_test.go:147: Inner b.N: 10000
//
// --- BENCH: BenchmarkIteratePrefixSingleKey
//
//	iterator_test.go:143: LSM files: 83
//	iterator_test.go:145: Outer b.N: 1
//
// PASS
// ok  	github.com/dgraph-io/badger	41.836s
//
// Only my laptop there's a 20% improvement in latency with ~80 files.
func BenchmarkIteratePrefixSingleKey(b *testing.B) {
	dir, err := os.MkdirTemp(".", "badger-test")
	y.Check(err)
	defer removeDir(dir)
	opts := getTestOptions(dir)
	db, err := Open(opts)
	y.Check(err)
	defer db.Close()

	N := 100000 // Should generate around 80 SSTables.
	val := []byte("OK")
	bkey := func(i int) []byte {
		return []byte(fmt.Sprintf("%06d", i))
	}

	batch := db.NewWriteBatch()
	for i := 0; i < N; i++ {
		y.Check(batch.Set(bkey(i), val))
	}
	y.Check(batch.Flush())
	var lsmFiles int
	err = filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if strings.HasSuffix(path, ".sst") {
			lsmFiles++
		}
		if err != nil {
			return err
		}
		return nil
	})
	y.Check(err)
	b.Logf("LSM files: %d", lsmFiles)
	b.Logf("Key splits: %v", db.Ranges(nil, 10000))
	b.Logf("Key splits with prefix: %v", db.Ranges([]byte("09"), 10000))

	b.Logf("Outer b.N: %d", b.N)
	b.Run("Key lookups", func(b *testing.B) {
		b.Logf("Inner b.N: %d", b.N)
		for i := 0; i < b.N; i++ {
			key := bkey(rand.Intn(N))
			err := db.View(func(txn *Txn) error {
				opt := DefaultIteratorOptions
				// NOTE: Comment opt.Prefix out here to compare the performance
				// difference between providing Prefix as an option, v/s not. I
				// see a 20% improvement when there are ~80 SSTables.
				opt.Prefix = key
				opt.AllVersions = true

				itr := txn.NewIterator(opt)
				defer itr.Close()

				var count int
				for itr.Seek(key); itr.ValidForPrefix(key); itr.Next() {
					count++
				}
				if count != 1 {
					b.Fatalf("Count must be one key: %s. Found: %d", key, count)
				}
				return nil
			})
			if err != nil {
				b.Fatalf("Error while View: %v", err)
			}
		}
	})
}
