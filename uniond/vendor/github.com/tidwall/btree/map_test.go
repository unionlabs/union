package btree

import (
	"fmt"
	"math/rand"
	"reflect"
	"sort"
	"sync"
	"testing"
	"time"
)

type testMapKind = int

func testMapMakeItem(x int) (item testMapKind) {
	return x
}

// testNewBTree must return an operational btree for testing.
func testMapNewBTree() *Map[testMapKind, testMapKind] {
	return new(Map[testMapKind, testMapKind])
}

func testMapNewBTreeDegrees(degree int) *Map[testMapKind, testMapKind] {
	return NewMap[testMapKind, testMapKind](degree)
}

func randMapKeys(N int) (keys []testMapKind) {
	keys = make([]testMapKind, N)
	for _, i := range rand.Perm(N) {
		keys[i] = testMapMakeItem(i)
	}
	return keys
}

func (tr *Map[K, V]) lt(a, b K) bool  { return a < b }
func (tr *Map[K, V]) eq(a, b K) bool  { return !(tr.lt(a, b) || tr.lt(b, a)) }
func (tr *Map[K, V]) lte(a, b K) bool { return tr.lt(a, b) || tr.eq(a, b) }
func (tr *Map[K, V]) gt(a, b K) bool  { return tr.lt(b, a) }
func (tr *Map[K, V]) gte(a, b K) bool { return tr.gt(a, b) || tr.eq(a, b) }

func mapKindsAreEqual(a, b []testMapKind) bool {
	var tr Map[testMapKind, testMapKind]
	if len(a) != len(b) {
		return false
	}
	for i := 0; i < len(a); i++ {
		if !tr.eq(a[i], b[i]) {
			return false
		}
	}
	return true
}

func TestMapMakeItemOrder(t *testing.T) {
	tr := testMapNewBTree()
	ints := []int{0, 1, 2, 3, 4, 10, 20, 30, 40, 100, 200, 300, 400}
	for i := 0; i < len(ints)-1; i++ {
		a := testMapMakeItem(ints[i])
		b := testMapMakeItem(ints[i+1])
		if !tr.lt(a, b) {
			t.Fatalf("bad ordering for testMakeItem: '%v' !< '%v'", a, b)
		}
	}
}

func TestMapDescend(t *testing.T) {
	tr := testMapNewBTree()
	var count int
	tr.Descend(testMapMakeItem(rand.Int()), func(item, value testMapKind) bool {
		count++
		return true
	})
	if count > 0 {
		t.Fatalf("expected 0, got %v", count)
	}
	var keys []testMapKind
	for i := 0; i < 1000; i += 10 {
		keys = append(keys, testMapMakeItem(i))
		tr.Set(keys[len(keys)-1], keys[len(keys)-1])
	}
	var exp []testMapKind
	tr.Reverse(func(item, value testMapKind) bool {
		exp = append(exp, item)
		return true
	})
	for i := 999; i >= 0; i-- {
		key := testMapMakeItem(i)
		var all []testMapKind
		tr.Descend(key, func(item, value testMapKind) bool {
			all = append(all, item)
			return true
		})
		for len(exp) > 0 && tr.lt(key, exp[0]) {
			exp = exp[1:]
		}
		var count int
		tr.Descend(key, func(item, value testMapKind) bool {
			if count == (i+1)%tr.max {
				return false
			}
			count++
			return true
		})
		if count > len(exp) {
			t.Fatalf("expected 1, got %v", count)
		}
		if !mapKindsAreEqual(exp, all) {
			fmt.Printf("exp: %v\n", exp)
			fmt.Printf("all: %v\n", all)
			t.Fatal("mismatch")
		}
		for j := 0; j < tr.Len(); j++ {
			count = 0
			tr.Descend(key, func(item, value testMapKind) bool {
				if count == j {
					return false
				}
				count++
				return true
			})
		}
	}
}

func TestMapAscend(t *testing.T) {
	tr := testMapNewBTree()
	var count int
	tr.Ascend(testMapMakeItem(1), func(item, value testMapKind) bool {
		count++
		return true
	})
	if count > 0 {
		t.Fatalf("expected 0, got %v", count)
	}
	var keys []testMapKind
	for i := 0; i < 1000; i += 10 {
		keys = append(keys, testMapMakeItem(i))
		tr.Set(keys[len(keys)-1], keys[len(keys)-1])
		tr.sane()
	}
	exp := keys
	for i := -1; i < 1000; i++ {
		key := testMapMakeItem(i)
		var all []testMapKind
		tr.Ascend(key, func(item, value testMapKind) bool {
			all = append(all, item)
			return true
		})
		for len(exp) > 0 && tr.lt(exp[0], key) {
			exp = exp[1:]
		}
		var count int
		tr.Ascend(key, func(item, value testMapKind) bool {
			if count == (i+1)%tr.max {
				return false
			}
			count++
			return true
		})
		if count > len(exp) {
			t.Fatalf("expected 1, got %v", count)
		}
		if !mapKindsAreEqual(exp, all) {
			t.Fatal("mismatch")
		}
	}
}

