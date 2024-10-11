// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"
	"math/big"
	"math/rand"
	"testing"
	"time"

	"github.com/tunabay/go-bitarray"
)

// also tests ToInt(), ToUint64()
func TestNewFromInt_rand(t *testing.T) {
	const testIterations = 30000
	rand.Seed(time.Now().UnixNano())

	rnd := rand.New(rand.NewSource(time.Now().UnixNano()))

	u32Max := big.NewInt(int64(0xffffffff))
	u64Max := big.NewInt(0)
	u64Max.Lsh(u32Max, 32)
	u64Max.Or(u64Max, u32Max)
	// t.Logf("MAX: % x", u64Max)

	rndMax := big.NewInt(0)
	rndMax.Lsh(u64Max, 64)
	rndMax.Or(rndMax, u64Max) // 128 bit
	// t.Logf("MAX: %x", rndMax)

	for i := 0; i < testIterations; i++ {
		v := big.NewInt(0)
		v.Rand(rnd, rndMax)
		v.Rsh(v, uint(rand.Intn(128)))
		vbin := fmt.Sprintf("%b", v)

		ba := bitarray.NewFromInt(v)
		ba.V()
		want := bitarray.MustParse(vbin)

		if !ba.Equal(want) {
			t.Errorf("unexpected result: %d", v)
			t.Logf(" got: %#b", ba)
			t.Logf(" got: %s", ba.D())
			t.Logf("want: %#b", want)
			t.FailNow()
		}
		// if i < 64 {
		// 	t.Logf("pass: %d", v)
		// 	t.Logf(" got: %x", ba)
		// }

		baO := ba.ZOptimize()
		baE := ba.ZExpand()
		gotO := baO.ToInt()
		gotE := baE.ToInt()
		if gotO.Cmp(v) != 0 {
			t.Errorf("unexpected value: got %x, want %x", gotO, v)
		}
		if gotE.Cmp(v) != 0 {
			t.Errorf("unexpected value: got %x, want %x", gotE, v)
		}

		v64b := big.NewInt(0)
		v64b.And(v, u64Max)
		v64 := v64b.Uint64()
		got64O := baO.ToUint64()
		got64E := baE.ToUint64()
		if got64O != v64 {
			t.Errorf("unexpected value: got %x, want %x", gotO, v64)
		}
		if got64E != v64 {
			t.Errorf("unexpected value: got %x, want %x", gotE, v64)
		}
	}
}

func TestBitArray_ToInt_edge(t *testing.T) {
	var ba *bitarray.BitArray
	want := big.NewInt(0)
	if got := ba.ToInt(); got.Cmp(want) != 0 {
		t.Errorf("unexpected value: got %x, want %x", got, want)
	}
	ba = bitarray.New()
	if got := ba.ToInt(); got.Cmp(want) != 0 {
		t.Errorf("unexpected value: got %x, want %x", got, want)
	}
	ba = bitarray.NewZeroFilled(1000)
	if got := ba.ToInt(); got.Cmp(want) != 0 {
		t.Errorf("unexpected value: got %x, want %x", got, want)
	}
}

func TestBitArray_ToUint64_edge(t *testing.T) {
	var ba *bitarray.BitArray
	if got := ba.ToUint64(); got != 0 {
		t.Errorf("unexpected value: got %x, want 0", got)
	}
	ba = bitarray.New()
	if got := ba.ToUint64(); got != 0 {
		t.Errorf("unexpected value: got %x, want 0", got)
	}
	ba = bitarray.NewZeroFilled(1000)
	if got := ba.ToUint64(); got != 0 {
		t.Errorf("unexpected value: got %x, want 0", got)
	}
}
