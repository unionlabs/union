// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"
	"math/rand"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/tunabay/go-bitarray"
)

func TestBitArray_Equal(t *testing.T) {
	const nillit = "<nil>"
	fail := func(x, y *bitarray.BitArray) {
		t.Helper()
		if t.Failed() {
			t.Logf("x: [%# b]", x)
			t.Logf("y: [%# b]", y)
			t.Logf("x: %s", x.D())
			t.Logf("y: %s", y.D())
			t.FailNow()
		}
	}
	test := func(x, y *bitarray.BitArray, exp bool) {
		t.Helper()
		if x.Equal(y) != exp {
			t.Errorf("unexpected x.Equal(y): got %t, want %t", !exp, exp)
		}
		fail(x, y)
		if x == nil || y == nil {
			return
		}
		xe, ye := x.ZExpand(), y.ZExpand()
		xo, yo := x.ZOptimize(), y.ZOptimize()
		switch {
		case xe.Equal(ye) != exp:
			t.Errorf("unexpected xe.Equal(ye): got %t, want %t", !exp, exp)
		case xe.Equal(yo) != exp:
			t.Errorf("unexpected xe.Equal(yo): got %t, want %t", !exp, exp)
		case xo.Equal(yo) != exp:
			t.Errorf("unexpected xo.Equal(yo): got %t, want %t", !exp, exp)
		}
		fail(x, y)
	}
	ptwo := func(xs, ys string) (x, y *bitarray.BitArray) {
		t.Helper()
		if xs != nillit {
			x = bitarray.MustParse(xs)
		}
		if ys != nillit {
			y = bitarray.MustParse(ys)
		}
		return
	}
	equal := func(xs, ys string) {
		t.Helper()
		x, y := ptwo(xs, ys)
		test(x, y, true)
		test(y, x, true)
	}
	differ := func(xs, ys string) {
		t.Helper()
		x, y := ptwo(xs, ys)
		test(x, y, false)
		test(y, x, false)
	}

	equal(nillit, nillit)
	equal(nillit, "")
	differ(nillit, "0")
	equal("0", "0")
	equal("00", "00")
	equal("000", "000")
	equal("0000", "0000")
	equal("0000-0", "0000-0")
	equal("0000-00", "0000-00")
	equal("0000-000", "0000-000")
	equal("0000-0000", "0000-0000")
	equal("0000-0000 0", "0000-0000 0")
	equal("0000-0000 0000-0000", "0000-0000 0000-0000")
	equal("0000-0000 0000-0000 0000-000", "0000-0000 0000-0000 0000-000")
	differ("0000-0000 0000-0000 0000-000", "0000-0000 0000-0000 0000-0000")
	differ("0000-0000 0000-0000 0000-0000", "0000-0000 0000-0000 0000-0000 0")
	differ("0", "00")
	differ("0", "000")
	differ("0", "0000")
	differ("0", "0000-0")
	differ("0", "0000-00")
	differ("0", "0000-000")
	differ("0", "0000-0000")
	differ("0", "0000-0000 0")
	differ("0", "0000-0000 00")
	differ("0000-000", "0000-0000")
	differ("0000-0000", "0000-0000 0")
	differ("0000-001", "0000-0010")
	differ("0000-0000 10", "0000-0000 100")
	equal("1", "1")
	differ("1", "0")
	differ("1", "10")
	differ("1", "100")
	differ("1", "1000")
	differ("1", "1000-0000")
	differ("1", "0000-0001")
	differ("1", "0000-0000 1")
	differ("1", "1000-0000 00")
	equal("0000-1111 1111-0000 0000-1111", "0000-1111 1111-0000 0000-1111")
	differ("0000-1111 1111-0000 0000-1111", "0000-1111 1111-0000 0000-1111 0")
	differ("0000-1111 1111-0000 0000-1111 1", "0000-1111 1111-0000 0000-1111 0")
	differ("0000-1111 1111-0000 0000-1111 1", "0000-1111 1011-0000 0000-1111 1")
	differ("0000-1110 1111-0000 0000-1111 1", "0000-1111 1111-0000 0000-1111 1")
}