func TestMapKeyValues(t *testing.T) {
	tr := testMapNewBTree()
	if len(tr.Keys()) != 0 {
		t.Fatalf("expected 0, got %v", len(tr.Keys()))
	}
	if len(tr.Values()) != 0 {
		t.Fatalf("expected 0, got %v", len(tr.Values()))
	}
	keys, values := tr.KeyValues()
	if len(keys) != 0 {
		t.Fatalf("expected 0, got %v", len(keys))
	}
	if len(values) != 0 {
		t.Fatalf("expected 0, got %v", len(values))
	}
	keys = nil
	values = nil
	for i := 0; i < 100000; i += 10 {
		keys = append(keys, testMapMakeItem(i))
		values = append(values, testMapMakeItem(i)*10)
		tr.Set(keys[len(keys)-1], values[len(values)-1])
		tr.sane()
	}
	keys2 := tr.Keys()
	values2 := tr.Values()
	if !kindsAreEqual(keys, keys2) {
		t.Fatalf("not equal")
	}
	if !kindsAreEqual(values, values2) {
		t.Fatalf("not equal")
	}
	keys2, values2 = tr.KeyValues()
	if !kindsAreEqual(keys, keys2) {
		t.Fatalf("not equal")
	}
	if !kindsAreEqual(values, values2) {
		t.Fatalf("not equal")
	}
}

func TestMapSimpleRandom(t *testing.T) {
	start := time.Now()
	for time.Since(start) < time.Second*2 {
		N := 100_000
		items := randMapKeys(N)
		tr := testMapNewBTree()
		tr.sane()
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Get(items[i]); ok || !tr.eq(v, tr.empty.value) {
				panic("!")
			}
			if v, ok := tr.Set(items[i], items[i]); ok || !tr.eq(v, tr.empty.value) {
				panic("!")
			}
			if v, ok := tr.Get(items[i]); !ok || !tr.eq(v, items[i]) {
				panic("!")
			}
		}
		tr.sane()
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Set(items[i], items[i]); !ok || !tr.eq(v, items[i]) {
				panic("!")
			}
		}
		pivot := items[len(items)/2]
		tr.Ascend(pivot, func(item, value testMapKind) bool {
			if tr.lt(item, pivot) {
				panic("!")
			}
			return true
		})
		var min testMapKind
		index := 0
		tr.Scan(func(item, value testMapKind) bool {
			if index == len(items)/2 {
				return false
			}
			if index > 0 {
				if tr.lt(item, min) {
					panic("!")
				}
			}
			min = item
			index++
			return true
		})
		tr.sane()
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Delete(items[i]); !ok || !tr.eq(v, items[i]) {
				panic("!")
			}
			if i%97 == 0 {
				tr.sane()
			}
			if v, ok := tr.Delete(items[i]); ok || !tr.eq(v, tr.empty.value) {
				panic("!")
			}
		}
		if tr.Len() != 0 {
			panic("!")
		}
		tr.sane()
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Delete(items[i]); ok || !tr.eq(v, tr.empty.value) {
				panic("!")
			}
		}
		tr.sane()
		tr.Scan(func(item, value testMapKind) bool {
			panic("!")
		})
	}
}

