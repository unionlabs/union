// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestBuffer_Slice(t *testing.T) {
	bs := ""
	set := func(s string) {
		t.Helper()
		bs = s
	}
	test := func(s, e int, exp string) {
		t.Helper()
		ba := bitarray.MustParse(bs)
		buf := bitarray.NewBufferFromBitArray(ba)
		bufSliced := buf.Slice(s, e)
		bufSliced.V()
		sliced := bufSliced.BitArray()
		expected := bitarray.MustParse(exp)
		if !sliced.Equal(expected) {
			t.Errorf("% b: [%d:%d]: unexpected slice:", ba, s, e)
			t.Logf(" got: %#b", sliced)
			t.Logf("want: %#b", expected)
			t.FailNow()
		}
	}
	testPanic := func(s, e int) {
		t.Helper()
		ba := bitarray.MustParse(bs)
		buf := bitarray.NewBufferFromBitArray(ba)
		var rbuf *bitarray.Buffer
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected:")
				t.Logf(" got: %#b", rbuf)
			}
		}()
		rbuf = buf.Slice(s, e)
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
	test(0, 2, "10")
	test(0, 3, "101")
	test(0, 4, "1010")
	test(0, 5, "1010-0")
	test(0, 6, "1010-01")
	test(0, 7, "1010-010")
	test(0, 8, "1010-0101")
	test(0, 9, "1010-0101 1")
	test(0, 10, "1010-0101 10")
	test(2, 2, "")
	test(2, 3, "1")
	test(2, 4, "10")
	test(2, 5, "10-0")
	test(2, 6, "10-01")
	test(2, 7, "10-010")
	test(2, 8, "10-0101")
	test(2, 9, "10-0101 1")
	test(2, 10, "10-0101 10")
	test(2, 11, "10-0101 101")
	test(2, 12, "10-0101 1010")
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

