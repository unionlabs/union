// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"strings"
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestNew(t *testing.T) {
	tcs := []struct {
		b []byte
		s string
	}{
		{[]byte{}, ""},
		{[]byte{0}, "0"},
		{[]byte{1}, "1"},
		{[]byte{0, 0}, "00"},
		{[]byte{0, 0, 0}, "000"},
		{[]byte{0, 0, 0, 0}, "0000"},
		{[]byte{0, 0, 0, 0, 0}, "00000"},
		{[]byte{0, 0, 0, 0, 0, 0}, "000000"},
		{[]byte{0, 0, 0, 0, 0, 0, 0}, "0000000"},
		{[]byte{0, 0, 0, 0, 0, 0, 0, 0}, "00000000"},
		{[]byte{0, 0, 0, 0, 0, 0, 0, 0, 0}, "000000000"},
		{[]byte{1, 1}, "11"},
		{[]byte{1, 1, 0, 0, 0}, "11000"},
		{[]byte{1, 1, 0, 0, 0, 0, 0, 0, 0}, "110000000"},
	}
	for _, tc := range tcs {
		ba := bitarray.New(tc.b...)
		ba.V()
		s := ba.String()
		if s != tc.s {
			t.Errorf("unexpected result: got %q, want %q", s, tc.s)
			t.Logf("input: %v", tc.b)
			t.Logf(" data: %s", ba.D())
			t.FailNow()
		}
	}
}

func TestNewFromBytes(t *testing.T) {
	b := bytes.Repeat([]byte{0b_1111_0011, 0b_1010_0101}, 3)
	tcs := []struct {
		n, o int
		s    string
	}{
		{0, 0, ""},
		{0, 1, ""},
		{0, 7, ""},
		{0, 8, ""},
		{0, 9, ""},
		{0, 47, ""},
		{0, 48, ""},
		{1, 0, "1"},
		{2, 0, "11"},
		{3, 0, "111"},
		{4, 0, "1111"},
		{5, 0, "11110"},
		{6, 0, "111100"},
		{7, 0, "1111001"},
		{8, 0, "11110011"},
		{9, 0, "111100111"},
		{10, 0, "1111001110"},
		{16, 0, "1111001110100101"},
		{17, 0, "11110011101001011"},
		{24, 0, "111100111010010111110011"},
		{25, 0, "1111001110100101111100111"},
		{24, 1, "111001110100101111100111"},
		{23, 2, "11001110100101111100111"},
		{22, 3, "1001110100101111100111"},
		{21, 4, "001110100101111100111"},
		{20, 5, "01110100101111100111"},
		{19, 6, "1110100101111100111"},
		{18, 7, "110100101111100111"},
		{17, 8, "10100101111100111"},
		{16, 9, "0100101111100111"},
		{10, 15, "1111100111"},
		{9, 16, "111100111"},
		{8, 17, "11100111"},
		{7, 18, "1100111"},
		{1, 47, "1"},
		{2, 46, "01"},
		{3, 45, "101"},
		{4, 44, "0101"},
		{5, 43, "00101"},
		{6, 42, "100101"},
		{7, 41, "0100101"},
		{8, 40, "10100101"},
		{9, 39, "110100101"},
		{48, 0, "111100111010010111110011101001011111001110100101"},
	}
	for _, tc := range tcs {
		ba := bitarray.NewFromBytes(b, tc.o, tc.n)
		ba.V()
		bs := ba.String()
		if bs != tc.s {
			t.Errorf("unexpected result: nBits=%d, off=%d", tc.n, tc.o)
			t.Logf(" src: %b", b)
			t.Logf(" got: %q", bs)
			t.Logf("want: %q", tc.s)
			t.Logf("  ba: %s", ba.D())
			t.FailNow()
		}
	}
}

