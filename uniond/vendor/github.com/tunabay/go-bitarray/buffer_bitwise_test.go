// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestBuffer_ToggleBitAt(t *testing.T) {
	buf := bitarray.NewBuffer(18)
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
	chk("0000-0000 0000-0000 00")
	buf.ToggleBitAt(0)
	chk("1000-0000 0000-0000 00")
	buf.ToggleBitAt(1)
	chk("1100-0000 0000-0000 00")
	buf.ToggleBitAt(2)
	chk("1110-0000 0000-0000 00")
	buf.ToggleBitAt(6)
	chk("1110-0010 0000-0000 00")
	buf.ToggleBitAt(7)
	chk("1110-0011 0000-0000 00")
	buf.ToggleBitAt(8)
	chk("1110-0011 1000-0000 00")
	buf.ToggleBitAt(0)
	chk("0110-0011 1000-0000 00")
	buf.ToggleBitAt(1)
	chk("0010-0011 1000-0000 00")
	buf.ToggleBitAt(7)
	chk("0010-0010 1000-0000 00")
	buf.ToggleBitAt(8)
	chk("0010-0010 0000-0000 00")
	buf.ToggleBitAt(17)
	chk("0010-0010 0000-0000 01")
	buf.ToggleBitAt(16)
	chk("0010-0010 0000-0000 11")
	buf.ToggleBitAt(15)
	chk("0010-0010 0000-0001 11")
	buf.ToggleBitAt(17)
	chk("0010-0010 0000-0001 10")
	buf.ToggleBitAt(10)
	chk("0010-0010 0010-0001 10")

	chkpanic := func(off int) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, buf=%s.", off, buf.D())
			}
		}()
		buf.ToggleBitAt(off)
	}
	chkpanic(-1)
	chkpanic(18)
}

func TestBuffer_ToggleBitsAt(t *testing.T) {
	buf := bitarray.NewBuffer(28)
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
	chk("0000-0000 0000-0000 0000-0000 0000")
	buf.ToggleBitsAt(0, 0)
	chk("0000-0000 0000-0000 0000-0000 0000")
	buf.ToggleBitsAt(28, 0)
	chk("0000-0000 0000-0000 0000-0000 0000")
	buf.ToggleBitsAt(0, 16)
	chk("1111-1111 1111-1111 0000-0000 0000")
	buf.ToggleBitsAt(0, 8)
	chk("0000-0000 1111-1111 0000-0000 0000")
	buf.ToggleBitsAt(16, 8)
	chk("0000-0000 1111-1111 1111-1111 0000")
	buf.ToggleBitsAt(0, 1)
	chk("1000-0000 1111-1111 1111-1111 0000")
	buf.ToggleBitsAt(0, 2)
	chk("0100-0000 1111-1111 1111-1111 0000")
	buf.ToggleBitsAt(0, 4)
	chk("1011-0000 1111-1111 1111-1111 0000")
	buf.ToggleBitsAt(0, 7)
	chk("0100-1110 1111-1111 1111-1111 0000")
	buf.ToggleBitsAt(1, 7)
	chk("0011-0001 1111-1111 1111-1111 0000")
	buf.ToggleBitsAt(0, 28)
	chk("1100-1110 0000-0000 0000-0000 1111")
	buf.ToggleBitsAt(6, 20)
	chk("1100-1101 1111-1111 1111-1111 0011")
	buf.ToggleBitsAt(2, 5)
	chk("1111-0011 1111-1111 1111-1111 0011")
	buf.ToggleBitsAt(9, 6)
	chk("1111-0011 1000-0001 1111-1111 0011")
	buf.ToggleBitsAt(16, 2)
	chk("1111-0011 1000-0001 0011-1111 0011")
	buf.ToggleBitsAt(21, 3)
	chk("1111-0011 1000-0001 0011-1000 0011")
	buf.ToggleBitsAt(7, 2)
	chk("1111-0010 0000-0001 0011-1000 0011")

	chkpanic := func(off, nBits int) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, nBits=%d, buf=%s.", off, nBits, buf.D())
			}
		}()
		buf.ToggleBitsAt(off, nBits)
	}
	chkpanic(-1, 0)
	chkpanic(-1, 1)
	chkpanic(0, -1)
	chkpanic(0, 29)
	chkpanic(29, 0)
	chkpanic(28, 1)
	chkpanic(27, 2)
	chkpanic(24, 5)
}

