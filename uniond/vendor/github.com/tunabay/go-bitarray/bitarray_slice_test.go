// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"math/rand"
	"testing"
	"time"

	"github.com/tunabay/go-bitarray"
)

func TestBitArray_Slice(t *testing.T) {
	bs := ""
	set := func(s string) {
		t.Helper()
		bs = s
		// t.Logf("data: %q", bs)
	}
	test := func(s, e int, exp string) {
		t.Helper()
		ba := bitarray.MustParse(bs)
		sliced := ba.Slice(s, e)
		sliced.V()
		slicedE := sliced.ZExpand()
		slicedO := sliced.ZOptimize()
		expected := bitarray.MustParse(exp)
		switch {
		case !sliced.Equal(expected):
			t.Errorf("% b: [%d:%d]: unexpected slice:", ba, s, e)
			t.Logf(" got: %#b", sliced)
			t.Logf(" got: %s", sliced.D())
		case !slicedE.Equal(expected):
			t.Errorf("% b: [%d:%d]: unexpected slice (e):", ba, s, e)
			t.Logf(" got: %#b", slicedE)
			t.Logf(" got: %s", slicedE.D())
		case !slicedO.Equal(expected):
			t.Errorf("% b: [%d:%d]: unexpected slice (o):", ba, s, e)
			t.Logf(" got: %#b", slicedO)
			t.Logf(" got: %s", slicedO.D())
		}
		if t.Failed() {
			t.Logf("want: %#b", expected)
			t.FailNow()
		}
		// t.Logf("pass: [%d:%d]: % b", s, e, sliced)
	}
	testPanic := func(s, e int) {
		t.Helper()
		ba := bitarray.MustParse(bs)
		var rba *bitarray.BitArray
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected:")
				t.Logf(" got: %#b", rba)
				t.Logf(" got: %s", rba.D())
			}
		}()
		rba = ba.Slice(s, e)
	}

	set("")
	test(0, 0, "")
	testPanic(0, 1)

	set("1111-11")
	test(0, 0, "")
	test(4, 4, "")
	test(0, 3, "111")
	test(0, 6, "1111-11")
	testPanic(-1, 1)
	testPanic(1, 0)
	testPanic(5, 0)
	testPanic(5, 3)
	testPanic(99, 3)
	testPanic(0, -1)

	set("1111-0000 1010-0101 1100-11")
	test(0, 0, "")
	test(0, 1, "1")
	test(1, 3, "11")
	test(0, 3, "111")
	test(0, 8, "1111-0000")
	test(2, 8, "11-0000")
	test(3, 11, "1-0000 101")
	test(9, 15, "010-010")
	test(0, 22, "1111-0000 1010-0101 1100-11")
	test(18, 22, "0011")
	test(21, 22, "1")

	set("0000-0000 0000-0000 0000-0000 0000-0000")
	test(0, 0, "")
	test(0, 1, "0")
	test(0, 7, "0000-000")
	test(0, 8, "0000-0000")
	test(0, 9, "0000-0000 0")
	test(0, 15, "0000-0000 0000-000")
	test(0, 16, "0000-0000 0000-0000")
	test(0, 17, "0000-0000 0000-0000 0")
	test(6, 15, "00 0000-000")
	test(6, 16, "00 0000-0000")
	test(6, 17, "00 0000-0000 0")
	test(0, 31, "0000-0000 0000-0000 0000-0000 0000-000")
	test(0, 32, "0000-0000 0000-0000 0000-0000 0000-0000")
	test(15, 31, "0 0000-0000 0000-000")
	test(15, 32, "0 0000-0000 0000-0000")
	test(16, 31, "0000-0000 0000-000")
	test(16, 32, "0000-0000 0000-0000")
	test(19, 31, "0-0000 0000-000")
	test(19, 32, "0-0000 0000-0000")
	test(24, 31, "0000-000")
	test(24, 32, "0000-0000")
	test(27, 31, "0-000")
	test(27, 32, "0-0000")

	set("0000-0000 0")
	test(0, 9, "0000-0000 0")
	testPanic(0, 10)
	testPanic(9, 10)

	set("1010-0101 1010-0101 1010-0101 1010-0101")
	test(0, 0, "")
	test(0, 1, "1")
	test(3, 6, "0-01")
	test(7, 8, "1")
	test(8, 9, "1")
	test(0, 7, "1010-010")
	test(0, 8, "1010-0101")
	test(0, 9, "1010-0101 1")
	test(0, 15, "1010-0101 1010-010")
	test(0, 16, "1010-0101 1010-0101")
	test(0, 17, "1010-0101 1010-0101 1")
	test(6, 15, "01 1010-010")
	test(6, 16, "01 1010-0101")
	test(6, 17, "01 1010-0101 1")
	test(10, 14, "10-01")
	test(0, 31, "1010-0101 1010-0101 1010-0101 1010-010")
	test(0, 32, "1010-0101 1010-0101 1010-0101 1010-0101")
	test(15, 31, "1 1010-0101 1010-010")
	test(15, 32, "1 1010-0101 1010-0101")
	test(16, 31, "1010-0101 1010-010")
	test(16, 32, "1010-0101 1010-0101")
	test(19, 31, "0-0101 1010-010")
	test(19, 32, "0-0101 1010-0101")
	test(24, 31, "1010-010")
	test(24, 32, "1010-0101")
	test(26, 29, "10-0")
	test(27, 31, "0-010")
	test(27, 32, "0-0101")

	set("1110-0011 1000-1110 0011-1000 1110-0011 1000")
	test(0, 0, "")
	test(0, 1, "1")
	test(2, 6, "10-00")
	test(6, 8, "11")
	test(8, 10, "10")
	test(0, 7, "1110-001")
	test(0, 8, "1110-0011")
	test(0, 9, "1110-0011 1")
	test(0, 15, "1110-0011 1000-111")
	test(0, 16, "1110-0011 1000-1110")
	test(0, 17, "1110-0011 1000-1110 0")
	test(5, 15, "011 1000-111")
	test(5, 16, "011 1000-1110")
	test(5, 17, "011 1000-1110 0")
	test(10, 14, "00-11")
	test(0, 31, "1110-0011 1000-1110 0011-1000 1110-001")
	test(0, 32, "1110-0011 1000-1110 0011-1000 1110-0011")
	test(0, 33, "1110-0011 1000-1110 0011-1000 1110-0011 1")
	test(0, 34, "1110-0011 1000-1110 0011-1000 1110-0011 10")
	test(0, 35, "1110-0011 1000-1110 0011-1000 1110-0011 100")
	test(0, 36, "1110-0011 1000-1110 0011-1000 1110-0011 1000")
	test(14, 31, "10 0011-1000 1110-001")
	test(14, 32, "10 0011-1000 1110-0011")
	test(14, 33, "10 0011-1000 1110-0011 1")
	test(14, 34, "10 0011-1000 1110-0011 10")
	test(14, 35, "10 0011-1000 1110-0011 100")
	test(14, 36, "10 0011-1000 1110-0011 1000")
	test(16, 31, "0011-1000 1110-001")
	test(16, 32, "0011-1000 1110-0011")
	test(16, 33, "0011-1000 1110-0011 1")
	test(16, 36, "0011-1000 1110-0011 1000")
	test(19, 31, "1-1000 1110-001")
	test(19, 32, "1-1000 1110-0011")
	test(19, 33, "1-1000 1110-0011 1")
	test(19, 36, "1-1000 1110-0011 1000")
	test(24, 31, "1110-001")
	test(24, 32, "1110-0011")
	test(25, 31, "110-001")
	test(26, 29, "10-0")
	test(27, 31, "0-001")
	test(27, 32, "0-0011")
}

