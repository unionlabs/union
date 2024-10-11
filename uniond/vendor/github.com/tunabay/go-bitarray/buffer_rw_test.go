// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestBuffer_BitAt(t *testing.T) {
	buf := bitarray.NewBuffer(18)
	chk := func(off int, want byte) {
		t.Helper()
		bit := buf.BitAt(off)
		if bit != want {
			t.Errorf("unexpected: off=%d: got %d, want %d.", off, bit, want)
		}
	}

	buf.PutBitArrayAt(0, bitarray.MustParse("1100-1111 0000-1010 11"))
	chk(0, 1)
	chk(1, 1)
	chk(7, 1)
	chk(8, 0)
	chk(15, 0)
	chk(16, 1)
	chk(17, 1)
	buf.PutBitArrayAt(0, bitarray.MustParse("0011-0000 1111-0101 00"))
	chk(0, 0)
	chk(1, 0)
	chk(7, 0)
	chk(8, 1)
	chk(15, 1)
	chk(16, 0)
	chk(17, 0)

	chkpanic := func(off int) {
		var bit byte
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, bit=%d.", off, bit)
			}
		}()
		bit = buf.BitAt(off)
	}
	chkpanic(-1)
	chkpanic(18)
}

func TestBuffer_PutBitAt(t *testing.T) {
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
	buf.PutBitAt(0, 1)
	buf.PutBitAt(1, 0)
	buf.PutBitAt(7, 1)
	buf.PutBitAt(8, 1)
	buf.PutBitAt(12, 1)
	buf.PutBitAt(15, 0)
	buf.PutBitAt(16, 0)
	buf.PutBitAt(17, 1)
	chk("1000-0001 1000-1000 01")
	buf.PutBitAt(0, 0)
	buf.PutBitAt(7, 0)
	buf.PutBitAt(8, 0)
	buf.PutBitAt(17, 0)
	chk("0000-0000 0000-1000 00")

	chkpanic := func(off int, bit byte) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, bit=%d.", off, bit)
			}
		}()
		buf.PutBitAt(off, bit)
	}
	chkpanic(-1, 1)
	chkpanic(18, 0)
}

func TestBuffer_BitArrayAt(t *testing.T) {
	buf := bitarray.NewBuffer(22)
	chk := func(off, nBits int, wantS string) {
		t.Helper()
		want := bitarray.MustParse(wantS)
		got := buf.BitArrayAt(off, nBits)
		if !got.Equal(want) {
			t.Errorf("unexpected: off=%d, nBits=%d", off, nBits)
			t.Logf(" got: %#b", got)
			t.Logf("want: %#b", want)
		}
	}

	buf.PutBitArrayAt(0, bitarray.MustParse("1100-1111 0000-1010 1100-11"))
	chk(0, 0, "")
	chk(22, 0, "")
	chk(0, 4, "1100")
	chk(2, 6, "00-1111")
	chk(4, 8, "1111-0000")
	chk(7, 1, "1")
	chk(7, 3, "100")
	chk(8, 1, "0")
	chk(4, 14, "1111 0000-1010 11")
	chk(18, 4, "0011")
	chk(21, 1, "1")

	chkpanic := func(off, nBits int) {
		var ba *bitarray.BitArray
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, nBits=%d, got=%#b.", off, nBits, ba)
			}
		}()
		ba = buf.BitArrayAt(off, nBits)
	}
	chkpanic(-1, 4)
	chkpanic(19, 4)
	chkpanic(0, -1)
	chkpanic(0, 99)
	chkpanic(99, 1)
}

