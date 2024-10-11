// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"errors"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/tunabay/go-bitarray"
)

// this also tests JoinBitArrayer.
func TestJoin(t *testing.T) {
	chk := func(got, want *bitarray.BitArray, src []*bitarray.BitArray) {
		t.Helper()
		got.V()
		if !got.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
			for i, s := range src {
				t.Logf("data: #%2d: %#b", i, s)
			}
			t.FailNow()
		}
	}
	test := func(s, sep string, elems ...string) {
		t.Helper()
		want := bitarray.MustParse(s)
		sepB := bitarray.MustParse(sep)
		bas := make([]*bitarray.BitArray, len(elems))
		basEA := make([]*bitarray.BitArray, len(elems))
		basE0 := make([]*bitarray.BitArray, len(elems))
		basE1 := make([]*bitarray.BitArray, len(elems))
		basI := make([]bitarray.BitArrayer, len(elems))
		basIEA := make([]bitarray.BitArrayer, len(elems))
		basIE0 := make([]bitarray.BitArrayer, len(elems))
		basIE1 := make([]bitarray.BitArrayer, len(elems))
		for i, elem := range elems {
			bas[i] = bitarray.MustParse(elem).ZOptimize()
			basI[i] = bas[i]
			basEA[i] = bas[i].ZExpand()
			basIEA[i] = basEA[i]
			basE0[i] = bas[i]
			basIE0[i] = basE0[i]
			if i&1 == 0 {
				basE0[i] = basEA[i]
				basIE0[i] = basE0[i]
			}
			basE1[i] = bas[i]
			basIE1[i] = basE1[i]
			if i&1 == 1 {
				basE1[i] = basEA[i]
				basIE1[i] = basE1[i]
			}
		}
		chk(bitarray.Join(bas, sepB), want, bas)
		chk(bitarray.Join(basEA, sepB), want, basEA)
		chk(bitarray.Join(basE0, sepB), want, basE0)
		chk(bitarray.Join(basE1, sepB), want, basE1)
		chk(bitarray.JoinBitArrayer(basI, sepB), want, bas)
		chk(bitarray.JoinBitArrayer(basIEA, sepB), want, basEA)
		chk(bitarray.JoinBitArrayer(basIE0, sepB), want, basE0)
		chk(bitarray.JoinBitArrayer(basIE1, sepB), want, basE1)
	}
	test("", "")
	test("", "111")
	test("", "", "")
	test("", "111", "")
	test("", "", "", "", "", "")
	test("101101101", "101", "", "", "", "")
	test("111", "00", "111")
	test("1111-1111", "00", "1111-1111")
	test("1111-0000 1010", "0000", "1111", "1010")
	test(
		"1010-1010 11 0000-0000 1111-1111 1111-1111 1111-1111 11"+
			" 0000-0000 0000-0000 0000-0000 1111-1111 1111-1111"+
			" 0000-0000 0000-0000 0000-0000 00",
		"",
		"1010-1010 11",
		"0000-0000",
		"1111-1111",
		"1111-1111 1111-1111 11",
		"0000-0000 0000-0000 0000-0000",
		"1111-1111 1111-1111",
		"0000-0000 0000-0000 0000-0000 00",
	)
}

func TestJoinBitArrayer_edge(t *testing.T) {
	sep := bitarray.MustParse("1111")
	z := bitarray.New()
	if got := bitarray.JoinBitArrayer(nil, nil); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer(nil, z); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer(nil, sep); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}

	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{}, nil); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{}, z); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{}, sep); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}

	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{nil}, nil); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{nil}, z); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{nil}, sep); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{z}, nil); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{z}, z); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{z}, sep); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}

	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{nil, nil}, nil); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{nil, nil}, z); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{nil, nil}, sep); !got.Equal(sep) {
		t.Errorf("unexpected result: got %#b, want %#b", got, sep)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{z, z}, nil); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{z, z}, z); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := bitarray.JoinBitArrayer([]bitarray.BitArrayer{z, z}, sep); !got.Equal(sep) {
		t.Errorf("unexpected result: got %#b, want %#b", got, sep)
	}
}