func TestBuffer_Slice_roOpsOnSlice(t *testing.T) {
	test := func(npad, padbit int, s string) {
		ba := bitarray.MustParse(s)
		buf := bitarray.NewBuffer(npad*2 + ba.Len())
		if padbit != 0 {
			buf.FillBitsAt(0, buf.Len(), 1)
		}
		padba := bitarray.NewZeroFilled(npad)
		if padbit != 0 {
			padba = bitarray.NewOneFilled(npad)
		}

		slice := buf.Slice(npad, npad+ba.Len())
		slice.V()
		slice.PutBitArrayAt(0, ba)

		// IsZero() on slice
		if got, want := slice.IsZero(), ba.IsZero(); got != want {
			t.Errorf("unexpected slice.IsZero(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %t", got)
			t.Logf(" want: %t", want)
		}

		// Len() on slice
		if got, want := slice.Len(), ba.Len(); got != want {
			t.Errorf("unexpected slice.Len(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %d", got)
			t.Logf(" want: %d", want)
		}

		// String() on slice
		if got := slice.String(); got != s {
			t.Errorf("unexpected slice.String(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %q", got)
			t.Logf(" want: %q", s)
		}

		// BitArray() on slice
		if got := slice.BitArray(); !got.Equal(ba) {
			t.Errorf("unexpected slice.BitArray(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %# b", got)
			t.Logf(" want: %# b", ba)
		}

		// Clone() on slice
		if got := slice.Clone().BitArray(); !got.Equal(ba) {
			t.Errorf("unexpected slice.Clone(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %# b", got)
			t.Logf(" want: %# b", ba)
		}

		// Format() on slice
		if got, want := fmt.Sprintf("%# b", slice), fmt.Sprintf("%# b", ba); got != want {
			t.Errorf("unexpected slice.Format(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %q", got)
			t.Logf(" want: %q", want)
		}

		// BitAt() on slice
		for i := 0; i < ba.Len(); i++ {
			if got, want := slice.BitAt(i), ba.BitAt(i); got != want {
				t.Errorf("unexpected slice.BitAt(%d): npad=%d, pad=%d, s=%q", i, npad, padbit, s)
				t.Logf("  got: %d", got)
				t.Logf(" want: %d", want)
			}
		}

		// BitArrayAt(full) on slice
		if got := slice.BitArrayAt(0, slice.Len()); !got.Equal(ba) {
			t.Errorf("unexpected slice.BitArrayAt(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %# b", got)
			t.Logf(" want: %# b", ba)
		}

		// LeadingZeros() on slice
		if got, want := slice.LeadingZeros(), ba.LeadingZeros(); got != want {
			t.Errorf("unexpected slice.LeadingZeros(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %d", got)
			t.Logf(" want: %d", want)
		}

		// TrailingZeros() on slice
		if got, want := slice.TrailingZeros(), ba.TrailingZeros(); got != want {
			t.Errorf("unexpected slice.TrailingZeros(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %d", got)
			t.Logf(" want: %d", want)
		}

		// OnesCount() on slice
		if got, want := slice.OnesCount(), ba.OnesCount(); got != want {
			t.Errorf("unexpected slice.OnesCount(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %d", got)
			t.Logf(" want: %d", want)
		}

		// String() on buf (tests the parent buf is not changed)
		if got, want := buf.String(), padba.String()+s+padba.String(); got != want {
			t.Errorf("unexpected buf.String(): npad=%d, pad=%d, s=%q", npad, padbit, s)
			t.Logf("  got: %q", got)
			t.Logf(" want: %q", want)
		}

		if t.Failed() {
			t.FailNow()
			t.Logf("  buf: %# b", buf)
			t.Logf("  buf: %s", buf.D())
			t.Logf("slice: %# b", slice)
			t.Logf("slice: %s", slice.D())
		}
	}

	tcs := []string{
		"",
		"0",
		"00",
		"000",
		"0000",
		"00000",
		"000000",
		"0000000",
		"00000000",
		"000000000",
		"0000000000",
		"000000000000000",
		"0000000000000000",
		"00000000000000000",
		"000111111111000",
		"0000111111100000",
		"00000111110000000",
		"11110111110000000",
		"11110111110000111",
		"000000000000000000000000000000000000000000000000000000000000000",
		"0000000000000000000000000000000000000000000000000000000000000000",
		"00000000000000000000000000000000000000000000000000000000000000000",
		"000000000000000000000000000000000000000000000000000000000000001",
		"0000000000000000000000000000000000000000000000000000000000000001",
		"00000000000000000000000000000000000000000000000000000000000000001",
		"100000000000000000000000000000000000000000000000000000000000000",
		"1000000000000000000000000000000000000000000000000000000000000000",
		"10000000000000000000000000000000000000000000000000000000000000000",
		"1",
		"11",
		"111",
		"1111",
		"11111",
		"111111",
		"1111111",
		"11111111",
		"111111111",
		"1111111111",
		"111111111111111",
		"1111111111111111",
		"11111111111111111",
		"111100001100111",
		"111100001100000",
		"1110000110011111",
		"0010000110011111",
		"11111000011001111",
		"00111000011001100",
		"110000110011111",
		"1111110000110011",
		"11110100110010011",
		"111111111111110101001010000101011111111101010001011110111010001",
		"1000000000000000000000011111111110000101001000010011111100000001",
		"01000111111100000000000000000000011111111110000111100000011100000",
		"1111000111100111100101001000111",
		"11110001111001111001010010001111",
		"111100011110011110010100100011111",
		"0000000111100111100101001000111",
		"00000001111001111001010010001111",
		"000000011110011110010100100011111",
		"1111000111100111100101001000100",
		"11110001111001111001010010001000",
		"111100011110011110010100100011000",
	}
	for _, tc := range tcs {
		for npad := 0; npad < 31; npad++ {
			for padbit := 0; padbit < 2; padbit++ {
				test(npad, padbit, tc)
			}
		}
	}
}

