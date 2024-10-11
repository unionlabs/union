/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
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

package table

import (
	"bytes"
	"crypto/sha256"
	"fmt"
	"hash/crc32"
	"math/rand"
	"os"
	"sort"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/cespare/xxhash/v2"
	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/y"
	"github.com/dgraph-io/ristretto"
)

func key(prefix string, i int) string {
	return prefix + fmt.Sprintf("%04d", i)
}

func getTestTableOptions() Options {
	return Options{
		Compression:          options.ZSTD,
		ZSTDCompressionLevel: 15,
		BlockSize:            4 * 1024,
		BloomFalsePositive:   0.01,
	}

}
func buildTestTable(t *testing.T, prefix string, n int, opts Options) *Table {
	if opts.BlockSize == 0 {
		opts.BlockSize = 4 * 1024
	}
	y.AssertTrue(n <= 10000)
	keyValues := make([][]string, n)
	for i := 0; i < n; i++ {
		k := key(prefix, i)
		v := fmt.Sprintf("%d", i)
		keyValues[i] = []string{k, v}
	}
	return buildTable(t, keyValues, opts)
}

// keyValues is n by 2 where n is number of pairs.
func buildTable(t *testing.T, keyValues [][]string, opts Options) *Table {
	b := NewTableBuilder(opts)
	defer b.Close()
	// TODO: Add test for file garbage collection here. No files should be left after the tests here.

	filename := fmt.Sprintf("%s%s%d.sst", os.TempDir(), string(os.PathSeparator), rand.Uint32())

	sort.Slice(keyValues, func(i, j int) bool {
		return keyValues[i][0] < keyValues[j][0]
	})
	for _, kv := range keyValues {
		y.AssertTrue(len(kv) == 2)
		b.Add(y.KeyWithTs([]byte(kv[0]), 0),
			y.ValueStruct{Value: []byte(kv[1]), Meta: 'A', UserMeta: 0}, 0)
	}
	tbl, err := CreateTable(filename, b)
	require.NoError(t, err, "writing to file failed")
	return tbl
}

func TestTableIterator(t *testing.T) {
	for _, n := range []int{99, 100, 101} {
		t.Run(fmt.Sprintf("n=%d", n), func(t *testing.T) {
			opts := getTestTableOptions()
			table := buildTestTable(t, "key", n, opts)
			defer func() { require.NoError(t, table.DecrRef()) }()
			it := table.NewIterator(0)
			defer it.Close()
			count := 0
			for it.Rewind(); it.Valid(); it.Next() {
				v := it.Value()
				k := y.KeyWithTs([]byte(key("key", count)), 0)
				require.EqualValues(t, k, it.Key())
				require.EqualValues(t, fmt.Sprintf("%d", count), string(v.Value))
				count++
			}
			require.Equal(t, count, n)
		})
	}
}

func TestSeekToFirst(t *testing.T) {
	for _, n := range []int{99, 100, 101, 199, 200, 250, 9999, 10000} {
		t.Run(fmt.Sprintf("n=%d", n), func(t *testing.T) {
			opts := getTestTableOptions()
			table := buildTestTable(t, "key", n, opts)
			defer func() { require.NoError(t, table.DecrRef()) }()
			it := table.NewIterator(0)
			defer it.Close()
			it.seekToFirst()
			require.True(t, it.Valid())
			v := it.Value()
			require.EqualValues(t, "0", string(v.Value))
			require.EqualValues(t, 'A', v.Meta)
		})
	}
}

func TestSeekToLast(t *testing.T) {
	for _, n := range []int{99, 100, 101, 199, 200, 250, 9999, 10000} {
		t.Run(fmt.Sprintf("n=%d", n), func(t *testing.T) {
			opts := getTestTableOptions()
			table := buildTestTable(t, "key", n, opts)
			defer func() { require.NoError(t, table.DecrRef()) }()
			it := table.NewIterator(0)
			defer it.Close()
			it.seekToLast()
			require.True(t, it.Valid())
			v := it.Value()
			require.EqualValues(t, fmt.Sprintf("%d", n-1), string(v.Value))
			require.EqualValues(t, 'A', v.Meta)
			it.prev()
			require.True(t, it.Valid())
			v = it.Value()
			require.EqualValues(t, fmt.Sprintf("%d", n-2), string(v.Value))
			require.EqualValues(t, 'A', v.Meta)
		})
	}
}

