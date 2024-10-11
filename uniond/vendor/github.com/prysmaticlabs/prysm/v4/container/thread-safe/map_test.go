package threadsafe

import (
	"sort"
	"sync"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

type safeMap struct {
	items map[int]string
	lock  sync.RWMutex
}

func (s *safeMap) Get(k int) (string, bool) {
	s.lock.RLock()
	defer s.lock.RUnlock()
	v, ok := s.items[k]
	return v, ok
}

func (s *safeMap) Put(i int, str string) {
	s.lock.Lock()
	defer s.lock.Unlock()
	s.items[i] = str
}

func (s *safeMap) Delete(i int) {
	s.lock.Lock()
	defer s.lock.Unlock()
	delete(s.items, i)
}

func BenchmarkMap_Concrete(b *testing.B) {
	mm := &safeMap{
		items: make(map[int]string),
	}
	for i := 0; i < b.N; i++ {
		for j := 0; j < 1000; j++ {
			mm.Put(j, "foo")
			mm.Get(j)
			mm.Delete(j)
		}
	}
}

func BenchmarkMap_Generic(b *testing.B) {
	items := make(map[int]string)
	mm := NewThreadSafeMap(items)
	for i := 0; i < b.N; i++ {
		for j := 0; j < 1000; j++ {
			mm.Put(j, "foo")
			mm.Get(j)
			mm.Delete(j)
		}
	}
}
func BenchmarkMap_GenericTx(b *testing.B) {
	items := make(map[int]string)
	mm := NewThreadSafeMap(items)
	for i := 0; i < b.N; i++ {
		for j := 0; j < 1000; j++ {
			mm.Do(func(mp map[int]string) {
				mp[j] = "foo"
				_ = mp[j]
				delete(mp, j)
			})
		}
	}
}

func TestMap(t *testing.T) {
	m := map[int]string{
		1:     "foo",
		200:   "bar",
		10000: "baz",
	}

	tMap := NewThreadSafeMap(m)
	keys := tMap.Keys()
	sort.IntSlice(keys).Sort()

	require.DeepEqual(t, []int{1, 200, 10000}, keys)
	require.Equal(t, 3, tMap.Len())

	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func(w *sync.WaitGroup, scopedMap *Map[int, string]) {
			defer w.Done()
			v, ok := scopedMap.Get(1)
			require.Equal(t, true, ok)
			require.Equal(t, "foo", v)

			scopedMap.Put(3, "nyan")

			v, ok = scopedMap.Get(3)
			require.Equal(t, true, ok)
			require.Equal(t, "nyan", v)

		}(&wg, tMap)
	}
	wg.Wait()

	tMap.Delete(3)

	_, ok := tMap.Get(3)
	require.Equal(t, false, ok)
}