func TestBuffer_PutBitArrayAt(t *testing.T) {
	buf := bitarray.NewBuffer(22)
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
	chk("0000-0000 0000-0000 0000-00")
	buf.PutBitArrayAt(0, nil)
	chk("0000-0000 0000-0000 0000-00")
	buf.PutBitArrayAt(22, bitarray.NewZeroFilled(0))
	chk("0000-0000 0000-0000 0000-00")
	buf.PutBitArrayAt(2, bitarray.NewOneFilled(9))
	chk("0011-1111 1110-0000 0000-00")
	buf.PutBitArrayAt(4, bitarray.NewZeroFilled(3))
	chk("0011-0001 1110-0000 0000-00")
	buf.PutBitArrayAt(7, bitarray.MustParse("1010-1010 10"))
	chk("0011-0001 0101-0101 0000-00")
	buf.PutBitArrayAt(16, bitarray.NewOneFilled(6))
	chk("0011-0001 0101-0101 1111-11")
	buf.PutBitArrayAt(14, bitarray.NewZeroFilled(6))
	chk("0011-0001 0101-0100 0000-11")
	buf.PutBitArrayAt(12, bitarray.NewZeroFilled(10))
	chk("0011-0001 0101-0000 0000-00")
	buf.PutBitArrayAt(0, bitarray.NewOneFilled(22))
	chk("1111-1111 1111-1111 1111-11")
	buf.PutBitArrayAt(8, bitarray.MustParse("0011 1100"))
	chk("1111-1111 0011-1100 1111-11")
	buf.PutBitArrayAt(21, bitarray.NewZeroFilled(1))
	chk("1111-1111 0011-1100 1111-10")

	chkpanic := func(off int, ba bitarray.BitArrayer) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, ba=%v.", off, ba)
			}
		}()
		buf.PutBitArrayAt(off, ba)
	}
	chkpanic(-1, nil)
	chkpanic(-1, bitarray.NewZeroFilled(0))
	chkpanic(-1, bitarray.NewZeroFilled(8))
	chkpanic(0, bitarray.NewZeroFilled(23))
	chkpanic(0, bitarray.NewOneFilled(23))
	chkpanic(22, bitarray.NewZeroFilled(1))
	chkpanic(23, bitarray.NewZeroFilled(0))
	chkpanic(24, bitarray.NewZeroFilled(0))
	chkpanic(16, bitarray.NewZeroFilled(7))
}

func TestBuffer_ByteAt(t *testing.T) {
	buf := bitarray.NewBuffer(22)
	chk := func(off int, want byte) {
		t.Helper()
		got := buf.ByteAt(off)
		if got != want {
			t.Errorf("unexpected: off=%d, got=%08b, want=%08b", off, got, want)
		}
	}

	buf.PutBitArrayAt(0, bitarray.MustParse("1100-1111 0000-1010 1100-11"))
	chk(0, 0b_1100_1111)
	chk(1, 0b_1001_1110)
	chk(2, 0b_0011_1100)
	chk(3, 0b_0111_1000)
	chk(4, 0b_1111_0000)
	chk(5, 0b_1110_0001)
	chk(6, 0b_1100_0010)
	chk(7, 0b_1000_0101)
	chk(8, 0b_0000_1010)
	chk(9, 0b_0001_0101)
	chk(10, 0b_0010_1011)
	chk(10, 0b_0010_1011)
	chk(14, 0b_1011_0011)

	chkpanic := func(off int) {
		var b byte
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, got=%08b.", off, b)
			}
		}()
		b = buf.ByteAt(off)
	}
	chkpanic(-1)
	chkpanic(15)
	buf.Resize(0, bitarray.AlignLeft)
	chkpanic(0)
}

func TestBuffer_PutByteAt(t *testing.T) {
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
	buf.PutByteAt(0, 0b_0000_0000)
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.PutByteAt(0, 0b_1010_1100)
	chk("1010_1100 0000-0000 0000-0000 0000-00")
	buf.PutByteAt(1, 0b_0011_0111)
	chk("1001_1011 1000-0000 0000-0000 0000-00")
	buf.PutByteAt(2, 0b_1111_1111)
	chk("1011_1111 1100-0000 0000-0000 0000-00")
	buf.PutByteAt(3, 0b_0000_0001)
	chk("1010_0000 0010-0000 0000-0000 0000-00")
	buf.PutByteAt(7, 0b_1100_0011)
	chk("1010_0001 1000-0110 0000-0000 0000-00")
	buf.PutByteAt(15, 0b_1111_1111)
	chk("1010_0001 1000-0111 1111-1110 0000-00")
	buf.PutByteAt(16, 0b_0011_0101)
	chk("1010_0001 1000-0111 0011-0101 0000-00")
	buf.PutByteAt(22, 0b_1110_0111)
	chk("1010_0001 1000-0111 0011-0111 1001-11")

	chkpanic := func(off int, b byte) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, ba=%#b.", off, buf)
			}
		}()
		buf.PutByteAt(off, b)
	}
	chkpanic(-1, 0)
	chkpanic(-99, 0)
	chkpanic(23, 0)
	chkpanic(30, 0)
	buf.Resize(0, bitarray.AlignLeft)
	chkpanic(0, 0)
}