func TestMapBTree(t *testing.T) {
	N := 10000
	tr := testMapNewBTree()
	tr.sane()
	keys := randMapKeys(N)

	// insert all items
	for _, key := range keys {
		if v, ok := tr.Set(key, key); ok || !tr.eq(v, tr.empty.value) {
			t.Fatal("expected false")
		}
		tr.sane()
	}

	// check length
	if tr.Len() != len(keys) {
		t.Fatalf("expected %v, got %v", len(keys), tr.Len())
	}

	// get each value
	for _, key := range keys {
		if v, ok := tr.Get(key); !ok || !tr.eq(v, key) {
			t.Fatalf("expected '%v', got '%v'", key, v)
		}
	}

	// scan all items
	var prev testMapKind
	var count int
	tr.Scan(func(item, value testMapKind) bool {
		if count > 0 {
			if tr.lte(item, prev) {
				t.Fatal("out of order")
			}
		}
		prev = item
		count++
		return true
	})
	if count != len(keys) {
		t.Fatalf("expected '%v', got '%v'", len(keys), count)
	}

	// reverse all items
	count = 0
	tr.Reverse(func(item, value testMapKind) bool {
		if count > 0 {
			if tr.gte(item, prev) {
				t.Fatal("out of order")
			}
		}
		prev = item
		count++
		return true
	})
	if count != len(keys) {
		t.Fatalf("expected '%v', got '%v'", len(keys), count)
	}

	// try to get an invalid item
	if v, ok := tr.Get(testMapMakeItem(-1)); ok || !tr.eq(v, tr.empty.value) {
		t.Fatal("expected nil")
	}

	// scan and quit at various steps
	for i := 0; i < 100; i++ {
		var j int
		tr.Scan(func(item, value testMapKind) bool {
			if j == i {
				return false
			}
			j++
			return true
		})
	}

	// reverse and quit at various steps
	for i := 0; i < 100; i++ {
		var j int
		tr.Reverse(func(item, value testMapKind) bool {
			if j == i {
				return false
			}
			j++
			return true
		})
	}

	// delete half the items
	for _, key := range keys[:len(keys)/2] {
		if v, ok := tr.Delete(key); !ok || !tr.eq(v, key) {
			t.Fatalf("expected '%v', got '%v'", key, v)
		}
	}

	// check length
	if tr.Len() != len(keys)/2 {
		t.Fatalf("expected %v, got %v", len(keys)/2, tr.Len())
	}

	// try delete half again
	for _, key := range keys[:len(keys)/2] {
		if v, ok := tr.Delete(key); ok || !tr.eq(v, tr.empty.value) {
			t.Fatal("expected false")
		}
		tr.sane()
	}

	// check length
	if tr.Len() != len(keys)/2 {
		t.Fatalf("expected %v, got %v", len(keys)/2, tr.Len())
	}

	// scan items
	count = 0
	tr.Scan(func(item, value testMapKind) bool {
		if count > 0 {
			if tr.lte(item, prev) {
				t.Fatal("out of order")
			}
		}
		prev = item
		count++
		return true
	})
	if count != len(keys)/2 {
		t.Fatalf("expected '%v', got '%v'", len(keys), count)
	}

	// replace second half
	for _, key := range keys[len(keys)/2:] {
		if v, ok := tr.Set(key, key); !ok || !tr.eq(v, key) {
			t.Fatalf("expected '%v', got '%v'", key, v)
		}
		tr.sane()
	}

	// delete next half the items
	for _, key := range keys[len(keys)/2:] {
		if v, ok := tr.Delete(key); !ok || !tr.eq(v, key) {
			t.Fatalf("expected '%v', got '%v'", key, v)
		}
		tr.sane()
	}

	// check length
	if tr.Len() != 0 {
		t.Fatalf("expected %v, got %v", 0, tr.Len())
	}

	// do some stuff on an empty tree
	if v, ok := tr.Get(keys[0]); ok || !tr.eq(v, tr.empty.value) {
		t.Fatal("expected nil")
	}
	tr.Scan(func(item, value testMapKind) bool {
		t.Fatal("should not be reached")
		return true
	})
	tr.Reverse(func(item, value testMapKind) bool {
		t.Fatal("should not be reached")
		return true
	})
	if v, ok := tr.Delete(testMapMakeItem(-1)); ok || !tr.eq(v, tr.empty.value) {
		t.Fatal("expected nil")
	}
	tr.sane()
}

func TestMapBTreeOne(t *testing.T) {
	tr := testMapNewBTree()
	tr.Set(testMapMakeItem(1), testMapMakeItem(1))
	tr.Delete(testMapMakeItem(1))
	tr.Set(testMapMakeItem(1), testMapMakeItem(1))
	tr.Delete(testMapMakeItem(1))
	tr.Set(testMapMakeItem(1), testMapMakeItem(1))
	tr.Delete(testMapMakeItem(1))
	if tr.Len() != 0 {
		panic("!")
	}
	tr.sane()
}

func TestMapBTree256(t *testing.T) {
	for degree := -1; degree < 256; degree++ {
		tr := testMapNewBTreeDegrees(degree)
		var n int
		for j := 0; j < 2; j++ {
			for _, i := range rand.Perm(256) {
				tr.Set(testMapMakeItem(i), testMapMakeItem(i))
				n++
				if tr.Len() != n {
					t.Fatalf("expected 256, got %d", n)
				}
			}
			for _, i := range rand.Perm(256) {
				if v, ok := tr.Get(testMapMakeItem(i)); !ok || !tr.eq(v, testMapMakeItem(i)) {
					t.Fatalf("expected %v, got %v", i, v)
				}
			}
			for _, i := range rand.Perm(256) {
				tr.Delete(testMapMakeItem(i))
				n--
				if tr.Len() != n {
					t.Fatalf("expected 256, got %d", n)
				}
			}
			for _, i := range rand.Perm(256) {
				if v, ok := tr.Get(testMapMakeItem(i)); ok || !tr.eq(v, tr.empty.value) {
					t.Fatal("expected nil")
				}
			}
		}
	}
}

func shuffleMapItems(keys []testMapKind) {
	for i := range keys {
		j := rand.Intn(i + 1)
		keys[i], keys[j] = keys[j], keys[i]
	}
}

func sortMapItems(keys []testMapKind) {
	tr := testMapNewBTree()
	sort.Slice(keys, func(i, j int) bool {
		return tr.lt(keys[i], keys[j])
	})
}