func TestBitArray_Append_edge(t *testing.T) {
	var ba0 *bitarray.BitArray
	ba1 := bitarray.MustParse("1010")
	if got := ba0.Append(); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := ba1.Append(); !got.Equal(ba1) {
		t.Errorf("unexpected result: got %#b, want %#b", got, ba1)
	}

	baL := bitarray.MustParse("1111-1100 0000-0000 0000-0000 00")
	baS := baL.Slice(0, 7)
	baA := bitarray.NewZeroFilled(8)
	baZ := bitarray.New()
	want := bitarray.MustParse("1111-1100 0000-000")
	if got := baS.Append(baA); !got.Equal(want) {
		t.Errorf("unexpected result: got %#b, want %#b", got, want)
	}

	if got := baS.Append(nil); !got.Equal(baS) {
		t.Errorf("unexpected result: got %#b, want %#b", got, baS)
	}
	if got := baS.Append(baZ); !got.Equal(baS) {
		t.Errorf("unexpected result: got %#b, want %#b", got, baS)
	}
	if got := baZ.Append(baZ); !got.IsZero() {
		t.Errorf("unexpected result: got %#b, want zero", got)
	}
	if got := ba0.Append(baS); !got.Equal(baS) {
		t.Errorf("unexpected result: got %#b, want %#b", got, baS)
	}
}

func TestBitArray_Append_rand(t *testing.T) {
	const testIterations = 10000
	rand.Seed(time.Now().UnixNano())

	for i := 0; i < testIterations; i++ {
		var ba *bitarray.BitArray
		switch rand.Intn(20) {
		case 0, 1, 2, 3:
			nBits := rand.Intn(64)
			ba = bitarray.PseudoRand(nBits, nil)
		case 4, 5, 6, 7:
			nBits := 8*(1+rand.Intn(64)) - 1 + rand.Intn(3)
			ba = bitarray.PseudoRand(nBits, nil)
		case 8:
			nBits := rand.Intn(1024)
			ba = bitarray.NewZeroFilled(nBits)
		case 9:
			nBits := rand.Intn(1024)
			ba = bitarray.NewOneFilled(nBits)
		default:
			nBits := rand.Intn(1024)
			ba = bitarray.PseudoRand(nBits, nil)
		}
		lim := ba.Len()
		if rand.Intn(100) == 0 {
			lim = ba.Len() - rand.Intn(ba.Len()>>1+1)
		}
		var myErr error
		if rand.Intn(50) == 0 {
			myErr = fmt.Errorf("custom error %d", i)
		}

		baS := ba.String()

		fn := func(i, b int) error {
			t.Helper()
			if baS[i] != '0'+byte(b) {
				return fmt.Errorf("%d: got %d, want %c", i, b, baS[i])
			}
			if lim < i {
				if myErr == nil {
					return bitarray.BreakIteration
				}
				return fmt.Errorf("test error: %d: %w", i, myErr)
			}
			return nil
		}
		if err := ba.Iterate(fn); err != nil {
			if myErr == nil || !errors.Is(err, myErr) {
				t.Errorf("unexpected error: %s", err)
				t.FailNow()
			}
		}
		if err := ba.ZExpand().Iterate(fn); err != nil {
			if myErr == nil || !errors.Is(err, myErr) {
				t.Errorf("unexpected bit (e): %s", err)
				t.FailNow()
			}
		}
		// if i < 32 {
		// 	t.Logf("pass: %#b", ba)
		// }
	}
}

func TestBitArray_Repeat_edge(t *testing.T) {
	ba1 := bitarray.NewZeroFilled(123)
	ba2 := ba1.Repeat(100)
	if !ba2.Equal(bitarray.NewZeroFilled(12300)) {
		t.Errorf("unexpected result: %#b", ba2)
		t.Logf("data: %s", ba2.D())
	}
	ba3 := ba1.Repeat(0)
	if !ba3.IsZero() {
		t.Errorf("unexpected result: %#b", ba3)
		t.Logf("data: %s", ba3.D())
	}
	ba4 := bitarray.New().Repeat(999)
	if !ba4.IsZero() {
		t.Errorf("unexpected result: %#b", ba4)
		t.Logf("data: %s", ba4.D())
	}
	func() {
		var ba *bitarray.BitArray
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: got %#b", ba)
			}
		}()
		ba = bitarray.MustParse("1010-10").Repeat(-1)
	}()
}