func TestNewFromBytes_panic(t *testing.T) {
	var ba *bitarray.BitArray
	testNoPanicZero := func(b []byte, nBits, off int) {
		t.Helper()
		ba = bitarray.NewFromBytes(b, off, nBits)
		if !ba.IsZero() {
			t.Errorf("zero expected: nBits=%d, off=%d", nBits, off)
			t.Logf("input: b=%v", b)
			t.Logf("  got: %#b", ba)
			t.Logf("  got: %s", ba.D())
		}
	}
	testPanic := func(b []byte, nBits, off int) {
		t.Helper()
		defer func() {
			t.Helper()
			if recover() == nil {
				t.Errorf("panic expected: nBits=%d, off=%d", nBits, off)
				t.Logf("input: b=%v", b)
				t.Logf("  got: %#b", ba)
				t.Logf("  got: %s", ba.D())
			}
		}()
		ba = bitarray.NewFromBytes(b, off, nBits)
	}
	testPanic(nil, -1, 0)
	testPanic(nil, 0, -1)
	testPanic([]byte{}, -1, 0)
	testPanic([]byte{}, 0, -1)
	testNoPanicZero(nil, 0, 0)
	testNoPanicZero([]byte{}, 0, 0)
	testPanic(nil, 1, 0)
	testPanic(nil, 0, 1)
	testPanic([]byte{}, 1, 0)
	testPanic([]byte{}, 0, 1)
	testPanic([]byte{}, 7, 0)
	testPanic([]byte{}, 8, 0)
	testPanic([]byte{}, 9, 0)
	testPanic([]byte{}, 0, 7)
	testPanic([]byte{}, 0, 8)
	testPanic([]byte{}, 0, 9)

	testNoPanicZero([]byte{0}, 0, 0)
	testNoPanicZero([]byte{0}, 0, 7)
	testNoPanicZero([]byte{0}, 0, 8)
	testPanic([]byte{0}, 0, 9)
	testPanic([]byte{0}, 9, 0)
	testPanic([]byte{0}, 1, 8)
	testPanic([]byte{0}, 4, 5)
	testPanic([]byte{0}, 7, 2)
	testPanic([]byte{0}, 32, 64)
	testNoPanicZero([]byte{0, 0, 0, 0}, 0, 32)
	testPanic([]byte{0, 0, 0, 0}, 16, 17)
	testPanic([]byte{0, 0, 0, 0}, 17, 16)
	testPanic([]byte{0, 0, 0, 0}, -1, 0)
	testPanic([]byte{0, 0, 0, 0}, 0, -1)
}

func TestNewFromByteBits(t *testing.T) {
	tcs := []struct {
		b []byte
		s string
	}{
		{nil, ""},
		{[]byte{}, ""},
		{[]byte{0}, "0"},
		{[]byte{0, 0}, "00"},
		{[]byte{0, 0, 0}, "000"},
		{[]byte{0, 0, 0, 0}, "0000"},
		{[]byte{0, 0, 0, 0, 0}, "0000-0"},
		{[]byte{0, 0, 0, 0, 0, 0}, "0000-00"},
		{[]byte{0, 0, 0, 0, 0, 0, 0}, "0000-000"},
		{[]byte{0, 0, 0, 0, 0, 0, 0, 0}, "0000-0000"},
		{[]byte{0, 0, 0, 0, 0, 0, 0, 0, 0}, "0000-0000 0"},
		{[]byte{0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1}, "0000-0000 1010-0000 1"},
		{[]byte{1}, "1"},
		{[]byte{1, 1}, "11"},
		{[]byte{1, 1, 1}, "111"},
		{[]byte{1, 1, 1, 1}, "1111"},
		{[]byte{1, 1, 1, 1, 1}, "1111-1"},
		{[]byte{1, 1, 1, 1, 1, 1}, "1111-11"},
		{[]byte{1, 1, 1, 1, 1, 1, 1}, "1111-111"},
		{[]byte{1, 1, 1, 1, 1, 1, 1, 1}, "1111-1111"},
		{[]byte{1, 1, 1, 1, 1, 1, 1, 1, 1}, "1111-1111 1"},
		{[]byte{1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 0}, "1111-1111 0111-0101 00"},
		{[]byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff}, "1111-1111 1"},
		{[]byte{0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe}, "0000-0000 0"},
		{[]byte{0xff, 0xfe, 0xff, 0xfe, 0xfe, 0xff, 0xfe, 0xff, 0xfe, 0xff}, "1010-0101 01"},
	}
	for _, tc := range tcs {
		ba0 := bitarray.NewFromByteBits(tc.b)
		want := bitarray.MustParse(tc.s)
		ba0.V()
		if !ba0.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", ba0)
			t.Logf(" got: %s", ba0.D())
			t.Logf("want: %#b", want)
		}
	}
}