func TestBuffer_AndAt(t *testing.T) {
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
	buf.AndAt(0, nil)
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	var nilba *bitarray.BitArray
	buf.AndAt(0, nilba)
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.AndAt(0, bitarray.NewZeroFilled(0))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.AndAt(30, bitarray.NewZeroFilled(0))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.AndAt(0, bitarray.NewZeroFilled(30))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.FillBitsAt(0, 30, 1)
	chk("1111-1111 1111-1111 1111-1111 1111-11")
	buf.AndAt(0, bitarray.NewZeroFilled(30))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.FillBitsAt(0, 30, 1)
	buf.AndAt(0, bitarray.MustParse("00000000"))
	chk("0000-0000 1111-1111 1111-1111 1111-11")
	buf.FillBitsAt(0, 30, 1)
	buf.AndAt(0, bitarray.MustParse("101010"))
	chk("1010-1011 1111-1111 1111-1111 1111-11")
	buf.AndAt(4, bitarray.MustParse("101010"))
	chk("1010-1010 1011-1111 1111-1111 1111-11")
	buf.AndAt(6, bitarray.MustParse("1111-0000 1111-0000"))
	chk("1010-1010 1000-0011 1100-0011 1111-11")
	buf.AndAt(7, bitarray.MustParse("10"))
	chk("1010-1010 0000-0011 1100-0011 1111-11")
	buf.AndAt(14, bitarray.MustParse("1100-1100 1100-1100"))
	chk("1010-1010 0000-0011 0000-0011 0011-00")
	buf.AndAt(8, bitarray.MustParse("1010-1010"))
	chk("1010-1010 0000-0010 0000-0011 0011-00")

	chkpanic := func(off int, ba bitarray.BitArrayer) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, ba=%v.", off, ba)
				t.Logf("buf: %s", buf.D())
			}
		}()
		buf.AndAt(off, ba)
	}
	chkpanic(-1, bitarray.NewZeroFilled(1))
	chkpanic(30, bitarray.NewZeroFilled(1))
	chkpanic(8, bitarray.NewOneFilled(23))
	chkpanic(0, bitarray.NewZeroFilled(31))
}

func TestBuffer_OrAt(t *testing.T) {
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
	buf.OrAt(0, nil)
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	var nilba *bitarray.BitArray
	buf.OrAt(0, nilba)
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.OrAt(0, bitarray.NewOneFilled(0))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.OrAt(30, bitarray.NewOneFilled(0))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.OrAt(0, bitarray.NewZeroFilled(30))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.OrAt(0, bitarray.NewOneFilled(30))
	chk("1111-1111 1111-1111 1111-1111 1111-11")
	buf.OrAt(0, bitarray.NewZeroFilled(30))
	chk("1111-1111 1111-1111 1111-1111 1111-11")
	buf.FillBitsAt(0, 30, 0)
	buf.OrAt(0, bitarray.NewOneFilled(4))
	chk("1111-0000 0000-0000 0000-0000 0000-00")
	buf.OrAt(6, bitarray.NewOneFilled(2))
	chk("1111-0011 0000-0000 0000-0000 0000-00")
	buf.OrAt(8, bitarray.MustParse("1010-1010"))
	chk("1111-0011 1010-1010 0000-0000 0000-00")
	buf.OrAt(22, bitarray.MustParse("10110011"))
	chk("1111-0011 1010-1010 0000-0010 1100-11")
	buf.OrAt(29, bitarray.MustParse("1"))
	chk("1111-0011 1010-1010 0000-0010 1100-11")
	buf.FillBitsAt(8, 20, 0)
	chk("1111-0011 0000-0000 0000-0000 0000-11")
	buf.OrAt(4, bitarray.MustParse("1010-1010 1010-1010 1010"))
	chk("1111-1011 1010-1010 1010-1010 0000-11")
	buf.OrAt(10, bitarray.MustParse("0101-0101 0101"))
	chk("1111-1011 1011-1111 1111-1110 0000-11")

	chkpanic := func(off int, ba bitarray.BitArrayer) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, ba=%v.", off, ba)
				t.Logf("buf: %s", buf.D())
			}
		}()
		buf.OrAt(off, ba)
	}
	chkpanic(-1, bitarray.NewZeroFilled(1))
	chkpanic(30, bitarray.NewZeroFilled(1))
	chkpanic(8, bitarray.NewOneFilled(23))
}