func TestSeek(t *testing.T) {
	opts := getTestTableOptions()
	table := buildTestTable(t, "k", 10000, opts)
	defer func() { require.NoError(t, table.DecrRef()) }()

	it := table.NewIterator(0)
	defer it.Close()

	var data = []struct {
		in    string
		valid bool
		out   string
	}{
		{"abc", true, "k0000"},
		{"k0100", true, "k0100"},
		{"k0100b", true, "k0101"}, // Test case where we jump to next block.
		{"k1234", true, "k1234"},
		{"k1234b", true, "k1235"},
		{"k9999", true, "k9999"},
		{"z", false, ""},
	}

	for _, tt := range data {
		it.seek(y.KeyWithTs([]byte(tt.in), 0))
		if !tt.valid {
			require.False(t, it.Valid())
			continue
		}
		require.True(t, it.Valid())
		k := it.Key()
		require.EqualValues(t, tt.out, string(y.ParseKey(k)))
	}
}

func TestSeekForPrev(t *testing.T) {
	opts := getTestTableOptions()
	table := buildTestTable(t, "k", 10000, opts)
	defer func() { require.NoError(t, table.DecrRef()) }()

	it := table.NewIterator(0)
	defer it.Close()

	var data = []struct {
		in    string
		valid bool
		out   string
	}{
		{"abc", false, ""},
		{"k0100", true, "k0100"},
		{"k0100b", true, "k0100"}, // Test case where we jump to next block.
		{"k1234", true, "k1234"},
		{"k1234b", true, "k1234"},
		{"k9999", true, "k9999"},
		{"z", true, "k9999"},
	}

	for _, tt := range data {
		it.seekForPrev(y.KeyWithTs([]byte(tt.in), 0))
		if !tt.valid {
			require.False(t, it.Valid())
			continue
		}
		require.True(t, it.Valid())
		k := it.Key()
		require.EqualValues(t, tt.out, string(y.ParseKey(k)))
	}
}

func TestIterateFromStart(t *testing.T) {
	// Vary the number of elements added.
	for _, n := range []int{99, 100, 101, 199, 200, 250, 9999, 10000} {
		t.Run(fmt.Sprintf("n=%d", n), func(t *testing.T) {
			opts := getTestTableOptions()
			table := buildTestTable(t, "key", n, opts)
			defer func() { require.NoError(t, table.DecrRef()) }()
			ti := table.NewIterator(0)
			defer ti.Close()
			ti.reset()
			ti.seekToFirst()
			require.True(t, ti.Valid())
			// No need to do a Next.
			// ti.Seek brings us to the first key >= "". Essentially a SeekToFirst.
			var count int
			for ; ti.Valid(); ti.next() {
				v := ti.Value()
				require.EqualValues(t, fmt.Sprintf("%d", count), string(v.Value))
				require.EqualValues(t, 'A', v.Meta)
				count++
			}
			require.EqualValues(t, n, count)
		})
	}
}

func TestIterateFromEnd(t *testing.T) {
	// Vary the number of elements added.
	for _, n := range []int{99, 100, 101, 199, 200, 250, 9999, 10000} {
		t.Run(fmt.Sprintf("n=%d", n), func(t *testing.T) {
			opts := getTestTableOptions()
			table := buildTestTable(t, "key", n, opts)
			defer func() { require.NoError(t, table.DecrRef()) }()
			ti := table.NewIterator(0)
			defer ti.Close()
			ti.reset()
			ti.seek(y.KeyWithTs([]byte("zzzzzz"), 0)) // Seek to end, an invalid element.
			require.False(t, ti.Valid())
			for i := n - 1; i >= 0; i-- {
				ti.prev()
				require.True(t, ti.Valid())
				v := ti.Value()
				require.EqualValues(t, fmt.Sprintf("%d", i), string(v.Value))
				require.EqualValues(t, 'A', v.Meta)
			}
			ti.prev()
			require.False(t, ti.Valid())
		})
	}
}

