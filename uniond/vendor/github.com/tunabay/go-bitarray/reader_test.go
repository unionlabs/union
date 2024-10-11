// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"errors"
	"io"
	"math/rand"
	"testing"
	"testing/iotest"
	"time"

	"github.com/tunabay/go-bitarray"
)

func TestReader_byteAligned(t *testing.T) {
	r := bitarray.NewReader(nil)
	if err := iotest.TestReader(r, []byte{}); err != nil {
		t.Errorf("TestReader: %+v", err)
	}

	var nilba *bitarray.BitArray
	r = bitarray.NewReader(nilba)
	if err := iotest.TestReader(r, []byte{}); err != nil {
		t.Errorf("TestReader: %+v", err)
	}

	r = bitarray.NewReader(bitarray.New())
	if err := iotest.TestReader(r, []byte{}); err != nil {
		t.Errorf("TestReader: %+v", err)
	}

	r = bitarray.NewReader(bitarray.NewZeroFilled(8))
	if err := iotest.TestReader(r, []byte{0}); err != nil {
		t.Errorf("TestReader: %+v", err)
	}

	r = bitarray.NewReader(bitarray.NewOneFilled(8))
	if err := iotest.TestReader(r, []byte{0xff}); err != nil {
		t.Errorf("TestReader: %+v", err)
	}

	r = bitarray.NewReader(bitarray.NewZeroFilled(8 * 256))
	if err := iotest.TestReader(r, bytes.Repeat([]byte{0}, 256)); err != nil {
		t.Errorf("TestReader: %+v", err)
	}

	r = bitarray.NewReader(bitarray.NewOneFilled(8 * 256))
	if err := iotest.TestReader(r, bytes.Repeat([]byte{0xff}, 256)); err != nil {
		t.Errorf("TestReader: %+v", err)
	}
}

func TestReader_byteAlignedRand(t *testing.T) {
	const testIterations = 5000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		var nBits int
		switch rand.Intn(10) {
		case 0:
			nBits = 1 + rand.Intn(64+1)
		case 1:
			nBits = 8*(1+rand.Intn(24)) - 1 + rand.Intn(3)
		case 2:
			nBits = 256 + rand.Intn(2048)
		default:
			nBits = 1 + rand.Intn(256)
		}
		ba := bitarray.PseudoRand(nBits, nil)
		ba = ba.ToPadded8()
		wantB, npad := ba.Bytes()
		if npad != 0 {
			t.Fatalf("unexpected npad=%d", npad)
		}
		r := bitarray.NewReader(ba)

		if err := iotest.TestReader(r, wantB); err != nil {
			t.Fatalf("TestReader: %+v", err)
		}
	}
}

func TestReader_Reset_edge(t *testing.T) {
	ba := bitarray.MustParse("1010-1010")
	r := bitarray.NewReader(ba)

	b, err := r.ReadBit()
	if b != 1 || err != nil {
		t.Errorf("unexpected: %d, %v", b, err)
	}
	r.Reset(nil)
	b, err = r.ReadBit()
	if !errors.Is(err, io.EOF) {
		t.Errorf("unexpected: %d, %v", b, err)
	}
}

