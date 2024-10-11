// Copyright 2017 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License. See the AUTHORS file
// for names of contributors.

package randvar

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestZeta(t *testing.T) {
	var zetaTests = []struct {
		n        uint64
		theta    float64
		expected float64
	}{
		{20, 0.99, 3.64309060779367},
		{200, 0.99, 6.02031118558},
		{1000, 0.99, 7.72895321728},
		{2000, 0.99, 8.47398788329},
		{10000, 0.99, 10.2243614596},
		{100000, 0.99, 12.7783380626},
		{1000000, 0.99, 15.391849746},
		{10000000, 0.99, 18.066242575},
		// TODO(peter): The last test case takes an excessively long time to run (7s).
		// {100000000, 0.99, 20.80293049},
	}

	t.Run("FromScratch", func(t *testing.T) {
		for _, test := range zetaTests {
			computedZeta := computeZetaFromScratch(test.n, test.theta)
			if math.Abs(computedZeta-test.expected) > 0.000000001 {
				t.Fatalf("expected %6.4f, got %6.4f", test.expected, computedZeta)
			}
		}
	})

	t.Run("Incrementally", func(t *testing.T) {
		// Theta cannot be 1 by definition, so this is a safe initial value.
		oldTheta := 1.0
		var oldZetaN float64
		var oldN uint64
		for _, test := range zetaTests {
			// If theta has changed, recompute from scratch
			if test.theta != oldTheta {
				oldZetaN = computeZetaFromScratch(test.n, test.theta)
				oldN = test.n
				continue
			}

			computedZeta := computeZetaIncrementally(oldN, test.n, test.theta, oldZetaN)
			if math.Abs(computedZeta-test.expected) > 0.000000001 {
				t.Fatalf("expected %6.4f, got %6.4f", test.expected, computedZeta)
			}

			oldZetaN = computedZeta
			oldN = test.n
		}
	})
}

func TestZetaIncMax(t *testing.T) {
	// Construct a zipf generator covering the range [0,10] incrementally.
	z0, err := NewZipf(0, 0, 0.99)
	require.NoError(t, err)

	for i := 0; i < 10; i++ {
		z0.IncMax(1)
	}

	// Construct a zipf generator covering the range [0,10] via the constructor.
	z10, err := NewZipf(0, 10, 0.99)
	require.NoError(t, err)

	z0.mu.Lock()
	defer z0.mu.Unlock()
	z10.mu.Lock()
	defer z10.mu.Unlock()
	if z0.mu.zetaN != z10.mu.zetaN {
		t.Fatalf("expected zetaN %v, but found %v", z10.mu.zetaN, z0.mu.zetaN)
	}
	if z0.mu.eta != z10.mu.eta {
		t.Fatalf("expected eta %v, but found %v", z10.mu.eta, z0.mu.eta)
	}
}

func TestNewZipf(t *testing.T) {
	var gens = []struct {
		min, max uint64
		theta    float64
	}{
		{0, 100, 0.99},
		{0, 100, 1.01},
	}

	for _, gen := range gens {
		_, err := NewZipf(gen.min, gen.max, gen.theta)
		require.NoError(t, err)
	}
}

func TestZipf(t *testing.T) {
	rng := NewRand()
	z, err := NewZipf(0, 99, 0.99)
	require.NoError(t, err)

	x := make([]int, 10000)
	for i := range x {
		x[i] = int(z.Uint64(rng))
	}

	if testing.Verbose() {
		dumpSamples(x)
	}
}
