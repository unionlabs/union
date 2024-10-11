// Copyright 2017 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// This file validates that the calculation in Uint64n corrects for
// possible bias.

package rand

import (
	"testing"
)

// modSource is used to probe the upper region of uint64 space. It
// generates values sequentially in [maxUint64-15,maxUint64]. With
// modEdge == 15 and maxUint64 == 1<<64-1 == 18446744073709551615,
// this means that Uint64n(10) will repeatedly probe the top range.
// We thus expect a bias to result unless the calculation in Uint64n
// gets the edge condition right. We test this by calling Uint64n 100
// times; the results should be perfectly evenly distributed across
// [0,10).
type modSource uint64

const modEdge = 15

func (m *modSource) Seed(uint64) {}

// Uint64 returns a non-pseudo-random 64-bit unsigned integer as a uint64.
func (m *modSource) Uint64() uint64 {
	if *m > modEdge {
		*m = 0
	}
	r := maxUint64 - *m
	*m++
	return uint64(r)
}

func TestUint64Modulo(t *testing.T) {
	var src modSource
	rng := New(&src)
	var result [10]uint64
	for i := 0; i < 100; i++ {
		result[rng.Uint64n(10)]++
	}
	for _, r := range result {
		if r != 10 {
			t.Fatal(result)
		}
	}
}