func TestTable(t *testing.T) {
	opts := getTestTableOptions()
	table := buildTestTable(t, "key", 10000, opts)
	defer func() { require.NoError(t, table.DecrRef()) }()
	ti := table.NewIterator(0)
	defer ti.Close()
	kid := 1010
	seek := y.KeyWithTs([]byte(key("key", kid)), 0)
	for ti.seek(seek); ti.Valid(); ti.next() {
		k := ti.Key()
		require.EqualValues(t, string(y.ParseKey(k)), key("key", kid))
		kid++
	}
	if kid != 10000 {
		t.Errorf("Expected kid: 10000. Got: %v", kid)
	}

	ti.seek(y.KeyWithTs([]byte(key("key", 99999)), 0))
	require.False(t, ti.Valid())

	ti.seek(y.KeyWithTs([]byte(key("key", -1)), 0))
	require.True(t, ti.Valid())
	k := ti.Key()
	require.EqualValues(t, string(y.ParseKey(k)), key("key", 0))
}

func TestIterateBackAndForth(t *testing.T) {
	opts := getTestTableOptions()
	table := buildTestTable(t, "key", 10000, opts)
	defer func() { require.NoError(t, table.DecrRef()) }()

	seek := y.KeyWithTs([]byte(key("key", 1010)), 0)
	it := table.NewIterator(0)
	defer it.Close()
	it.seek(seek)
	require.True(t, it.Valid())
	k := it.Key()
	require.EqualValues(t, seek, k)

	it.prev()
	it.prev()
	require.True(t, it.Valid())
	k = it.Key()
	require.EqualValues(t, key("key", 1008), string(y.ParseKey(k)))

	it.next()
	it.next()
	require.True(t, it.Valid())
	k = it.Key()
	require.EqualValues(t, key("key", 1010), y.ParseKey(k))

	it.seek(y.KeyWithTs([]byte(key("key", 2000)), 0))
	require.True(t, it.Valid())
	k = it.Key()
	require.EqualValues(t, key("key", 2000), y.ParseKey(k))

	it.prev()
	require.True(t, it.Valid())
	k = it.Key()
	require.EqualValues(t, key("key", 1999), y.ParseKey(k))

	it.seekToFirst()
	k = it.Key()
	require.EqualValues(t, key("key", 0), string(y.ParseKey(k)))
}

func TestUniIterator(t *testing.T) {
	opts := getTestTableOptions()
	table := buildTestTable(t, "key", 10000, opts)
	defer func() { require.NoError(t, table.DecrRef()) }()
	{
		it := table.NewIterator(0)
		defer it.Close()
		var count int
		for it.Rewind(); it.Valid(); it.Next() {
			v := it.Value()
			require.EqualValues(t, fmt.Sprintf("%d", count), string(v.Value))
			require.EqualValues(t, 'A', v.Meta)
			count++
		}
		require.EqualValues(t, 10000, count)
	}
	{
		it := table.NewIterator(REVERSED)
		defer it.Close()
		var count int
		for it.Rewind(); it.Valid(); it.Next() {
			v := it.Value()
			require.EqualValues(t, fmt.Sprintf("%d", 10000-1-count), string(v.Value))
			require.EqualValues(t, 'A', v.Meta)
			count++
		}
		require.EqualValues(t, 10000, count)
	}
}

