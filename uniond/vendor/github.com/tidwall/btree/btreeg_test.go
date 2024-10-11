package btree

import (
	"fmt"
	"math/rand"
	"os"
	"runtime"
	"sort"
	"strconv"
	"sync/atomic"
	"testing"
	"time"
)

func init() {
	seed, err := strconv.ParseInt(os.Getenv("SEED"), 10, 64)
	if err != nil {
		seed = time.Now().UnixNano()
	}
	fmt.Printf("seed: %d\n", seed)
	rand.Seed(seed)
}

// testKind is the item type.
// It's important to use the equal symbol, which tells Go to create an alias of
// the type, rather than creating an entirely new type.
type testKind = int

func testLess(a, b testKind) bool {
	return a < b
}

// The functions below, which begin with "test*", are required by the
// btree_test.go file. If you choose not use include the btree_test.go file in
// your project then these functions may be omitted.

// testMakeItem must return a valid item for testing.
// It's required that the returned item maintains equal order as the
// provided int, such that:
//
//	testMakeItem(0) < testMakeItem(1) < testMakeItem(2) < testMakeItem(10)
func testMakeItem(x int) (item testKind) {
	return x
}

// testNewBTree must return an operational btree for testing.
func testNewBTree() *BTreeG[testKind] {
	return NewBTreeG(testLess)
}

func randKeys(N int) (keys []testKind) {
	keys = make([]testKind, N)
	for _, i := range rand.Perm(N) {
		keys[i] = testMakeItem(i)
	}
	return keys
}

func (tr *BTreeG[T]) lt(a, b T) bool  { return tr.less(a, b) }
func (tr *BTreeG[T]) eq(a, b T) bool  { return !(tr.lt(a, b) || tr.lt(b, a)) }
func (tr *BTreeG[T]) lte(a, b T) bool { return tr.lt(a, b) || tr.eq(a, b) }
func (tr *BTreeG[T]) gt(a, b T) bool  { return tr.lt(b, a) }
func (tr *BTreeG[T]) gte(a, b T) bool { return tr.gt(a, b) || tr.eq(a, b) }

