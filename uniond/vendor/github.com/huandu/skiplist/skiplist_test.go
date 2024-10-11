// Copyright 2011 Huan Du. All rights reserved.
// Licensed under the MIT license that can be found in the LICENSE file.

package skiplist

import (
	"fmt"
	"math"
	"math/rand"
	"testing"
	"time"

	"github.com/huandu/go-assert"
)

func TestBasicCRUD(t *testing.T) {
	a := assert.New(t)
	list := New(Float64)
	a.Assert(list.Len() == 0)
	a.Equal(list.Find(0), nil)

	elem1 := list.Set(12.34, "first")
	a.Assert(elem1 != nil)
	a.Equal(list.Len(), 1)
	a.Equal(list.Front(), elem1)
	a.Equal(list.Back(), elem1)
	a.Equal(elem1.Next(), nil)
	a.Equal(elem1.Prev(), nil)
	a.Equal(list.Find(0), elem1)
	a.Equal(list.Find(12.34), elem1)
	a.Equal(list.Find(15), nil)

	assertSanity(a, list)

	elem2 := list.Set(23.45, "second")
	a.Assert(elem2 != nil)
	a.NotEqual(elem1, elem2)
	a.Equal(list.Len(), 2)
	a.Equal(list.Front(), elem1)
	a.Equal(list.Back(), elem2)
	a.Equal(elem2.Next(), nil)
	a.Equal(elem2.Prev(), elem1)
	a.Equal(list.Find(-10), elem1)
	a.Equal(list.Find(15), elem2)
	a.Equal(list.Find(25), nil)

	assertSanity(a, list)

	elem3 := list.Set(16.78, "middle")
	a.Assert(elem3 != nil)
	a.NotEqual(elem3, elem1)
	a.NotEqual(elem3, elem2)
	a.Equal(list.Len(), 3)
	a.Equal(list.Front(), elem1)
	a.Equal(list.Back(), elem2)
	a.Equal(elem3.Next(), elem2)
	a.Equal(elem3.Prev(), elem1)
	a.Equal(list.Find(-20), elem1)
	a.Equal(list.Find(15), elem3)
	a.Equal(list.Find(20), elem2)

	assertSanity(a, list)

	elem4 := list.Set(9.01, "very beginning")
	a.Assert(elem4 != nil)
	a.NotEqual(elem4, elem1)
	a.NotEqual(elem4, elem2)
	a.NotEqual(elem4, elem3)
	a.Equal(list.Len(), 4)
	a.Equal(list.Front(), elem4)
	a.Equal(list.Back(), elem2)
	a.Equal(elem4.Next(), elem1)
	a.Equal(elem4.Prev(), nil)
	a.Equal(list.Find(0), elem4)
	a.Equal(list.Find(15), elem3)
	a.Equal(list.Find(20), elem2)

	assertSanity(a, list)

	elem5 := list.Set(16.78, "middle overwrite")
	a.Assert(elem3 != nil)
	a.NotEqual(elem3, elem1)
	a.NotEqual(elem3, elem2)
	a.Equal(elem5, elem3)
	a.NotEqual(elem5, elem4)
	a.Equal(list.Len(), 4)
	a.Equal(list.Front(), elem4)
	a.Equal(list.Back(), elem2)
	a.Equal(elem5.Next(), elem2)
	a.Equal(elem5.Prev(), elem1)
	a.Equal(list.Find(15), elem5)
	a.Equal(list.Find(16.78), elem5)
	a.Equal(list.Find(16.79), elem2)
	a.Equal(list.FindNext(nil, 15), elem5)
	a.Equal(list.FindNext(nil, 16.78), elem5)
	a.Equal(list.FindNext(nil, 16.79), elem2)
	a.Equal(list.FindNext(elem1, 15), elem5)
	a.Equal(list.FindNext(elem5, 15), elem5)
	a.Equal(list.FindNext(elem5, 30), nil)

	min1_2 := func(a, b int) int {
		if a < b {
			return a / 2
		}
		return b / 2
	}
	a.Equal(elem5.NextLevel(0), elem5.Next())
	a.Equal(elem5.NextLevel(-1), nil)
	a.Equal(elem5.NextLevel(min1_2(elem2.Level(), elem5.Level())), elem2)
	a.Equal(elem5.NextLevel(elem2.Level()), nil)
	a.Equal(elem5.PrevLevel(0), elem5.Prev())
	a.Equal(elem5.PrevLevel(min1_2(elem1.Level(), elem5.Level())), elem1)
	a.Equal(elem5.PrevLevel(-1), nil)

	a.Assert(list.Remove(9999) == nil)
	a.Equal(list.Len(), 4)
	a.Assert(list.Remove(13.24) == nil)
	a.Equal(list.Len(), 4)

	assertSanity(a, list)

	list.SetMaxLevel(1)
	assertSanity(a, list)
	list.SetMaxLevel(128)
	assertSanity(a, list)
	list.SetMaxLevel(32)
	assertSanity(a, list)
	list.SetMaxLevel(32)
	assertSanity(a, list)

	elem2Removed := list.Remove(elem2.Key())
	a.Assert(elem2Removed != nil)
	a.Equal(elem2Removed, elem2)
	a.Assert(elem2Removed.Prev() == nil)
	a.Assert(elem2Removed.Next() == nil)
	a.Equal(list.Len(), 3)
	a.Equal(list.Front(), elem4)
	a.Equal(list.Back(), elem5)
	a.Equal(list.Find(-99), elem4)
	a.Equal(list.Find(10), elem1)
	a.Equal(list.Find(15), elem3)
	a.Equal(list.Find(20), nil)

	assertSanity(a, list)

	front := list.RemoveFront()
	a.Assert(front == elem4)
	a.Equal(list.Len(), 2)
	a.Equal(list.Front(), elem1)
	a.Equal(list.Back(), elem5)
	a.Equal(list.Find(-99), elem1)

	back := list.RemoveBack()
	a.Assert(back == elem5)
	a.Equal(list.Len(), 1)
	a.Equal(list.Front(), elem1)
	a.Equal(list.Back(), elem1)
	a.Equal(list.Find(15), nil)
	a.Equal(list.FindNext(nil, 10), elem1)
	a.Equal(list.FindNext(elem1, 10), elem1)
	a.Equal(list.FindNext(nil, 15), nil)

	assertSanity(a, list)

	list.Init()
	a.Equal(list.Len(), 0)
	a.Equal(list.Get(12.34), nil)
}

