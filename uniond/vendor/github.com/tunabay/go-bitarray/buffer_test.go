// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"math/rand"
	"strings"
	"testing"
	"time"

	"github.com/tunabay/go-bitarray"
)

func TestNewBuffer(t *testing.T) {
	for i := 0; i < 64*4+2; i++ {
		buf := bitarray.NewBuffer(i)
		buf.V()
		if buf.Len() != i {
			t.Errorf("invalid Len: got %d, want %d", buf.Len(), i)
			t.Logf(" got: %s", buf.D())
		}
		if ba := buf.BitArray(); ba.OnesCount() != 0 {
			t.Errorf("invalid BitArray: %s", ba)
			t.Logf(" got: %s", buf.D())
			t.Logf("  ba: %s", ba.D())
		}
	}
	func() {
		var buf *bitarray.Buffer
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: got %s", buf.D())
			}
		}()
		buf = bitarray.NewBuffer(-1)
	}()
}

func TestNewBufferFromBitArray(t *testing.T) {
	chk := func(got *bitarray.Buffer, bawant *bitarray.BitArray) {
		got.V()
		bagot := got.BitArray()
		bagot.V()
		if !bawant.Equal(bagot) {
			t.Error("unexpected BitArray exporeted:")
			t.Logf(" got: %s", got.String())
			t.Logf(" got: %s", got.D())
			t.Logf(" got: %#b", bagot)
			t.Logf(" got: %s", bagot.D())
			t.Logf("want: %#b", bawant)
		}
		gLen, wLen := got.Len(), bawant.Len()
		if gLen != wLen {
			t.Errorf("unexpected Len: got %d, want %d", gLen, wLen)
		}
	}
	for i := 0; i < 64*4+2; i++ {
		ba := bitarray.NewZeroFilled(i)
		buf := bitarray.NewBufferFromBitArray(ba)
		chk(buf, ba)

		ba = bitarray.NewOneFilled(i)
		buf = bitarray.NewBufferFromBitArray(ba)
		chk(buf, ba)
	}
	tds := []string{
		"1100-101",
		"1100-1010",
		"1100-1010 1",
		"1100-1010 10",
		"1100-1010 1010-1111 1111-0000 1010-111",
		"1100-1010 1010-1111 1111-0000 1010-1111",
		"1100-1010 1010-1111 1111-0000 1010-1111 1",
		"1100-1010 1010-1111 1111-0000 1010-1111 11",
	}
	for _, td := range tds {
		ba := bitarray.MustParse(td)
		buf := bitarray.NewBufferFromBitArray(ba)
		chk(buf, ba)
	}

	bufn := bitarray.NewBufferFromBitArray(nil)
	switch {
	case bufn == nil:
		t.Errorf("unexpected nil buf.")
	case !bufn.IsZero() || bufn.Len() != 0:
		t.Errorf("unexpected buf: %s", bufn.D())
	}
}

func TestNewBufferFromByteSlicePartial(t *testing.T) {
	dat := []byte{0b_1111_0000, 0b_1010_0101, 0b_1100_0011, 0b_0101_1010}
	chk := func(off, nBits int, wantS string) {
		want := bitarray.MustParse(wantS)
		buf := bitarray.NewBufferFromByteSlicePartial(dat, off, nBits)
		buf.V()
		if ba := buf.BitArray(); !ba.Equal(want) {
			t.Error("unexpected buffer:")
			t.Logf(" got: %# b", ba)
			t.Logf("want: %# b", want)
			t.Logf(" buf: %s", buf.D())
		}
	}
	chk(0, 0, "")
	chk(0, 1, "1")
	chk(0, 4, "1111")
	chk(0, 7, "1111-000")
	chk(0, 8, "1111-0000")
	chk(0, 9, "1111-0000 1")
	chk(0, 15, "1111-0000 1010-010")
	chk(0, 16, "1111-0000 1010-0101")
	chk(0, 17, "1111-0000 1010-0101 1")
	chk(0, 32, "1111-0000 1010-0101 1100-0011 0101-1010")
	chk(1, 4, "111-0")
	chk(2, 4, "11-00")
	chk(3, 4, "1-000")
	chk(4, 4, "0000")
	chk(4, 6, "0000 10")
	chk(5, 5, "000 10")
	chk(6, 4, "00 10")
	chk(7, 0, "")
	chk(7, 1, "0")
	chk(7, 2, "0 1")
	chk(7, 4, "0 101")
	chk(7, 7, "0 1010-01")
	chk(7, 8, "0 1010-010")
	chk(7, 9, "0 1010-0101")
	chk(7, 10, "0 1010-0101 1")
	chk(7, 15, "0 1010-0101 1100-00")
	chk(7, 16, "0 1010-0101 1100-001")
	chk(7, 17, "0 1010-0101 1100-0011")
	chk(7, 18, "0 1010-0101 1100-0011 0")
	chk(7, 24, "0 1010-0101 1100-0011 0101-101")
	chk(7, 25, "0 1010-0101 1100-0011 0101-1010")
	chk(8, 0, "")
	chk(8, 1, "1")
	chk(8, 4, "1010")
	chk(8, 7, "1010-010")
	chk(8, 8, "1010-0101")
	chk(8, 9, "1010-0101 1")
	chk(8, 15, "1010-0101 1100-001")
	chk(8, 16, "1010-0101 1100-0011")
	chk(8, 17, "1010-0101 1100-0011 0")
	chk(31, 0, "")
	chk(31, 1, "0")
	chk(32, 0, "")
	chkpanic := func(off, nBits int) {
		var buf *bitarray.Buffer
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, nBits=%d", off, nBits)
				t.Errorf("buf: %s", buf.D())
			}
		}()
		buf = bitarray.NewBufferFromByteSlicePartial(dat, off, nBits)
	}
	chkpanic(-1, 4)
	chkpanic(0, -1)
	chkpanic(0, 33)
	chkpanic(1, 32)
	chkpanic(31, 2)
	chkpanic(32, 1)
	chkpanic(33, 0)
	chkpanic(64, 4)
}

