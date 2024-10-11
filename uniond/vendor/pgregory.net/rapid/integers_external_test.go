// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"flag"
	"math/bits"
	"reflect"
	"sort"
	"strconv"
	"testing"

	. "pgregory.net/rapid"
)

var (
	flaky = flag.Bool("flaky.ext", false, "run flaky external tests")
	rv    = reflect.ValueOf
)

func TestIntExamples(t *testing.T) {
	gens := []*Generator[any]{
		Int().AsAny(),
		IntMin(-3).AsAny(),
		IntMax(3).AsAny(),
		IntRange(-3, 7).AsAny(),
		IntRange(-1000, 1000000).AsAny(),
		IntRange(0, 9).AsAny(),
		IntRange(0, 15).AsAny(),
		IntRange(10, 100).AsAny(),
		IntRange(100, 10000).AsAny(),
		IntRange(100, 1000000).AsAny(),
		IntRange(100, 1<<60-1).AsAny(),
	}

	for _, g := range gens {
		t.Run(g.String(), func(t *testing.T) {
			var vals []int
			for i := 0; i < 100; i++ {
				vals = append(vals, g.Example().(int))
			}
			sort.Ints(vals)

			for _, i := range vals {
				t.Log(i)
			}
		})
	}
}

func TestIntMinMaxRange(t *testing.T) {
	t.Parallel()

	data := []struct {
		g      *Generator[any]
		min    func(any) *Generator[any]
		max    func(any) *Generator[any]
		range_ func(any, any) *Generator[any]
	}{
		{
			Int().AsAny(),
			func(i any) *Generator[any] { return IntMin(i.(int)).AsAny() },
			func(i any) *Generator[any] { return IntMax(i.(int)).AsAny() },
			func(i, j any) *Generator[any] { return IntRange(i.(int), j.(int)).AsAny() },
		},
		{
			Int8().AsAny(),
			func(i any) *Generator[any] { return Int8Min(i.(int8)).AsAny() },
			func(i any) *Generator[any] { return Int8Max(i.(int8)).AsAny() },
			func(i, j any) *Generator[any] { return Int8Range(i.(int8), j.(int8)).AsAny() },
		},
		{
			Int16().AsAny(),
			func(i any) *Generator[any] { return Int16Min(i.(int16)).AsAny() },
			func(i any) *Generator[any] { return Int16Max(i.(int16)).AsAny() },
			func(i, j any) *Generator[any] { return Int16Range(i.(int16), j.(int16)).AsAny() },
		},
		{
			Int32().AsAny(),
			func(i any) *Generator[any] { return Int32Min(i.(int32)).AsAny() },
			func(i any) *Generator[any] { return Int32Max(i.(int32)).AsAny() },
			func(i, j any) *Generator[any] { return Int32Range(i.(int32), j.(int32)).AsAny() },
		},
		{
			Int64().AsAny(),
			func(i any) *Generator[any] { return Int64Min(i.(int64)).AsAny() },
			func(i any) *Generator[any] { return Int64Max(i.(int64)).AsAny() },
			func(i, j any) *Generator[any] { return Int64Range(i.(int64), j.(int64)).AsAny() },
		},
	}

	for _, d := range data {
		t.Run(d.g.String(), MakeCheck(func(t *T) {
			min := d.g.Draw(t, "min")
			max := d.g.Draw(t, "max")
			if rv(min).Int() > rv(max).Int() {
				t.Skip("min > max")
			}

			i := d.min(min).Draw(t, "i")
			if rv(i).Int() < rv(min).Int() {
				t.Fatalf("got %v which is less than min %v", i, min)
			}

			j := d.max(max).Draw(t, "j")
			if rv(j).Int() > rv(max).Int() {
				t.Fatalf("got %v which is more than max %v", j, max)
			}

			k := d.range_(min, max).Draw(t, "k")
			if rv(k).Int() < rv(min).Int() || rv(k).Int() > rv(max).Int() {
				t.Fatalf("got %v which is out of bounds [%v, %v]", k, min, max)
			}
		}))
	}
}