func TestMapRandom(t *testing.T) {
	N := 200000
	keys := randMapKeys(N)
	tr := testMapNewBTree()
	tr.sane()
	if _, v, ok := tr.Min(); ok || !tr.eq(v, tr.empty.value) {
		t.Fatalf("expected nil")
	}
	if _, v, ok := tr.Max(); ok || !tr.eq(v, tr.empty.value) {
		t.Fatalf("expected nil")
	}
	if _, v, ok := tr.PopMin(); ok || !tr.eq(v, tr.empty.value) {
		t.Fatalf("expected nil")
	}
	if _, v, ok := tr.PopMax(); ok || !tr.eq(v, tr.empty.value) {
		t.Fatalf("expected nil")
	}
	if tr.Height() != 0 {
		t.Fatalf("expected 0, got %d", tr.Height())
	}
	tr.sane()
	shuffleMapItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Set(keys[i], keys[i]); ok || !tr.eq(v, tr.empty.value) {
			t.Fatalf("expected nil")
		}
		if i%123 == 0 {
			tr.sane()
		}
	}
	tr.sane()
	sortMapItems(keys)
	var n int
	tr.Scan(func(item, value testMapKind) bool {
		n++
		return false
	})
	if n != 1 {
		t.Fatalf("expected 1, got %d", n)
	}

	n = 0
	tr.Scan(func(item, value testMapKind) bool {
		if !tr.eq(item, keys[n]) {
			t.Fatalf("expected %v, got %v", keys[n], item)
		}
		n++
		return true
	})
	if n != len(keys) {
		t.Fatalf("expected %d, got %d", len(keys), n)
	}
	if tr.Len() != len(keys) {
		t.Fatalf("expected %d, got %d", tr.Len(), len(keys))
	}

	for i := 0; i < tr.Len(); i++ {
		if _, v, ok := tr.GetAt(i); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected %v, got %v", keys[i], v)
		}
	}

	n = 0
	tr.Reverse(func(item, value testMapKind) bool {
		n++
		return false
	})
	if n != 1 {
		t.Fatalf("expected 1, got %d", n)
	}
	n = 0
	tr.Reverse(func(item, value testMapKind) bool {
		if !tr.eq(item, keys[len(keys)-n-1]) {
			t.Fatalf("expected %v, got %v", keys[len(keys)-n-1], item)
		}
		n++
		return true
	})
	if n != len(keys) {
		t.Fatalf("expected %d, got %d", len(keys), n)
	}
	if tr.Len() != len(keys) {
		t.Fatalf("expected %d, got %d", tr.Len(), len(keys))
	}

	tr.sane()

	n = 0
	for i := 0; i < 1000; i++ {
		n := 0
		tr.Scan(func(item, value testMapKind) bool {
			if n == i {
				return false
			}
			n++
			return true
		})
		if n != i {
			t.Fatalf("expected %d, got %d", i, n)
		}
	}

	n = 0
	for i := 0; i < 1000; i++ {
		n = 0
		tr.Reverse(func(item, value testMapKind) bool {
			if n == i {
				return false
			}
			n++
			return true
		})
		if n != i {
			t.Fatalf("expected %d, got %d", i, n)
		}
	}

	sortMapItems(keys)
	for i := 0; i < len(keys); i++ {
		var res testMapKind
		var j int
		tr.Ascend(keys[i], func(item, value testMapKind) bool {
			if j == 0 {
				res = item
			}
			j++
			return j == i%500
		})
		if !tr.eq(res, keys[i]) {
			t.Fatal("not equal")
		}
	}
	for i := len(keys) - 1; i >= 0; i-- {
		var res testMapKind
		var j int
		tr.Descend(keys[i], func(item, value testMapKind) bool {
			if j == 0 {
				res = item
			}
			j++
			return j == i%500
		})
		if !tr.eq(res, keys[i]) {
			t.Fatal("not equal")
		}
	}

	if tr.Height() == 0 {
		t.Fatalf("expected non-zero")
	}
	if _, v, ok := tr.Min(); !ok || !tr.eq(v, keys[0]) {
		t.Fatalf("expected '%v', got '%v'", keys[0], v)
	}
	if _, v, ok := tr.Max(); !ok || !tr.eq(v, keys[len(keys)-1]) {
		t.Fatalf("expected '%v', got '%v'", keys[len(keys)-1], v)
	}
	if _, v, ok := tr.PopMin(); !ok || !tr.eq(v, keys[0]) {
		t.Fatalf("expected '%v', got '%v'", keys[0], v)
	}
	tr.sane()
	if _, v, ok := tr.PopMax(); !ok || !tr.eq(v, keys[len(keys)-1]) {
		t.Fatalf("expected '%v', got '%v'", keys[len(keys)-1], v)
	}
	tr.sane()
	tr.Set(keys[0], keys[0])
	tr.Set(keys[len(keys)-1], keys[len(keys)-1])
	shuffleMapItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Get(keys[i]); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
		if v, ok := tr.Get(keys[i]); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
	}
	sortMapItems(keys)
	for i := 0; i < len(keys); i++ {
		if _, v, ok := tr.PopMin(); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
	}
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Set(keys[i], keys[i]); ok || !tr.eq(v, tr.empty.value) {
			t.Fatalf("expected nil")
		}
	}
	for i := len(keys) - 1; i >= 0; i-- {
		if _, v, ok := tr.PopMax(); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
	}
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Set(keys[i], keys[i]); ok || !tr.eq(v, tr.empty.value) {
			t.Fatalf("expected nil")
		}
	}
	if v, ok := tr.Delete(testMapMakeItem(-1)); ok || !tr.eq(v, tr.empty.value) {
		t.Fatal("expected nil")
	}
	tr.sane()
	shuffleMapItems(keys)
	if v, ok := tr.Delete(keys[len(keys)/2]); !ok || !tr.eq(v, keys[len(keys)/2]) {
		t.Fatalf("expected '%v', got '%v'", keys[len(keys)/2], v)
	}
	tr.sane()
	if v, ok := tr.Delete(keys[len(keys)/2]); ok || !tr.eq(v, tr.empty.value) {
		t.Fatalf("expected '%v', got '%v'", tr.empty.value, v)
	}
	tr.sane()
	tr.Set(keys[len(keys)/2], keys[len(keys)/2])
	tr.sane()
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Delete(keys[i]); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
		if v, ok := tr.Get(keys[i]); ok || !tr.eq(v, tr.empty.value) {
			t.Fatalf("expected nil")
		}
		if v, ok := tr.Get(keys[i]); ok || !tr.eq(v, tr.empty.value) {
			t.Fatalf("expected nil")
		}
		if i%97 == 0 {
			tr.sane()
		}
	}
	if tr.Height() != 0 {
		t.Fatalf("expected 0, got %d", tr.Height())
	}
	shuffleMapItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Load(keys[i], keys[i]); ok || !tr.eq(v, tr.empty.value) {
			t.Fatalf("expected nil")
		}
		if i%97 == 0 {
			tr.sane()
		}
	}
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Get(keys[i]); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
	}
	shuffleMapItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Delete(keys[i]); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
		if v, ok := tr.Get(keys[i]); ok || !tr.eq(v, tr.empty.value) {
			t.Fatalf("expected nil")
		}
	}
	sortMapItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Load(keys[i], keys[i]); ok || !tr.eq(v, tr.empty.value) {
			t.Fatalf("expected nil")
		}
		if i%97 == 0 {
			tr.sane()
		}
	}
	shuffleMapItems(keys)
	if v, ok := tr.Load(keys[0], keys[0]); !ok || !tr.eq(v, keys[0]) {
		t.Fatalf("expected '%v', got '%v'", keys[0], v)
	}
	tr.sane()
}

