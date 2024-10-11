package btree

import (
	"math/rand"
	"reflect"
	"sort"
	"sync"
	"testing"
	"time"
)

func TestSet(t *testing.T) {
	N := 1_000_000
	var tr Set[int]
	for i := 0; i < N; i++ {
		tr.Load(i)
	}
	assert(tr.Len() == N)
	for i := 0; i < N; i++ {
		assert(tr.Contains(i))
	}

	count := 0
	tr.Scan(func(_ int) bool {
		count++
		return true
	})
	assert(count == N)
	count = 0
	tr.Ascend(N/2, func(_ int) bool {
		count++
		return true
	})
	assert(count == N/2)

	count = 0
	tr.Reverse(func(_ int) bool {
		count++
		return true
	})
	assert(count == N)
	count = 0
	tr.Descend(N/2, func(_ int) bool {
		count++
		return true
	})
	assert(count == N/2+1)

	for i := 0; i < N; i++ {
		tr.Delete(i)
	}

	dotup := func(v int, ok bool) interface{} {
		if !ok {
			return nil
		}
		return v
	}

	assert(tr.Len() == 0)
	assert(dotup(tr.Min()) == nil)
	assert(dotup(tr.Max()) == nil)
	assert(dotup(tr.PopMin()) == nil)
	assert(dotup(tr.PopMax()) == nil)
	for i := 0; i < N; i++ {
		assert(!tr.Contains(i))
	}
	for i := 0; i < N; i++ {
		tr.Insert(i)
	}
	assert(tr.Len() == N)
	for i := 0; i < N; i++ {
		tr.Insert(i)
	}
	assert(tr.Len() == N)
	for i := 0; i < N; i++ {
		tr.Load(i)
	}
	assert(tr.Len() == N)
	assert(dotup(tr.Min()) == 0)
	assert(dotup(tr.Max()) == N-1)
	assert(dotup(tr.PopMin()) == 0)
	assert(dotup(tr.PopMax()) == N-1)
	tr.Insert(0)
	tr.Insert(N - 1)
	assert(dotup(tr.GetAt(0)) == 0)
	assert(dotup(tr.GetAt(N)) == nil)
	tr.Insert(N - 1)
	assert(tr.Height() > 0)
	assert(dotup(tr.DeleteAt(0)) == 0)
	tr.Insert(0)
	assert(dotup(tr.DeleteAt(N-1)) == N-1)
	assert(dotup(tr.DeleteAt(N)) == nil)
	tr.Insert(N - 1)

	count = 0
	tr.Scan(func(item int) bool {
		count++
		return true
	})

	assert(count == N)

	func() {
		defer func() {
			msg, ok := recover().(string)
			assert(ok && msg == "nil item")
		}()
		tr := NewNonConcurrent(intLess)
		tr.Set(nil)
	}()
	func() {
		defer func() {
			msg, ok := recover().(string)
			assert(ok && msg == "nil item")
		}()
		tr := NewNonConcurrent(intLess)
		tr.Load(nil)
	}()
	for i := 0; i < N; i++ {
		assert(tr.Contains(i))
	}
	for i := 0; i < N; i++ {
		tr.Delete(i)
	}
	for i := 0; i < N; i++ {
		assert(!tr.Contains(i))
	}
	assert(tr.base.lt(1, 2))
	assert(tr.base.lt(2, 10))
}

func TestSetClear(t *testing.T) {
	var tr Set[int]
	for i := 0; i < 100; i++ {
		tr.Insert(i)
	}
	assert(tr.Len() == 100)
	tr.Clear()
	assert(tr.Len() == 0)
	for i := 0; i < 100; i++ {
		tr.Insert(i)
	}
	assert(tr.Len() == 100)
}