func kindsAreEqual(a, b []testKind) bool {
	tr := NewBTreeG(testLess)
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

func TestGenericMakeItemOrder(t *testing.T) {
	tr := testNewBTree()
	ints := []int{0, 1, 2, 3, 4, 10, 20, 30, 40, 100, 200, 300, 400}
	for i := 0; i < len(ints)-1; i++ {
		a := testMakeItem(ints[i])
		b := testMakeItem(ints[i+1])
		if !tr.lt(a, b) {
			t.Fatalf("bad ordering for testMakeItem: '%v' !< '%v'", a, b)
		}
	}
}

func TestGenericDescend(t *testing.T) {
	tr := testNewBTree()
	var count int
	tr.Descend(testMakeItem(rand.Int()), func(item testKind) bool {
		count++
		return true
	})
	if count > 0 {
		t.Fatalf("expected 0, got %v", count)
	}
	var keys []testKind
	for i := 0; i < 1000; i += 10 {
		keys = append(keys, testMakeItem(i))
		tr.Set(keys[len(keys)-1])
	}
	var exp []testKind
	tr.Reverse(func(item testKind) bool {
		exp = append(exp, item)
		return true
	})
	for i := 999; i >= 0; i-- {
		key := testMakeItem(i)
		var all []testKind
		tr.Descend(key, func(item testKind) bool {
			all = append(all, item)
			return true
		})
		for len(exp) > 0 && tr.Less(key, exp[0]) {
			exp = exp[1:]
		}
		var count int
		tr.Descend(key, func(item testKind) bool {
			if count == (i+1)%tr.max {
				return false
			}
			count++
			return true
		})
		if count > len(exp) {
			t.Fatalf("expected 1, got %v", count)
		}
		if !kindsAreEqual(exp, all) {
			fmt.Printf("exp: %v\n", exp)
			fmt.Printf("all: %v\n", all)
			t.Fatal("mismatch")
		}
		for j := 0; j < tr.Len(); j++ {
			count = 0
			tr.Descend(key, func(item testKind) bool {
				if count == j {
					return false
				}
				count++
				return true
			})
		}
	}
}

func TestGenericDescendHint(t *testing.T) {
	tr := testNewBTree()
	var count int
	var hint PathHint
	tr.DescendHint(testMakeItem(rand.Int()), func(item testKind) bool {
		count++
		return true
	}, &hint)
	if count > 0 {
		t.Fatalf("expected 0, got %v", count)
	}
	var keys []testKind
	for i := 0; i < 1000; i += 10 {
		keys = append(keys, testMakeItem(i))
		tr.Set(keys[len(keys)-1])
	}
	var exp []testKind
	tr.Reverse(func(item testKind) bool {
		exp = append(exp, item)
		return true
	})
	for i := 999; i >= 0; i-- {
		key := testMakeItem(i)
		var all []testKind
		tr.DescendHint(key, func(item testKind) bool {
			all = append(all, item)
			return true
		}, &hint)
		for len(exp) > 0 && tr.Less(key, exp[0]) {
			exp = exp[1:]
		}
		var count int
		tr.DescendHint(key, func(item testKind) bool {
			if count == (i+1)%tr.max {
				return false
			}
			count++
			return true
		}, &hint)
		if count > len(exp) {
			t.Fatalf("expected 1, got %v", count)
		}
		if !kindsAreEqual(exp, all) {
			fmt.Printf("exp: %v\n", exp)
			fmt.Printf("all: %v\n", all)
			t.Fatal("mismatch")
		}
		for j := 0; j < tr.Len(); j++ {
			count = 0
			tr.DescendHint(key, func(item testKind) bool {
				if count == j {
					return false
				}
				count++
				return true
			}, &hint)
		}
	}
}

func TestGenericAscend(t *testing.T) {
	tr := testNewBTree()
	var count int
	tr.Ascend(testMakeItem(1), func(item testKind) bool {
		count++
		return true
	})
	if count > 0 {
		t.Fatalf("expected 0, got %v", count)
	}
	var keys []testKind
	for i := 0; i < 1000; i += 10 {
		keys = append(keys, testMakeItem(i))
		tr.Set(keys[len(keys)-1])
		tr.sane()
	}
	exp := keys
	for i := -1; i < 1000; i++ {
		key := testMakeItem(i)
		var all []testKind
		tr.Ascend(key, func(item testKind) bool {
			all = append(all, item)
			return true
		})
		for len(exp) > 0 && tr.Less(exp[0], key) {
			exp = exp[1:]
		}
		var count int
		tr.Ascend(key, func(item testKind) bool {
			if count == (i+1)%tr.max {
				return false
			}
			count++
			return true
		})
		if count > len(exp) {
			t.Fatalf("expected 1, got %v", count)
		}
		if !kindsAreEqual(exp, all) {
			t.Fatal("mismatch")
		}
	}
}

func TestGenericAscendHint(t *testing.T) {
	tr := testNewBTree()
	var hint PathHint
	var count int
	tr.AscendHint(testMakeItem(1), func(item testKind) bool {
		count++
		return true
	}, &hint)
	if count > 0 {
		t.Fatalf("expected 0, got %v", count)
	}
	var keys []testKind
	for i := 0; i < 1000; i += 10 {
		keys = append(keys, testMakeItem(i))
		tr.Set(keys[len(keys)-1])
		tr.sane()
	}
	exp := keys
	for i := -1; i < 1000; i++ {
		key := testMakeItem(i)
		var all []testKind
		tr.AscendHint(key, func(item testKind) bool {
			all = append(all, item)
			return true
		}, &hint)
		for len(exp) > 0 && tr.Less(exp[0], key) {
			exp = exp[1:]
		}
		var count int
		tr.AscendHint(key, func(item testKind) bool {
			if count == (i+1)%tr.max {
				return false
			}
			count++
			return true
		}, &hint)
		if count > len(exp) {
			t.Fatalf("expected 1, got %v", count)
		}
		if !kindsAreEqual(exp, all) {
			t.Fatal("mismatch")
		}
	}
}

func TestGenericItems(t *testing.T) {
	tr := testNewBTree()
	if len(tr.Items()) != 0 {
		t.Fatalf("expected 0, got %v", len(tr.Items()))
	}
	var keys []testKind
	for i := 0; i < 100000; i += 10 {
		keys = append(keys, testMakeItem(i))
		tr.Set(keys[len(keys)-1])
		tr.sane()
	}
	keys2 := tr.Items()
	if !kindsAreEqual(keys, keys2) {
		t.Fatal("mismatch")
	}
}

func TestGenericSimpleRandom(t *testing.T) {
	start := time.Now()
	for time.Since(start) < time.Second*2 {
		N := 100_000
		items := randKeys(N)
		tr := testNewBTree()
		tr.sane()
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Get(items[i]); ok || !tr.eq(v, tr.empty) {
				panic("!")
			}
			if v, ok := tr.Set(items[i]); ok || !tr.eq(v, tr.empty) {
				panic("!")
			}
			if v, ok := tr.Get(items[i]); !ok || !tr.eq(v, items[i]) {
				panic("!")
			}
		}
		tr.sane()
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Set(items[i]); !ok || !tr.eq(v, items[i]) {
				panic("!")
			}
		}
		pivot := items[len(items)/2]
		tr.Ascend(pivot, func(item testKind) bool {
			if tr.Less(item, pivot) {
				panic("!")
			}
			return true
		})
		var min testKind
		index := 0
		tr.Scan(func(item testKind) bool {
			if index == len(items)/2 {
				return false
			}
			if index > 0 {
				if tr.Less(item, min) {
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
			if v, ok := tr.Delete(items[i]); ok || !tr.eq(v, tr.empty) {
				panic("!")
			}
		}
		if tr.Len() != 0 {
			panic("!")
		}
		tr.sane()
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Delete(items[i]); ok || !tr.eq(v, tr.empty) {
				panic("!")
			}
		}
		tr.sane()
		tr.Scan(func(item testKind) bool {
			panic("!")
		})
	}
}

func TestGenericBTree(t *testing.T) {
	N := 10000
	tr := testNewBTree()
	tr.sane()
	keys := randKeys(N)

	// insert all items
	for _, key := range keys {
		if v, ok := tr.Set(key); ok || !tr.eq(v, tr.empty) {
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
	var prev testKind
	var count int
	tr.Scan(func(item testKind) bool {
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
	tr.Reverse(func(item testKind) bool {
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
	if v, ok := tr.Get(testMakeItem(-1)); ok || !tr.eq(v, tr.empty) {
		t.Fatal("expected nil")
	}

	// scan and quit at various steps
	for i := 0; i < 100; i++ {
		var j int
		tr.Scan(func(item testKind) bool {
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
		tr.Reverse(func(item testKind) bool {
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
		if v, ok := tr.Delete(key); ok || !tr.eq(v, tr.empty) {
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
	tr.Scan(func(item testKind) bool {
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
		if v, ok := tr.Set(key); !ok || !tr.eq(v, key) {
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
	if v, ok := tr.Get(keys[0]); ok || !tr.eq(v, tr.empty) {
		t.Fatal("expected nil")
	}
	tr.Scan(func(item testKind) bool {
		t.Fatal("should not be reached")
		return true
	})
	tr.Reverse(func(item testKind) bool {
		t.Fatal("should not be reached")
		return true
	})
	if v, ok := tr.Delete(testMakeItem(-1)); ok || !tr.eq(v, tr.empty) {
		t.Fatal("expected nil")
	}
	tr.sane()
}

func TestGenericBTreeOne(t *testing.T) {
	tr := testNewBTree()
	tr.Set(testMakeItem(1))
	tr.Delete(testMakeItem(1))
	tr.Set(testMakeItem(1))
	tr.Delete(testMakeItem(1))
	tr.Set(testMakeItem(1))
	tr.Delete(testMakeItem(1))
	if tr.Len() != 0 {
		panic("!")
	}
	tr.sane()
}

func TestGenericBTree256(t *testing.T) {
	tr := testNewBTree()
	var n int
	for j := 0; j < 2; j++ {
		for _, i := range rand.Perm(256) {
			tr.Set(testMakeItem(i))
			n++
			if tr.Len() != n {
				t.Fatalf("expected 256, got %d", n)
			}
		}
		for _, i := range rand.Perm(256) {
			if v, ok := tr.Get(testMakeItem(i)); !ok || !tr.eq(v, testMakeItem(i)) {
				t.Fatalf("expected %v, got %v", i, v)
			}
		}
		for _, i := range rand.Perm(256) {
			tr.Delete(testMakeItem(i))
			n--
			if tr.Len() != n {
				t.Fatalf("expected 256, got %d", n)
			}
		}
		for _, i := range rand.Perm(256) {
			if v, ok := tr.Get(testMakeItem(i)); ok || !tr.eq(v, tr.empty) {
				t.Fatal("expected nil")
			}
		}
	}
}

func shuffleItems(keys []testKind) {
	for i := range keys {
		j := rand.Intn(i + 1)
		keys[i], keys[j] = keys[j], keys[i]
	}
}

func sortItems(keys []testKind) {
	tr := testNewBTree()
	sort.Slice(keys, func(i, j int) bool {
		return tr.lt(keys[i], keys[j])
	})
}

func TestGenericRandom(t *testing.T) {
	N := 200000
	keys := randKeys(N)
	tr := testNewBTree()
	tr.sane()
	if v, ok := tr.Min(); ok || !tr.eq(v, tr.empty) {
		t.Fatalf("expected nil")
	}
	if v, ok := tr.Max(); ok || !tr.eq(v, tr.empty) {
		t.Fatalf("expected nil")
	}
	if v, ok := tr.PopMin(); ok || !tr.eq(v, tr.empty) {
		t.Fatalf("expected nil")
	}
	if v, ok := tr.PopMax(); ok || !tr.eq(v, tr.empty) {
		t.Fatalf("expected nil")
	}
	if tr.Height() != 0 {
		t.Fatalf("expected 0, got %d", tr.Height())
	}
	tr.sane()
	shuffleItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Set(keys[i]); ok || !tr.eq(v, tr.empty) {
			t.Fatalf("expected nil")
		}
		if i%123 == 0 {
			tr.sane()
		}
	}
	tr.sane()
	sortItems(keys)
	var n int
	tr.Scan(func(item testKind) bool {
		n++
		return false
	})
	if n != 1 {
		t.Fatalf("expected 1, got %d", n)
	}

	n = 0
	tr.Scan(func(item testKind) bool {
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
		if v, ok := tr.GetAt(i); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected %v, got %v", keys[i], v)
		}
	}

	n = 0
	tr.Reverse(func(item testKind) bool {
		n++
		return false
	})
	if n != 1 {
		t.Fatalf("expected 1, got %d", n)
	}
	n = 0
	tr.Reverse(func(item testKind) bool {
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
		tr.Scan(func(item testKind) bool {
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
		tr.Reverse(func(item testKind) bool {
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

	sortItems(keys)
	for i := 0; i < len(keys); i++ {
		var res testKind
		var j int
		tr.Ascend(keys[i], func(item testKind) bool {
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
		var res testKind
		var j int
		tr.Descend(keys[i], func(item testKind) bool {
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
	if v, ok := tr.Min(); !ok || !tr.eq(v, keys[0]) {
		t.Fatalf("expected '%v', got '%v'", keys[0], v)
	}
	if v, ok := tr.Max(); !ok || !tr.eq(v, keys[len(keys)-1]) {
		t.Fatalf("expected '%v', got '%v'", keys[len(keys)-1], v)
	}
	if v, ok := tr.PopMin(); !ok || !tr.eq(v, keys[0]) {
		t.Fatalf("expected '%v', got '%v'", keys[0], v)
	}
	tr.sane()
	if v, ok := tr.PopMax(); !ok || !tr.eq(v, keys[len(keys)-1]) {
		t.Fatalf("expected '%v', got '%v'", keys[len(keys)-1], v)
	}
	tr.sane()
	tr.Set(keys[0])
	tr.Set(keys[len(keys)-1])
	shuffleItems(keys)
	var hint PathHint
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Get(keys[i]); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
		if v, ok := tr.GetHint(keys[i], &hint); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
	}
	sortItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.PopMin(); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
	}
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Set(keys[i]); ok || !tr.eq(v, tr.empty) {
			t.Fatalf("expected nil")
		}
	}
	for i := len(keys) - 1; i >= 0; i-- {
		if v, ok := tr.PopMax(); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
	}
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Set(keys[i]); ok || !tr.eq(v, tr.empty) {
			t.Fatalf("expected nil")
		}
	}
	if v, ok := tr.Delete(testMakeItem(-1)); ok || !tr.eq(v, tr.empty) {
		t.Fatal("expected nil")
	}
	tr.sane()
	shuffleItems(keys)
	if v, ok := tr.Delete(keys[len(keys)/2]); !ok || !tr.eq(v, keys[len(keys)/2]) {
		t.Fatalf("expected '%v', got '%v'", keys[len(keys)/2], v)
	}
	tr.sane()
	if v, ok := tr.Delete(keys[len(keys)/2]); ok || !tr.eq(v, tr.empty) {
		t.Fatalf("expected '%v', got '%v'", tr.empty, v)
	}
	tr.sane()
	tr.Set(keys[len(keys)/2])
	tr.sane()
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Delete(keys[i]); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
		if v, ok := tr.Get(keys[i]); ok || !tr.eq(v, tr.empty) {
			t.Fatalf("expected nil")
		}
		if v, ok := tr.GetHint(keys[i], &hint); ok || !tr.eq(v, tr.empty) {
			t.Fatalf("expected nil")
		}
		if i%97 == 0 {
			tr.sane()
		}
	}
	if tr.Height() != 0 {
		t.Fatalf("expected 0, got %d", tr.Height())
	}
	shuffleItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Load(keys[i]); ok || !tr.eq(v, tr.empty) {
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
	shuffleItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Delete(keys[i]); !ok || !tr.eq(v, keys[i]) {
			t.Fatalf("expected '%v', got '%v'", keys[i], v)
		}
		if v, ok := tr.Get(keys[i]); ok || !tr.eq(v, tr.empty) {
			t.Fatalf("expected nil")
		}
	}
	sortItems(keys)
	for i := 0; i < len(keys); i++ {
		if v, ok := tr.Load(keys[i]); ok || !tr.eq(v, tr.empty) {
			t.Fatalf("expected nil")
		}
		if i%97 == 0 {
			tr.sane()
		}
	}
	shuffleItems(keys)
	if v, ok := tr.Load(keys[0]); !ok || !tr.eq(v, keys[0]) {
		t.Fatalf("expected '%v', got '%v'", keys[0], v)
	}
	tr.sane()
}

func TestGenericLess(t *testing.T) {
	tr := testNewBTree()
	if !tr.Less(testMakeItem(1), testMakeItem(2)) {
		panic("invalid")
	}
	if tr.Less(testMakeItem(2), testMakeItem(1)) {
		panic("invalid")
	}
	if tr.Less(testMakeItem(1), testMakeItem(1)) {
		panic("invalid")
	}
}

func TestGenericDeleteRandom(t *testing.T) {
	N := 2_000_000
	tr := testNewBTree()
	for i := 0; i < N; i++ {
		tr.Load(testMakeItem(i))
	}
	tr.sane()
	for tr.Len() > 0 {
		var item testKind
		var ok bool
		switch rand.Int() % 3 {
		case 0:
			item, ok = tr.GetAt(tr.Len() / 2)
		case 1:
			item, ok = tr.Min()
		case 2:
			item, ok = tr.Max()
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

func TestGenericDeleteAt(t *testing.T) {
	N := 10_000
	tr := testNewBTree()
	keys := randKeys(N)
	for _, key := range keys {
		tr.Set(key)
	}
	tr.sane()
	for tr.Len() > 0 {
		index := rand.Intn(tr.Len())
		item1, ok1 := tr.GetAt(index)
		item2, ok2 := tr.DeleteAt(index)
		if !ok1 || !ok2 || !tr.eq(item1, item2) {
			panic("mismatch")
		}
		tr.sane()
	}
}

func TestGenericCopy(t *testing.T) {
	items := randKeys(100000)
	itemsM := testNewBTree()
	for i := 0; i < len(items); i++ {
		itemsM.Set(items[i])
	}
	tr := testNewBTree()
	for i := 0; i < len(items); i++ {
		tr.Set(items[i])
	}
	var wait atomic.Int32
	var testCopyDeep func(tr *BTreeG[testKind], parent bool)

	testCopyDeep = func(tr *BTreeG[testKind], parent bool) {
		defer func() { wait.Add(-1) }()
		if parent {
			// 2 grandchildren
			for i := 0; i < 2; i++ {
				wait.Add(1)
				go testCopyDeep(tr.Copy(), false)
			}
		}

		items2 := make([]testKind, 10000)
		for i := 0; i < len(items2); i++ {
			x := testMakeItem(rand.Int())
			_, ok := itemsM.Get(x)
			for ok {
				x = testMakeItem(rand.Int())
				_, ok = itemsM.Get(x)
			}
			items2[i] = x
		}
		for i := 0; i < len(items2); i++ {
			if v, ok := tr.Set(items2[i]); ok || !tr.eq(v, tr.empty) {
				panic("!")
			}
		}
		tr.sane()
		if tr.Len() != len(items)+len(items2) {
			panic("!")
		}
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Get(items[i]); !ok || !tr.eq(v, items[i]) {
				panic("!")
			}
		}
		for i := 0; i < len(items2); i++ {
			if v, ok := tr.Get(items2[i]); !ok || !tr.eq(v, items2[i]) {
				panic("!")
			}
		}

		for i := 0; i < len(items); i++ {
			if v, ok := tr.Delete(items[i]); !ok || !tr.eq(v, items[i]) {
				panic("!")
			}
		}
		tr.sane()
		if tr.Len() != len(items2) {
			panic("!")
		}
		for i := 0; i < len(items2); i++ {
			if v, ok := tr.Get(items2[i]); !ok || !tr.eq(v, items2[i]) {
				panic("!")
			}
		}
		sortItems(items2)
		var i int
		for len(items2) > 0 {
			if i%2 == 0 {
				if v, ok := tr.PopMin(); !ok || !tr.eq(v, items2[0]) {
					panic("!")
				}
				items2 = items2[1:]
			} else {
				if v, ok := tr.PopMax(); !ok || !tr.eq(v, items2[len(items2)-1]) {
					panic("!")
				}
				items2 = items2[:len(items2)-1]
			}
			if i%123 == 0 {
				tr.sane()
				if tr.Len() != len(items2) {
					panic("!")
				}
				for i := 0; i < len(items2); i++ {
					if v, ok := tr.Get(items2[i]); !ok || !tr.eq(v, items2[i]) {
						panic("!")
					}
				}
			}
			i++
		}
		tr.sane()
		if tr.Len() != len(items2) {
			panic("!")
		}
	}

	// 10 children
	for i := 0; i < 10; i++ {
		wait.Add(1)
		go testCopyDeep(tr.Copy(), true)
	}

	for wait.Load() > 0 {
		tr.sane()
		if tr.Len() != len(items) {
			panic("!")
		}
		for i := 0; i < len(items); i++ {
			if v, ok := tr.Get(items[i]); !ok || !tr.eq(v, items[i]) {
				panic("!")
			}
		}
		runtime.Gosched()
	}
}

func TestGenericVarious(t *testing.T) {
	N := 1_000_000
	tr := testNewBTree()
	var hint PathHint
	for _, i := range randKeys(N) {
		if v, ok := tr.SetHint(i, &hint); ok || !tr.eq(v, tr.empty) {
			panic("!")
		}
	}
	for _, i := range randKeys(N) {
		if v, ok := tr.GetHint(i, &hint); !ok || !tr.eq(v, i) {
			panic("!")
		}
	}
	for _, i := range randKeys(N) {
		if v, ok := tr.DeleteHint(i, &hint); !ok || !tr.eq(v, i) {
			panic("!")
		}
	}
	if v, ok := tr.DeleteAt(0); ok || !tr.eq(v, tr.empty) {
		panic("!")
	}
	if v, ok := tr.GetAt(0); ok || !tr.eq(v, tr.empty) {
		panic("!")
	}
	for i := 0; i < N; i++ {
		item := testMakeItem(i)
		if v, ok := tr.SetHint(item, &hint); ok || !tr.eq(v, tr.empty) {
			panic("!")
		}
		item = testMakeItem(i)
		if v, ok := tr.SetHint(item, &hint); !ok || !tr.eq(v, item) {
			panic("!")
		}
		item = testMakeItem(i)
		if v, ok := tr.SetHint(item, &hint); !ok || !tr.eq(v, item) {
			panic("!")
		}
	}
	for i := 0; i < N; i++ {
		item := testMakeItem(i)
		if v, ok := tr.GetHint(item, &hint); !ok || !tr.eq(v, item) {
			panic("!")
		}
	}
	for i := 0; i < 100; i++ {
		var count int
		tr.Walk(func(_ []testKind) bool {
			if count == i {
				return false
			}
			count++
			return true
		})
	}

	for i := 0; i < N; i++ {
		item := testMakeItem(i)
		if v, ok := tr.DeleteHint(item, &hint); !ok || !tr.eq(v, item) {
			panic("!")
		}
	}
}

func (tr *BTreeG[T]) sane() {
	if err := tr.Sane(); err != nil {
		panic(err)
	}
}

type saneError string

func (err saneError) Error() string {
	return string(err)
}

// btree_sane returns true if the entire btree and every node are valid.
// - height of all leaves are the equal to the btree height.
// - deep count matches the btree count.
// - all nodes have the correct number of items and counts.
// - all items are in order.
func (tr *BTreeG[T]) Sane() error {
	if tr == nil {
		return nil
	}
	if !tr.saneheight() {
		return saneError("!sane-height")
	}
	if tr.Len() != tr.count || tr.deepcount() != tr.count {
		return saneError("!sane-count")
	}
	if !tr.saneprops() {
		return saneError("!sane-props")
	}
	if !tr.saneorder() {
		return saneError("!sane-order")
	}
	if !tr.sanenils() {
		return saneError("!sane-nils")
	}
	return nil
}

// btree_saneheight returns true if the height of all leaves match the height
// of the btree.
func (tr *BTreeG[T]) saneheight() bool {
	height := tr.Height()
	if tr.root != nil {
		if height == 0 {
			return false
		}
		return tr.root.saneheight(1, height)
	}
	return height == 0
}

func (n *node[T]) saneheight(height, maxheight int) bool {
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
func (tr *BTreeG[T]) deepcount() int {
	if tr.root != nil {
		return tr.root.deepcount()
	}
	return 0
}

func (n *node[T]) deepcount() int {
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

func (tr *BTreeG[T]) nodesaneprops(n *node[T], height int) bool {
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

func (tr *BTreeG[T]) saneprops() bool {
	if tr.root != nil {
		return tr.nodesaneprops(tr.root, 1)
	}
	return true
}

func (tr *BTreeG[T]) sanenilsnode(n *node[T]) bool {
	items := n.items[:cap(n.items):cap(n.items)]
	for i := len(n.items); i < len(items); i++ {
		if !tr.eq(items[i], tr.empty) {
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
func (tr *BTreeG[T]) sanenils() bool {
	if tr.root != nil {
		return tr.sanenilsnode(tr.root)
	}
	return true
}

func (tr *BTreeG[T]) saneorder() bool {
	var last T
	var count int
	var bad bool
	tr.Walk(func(items []T) bool {
		for _, item := range items {
			if count > 0 {
				if !tr.Less(last, item) {
					bad = true
					return false
				}
			}
			last = item
			count++
		}
		return true
	})
	return !bad && count == tr.count
}

func TestGenericIter(t *testing.T) {
	N := 100_000
	tr := testNewBTree()
	var all []testKind
	for i := 0; i < N; i++ {
		tr.Load(testMakeItem(i))
		all = append(all, testMakeItem(i))
	}
	var count int
	var i int
	iter := tr.Iter()
	for ok := iter.First(); ok; ok = iter.Next() {
		if !tr.eq(all[i], iter.Item()) {
			panic("!")
		}
		count++
		i++
	}
	if count != N {
		t.Fatalf("expected %v, got %v", N, count)
	}
	iter.Release()
	count = 0
	i = len(all) - 1
	iter = tr.Iter()
	for ok := iter.Last(); ok; ok = iter.Prev() {
		if !tr.eq(all[i], iter.Item()) {
			panic("!")
		}
		i--
		count++
	}
	if count != N {
		t.Fatalf("expected %v, got %v", N, count)
	}
	iter.Release()
	i = 0
	iter = tr.Iter()
	for ok := iter.First(); ok; ok = iter.Next() {
		if !tr.eq(all[i], iter.Item()) {
			panic("!")
		}
		i++
	}
	i--
	for ok := iter.Prev(); ok; ok = iter.Prev() {
		i--
		if !tr.eq(all[i], iter.Item()) {
			panic("!")
		}

	}
	if i != 0 {
		panic("!")
	}

	i++
	for ok := iter.Next(); ok; ok = iter.Next() {
		if !tr.eq(all[i], iter.Item()) {
			panic("!")
		}
		i++

	}
	if i != N {
		panic("!")
	}

	i = 0
	for ok := iter.First(); ok; ok = iter.Next() {
		if !tr.eq(all[i], iter.Item()) {
			panic("!")
		}
		if tr.eq(iter.Item(), testMakeItem(N/2)) {
			for ok = iter.Prev(); ok; ok = iter.Prev() {
				i--
				if !tr.eq(all[i], iter.Item()) {
					panic("!")
				}
			}
			break
		}
		i++
	}
	iter.Release()

}

func TestGenericIterSeek(t *testing.T) {
	tr := NewBTreeG(func(a, b int) bool {
		return a < b
	})
	var all []int
	for i := 0; i < 10000; i++ {
		tr.Set(i * 2)
		all = append(all, i)
	}
	_ = all
	{
		iter := tr.Iter()
		var vals []int
		for ok := iter.Seek(501); ok; ok = iter.Next() {
			vals = append(vals, iter.Item())
		}
		iter.Release()
		assert(vals[0] == 502 && vals[1] == 504)
	}
	{
		iter := tr.Iter()
		var vals []int
		for ok := iter.Seek(501); ok; ok = iter.Prev() {
			vals = append(vals, iter.Item())
		}
		iter.Release()
		assert(vals[0] == 502 && vals[1] == 500)
	}
}

func TestGenericIterSeekHint(t *testing.T) {
	tr := NewBTreeG(func(a, b int) bool {
		return a < b
	})
	var all []int
	for i := 0; i < 10000; i++ {
		tr.Set(i * 2)
		all = append(all, i)
	}
	var hint PathHint
	_ = all
	{
		iter := tr.Iter()
		var vals []int
		for ok := iter.SeekHint(501, &hint); ok; ok = iter.Next() {
			vals = append(vals, iter.Item())
		}
		iter.Release()
		assert(vals[0] == 502 && vals[1] == 504)
	}
	{
		iter := tr.Iter()
		var vals []int
		for ok := iter.SeekHint(501, &hint); ok; ok = iter.Prev() {
			vals = append(vals, iter.Item())
		}
		iter.Release()
		assert(vals[0] == 502 && vals[1] == 500)
	}
}

func TestGenericIterSeekPrefix(t *testing.T) {
	tr := NewBTreeG(func(a, b int) bool {
		return a < b
	})
	count := 10_000
	for i := 0; i < count; i++ {
		tr.Set(i * 2)
	}
	for i := 0; i < count; i++ {
		iter := tr.Iter()
		ret := iter.Seek(i*2 - 1)
		assert(ret == true)
		iter.Release()
	}
}