func TestMapLess(t *testing.T) {
	tr := testMapNewBTree()
	if !tr.lt(testMapMakeItem(1), testMapMakeItem(2)) {
		panic("invalid")
	}
	if tr.lt(testMapMakeItem(2), testMapMakeItem(1)) {
		panic("invalid")
	}
	if tr.lt(testMapMakeItem(1), testMapMakeItem(1)) {
		panic("invalid")
	}
}

func TestMapDeleteRandom(t *testing.T) {
	N := 2_000_000
	tr := testMapNewBTree()
	for i := 0; i < N; i++ {
		tr.Load(testMapMakeItem(i), testMapMakeItem(i))
	}
	tr.sane()
	for tr.Len() > 0 {
		var item testMapKind
		var ok bool
		switch rand.Int() % 3 {
		case 0:
			_, item, ok = tr.GetAt(tr.Len() / 2)
		case 1:
			_, item, ok = tr.Min()
		case 2:
			_, item, ok = tr.Max()
		}
		if !ok {
			panic("!")
		}
		v, ok := tr.Delete(item)
		if !ok || !tr.eq(v, item) {
			panic("!")
		}
	}
}

func TestMapDeleteAt(t *testing.T) {
	N := 10_000
	tr := testMapNewBTree()
	keys := randMapKeys(N)
	for _, key := range keys {
		tr.Set(key, key)
	}
	tr.sane()
	for tr.Len() > 0 {
		index := rand.Intn(tr.Len())
		_, item1, ok1 := tr.GetAt(index)
		_, item2, ok2 := tr.DeleteAt(index)
		if !ok1 || !ok2 || !tr.eq(item1, item2) {
			panic("mismatch")
		}
		tr.sane()
	}
}

func TestMapVarious(t *testing.T) {
	N := 1_000_000
	tr := testMapNewBTree()
	for _, i := range randMapKeys(N) {
		if v, ok := tr.Set(i, i); ok || !tr.eq(v, tr.empty.value) {
			panic("!")
		}
	}
	for _, i := range randMapKeys(N) {
		if v, ok := tr.Get(i); !ok || !tr.eq(v, i) {
			panic("!")
		}
	}
	for _, i := range randMapKeys(N) {
		if v, ok := tr.Delete(i); !ok || !tr.eq(v, i) {
			panic("!")
		}
	}
	if _, v, ok := tr.DeleteAt(0); ok || !tr.eq(v, tr.empty.value) {
		panic("!")
	}
	if _, v, ok := tr.GetAt(0); ok || !tr.eq(v, tr.empty.value) {
		panic("!")
	}
	for i := 0; i < N; i++ {
		item := testMapMakeItem(i)
		if v, ok := tr.Set(item, item); ok || !tr.eq(v, tr.empty.value) {
			panic("!")
		}
		item = testMapMakeItem(i)
		if v, ok := tr.Set(item, item); !ok || !tr.eq(v, item) {
			panic("!")
		}
		item = testMapMakeItem(i)
		if v, ok := tr.Set(item, item); !ok || !tr.eq(v, item) {
			panic("!")
		}
	}
	for i := 0; i < N; i++ {
		item := testMapMakeItem(i)
		if v, ok := tr.Get(item); !ok || !tr.eq(v, item) {
			panic("!")
		}
	}
	for i := 0; i < 100; i++ {
		var count int
		tr.Scan(func(_, _ testMapKind) bool {
			if count == i {
				return false
			}
			count++
			return true
		})
	}

	for i := 0; i < N; i++ {
		item := testMapMakeItem(i)
		if v, ok := tr.Delete(item); !ok || !tr.eq(v, item) {
			panic("!")
		}
	}
}

