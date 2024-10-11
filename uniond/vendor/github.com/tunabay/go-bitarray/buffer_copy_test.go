// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestBuffer_CopyBitsFromBytes(t *testing.T) {
	// (off int, b []byte, bOff, nBits int)
	buf := bitarray.NewBuffer(30)
	chk := func(wantS string) {
		t.Helper()
		buf.V()
		got := buf.BitArray()
		want := bitarray.MustParse(wantS)
		if !got.Equal(want) {
			t.Error("unexpected:")
			t.Logf(" got: %#b", got)
			t.Logf("want: %#b", want)
			t.Logf(" buf: %s", buf.D())
		}
	}
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.CopyBitsFromBytes(0, []byte{}, 0, 0)
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.CopyBitsFromBytes(0, []byte{0xA5}, 0, 4)
	chk("1010-0000 0000-0000 0000-0000 0000-00")
	buf.CopyBitsFromBytes(4, []byte{0xA5}, 4, 4)
	chk("1010-0101 0000-0000 0000-0000 0000-00")
	buf.CopyBitsFromBytes(6, []byte{0x3c}, 2, 4)
	chk("1010-0111 1100-0000 0000-0000 0000-00")
	buf.CopyBitsFromBytes(8, []byte{0xFA, 0xAA, 0xAF}, 4, 16)
	chk("1010-0111 1010-1010 1010-1010 0000-00")
	buf.CopyBitsFromBytes(12, []byte{0xF5, 0x55, 0x5F}, 4, 17)
	chk("1010-0111 1010-0101 0101-0101 0101-10")
	buf.CopyBitsFromBytes(18, []byte{0x00, 0x00, 0xFF, 0xF0}, 16, 12)
	chk("1010-0111 1010-0101 0111-1111 1111-11")
	buf.CopyBitsFromBytes(29, []byte{0xFF, 0xFF, 0xFF, 0xFE}, 31, 1)
	chk("1010-0111 1010-0101 0111-1111 1111-10")
	buf.CopyBitsFromBytes(15, []byte{0xFF, 0xFF, 0x7F, 0xFF}, 16, 1)
	chk("1010-0111 1010-0100 0111-1111 1111-10")
	buf.CopyBitsFromBytes(30, nil, 0, 0)
	chk("1010-0111 1010-0100 0111-1111 1111-10")
	buf.CopyBitsFromBytes(0, nil, 0, 0)
	chk("1010-0111 1010-0100 0111-1111 1111-10")
	buf.CopyBitsFromBytes(16, []byte{0x88, 0x88}, 0, 14)
	chk("1010-0111 1010-0100 1000-1000 1000-10")

	chkpanic := func(off int, b []byte, bOff, nBits int) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, ba=%#b.", off, buf)
			}
		}()
		buf.CopyBitsFromBytes(off, b, bOff, nBits)
	}
	chkpanic(-1, []byte{}, 0, 0)
	chkpanic(31, []byte{}, 0, 0)
	chkpanic(23, []byte{0}, 0, 8)
	chkpanic(24, []byte{0}, 1, 7)
	chkpanic(15, []byte{0, 0}, 0, 16)
	chkpanic(16, []byte{0, 0}, 1, 15)
	chkpanic(17, []byte{0, 0}, 2, 14)
}