// Try having only one table.
func TestConcatIteratorOneTable(t *testing.T) {
	opts := getTestTableOptions()
	tbl := buildTable(t, [][]string{
		{"k1", "a1"},
		{"k2", "a2"},
	}, opts)
	defer func() { require.NoError(t, tbl.DecrRef()) }()

	it := NewConcatIterator([]*Table{tbl}, 0)
	defer it.Close()

	it.Rewind()
	require.True(t, it.Valid())
	k := it.Key()
	require.EqualValues(t, "k1", string(y.ParseKey(k)))
	vs := it.Value()
	require.EqualValues(t, "a1", string(vs.Value))
	require.EqualValues(t, 'A', vs.Meta)
}

func TestConcatIterator(t *testing.T) {
	opts := getTestTableOptions()
	tbl := buildTestTable(t, "keya", 10000, opts)
	defer func() { require.NoError(t, tbl.DecrRef()) }()
	tbl2 := buildTestTable(t, "keyb", 10000, opts)
	defer func() { require.NoError(t, tbl2.DecrRef()) }()
	tbl3 := buildTestTable(t, "keyc", 10000, opts)
	defer func() { require.NoError(t, tbl3.DecrRef()) }()

	{
		it := NewConcatIterator([]*Table{tbl, tbl2, tbl3}, 0)
		defer it.Close()
		it.Rewind()
		require.True(t, it.Valid())
		var count int
		for ; it.Valid(); it.Next() {
			vs := it.Value()
			require.EqualValues(t, fmt.Sprintf("%d", count%10000), string(vs.Value))
			require.EqualValues(t, 'A', vs.Meta)
			count++
		}
		require.EqualValues(t, 30000, count)

		it.Seek(y.KeyWithTs([]byte("a"), 0))
		require.EqualValues(t, "keya0000", string(y.ParseKey(it.Key())))
		vs := it.Value()
		require.EqualValues(t, "0", string(vs.Value))

		it.Seek(y.KeyWithTs([]byte("keyb"), 0))
		require.EqualValues(t, "keyb0000", string(y.ParseKey(it.Key())))
		vs = it.Value()
		require.EqualValues(t, "0", string(vs.Value))

		it.Seek(y.KeyWithTs([]byte("keyb9999b"), 0))
		require.EqualValues(t, "keyc0000", string(y.ParseKey(it.Key())))
		vs = it.Value()
		require.EqualValues(t, "0", string(vs.Value))

		it.Seek(y.KeyWithTs([]byte("keyd"), 0))
		require.False(t, it.Valid())
	}
	{
		it := NewConcatIterator([]*Table{tbl, tbl2, tbl3}, REVERSED)
		defer it.Close()
		it.Rewind()
		require.True(t, it.Valid())
		var count int
		for ; it.Valid(); it.Next() {
			vs := it.Value()
			require.EqualValues(t, fmt.Sprintf("%d", 10000-(count%10000)-1), string(vs.Value))
			require.EqualValues(t, 'A', vs.Meta)
			count++
		}
		require.EqualValues(t, 30000, count)

		it.Seek(y.KeyWithTs([]byte("a"), 0))
		require.False(t, it.Valid())

		it.Seek(y.KeyWithTs([]byte("keyb"), 0))
		require.EqualValues(t, "keya9999", string(y.ParseKey(it.Key())))
		vs := it.Value()
		require.EqualValues(t, "9999", string(vs.Value))

		it.Seek(y.KeyWithTs([]byte("keyb9999b"), 0))
		require.EqualValues(t, "keyb9999", string(y.ParseKey(it.Key())))
		vs = it.Value()
		require.EqualValues(t, "9999", string(vs.Value))

		it.Seek(y.KeyWithTs([]byte("keyd"), 0))
		require.EqualValues(t, "keyc9999", string(y.ParseKey(it.Key())))
		vs = it.Value()
		require.EqualValues(t, "9999", string(vs.Value))
	}
}