func TestBitArray_Slice_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())

	n := 0
	for i := 0; i < testIterations/1000; i++ {
		ba := bitarray.PseudoRand(0x100, nil) // random 256 bits
		srcStr := ba.String()
		for j := 0; j < 1000 && n < testIterations; j++ {
			ss := rand.Intn(0x101)      // 0 .. 256
			sl := rand.Intn(0x101 - ss) // 0 .. 256-ss
			se := ss + sl               // ss .. 256

			expected := bitarray.MustParse(srcStr[ss:se])
			sliced := ba.Slice(ss, se)
			sliced.V()
			slicedE := sliced.ZExpand()
			slicedO := sliced.ZOptimize()
			if !sliced.Equal(expected) || !slicedE.Equal(expected) || !slicedO.Equal(expected) {
				t.Errorf("unxepected slice of [%d:%d] len=%d:", ss, se, se-ss)
				t.Logf(" all: % s", ba)
				t.Logf("want: % s", expected)
				t.Logf("want: %s", expected.D())
				t.Logf(" got: % s", sliced)
				t.Logf(" got: %s", sliced.D())
				t.FailNow()
			}
			if i == 0 && j < 32 {
				// t.Logf("pass: [%d:%d] % s", ss, se, sliced)
			}
			n++
		}
	}
}