type testCustomComparable struct {
	High, Low int
}

func TestCustomComparable(t *testing.T) {
	a := assert.New(t)
	comparable := GreaterThanFunc(func(k1, k2 interface{}) int {
		v1 := k1.(*testCustomComparable)
		v2 := k2.(*testCustomComparable)

		if v1.High > v2.High {
			return 1
		}

		if v1.High < v2.High {
			return -1
		}

		if v1.Low > v2.Low {
			return 1
		}

		if v1.Low < v2.Low {
			return -1
		}

		return 0
	})
	k1 := &testCustomComparable{10, 10}
	k2 := &testCustomComparable{10, 7}
	k3 := &testCustomComparable{11, 3}

	list := New(comparable)
	list.Set(k1, "k1")
	list.Set(k2, "k2")
	list.Set(k3, "k3")

	a.Equal(list.Front(), list.Get(k2))
	a.Equal(list.Back(), list.Get(k3))
	a.Equal(list.Find(k1), list.Get(k1))
	a.Equal(list.Find(k2), list.Get(k2))
	a.Equal(list.Find(k3), list.Get(k3))
	a.Equal(list.Find(&testCustomComparable{High: 0, Low: 0}), list.Get(k2))
	a.Equal(list.Find(&testCustomComparable{High: 99, Low: 99}), nil)
	a.Equal(list.FindNext(nil, k1), list.Get(k1))
	a.Equal(list.FindNext(list.Get(k2), k1), list.Get(k1))
	a.Equal(list.FindNext(list.Get(k3), k1), list.Get(k3))
	a.Equal(list.FindNext(list.Get(k3), &testCustomComparable{High: 99, Low: 99}), nil)

	// Reset list to a new one.
	list = New(Reverse(comparable))
	list.Set(k1, "k1")
	list.Set(k2, "k2")
	list.Set(k3, "k3")

	a.Equal(list.Front(), list.Get(k3))
	a.Equal(list.Back(), list.Get(k2))

	// Reset list again to a new one.
	list = New(LessThanFunc(comparable))
	list.Set(k1, "k1")
	list.Set(k2, "k2")
	list.Set(k3, "k3")

	a.Equal(list.Front(), list.Get(k3))
	a.Equal(list.Back(), list.Get(k2))
}