func TestMergingIterator(t *testing.T) {
	opts := getTestTableOptions()
	tbl1 := buildTable(t, [][]string{
		{"k1", "a1"},
		{"k4", "a4"},
		{"k5", "a5"},
	}, opts)
	defer func() { require.NoError(t, tbl1.DecrRef()) }()

	tbl2 := buildTable(t, [][]string{
		{"k2", "b2"},
		{"k3", "b3"},
		{"k4", "b4"},
	}, opts)
	defer func() { require.NoError(t, tbl2.DecrRef()) }()

	expected := []struct {
		key   string
		value string
	}{
		{"k1", "a1"},
		{"k2", "b2"},
		{"k3", "b3"},
		{"k4", "a4"},
		{"k5", "a5"},
	}

	it1 := tbl1.NewIterator(0)
	it2 := NewConcatIterator([]*Table{tbl2}, 0)
	it := NewMergeIterator([]y.Iterator{it1, it2}, false)
	defer it.Close()

	var i int
	for it.Rewind(); it.Valid(); it.Next() {
		k := it.Key()
		vs := it.Value()
		require.EqualValues(t, expected[i].key, string(y.ParseKey(k)))
		require.EqualValues(t, expected[i].value, string(vs.Value))
		require.EqualValues(t, 'A', vs.Meta)
		i++
	}
	require.Equal(t, i, len(expected))
	require.False(t, it.Valid())
}

func TestMergingIteratorReversed(t *testing.T) {
	opts := getTestTableOptions()
	tbl1 := buildTable(t, [][]string{
		{"k1", "a1"},
		{"k2", "a2"},
		{"k4", "a4"},
		{"k5", "a5"},
	}, opts)
	defer func() { require.NoError(t, tbl1.DecrRef()) }()

	tbl2 := buildTable(t, [][]string{
		{"k1", "b2"},
		{"k3", "b3"},
		{"k4", "b4"},
		{"k5", "b5"},
	}, opts)
	defer func() { require.NoError(t, tbl2.DecrRef()) }()

	expected := []struct {
		key   string
		value string
	}{
		{"k5", "a5"},
		{"k4", "a4"},
		{"k3", "b3"},
		{"k2", "a2"},
		{"k1", "a1"},
	}

	it1 := tbl1.NewIterator(REVERSED)
	it2 := NewConcatIterator([]*Table{tbl2}, REVERSED)
	it := NewMergeIterator([]y.Iterator{it1, it2}, true)
	defer it.Close()

	var i int
	for it.Rewind(); it.Valid(); it.Next() {
		k := it.Key()
		vs := it.Value()
		require.EqualValues(t, expected[i].key, string(y.ParseKey(k)))
		require.EqualValues(t, expected[i].value, string(vs.Value))
		require.EqualValues(t, 'A', vs.Meta)
		i++
	}

	require.Equal(t, i, len(expected))
	require.False(t, it.Valid())
}

// Take only the first iterator.
func TestMergingIteratorTakeOne(t *testing.T) {
	opts := getTestTableOptions()
	t1 := buildTable(t, [][]string{
		{"k1", "a1"},
		{"k2", "a2"},
	}, opts)
	defer func() { require.NoError(t, t1.DecrRef()) }()
	t2 := buildTable(t, [][]string{{"l1", "b1"}}, opts)
	defer func() { require.NoError(t, t2.DecrRef()) }()

	it1 := NewConcatIterator([]*Table{t1}, 0)
	it2 := NewConcatIterator([]*Table{t2}, 0)
	it := NewMergeIterator([]y.Iterator{it1, it2}, false)
	defer it.Close()

	it.Rewind()
	require.True(t, it.Valid())
	k := it.Key()
	require.EqualValues(t, "k1", string(y.ParseKey(k)))
	vs := it.Value()
	require.EqualValues(t, "a1", string(vs.Value))
	require.EqualValues(t, 'A', vs.Meta)
	it.Next()

	require.True(t, it.Valid())
	k = it.Key()
	require.EqualValues(t, "k2", string(y.ParseKey(k)))
	vs = it.Value()
	require.EqualValues(t, "a2", string(vs.Value))
	require.EqualValues(t, 'A', vs.Meta)
	it.Next()

	k = it.Key()
	require.EqualValues(t, "l1", string(y.ParseKey(k)))
	vs = it.Value()
	require.EqualValues(t, "b1", string(vs.Value))
	require.EqualValues(t, 'A', vs.Meta)
	it.Next()

	require.False(t, it.Valid())
}

