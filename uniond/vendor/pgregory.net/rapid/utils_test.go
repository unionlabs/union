// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import (
	"flag"
	"fmt"
	"math"
	"math/bits"
	"sort"
	"strconv"
	"strings"
	"testing"
)

var flaky = flag.Bool("flaky", false, "run flaky tests")

func createRandomBitStream(t *testing.T) bitStream {
	t.Helper()

	seed := baseSeed()
	t.Logf("random seed %v", seed)

	return newRandomBitStream(seed, false)
}

func TestGenFloat01(t *testing.T) {
	t.Parallel()

	s1 := &bufBitStream{buf: []uint64{0}}
	f1 := genFloat01(s1)
	if f1 != 0 {
		t.Errorf("got %v instead of 0", f1)
	}

	s2 := &bufBitStream{buf: []uint64{math.MaxUint64}}
	f2 := genFloat01(s2)
	if f2 == 1 {
		t.Errorf("got impossible 1")
	}
}

func TestGenGeom(t *testing.T) {
	t.Parallel()

	s1 := &bufBitStream{buf: []uint64{0}}
	i1 := genGeom(s1, 0.1)
	if i1 != 0 {
		t.Errorf("got %v instead of 0 for 0.1", i1)
	}

	s2 := &bufBitStream{buf: []uint64{0}}
	i2 := genGeom(s2, 1)
	if i2 != 0 {
		t.Errorf("got %v instead of 0 for 1", i2)
	}
}

func TestGenGeomMean(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	s := newRandomBitStream(baseSeed(), false)

	for i := 0; i < 100; i++ {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			p := genFloat01(s)

			var geoms []uint64
			for i := 0; i < 10000; i++ {
				geoms = append(geoms, genGeom(s, p))
			}

			avg := 0.0
			for _, f := range geoms {
				avg += float64(f)
			}
			avg /= float64(len(geoms))

			mean := (1 - p) / p
			if math.Abs(avg-mean) > 0.5 { // true science
				t.Fatalf("for p=%v geom avg=%v vs expected mean=%v", p, avg, mean)
			}
		})
	}
}

func TestUintsExamplesHist(t *testing.T) {
	s := newRandomBitStream(baseSeed(), false)

	for _, n := range []int{2, 3, 4, 5, 6, 8, 16, 32, 64} {
		t.Run(strconv.Itoa(n), func(t *testing.T) {
			var lines []string
			for i := 0; i < 50; i++ {
				n, _, _ := genUintN(s, bitmask64(uint(n)), true)
				b := bits.Len64(n)
				l := fmt.Sprintf("% 24d %s % 3d", n, strings.Repeat("*", b)+strings.Repeat(" ", 64-b), b)
				lines = append(lines, l)
			}

			sort.Strings(lines)
			t.Log("\n" + strings.Join(lines, "\n"))
		})
	}
}

func ensureWithin3Sigma(t *testing.T, ctx any, y int, n int, p float64) {
	t.Helper()

	mu := float64(n) * p
	s := math.Sqrt(float64(n) * p * (1 - p))

	if float64(y) < mu-3*s || float64(y) > mu+3*s {
		if ctx != nil {
			t.Errorf("for %v: got %v out of %v (p %v, mu %v, stddev %v)", ctx, y, n, p, mu, s)
		} else {
			t.Errorf("got %v out of %v (p %v, mu %v, stddev %v)", y, n, p, mu, s)
		}
	}
}

func TestGenUintN(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	s := createRandomBitStream(t)
	max := []uint64{0, 1, 2, 5, 13}

	for _, m := range max {
		r := make([]int, m+1)
		n := 1000
		for i := 0; i < n; i++ {
			u, _, _ := genUintN(s, m, false)
			r[u]++
		}

		for u := range r {
			ensureWithin3Sigma(t, m, r[u], n, 1/float64(m+1))
		}
	}
}

func TestGenUintRange(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	s := createRandomBitStream(t)
	ranges := [][]uint64{
		{0, 0},
		{0, 1},
		{0, 2},
		{1, 1},
		{1, 3},
		{3, 7},
		{math.MaxUint64 - 3, math.MaxUint64},
		{math.MaxUint64 - 1, math.MaxUint64},
		{math.MaxUint64, math.MaxUint64},
	}

	for _, r := range ranges {
		m := map[uint64]int{}
		n := 1000
		for i := 0; i < n; i++ {
			u, _, _ := genUintRange(s, r[0], r[1], false)
			m[u]++
		}

		for u := range m {
			if u < r[0] || u > r[1] {
				t.Errorf("%v out of range [%v, %v]", u, r[0], r[1])
			}
			ensureWithin3Sigma(t, fmt.Sprintf("%v from %v", u, r), m[u], n, 1/float64(r[1]-r[0]+1))
		}
	}
}