func TestBitArray_Compare(t *testing.T) {
	src := []string{
		"",
		"0",
		"00",
		"000",
		"0000",
		"0000-0",
		"0000-00",
		"0000-000",
		"0000-0000",
		"0000-0000 0",
		"0000-0000 00",
		"0000-0000 000",
		"0000-0000 0000",
		"0000-0000 0000-0",
		"0000-0000 0000-00",
		"0000-0000 0000-000",
		"0000-0000 0000-0000",
		"0000-0000 0000-0000 0",
		"0000-0000 0000-0000 00",
		"0000-0001",
		"0000-0001 00",
		"0000-0001 0000-0000 0000-0000",
		"0000-0001 0000-0000 0000-0000 0",
		"0000-0001 0000-0000 0000-0000 1",
		"0000-0001 0000-0000 0000-0001",
		"0000-0001 0000-0000 0000-0001 0",
		"0000-0001 0000-0000 0000-0001 1",
		"0000-0001 0000-0001",
		"0000-0001 01",
		"0000-0001 1",
		"0000-0001 11",
		"0000-0010",
		"0000-0100",
		"0000-1000",
		"0000-1100 0000-0000 0000-0000 0000-0000 0000-000",
		"0000-1100 0000-0000 0000-0000 0000-0000 0000-0000",
		"0000-1100 0000-0000 0000-0000 0000-0000 0000-0000 0",
		"0000-1100 0000-0000 0000-0000 0000-0000 0000-0000 00",
		"0000-1100 0000-0000 0000-0000 0000-0000 0000-0000 0000",
		"0000-1100 0000-0000 0000-0000 0000-0000 0000-0000 0011",
		"0000-1100 0000-0000 0000-0000 0000-0000 0000-0000 01",
		"0000-1100 0000-0000 0000-0000 0100-0000 0000-000",
		"0000-1100 0000-0000 0000-0000 1000-0000 0000-00",
		"0000-1100 0000-0000 0000-0000 1000-0000 0000-000",
		"0000-1101 1111-1110 0000-0000",
		"0000-1101 1111-1110 0000-0000 0",
		"0000-1101 1111-1110 0000-0001",
		"0000-1101 1111-1110 0000-001",
		"0000-1101 1111-1110 0000-01",
		"0000-1101 1111-1110 0000-1",
		"0000-1101 1111-1110 0000-10",
		"0000-1101 1111-1110 0000-100",
		"0000-1101 1111-1110 0000-1000",
		"0000-1101 1111-1110 0000-11",
		"0000-1101 1111-1110 0001",
		"0000-1101 1111-1111 0",
		"0000-1101 1111-1111 00",
		"0000-1101 1111-1111 000",
		"0000-1101 1111-1111 1",
		"0000-1101 1111-1111 11",
		"0000-1101 1111-1111 111",
		"0000-1111",
		"0000-1111 0",
		"0000-1111 00",
		"0000-1111 1",
		"0000-1111 10",
		"0000-1111 11",
		"0000-1111 1100",
		"0000-1111 1100-1100",
		"0000-1111 1100-1100 0",
		"0000-1111 1100-1100 00",
		"0000-1111 1100-1100 01",
		"0000-1111 1100-1100 1",
		"0000-1111 1100-1100 10",
		"0000-1111 1100-1100 11",
		"01",
		"010",
		"0101",
		"0101-0",
		"0101-01",
		"0101-010",
		"0101-0101",
		"0101-0101 0",
		"0101-0101 01",
		"0111",
		"0111-0",
		"0111-00",
		"0111-01",
		"0111-1",
		"0111-10",
		"0111-101",
		"0111-11",
		"0111-1100",
		"0111-1100 0",
		"0111-1101",
		"1",
		"10",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-00",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0000",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0000 00",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0000 000",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0000 1",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0000 10",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0000 100",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0000 11",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-0001",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-001",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-01",
		"1000-0000 0000-0000 0000-0000 0000-0000 0000-0000 0000-1",
		"11",
		"111",
		"1111",
		"1111-1",
		"1111-11",
		"1111-111",
		"1111-1111",
		"1111-1111 1",
		"1111-1111 11",
		"1111-1111 111",
		"1111-1111 1111",
		"1111-1111 1111-1",
		"1111-1111 1111-11",
		"1111-1111 1111-110",
		"1111-1111 1111-111",
		"1111-1111 1111-1111",
		"1111-1111 1111-1111 1",
		"1111-1111 1111-1111 11",
	}
	dat := make([]*bitarray.BitArray, len(src))
	for i, s := range src {
		if i+1 < len(src) {
			next := src[i+1]
			if next <= s {
				t.Errorf("misordered data samples:")
				t.Logf("#%3d: %q", i, s)
				t.Logf("#%3d: %q", i+1, next)
				t.FailNow()
			}
		}
		dat[i] = bitarray.MustParse(s)
		// t.Logf("data: %s", dat[i].D())
	}
	for i := 0; i < len(dat); i++ {
		x := dat[i]
		xe := x.ZExpand()
		xo := x.ZOptimize()
		for j := 0; j < len(dat); j++ {
			y := dat[j]
			ye := y.ZExpand()
			yo := y.ZOptimize()
			c := bitarray.Compare(x, y)
			switch {
			case i == j && c != 0:
				t.Errorf("unexpected result: got %d, want 0, x == y:", c)
			case i < j && c != -1:
				t.Errorf("unexpected result: got %d, want -1, x < y:", c)
			case j < i && c != +1:
				t.Errorf("unexpected result: got %d, want +1, y < x:", c)
			}
			if ca := bitarray.Compare(xe, ye); ca != c {
				t.Errorf("unexpected result (e,e): got %d, want %d", ca, c)
			}
			if ca := bitarray.Compare(xe, yo); ca != c {
				t.Errorf("unexpected result (e,o): got %d, want %d", ca, c)
			}
			if ca := bitarray.Compare(xo, yo); ca != c {
				t.Errorf("unexpected result (o,o): got %d, want %d", ca, c)
			}
			if t.Failed() {
				t.Logf("x: [%# b]", x)
				t.Logf("y: [%# b]", y)
				t.Logf("x: %s", x.D())
				t.Logf("y: %s", y.D())
				t.FailNow()
			}
			// t.Logf("passed: [%# b] %c [%# b]", x, "<=>"[c+1], y)
		}
	}
}