func TestNewZeroFilled(t *testing.T) {
	testPanic := func(nBits int) {
		t.Helper()
		var ba *bitarray.BitArray
		defer func() {
			t.Helper()
			if recover() == nil {
				t.Errorf("panic expected.")
				t.Logf("got: %#b", ba)
				t.Logf("got: %s", ba.D())
			}
		}()
		ba = bitarray.NewZeroFilled(nBits)
	}
	testPanic(-1)
	testPanic(-7)

	ba0 := bitarray.NewZeroFilled(0)
	ba0.V()
	if !ba0.IsZero() || ba0.Len() != 0 || ba0.NumPadding() != 0 {
		t.Errorf("unexpected: got %#b, want zero", ba0)
	}

	tcs := []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 15, 16, 17, 63, 64, 65}
	for _, tc := range tcs {
		ba := bitarray.NewZeroFilled(tc)
		ba.V()
		baE := ba.ZExpand()
		baE.V()
		got := ba.String()
		gotE := baE.String()
		exp := strings.Repeat("0", tc)
		if got != exp || gotE != exp {
			t.Errorf("unexpected: got %q, want %q", got, exp)
		}
	}
}

func TestNewOneFilled(t *testing.T) {
	testPanic := func(nBits int) {
		t.Helper()
		var ba *bitarray.BitArray
		defer func() {
			t.Helper()
			if recover() == nil {
				t.Errorf("panic expected.")
				t.Logf("got: %#b", ba)
				t.Logf("got: %s", ba.D())
			}
		}()
		ba = bitarray.NewOneFilled(nBits)
	}
	testPanic(-1)
	testPanic(-7)

	ba0 := bitarray.NewOneFilled(0)
	ba0.V()
	if !ba0.IsZero() || ba0.Len() != 0 || ba0.NumPadding() != 0 {
		t.Errorf("unexpected: got %#b, want zero", ba0)
	}

	tcs := []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 15, 16, 17, 63, 64, 65}
	for _, tc := range tcs {
		ba := bitarray.NewOneFilled(tc)
		ba.V()
		got := ba.String()
		exp := strings.Repeat("1", tc)
		if got != exp {
			t.Errorf("unexpected: got %q, want %q", got, exp)
		}
	}
}

func TestNewByRunLength(t *testing.T) {
	ba0 := bitarray.NewByRunLength()
	if !ba0.IsZero() || ba0.Len() != 0 || ba0.NumPadding() != 0 {
		t.Errorf("unexpected: got %#b, want zero", ba0)
	}

	tcs := []*bitarray.BitArray{
		bitarray.NewByRunLength(),
		bitarray.MustParse(""),

		bitarray.NewByRunLength(0, 0, 0),
		bitarray.MustParse(""),

		bitarray.NewByRunLength(1),
		bitarray.MustParse("0"),

		bitarray.NewByRunLength(0, 1),
		bitarray.MustParse("1"),

		bitarray.NewByRunLength(100, 0, 100),
		bitarray.MustParse(strings.Repeat("0", 200)),
	}
	for i := 0; i < len(tcs); i += 2 {
		x := tcs[i]
		xE := x.ZExpand()
		xO := x.ZOptimize()
		x.V()
		xE.V()
		xO.V()
		y := tcs[i+1]
		switch {
		case !x.Equal(y):
			t.Error("unexpected:")
			t.Logf(" got: %#b", x)
			t.Logf(" got: %s", x.D())
		case !xE.Equal(y):
			t.Error("unexpected (e):")
			t.Logf(" got: %#b", xE)
			t.Logf(" got: %s", xE.D())
		case !xO.Equal(y):
			t.Error("unexpected (o):")
			t.Logf(" got: %#b", xO)
			t.Logf(" got: %s", xO.D())
		}
		if t.Failed() {
			t.Logf("want: %#b", y)
			t.FailNow()
		}
	}
}