func TestBitArray_ToWidth(t *testing.T) {
	tcs := []struct {
		w       int
		s, l, r string
	}{
		{0, "0000-0000", "", ""},
		{0, "1111-1111", "", ""},
		{12, "1111-1111 1100", "1111-1111 1100", "1111-1111 1100"},
		{1, "1010-1010", "1", "0"},
		{5, "0000-000", "00000", "00000"},
		{4, "1100-0001", "1100", "0001"},
		// TODO: more
	}
	chk := func(got, want *bitarray.BitArray) {
		t.Helper()
		got.V()
		if !got.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
			t.FailNow()
		}
	}
	for _, tc := range tcs {
		expL := bitarray.MustParse(tc.l).ZOptimize()
		expR := bitarray.MustParse(tc.r).ZOptimize()
		ba := bitarray.MustParse(tc.s)
		baE := ba.ZExpand()

		chk(ba.ToWidth(tc.w, bitarray.AlignLeft), expL)
		chk(baE.ToWidth(tc.w, bitarray.AlignLeft), expL)
		chk(ba.ToWidth(tc.w, bitarray.AlignRight), expR)
		chk(baE.ToWidth(tc.w, bitarray.AlignRight), expR)
	}
	func() {
		var ba *bitarray.BitArray
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: got %#b", ba)
			}
		}()
		ba = bitarray.MustParse("10101").ToWidth(-1, bitarray.AlignLeft)
	}()
}

func TestBitArray_TrimPrefix(t *testing.T) {
	tdt := []string{
		"", "", "",
		"", "0011", "",
		"", "1", "",
		"0", "", "0",
		"0", "0", "",
		"0", "1", "0",
		"1", "", "1",
		"1", "0", "1",
		"1", "1", "",
		"0000-0000 0000-0000", "", "0000-0000 0000-0000",
		"0000-0000 0000-0000", "0", "0000-0000 0000-000",
		"0000-0000 0000-0000", "0000", "0000-0000 0000",
		"0000-0000 0000-0000", "0001", "0000-0000 0000-0000",
		"1111-1111 0000-0011", "0000", "1111-1111 0000-0011",
		"1111-1111 0000-0011", "111", "1111-1000 0001-1",
		"1111-1111 0000-0011", "1111-1111 00", "0000-11",
		"0101-1111 0101-1111 0101-11", "01", "0111-1101 0111-1101 0111",
	}
	chk := func(got, want *bitarray.BitArray) {
		t.Helper()
		got.V()
		if !got.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
			t.FailNow()
		}
	}
	for i := 0; i < len(tdt); i += 3 {
		ba0 := bitarray.MustParse(tdt[i]).ZOptimize()
		ba1 := bitarray.MustParse(tdt[i+1]).ZOptimize()
		exp := bitarray.MustParse(tdt[i+2])
		ba0E := ba0.ZExpand()
		ba1E := ba1.ZExpand()

		chk(ba0.TrimPrefix(ba1), exp)
		chk(ba0.TrimPrefix(ba1E), exp)
		chk(ba0E.TrimPrefix(ba1), exp)
		chk(ba0E.TrimPrefix(ba1E), exp)
	}
}