func (tr *Map[K, V]) sane() {
	if err := tr.Sane(); err != nil {
		panic(err)
	}
}

type saneMapError string

func (err saneMapError) Error() string {
	return string(err)
}

// btree_sane returns true if the entire btree and every node are valid.
// - height of all leaves are the equal to the btree height.
// - deep count matches the btree count.
// - all nodes have the correct number of items and counts.
// - all items are in order.
func (tr *Map[K, V]) Sane() error {
	if tr == nil {
		return nil
	}
	if !tr.saneheight() {
		return saneMapError("!sane-height")
	}
	if tr.Len() != tr.count || tr.deepcount() != tr.count {
		return saneMapError("!sane-count")
	}
	if !tr.saneprops() {
		return saneMapError("!sane-props")
	}
	if !tr.saneorder() {
		return saneMapError("!sane-order")
	}
	if !tr.sanenils() {
		return saneMapError("!sane-nils")
	}
	return nil
}

// btree_saneheight returns true if the height of all leaves match the height
// of the btree.
func (tr *Map[K, V]) saneheight() bool {
	height := tr.Height()
	if tr.root != nil {
		if height == 0 {
			return false
		}
		return tr.root.saneheight(1, height)
	}
	return height == 0
}

func (n *mapNode[K, V]) saneheight(height, maxheight int) bool {
	if n.leaf() {
		if height != maxheight {
			return false
		}
	} else {
		i := 0
		for ; i < len(n.items); i++ {
			if !(*n.children)[i].saneheight(height+1, maxheight) {
				return false
			}
		}
		if !(*n.children)[i].saneheight(height+1, maxheight) {
			return false
		}
	}
	return true
}

// btree_deepcount returns the number of items in the btree.
func (tr *Map[K, V]) deepcount() int {
	if tr.root != nil {
		return tr.root.deepcount()
	}
	return 0
}

func (n *mapNode[K, V]) deepcount() int {
	count := len(n.items)
	if !n.leaf() {
		for i := 0; i <= len(n.items); i++ {
			count += (*n.children)[i].deepcount()
		}
	}
	if n.count != count {
		return -1
	}
	return count
}

func (tr *Map[K, V]) nodesaneprops(n *mapNode[K, V], height int) bool {
	if height == 1 {
		if len(n.items) < 1 || len(n.items) > tr.max {
			println(len(n.items) < 1)
			return false
		}
	} else {
		if len(n.items) < tr.min || len(n.items) > tr.max {
			println(2)
			return false
		}
	}
	if !n.leaf() {
		if len(*n.children) != len(n.items)+1 {
			println(3)
			return false
		}
		for i := 0; i < len(n.items); i++ {
			if !tr.nodesaneprops((*n.children)[i], height+1) {
				println(4)
				return false
			}
		}
		if !tr.nodesaneprops((*n.children)[len(n.items)], height+1) {
			println(5)
			return false
		}
	}
	return true
}

func (tr *Map[K, V]) saneprops() bool {
	if tr.root != nil {
		return tr.nodesaneprops(tr.root, 1)
	}
	return true
}

func (tr *Map[K, V]) sanenilsnode(n *mapNode[K, V]) bool {
	items := n.items[:cap(n.items):cap(n.items)]
	for i := len(n.items); i < len(items); i++ {
		if !tr.eq(items[i].key, tr.empty.key) {
			return false
		}
	}
	if !n.leaf() {
		for i := 0; i < len(*n.children); i++ {
			if (*n.children)[i] == nil {
				return false
			}
		}
		children := (*n.children)[:cap(*n.children):cap(*n.children)]
		for i := len(*n.children); i < len(children); i++ {
			if children[i] != nil {
				return false
			}
		}
		for i := 0; i < len(*n.children); i++ {
			if !tr.sanenilsnode((*n.children)[i]) {
				return false
			}
		}
	}
	return true
}

// sanenils checks that all the slots in the item slice that are not used,
//
//	n.items[len(n.items):cap(n.items):cap(n.items)]
//
// are equal to the empty value of the kind.
func (tr *Map[K, V]) sanenils() bool {
	if tr.root != nil {
		return tr.sanenilsnode(tr.root)
	}
	return true
}

func (tr *Map[K, V]) saneorder() bool {
	var last K
	var count int
	var bad bool
	tr.Scan(func(key K, value V) bool {
		if count > 0 {
			if !tr.lt(last, key) {
				bad = true
				return false
			}
		}
		last = key
		count++
		return true
	})
	return !bad && count == tr.count
}