func TestBuffer_Slice_rwOpsOnSlice(t *testing.T) {
	var buf, slice *bitarray.Buffer

	test := func(tag, sliceS, bufS string) {
		t.Helper()
		sliceW := bitarray.MustParse(sliceS)
		bufW := bitarray.MustParse(bufS)
		if got := slice.BitArray(); !got.Equal(sliceW) {
			t.Errorf("%s: unexpected slice:", tag)
			t.Logf("  got: %# b", got)
			t.Logf(" want: %# b", sliceW)
		}
		if got := buf.BitArray(); !got.Equal(bufW) {
			t.Errorf("%s: unexpected buf:", tag)
			t.Logf("  got: %# b", got)
			t.Logf(" want: %# b", bufW)
		}
		if t.Failed() {
			t.FailNow()
			t.Logf("  buf: %# b", buf)
			t.Logf("  buf: %s", buf.D())
			t.Logf("slice: %# b", slice)
			t.Logf("slice: %s", slice.D())
		}
	}

	allZero31 := bitarray.NewBufferFromBitArray(bitarray.NewZeroFilled(31))
	allOne33 := bitarray.NewBufferFromBitArray(bitarray.NewOneFilled(33))
	allZeroS15 := allZero31.Slice(5, 20)

	// initial
	buf = bitarray.NewBuffer(14)
	slice = buf.Slice(1, 12)
	slice.V()
	// 0[000-0000 0000]-00
	test("initial1", "000-0000 0000", "0000-0000 0000-00")
	slice = slice.Slice(3, 10)
	slice.V()
	// 0000-[0000 000]0-00
	test("initial2", "0000 000", "0000-0000 0000-00")

	// CopyBits
	if n := bitarray.CopyBits(slice, allOne33); n != 7 {
		t.Errorf("CopyBits: unexpected n: got %d, want 7.", n)
	}
	// 0000-[1111 111]0-00
	test("CopyBits", "1111-111", "0000-1111 1110-00")

	// CopyBitsFromBytes(00010)
	slice.CopyBitsFromBytes(1, []byte{8}, 1, 5)
	// 0000-[1000 101]0-00
	test("CopyBitsFromBytes", "1000-101", "0000-1000 1010-00")

	// CopyBitsToBytes
	tb := make([]byte, 1)
	slice.CopyBitsToBytes(4, tb, 2, 3)
	if tb[0] != 0b_0010_1000 {
		t.Errorf("CopyBitsToBytes: unexpected tb: got %08b, want 0010_1000.", tb[0])
	}
	// 0000-[1000 101]0-00 (unchanged)
	test("CopyBitsToBytes", "1000-101", "0000-1000 1010-00")

	// CopyBitsN
	if n := bitarray.CopyBitsN(slice, allZero31, 5); n != 5 {
		t.Errorf("CopyBitsN: unexpected n: got %d, want 5.", n)
	}
	// 0000-[0000 001]0-00
	test("CopyBitsFromBytes", "0000-001", "0000-0000 0010-00")

	// CopyBits to parent
	if n := bitarray.CopyBits(buf, allOne33); n != 14 {
		t.Errorf("CopyBits: unexpected n: got %d, want 14.", n)
	}
	// 1111-[1111 111]1-11
	test("CopyBits(parent)", "1111-111", "1111-1111 1111-11")

	// CopyBitsPartial
	if n := bitarray.CopyBitsPartial(slice, allZeroS15, 2, 3, 5); n != 5 {
		t.Errorf("CopyBitsPartial: unexpected n: got %d, want 5.", n)
	}
	// 1111-[1100 000]1-11
	test("CopyBitsPartial", "1100-000", "1111-1100 0001-11")

	// PutBitAt
	buf.PutBitAt(0, 0)
	buf.PutBitAt(3, 0)
	// 0110-[1100 000]1-11
	test("PutBitAt", "1100-000", "0110-1100 0001-11")
	buf.PutBitAt(11, 0)
	buf.PutBitAt(13, 0)
	// 0110-[1100 000]0-10
	test("PutBitAt", "1100-000", "0110-1100 0000-10")
	slice.PutBitAt(0, 0)
	// 0110-[0100 000]0-10
	test("PutBitAt", "0100-000", "0110-0100 0000-10")
	slice.PutBitAt(5, 1)
	slice.PutBitAt(6, 1)
	// 0110-[0100 011]0-10
	test("PutBitAt", "0100-011", "0110-0100 0110-10")

	// BitArrayAt
	if ba := slice.BitArrayAt(1, 5); !ba.Equal(bitarray.MustParse("10001")) {
		t.Errorf("BitArrayAt: unexpected result: got %q, want 10001.", ba)
	}
	// 0110-[0100 011]0-10 (unchanged)
	test("BitArrayAt", "0100-011", "0110-0100 0110-10")

	// PutBitArrayAt
	slice.PutBitArrayAt(1, bitarray.MustParse("11011"))
	// 0110-[0110 111]0-10
	test("PutBitArrayAt", "0110-111", "0110-0110 1110-10")

	// byte opes
	// 0110-0110 1110-1000 0000-0000 0000-0000 0000
	buf.Resize(36, bitarray.AlignLeft)
	buf.PutBitArrayAt(5, bitarray.NewOneFilled(15))
	// 0110-0111 1111-1111 1111-0000 0000-0000 0000 (unlinked)
	test("Resize", "0110-111", "0110-0111 1111-1111 1111-0000 0000-0000 0000")

	slice = buf.Slice(5, 34)
	slice.V()
	// 0110-0[111 1111-1111 1111-0000 0000-0000 00]00
	test("Slice-2", "111 1111-1111 1111-0000 0000-0000 00", "0110-0111 1111-1111 1111-0000 0000-0000 0000")

	slice = slice.Slice(5, 26)
	slice.V()
	// 0110-0111 11[11-1111 1111-0000 0000-000]0 0000
	test("Slice-3", "11-1111 1111-0000 0000-000", "0110-0111 1111-1111 1111-0000 0000-0000 0000")

	// ByteAt
	// [11-1111 (1111-0000) 0000-000]
	if b := slice.ByteAt(6); b != 0b_1111_0000 {
		t.Errorf("ByteAt: unexpected result: got %08b, want 11110000.", b)
	}
	// [11-1111 1(111-0000 0)000-000]
	if b := slice.ByteAt(7); b != 0b_1110_0000 {
		t.Errorf("ByteAt: unexpected result: got %08b, want 11100000.", b)
	}

	// PutByteAt
	// 0110-0111 11[11-1111 (1111-0000) 0000-000]0 0000
	// 0110-0111 11[11-1111 (1000-0011) 0000-000]0 0000
	slice.PutByteAt(6, 0b_1000_0011)
	test("PutByteAt-1", "11-1111 1000-0011 0000-000", "0110-0111 1111-1111 1000-0011 0000-0000 0000")

	// 0110-0111 11[11-1111 1000-0(011 0000-0)00]0 0000
	// 0110-0111 11[11-1111 1000-0(100 0111-1)00]0 0000
	slice.PutByteAt(11, 0b_1000_1111)
	test("PutByteAt-2", "11-1111 1000-0100 0111-100", "0110-0111 1111-1111 1000-0100 0111-1000 0000")

	// BytesAt
	// 0110-0111 11[1(1-1111 1000-0100 011)1-100]0 0000
	if b := slice.BytesAt(1, 2); !bytes.Equal(b, []byte{0b_1111_1100, 0b_0010_0011}) {
		t.Errorf("BytesAt: unexpected result: got %08b, want [1111-1100 0010-0011].", b)
	}
	// 0110-0111 11[11-(1111 1000-0100 0111)-100]0 0000
	if b := slice.BytesAt(2, 2); !bytes.Equal(b, []byte{0b_1111_1000, 0b_0100_0111}) {
		t.Errorf("BytesAt: unexpected result: got %08b, want [1111-1000 0100-0111].", b)
	}
	// 0110-0111 11[11-1111 1000-0100 0111-100]0 0000 (unchanged)
	test("BytesAt", "11-1111 1000-0100 0111-100", "0110-0111 1111-1111 1000-0100 0111-1000 0000")

	// PutBytesAt
	// 0110-0111 11[1(1-1111 1000-0100 011)1-100]0 0000
	// 0110-0111 11[1(0-0110 0111-1001 100)1-100]0 0000
	slice.PutBytesAt(1, []byte{0b_0011_0011, 0b_1100_1100})
	test("BytesAt", "10-0110 0111-1001 1001-100", "0110-0111 1110-0110 0111-1001 1001-1000 0000")

	// 0110-0111 11[10-(0110 0111-1001 1001)-100]0 0000
	// 0110-0111 11[10-(1111 0000-1111 0000)-100]0 0000
	slice.PutBytesAt(2, []byte{0b_1111_0000, 0b_1111_0000})
	test("BytesAt", "10-1111 0000-1111 0000-100", "0110-0111 1110-1111 0000-1111 0000-1000 0000")
}