func TestBitArray_IsZero(t *testing.T) {
	var ba *bitarray.BitArray
	if !ba.IsZero() {
		t.Error("unexpected nil.IsZero(): got false, want true")
	}
	ba = &bitarray.BitArray{}
	if !ba.IsZero() {
		t.Error("unexpected zero.IsZero(): got false, want true")
	}
	ba = bitarray.NewZeroFilled(0)
	if !ba.IsZero() {
		t.Error("unexpected zero.IsZero(): got false, want true")
	}
	ba = bitarray.NewZeroFilled(1)
	if ba.IsZero() {
		t.Error("unexpected 0.IsZero(): got true, want false")
	}
}

func TestBitArray_Len_edge(t *testing.T) {
	var ba *bitarray.BitArray
	if n := ba.Len(); n != 0 {
		t.Errorf("unexpected nil.Len(): got %d, want 0", n)
	}
	ba = &bitarray.BitArray{}
	if n := ba.Len(); n != 0 {
		t.Errorf("unexpected zero.Len(): got %d, want 0", n)
	}
	ba = bitarray.NewZeroFilled(0)
	if n := ba.Len(); n != 0 {
		t.Errorf("unexpected zero.Len(): got %d, want 0", n)
	}
	ba = bitarray.NewZeroFilled(1000)
	if n := ba.Len(); n != 1000 {
		t.Errorf("unexpected Len(): got %d, want 1000", n)
	}
	if n := ba.ZExpand().Len(); n != 1000 {
		t.Errorf("unexpected Len(): got %d, want 1000", n)
	}
	ba = bitarray.NewOneFilled(1000)
	if n := ba.Len(); n != 1000 {
		t.Errorf("unexpected Len(): got %d, want 1000", n)
	}
}

func TestBitArray_NumPadding_edge(t *testing.T) {
	var ba *bitarray.BitArray
	if n := ba.NumPadding(); n != 0 {
		t.Errorf("unexpected nil.NumPadding(): got %d, want 0", n)
	}
	ba = &bitarray.BitArray{}
	if n := ba.NumPadding(); n != 0 {
		t.Errorf("unexpected zero.NumPadding(): got %d, want 0", n)
	}
	ba = bitarray.NewZeroFilled(0)
	if n := ba.NumPadding(); n != 0 {
		t.Errorf("unexpected zero.NumPadding(): got %d, want 0", n)
	}
	ba = bitarray.NewZeroFilled(1001)
	if n := ba.NumPadding(); n != 7 {
		t.Errorf("unexpected NumPadding(): got %d, want 7", n)
	}
	if n := ba.ZExpand().NumPadding(); n != 7 {
		t.Errorf("unexpected NumPadding(): got %d, want 7", n)
	}
	ba = bitarray.NewOneFilled(1001)
	if n := ba.NumPadding(); n != 7 {
		t.Errorf("unexpected NumPadding(): got %d, want 7", n)
	}
}

func TestBitArray_String_edge(t *testing.T) {
	ba := &bitarray.BitArray{}
	if s := ba.String(); s != "" {
		t.Errorf(`unexpected zero.String(): got %q, want ""`, s)
	}
	ba = bitarray.NewZeroFilled(0)
	if s := ba.String(); s != "" {
		t.Errorf(`unexpected zero.String(): got %q, want ""`, s)
	}
	ba = bitarray.NewZeroFilled(1000)
	z1000 := strings.Repeat("0", 1000)
	if s := ba.String(); s != z1000 {
		t.Errorf(`unexpected String(): got %q, want %q`, s, z1000)
	}
	if s := ba.ZExpand().String(); s != z1000 {
		t.Errorf(`unexpected String(): got %q, want %q`, s, z1000)
	}
	ba = bitarray.NewOneFilled(1000)
	o1000 := strings.Repeat("1", 1000)
	if s := ba.String(); s != o1000 {
		t.Errorf(`unexpected String(): got %q, want %q`, s, o1000)
	}
}