func TestMapIter(t *testing.T) {
	N := 100_000
	tr := testMapNewBTree()
	var all []testMapKind
	for i := 0; i < N; i++ {
		tr.Load(testMapMakeItem(i), testMapMakeItem(i))
		all = append(all, testMapMakeItem(i))
	}
	var count int
	var i int
	iter := tr.Iter()
	for ok := iter.First(); ok; ok = iter.Next() {
		if !tr.eq(all[i], iter.Key()) {
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
		if !tr.eq(all[i], iter.Key()) {
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
		if !tr.eq(all[i], iter.Key()) {
			panic("!")
		}
		i++
	}
	i--
	for ok := iter.Prev(); ok; ok = iter.Prev() {
		i--
		if !tr.eq(all[i], iter.Key()) {
			panic("!")
		}

	}
	if i != 0 {
		panic("!")
	}

	i++
	for ok := iter.Next(); ok; ok = iter.Next() {
		if !tr.eq(all[i], iter.Key()) {
			panic("!")
		}
		i++

	}
	if i != N {
		panic("!")
	}

	i = 0
	for ok := iter.First(); ok; ok = iter.Next() {
		if !tr.eq(all[i], iter.Key()) {
			panic("!")
		}
		if tr.eq(iter.Key(), testMapMakeItem(N/2)) {
			for ok = iter.Prev(); ok; ok = iter.Prev() {
				i--
				if !tr.eq(all[i], iter.Key()) {
					panic("!")
				}
			}
			break
		}
		i++
	}

}

func TestMapIterSeek(t *testing.T) {
	var tr Map[int, struct{}]

	var all []int
	for i := 0; i < 10000; i++ {
		tr.Set(i*2, struct{}{})
		all = append(all, i)
	}
	_ = all
	{
		iter := tr.Iter()
		var vals []int
		for ok := iter.Seek(501); ok; ok = iter.Next() {
			vals = append(vals, iter.Key())
		}
		assert(vals[0] == 502 && vals[1] == 504)
	}
	{
		iter := tr.Iter()
		var vals []int
		for ok := iter.Seek(501); ok; ok = iter.Prev() {
			vals = append(vals, iter.Key())
		}
		assert(vals[0] == 502 && vals[1] == 500)
	}
}

func TestMapIterSeekPrefix(t *testing.T) {
	var tr Map[int, struct{}]
	count := 10_000
	for i := 0; i < count; i++ {
		tr.Set(i*2, struct{}{})
	}
	for i := 0; i < count; i++ {
		iter := tr.Iter()
		ret := iter.Seek(i*2 - 1)
		assert(ret == true)
	}
}

func copyMapEntries(m *Map[int, int]) []mapPair[int, int] {
	all := make([]mapPair[int, int], m.Len())
	keys := m.Keys()
	vals := m.Values()
	for i := 0; i < len(keys); i++ {
		all[i].key = keys[i]
		all[i].value = vals[i]
	}
	sort.Slice(all, func(i, j int) bool {
		return all[i].key < all[j].key
	})
	return all
}

func mapEntriesEqual(a, b []mapPair[int, int]) bool {
	return reflect.DeepEqual(a, b)
}

func copyMapTest(N int, m1 *Map[int, int], e11 []mapPair[int, int], deep bool) {
	e12 := copyMapEntries(m1)
	if !mapEntriesEqual(e11, e12) {
		panic("!")
	}

	// Make a copy and compare the values
	m2 := m1.Copy()
	e21 := copyMapEntries(m1)
	if !mapEntriesEqual(e21, e12) {
		panic("!")
	}

	// Delete every other key
	var e22 []mapPair[int, int]
	for i, j := range rand.Perm(N) {
		if i&1 == 0 {
			e22 = append(e22, e21[j])
		} else {
			prev, deleted := m2.Delete(e21[j].key)
			if !deleted {
				panic("!")
			}
			if prev != e21[j].value {

				panic("!")
			}
		}
	}
	if m2.Len() != N/2 {
		panic("!")
	}
	sort.Slice(e22, func(i, j int) bool {
		return e22[i].key < e22[j].key
	})
	e23 := copyMapEntries(m2)
	if !mapEntriesEqual(e23, e22) {
		panic("!")
	}
	if !deep {
		var wg sync.WaitGroup
		wg.Add(2)
		go func() {
			defer wg.Done()
			copyMapTest(N/2, m2, e23, true)
		}()
		go func() {
			defer wg.Done()
			copyMapTest(N/2, m2, e23, true)
		}()
		wg.Wait()
	}
	e24 := copyMapEntries(m2)
	if !mapEntriesEqual(e24, e23) {
		panic("!")
	}
}

func TestMapCopy(t *testing.T) {
	N := 1_000
	// create the initial map
	m1 := new(Map[int, int])
	for m1.Len() < N {
		m1.Set(rand.Int(), rand.Int())
	}
	e11 := copyMapEntries(m1)
	dur := time.Second * 2
	var wg sync.WaitGroup
	for i := 0; i < 16; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			start := time.Now()
			for time.Since(start) < dur {
				copyMapTest(N, m1, e11, false)
			}
		}()
	}
	wg.Wait()
	e12 := copyMapEntries(m1)
	if !mapEntriesEqual(e11, e12) {
		panic("!")
	}
}