// Take only the second iterator.
func TestMergingIteratorTakeTwo(t *testing.T) {
	opts := getTestTableOptions()
	t1 := buildTable(t, [][]string{{"l1", "b1"}}, opts)
	defer func() { require.NoError(t, t1.DecrRef()) }()

	t2 := buildTable(t, [][]string{
		{"k1", "a1"},
		{"k2", "a2"},
	}, opts)
	defer func() { require.NoError(t, t2.DecrRef()) }()

	it1 := NewConcatIterator([]*Table{t1}, 0)
	it2 := NewConcatIterator([]*Table{t2}, 0)
	it := NewMergeIterator([]y.Iterator{it1, it2}, false)
	defer it.Close()

	it.Rewind()
	require.True(t, it.Valid())
	k := it.Key()
	require.EqualValues(t, "k1", string(y.ParseKey(k)))
	vs := it.Value()
	require.EqualValues(t, "a1", string(vs.Value))
	require.EqualValues(t, 'A', vs.Meta)
	it.Next()

	require.True(t, it.Valid())
	k = it.Key()
	require.EqualValues(t, "k2", string(y.ParseKey(k)))
	vs = it.Value()
	require.EqualValues(t, "a2", string(vs.Value))
	require.EqualValues(t, 'A', vs.Meta)
	it.Next()
	require.True(t, it.Valid())

	k = it.Key()
	require.EqualValues(t, "l1", string(y.ParseKey(k)))
	vs = it.Value()
	require.EqualValues(t, "b1", string(vs.Value))
	require.EqualValues(t, 'A', vs.Meta)
	it.Next()

	require.False(t, it.Valid())
}

func TestTableBigValues(t *testing.T) {
	value := func(i int) []byte {
		return []byte(fmt.Sprintf("%01048576d", i)) // Return 1MB value which is > math.MaxUint16.
	}

	rand.Seed(time.Now().UnixNano())

	n := 100 // Insert 100 keys.
	opts := Options{Compression: options.ZSTD, BlockSize: 4 * 1024, BloomFalsePositive: 0.01,
		TableSize: uint64(n) * 1 << 20}
	builder := NewTableBuilder(opts)
	defer builder.Close()
	for i := 0; i < n; i++ {
		key := y.KeyWithTs([]byte(key("", i)), uint64(i+1))
		vs := y.ValueStruct{Value: value(i)}
		builder.Add(key, vs, 0)
	}

	filename := fmt.Sprintf("%s%s%d.sst", os.TempDir(), string(os.PathSeparator), rand.Uint32())
	tbl, err := CreateTable(filename, builder)
	require.NoError(t, err, "unable to open table")
	defer func() { require.NoError(t, tbl.DecrRef()) }()

	itr := tbl.NewIterator(0)
	require.True(t, itr.Valid())

	count := 0
	for itr.Rewind(); itr.Valid(); itr.Next() {
		require.Equal(t, []byte(key("", count)), y.ParseKey(itr.Key()), "keys are not equal")
		require.Equal(t, value(count), itr.Value().Value, "values are not equal")
		count++
	}
	require.False(t, itr.Valid(), "table iterator should be invalid now")
	require.Equal(t, n, count)
	require.Equal(t, n, int(tbl.MaxVersion()))
}

