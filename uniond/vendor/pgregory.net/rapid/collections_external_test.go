// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"reflect"
	"strconv"
	"testing"

	. "pgregory.net/rapid"
)

func TestSliceOf(t *testing.T) {
	t.Parallel()

	gens := []*Generator[any]{
		SliceOf(Bool()).AsAny(),
		SliceOf(Byte()).AsAny(),
		SliceOf(Int()).AsAny(),
		SliceOf(Uint()).AsAny(),
	}

	for _, g := range gens {
		t.Run(g.String(), MakeCheck(func(t *T) {
			v := g.Draw(t, "v")
			if rv(v).Kind() != reflect.Slice {
				t.Fatalf("got not a slice")
			}
			if rv(v).Len() == 0 {
				t.Skip("empty")
			}
		}))
	}
}

func TestSliceOfDistinct(t *testing.T) {
	t.Parallel()

	g := SliceOfDistinct(Int(), ID[int])

	Check(t, func(t *T) {
		s := g.Draw(t, "s")
		m := map[int]struct{}{}
		for _, i := range s {
			m[i] = struct{}{}
		}
		if len(m) != len(s) {
			t.Fatalf("%v unique out of %v", len(m), len(s))
		}
	})
}

func TestSliceOfDistinctBy(t *testing.T) {
	t.Parallel()

	g := SliceOfDistinct(Int(), func(i int) string { return strconv.Itoa(i % 5) })

	Check(t, func(t *T) {
		s := g.Draw(t, "s")
		m := map[int]struct{}{}
		for _, i := range s {
			m[i%5] = struct{}{}
		}
		if len(m) != len(s) {
			t.Fatalf("%v unique out of %v", len(m), len(s))
		}
	})
}

func TestMapOf(t *testing.T) {
	t.Parallel()

	gens := []*Generator[any]{
		MapOf(Bool(), Int()).AsAny(),
		MapOf(Int(), Uint()).AsAny(),
		MapOf(Uint(), SliceOf(Bool())).AsAny(),
	}

	for _, g := range gens {
		t.Run(g.String(), MakeCheck(func(t *T) {
			v := g.Draw(t, "v")
			if rv(v).Kind() != reflect.Map {
				t.Fatalf("got not a map")
			}
			if rv(v).Len() == 0 {
				t.Skip("empty")
			}
		}))
	}
}

func TestMapOfValues(t *testing.T) {
	t.Parallel()

	g := MapOfValues(Custom(genStruct), func(s testStruct) int { return s.x })

	Check(t, func(t *T) {
		m := g.Draw(t, "m")
		for k, v := range m {
			if k != v.x {
				t.Fatalf("got key %v with value %v", k, v)
			}
		}
	})
}

func TestCollectionLenLimits(t *testing.T) {
	t.Parallel()

	genFuncs := []func(i, j int) *Generator[any]{
		func(i, j int) *Generator[any] { return StringOfN(Int32Range('A', 'Z'), i, j, -1).AsAny() },
		func(i, j int) *Generator[any] { return SliceOfN(Byte(), i, j).AsAny() },
		func(i, j int) *Generator[any] { return SliceOfNDistinct(Byte(), i, j, ID[byte]).AsAny() },
		func(i, j int) *Generator[any] {
			return SliceOfNDistinct(Int(), i, j, func(n int) int { return n % j }).AsAny()
		},
		func(i, j int) *Generator[any] { return MapOfN(Int(), Int(), i, j).AsAny() },
		func(i, j int) *Generator[any] { return MapOfNValues(Int(), i, j, ID[int]).AsAny() },
		func(i, j int) *Generator[any] {
			return MapOfNValues(Int(), i, j, func(n int) int { return n % j }).AsAny()
		},
	}

	for i, gf := range genFuncs {
		t.Run(strconv.Itoa(i), MakeCheck(func(t *T) {
			minLen := IntRange(0, 256).Draw(t, "minLen")
			maxLen := IntMin(minLen).Draw(t, "maxLen")

			s := rv(gf(minLen, maxLen).Draw(t, "s"))
			if s.Len() < minLen {
				t.Fatalf("got collection of length %v with minLen %v", s.Len(), minLen)
			}
			if s.Len() > maxLen {
				t.Fatalf("got collection of length %v with maxLen %v", s.Len(), maxLen)
			}
		}))
	}
}