type testNonCopyItem struct {
	data string
}

func newTestNonCopyItem(data string) *testNonCopyItem {
	return &testNonCopyItem{data: data}
}

type testCopyItem struct {
	data string
}

func newTestCopyItem(data string) *testCopyItem {
	return &testCopyItem{data: data}
}

func (item *testCopyItem) Copy() *testCopyItem {
	return &testCopyItem{data: item.data}
}

type testIsoCopyItem struct {
	data string
}

func newTestIsoCopyItem(data string) *testIsoCopyItem {
	return &testIsoCopyItem{data: data}
}

func (item *testIsoCopyItem) Copy() *testIsoCopyItem {
	return &testIsoCopyItem{data: item.data}
}

func TestMapValueCopy(t *testing.T) {
	t.Run("without-copy", func(t *testing.T) {
		var m Map[string, *testNonCopyItem]
		m.Set("hello", newTestNonCopyItem("world"))

		m2 := m.Copy()

		v, _ := m.Get("hello")
		assert(v.data == "world")
		v, _ = m2.Get("hello")
		assert(v.data == "world")

		// now get and change the value, mutable, this will affect both trees
		v, _ = m.GetMut("hello")
		v.data = "planet"

		v, _ = m.Get("hello")
		assert(v.data == "planet")
		v, _ = m2.Get("hello")
		assert(v.data == "planet")
	})
	t.Run("with-copy", func(t *testing.T) {
		var m Map[string, *testCopyItem]
		m.Set("hello", newTestCopyItem("world"))

		m2 := m.Copy()

		v, _ := m.Get("hello")
		assert(v.data == "world")
		v, _ = m2.Get("hello")
		assert(v.data == "world")

		// now get and change the value, mutable, this will only affect the
		// first tree.
		v, _ = m.GetMut("hello")
		v.data = "planet"

		v, _ = m.Get("hello")
		assert(v.data == "planet")
		v, _ = m2.Get("hello")
		assert(v.data == "world")
	})
	t.Run("with-isocopy", func(t *testing.T) {
		var m Map[string, *testIsoCopyItem]
		m.Set("hello", newTestIsoCopyItem("world"))

		m2 := m.Copy()

		v, _ := m.Get("hello")
		assert(v.data == "world")
		v, _ = m2.Get("hello")
		assert(v.data == "world")

		// now get and change the value, mutable, this will only affect the
		// first tree.
		v, _ = m.GetMut("hello")
		v.data = "planet"

		v, _ = m.Get("hello")
		assert(v.data == "planet")
		v, _ = m2.Get("hello")
		assert(v.data == "world")
	})

}

func TestMapDeepCopy(t *testing.T) {

	Ncols := 1000
	Nvals := 1000

	// Create a collection of maps that are each a collection of key/value
	// pairs of string.
	cols1 := NewMap[string, *Map[string, string]](4)
	for i := 0; i < Ncols; i++ {
		col := NewMap[string, string](4)
		for j := 0; j < Nvals; j++ {
			col.Set(fmt.Sprintf("key:%d", j), fmt.Sprintf("val:%d", j))
		}
		cols1.Set(fmt.Sprintf("col:%d", i), col)
	}

	// Copy the root tree

	cols2 := cols1.Copy()

	// Update the second cols2 by deleting half the entries
	for i := 0; i < Ncols; i++ {
		col, _ := cols2.GetMut(fmt.Sprintf("col:%d", i))
		for j := 0; j < Nvals; j += 2 {
			col.Delete(fmt.Sprintf("key:%d", j))
		}
	}

	// Now Count the total of keys in all collections
	var count1 int
	for i := 0; i < Ncols; i++ {
		col, _ := cols1.Get(fmt.Sprintf("col:%d", i))
		count1 += col.Len()
	}

	var count2 int
	for i := 0; i < Ncols; i++ {
		col, _ := cols2.Get(fmt.Sprintf("col:%d", i))
		count2 += col.Len()
	}

	assert(count1 == Ncols*Nvals)
	assert(count2 == Ncols*Nvals/2)

	// Copy again, but this time use Get instead of GetMut
	cols2 = cols1.Copy()

	// Update the second cols2 by deleting half the entries
	for i := 0; i < Ncols; i++ {
		col, _ := cols2.Get(fmt.Sprintf("col:%d", i))
		for j := 0; j < Nvals; j += 2 {
			col.Delete(fmt.Sprintf("key:%d", j))
		}
	}

	// Now Count the total of keys in all collections
	count1 = 0
	for i := 0; i < Ncols; i++ {
		col, _ := cols1.Get(fmt.Sprintf("col:%d", i))
		count1 += col.Len()
	}

	count2 = 0
	for i := 0; i < Ncols; i++ {
		col, _ := cols2.Get(fmt.Sprintf("col:%d", i))
		count2 += col.Len()
	}
	assert(count1 == Ncols*Nvals/2)
	assert(count2 == Ncols*Nvals/2)
}
