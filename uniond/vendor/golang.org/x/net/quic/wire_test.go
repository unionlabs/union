// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"testing"
)

func TestConsumeVarint(t *testing.T) {
	for _, test := range []struct {
		b       []byte
		want    uint64
		wantLen int
	}{
		{[]byte{0x00}, 0, 1},
		{[]byte{0x3f}, 63, 1},
		{[]byte{0x40, 0x00}, 0, 2},
		{[]byte{0x7f, 0xff}, 16383, 2},
		{[]byte{0x80, 0x00, 0x00, 0x00}, 0, 4},
		{[]byte{0xbf, 0xff, 0xff, 0xff}, 1073741823, 4},
		{[]byte{0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00}, 0, 8},
		{[]byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff}, 4611686018427387903, 8},
		// Example cases from https://www.rfc-editor.org/rfc/rfc9000.html#section-a.1
		{[]byte{0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c}, 151288809941952652, 8},
		{[]byte{0x9d, 0x7f, 0x3e, 0x7d}, 494878333, 4},
		{[]byte{0x7b, 0xbd}, 15293, 2},
		{[]byte{0x25}, 37, 1},
		{[]byte{0x40, 0x25}, 37, 2},
	} {
		got, gotLen := consumeVarint(test.b)
		if got != test.want || gotLen != test.wantLen {
			t.Errorf("consumeVarint(%x) = %v, %v; want %v, %v", test.b, got, gotLen, test.want, test.wantLen)
		}
		// Extra data in the buffer is ignored.
		b := append(test.b, 0)
		got, gotLen = consumeVarint(b)
		if got != test.want || gotLen != test.wantLen {
			t.Errorf("consumeVarint(%x) = %v, %v; want %v, %v", b, got, gotLen, test.want, test.wantLen)
		}
		// Short buffer results in an error.
		for i := 1; i <= len(test.b); i++ {
			b = test.b[:len(test.b)-i]
			got, gotLen = consumeVarint(b)
			if got != 0 || gotLen >= 0 {
				t.Errorf("consumeVarint(%x) = %v, %v; want 0, -1", b, got, gotLen)
			}
		}
	}
}

func TestAppendVarint(t *testing.T) {
	for _, test := range []struct {
		v    uint64
		want []byte
	}{
		{0, []byte{0x00}},
		{63, []byte{0x3f}},
		{16383, []byte{0x7f, 0xff}},
		{1073741823, []byte{0xbf, 0xff, 0xff, 0xff}},
		{4611686018427387903, []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff}},
		// Example cases from https://www.rfc-editor.org/rfc/rfc9000.html#section-a.1
		{151288809941952652, []byte{0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c}},
		{494878333, []byte{0x9d, 0x7f, 0x3e, 0x7d}},
		{15293, []byte{0x7b, 0xbd}},
		{37, []byte{0x25}},
	} {
		got := appendVarint([]byte{}, test.v)
		if !bytes.Equal(got, test.want) {
			t.Errorf("AppendVarint(nil, %v) = %x, want %x", test.v, got, test.want)
		}
		if gotLen, wantLen := sizeVarint(test.v), len(got); gotLen != wantLen {
			t.Errorf("SizeVarint(%v) = %v, want %v", test.v, gotLen, wantLen)
		}
	}
}

func TestConsumeUint32(t *testing.T) {
	for _, test := range []struct {
		b       []byte
		want    uint32
		wantLen int
	}{
		{[]byte{0x01, 0x02, 0x03, 0x04}, 0x01020304, 4},
		{[]byte{0x01, 0x02, 0x03}, 0, -1},
	} {
		if got, n := consumeUint32(test.b); got != test.want || n != test.wantLen {
			t.Errorf("consumeUint32(%x) = %v, %v; want %v, %v", test.b, got, n, test.want, test.wantLen)
		}
	}
}

func TestConsumeUint64(t *testing.T) {
	for _, test := range []struct {
		b       []byte
		want    uint64
		wantLen int
	}{
		{[]byte{0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08}, 0x0102030405060708, 8},
		{[]byte{0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07}, 0, -1},
	} {
		if got, n := consumeUint64(test.b); got != test.want || n != test.wantLen {
			t.Errorf("consumeUint32(%x) = %v, %v; want %v, %v", test.b, got, n, test.want, test.wantLen)
		}
	}
}