func TestBitArray_Bytes(t *testing.T) {
	tcs := []struct {
		s, b string
		pad  int
	}{
		{"", "", 0},
		{"0", "00", 7},
		{"00", "00", 6},
		{"000", "00", 5},
		{"0000", "00", 4},
		{"0000-0", "00", 3},
		{"0000-00", "00", 2},
		{"0000-000", "00", 1},
		{"0000-0000", "00", 0},
		{"0000-0000 0", "0000", 7},
		{"0000-0000 00", "0000", 6},
		{"1", "80", 7},
		{"10", "80", 6},
		// TODO: more
	}
	for _, tc := range tcs {
		ba := bitarray.MustParse(tc.s).ZOptimize()
		baE := ba.ZExpand()
		want, _ := hex.DecodeString(tc.b)
		if b, pad := ba.Bytes(); !bytes.Equal(b, want) || pad != tc.pad {
			t.Error("unexpected result:")
			t.Logf(" got: %x, pad=%d", b, pad)
			t.Logf("want: %s, pad=%d", tc.b, tc.pad)
			t.Logf("  ba: %#b", ba)
			t.Logf("  ba: %s", ba.D())
			t.FailNow()
		}
		if b, pad := baE.Bytes(); !bytes.Equal(b, want) || pad != tc.pad {
			t.Error("unexpected result (e):")
			t.Logf(" got: %x, pad=%d", b, pad)
			t.Logf("want: %s, pad=%d", tc.b, tc.pad)
			t.Logf("  ba: %#b", baE)
			t.Logf("  ba: %s", baE.D())
			t.FailNow()
		}
	}
}

func TestBitArray_BitArray(t *testing.T) {
	ba := bitarray.MustParse("0001-0000 1111-1111")
	if cp := ba.BitArray(); cp != ba {
		t.Errorf("unexpected result: %p != %p", cp, ba)
	}
}

func TestBitArray_BitAt(t *testing.T) {
	mustPanic := func(ba *bitarray.BitArray, off int) {
		t.Helper()
		var b byte
		defer func() {
			t.Helper()
			if recover() == nil {
				t.Errorf("expected panic: got %d", b)
			}
		}()
		b = ba.BitAt(off)
	}

	var ba *bitarray.BitArray
	mustPanic(ba, 0)
	mustPanic(ba, -1)
	mustPanic(ba, 1)

	ba = &bitarray.BitArray{}
	mustPanic(ba, 0)
	mustPanic(ba, -1)
	mustPanic(ba, 1)

	ba = bitarray.NewZeroFilled(1001)
	mustPanic(ba, -1)
	mustPanic(ba, 2000)
	if b := ba.BitAt(123); b != 0 {
		t.Errorf("unexpected: got %d, want 0", b)
	}
	if b := ba.ZExpand().BitAt(456); b != 0 {
		t.Errorf("unexpected: got %d, want 0", b)
	}

	ba = bitarray.NewOneFilled(1001)
	mustPanic(ba, -1)
	mustPanic(ba, 2000)
	if b := ba.BitAt(234); b != 1 {
		t.Errorf("unexpected: got %d, want 1", b)
	}
	if b := ba.ZExpand().BitAt(567); b != 1 {
		t.Errorf("unexpected: got %d, want 1", b)
	}
}

func TestBitArray_Hash(t *testing.T) {
	m := make(map[string]struct{})

	add := func(ba *bitarray.BitArray) {
		ba = ba.ZOptimize()
		baE := ba.ZExpand()
		h := ba.Hash(sha256.New())
		hE := baE.Hash(sha256.New())
		m[hex.EncodeToString(h)] = struct{}{}
		m[hex.EncodeToString(hE)] = struct{}{}
	}

	for i := 0; i < 200; i++ { // 200
		add(bitarray.NewZeroFilled(i))
	}
	for i := 0; i < 200; i++ { // 199, #0 is duplicated
		add(bitarray.NewOneFilled(i))
	}

	var ba *bitarray.BitArray
	add(ba) // duplicated

	// 9
	add(bitarray.MustParse("1")) // duplicated
	add(bitarray.MustParse("10"))
	add(bitarray.MustParse("100"))
	add(bitarray.MustParse("1000"))
	add(bitarray.MustParse("1000-0"))
	add(bitarray.MustParse("1000-00"))
	add(bitarray.MustParse("1000-000"))
	add(bitarray.MustParse("1000-0000"))
	add(bitarray.MustParse("1000-0000 0"))
	add(bitarray.MustParse("1000-0000 00"))

	// 9
	add(bitarray.MustParse("0")) // duplicated
	add(bitarray.MustParse("01"))
	add(bitarray.MustParse("001"))
	add(bitarray.MustParse("0001"))
	add(bitarray.MustParse("0000-1"))
	add(bitarray.MustParse("0000-01"))
	add(bitarray.MustParse("0000-001"))
	add(bitarray.MustParse("0000-0001"))
	add(bitarray.MustParse("0000-0000 1"))
	add(bitarray.MustParse("0000-0000 01"))

	want := 200 + 199 + 9 + 9
	if len(m) != want {
		t.Errorf("unexpected number of keys: got %d, want %d", len(m), want)
	}
}