func TestBuffer_Len_edge(t *testing.T) {
	var buf *bitarray.Buffer
	if n := buf.Len(); n != 0 {
		t.Errorf("unexpected nil.Len: got %d, want 0", n)
	}
	if !buf.IsZero() {
		t.Errorf("unexpected nil.IsZero: got false, want true")
	}
	buf = &bitarray.Buffer{}
	if n := buf.Len(); n != 0 {
		t.Errorf("unexpected zerov.Len: got %d, want 0", n)
	}
	if !buf.IsZero() {
		t.Errorf("unexpected zerov.IsZero: got false, want true")
	}
	// other normal cases are covered by TestNewBufferFromBitArray()
}

func TestBuffer_Clone(t *testing.T) {
	chk := func(buf *bitarray.Buffer, want *bitarray.BitArray) {
		t.Helper()
		ba := buf.BitArray()
		if !ba.Equal(want) {
			t.Error("unexpected:")
			t.Logf(" got: %#b", ba)
			t.Logf("data: %s", buf.D())
			t.Logf("want: %#b", want)
		}
	}
	test := func(ba *bitarray.BitArray) {
		buf := bitarray.NewBufferFromBitArray(ba)
		buf.V()
		buf2 := buf.Clone()
		buf2.V()
		chk(buf2, ba)
		buf.PutBitArrayAt(0, bitarray.NewZeroFilled(ba.Len()))
		chk(buf2, ba)
		buf.PutBitArrayAt(0, bitarray.NewOneFilled(ba.Len()))
		chk(buf2, ba)
	}
	for i := 0; i < 130; i++ {
		ba := bitarray.NewZeroFilled(i).ZOptimize()
		test(ba)
		test(ba.ZExpand())
		ba = bitarray.NewOneFilled(i)
		test(ba)
	}
}

// tests BitArray, Len, Clone, String
func TestBuffer_rand(t *testing.T) {
	const testIterations = 30000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		var nBits int
		switch rand.Intn(10) {
		case 0:
			nBits = rand.Intn(66)
		case 1:
			nBits = 8*(1+rand.Intn(32)) - 1 + rand.Intn(3)
		case 2:
			nBits = 256 + rand.Intn(2048)
		default:
			nBits = rand.Intn(256)
		}
		ba := bitarray.PseudoRand(nBits, nil)
		buf := bitarray.NewBufferFromBitArray(ba)
		buf.V()
		got := buf.BitArray()
		got.V()
		if !got.Equal(ba) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", ba)
		}
		buf2 := buf.Clone()
		buf2.V()
		got = buf2.BitArray()
		if !got.Equal(ba) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", ba)
		}
		gotS := buf.String()
		wantS := ba.String()
		if gotS != wantS {
			t.Error("unexpected result:")
			t.Logf(" got: %s", gotS)
			t.Logf("want: %s", wantS)
			t.Logf("data: %s", buf.D())
		}
		if buf.Len() != ba.Len() {
			t.Errorf("unexpected len: got %d, want %d", buf.Len(), ba.Len())
		}
	}
}