func TestBitArray_TrimSuffix(t *testing.T) {
	tdt := []string{
		"", "", "",
		"", "0011", "",
		"", "1", "",
		"0", "", "0",
		"0", "0", "",
		"0", "1", "0",
		"1", "", "1",
		"1", "0", "1",
		"1", "1", "",
		"0000-0000 0000-0000", "", "0000-0000 0000-0000",
		"0000-0000 0000-0000", "0", "0000-0000 0000-000",
		"0000-0000 0000-0000", "0000", "0000-0000 0000",
		"0000-0000 0000-0000", "0001", "0000-0000 0000-0000",
		"1111-1111 0000-0011", "0000", "1111-1111 0000-0011",
		"1111-1111 0000-0011", "011", "1111-1111 0000-0",
		"1111-1111 0000-0011", "1111 0000-0011", "1111",
		"0101-1111 0101-1111 0101-11", "0111", "0101-1111 0101-1111 01",
	}
	chk := func(got, want *bitarray.BitArray) {
		t.Helper()
		got.V()
		if !got.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
			t.FailNow()
		}
	}
	for i := 0; i < len(tdt); i += 3 {
		ba0 := bitarray.MustParse(tdt[i]).ZOptimize()
		ba1 := bitarray.MustParse(tdt[i+1]).ZOptimize()
		exp := bitarray.MustParse(tdt[i+2])
		ba0E := ba0.ZExpand()
		ba1E := ba1.ZExpand()
		chk(ba0.TrimSuffix(ba1), exp)
		chk(ba0.TrimSuffix(ba1E), exp)
		chk(ba0E.TrimSuffix(ba1), exp)
		chk(ba0E.TrimSuffix(ba1E), exp)
	}
}

func TestBitArray_TrimLeadingZeros(t *testing.T) {
	tdt := []string{
		"", "",
		"0", "",
		"1", "1",
		"1100", "1100",
		"0011", "11",
		"0000-0000", "",
		"0000-1010", "1010",
		"0000-0000 0000", "",
		"0000-0000 1010", "1010",
		"0000-0000 0000-1010", "1010",
		"0000-0000 0000-0001 010", "1010",
		"0000-0000 0000-0000 0010-10", "1010",
		"0000-0000 0000-0000 0000-0101 0", "1010",
		"0x_0000_0000 0x_0000_0000 0x_0000 0b_0011_11", "1111",
	}
	chk := func(got, want *bitarray.BitArray) {
		t.Helper()
		got.V()
		if !got.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
			t.FailNow()
		}
	}
	for i := 0; i < len(tdt); i += 2 {
		ba0 := bitarray.MustParse(tdt[i]).ZOptimize()
		exp := bitarray.MustParse(tdt[i+1])
		ba0E := ba0.ZExpand()
		chk(ba0.TrimLeadingZeros(), exp)
		chk(ba0E.TrimLeadingZeros(), exp)
	}
}

func TestBitArray_TrimTrailingZeros(t *testing.T) {
	tdt := []string{
		"", "",
		"0", "",
		"1", "1",
		"1100", "11",
		"0011", "0011",
		"0000-0000", "",
		"0000-1010", "0000-101",
		"0000-0101", "0000-0101",
		"1010-0000", "101",
		"0000-0000 0000", "",
		"0101-0000 0000", "0101",
		"0000-0000 0101", "0000-0000 0101",
		"0101-0000 0000-0000", "0101",
		"0101-0000 0000-0000 000", "0101",
		"0101-0000 0000-0000 0000-00", "0101",
		"0101-0000 0000-0000 0000-0000 0", "0101",
		"0101-0000 0000-0000 0000-0001 0", "0101-0000 0000-0000 0000-0001",
		"1111 0x_0000_0000 0x_0000_0000 0x_0000 0", "1111",
	}
	chk := func(got, want *bitarray.BitArray) {
		t.Helper()
		got.V()
		if !got.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
			t.FailNow()
		}
	}
	for i := 0; i < len(tdt); i += 2 {
		ba0 := bitarray.MustParse(tdt[i]).ZOptimize()
		exp := bitarray.MustParse(tdt[i+1])
		ba0E := ba0.ZExpand()
		chk(ba0.TrimTrailingZeros(), exp)
		chk(ba0E.TrimTrailingZeros(), exp)
	}
}