// This test is for verifying checksum failure during table open.
func TestTableChecksum(t *testing.T) {
	rand.Seed(time.Now().Unix())
	// we are going to write random byte at random location in table file.
	rb := make([]byte, 100)
	rand.Read(rb)
	opts := getTestTableOptions()
	opts.ChkMode = options.OnTableAndBlockRead
	// When verifying checksum capability, we find it simpler to disable compression
	// since randomly initializing bytes can kill the compression storage.
	opts.Compression = options.None
	tbl := buildTestTable(t, "k", 10000, opts)
	defer func() { require.NoError(t, tbl.DecrRef()) }()
	// Write random bytes at location guaranteed to not be in range of
	// metadata for block. (No particular reason for the value 128,
	// it just avoids the sensitive block size or other metadata blocks).
	start := 128
	n := copy(tbl.Data[start:], rb)
	require.Equal(t, n, len(rb))

	require.Panics(t, func() {
		// Either OpenTable will panic on corrupted data or the checksum verification will fail.
		_, err := OpenTable(tbl.MmapFile, opts)
		if strings.Contains(err.Error(), "checksum") {
			panic("checksum mismatch")
		} else {
			require.NoError(t, err)
		}
	})
}

var cacheConfig = ristretto.Config[[]byte, *Block]{
	NumCounters: 1000000 * 10,
	MaxCost:     1000000,
	BufferItems: 64,
	Metrics:     true,
}

func BenchmarkRead(b *testing.B) {
	n := int(5 * 1e6)
	tbl := getTableForBenchmarks(b, n, nil)
	defer func() { _ = tbl.DecrRef() }()

	b.ResetTimer()
	// Iterate b.N times over the entire table.
	for i := 0; i < b.N; i++ {
		func() {
			it := tbl.NewIterator(0)
			defer it.Close()
			for it.seekToFirst(); it.Valid(); it.next() {
			}
		}()
	}
}

func BenchmarkReadAndBuild(b *testing.B) {
	n := int(5 * 1e6)

	var cache, _ = ristretto.NewCache(&cacheConfig)
	tbl := getTableForBenchmarks(b, n, cache)
	defer func() { _ = tbl.DecrRef() }()

	b.ResetTimer()
	// Iterate b.N times over the entire table.
	for i := 0; i < b.N; i++ {
		func() {
			opts := Options{Compression: options.ZSTD, BlockSize: 4 * 0124, BloomFalsePositive: 0.01}
			opts.BlockCache = cache
			newBuilder := NewTableBuilder(opts)
			it := tbl.NewIterator(0)
			defer it.Close()
			for it.seekToFirst(); it.Valid(); it.next() {
				vs := it.Value()
				newBuilder.Add(it.Key(), vs, 0)
			}
			newBuilder.Finish()
		}()
	}
}

func BenchmarkReadMerged(b *testing.B) {
	n := int(5 * 1e6)
	m := 5 // Number of tables.
	y.AssertTrue((n % m) == 0)
	tableSize := n / m
	var tables []*Table

	var cache, err = ristretto.NewCache(&cacheConfig)
	require.NoError(b, err)

	for i := 0; i < m; i++ {
		filename := fmt.Sprintf("%s%s%d.sst", os.TempDir(), string(os.PathSeparator), rand.Uint32())
		opts := Options{Compression: options.ZSTD, BlockSize: 4 * 1024, BloomFalsePositive: 0.01}
		opts.BlockCache = cache
		builder := NewTableBuilder(opts)
		for j := 0; j < tableSize; j++ {
			id := j*m + i // Arrays are interleaved.
			// id := i*tableSize+j (not interleaved)
			k := fmt.Sprintf("%016x", id)
			v := fmt.Sprintf("%d", id)
			builder.Add([]byte(k), y.ValueStruct{Value: []byte(v), Meta: 123, UserMeta: 0}, 0)
		}
		tbl, err := CreateTable(filename, builder)
		y.Check(err)
		builder.Close()
		tables = append(tables, tbl)
		defer func() { _ = tbl.DecrRef() }()
	}

	b.ResetTimer()
	// Iterate b.N times over the entire table.
	for i := 0; i < b.N; i++ {
		func() {
			var iters []y.Iterator
			for _, tbl := range tables {
				iters = append(iters, tbl.NewIterator(0))
			}
			it := NewMergeIterator(iters, false)
			defer it.Close()
			for it.Rewind(); it.Valid(); it.Next() {
			}
		}()
	}
}