func TestReader_Read(t *testing.T) {
	ba := bitarray.MustParse("0011-1111 0000-1111 1010-1010 1100-1100 11")

	// full + 2 bits
	r := bitarray.NewReader(ba)
	gotB, err := io.ReadAll(r)
	if err == nil || !errors.Is(err, bitarray.ErrFractionalBitsBeforeEOF) {
		t.Errorf("unexpected err: got %s", err)
	}
	if !bytes.Equal(gotB, []byte{0b_0011_1111, 0b_0000_1111, 0b_1010_1010, 0b_1100_1100}) {
		t.Errorf("unexpected data read: %b", gotB)
	}
	tail2ba, err := r.ReadBitArray(99)
	if err != nil {
		t.Errorf("unexpected err: got %s", err)
	}
	if !tail2ba.Equal(bitarray.MustParse("11")) {
		t.Errorf("unexpected fractional bits read: got %#b", tail2ba)
	}
	buf1 := make([]byte, 1)
	rn, err := io.ReadFull(r, buf1)
	if rn != 0 || err == nil || !errors.Is(err, io.EOF) {
		t.Errorf("unexpected read: n=%d, err=%v", rn, err)
	}

	// 4 bits + 3 bytes + 2 bits
	r = bitarray.NewReader(ba)
	head4ba, err := r.ReadBitArray(4)
	if err != nil {
		t.Errorf("unexpected err: got %s", err)
	}
	if !head4ba.Equal(bitarray.MustParse("0011")) {
		t.Errorf("unexpected leading bits read: got %#b", head4ba)
	}
	buf3 := make([]byte, 3)
	rn, err = io.ReadFull(r, buf3)
	if rn != 3 || err != nil {
		t.Errorf("unexpected read: n=%d, err=%v", rn, err)
		t.Logf("%b", buf3)
	}
	if !bytes.Equal(buf3, []byte{0b_1111_0000, 0b_1111_1010, 0b_1010_1100}) {
		t.Errorf("unexpected data read: %b", buf3)
	}
	rn, err = io.ReadFull(r, buf1)
	if rn != 0 || err == nil || !errors.Is(err, bitarray.ErrFractionalBitsBeforeEOF) {
		t.Errorf("unexpected read: n=%d, err=%v", rn, err)
	}

	tail6ba, err := r.ReadBitArray(6)
	if err != nil {
		t.Errorf("unexpected err: got %s", err)
	}
	if !tail6ba.Equal(bitarray.MustParse("110011")) {
		t.Errorf("unexpected fractional bits read: got %#b", tail6ba)
	}
	rn, err = io.ReadFull(r, buf1)
	if rn != 0 || err == nil || !errors.Is(err, io.EOF) {
		t.Errorf("unexpected read: n=%d, err=%v", rn, err)
	}

	// 2 bits + full
	r = bitarray.NewReader(ba)
	head2b, err := r.ReadBit()
	if err != nil {
		t.Errorf("unexpected err: got %s", err)
	}
	if head2b != 0 {
		t.Errorf("unexpected bit[0]: got %d, want 0", head2b)
	}
	head2b, err = r.ReadBit()
	if err != nil {
		t.Errorf("unexpected err: got %s", err)
	}
	if head2b != 0 {
		t.Errorf("unexpected bit[1]: got %d, want 0", head2b)
	}
	gotB, err = io.ReadAll(r)
	if err != nil {
		t.Errorf("unexpected err: got %s", err)
	}
	if !bytes.Equal(gotB, []byte{0b_1111_1100, 0b_0011_1110, 0b_1010_1011, 0b_0011_0011}) {
		t.Errorf("unexpected data read: %b", gotB)
	}
	rn, err = io.ReadFull(r, buf1)
	if rn != 0 || err == nil || !errors.Is(err, io.EOF) {
		t.Errorf("unexpected read: n=%d, err=%v", rn, err)
	}
}

func TestReader_ReadByte(t *testing.T) {
	ba := bitarray.MustParse("0011-1111 0000-1111 1010-1010 1100-1100 11")

	// bytes + 2 bits
	r := bitarray.NewReader(ba)
	bs := make([]byte, 4)
	for i := range bs {
		b, err := r.ReadByte()
		if err != nil {
			t.Errorf("unexpected err: i=%d: %s", i, err)
		}
		bs[i] = b
	}
	if !bytes.Equal(bs, []byte{0b_0011_1111, 0b_0000_1111, 0b_1010_1010, 0b_1100_1100}) {
		t.Errorf("unexpected data read: %b", bs)
	}
	b, err := r.ReadByte()
	switch {
	case err == nil:
		t.Errorf("error expected, no error: %08b", b)
	case !errors.Is(err, bitarray.ErrFractionalBitsBeforeEOF):
		t.Errorf("unexpected error: %s", err)
	}
	tail2ba, err := r.ReadBitArray(99)
	if err != nil {
		t.Errorf("unexpected error: %s", err)
	}
	if !tail2ba.Equal(bitarray.MustParse("11")) {
		t.Errorf("unexpected data read: got %#b, want 11", tail2ba)
	}

	// 2 bits + bytes
	r = bitarray.NewReader(ba)
	head2ba, err := r.ReadBitArray(2)
	if err != nil {
		t.Errorf("unexpected error: %s", err)
	}
	if !head2ba.Equal(bitarray.MustParse("00")) {
		t.Errorf("unexpected data read: got %#b, want 00", head2ba)
	}
	bs = make([]byte, 5)
	for i := range bs {
		b, err := r.ReadByte()
		if err != nil {
			if errors.Is(err, io.EOF) {
				break // expected
			}
			t.Errorf("unexpected error: %s", err)
		}
		bs[i] = b
		if i == 4 {
			t.Errorf("expected error, no error: %08b", bs[i])
		}
	}
	if !bytes.Equal(bs[:4], []byte{0b_1111_1100, 0b_0011_1110, 0b_1010_1011, 0b_0011_0011}) {
		t.Errorf("unexpected data read: %b", bs[:4])
	}

	// zero filled + 3
	ba = bitarray.MustParse("0000-0000 0000-0000 0000-0000 000")
	r = bitarray.NewReader(ba)
	for i := 0; i < 3; i++ {
		bit, err := r.ReadBit()
		switch {
		case err != nil:
			t.Errorf("unexpected error: %s", err)
		case bit != 0:
			t.Errorf("unexpected bit: got %d, want 1", bit)
		}
	}
	for i := 0; i < 3; i++ {
		b, err := r.ReadByte()
		switch {
		case err != nil:
			t.Errorf("unexpected error: %s", err)
		case b != 0:
			t.Errorf("unexpected byte: got %08b, want 0", b)
		}
	}
	b, err = r.ReadByte()
	switch {
	case err == nil:
		t.Errorf("expected error, no error: got %08b", b)
	case !errors.Is(err, io.EOF):
		t.Errorf("unexpected error: %s", err)
	}

	// nil ba
	var nilba *bitarray.BitArray
	r = bitarray.NewReader(nilba)
	b, err = r.ReadByte()
	switch {
	case err == nil:
		t.Errorf("error expected, no error: %08b", b)
	case !errors.Is(err, io.EOF):
		t.Errorf("unexpected error: %s", err)
	}
}

