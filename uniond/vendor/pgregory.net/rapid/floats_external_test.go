// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"math"
	"sort"
	"testing"

	. "pgregory.net/rapid"
)

func TestFloatNoInf(t *testing.T) {
	t.Parallel()

	gens := []*Generator[any]{
		Float32().AsAny(),
		Float32Min(0).AsAny(),
		Float32Max(0).AsAny(),
		Float64().AsAny(),
		Float64Min(0).AsAny(),
		Float64Max(0).AsAny(),
	}

	for _, g := range gens {
		t.Run(g.String(), MakeCheck(func(t *T) {
			f := g.Draw(t, "f")
			if math.IsInf(rv(f).Float(), 0) {
				t.Fatalf("got infinity: %v", f)
			}
		}))
	}
}

func TestFloatExamples(t *testing.T) {
	gens := []*Generator[any]{
		Float32().AsAny(),
		Float32Min(-0.1).AsAny(),
		Float32Min(1).AsAny(),
		Float32Max(0.1).AsAny(),
		Float32Max(2.5).AsAny(),
		Float32Range(0.3, 0.30001).AsAny(),
		Float32Range(0.3, 0.301).AsAny(),
		Float32Range(0.3, 0.7).AsAny(),
		Float32Range(math.E, math.Pi).AsAny(),
		Float32Range(0, 1).AsAny(),
		Float32Range(1, 2.5).AsAny(),
		Float32Range(0, 100).AsAny(),
		Float32Range(0, 10000).AsAny(),
		Float64().AsAny(),
		Float64Min(-0.1).AsAny(),
		Float64Min(1).AsAny(),
		Float64Max(0.1).AsAny(),
		Float64Max(2.5).AsAny(),
		Float64Range(0.3, 0.30000001).AsAny(),
		Float64Range(0.3, 0.301).AsAny(),
		Float64Range(0.3, 0.7).AsAny(),
		Float64Range(math.E, math.Pi).AsAny(),
		Float64Range(0, 1).AsAny(),
		Float64Range(1, 2.5).AsAny(),
		Float64Range(0, 100).AsAny(),
		Float64Range(0, 10000).AsAny(),
	}

	for _, g := range gens {
		t.Run(g.String(), func(t *testing.T) {
			var vals []float64
			var vals32 bool
			for i := 0; i < 100; i++ {
				f := g.Example()
				_, vals32 = f.(float32)
				vals = append(vals, rv(f).Float())
			}
			sort.Float64s(vals)

			for _, f := range vals {
				if vals32 {
					t.Logf("%30g %10.3g % 5d % 20d % 16x", f, f, int(math.Log10(math.Abs(f))), int64(f), math.Float32bits(float32(f)))
				} else {
					t.Logf("%30g %10.3g % 5d % 20d % 16x", f, f, int(math.Log10(math.Abs(f))), int64(f), math.Float64bits(f))
				}
			}
		})
	}
}

func TestFloat32BoundCoverage(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		min := Float32().Draw(t, "min")
		max := Float32().Draw(t, "max")
		if min > max {
			min, max = max, min
		}

		g := Float32Range(min, max)
		var gotMin, gotMax, gotZero bool
		for i := 0; i < 400; i++ {
			f := g.Example(i)

			gotMin = gotMin || f == min
			gotMax = gotMax || f == max
			gotZero = gotZero || f == 0

			if gotMin && gotMax && (min > 0 || max < 0 || gotZero) {
				return
			}
		}

		t.Fatalf("[%v, %v]: got min %v, got max %v, got zero %v", min, max, gotMin, gotMax, gotZero)
	})
}

func TestFloat64BoundCoverage(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		min := Float64().Draw(t, "min")
		max := Float64().Draw(t, "max")
		if min > max {
			min, max = max, min
		}

		g := Float64Range(min, max)
		var gotMin, gotMax, gotZero bool
		for i := 0; i < 400; i++ {
			f := g.Example(i)

			gotMin = gotMin || f == min
			gotMax = gotMax || f == max
			gotZero = gotZero || f == 0

			if gotMin && gotMax && (min > 0 || max < 0 || gotZero) {
				return
			}
		}

		t.Fatalf("[%v, %v]: got min %v, got max %v, got zero %v", min, max, gotMin, gotMax, gotZero)
	})
}