func TestBuffer_RawBytes(t *testing.T) {
	chk := func(buf *bitarray.Buffer, want ...byte) {
		t.Helper()
		got := buf.RawBytes()
		if !bytes.Equal(got, want) {
			t.Error("unexpected RawBytes():")
			t.Logf(" got: %08b", got)
			t.Logf("want: %08b", want)
			t.Logf(" buf: %s", buf.D())
		}
	}

	ba := bitarray.MustParse("1100-1111 0000-1010 1100-1110 0011-01")
	buf := bitarray.NewBufferFromBitArray(ba)
	chk(
		buf,
		0b_1100_1111, 0b_0000_1010, 0b_1100_1110, 0b_0011_0100,
	)
	chk(
		buf.Slice(0, 17),
		0b_1100_1111, 0b_0000_1010, 0b_1100_1110,
	)
	chk(
		buf.Slice(4, 28),
		0b_1111_0000, 0b_1010_1100, 0b_1110_0011,
	)
	chk(
		buf.Slice(4, 24),
		0b_1111_0000, 0b_1010_1100, 0b_1110_0000,
	)
	chk(
		buf.Slice(4, 24).SliceToEnd(4),
		0b_0000_1010, 0b_1100_1110,
	)
	chk(
		buf.Slice(4, 24).Slice(4, 16),
		0b_0000_1010, 0b_1100_1110,
	)
	chk(
		buf.Slice(4, 24).SliceToEnd(6),
		0b_0010_1011, 0b_0011_1000,
	)
}

func TestBuffer_Bytes(t *testing.T) {
	chk := func(buf *bitarray.Buffer, want ...byte) {
		t.Helper()
		got := buf.Bytes()
		if !bytes.Equal(got, want) {
			t.Error("unexpected Bytes():")
			t.Logf(" got: %08b", got)
			t.Logf("want: %08b", want)
			t.Logf(" buf: %s", buf.D())
		}
	}

	ba := bitarray.MustParse("1100-1111 0000-1010 1100-1110 0011-01")
	buf := bitarray.NewBufferFromBitArray(ba)
	chk(
		buf,
		0b_1100_1111, 0b_0000_1010, 0b_1100_1110, 0b_0011_0100,
	)
	chk(
		buf.Slice(0, 17),
		0b_1100_1111, 0b_0000_1010, 0b_1000_0000,
	)
	chk(
		buf.Slice(4, 28),
		0b_1111_0000, 0b_1010_1100, 0b_1110_0011,
	)
	chk(
		buf.Slice(4, 24),
		0b_1111_0000, 0b_1010_1100, 0b_1110_0000,
	)
	chk(
		buf.Slice(4, 24).SliceToEnd(4),
		0b_0000_1010, 0b_1100_1110,
	)
	chk(
		buf.Slice(4, 24).Slice(4, 16),
		0b_0000_1010, 0b_1100_0000,
	)
	chk(
		buf.Slice(4, 24).SliceToEnd(6),
		0b_0010_1011, 0b_0011_1000,
	)
}

