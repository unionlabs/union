// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"errors"
	"fmt"
	"math/rand"
	"testing"
	"testing/iotest"
	"time"

	"github.com/tunabay/go-bitarray"
)

func TestNewBuilder(t *testing.T) {
	bb1 := bitarray.NewBuilder(
		bitarray.NewZeroFilled(10),
		nil,
		bitarray.NewOneFilled(10),
		bitarray.New(),
		bitarray.MustParse("0101"),
		bitarray.NewZeroFilled(10).ZExpand(),
		bitarray.NewOneFilled(10),
		nil,
	)
	got1 := bb1.BitArray()
	got1.V()
	want1 := bitarray.MustParse("0000000000 1111111111 0101 0000000000 1111111111")
	if !got1.Equal(want1) {
		t.Error("unexpected result:")
		t.Logf(" got: %#b", got1)
		t.Logf(" got: %s", got1.D())
		t.Logf("want: %#b", want1)
	}

	bb2 := bitarray.NewBuilder()
	got2 := bb2.BitArray()
	got2.V()
	if !got2.IsZero() {
		t.Error("unexpected result: want zero")
		t.Logf(" got: %#b", got1)
		t.Logf(" got: %s", got1.D())
	}
}

func TestBuilder_BitArray(t *testing.T) {
	b1 := []byte{0b_1111_0000, 0b_1010_1010, 0b_0000_1111, 0b_1100_1100}
	bb1 := bitarray.NewBuilder(
		nil,
		bitarray.NewZeroFilled(5),           // 00000
		bitarray.NewOneFilled(5),            // 11111
		bitarray.NewByRunLength(1, 2, 3, 4), // 0110001111
	)
	bb1.A(b1, 5, 0, false)   // (empty)
	bb1.A(b1, 2, 5, false)   // 11000
	bb1.A(b1, 22, 10, false) // 11 1100 1100
	bb1.A(b1, 4, 4, true)    // 0000
	got1 := bb1.BitArray()
	got1.V()
	want1 := bitarray.MustParse("00000 11111 0110001111 11000 11 1100 1100 0000")
	if !got1.Equal(want1) {
		t.Error("unexpected result:")
		t.Logf(" got: %#b", got1)
		t.Logf(" got: %s", got1.D())
		t.Logf("want: %#b", want1)
	}
	if bb1.String() != want1.String() {
		t.Error("unexpected result:")
		t.Logf(" got: %s", bb1.String())
		t.Logf("want: %s", want1.String())
	}
	if bb1.Len() != want1.Len() {
		t.Errorf("unexpected Len: got %d, want %d", bb1.Len(), want1.Len())
	}
}

func TestBuilder_String(t *testing.T) {
	b1 := []byte{0b_1111_0000, 0b_1010_1010, 0b_0000_1111, 0b_1100_1100}
	bb1 := bitarray.NewBuilder(
		nil,
		bitarray.NewZeroFilled(7),           // 0000000
		bitarray.NewOneFilled(7),            // 1111111
		bitarray.NewByRunLength(4, 3, 2, 1), // 0000111001
	)
	bb1.A(b1, 15, 0, false) // (empty)
	bb1.A(b1, 6, 4, false)  // 0010
	bb1.A(b1, 6, 14, false) // 00 10101010 0000
	bb1.A(b1, 12, 8, false) // 10100000
	bb1.A(b1, 16, 9, false) // 00001111 1
	bb1.A(b1, 4, 4, true)   // 0000
	got1 := bb1.String()
	want1 := "000000011111110000111001001000101010100000101000000000111110000"
	if got1 != want1 {
		t.Error("unexpected result:")
		t.Logf(" got: %s", got1)
		t.Logf("want: %s", want1)
	}
	if bb1.Len() != len(want1) {
		t.Errorf("unexpected Len: got %d, want %d", bb1.Len(), len(want1))
	}
}