func TestBitArray_MapKey(t *testing.T) {
	m := make(map[string]struct{})

	add := func(ba *bitarray.BitArray) {
		m[ba.ZOptimize().MapKey()] = struct{}{}
		m[ba.ZExpand().MapKey()] = struct{}{}
	}

	for i := 0; i < 200; i++ { // 200
		add(bitarray.NewZeroFilled(i))
	}
	for i := 0; i < 200; i++ { // 199, #0 is duplicated
		add(bitarray.NewOneFilled(i))
	}

	var ba *bitarray.BitArray
	add(ba) // duplicated

	// 9
	add(bitarray.MustParse("1")) // duplicated
	add(bitarray.MustParse("10"))
	add(bitarray.MustParse("100"))
	add(bitarray.MustParse("1000"))
	add(bitarray.MustParse("1000-0"))
	add(bitarray.MustParse("1000-00"))
	add(bitarray.MustParse("1000-000"))
	add(bitarray.MustParse("1000-0000"))
	add(bitarray.MustParse("1000-0000 0"))
	add(bitarray.MustParse("1000-0000 00"))

	// 9
	add(bitarray.MustParse("0")) // duplicated
	add(bitarray.MustParse("01"))
	add(bitarray.MustParse("001"))
	add(bitarray.MustParse("0001"))
	add(bitarray.MustParse("0000-1"))
	add(bitarray.MustParse("0000-01"))
	add(bitarray.MustParse("0000-001"))
	add(bitarray.MustParse("0000-0001"))
	add(bitarray.MustParse("0000-0000 1"))
	add(bitarray.MustParse("0000-0000 01"))

	want := 200 + 199 + 9 + 9
	if len(m) != want {
		t.Errorf("unexpected number of keys: got %d, want %d", len(m), want)
	}
}

func TestBitArray_ToPadded8(t *testing.T) {
	tdt := []string{
		"", "",
		"0", "0000-0000",
		"00", "0000-0000",
		"000", "0000-0000",
		"0000", "0000-0000",
		"0000-0", "0000-0000",
		"0000-00", "0000-0000",
		"0000-000", "0000-0000",
		"0000-0000", "0000-0000",
		"0000-0000 0", "0000-0000 0000-0000",
		"0000-0000 00", "0000-0000 0000-0000",
		"0000-0000 1010", "0000-0000 1010-0000",
		"1", "1000-0000",
		"11", "1100-0000",
		"111", "1110-0000",
		"1111", "1111-0000",
		"1111-1", "1111-1000",
		"1111-11", "1111-1100",
		"1111-111", "1111-1110",
		"1111-1111", "1111-1111",
		"1111-1111 1", "1111-1111 1000-0000",
		"1111-1111 0", "1111-1111 0000-0000",
		"0xffffffff 0", "0xffffffff 0000-0000",
	}
	chk := func(got, want *bitarray.BitArray) {
		t.Helper()
		got.V()
		if !got.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
		}
	}
	for i := 0; i < len(tdt); i += 2 {
		ba0 := bitarray.MustParse(tdt[i]).ZOptimize()
		want := bitarray.MustParse(tdt[i+1])
		got := ba0.ToPadded8()
		gotE := ba0.ZExpand().ToPadded8()
		chk(got, want)
		chk(gotE, want)
	}
}