func TestGenIntRange(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	s := createRandomBitStream(t)
	ranges := [][]int64{
		{0, 0},
		{0, 1},
		{0, 2},
		{1, 1},
		{1, 3},
		{3, 7},
		{math.MaxInt64 - 3, math.MaxInt64},
		{math.MaxInt64 - 1, math.MaxInt64},
		{math.MaxInt64, math.MaxInt64},
		{-1, -1},
		{-2, -1},
		{-3, 0},
		{-1, 1},
		{-1, 3},
		{-3, 7},
		{-7, -3},
		{math.MinInt64, math.MinInt64 + 3},
		{math.MinInt64, math.MinInt64 + 1},
		{math.MinInt64, math.MinInt64},
	}

	for _, r := range ranges {
		m := map[int64]int{}
		n := 1000
		for i := 0; i < n; i++ {
			u, _, _ := genIntRange(s, r[0], r[1], false)
			m[u]++
		}

		for u := range m {
			if u < r[0] || u > r[1] {
				t.Errorf("%v out of range [%v, %v]", u, r[0], r[1])
			}
			ensureWithin3Sigma(t, fmt.Sprintf("%v from %v", u, r), m[u], n, 1/float64(r[1]-r[0]+1))
		}
	}
}

func TestFlipBiasedCoin(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	s := createRandomBitStream(t)
	ps := []float64{0, 0.3, 0.5, 0.7, 1}

	for _, p := range ps {
		n := 1000
		y := 0
		for i := 0; i < n; i++ {
			if flipBiasedCoin(s, p) {
				y++
			}
		}

		ensureWithin3Sigma(t, p, y, n, p)
	}
}

func TestLoadedDie(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	s := createRandomBitStream(t)
	weights := [][]int{
		{1},
		{1, 2},
		{3, 2, 1},
		{1, 2, 4, 2, 1},
	}

	for _, ws := range weights {
		d := newLoadedDie(ws)
		n := 1000
		r := make([]int, len(ws))

		for i := 0; i < n; i++ {
			r[d.roll(s)]++
		}

		total := 0
		for _, w := range ws {
			total += w
		}

		for i, w := range ws {
			ensureWithin3Sigma(t, ws, r[i], n, float64(w)/float64(total))
		}
	}
}

func TestRepeat(t *testing.T) {
	t.Parallel()
	if !*flaky {
		t.Skip("flaky")
	}

	s := createRandomBitStream(t)
	mmas := [][3]int{
		{0, 0, 0},
		{0, 1, 0},
		{0, 1, 1},
		{1, 1, 1},
		{3, 3, 3},
		{3, 7, 3},
		{3, 7, 5},
		{3, 7, 7},
		{0, 10, 5},
		{1, 10, 6},
		{0, 50, 5},
		{1, 50, 6},
		{1000, math.MaxInt32, 1000 + 1},
		{1000, math.MaxInt32, 1000 + 2},
		{1000, math.MaxInt32, 1000 + 7},
		{1000, math.MaxInt32, 1000 + 13},
		{1000, math.MaxInt32, 1000 + 100},
		{1000, math.MaxInt32, 1000 + 1000},
	}

	for _, mma := range mmas {
		min, max, avg := mma[0], mma[1], mma[2]

		n := 5000
		c := make([]int, n)
		for i := 0; i < n; i++ {
			r := newRepeat(min, max, float64(avg), "")
			for r.more(s) {
				c[i]++
			}

			if c[i] < min || c[i] > max {
				t.Errorf("got %v tries with bounds [%v, %v]", c[i], min, max)
			}
		}

		if min == 1000 && max == math.MaxInt32 {
			mu := float64(0)
			for _, e := range c {
				mu += float64(e)
			}
			mu /= float64(len(c))

			diff := math.Abs(mu - float64(avg))
			if diff > 0.5 { // true science
				t.Errorf("real avg %v vs desired %v, diff %v (%v tries)", mu, avg, diff, n)
			}
		}
	}
}