func TestBuffer_XorAt(t *testing.T) {
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
	buf.XorAt(0, nil)
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	var nilba *bitarray.BitArray
	buf.XorAt(0, nilba)
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.XorAt(0, bitarray.NewOneFilled(0))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.XorAt(30, bitarray.NewOneFilled(0))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.XorAt(0, bitarray.NewZeroFilled(30))
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.XorAt(0, bitarray.NewOneFilled(30))
	chk("1111-1111 1111-1111 1111-1111 1111-11")
	buf.XorAt(0, bitarray.NewZeroFilled(30))
	chk("1111-1111 1111-1111 1111-1111 1111-11")
	buf.XorAt(0, bitarray.NewOneFilled(4))
	chk("0000-1111 1111-1111 1111-1111 1111-11")
	buf.XorAt(6, bitarray.NewOneFilled(2))
	chk("0000-1100 1111-1111 1111-1111 1111-11")
	buf.XorAt(8, bitarray.MustParse("1010-1010"))
	chk("0000-1100 0101-0101 1111-1111 1111-11")
	buf.XorAt(22, bitarray.MustParse("10110011"))
	chk("0000-1100 0101-0101 1111-1101 0011-00")
	buf.XorAt(29, bitarray.MustParse("1"))
	chk("0000-1100 0101-0101 1111-1101 0011-01")
	buf.XorAt(10, bitarray.NewOneFilled(20))
	chk("0000-1100 0110-1010 0000-0010 1100-10")
	buf.XorAt(4, bitarray.MustParse("1010-1010 1010-1010 1010"))
	chk("0000-0110 1100-0000 1010-1000 1100-10")
	buf.XorAt(10, bitarray.MustParse("0101-0101 0101"))
	chk("0000-0110 1101-0101 1111-1100 1100-10")

	chkpanic := func(off int, ba bitarray.BitArrayer) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, ba=%v.", off, ba)
				t.Logf("buf: %s", buf.D())
			}
		}()
		buf.XorAt(off, ba)
	}
	chkpanic(-1, bitarray.NewZeroFilled(1))
	chkpanic(30, bitarray.NewZeroFilled(1))
	chkpanic(8, bitarray.NewOneFilled(23))
}

func TestBuffer_LeadingZeros(t *testing.T) {
	buf := bitarray.NewBuffer(30)
	buf.PutBitArrayAt(6, bitarray.MustParse("0000-0111 0000-0111 0011"))
	if n := buf.LeadingZeros(); n != 11 {
		t.Errorf("unexpected: got %d, want 11", n)
	}
	buf.PutBitArrayAt(6, bitarray.MustParse("101"))
	if n := buf.LeadingZeros(); n != 6 {
		t.Errorf("unexpected: got %d, want 6", n)
	}
	buf.Resize(0, bitarray.AlignLeft)
	if n := buf.LeadingZeros(); n != 0 {
		t.Errorf("unexpected: got %d, want 0", n)
	}
}

func TestBuffer_TrailingZeros(t *testing.T) {
	buf := bitarray.NewBuffer(30)
	buf.PutBitArrayAt(6, bitarray.MustParse("1111-0111 0000-0111"))
	if n := buf.TrailingZeros(); n != 8 {
		t.Errorf("unexpected: got %d, want 8", n)
	}
	buf.PutBitArrayAt(22, bitarray.MustParse("1010"))
	if n := buf.TrailingZeros(); n != 5 {
		t.Errorf("unexpected: got %d, want 5", n)
	}
	buf.Resize(0, bitarray.AlignLeft)
	if n := buf.TrailingZeros(); n != 0 {
		t.Errorf("unexpected: got %d, want 0", n)
	}
}

func TestBuffer_OnesCount(t *testing.T) {
	buf := bitarray.NewBuffer(30)
	buf.PutBitArrayAt(6, bitarray.MustParse("1111-0111 0000-0111"))
	if n := buf.OnesCount(); n != 10 {
		t.Errorf("unexpected: got %d, want 10", n)
	}
	buf.PutBitArrayAt(2, bitarray.MustParse("1010"))
	if n := buf.OnesCount(); n != 12 {
		t.Errorf("unexpected: got %d, want 12", n)
	}
	buf.Resize(0, bitarray.AlignLeft)
	if n := buf.OnesCount(); n != 0 {
		t.Errorf("unexpected: got %d, want 0", n)
	}
}
