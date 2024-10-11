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

package table

import (
	"fmt"
	"math/rand"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/fb"
	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/pb"
	"github.com/dgraph-io/badger/v4/y"
	"github.com/dgraph-io/ristretto"
)

func TestTableIndex(t *testing.T) {
	rand.Seed(time.Now().Unix())
	keysCount := 100000
	key := make([]byte, 32)
	_, err := rand.Read(key)
	require.NoError(t, err)
	cache, err := ristretto.NewCache[uint64, *fb.TableIndex](&ristretto.Config[uint64, *fb.TableIndex]{
		NumCounters: 1000,
		MaxCost:     1 << 20,
		BufferItems: 64,
	})
	require.NoError(t, err)
	subTest := []struct {
		name string
		opts Options
	}{
		{
			name: "No encyption/compression",
			opts: Options{
				BlockSize:          4 * 1024,
				BloomFalsePositive: 0.01,
				TableSize:          30 << 20,
			},
		},
		{
			// Encryption mode.
			name: "Only encryption",
			opts: Options{
				BlockSize:          4 * 1024,
				BloomFalsePositive: 0.01,
				TableSize:          30 << 20,
				DataKey:            &pb.DataKey{Data: key},
				IndexCache:         cache,
			},
		},
		{
			// Compression mode.
			name: "Only compression",
			opts: Options{
				BlockSize:            4 * 1024,
				BloomFalsePositive:   0.01,
				TableSize:            30 << 20,
				Compression:          options.ZSTD,
				ZSTDCompressionLevel: 3,
			},
		},
		{
			// Compression mode and encryption.
			name: "Compression and encryption",
			opts: Options{
				BlockSize:            4 * 1024,
				BloomFalsePositive:   0.01,
				TableSize:            30 << 20,
				Compression:          options.ZSTD,
				ZSTDCompressionLevel: 3,
				DataKey:              &pb.DataKey{Data: key},
				IndexCache:           cache,
			},
		},
	}

	for _, tt := range subTest {
		t.Run(tt.name, func(t *testing.T) {
			opt := tt.opts
			builder := NewTableBuilder(opt)
			defer builder.Close()
			filename := fmt.Sprintf("%s%c%d.sst", os.TempDir(), os.PathSeparator, rand.Uint32())

			blockFirstKeys := make([][]byte, 0)
			blockCount := 0
			for i := 0; i < keysCount; i++ {
				k := y.KeyWithTs([]byte(fmt.Sprintf("%016x", i)), uint64(i+1))
				v := fmt.Sprintf("%d", i)
				vs := y.ValueStruct{Value: []byte(v)}
				if i == 0 { // This is first key for first block.
					blockFirstKeys = append(blockFirstKeys, k)
					blockCount = 1
				} else if builder.shouldFinishBlock(k, vs) {
					blockCount++
					blockFirstKeys = append(blockFirstKeys, k)
				}
				builder.Add(k, vs, 0)
			}
			tbl, err := CreateTable(filename, builder)
			require.NoError(t, err, "unable to open table")

			if opt.DataKey == nil {
				// key id is zero if there is no datakey.
				require.Equal(t, tbl.KeyID(), uint64(0))
			}

			// Ensure index is built correctly
			require.Equal(t, blockCount, tbl.offsetsLength())
			idx, err := tbl.readTableIndex()
			require.NoError(t, err)
			for i := 0; i < idx.OffsetsLength(); i++ {
				var bo fb.BlockOffset
				require.True(t, idx.Offsets(&bo, i))
				require.Equal(t, blockFirstKeys[i], bo.KeyBytes())
			}
			require.Equal(t, keysCount, int(tbl.MaxVersion()))
			tbl.Close(-1)
			require.NoError(t, os.RemoveAll(filename))
		})
	}
}

func TestInvalidCompression(t *testing.T) {
	keyPrefix := "key"
	opts := Options{BlockSize: 4 << 10, Compression: options.ZSTD}
	tbl := buildTestTable(t, keyPrefix, 1000, opts)
	defer func() { require.NoError(t, tbl.DecrRef()) }()
	mf := tbl.MmapFile
	t.Run("with correct decompression algo", func(t *testing.T) {
		_, err := OpenTable(mf, opts)
		require.NoError(t, err)
	})
	t.Run("with incorrect decompression algo", func(t *testing.T) {
		// Set incorrect compression algorithm.
		opts.Compression = options.Snappy
		_, err := OpenTable(mf, opts)
		require.Error(t, err)
	})
}