func TestBitArray_ToPadded64(t *testing.T) {
	tdt := []string{
		"", "",
		"0", "0x_0000_0000_0000_0000",
		"1", "0x_8000_0000_0000_0000",
		"0x_0000_0000_0000_0000 0x_0000_0000_0000_000", "0x_0000_0000_0000_0000 0x_0000_0000_0000_0000",
		"0x_0000_0000_0000_0000 0x_0000_0000_0000_0000", "0x_0000_0000_0000_0000 0x_0000_0000_0000_0000",
		"0x_0000_dead_beef_cafe 0x_0000_8888_0000_000", "0x_0000_dead_beef_cafe 0x_0000_8888_0000_0000",
		"0x_0000_dead_beef_cafe 0x_0000_8888_0000_0000", "0x_0000_dead_beef_cafe 0x_0000_8888_0000_0000",
		"0x_f0f0_f0f0_f0f0_f0f0 100", "0x_f0f0_f0f0_f0f0_f0f0 0x_8000_0000_0000_0000",
	}
	chk := func(got, want *bitarray.BitArray) {
		t.Helper()
		got.V()
		if !got.Equal(want) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
		}
	}
	for i := 0; i < len(tdt); i += 2 {
		ba0 := bitarray.MustParse(tdt[i]).ZOptimize()
		want := bitarray.MustParse(tdt[i+1])
		got := ba0.ToPadded64()
		gotE := ba0.ZExpand().ToPadded64()
		chk(got, want)
		chk(gotE, want)
	}
}

func TestBitArray_ToByteBits(t *testing.T) {
	tcs := []struct {
		s    string
		want []byte
	}{
		{"", []byte{}},
		{"0", []byte{0}},
		{"1", []byte{1}},
		{"0000-000", []byte{0, 0, 0, 0, 0, 0, 0}},
		{"0000-0000", []byte{0, 0, 0, 0, 0, 0, 0, 0}},
		{"0000-0000 0", []byte{0, 0, 0, 0, 0, 0, 0, 0, 0}},
		{"1111-111", []byte{1, 1, 1, 1, 1, 1, 1}},
		{"1111-1111", []byte{1, 1, 1, 1, 1, 1, 1, 1}},
		{"1111-1111 1", []byte{1, 1, 1, 1, 1, 1, 1, 1, 1}},
		{"1010-0101 1111-0011 101", []byte{1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1}},
	}
	chk := func(got, want []byte) {
		if !bytes.Equal(got, want) {
			t.Error("unexpected result:")
			t.Logf(" got: %d", got)
			t.Logf("want: %d", want)
		}
	}
	for _, tc := range tcs {
		ba0 := bitarray.MustParse(tc.s).ZOptimize()
		got := ba0.ToByteBits()
		gotE := ba0.ZExpand().ToByteBits()
		chk(got, tc.want)
		chk(gotE, tc.want)
	}
}

func TestBitArray_RepeatEach(t *testing.T) {
	tcs := []struct {
		n        int
		src, dst string
	}{
		{0, "", ""},
		{1, "", ""},
		{0, "0000", ""},
		{0, "1111", ""},
		{1, "0000", "0000"},
		{1, "0000-0000", "0000-0000"},
		{1, "1111-1111", "1111-1111"},
		{2, "0000-0000", "0000-0000 0000-0000"},
		{2, "0000-0000 00", "0000-0000 0000-0000 0000"},
		{3, "0000-0000 00", "0000-0000 0000-0000 0000-0000 0000-00"},
		{2, "1111-1111", "1111-1111 1111-1111"},
		{2, "1111-1111 111", "1111-1111 1111-1111 1111-11"},
		{2, "1010-0101", "1100-1100 0011-0011"},
		{2, "1010-0101 11", "1100-1100 0011-0011 1111"},
		{4, "1010-1100 01", "1111-0000 1111-0000 1111-1111 0000-0000 0000-1111"},
	}
	chk := func(n int, src, want *bitarray.BitArray) {
		t.Helper()
		got := src.RepeatEach(n)
		got.V()
		if !got.Equal(want) {
			t.Errorf("unexpected: n=%d", n)
			t.Logf(" src: %#b", src)
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
			t.FailNow()
		}
	}
	for _, tc := range tcs {
		ba0 := bitarray.MustParse(tc.src).ZOptimize()
		ba0E := ba0.ZExpand()
		exp := bitarray.MustParse(tc.dst)
		chk(tc.n, ba0, exp)
		chk(tc.n, ba0E, exp)
	}
	func() {
		var ba *bitarray.BitArray
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: got %#b", ba)
			}
		}()
		src := bitarray.MustParse("0101-0101 01")
		ba = src.RepeatEach(-1)
	}()
}