func TestSetIter(t *testing.T) {
	N := 100_000
	lt := func(a, b int) bool { return a < b }
	eq := func(a, b int) bool { return !lt(a, b) && !lt(b, a) }
	var tr Set[int]
	var all []int
	for i := 0; i < N; i++ {
		tr.Load(i)
		all = append(all, i)
	}
	var count int
	var i int
	iter := tr.Iter()
	for ok := iter.First(); ok; ok = iter.Next() {
		if !eq(all[i], iter.Key()) {
			panic("!")
		}
		count++
		i++
	}
	if count != N {
		t.Fatalf("expected %v, got %v", N, count)
	}

	count = 0
	i = len(all) - 1
	iter = tr.Iter()
	for ok := iter.Last(); ok; ok = iter.Prev() {
		if !eq(all[i], iter.Key()) {
			panic("!")
		}
		i--
		count++
	}
	if count != N {
		t.Fatalf("expected %v, got %v", N, count)
	}

	i = 0
	iter = tr.Iter()
	for ok := iter.First(); ok; ok = iter.Next() {
		if !eq(all[i], iter.Key()) {
			panic("!")
		}
		i++
	}
	i--
	for ok := iter.Prev(); ok; ok = iter.Prev() {
		i--
		if !eq(all[i], iter.Key()) {
			panic("!")
		}

	}
	if i != 0 {
		panic("!")
	}

	i++
	for ok := iter.Next(); ok; ok = iter.Next() {
		if !eq(all[i], iter.Key()) {
			panic("!")
		}
		i++

	}
	if i != N {
		panic("!")
	}

	i = 0
	for ok := iter.First(); ok; ok = iter.Next() {
		if !eq(all[i], iter.Key()) {
			panic("!")
		}
		if eq(iter.Key(), N/2) {
			for ok = iter.Prev(); ok; ok = iter.Prev() {
				i--
				if !eq(all[i], iter.Key()) {
					panic("!")
				}
			}
			break
		}
		i++
	}
}

func copySetEntries(m *Set[int]) []int {
	all := m.Keys()
	sort.Ints(all)
	return all
}

func setEntriesEqual(a, b []int) bool {
	return reflect.DeepEqual(a, b)
}

func copySetTest(N int, s1 *Set[int], e11 []int, deep bool) {
	e12 := copySetEntries(s1)
	if !setEntriesEqual(e11, e12) {
		panic("!")
	}

	// Make a copy and compare the values
	s2 := s1.Copy()
	e21 := copySetEntries(s1)
	if !setEntriesEqual(e21, e12) {
		panic("!")
	}

	// Delete every other key
	var e22 []int
	for i, j := range rand.Perm(N) {
		if i&1 == 0 {

			e22 = append(e22, e21[j])
		} else {
			s2.Delete(e21[j])
		}
	}

	if s2.Len() != N/2 {
		panic("!")
	}
	sort.Ints(e22)
	e23 := copySetEntries(s2)
	if !setEntriesEqual(e23, e22) {
		panic("!")
	}
	if !deep {
		var wg sync.WaitGroup
		wg.Add(2)
		go func() {
			defer wg.Done()
			copySetTest(N/2, s2, e23, true)
		}()
		go func() {
			defer wg.Done()
			copySetTest(N/2, s2, e23, true)
		}()
		wg.Wait()
	}
	e24 := copySetEntries(s2)
	if !setEntriesEqual(e24, e23) {
		panic("!")
	}

}

func TestSetCopy(t *testing.T) {
	N := 1_000
	// create the initial map

	s1 := new(Set[int])
	for s1.Len() < N {
		s1.Insert(rand.Int())
	}
	e11 := copySetEntries(s1)
	dur := time.Second * 2
	var wg sync.WaitGroup
	for i := 0; i < 16; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			start := time.Now()
			for time.Since(start) < dur {
				copySetTest(N, s1, e11, false)
			}
		}()
	}
	wg.Wait()
	e12 := copySetEntries(s1)
	if !setEntriesEqual(e11, e12) {
		panic("!")
	}
}