func BenchmarkBuilder(b *testing.B) {
	rand.Seed(time.Now().Unix())
	key := func(i int) []byte {
		return []byte(fmt.Sprintf("%032d", i))
	}

	val := make([]byte, 32)
	rand.Read(val)
	vs := y.ValueStruct{Value: val}

	keysCount := 1300000 // This number of entries consumes ~64MB of memory.

	var keyList [][]byte
	for i := 0; i < keysCount; i++ {
		keyList = append(keyList, key(i))
	}
	bench := func(b *testing.B, opt *Options) {
		b.SetBytes(int64(keysCount) * (32 + 32))
		opt.BlockSize = 4 * 1024
		opt.BloomFalsePositive = 0.01
		opt.TableSize = 5 << 20

		b.ResetTimer()
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			builder := NewTableBuilder(*opt)
			for j := 0; j < keysCount; j++ {
				builder.Add(keyList[j], vs, 0)
			}

			_ = builder.Finish()
			builder.Close()
		}
	}

	b.Run("no compression", func(b *testing.B) {
		var opt Options
		opt.Compression = options.None
		bench(b, &opt)
	})
	b.Run("encryption", func(b *testing.B) {
		var opt Options
		cache, err := ristretto.NewCache(&ristretto.Config[uint64, *fb.TableIndex]{
			NumCounters: 1000,
			MaxCost:     1 << 20,
			BufferItems: 64,
		})
		require.NoError(b, err)
		opt.IndexCache = cache
		key := make([]byte, 32)
		rand.Read(key)
		opt.DataKey = &pb.DataKey{Data: key}
		bench(b, &opt)
	})
	b.Run("snappy compression", func(b *testing.B) {
		var opt Options
		opt.Compression = options.Snappy
		bench(b, &opt)
	})
	b.Run("zstd compression", func(b *testing.B) {
		var opt Options
		opt.Compression = options.ZSTD
		b.Run("level 1", func(b *testing.B) {
			opt.ZSTDCompressionLevel = 1
			bench(b, &opt)
		})
		b.Run("level 3", func(b *testing.B) {
			opt.ZSTDCompressionLevel = 3
			bench(b, &opt)
		})
		b.Run("level 15", func(b *testing.B) {
			opt.ZSTDCompressionLevel = 15
			bench(b, &opt)
		})
	})
}

func TestBloomfilter(t *testing.T) {
	keyPrefix := "p"
	keyCount := 1000

	createAndTest := func(t *testing.T, withBlooms bool) {
		opts := Options{
			BloomFalsePositive: 0.0,
		}
		if withBlooms {
			opts.BloomFalsePositive = 0.01
		}
		tab := buildTestTable(t, keyPrefix, keyCount, opts)
		defer func() { require.NoError(t, tab.DecrRef()) }()
		require.Equal(t, withBlooms, tab.hasBloomFilter)
		// Forward iteration
		it := tab.NewIterator(0)
		c := 0
		for it.Rewind(); it.Valid(); it.Next() {
			c++
			hash := y.Hash(y.ParseKey(it.Key()))
			require.False(t, tab.DoesNotHave(hash))
		}
		require.Equal(t, keyCount, c)

		// Backward iteration
		it = tab.NewIterator(REVERSED)
		c = 0
		for it.Rewind(); it.Valid(); it.Next() {
			c++
			hash := y.Hash(y.ParseKey(it.Key()))
			require.False(t, tab.DoesNotHave(hash))
		}
		require.Equal(t, keyCount, c)

		// Ensure tab.DoesNotHave works
		hash := y.Hash([]byte("foo"))
		require.Equal(t, withBlooms, tab.DoesNotHave(hash))
	}

	t.Run("build with bloom filter", func(t *testing.T) {
		createAndTest(t, true)
	})
	t.Run("build without bloom filter", func(t *testing.T) {
		createAndTest(t, false)
	})
}
func TestEmptyBuilder(t *testing.T) {
	opts := Options{BloomFalsePositive: 0.1}
	b := NewTableBuilder(opts)
	defer b.Close()
	require.Equal(t, []byte{}, b.Finish())

}