func TestBitArray_Compare_rand(t *testing.T) {
	const testIterations = 256
	const listSize = 256
	rand.Seed(time.Now().UnixNano())
	mkdat := func() []*bitarray.BitArray {
		t.Helper()
		src := make([]string, listSize)
		for i := range src {
			n := rand.Intn(256)
			if n == 0 || rand.Intn(64) == 0 {
				continue
			}
			b := make([]byte, (n+7)>>3)
			rand.Read(b)
			s := fmt.Sprintf("%08b", b)
			s = strings.Trim(s, "[]")
			s = strings.ReplaceAll(s, " ", "")
			src[i] = s[:n]
		}
		sort.Strings(src)
		bas := make([]*bitarray.BitArray, 0, listSize)
		for i, s := range src {
			if 0 < i && s == src[i-1] {
				continue
			}
			ba := bitarray.MustParse(s)
			bas = append(bas, ba)
			// t.Logf("dat: %# b", ba)
		}
		return bas
	}
	for i := 0; i < testIterations; i++ {
		dat := mkdat()
		for xi, x := range dat {
			for yi, y := range dat {
				c := bitarray.Compare(x, y)
				switch {
				case xi == yi && c != 0:
					t.Errorf("unexpected result: got %d, want 0, x == y:", c)
				case xi < yi && c != -1:
					t.Errorf("unexpected result: got %d, want -1, x < y:", c)
				case yi < xi && c != +1:
					t.Errorf("unexpected result: got %d, want +1, y < x:", c)
				}
				if t.Failed() {
					t.Logf("x: [%# b]", x)
					t.Logf("y: [%# b]", y)
					t.Logf("x: %s", x.D())
					t.Logf("y: %s", y.D())
					t.FailNow()
				}
				// t.Logf("passed: [%# b] %c [%# b]", x, "<=>"[c+1], y)
			}
		}
	}
}