func TestBuffer_BytesAt(t *testing.T) {
	buf := bitarray.NewBuffer(30)
	chk := func(off, nBytes int, want ...byte) {
		t.Helper()
		got := buf.BytesAt(off, nBytes)
		if !bytes.Equal(got, want) {
			t.Errorf("unexpected: off=%d, nBytes=%d:", off, nBytes)
			t.Logf(" got: %08b", got)
			t.Logf("want: %08b", want)
			t.Logf(" buf: %s", buf.D())
		}
	}

	buf.PutBitArrayAt(0, bitarray.MustParse("1100-1111 0000-1010 1100-1110 0011-01"))
	chk(0, 0)
	chk(30, 0)
	chk(0, 1, 0b_1100_1111)
	chk(0, 2, 0b_1100_1111, 0b_0000_1010)
	chk(0, 3, 0b_1100_1111, 0b_0000_1010, 0b_1100_1110)
	chk(1, 3, 0b_1001_1110, 0b_0001_0101, 0b_1001_1100)
	chk(2, 3, 0b_0011_1100, 0b_0010_1011, 0b_0011_1000)
	chk(3, 3, 0b_0111_1000, 0b_0101_0110, 0b_0111_0001)
	chk(4, 3, 0b_1111_0000, 0b_1010_1100, 0b_1110_0011)
	chk(5, 3, 0b_1110_0001, 0b_0101_1001, 0b_1100_0110)
	chk(6, 3, 0b_1100_0010, 0b_1011_0011, 0b_1000_1101)
	chk(7, 2, 0b_1000_0101, 0b_0110_0111)
	chk(8, 2, 0b_0000_1010, 0b_1100_1110)
	chk(9, 2, 0b_0001_0101, 0b_1001_1100)
	chk(22, 1, 0b_1000_1101)

	chkpanic := func(off, nBytes int) {
		var b []byte
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, nBytes=%d, got=%08b.", off, nBytes, b)
			}
		}()
		b = buf.BytesAt(off, nBytes)
	}
	chkpanic(-1, 1)
	chkpanic(-1, 0)
	chkpanic(0, 99)
	chkpanic(0, 4)
	chkpanic(0, -1)
	chkpanic(31, 0)
	chkpanic(23, 1)
	chkpanic(15, 2)
	chkpanic(7, 3)
	buf.Resize(0, bitarray.AlignLeft)
	chkpanic(0, 1)
	chkpanic(1, 0)
}

func TestBuffer_PutBytesAt(t *testing.T) {
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
	buf.PutBytesAt(0, []byte{})
	chk("0000-0000 0000-0000 0000-0000 0000-00")
	buf.PutBytesAt(0, []byte{0xAA})
	chk("1010-1010 0000-0000 0000-0000 0000-00")
	buf.PutBytesAt(4, []byte{0x55})
	chk("1010-0101 0101-0000 0000-0000 0000-00")
	buf.PutBytesAt(6, []byte{0xF3, 0xCC})
	chk("1010-0111 1100-1111 0011-0000 0000-00")
	buf.PutBytesAt(8, []byte{0x00, 0xFF})
	chk("1010-0111 0000-0000 1111-1111 0000-00")
	buf.PutBytesAt(6, []byte{0xAA, 0xAA, 0xAA})
	chk("1010-0110 1010-1010 1010-1010 1010-10")
	buf.PutBytesAt(6, []byte{0x00, 0x00, 0x00})
	chk("1010-0100 0000-0000 0000-0000 0000-00")
	buf.PutBytesAt(22, []byte{0x55})
	chk("1010-0100 0000-0000 0000-0001 0101-01")
	buf.PutBytesAt(7, []byte{0xFF, 0xAA})
	chk("1010-0101 1111-1111 0101-0101 0101-01")
	buf.PutBytesAt(30, []byte{})
	chk("1010-0101 1111-1111 0101-0101 0101-01")
	buf.PutBytesAt(29, []byte{})
	chk("1010-0101 1111-1111 0101-0101 0101-01")
	buf.PutBytesAt(22, []byte{0x00})
	chk("1010-0101 1111-1111 0101-0100 0000-00")
	buf.PutBytesAt(14, []byte{0xFF, 0xFF})
	chk("1010-0101 1111-1111 1111-1111 1111-11")
	buf.PutBytesAt(30, nil)
	chk("1010-0101 1111-1111 1111-1111 1111-11")
	buf.PutBytesAt(0, nil)
	chk("1010-0101 1111-1111 1111-1111 1111-11")
	buf.PutBytesAt(16, []byte{0x11})
	chk("1010-0101 1111-1111 0001-0001 1111-11")

	chkpanic := func(off int, b []byte) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, ba=%#b.", off, buf)
			}
		}()
		buf.PutBytesAt(off, b)
	}
	chkpanic(-1, []byte{})
	chkpanic(-99, []byte{})
	chkpanic(31, []byte{})
	chkpanic(23, []byte{0xff})
	chkpanic(15, []byte{0xff, 0xff})
	chkpanic(7, []byte{0xff, 0xff, 0xff})
	chkpanic(0, []byte{0xff, 0xff, 0xff, 0xff})
}