func TestBuffer_CopyBitsToBytes(t *testing.T) {
	bs := make([]byte, 4)
	buf := bitarray.NewBuffer(30)
	chk := func(wantB ...byte) {
		t.Helper()
		if !bytes.Equal(bs, wantB) {
			t.Error("unexpected:")
			t.Logf(" got: %08b", bs)
			t.Logf("want: %08b", wantB)
			t.Logf(" buf: %#b", buf)
		}
	}
	buf.PutBitArrayAt(0, bitarray.MustParse("1010-1111 0101-1100 1000-1000 0000-01"))
	buf.CopyBitsToBytes(0, bs, 0, 0)
	chk(0b_0000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000)
	buf.CopyBitsToBytes(0, bs, 0, 30)
	chk(0b_1010_1111, 0b_0101_1100, 0b_1000_1000, 0b_0000_0100)
	buf.CopyBitsToBytes(24, bs, 0, 4)
	chk(0b_0000_1111, 0b_0101_1100, 0b_1000_1000, 0b_0000_0100)
	buf.CopyBitsToBytes(8, bs, 4, 4)
	chk(0b_0000_0101, 0b_0101_1100, 0b_1000_1000, 0b_0000_0100)
	buf.CopyBitsToBytes(0, bs, 10, 18)
	chk(0b_0000_0101, 0b_0110_1011, 0b_1101_0111, 0b_0010_0100)
	buf.CopyBitsToBytes(0, bs, 31, 1)
	chk(0b_0000_0101, 0b_0110_1011, 0b_1101_0111, 0b_0010_0101)
	buf.CopyBitsToBytes(29, bs, 0, 1)
	chk(0b_1000_0101, 0b_0110_1011, 0b_1101_0111, 0b_0010_0101)
	buf.CopyBitsToBytes(8, bs, 0, 16)
	chk(0b_0101_1100, 0b_1000_1000, 0b_1101_0111, 0b_0010_0101)
	buf.CopyBitsToBytes(5, bs, 10, 2)
	chk(0b_0101_1100, 0b_1011_1000, 0b_1101_0111, 0b_0010_0101)

	chkpanic := func(off int, b []byte, bOff, nBits int) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, ba=%#b.", off, buf)
			}
		}()
		buf.CopyBitsToBytes(off, b, bOff, nBits)
	}
	chkpanic(-1, []byte{}, 0, 0)
	chkpanic(31, []byte{}, 0, 0)
	chkpanic(23, []byte{0}, 0, 8)
	chkpanic(24, []byte{0}, 1, 7)
	chkpanic(15, []byte{0, 0}, 0, 16)
	chkpanic(16, []byte{0, 0}, 1, 15)
	chkpanic(17, []byte{0, 0}, 2, 14)
}