func TestConsumeVarintBytes(t *testing.T) {
	for _, test := range []struct {
		b       []byte
		want    []byte
		wantLen int
	}{
		{[]byte{0x00}, []byte{}, 1},
		{[]byte{0x40, 0x00}, []byte{}, 2},
		{[]byte{0x04, 0x01, 0x02, 0x03, 0x04}, []byte{0x01, 0x02, 0x03, 0x04}, 5},
		{[]byte{0x40, 0x04, 0x01, 0x02, 0x03, 0x04}, []byte{0x01, 0x02, 0x03, 0x04}, 6},
	} {
		got, gotLen := consumeVarintBytes(test.b)
		if !bytes.Equal(got, test.want) || gotLen != test.wantLen {
			t.Errorf("consumeVarintBytes(%x) = {%x}, %v; want {%x}, %v", test.b, got, gotLen, test.want, test.wantLen)
		}
		// Extra data in the buffer is ignored.
		b := append(test.b, 0)
		got, gotLen = consumeVarintBytes(b)
		if !bytes.Equal(got, test.want) || gotLen != test.wantLen {
			t.Errorf("consumeVarintBytes(%x) = {%x}, %v; want {%x}, %v", b, got, gotLen, test.want, test.wantLen)
		}
		// Short buffer results in an error.
		for i := 1; i <= len(test.b); i++ {
			b = test.b[:len(test.b)-i]
			got, gotLen := consumeVarintBytes(b)
			if len(got) > 0 || gotLen > 0 {
				t.Errorf("consumeVarintBytes(%x) = {%x}, %v; want {}, -1", b, got, gotLen)
			}
		}

	}
}

func TestConsumeVarintBytesErrors(t *testing.T) {
	for _, b := range [][]byte{
		{0x01},
		{0x40, 0x01},
	} {
		got, gotLen := consumeVarintBytes(b)
		if len(got) > 0 || gotLen > 0 {
			t.Errorf("consumeVarintBytes(%x) = {%x}, %v; want {}, -1", b, got, gotLen)
		}
	}
}

func TestConsumeUint8Bytes(t *testing.T) {
	for _, test := range []struct {
		b       []byte
		want    []byte
		wantLen int
	}{
		{[]byte{0x00}, []byte{}, 1},
		{[]byte{0x01, 0x00}, []byte{0x00}, 2},
		{[]byte{0x04, 0x01, 0x02, 0x03, 0x04}, []byte{0x01, 0x02, 0x03, 0x04}, 5},
	} {
		got, gotLen := consumeUint8Bytes(test.b)
		if !bytes.Equal(got, test.want) || gotLen != test.wantLen {
			t.Errorf("consumeUint8Bytes(%x) = {%x}, %v; want {%x}, %v", test.b, got, gotLen, test.want, test.wantLen)
		}
		// Extra data in the buffer is ignored.
		b := append(test.b, 0)
		got, gotLen = consumeUint8Bytes(b)
		if !bytes.Equal(got, test.want) || gotLen != test.wantLen {
			t.Errorf("consumeUint8Bytes(%x) = {%x}, %v; want {%x}, %v", b, got, gotLen, test.want, test.wantLen)
		}
		// Short buffer results in an error.
		for i := 1; i <= len(test.b); i++ {
			b = test.b[:len(test.b)-i]
			got, gotLen := consumeUint8Bytes(b)
			if len(got) > 0 || gotLen > 0 {
				t.Errorf("consumeUint8Bytes(%x) = {%x}, %v; want {}, -1", b, got, gotLen)
			}
		}

	}
}

func TestConsumeUint8BytesErrors(t *testing.T) {
	for _, b := range [][]byte{
		{0x01},
		{0x04, 0x01, 0x02, 0x03},
	} {
		got, gotLen := consumeUint8Bytes(b)
		if len(got) > 0 || gotLen > 0 {
			t.Errorf("consumeUint8Bytes(%x) = {%x}, %v; want {}, -1", b, got, gotLen)
		}
	}
}

func TestAppendUint8Bytes(t *testing.T) {
	var got []byte
	got = appendUint8Bytes(got, []byte{})
	got = appendUint8Bytes(got, []byte{0xaa, 0xbb})
	want := []byte{
		0x00,
		0x02, 0xaa, 0xbb,
	}
	if !bytes.Equal(got, want) {
		t.Errorf("appendUint8Bytes {}, {aabb} = {%x}; want {%x}", got, want)
	}
}

func TestAppendVarintBytes(t *testing.T) {
	var got []byte
	got = appendVarintBytes(got, []byte{})
	got = appendVarintBytes(got, []byte{0xaa, 0xbb})
	want := []byte{
		0x00,
		0x02, 0xaa, 0xbb,
	}
	if !bytes.Equal(got, want) {
		t.Errorf("appendVarintBytes {}, {aabb} = {%x}; want {%x}", got, want)
	}
}
