package cachekv

import (
	"strconv"
	"testing"

	"cosmossdk.io/store/cachekv/internal"
)

func BenchmarkLargeUnsortedMisses(b *testing.B) {
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		store := generateStore()
		b.StartTimer()

		for k := 0; k < 10000; k++ {
			// cache has A + Z values
			// these are within range, but match nothing
			store.dirtyItems([]byte("B1"), []byte("B2"))
		}
	}
}

func generateStore() *Store {
	cache := map[string]*cValue{}
	unsorted := map[string]struct{}{}
	for i := 0; i < 5000; i++ {
		key := "A" + strconv.Itoa(i)
		unsorted[key] = struct{}{}
		cache[key] = &cValue{}
	}

	for i := 0; i < 5000; i++ {
		key := "Z" + strconv.Itoa(i)
		unsorted[key] = struct{}{}
		cache[key] = &cValue{}
	}

	return &Store{
		cache:         cache,
		unsortedCache: unsorted,
		sortedCache:   internal.NewBTree(),
	}
}