func TestBuffer_Resize_edge(t *testing.T) {
	chk := func(gotBuf *bitarray.Buffer, wantS string) {
		t.Helper()
		gotBuf.V()
		got := gotBuf.BitArray()
		want := bitarray.MustParse(wantS)
		if !got.Equal(want) {
			t.Error("unexpected:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
		}
	}
	var nilba *bitarray.BitArray
	buf := bitarray.NewBufferFromBitArray(nilba)
	buf.V()
	buf.Resize(20, bitarray.AlignLeft)
	buf.V()
	buf.PutByteAt(4, 0xff)
	chk(buf, "0000-1111 1111-0000 0000")

	buf.Resize(18, bitarray.AlignRight)
	buf.V()
	buf.FillBitsAt(12, 2, 1)
	chk(buf, "00-1111 1111-0011 0000")

	buf.Resize(12, bitarray.AlignLeft)
	chk(buf, "00-1111 1111-00")

	buf.Resize(1, bitarray.AlignLeft)
	chk(buf, "0")

	buf.Resize(19, bitarray.AlignLeft)
	chk(buf, "0000-0000 0000-0000 000")

	buf.Resize(0, bitarray.AlignLeft)
	chk(buf, "")

	buf.Resize(15, bitarray.AlignLeft)
	chk(buf, "0000-0000 0000-000")

	// negative size should cause a panic
	func() {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: got %s", buf.D())
			}
		}()
		buf.Resize(-1, bitarray.AlignLeft)
	}()
}

func TestBuffer_Resize_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())
	rndLen := func() int {
		switch rand.Intn(5) {
		case 0:
			return rand.Intn(66)
		case 1:
			return 8*(1+rand.Intn(32)) - 1 + rand.Intn(3)
		case 2:
			return 256 + rand.Intn(2048)
		}
		return rand.Intn(256)
	}
	rndAlign := func() bitarray.Alignment {
		if rand.Intn(2) == 0 {
			return bitarray.AlignLeft
		}
		return bitarray.AlignRight
	}
	for i := 0; i < testIterations/10; i++ {
		nBits := rndLen()
		ba := bitarray.PseudoRand(nBits, nil)
		buf := bitarray.NewBufferFromBitArray(ba)
		buf.V()
		for j := 0; j < 10; j++ {
			oldNBits := buf.Len()
			oldS := buf.String()
			oldD := buf.D()
			newNBits := rndLen()
			align := rndAlign()
			newS := oldS
			switch {
			case align == bitarray.AlignLeft && newNBits < oldNBits:
				newS = oldS[:newNBits]
			case align == bitarray.AlignLeft && oldNBits < newNBits:
				newS = oldS + strings.Repeat("0", newNBits-oldNBits)
			case align == bitarray.AlignRight && newNBits < oldNBits:
				newS = oldS[oldNBits-newNBits:]
			case align == bitarray.AlignRight && oldNBits < newNBits:
				newS = strings.Repeat("0", newNBits-oldNBits) + oldS
			}

			buf.Resize(newNBits, align)
			buf.V()
			if n := buf.Len(); n != newNBits {
				t.Errorf("unexpected len: got %d, want %d", n, newNBits)
			}
			gotS := buf.String()
			if gotS != newS {
				t.Errorf("unexpected result: %d -> %d", oldNBits, newNBits)
				t.Logf(" got: %s", gotS)
				t.Logf("want: %s", newS)
				t.Logf(" old: %s", oldS)
				t.Logf(" old: %s", oldD)
			}
			// if i < 32 {
			// 	t.Logf("%3d: %3d: %4d -> %4d:", i, j, oldNBits, newNBits)
			// }
		}
	}
}

func TestBuffer_FillBitsAt(t *testing.T) {
	chk := func(gotBuf *bitarray.Buffer, wantS string) {
		t.Helper()
		gotBuf.V()
		got := gotBuf.BitArray()
		want := bitarray.MustParse(wantS)
		if !got.Equal(want) {
			t.Error("unexpected:")
			t.Logf(" got: %#b", got)
			t.Logf(" got: %s", got.D())
			t.Logf("want: %#b", want)
		}
	}
	buf := bitarray.NewBuffer(20)
	chk(buf, "0000-0000 0000-0000 0000")
	buf.FillBitsAt(0, 10, 1)
	chk(buf, "1111-1111 1100-0000 0000")
	buf.FillBitsAt(4, 8, 0)
	chk(buf, "1111-0000 0000-0000 0000")
	buf.FillBitsAt(6, 8, 1)
	chk(buf, "1111-0011 1111-1100 0000")
	buf.FillBitsAt(12, 4, 1)
	chk(buf, "1111-0011 1111-1111 0000")
	buf.FillBitsAt(10, 6, 0)
	chk(buf, "1111-0011 1100-0000 0000")
	buf.FillBitsAt(2, 18, 1)
	chk(buf, "1111-1111 1111-1111 1111")
	buf.FillBitsAt(8, 8, 0)
	chk(buf, "1111-1111 0000-0000 1111")
	buf.FillBitsAt(0, 0, 0)
	chk(buf, "1111-1111 0000-0000 1111")
	buf.FillBitsAt(20, 0, 0)
	chk(buf, "1111-1111 0000-0000 1111")

	chkpanic := func(off, nBits int, bit byte) {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: off=%d, nBits=%d, bit=%d.", off, nBits, bit)
			}
		}()
		buf.FillBitsAt(off, nBits, bit)
	}
	chkpanic(-1, 4, 1)
	chkpanic(0, -1, 1)
	chkpanic(16, 8, 1)
	chkpanic(99, 8, 1)
	chkpanic(0, 99, 1)
}