func BenchmarkChecksum(b *testing.B) {
	keySz := []int{KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB, 32 * KB, 64 * KB, 128 * KB, 256 * KB, MB}
	for _, kz := range keySz {
		key := make([]byte, kz)
		b.Run(fmt.Sprintf("CRC %d", kz), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				crc32.ChecksumIEEE(key)
			}
		})
		b.Run(fmt.Sprintf("xxHash64 %d", kz), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				xxhash.Sum64(key)
			}
		})
		b.Run(fmt.Sprintf("SHA256 %d", kz), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				sha256.Sum256(key)
			}
		})
	}
}

func BenchmarkRandomRead(b *testing.B) {
	n := int(5 * 1e6)
	tbl := getTableForBenchmarks(b, n, nil)
	defer func() { _ = tbl.DecrRef() }()

	r := rand.New(rand.NewSource(time.Now().Unix()))

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		itr := tbl.NewIterator(0)
		no := r.Intn(n)
		k := []byte(fmt.Sprintf("%016x", no))
		v := []byte(fmt.Sprintf("%d", no))
		itr.Seek(k)
		if !itr.Valid() {
			b.Fatal("itr should be valid")
		}
		v1 := itr.Value().Value

		if !bytes.Equal(v, v1) {
			fmt.Println("value does not match")
			b.Fatal()
		}
		itr.Close()
	}
}

func getTableForBenchmarks(b *testing.B, count int, cache *ristretto.Cache[[]byte, *Block]) *Table {
	rand.Seed(time.Now().Unix())
	opts := Options{Compression: options.ZSTD, BlockSize: 4 * 1024, BloomFalsePositive: 0.01}
	if cache == nil {
		var err error
		cache, err = ristretto.NewCache(&cacheConfig)
		require.NoError(b, err)
	}
	opts.BlockCache = cache
	builder := NewTableBuilder(opts)
	defer builder.Close()
	filename := fmt.Sprintf("%s%s%d.sst", os.TempDir(), string(os.PathSeparator), rand.Uint32())
	for i := 0; i < count; i++ {
		k := fmt.Sprintf("%016x", i)
		v := fmt.Sprintf("%d", i)
		builder.Add([]byte(k), y.ValueStruct{Value: []byte(v)}, 0)
	}

	tbl, err := CreateTable(filename, builder)
	require.NoError(b, err, "unable to open table")
	return tbl
}

func TestMain(m *testing.M) {
	rand.Seed(time.Now().UTC().UnixNano())
	os.Exit(m.Run())
}

// Run this test with command "go test -race -run TestDoesNotHaveRace"
func TestDoesNotHaveRace(t *testing.T) {
	opts := getTestTableOptions()
	table := buildTestTable(t, "key", 10000, opts)
	defer func() { require.NoError(t, table.DecrRef()) }()

	var wg sync.WaitGroup
	wg.Add(5)
	for i := 0; i < 5; i++ {
		go func() {
			require.True(t, table.DoesNotHave(uint32(1237882)))
			wg.Done()
		}()
	}
	wg.Wait()
}

func TestMaxVersion(t *testing.T) {
	opt := getTestTableOptions()
	b := NewTableBuilder(opt)
	defer b.Close()

	filename := fmt.Sprintf("%s%s%d.sst", os.TempDir(), string(os.PathSeparator), rand.Uint32())
	N := 1000
	for i := 0; i < N; i++ {
		b.Add(y.KeyWithTs([]byte(fmt.Sprintf("foo:%d", i)), uint64(i+1)), y.ValueStruct{}, 0)
	}
	table, err := CreateTable(filename, b)
	require.NoError(t, err)
	require.Equal(t, N, int(table.MaxVersion()))
}
