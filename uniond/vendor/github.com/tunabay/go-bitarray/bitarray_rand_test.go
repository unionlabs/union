// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"math/rand"
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestRand(t *testing.T) {
	ba0, err := bitarray.Rand(0)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	ba0.V()
	if ba0.Len() != 0 {
		t.Errorf("unexpected Len: got %d, want 0", ba0.Len())
		t.Logf("data: %s", ba0.D())
	}
	if !ba0.IsZero() {
		t.Errorf("unexpected IsZero: got %t, want true", ba0.IsZero())
		t.Logf("data: %s", ba0.D())
	}

	sizes := []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 15, 16, 17, 63, 64, 65, 257}
	for _, sz := range sizes {
		expUniq := 8192
		if sz < 13 {
			expUniq = 1 << sz
		}
		hist := make(map[string]int, expUniq)
		for i := 0; i < 8192; i++ {
			ba, err := bitarray.Rand(sz)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			ba.V()
			if ba.Len() != sz {
				t.Errorf("unexpected Len: got %d, want %d", ba.Len(), sz)
				t.Logf("data: %s", ba.D())
			}
			s := ba.String()
			if cnt, dup := hist[s]; dup {
				hist[s] = cnt + 1
			} else {
				hist[s] = 1
			}
		}
		passUniq := expUniq
		switch sz {
		case 15:
			passUniq = 5734 // -30%
		case 16:
			passUniq = 6553 // -20%
		case 17:
			passUniq = 7373 // -10%
		}
		uniq := len(hist)
		if uniq < passUniq {
			t.Errorf(
				"sz=%d: data not distributed as expected: uniq=%d, threshold=%d, ideal=%d",
				sz, uniq, passUniq, expUniq,
			)
		} else {
			// t.Logf("sz=%d: uniq=%d, threshold=%d, ideal=%d", sz, uniq, passUniq, expUniq)
		}
	}
}

func TestPseudoRand(t *testing.T) {
	ba0 := bitarray.PseudoRand(0, nil)
	ba0.V()
	if ba0.Len() != 0 {
		t.Errorf("unexpected Len: got %d, want 0", ba0.Len())
		t.Logf("data: %s", ba0.D())
	}
	if !ba0.IsZero() {
		t.Errorf("unexpected IsZero: got %t, want true", ba0.IsZero())
		t.Logf("data: %s", ba0.D())
	}

	myRand := rand.New(rand.NewSource(1234))
	sizes := []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 15, 16, 17, 63, 64, 65, 257}
	for i, sz := range sizes {
		randSrc := myRand
		if i&1 == 0 {
			randSrc = nil
		}
		expUniq := 8192
		if sz < 13 {
			expUniq = 1 << sz
		}
		hist := make(map[string]int, expUniq)
		for i := 0; i < 8192; i++ {
			ba := bitarray.PseudoRand(sz, randSrc)
			ba.V()
			if ba.Len() != sz {
				t.Errorf("unexpected Len: got %d, want %d", ba.Len(), sz)
				t.Logf("data: %s", ba.D())
			}
			s := ba.String()
			if cnt, dup := hist[s]; dup {
				hist[s] = cnt + 1
			} else {
				hist[s] = 1
			}
		}
		passUniq := expUniq
		switch sz {
		case 15:
			passUniq = 5734 // -30%
		case 16:
			passUniq = 6553 // -20%
		case 17:
			passUniq = 7373 // -10%
		}
		uniq := len(hist)
		if uniq < passUniq {
			t.Errorf(
				"sz=%d: data not distributed as expected: uniq=%d, threshold=%d, ideal=%d",
				sz, uniq, passUniq, expUniq,
			)
		} else {
			// t.Logf("sz=%d: uniq=%d, threshold=%d, ideal=%d", sz, uniq, passUniq, expUniq)
		}
	}
}
