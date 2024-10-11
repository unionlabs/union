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
	"sync"
	"testing"
	"time"

	"math/rand"

	"github.com/stretchr/testify/require"
)

// $ go test -failfast -run xxx -bench . -benchmem  -count 10 > out.txt
// $ benchstat out.txt
// name                 time/op
// Allocation/Pool-8    200µs ± 5%
// Allocation/Calloc-8  100µs ±11%
//
// name                 alloc/op
// Allocation/Pool-8     477B ±29%
// Allocation/Calloc-8  4.00B ± 0%
//
// name                 allocs/op
// Allocation/Pool-8     1.00 ± 0%
// Allocation/Calloc-8   0.00
func BenchmarkAllocation(b *testing.B) {
	b.Run("Pool", func(b *testing.B) {
		pool := sync.Pool{
			New: func() interface{} {
				return make([]byte, 4<<10)
			},
		}
		b.RunParallel(func(pb *testing.PB) {
			source := rand.NewSource(time.Now().UnixNano())
			r := rand.New(source)
			for pb.Next() {
				x := pool.Get().([]byte)
				sz := r.Intn(100) << 10
				if len(x) < sz {
					x = make([]byte, sz)
				}
				r.Read(x)
				//nolint:staticcheck
				pool.Put(x)
			}
		})
	})

	b.Run("Calloc", func(b *testing.B) {
		b.RunParallel(func(pb *testing.PB) {
			source := rand.NewSource(time.Now().UnixNano())
			r := rand.New(source)
			for pb.Next() {
				sz := r.Intn(100) << 10
				x := Calloc(sz, "test")
				r.Read(x)
				Free(x)
			}
		})
	})
}

func TestCalloc(t *testing.T) {
	// Check if we're using jemalloc.
	// JE_MALLOC_CONF="abort:true,tcache:false"

	StatsPrint()
	buf := CallocNoRef(1, "test")
	if len(buf) == 0 {
		t.Skipf("Not using jemalloc. Skipping test.")
	}
	Free(buf)
	require.Equal(t, int64(0), NumAllocBytes())

	buf1 := Calloc(128, "test")
	require.Equal(t, int64(128), NumAllocBytes())
	buf2 := Calloc(128, "test")
	require.Equal(t, int64(256), NumAllocBytes())

	Free(buf1)
	require.Equal(t, int64(128), NumAllocBytes())

	// _ = buf2
	Free(buf2)
	require.Equal(t, int64(0), NumAllocBytes())
	fmt.Println(Leaks())

	// Double free would panic when debug mode is enabled in jemalloc.
	// Free(buf2)
	// require.Equal(t, int64(0), NumAllocBytes())
}