func TestRandomList(t *testing.T) {
	a := assert.New(t)

	const seed = 0xa30378d2
	const N = 1000000
	source := rand.NewSource(seed)
	rnd := rand.New(source)
	list := New(Int64Desc)

	for i := 0; i < N; i++ {
		key := rnd.Intn(N)
		list.Set(key, i)
	}

	for i := 0; i < N; i++ {
		switch i % 4 {
		case 0:
			key := rnd.Intn(N)
			list.Remove(key)

		case 1:
			key := rnd.Intn(N)
			list.Set(key, i)

		case 2:
			list.RemoveBack()

		case 3:
			list.RemoveFront()
		}
	}

	assertSanity(a, list)
}

func BenchmarkDefaultWorstInserts(b *testing.B) {
	list := New(Int)

	for i := 0; i < b.N; i++ {
		list.Set(i, i)
	}
}

func BenchmarkDefaultBestInserts(b *testing.B) {
	list := New(IntDesc)

	for i := 0; i < b.N; i++ {
		var v interface{} = i
		list.Set(v, v)
	}
}

func BenchmarkRandomSelect(b *testing.B) {
	list := New(IntDesc)
	keys := make([]interface{}, 0, b.N)

	for i := 0; i < b.N; i++ {
		keys = append(keys, i)
	}

	rnd := rand.New(rand.NewSource(time.Now().UnixNano()))
	rnd.Shuffle(b.N, func(i, j int) {
		keys[i], keys[j] = keys[j], keys[i]
	})

	for i := 0; i < b.N; i++ {
		list.Set(keys[i], i)
	}

	rnd.Shuffle(b.N, func(i, j int) {
		keys[i], keys[j] = keys[j], keys[i]
	})

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		key := keys[i]
		list.Get(key)
	}
}

func ExampleSkipList() {
	// Create a skip list with int key.
	list := New(Int)

	// Add some values. Value can be anything.
	list.Set(12, "hello world")
	list.Set(34, 56)
	list.Set(78, 90.12)

	// Get element by index.
	elem := list.Get(34)                // Value is stored in elem.Value.
	fmt.Println(elem.Value)             // Output: 56
	next := elem.Next()                 // Get next element.
	prev := next.Prev()                 // Get previous element.
	fmt.Println(next.Value, prev.Value) // Output: 90.12    56

	// Or, directly get value just like a map
	val, ok := list.GetValue(34)
	fmt.Println(val, ok) // Output: 56  true

	// Find first elements with score greater or equal to key
	foundElem := list.Find(30)
	fmt.Println(foundElem.Key(), foundElem.Value) // Output: 34 56

	// Remove an element for key.
	list.Remove(34)
}

func ExampleGreaterThanFunc() {
	type T struct {
		Rad float64
	}
	list := New(GreaterThanFunc(func(k1, k2 interface{}) int {
		s1 := math.Sin(k1.(T).Rad)
		s2 := math.Sin(k2.(T).Rad)

		if s1 > s2 {
			return 1
		} else if s1 < s2 {
			return -1
		}

		return 0
	}))
	list.Set(T{math.Pi / 8}, "sin(π/8)")
	list.Set(T{math.Pi / 2}, "sin(π/2)")
	list.Set(T{math.Pi}, "sin(π)")

	fmt.Println(list.Front().Value) // Output: sin(π)
	fmt.Println(list.Back().Value)  // Output: sin(π/2)

	// Output:
	// sin(π)
	// sin(π/2)
}