func TestUintMinMaxRange(t *testing.T) {
	t.Parallel()

	data := []struct {
		g      *Generator[any]
		min    func(any) *Generator[any]
		max    func(any) *Generator[any]
		range_ func(any, any) *Generator[any]
	}{
		{
			Byte().AsAny(),
			func(i any) *Generator[any] { return ByteMin(i.(byte)).AsAny() },
			func(i any) *Generator[any] { return ByteMax(i.(byte)).AsAny() },
			func(i, j any) *Generator[any] { return ByteRange(i.(byte), j.(byte)).AsAny() },
		},
		{
			Uint().AsAny(),
			func(i any) *Generator[any] { return UintMin(i.(uint)).AsAny() },
			func(i any) *Generator[any] { return UintMax(i.(uint)).AsAny() },
			func(i, j any) *Generator[any] { return UintRange(i.(uint), j.(uint)).AsAny() },
		},
		{
			Uint8().AsAny(),
			func(i any) *Generator[any] { return Uint8Min(i.(uint8)).AsAny() },
			func(i any) *Generator[any] { return Uint8Max(i.(uint8)).AsAny() },
			func(i, j any) *Generator[any] { return Uint8Range(i.(uint8), j.(uint8)).AsAny() },
		},
		{
			Uint16().AsAny(),
			func(i any) *Generator[any] { return Uint16Min(i.(uint16)).AsAny() },
			func(i any) *Generator[any] { return Uint16Max(i.(uint16)).AsAny() },
			func(i, j any) *Generator[any] { return Uint16Range(i.(uint16), j.(uint16)).AsAny() },
		},
		{
			Uint32().AsAny(),
			func(i any) *Generator[any] { return Uint32Min(i.(uint32)).AsAny() },
			func(i any) *Generator[any] { return Uint32Max(i.(uint32)).AsAny() },
			func(i, j any) *Generator[any] { return Uint32Range(i.(uint32), j.(uint32)).AsAny() },
		},
		{
			Uint64().AsAny(),
			func(i any) *Generator[any] { return Uint64Min(i.(uint64)).AsAny() },
			func(i any) *Generator[any] { return Uint64Max(i.(uint64)).AsAny() },
			func(i, j any) *Generator[any] { return Uint64Range(i.(uint64), j.(uint64)).AsAny() },
		},
		{
			Uintptr().AsAny(),
			func(i any) *Generator[any] { return UintptrMin(i.(uintptr)).AsAny() },
			func(i any) *Generator[any] { return UintptrMax(i.(uintptr)).AsAny() },
			func(i, j any) *Generator[any] { return UintptrRange(i.(uintptr), j.(uintptr)).AsAny() },
		},
	}

	for _, d := range data {
		t.Run(d.g.String(), MakeCheck(func(t *T) {
			min := d.g.Draw(t, "min")
			max := d.g.Draw(t, "max")
			if rv(min).Uint() > rv(max).Uint() {
				t.Skip("min > max")
			}

			i := d.min(min).Draw(t, "i")
			if rv(i).Uint() < rv(min).Uint() {
				t.Fatalf("got %v which is less than min %v", i, min)
			}

			j := d.max(max).Draw(t, "j")
			if rv(j).Uint() > rv(max).Uint() {
				t.Fatalf("got %v which is more than max %v", j, max)
			}

			k := d.range_(min, max).Draw(t, "k")
			if rv(k).Uint() < rv(min).Uint() || rv(k).Uint() > rv(max).Uint() {
				t.Fatalf("got %v which is out of bounds [%v, %v]", k, min, max)
			}
		}))
	}
}

func TestIntBoundCoverage(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		min := Int().Draw(t, "min")
		max := Int().Draw(t, "max")
		if min > max {
			min, max = max, min
		}

		g := IntRange(min, max)
		var gotMin, gotMax, gotZero bool
		for i := 0; i < 250; i++ {
			n := g.Example(i)

			gotMin = gotMin || n == min
			gotMax = gotMax || n == max
			gotZero = gotZero || n == 0

			if gotMin && gotMax && (min > 0 || max < 0 || gotZero) {
				return
			}
		}

		t.Fatalf("[%v, %v]: got min %v, got max %v, got zero %v", min, max, gotMin, gotMax, gotZero)
	})
}

func TestByteCoverage(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	for b := 0; b < 256; b++ {
		t.Run(strconv.Itoa(b), func(t *testing.T) {
			_ = Byte().Filter(func(v byte) bool { return v == byte(b) }).Example()
		})
	}
}

func TestIntCoverage(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	filters := []func(int) bool{
		func(i int) bool { return i == 0 },
		func(i int) bool { return i == 1 },
		func(i int) bool { return i == -1 },
		func(i int) bool { return i%2 == 0 },
		func(i int) bool { return i%17 == 0 },
		func(i int) bool { return i > 0 && i < 100 },
		func(i int) bool { return i < 0 && i > -100 },
		func(i int) bool { return i > 1<<30 },
		func(i int) bool { return i < -(1 << 30) },
	}

	if bits.UintSize == 64 {
		filters = append(filters, func(i int) bool { return i > 1<<62 })
		filters = append(filters, func(i int) bool { return i < -(1 << 62) })
	}

	for i, fn := range filters {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			_ = Int().Filter(fn).Example()
		})
	}
}

func TestUintCoverage(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	filters := []func(uint) bool{
		func(i uint) bool { return i == 0 },
		func(i uint) bool { return i == 1 },
		func(i uint) bool { return i%2 == 0 },
		func(i uint) bool { return i%17 == 0 },
		func(i uint) bool { return i > 0 && i < 100 },
		func(i uint) bool { return i > 1<<31 },
	}

	if bits.UintSize == 64 {
		filters = append(filters, func(i uint) bool { return i > 1<<63 })
	}

	for i, fn := range filters {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			_ = Uint().Filter(fn).Example()
		})
	}
}