func TestBuilder_WriteByteBits_rand(t *testing.T) {
	const (
		testIterations = 1000
		testAppends    = 30
	)
	var (
		bb   bitarray.Builder
		want string
	)

	appendB := func(n int) {
		bits := make([]byte, n)
		for i := 0; i < n; i++ {
			bits[i] = byte(rand.Intn(2))
			want += string([]byte{'0' + bits[i]})
		}
		bb.WriteByteBits(bits)
	}

	for i := 0; i < testIterations; i++ {
		bb.Reset()
		want = ""
		for j := 0; j < testAppends; j++ {
			appendB(rand.Intn(128))
			gsd := bb.String()
			gsa := bb.BitArray().String()
			if gsd != want || gsa != want {
				t.Errorf("unexpected result:")
				t.Logf("  got(direct): %s", gsd)
				t.Logf(" got(convert): %s", gsa)
				t.Logf("         want: %s", want)
			}
		}
		// t.Logf("pass: %s", bb.String())
	}
}

func TestBuilder_WriteBitsFromBytes_rand(t *testing.T) {
	const (
		testIterations = 2000
		testAppends    = 50
	)
	var (
		bb   bitarray.Builder
		want string
	)
	hist := make([]string, 0, 100)

	logf := func(f string, p ...interface{}) {
		hist = append(hist, fmt.Sprintf(f, p...))
	}
	appendB := func() {
		off := rand.Intn(32)
		nBits := rand.Intn(32)
		logf("append: %d bits, off %d", nBits, off)
		nBytes := (off + nBits + 7) >> 3
		buf := make([]byte, nBytes)
		for i := off; i < off+nBits; i++ {
			b := byte(rand.Intn(2)) & 1
			iby := i >> 3
			ibi := i & 7
			buf[iby] |= b << (7 - ibi)
			want += string([]byte{'0' + b})
		}
		logf("append: %08b", buf)
		bb.WriteBitsFromBytes(buf, off, nBits)
	}

	for i := 0; i < testIterations; i++ {
		bb.Reset()
		want = ""
		hist = hist[:0]
		for j := 0; j < testAppends; j++ {
			appendB()
			gsd := bb.String()
			gsa := bb.BitArray().String()
			if gsd != want || gsa != want {
				t.Errorf("unexpected result:")
				t.Logf("  got(direct): %s", gsd)
				t.Logf(" got(convert): %s", gsa)
				t.Logf("         want: %s", want)
				t.Logf("history:")
				for k, h := range hist {
					t.Logf("#%3d: %s", k, h)
				}
				t.FailNow()
			}
		}
		// t.Logf("pass: %s", bb.String())
	}
}

func TestBuilder_Write(t *testing.T) {
	buf := []byte{0xff, 0x00, 0xcc, 0x33, 0xaa, 0x55, 0xee, 0x77}
	bb := bitarray.NewBuilder(bitarray.NewOneFilled(2))
	wantS := "11"
	add := func(off, n int) {
		sb := buf[off : off+n]
		for _, b := range sb {
			wantS += fmt.Sprintf("%08b", b)
		}
		wn, err := bb.Write(sb)
		if wn != n || err != nil {
			t.Errorf("Write: n=%d, want=%d, err=%s", wn, n, err)
		}
	}
	add(3, 0)
	add(0, 3)
	add(0, 8)
	add(1, 6)
	add(7, 1)
	add(5, 2)

	if gotS := bb.String(); gotS != wantS {
		t.Error("unexpected result:")
		t.Logf(" got: %s", gotS)
		t.Logf("want: %s", wantS)
	}
	if bb.Len() != len(wantS) {
		t.Errorf("unexpected result: got %d, want %d", bb.Len(), len(wantS))
	}
	gotB := bb.BitArray()
	gotB.V()
	wantB := bitarray.MustParse(wantS)
	if !gotB.Equal(wantB) {
		t.Error("unexpected result:")
		t.Logf(" got: %#b", gotB)
		t.Logf(" got: %s", gotB.D())
		t.Logf("want: %#b", wantB)
	}
}

func TestBuilder_WriteByte_zero(t *testing.T) {
	const nBytes = 123
	nBits := nBytes << 3

	bb := bitarray.NewBuilder()
	for i := 0; i < nBytes; i++ {
		if err := bb.WriteByte(0); err != nil {
			t.Fatalf("WriteByte: %s", err)
		}
	}
	wantB := bitarray.NewZeroFilled(nBits)
	wantS := wantB.String()
	if n := bb.Len(); n != nBits {
		t.Errorf("unexpeted Len: got %d, want %d", n, nBits)
	}
	if gotS := bb.String(); gotS != wantS {
		t.Error("unexpected result:")
		t.Logf(" got: %s", gotS)
		t.Logf("want: %s", wantS)
	}
	gotB := bb.BitArray()
	gotB.V()
	if !gotB.Equal(wantB) {
		t.Error("unexpected result:")
		t.Logf(" got: %#b", gotB)
		t.Logf(" got: %s", gotB.D())
		t.Logf("want: %#b", wantB)
	}
}