func TestCopyBits(t *testing.T) {
	tcs := []struct {
		dst, src, res string
		n             int
	}{
		{"0", "", "0", 0},
		{"1", "", "1", 0},
		{"0000-0000 0", "", "0000-0000 0", 0},
		{"1111-1111 1", "", "1111-1111 1", 0},
		{"", "1", "", 0},
		{"", "1111-1111", "", 0},
		{"0", "1", "1", 1},
		{"1", "0", "0", 1},
		{"0", "1000-0000 0", "1", 1},
		{"1", "0111-1111 1", "0", 1},
		{"0000-0000", "1", "1000-0000", 1},
		{"1111-1111", "0", "0111-1111", 1},
		{"0000-0000", "1111-1111", "1111-1111", 8},
		{"1111-1111", "0000-0000", "0000-0000", 8},
		{"0000-0000", "1111-1111 0000-00", "1111-1111", 8},
		{"1111-1111", "0000-0000 1111-11", "0000-0000", 8},
		{"0000-0000 0000-0000 0", "1", "1000-0000 0000-0000 0", 1},
		{"1111-1111 1111-1111 1", "0", "0111-1111 1111-1111 1", 1},
		{"0000-0000 0000-0000 0", "1111-1111 11", "1111-1111 1100-0000 0", 10},
		{"1111-1111 1111-1111 1", "0000-0000 00", "0000-0000 0011-1111 1", 10},
		{"0000-0000 0000-0000 0", "1111-1010 1010-1010 1111", "1111-1010 1010-1010 1", 17},
		{"1111-1111 1111-1111 1", "0000-1010 1010-1010 0000", "0000-1010 1010-1010 0", 17},
	}
	for _, tc := range tcs {
		dstb := bitarray.MustParse(tc.dst)
		srcb := bitarray.MustParse(tc.src)
		want := bitarray.MustParse(tc.res)
		dst := bitarray.NewBufferFromBitArray(dstb)
		src := bitarray.NewBufferFromBitArray(srcb)
		n := bitarray.CopyBits(dst, src)
		got := dst.BitArray()
		if n != tc.n {
			t.Errorf("unexpected n: got %d, want %d.", n, tc.n)
		}
		if !got.Equal(want) {
			t.Errorf("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf("want: %#b", want)
		}
		if t.Failed() {
			t.Errorf("failed: CopyBits(%q, %q)", tc.dst, tc.src)
			t.FailNow()
		}
	}
}

func TestCopyBitsN(t *testing.T) {
	tcs := []struct {
		dst, src, res string
		n, w          int
	}{
		{"0", "", "0", 0, 0},
		{"1", "", "1", 7, 0},
		{"0000-0000 0", "", "0000-0000 0", 0, 0},
		{"1111-1111 1", "", "1111-1111 1", 7, 0},
		{"", "1", "", 0, 0},
		{"", "1111-1111", "", 7, 0},
		{"0", "1", "0", 0, 0},
		{"0", "1", "1", 1, 1},
		{"1", "0", "0", 7, 1},
		{"0", "1000-0000 0", "1", 1, 1},
		{"1", "0111-1111 1", "0", 7, 1},
		{"0000-0000", "1", "1000-0000", 1, 1},
		{"1111-1111", "0", "0111-1111", 9, 1},
		{"0000-0000", "1111-1111", "1111-0000", 4, 4},
		{"0000-0000", "1111-1111", "1111-1111", 99, 8},
		{"1111-1111", "0000-0000", "0000-0001", 7, 7},
		{"1111-1111", "0000-0000", "0000-0000", 8, 8},
		{"0000-0000", "1111-1111 0000-00", "1110-0000", 3, 3},
		{"0000-0000", "1111-1111 0000-00", "1111-1111", 10, 8},
		{"0000-0000", "1111-1111 0000-00", "1111-1111", 99, 8},
		{"1111-1111", "0000-0000 1111-11", "0000-0001", 7, 7},
		{"1111-1111", "0000-0000 1111-11", "0000-0000", 8, 8},
		{"1111-1111", "0000-0000 1111-11", "0000-0000", 10, 8},
		{"0000-0000 0000-0000 0", "1", "1000-0000 0000-0000 0", 16, 1},
		{"1111-1111 1111-1111 1", "0", "0111-1111 1111-1111 1", 32, 1},
		{"0000-0000 0000-0000 0", "1111-1111 11", "1111-1100 0000-0000 0", 6, 6},
		{"0000-0000 0000-0000 0", "1111-1111 11", "1111-1111 1100-0000 0", 16, 10},
		{"1111-1111 1111-1111 1", "0000-0000 00", "0111-1111 1111-1111 1", 1, 1},
		{"1111-1111 1111-1111 1", "0000-0000 00", "0000-0000 0011-1111 1", 99, 10},
		{"0000-0000 0000-0000 0", "1111-1010 1010-1010 1111", "1111-1010 1010-0000 0", 12, 12},
		{"0000-0000 0000-0000 0", "1111-1010 1010-1010 1111", "1111-1010 1010-1010 1", 17, 17},
		{"1111-1111 1111-1111 1", "0000-1010 1010-1010 0000", "0000-1010 1010-1010 0", 32, 17},
	}
	for _, tc := range tcs {
		dstb := bitarray.MustParse(tc.dst)
		srcb := bitarray.MustParse(tc.src)
		want := bitarray.MustParse(tc.res)
		dst := bitarray.NewBufferFromBitArray(dstb)
		src := bitarray.NewBufferFromBitArray(srcb)
		n := bitarray.CopyBitsN(dst, src, tc.n)
		got := dst.BitArray()
		if n != tc.w {
			t.Errorf("unexpected n: got %d, want %d.", n, tc.w)
		}
		if !got.Equal(want) {
			t.Errorf("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf("want: %#b", want)
		}
		if t.Failed() {
			t.Errorf("failed: CopyBitsN(%q, %q, %d)", tc.dst, tc.src, tc.n)
			t.FailNow()
		}
	}
}

func TestCopyBitsPartial(t *testing.T) {
	tcs := []struct {
		dst, src, res string
		do, so, n, w  int
	}{
		{"0", "", "0", 0, 0, 0, 0},
		{"1", "", "1", 0, 0, 7, 0},
		{"0000-0000 0", "", "0000-0000 0", 0, 0, 0, 0},
		{"0000-0000 0", "", "0000-0000 0", 8, 0, 0, 0},
		{"1111-1111 1", "", "1111-1111 1", 3, 0, 7, 0},
		{"", "1", "", 0, 0, 0, 0},
		{"", "1111-1111", "", 0, 0, 7, 0},
		{"0", "1", "0", 0, 0, 0, 0},
		{"0", "00100", "1", 0, 2, 1, 1},
		{"1", "1110", "0", 0, 3, 7, 1},
		{"0", "1000-0000 0", "1", 0, 0, 1, 1},
		{"0", "0000-0000 1", "1", 0, 8, 1, 1},
		{"1", "0111-1111 1", "0", 0, 0, 7, 1},
		{"1", "1111-1110 1", "0", 0, 7, 7, 1},
		{"0000-0000", "1", "1000-0000", 0, 0, 1, 1},
		{"0000-0000", "1", "0000-1000", 4, 0, 7, 1},
		{"1111-1111", "0", "0111-1111", 0, 0, 9, 1},
		{"1111-1111", "0", "1111-1110", 7, 0, 9, 1},
		{"0000-0000", "1111-1111", "1111-0000", 0, 0, 4, 4},
		{"0000-0000", "1111-1101", "0011-0100", 2, 4, 99, 4},
		{"0000-0000", "1111-1111", "1111-1111", 0, 0, 99, 8},
		{"0000-0000", "1101-1111", "0000-1101", 4, 0, 99, 4},
		{"1111-1111", "0000-0000", "0000-0001", 0, 0, 7, 7},
		{"1111-1111", "0000-0000", "0000-0000", 0, 0, 8, 8},
		{"0000-0000", "1111-1111 0000-00", "1110-0000", 0, 0, 3, 3},
		{"0000-0000", "1111-1111 0000-00", "1111-1111", 0, 0, 10, 8},
		{"0000-0000", "1111-1111 0000-00", "1111-1111", 0, 0, 99, 8},
		{"0000-0000", "1111-1111 0001-00", "1111-0001", 0, 4, 99, 8},
		{"1111-1111", "0000-0000 1111-11", "0000-0001", 0, 0, 7, 7},
		{"1111-1111", "0000-0000 1111-11", "0000-0000", 0, 0, 8, 8},
		{"1111-1111", "0000-0000 1111-11", "0000-0000", 0, 0, 10, 8},
		{"1111-1111", "0000-0000 1100-11", "1001-1001", 1, 6, 6, 6},
		{"0000-0000 0000-0000 0", "1", "1000-0000 0000-0000 0", 0, 0, 16, 1},
		{"0000-0000 0000-0000 0", "1", "0000-0000 0000-0000 1", 16, 0, 99, 1},
		{"1111-1111 1111-1111 1", "0", "0111-1111 1111-1111 1", 0, 0, 32, 1},
		{"1111-1111 1111-1111 1", "0", "1111-1111 1111-1110 1", 15, 0, 1, 1},
		{"0000-0000 0000-0000 0", "1111-1111 11", "1111-1100 0000-0000 0", 0, 0, 6, 6},
		{"0000-0000 0000-0000 0", "1111-1111 11", "1111-1111 1100-0000 0", 0, 0, 16, 10},
		{"1111-1111 1111-1111 1", "0000-0000 00", "0111-1111 1111-1111 1", 0, 0, 1, 1},
		{"1111-1111 1111-1111 1", "0000-0000 00", "0000-0000 0011-1111 1", 0, 0, 99, 10},
		{"0000-0000 0000-0000 0", "1111-1010 1010-1010 1111", "1111-1010 1010-0000 0", 0, 0, 12, 12},
		{"0000-0000 0000-0000 0", "1111-1010 1010-1010 1111", "1111-1010 1010-1010 1", 0, 0, 17, 17},
		{"1111-1111 1111-1111 1", "0000-1010 1010-1010 0000", "0000-1010 1010-1010 0", 0, 0, 32, 17},
	}
	for _, tc := range tcs {
		dstb := bitarray.MustParse(tc.dst)
		srcb := bitarray.MustParse(tc.src)
		want := bitarray.MustParse(tc.res)
		dst := bitarray.NewBufferFromBitArray(dstb)
		src := bitarray.NewBufferFromBitArray(srcb)
		n := bitarray.CopyBitsPartial(dst, src, tc.do, tc.so, tc.n)
		got := dst.BitArray()
		if n != tc.w {
			t.Errorf("unexpected n: got %d, want %d.", n, tc.w)
		}
		if !got.Equal(want) {
			t.Errorf("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf("want: %#b", want)
		}
		if t.Failed() {
			t.Errorf("failed: CopyBitsPartial(%q, %q, %d, %d, %d)", tc.dst, tc.src, tc.do, tc.so, tc.n)
			t.FailNow()
		}
	}
}
