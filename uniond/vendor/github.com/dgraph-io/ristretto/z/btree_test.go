/*
 * Copyright 2020 Dgraph Labs, Inc. and Contributors
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

package z

import (
	"fmt"
	"math"
	"math/rand"
	"os"
	"path/filepath"
	"sort"
	"testing"
	"time"

	"github.com/dgraph-io/ristretto/z/simd"
	"github.com/dustin/go-humanize"
	"github.com/stretchr/testify/require"
)

var tmp int

func setPageSize(sz int) {
	pageSize = sz
	maxKeys = (pageSize / 16) - 1
}

func TestTree(t *testing.T) {
	bt := NewTree("TestTree")
	defer func() { require.NoError(t, bt.Close()) }()

	N := uint64(256 * 256)
	for i := uint64(1); i < N; i++ {
		bt.Set(i, i)
	}
	for i := uint64(1); i < N; i++ {
		require.Equal(t, i, bt.Get(i))
	}

	bt.DeleteBelow(100)
	for i := uint64(1); i < 100; i++ {
		require.Equal(t, uint64(0), bt.Get(i))
	}
	for i := uint64(100); i < N; i++ {
		require.Equal(t, i, bt.Get(i))
	}
}

func TestTreePersistent(t *testing.T) {
	dir, err := os.MkdirTemp("", "")
	require.NoError(t, err)
	defer os.RemoveAll(dir)
	path := filepath.Join(dir, "tree.buf")

	// Create a tree and validate the data.
	bt1, err := NewTreePersistent(path)
	require.NoError(t, err)
	N := uint64(64 << 10)
	for i := uint64(1); i < N; i++ {
		bt1.Set(i, i*2)
	}
	for i := uint64(1); i < N; i++ {
		require.Equal(t, i*2, bt1.Get(i))
	}
	bt1Stats := bt1.Stats()
	require.NoError(t, bt1.Close())

	// Reopen tree and validate the data.
	bt2, err := NewTreePersistent(path)
	require.NoError(t, err)
	require.Equal(t, bt2.freePage, bt1.freePage)
	require.Equal(t, bt2.nextPage, bt1.nextPage)
	bt2Stats := bt2.Stats()
	// When reopening a tree, the allocated size becomes the file size.
	// We don't need to compare this, it doesn't change anything in the tree.
	bt2Stats.Allocated = bt1Stats.Allocated
	require.Equal(t, bt1Stats, bt2Stats)
	for i := uint64(1); i < N; i++ {
		require.Equal(t, i*2, bt2.Get(i))
	}
	// Delete all the data. This will change the value of bt.freePage.
	bt2.DeleteBelow(math.MaxUint64)
	bt2Stats = bt2.Stats()
	require.NoError(t, bt2.Close())

	// Reopen tree and validate the data.
	bt3, err := NewTreePersistent(path)
	require.NoError(t, err)
	require.Equal(t, bt2.freePage, bt3.freePage)
	require.Equal(t, bt2.nextPage, bt3.nextPage)
	bt3Stats := bt3.Stats()
	bt3Stats.Allocated = bt2Stats.Allocated
	require.Equal(t, bt2Stats, bt3Stats)
	require.NoError(t, bt3.Close())
}

func TestTreeBasic(t *testing.T) {
	setAndGet := func() {
		bt := NewTree("TestTreeBasic")
		defer func() { require.NoError(t, bt.Close()) }()

		N := uint64(1 << 20)
		mp := make(map[uint64]uint64)
		for i := uint64(1); i < N; i++ {
			key := uint64(rand.Int63n(1<<60) + 1)
			bt.Set(key, key)
			mp[key] = key
		}
		for k, v := range mp {
			require.Equal(t, v, bt.Get(k))
		}

		stats := bt.Stats()
		t.Logf("final stats: %+v\n", stats)
	}
	setAndGet()
	defer setPageSize(os.Getpagesize())
	setPageSize(16 << 5)
	setAndGet()
}

func TestTreeReset(t *testing.T) {
	bt := NewTree("TestTreeReset")
	defer func() { require.NoError(t, bt.Close()) }()

	N := 1 << 10
	val := rand.Uint64()
	for i := 0; i < N; i++ {
		bt.Set(rand.Uint64(), val)
	}

	// Truncate it to small size that is less than pageSize.
	bt.Reset()

	stats := bt.Stats()
	// Verify the tree stats.
	require.Equal(t, 2, stats.NumPages)
	require.Equal(t, 1, stats.NumLeafKeys)
	require.Equal(t, 2*pageSize, stats.Bytes)
	expectedOcc := float64(1) * 100 / float64(2*maxKeys)
	require.InDelta(t, expectedOcc, stats.Occupancy, 0.01)
	require.Zero(t, stats.NumPagesFree)
	// Check if we can reinsert the data.
	mp := make(map[uint64]uint64)
	for i := 0; i < N; i++ {
		k := rand.Uint64()
		mp[k] = val
		bt.Set(k, val)
	}
	for k, v := range mp {
		require.Equal(t, v, bt.Get(k))
	}
}

func TestTreeCycle(t *testing.T) {
	bt := NewTree("TestTreeCycle")
	defer func() { require.NoError(t, bt.Close()) }()

	val := uint64(0)
	for i := 0; i < 16; i++ {
		for j := 0; j < 1e6+i*1e4; j++ {
			val += 1
			bt.Set(rand.Uint64(), val)
		}
		before := bt.Stats()
		bt.DeleteBelow(val - 1e4)
		after := bt.Stats()
		t.Logf("Cycle %d Done. Before: %+v -> After: %+v\n", i, before, after)
	}

	bt.DeleteBelow(val)
	stats := bt.Stats()
	t.Logf("stats: %+v\n", stats)
	require.LessOrEqual(t, stats.Occupancy, 1.0)
	require.GreaterOrEqual(t, stats.NumPagesFree, int(float64(stats.NumPages)*0.95))
}

func TestTreeIterateKV(t *testing.T) {
	bt := NewTree("TestTreeIterateKV")
	defer func() { require.NoError(t, bt.Close()) }()

	// Set entries: (i, i*10)
	const n = uint64(1 << 20)
	for i := uint64(1); i <= n; i++ {
		bt.Set(i, i*10)
	}

	// Validate entries: (i, i*10)
	// Set entries: (i, i*20)
	count := uint64(0)
	bt.IterateKV(func(k, v uint64) uint64 {
		require.Equal(t, k*10, v)
		count++
		return k * 20
	})
	require.Equal(t, n, count)

	// Validate entries: (i, i*20)
	count = uint64(0)
	bt.IterateKV(func(k, v uint64) uint64 {
		require.Equal(t, k*20, v)
		count++
		return 0
	})
	require.Equal(t, n, count)
}

func TestOccupancyRatio(t *testing.T) {
	// atmax 4 keys per node
	setPageSize(16 * 5)
	defer setPageSize(os.Getpagesize())
	require.Equal(t, 4, maxKeys)

	bt := NewTree("TestOccupancyRatio")
	defer func() { require.NoError(t, bt.Close()) }()

	expectedRatio := float64(1) * 100 / float64(2*maxKeys) // 2 because we'll have 2 pages.
	stats := bt.Stats()
	t.Logf("Expected ratio: %.2f. MaxKeys: %d. Stats: %+v\n", expectedRatio, maxKeys, stats)
	require.InDelta(t, expectedRatio, stats.Occupancy, 0.01)
	for i := uint64(1); i <= 3; i++ {
		bt.Set(i, i)
	}
	// Tree structure will be:
	//    [2,Max,_,_]
	//  [1,2,_,_]  [3,Max,_,_]
	expectedRatio = float64(4) * 100 / float64(3*maxKeys)
	stats = bt.Stats()
	t.Logf("Expected ratio: %.2f. MaxKeys: %d. Stats: %+v\n", expectedRatio, maxKeys, stats)
	require.InDelta(t, expectedRatio, stats.Occupancy, 0.01)
	bt.DeleteBelow(2)
	// Tree structure will be:
	//    [2,Max,_]
	//  [2,_,_,_]  [3,Max,_,_]
	expectedRatio = float64(3) * 100 / float64(3*maxKeys)
	stats = bt.Stats()
	t.Logf("Expected ratio: %.2f. MaxKeys: %d. Stats: %+v\n", expectedRatio, maxKeys, stats)
	require.InDelta(t, expectedRatio, stats.Occupancy, 0.01)
}

func TestNode(t *testing.T) {
	n := getNode(make([]byte, pageSize))
	for i := uint64(1); i < 16; i *= 2 {
		n.set(i, i)
	}
	n.print(0)
	require.True(t, 0 == n.get(5))
	n.set(5, 5)
	n.print(0)

	n.setBit(0)
	require.False(t, n.isLeaf())
	n.setBit(bitLeaf)
	require.True(t, n.isLeaf())
}

func TestNodeBasic(t *testing.T) {
	n := getNode(make([]byte, pageSize))
	N := uint64(256)
	mp := make(map[uint64]uint64)
	for i := uint64(1); i < N; i++ {
		key := uint64(rand.Int63n(1<<60) + 1)
		n.set(key, key)
		mp[key] = key
	}
	for k, v := range mp {
		require.Equal(t, v, n.get(k))
	}
}

func TestNode_MoveRight(t *testing.T) {
	n := getNode(make([]byte, pageSize))
	N := uint64(10)
	for i := uint64(1); i < N; i++ {
		n.set(i, i)
	}
	n.moveRight(5)
	n.iterate(func(n node, i int) {
		if i < 5 {
			require.Equal(t, uint64(i+1), n.key(i))
			require.Equal(t, uint64(i+1), n.val(i))
		} else if i > 5 {
			require.Equal(t, uint64(i), n.key(i))
			require.Equal(t, uint64(i), n.val(i))
		}
	})
}

func TestNodeCompact(t *testing.T) {
	n := getNode(make([]byte, pageSize))
	n.setBit(bitLeaf)
	N := uint64(128)
	mp := make(map[uint64]uint64)
	for i := uint64(1); i < N; i++ {
		key := i
		val := uint64(10)
		if i%2 == 0 {
			val = 20
			mp[key] = 20
		}
		n.set(key, val)
	}

	require.Equal(t, int(N/2), n.compact(11))
	for k, v := range mp {
		require.Equal(t, v, n.get(k))
	}
	require.Equal(t, uint64(127), n.maxKey())
}

func BenchmarkPurge(b *testing.B) {
	N := 16 << 20
	b.Run("go-mem", func(b *testing.B) {
		m := make(map[uint64]uint64)
		for i := 0; i < N; i++ {
			m[rand.Uint64()] = uint64(i)
		}
	})

	b.Run("btree", func(b *testing.B) {
		start := time.Now()
		bt := NewTree("BenchmarkPurge")
		defer func() { require.NoError(b, bt.Close()) }()
		for i := 0; i < N; i++ {
			bt.Set(rand.Uint64(), uint64(i))
		}
		b.Logf("Populate took: %s. stats: %+v\n", time.Since(start), bt.Stats())

		start = time.Now()
		before := bt.Stats()
		bt.DeleteBelow(uint64(N - 1<<20))
		after := bt.Stats()
		b.Logf("Purge took: %s. Before: %+v After: %+v\n", time.Since(start), before, after)
	})
}

func BenchmarkWrite(b *testing.B) {
	b.Run("map", func(b *testing.B) {
		mp := make(map[uint64]uint64)
		for n := 0; n < b.N; n++ {
			k := rand.Uint64()
			mp[k] = k
		}
	})
	b.Run("btree", func(b *testing.B) {
		bt := NewTree("BenchmarkWrite")
		defer func() { require.NoError(b, bt.Close()) }()
		b.ResetTimer()
		for n := 0; n < b.N; n++ {
			k := rand.Uint64()
			bt.Set(k, k)
		}
	})
}

// goos: linux
// goarch: amd64
// pkg: github.com/dgraph-io/ristretto/z
// BenchmarkRead/map-4         	10845322	       109 ns/op
// BenchmarkRead/btree-4       	 2744283	       430 ns/op
// Cumulative for 10 runs.
// name          time/op
// Read/map-4    105ns ± 1%
// Read/btree-4  422ns ± 1%
func BenchmarkRead(b *testing.B) {
	N := 10 << 20
	mp := make(map[uint64]uint64)
	for i := 0; i < N; i++ {
		k := uint64(rand.Intn(2*N)) + 1
		mp[k] = k
	}
	b.Run("map", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			k := uint64(rand.Intn(2 * N))
			v, ok := mp[k]
			_, _ = v, ok
		}
	})

	bt := NewTree("BenchmarkRead")
	defer func() { require.NoError(b, bt.Close()) }()
	for i := 0; i < N; i++ {
		k := uint64(rand.Intn(2*N)) + 1
		bt.Set(k, k)
	}
	stats := bt.Stats()
	fmt.Printf("Num pages: %d Size: %s\n", stats.NumPages,
		humanize.IBytes(uint64(stats.Bytes)))
	fmt.Println("Writes done.")

	b.Run("btree", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			k := uint64(rand.Intn(2*N)) + 1
			v := bt.Get(k)
			_ = v
		}
	})
}

func BenchmarkSearch(b *testing.B) {
	linear := func(n node, k uint64, N int) int {
		for i := 0; i < N; i++ {
			if ki := n.key(i); ki >= k {
				return i
			}
		}
		return N
	}
	binary := func(n node, k uint64, N int) int {
		return sort.Search(N, func(i int) bool {
			return n.key(i) >= k
		})
	}
	unroll4 := func(n node, k uint64, N int) int {
		if len(n[:2*N]) < 8 {
			for i := 0; i < N; i++ {
				if ki := n.key(i); ki >= k {
					return i
				}
			}
			return N
		}
		return int(simd.Search(n[:2*N], k))
	}

	jumpBy := []int{8, 16, 32, 64, 128, 196, 255}
	for _, sz := range jumpBy {
		f, err := os.CreateTemp(".", "tree")
		require.NoError(b, err)

		mf, err := OpenMmapFileUsing(f, pageSize, true)
		if err != NewFile {
			require.NoError(b, err)
		}

		n := getNode(mf.Data)
		for i := 1; i <= sz; i++ {
			n.set(uint64(i), uint64(i))
		}

		b.Run(fmt.Sprintf("linear-%d", sz), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				tmp = linear(n, math.MaxUint64, sz)
			}
		})
		b.Run(fmt.Sprintf("binary-%d", sz), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				tmp = binary(n, uint64(sz), sz)
			}
		})
		b.Run(fmt.Sprintf("unrolled-asm-%d", sz), func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				tmp = unroll4(n, math.MaxUint64, sz)
			}
		})
		mf.Close(0)
		os.Remove(f.Name())
	}
}

// This benchmark when run on dgus-delta, performed marginally better with threshold=32.
// CustomSearch/sz-64_th-1-4     49.9ns ± 1% (fully binary)
// CustomSearch/sz-64_th-16-4    63.3ns ± 0%
// CustomSearch/sz-64_th-32-4    58.7ns ± 7%
// CustomSearch/sz-64_th-64-4    63.9ns ± 7% (fully linear)

// CustomSearch/sz-128_th-32-4   70.2ns ± 1%

// CustomSearch/sz-255_th-1-4    77.3ns ± 0% (fully binary)
// CustomSearch/sz-255_th-16-4   68.2ns ± 1%
// CustomSearch/sz-255_th-32-4   67.0ns ± 7%
// CustomSearch/sz-255_th-64-4   85.5ns ±19%
// CustomSearch/sz-255_th-256-4   129ns ± 6% (fully linear)

func BenchmarkCustomSearch(b *testing.B) {
	mixed := func(n node, k uint64, N int, threshold int) int {
		lo, hi := 0, N
		// Reduce the search space using binary seach and then do linear search.
		for hi-lo > threshold {
			mid := (hi + lo) / 2
			km := n.key(mid)
			if k == km {
				return mid
			}
			if k > km {
				// key is greater than the key at mid, so move right.
				lo = mid + 1
			} else {
				// else move left.
				hi = mid
			}
		}
		for i := lo; i <= hi; i++ {
			if ki := n.key(i); ki >= k {
				return i
			}
		}
		return N
	}

	for _, sz := range []int{64, 128, 255} {
		n := getNode(make([]byte, pageSize))
		for i := 1; i <= sz; i++ {
			n.set(uint64(i), uint64(i))
		}

		mk := sz + 1
		for th := 1; th <= sz+1; th *= 2 {
			b.Run(fmt.Sprintf("sz-%d th-%d", sz, th), func(b *testing.B) {
				for i := 0; i < b.N; i++ {
					k := uint64(rand.Intn(mk))
					tmp = mixed(n, k, sz, th)
				}
			})
		}
	}
}