func TestReader_ReadBitArray(t *testing.T) {
	ba := bitarray.MustParse("0011-1111 0000-1111 1010-1010 1100-1100 11")

	r := bitarray.NewReader(ba)
	b, err := r.ReadBitArray(6)
	switch {
	case err != nil:
		t.Errorf("unexpected error: %s", err)
	case !b.Equal(bitarray.MustParse("001111")):
		t.Errorf("unexpected read: got %#b, want 0011-11", b)
	}

	b, err = r.ReadBitArray(0)
	switch {
	case err != nil:
		t.Errorf("unexpected error: %s", err)
	case !b.IsZero():
		t.Errorf("unexpected read: got %#b, want zero", b)
	}

	func() {
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: got %#b", b)
			}
		}()
		b, err = r.ReadBitArray(-1)
		t.Logf("err: %s", err)
	}()

	b, err = r.ReadBitArray(16)
	switch {
	case err != nil:
		t.Errorf("unexpected error: %s", err)
	case !b.Equal(bitarray.MustParse("11 0000-1111 1010-10")):
		t.Errorf("unexpected read: got %#b, want 1100-0011 1110-1010", b)
	}

	b, err = r.ReadBitArray(1)
	switch {
	case err != nil:
		t.Errorf("unexpected error: %s", err)
	case !b.Equal(bitarray.MustParse("1")):
		t.Errorf("unexpected read: got %#b, want 1", b)
	}

	b, err = r.ReadBitArray(99)
	switch {
	case err != nil:
		t.Errorf("unexpected error: %s", err)
	case !b.Equal(bitarray.MustParse("0 1100-1100 11")):
		t.Errorf("unexpected read: got %#b, want 0110-0110 011", b)
	}

	b, err = r.ReadBitArray(8)
	switch {
	case err == nil:
		t.Errorf("expected error, no error: got %#b", b)
	case !errors.Is(err, io.EOF):
		t.Errorf("unexpected error: %s", err)
	}

	var nilba *bitarray.BitArray
	r = bitarray.NewReader(nilba)
	b, err = r.ReadBitArray(99)
	switch {
	case err == nil:
		t.Errorf("expected error, no error: got %#b", b)
	case !errors.Is(err, io.EOF):
		t.Errorf("unexpected error: %s", err)
	}
}