func assertSanity(a *assert.A, list *SkipList) {
	l := list.Len()
	maxLevel := len(list.levels) // Actual max level can be larger than list.MaxLevel().
	cnt := 0
	a.Use(&l, &cnt, &maxLevel)
	a.Assert(l >= 0)
	a.Assert(maxLevel >= list.MaxLevel())

	if l == 0 {
		return
	}

	// Collect all elements.
	allElems := make([]*Element, 0, l)

	for elem := list.Front(); elem != nil; elem = elem.Next() {
		allElems = append(allElems, elem)
		cnt++

		a.Assert(elem.list == list)
	}

	a.Assert(cnt == l)
	a.Equal(allElems[0], list.Front())
	a.Equal(allElems[l-1], list.Back())

	// Score must be sorted.
	prevScore := allElems[0].Score()
	comp := list.comparable
	a.Use(&prevScore)

	for i := 1; i < l; i++ {
		score := allElems[i].Score()
		k1 := allElems[i-1].Key()
		k2 := allElems[i].Key()

		// a.Use(&i, &score, &k1, k2)
		a.Assert(prevScore <= score)
		a.Assert(comp.Compare(k1, k2) < 0)

		prevScore = score
	}

	// All levels are well-orgnized.
	for i := 0; i < maxLevel; i++ {
		var prev *Element
		elem := list.levels[i]

		for elem != nil {
			level := elem.Level()

			// a.Use(&i, &level)
			a.Assert(level > i)
			a.Equal(elem.PrevLevel(i), prev)

			prev = elem
			elem = elem.NextLevel(i)
		}
	}

	// Prev and levels must be correct.
	for _, elem := range allElems {
		if prev := elem.Prev(); prev != nil {
			a.Equal(prev.Next(), elem)
		}
	}
}

func TestUint64(t *testing.T) {
	a := assert.New(t)
	list := New(Uint64)
	a.Assert(list.Len() == 0)

	elem1 := list.Set(uint64(0xF141000000000404), "uint64-404")
	a.Assert(elem1 != nil)
	elem2 := list.Set(uint64(0xF141000000000405), "uint64-405")
	a.Assert(elem2 != nil)
	elem3 := list.Set(uint64(0xF141000000000201), "uint64-201")
	a.Assert(elem3 != nil)
	elem4 := list.Set(uint64(0xF141000000000200), "uint64-200")
	a.Assert(elem4 != nil)

	a.Assert(list.Get(uint64(0xF141000000000404)).Value == "uint64-404")
	a.Assert(list.Get(uint64(0xF141000000000405)).Value == "uint64-405")
	a.Assert(list.Get(uint64(0xF141000000000201)).Value == "uint64-201")
	a.Assert(list.Get(uint64(0xF141000000000200)).Value == "uint64-200")
}

func TestInt64(t *testing.T) {
	a := assert.New(t)
	list := New(Int64)
	a.Assert(list.Len() == 0)

	elem1 := list.Set(int64(0x2141000000000404), "int64-404")
	a.Assert(elem1 != nil)
	elem2 := list.Set(int64(0x2141000000000405), "int64-405")
	a.Assert(elem2 != nil)
	elem3 := list.Set(int64(0x2141000000000201), "int64-201")
	a.Assert(elem3 != nil)
	elem4 := list.Set(int64(0x2141000000000200), "int64-200")
	a.Assert(elem4 != nil)

	a.Assert(list.Get(int64(0x2141000000000404)).Value == "int64-404")
	a.Assert(list.Get(int64(0x2141000000000405)).Value == "int64-405")
	a.Assert(list.Get(int64(0x2141000000000201)).Value == "int64-201")
	a.Assert(list.Get(int64(0x2141000000000200)).Value == "int64-200")
}