func TestBuilder_WriteByte_rand(t *testing.T) {
	const testIterations = 10000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		bb := bitarray.NewBuilder()
		wantS := ""
		maxAdds := 16
		if rand.Intn(5) == 0 {
			maxAdds = 256
		}
		nadds := rand.Intn(maxAdds)
		for j := 0; j < nadds; j++ {
			if rand.Intn(5) == 0 {
				nBits := 1 + rand.Intn(7)
				for k := 0; k < nBits; k++ {
					bit := byte(rand.Intn(2))
					if err := bb.WriteBit(bit); err != nil {
						t.Fatalf("WriteBit: %s", err)
					}
					wantS += fmt.Sprintf("%b", bit)
				}
				continue
			}
			b := byte(rand.Intn(0x100))
			if err := bb.WriteByte(b); err != nil {
				t.Fatalf("WriteByte: %s", err)
			}
			wantS += fmt.Sprintf("%08b", b)
		}
		if gotS := bb.String(); gotS != wantS {
			t.Error("unexpected result:")
			t.Logf(" got: %s", gotS)
			t.Logf("want: %s", wantS)
		}
		if bb.Len() != len(wantS) {
			t.Errorf("unexpected len: got %d, want %d", bb.Len(), len(wantS))
		}
		gotB := bb.BitArray()
		gotB.V()
		wantB := bitarray.MustParse(wantS)
		if !gotB.Equal(wantB) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", gotB)
			t.Logf(" got: %s", gotB.D())
			t.Logf("want: %#b", wantB)
		}
		// if i < 32 {
		// 	t.Logf("pass: %#b", gotB)
		// }
	}
}

func TestBuilder_ReadFrom(t *testing.T) {
	b := []byte{0b_1111_1010, 0b_0000_0101, 0b_1110_0111, 0b_1100_0011}
	r := iotest.OneByteReader(bytes.NewBuffer(b))

	bb := bitarray.NewBuilder(bitarray.NewZeroFilled(3))
	rn, err := bb.ReadFrom(r)
	if err != nil {
		t.Errorf("ReadFrom: %s", err)
	}
	if rn != 4 {
		t.Errorf("unexpected length read: got %d, want 4", rn)
	}
	bb.WriteByteBits([]byte{0, 1})

	gotS := bb.String()
	gotB := bb.BitArray()
	gotB.V()

	wantS := "0001111101000000101111001111100001101"
	wantB := bitarray.MustParse(wantS)

	if gotS != wantS {
		t.Error("unexpected result:")
		t.Logf(" got: %s", gotS)
		t.Logf("want: %s", wantS)
	}
	if bb.Len() != len(wantS) {
		t.Errorf("unexpected len: got %d, want %d", bb.Len(), len(wantS))
	}
	if !gotB.Equal(wantB) {
		t.Error("unexpected result:")
		t.Logf(" got: %#b", gotB)
		t.Logf(" got: %s", gotB.D())
		t.Logf("want: %#b", wantB)
	}
}

func TestBuilder_ReadFrom_error(t *testing.T) {
	myErr := errors.New("test")
	r := iotest.ErrReader(myErr)

	bb := bitarray.NewBuilder(bitarray.NewZeroFilled(100))
	rn, err := bb.ReadFrom(r)
	if err != nil {
		if !errors.Is(err, myErr) {
			t.Errorf("ReadFrom: unexpected error: %s", err)
		}
	} else {
		t.Errorf("ReadFrom: expected error, but no error.")
	}
	if rn != 0 {
		t.Errorf("unexpected length read: got %d, want 0", rn)
	}
}