func TestReader_ReadBit(t *testing.T) {
	ba := bitarray.MustParse("0011-1111 0000-1111 1010-1010 1100-110")
	r := bitarray.NewReader(ba)
	for i := 0; i < ba.Len(); i++ {
		bit, err := r.ReadBit()
		switch {
		case err != nil:
			t.Errorf("unexpected error: %d: %s", i, err)
		case bit != ba.BitAt(i):
			t.Errorf("unexpected read: %d: got %d, want %d", i, bit, ba.BitAt(i))
		}
	}
	bit, err := r.ReadBit()
	switch {
	case err == nil:
		t.Errorf("expected error, no error: got %d", bit)
	case !errors.Is(err, io.EOF):
		t.Errorf("unexpected error: %s", err)
	}

	ba = bitarray.NewZeroFilled(258)
	r = bitarray.NewReader(ba)
	for i := 0; i < ba.Len(); i++ {
		bit, err := r.ReadBit()
		switch {
		case err != nil:
			t.Errorf("unexpected error: %d: %s", i, err)
		case bit != 0:
			t.Errorf("unexpected read: %d: got %d, want 0", i, bit)
		}
	}
	bit, err = r.ReadBit()
	switch {
	case err == nil:
		t.Errorf("expected error, no error: got %d", bit)
	case !errors.Is(err, io.EOF):
		t.Errorf("unexpected error: %s", err)
	}

	var nilba *bitarray.BitArray
	r = bitarray.NewReader(nilba)
	bit, err = r.ReadBit()
	switch {
	case err == nil:
		t.Errorf("expected error, no error: got %d", bit)
	case !errors.Is(err, io.EOF):
		t.Errorf("unexpected error: %s", err)
	}
}

func TestReader_ReadBits_rand(t *testing.T) {
	const testIterations = 200000
	rand.Seed(time.Now().UnixNano())
	var r *bitarray.Reader
	for i := 0; i < testIterations; i++ {
		var nBits int
		switch rand.Intn(10) {
		case 0:
			nBits = 1 + rand.Intn(64+1)
		case 1:
			nBits = 8*(1+rand.Intn(24)) - 1 + rand.Intn(3)
		case 2:
			nBits = 256 + rand.Intn(2048)
		default:
			nBits = 1 + rand.Intn(256)
		}
		var ba *bitarray.BitArray
		switch rand.Intn(32) {
		case 0:
			ba = bitarray.NewZeroFilled(nBits)
		case 1:
			ba = bitarray.NewOneFilled(nBits)
		default:
			ba = bitarray.PseudoRand(nBits, nil)
		}

		var bufLen int
		switch rand.Intn(10) {
		case 0:
			bufLen = 1 + rand.Intn(8)
		case 1:
			bufLen = 64 + rand.Intn(128+8)
		case 2:
			bufLen = 8*(1+rand.Intn(8)) - 1 + rand.Intn(3)
		default:
			bufLen = 1 + rand.Intn(256)
		}
		buf := bitarray.NewBuffer(bufLen)
		bufRnd := bitarray.PseudoRand(buf.Len(), nil)

		if r == nil {
			r = bitarray.NewReader(ba)
		} else {
			r.Reset(ba)
		}

		nRead := 0
		for nRead < ba.Len() {
			buf.XorAt(0, bufRnd)
			n, err := r.ReadBits(buf)
			if err != nil {
				t.Errorf("unexpected error: %s", err)
				t.Logf("nRead=%d", nRead)
				t.Logf("buf: %s", buf.String())
				t.Logf("src: %s", ba.String())
				t.FailNow()
			}
			if ba.Len() < nRead+n {
				t.Errorf("too many read: %d for len=%d", nRead+n, ba.Len())
				t.FailNow()
			}
			got := buf.BitArrayAt(0, n)
			want := ba.Slice(nRead, nRead+n)
			if !got.Equal(want) {
				t.Errorf("unexpected read: nRead=%d, n=%d", nRead, n)
				t.Logf(" got: %#b", got)
				t.Logf("want: %#b", want)
				t.FailNow()
			}
			// if i < 32 {
			// 	t.Logf("%3d: nRead=%d: %#b", i, nRead, got)
			// }
			nRead += n
		}

		n, err := r.ReadBits(buf)
		switch {
		case err == nil:
			t.Errorf("expected error, no error: read %d bits", n)
		case !errors.Is(err, io.EOF):
			t.Errorf("unexpected error: %s", err)
		}
	}
}

func TestReader_ReadBits_edge(t *testing.T) {
	ba := bitarray.MustParse("1010-1010")
	r := bitarray.NewReader(ba)

	buf := &bitarray.Buffer{}

	n, err := r.ReadBits(buf)
	switch {
	case err != nil:
		t.Errorf("unexpected error: %s", err)
	case n != 0:
		t.Errorf("unexpected read: n=%d, b=%s", n, buf.String())
	}
}
