// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import (
	"math"
	"testing"
)

func TestFloatConversionRoundtrip(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		u := uint32(t.s.drawBits(32))
		f := math.Float32frombits(u)
		if math.IsNaN(float64(f)) {
			t.Skip("NaN") // we can get NaNs with different bit patterns back
		}
		g := float32(float64(f))
		if g != f {
			t.Fatalf("got %v (0x%x) back from %v (0x%x)", g, math.Float32bits(g), f, math.Float32bits(f))
		}
	})
}

func TestUfloat32FromParts(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		f := Float32Min(0).Draw(t, "f")
		g := ufloat32FromParts(ufloat32Parts(f))
		if g != f {
			t.Fatalf("got %v (0x%x) back from %v (0x%x)", g, math.Float32bits(g), f, math.Float32bits(f))
		}
	})
}

func TestUfloat64FromParts(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		f := Float64Min(0).Draw(t, "f")
		g := ufloat64FromParts(ufloat64Parts(f))
		if g != f {
			t.Fatalf("got %v (0x%x) back from %v (0x%x)", g, math.Float64bits(g), f, math.Float64bits(f))
		}
	})
}

func TestGenUfloat32Range(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		min := Float32Min(0).Draw(t, "min")
		max := Float32Min(0).Draw(t, "max")
		if min > max {
			min, max = max, min
		}

		f := ufloat32FromParts(genUfloatRange(t.s, float64(min), float64(max), float32SignifBits))
		if f < min || f > max {
			t.Fatalf("%v (0x%x) outside of [%v, %v] ([0x%x, 0x%x])", f, math.Float32bits(f), min, max, math.Float32bits(min), math.Float32bits(max))
		}
	})
}

func TestGenUfloat64Range(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		min := Float64Min(0).Draw(t, "min")
		max := Float64Min(0).Draw(t, "max")
		if min > max {
			min, max = max, min
		}

		f := ufloat64FromParts(genUfloatRange(t.s, min, max, float64SignifBits))
		if f < min || f > max {
			t.Fatalf("%v (0x%x) outside of [%v, %v] ([0x%x, 0x%x])", f, math.Float64bits(f), min, max, math.Float64bits(min), math.Float64bits(max))
		}
	})
}

func TestGenFloat32Range(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		min := Float32().Draw(t, "min")
		max := Float32().Draw(t, "max")
		if min > max {
			min, max = max, min
		}

		f := float32FromParts(genFloatRange(t.s, float64(min), float64(max), float32SignifBits))
		if f < min || f > max {
			t.Fatalf("%v (0x%x) outside of [%v, %v] ([0x%x, 0x%x])", f, math.Float32bits(f), min, max, math.Float32bits(min), math.Float32bits(max))
		}
	})
}

func TestGenFloat64Range(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		min := Float64().Draw(t, "min")
		max := Float64().Draw(t, "max")
		if min > max {
			min, max = max, min
		}

		f := float64FromParts(genFloatRange(t.s, min, max, float64SignifBits))
		if f < min || f > max {
			t.Fatalf("%v (0x%x) outside of [%v, %v] ([0x%x, 0x%x])", f, math.Float64bits(f), min, max, math.Float64bits(min), math.Float64bits(max))
		}
	})
}