func TestBuilder_WriteBitArray(t *testing.T) {
	bas := []*bitarray.BitArray{
		nil, // overwritten
		nil,
		bitarray.New(),
		bitarray.MustParse("1111-00"),
		bitarray.NewZeroFilled(200),
		bitarray.NewZeroFilled(200).ZExpand(),
	}
	for i, ba := range bas {
		bb := bitarray.NewBuilder()
		var wn int
		var err error
		if i == 0 {
			wn, err = bb.WriteBitArray(nil) // untyped
		} else {
			wn, err = bb.WriteBitArray(ba)
		}
		if wn != ba.Len() {
			t.Errorf("unexpected length: got %d, want %d", wn, ba.Len())
		}
		if err != nil {
			t.Errorf("WriteBitArray: %s", err)
		}
		if bb.Len() != ba.Len() {
			t.Errorf("unexpected length: got %d, want %d", bb.Len(), ba.Len())
		}
		wantS := ""
		if ba != nil {
			wantS = ba.String()
		}
		if gotS := bb.String(); gotS != wantS {
			t.Error("unexpected result:")
			t.Logf(" got: %s", gotS)
			t.Logf("want: %s", wantS)
		}
		gotB := bb.BitArray()
		gotB.V()
		if !gotB.Equal(ba) {
			t.Error("unexpected result:")
			t.Logf(" got: %#b", gotB)
			t.Logf(" got: %s", gotB.D())
			t.Logf("want: %#b", ba)
		}
	}
}

func TestBuilder_WriteBitArray_multi(t *testing.T) {
	var nilba *bitarray.BitArray
	bas := []bitarray.BitArrayer{
		bitarray.New(),
		bitarray.NewZeroFilled(30).ZOptimize(),
		bitarray.NewByRunLength(0, 7, 5, 3), // 111111100000111
		bitarray.NewZeroFilled(20).ZExpand(),
		bitarray.NewOneFilled(10).ZExpand(),
		nilba,
		bitarray.MustParse("0101-1111 1010-0111"),
		bitarray.MustParse("0"),
	}

	bb := bitarray.NewBuilder()
	for _, bai := range bas {
		var nBits int
		if bai != nil {
			ba := bai.BitArray()
			nBits = ba.Len()
		}
		wn, err := bb.WriteBitArray(bai)
		if err != nil {
			t.Errorf("WriteBitArray: %s", err)
		}
		if wn != nBits {
			t.Errorf("WriteBitArray: unexpected len: got %d, want %d", wn, nBits)
		}
	}

	gotS := bb.String()
	wantS := "000000000000000000000000000000" + "111111100000111" +
		"00000000000000000000" + "1111111111" +
		"01011111" + "10100111" + "0"
	if gotS != wantS {
		t.Error("unexpected result:")
		t.Logf(" got: %s", gotS)
		t.Logf("want: %s", wantS)
	}
	if bb.Len() != len(wantS) {
		t.Errorf("unexpected result: got %d, want %d", bb.Len(), len(wantS))
	}

	gotB := bb.BitArray()
	gotB.V()
	wantB := bitarray.MustParse(wantS)
	if !gotB.Equal(wantB) {
		t.Error("unexpected result:")
		t.Logf(" got: %#b", gotB)
		t.Logf(" got: %s", gotB.D())
		t.Logf("want: %#b", wantB)
	}
}

func TestBuilder_WriteBits(t *testing.T) {
	srcBA := bitarray.MustParse("1111-0000 1100-0011").Repeat(10)
	srcBuf := bitarray.NewBufferFromBitArray(srcBA)
	wantS := ""
	bb := bitarray.NewBuilder()

	add := func(buf *bitarray.Buffer) {
		t.Helper()
		wantS += buf.String()
		n, err := bb.WriteBits(buf)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}
		if n != buf.Len() {
			t.Fatalf("unexpected n: got %d, want %d", n, buf.Len())
		}
		if got, want := bb.BitArray(), bitarray.MustParse(wantS); !got.Equal(want) {
			t.Errorf("unexpected result:")
			t.Logf(" got: %#b", got)
			t.Logf("want: %#b", want)
		}
	}
	for i := 0; i < 35; i++ {
		for j := 0; j < 35; j++ {
			add(srcBuf.Slice(i, i+j))
		}
	}
	srcBuf.FillBits(0)

	if got, want := bb.BitArray(), bitarray.MustParse(wantS); !got.Equal(want) {
		t.Errorf("unexpected result:")
		t.Logf(" got: %#b", got)
		t.Logf("want: %#b", want)
	}
}
