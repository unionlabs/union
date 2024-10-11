// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"encoding/binary"
	"testing"
)

func TestDecodePacketNumber(t *testing.T) {
	for _, test := range []struct {
		largest   packetNumber
		truncated packetNumber
		want      packetNumber
		size      int
	}{{
		largest:   0,
		truncated: 1,
		size:      4,
		want:      1,
	}, {
		largest:   0,
		truncated: 0,
		size:      1,
		want:      0,
	}, {
		largest:   0x00,
		truncated: 0x01,
		size:      1,
		want:      0x01,
	}, {
		largest:   0x00,
		truncated: 0xff,
		size:      1,
		want:      0xff,
	}, {
		largest:   0xff,
		truncated: 0x01,
		size:      1,
		want:      0x101,
	}, {
		largest:   0x1000,
		truncated: 0xff,
		size:      1,
		want:      0xfff,
	}, {
		largest:   0xa82f30ea,
		truncated: 0x9b32,
		size:      2,
		want:      0xa82f9b32,
	}} {
		got := decodePacketNumber(test.largest, test.truncated, test.size)
		if got != test.want {
			t.Errorf("decodePacketNumber(largest=0x%x, truncated=0x%x, size=%v) = 0x%x, want 0x%x", test.largest, test.truncated, test.size, got, test.want)
		}
	}
}

func TestEncodePacketNumber(t *testing.T) {
	for _, test := range []struct {
		largestAcked packetNumber
		pnum         packetNumber
		wantSize     int
	}{{
		largestAcked: -1,
		pnum:         0,
		wantSize:     1,
	}, {
		largestAcked: 1000,
		pnum:         1000 + 0x7f,
		wantSize:     1,
	}, {
		largestAcked: 1000,
		pnum:         1000 + 0x80, // 0x468
		wantSize:     2,
	}, {
		largestAcked: 0x12345678,
		pnum:         0x12345678 + 0x7fff, // 0x305452663
		wantSize:     2,
	}, {
		largestAcked: 0x12345678,
		pnum:         0x12345678 + 0x8000,
		wantSize:     3,
	}, {
		largestAcked: 0,
		pnum:         0x7fffff,
		wantSize:     3,
	}, {
		largestAcked: 0,
		pnum:         0x800000,
		wantSize:     4,
	}, {
		largestAcked: 0xabe8bc,
		pnum:         0xac5c02,
		wantSize:     2,
	}, {
		largestAcked: 0xabe8bc,
		pnum:         0xace8fe,
		wantSize:     3,
	}} {
		size := packetNumberLength(test.pnum, test.largestAcked)
		if got, want := size, test.wantSize; got != want {
			t.Errorf("packetNumberLength(num=%x, maxAck=%x) = %v, want %v", test.pnum, test.largestAcked, got, want)
		}
		var enc packetNumber
		switch size {
		case 1:
			enc = test.pnum & 0xff
		case 2:
			enc = test.pnum & 0xffff
		case 3:
			enc = test.pnum & 0xffffff
		case 4:
			enc = test.pnum & 0xffffffff
		}
		wantBytes := binary.BigEndian.AppendUint32(nil, uint32(enc))[4-size:]
		gotBytes := appendPacketNumber(nil, test.pnum, test.largestAcked)
		if !bytes.Equal(gotBytes, wantBytes) {
			t.Errorf("appendPacketNumber(num=%v, maxAck=%x) = {%x}, want {%x}", test.pnum, test.largestAcked, gotBytes, wantBytes)
		}
		gotNum := decodePacketNumber(test.largestAcked, enc, size)
		if got, want := gotNum, test.pnum; got != want {
			t.Errorf("packetNumberLength(num=%x, maxAck=%x) = %v, but decoded number=%x", test.pnum, test.largestAcked, size, got)
		}
	}
}

func FuzzPacketNumber(f *testing.F) {
	truncatedNumber := func(in []byte) packetNumber {
		var truncated packetNumber
		for _, b := range in {
			truncated = (truncated << 8) | packetNumber(b)
		}
		return truncated
	}
	f.Fuzz(func(t *testing.T, in []byte, largestAckedInt64 int64) {
		largestAcked := packetNumber(largestAckedInt64)
		if len(in) < 1 || len(in) > 4 || largestAcked < 0 || largestAcked > maxPacketNumber {
			return
		}
		truncatedIn := truncatedNumber(in)
		decoded := decodePacketNumber(largestAcked, truncatedIn, len(in))

		// Check that the decoded packet number's least significant bits match the input.
		var mask packetNumber
		for i := 0; i < len(in); i++ {
			mask = (mask << 8) | 0xff
		}
		if truncatedIn != decoded&mask {
			t.Fatalf("decoding mismatch: input=%x largestAcked=%v decoded=0x%x", in, largestAcked, decoded)
		}

		// We don't support encoding packet numbers less than largestAcked (since packet numbers
		// never decrease), so skip the encoder tests if this would make us go backwards.
		if decoded < largestAcked {
			return
		}

		// We might encode this number using a different length than we received,
		// but the common portions should match.
		encoded := appendPacketNumber(nil, decoded, largestAcked)
		a, b := in, encoded
		if len(b) < len(a) {
			a, b = b, a
		}
		for len(a) < len(b) {
			b = b[1:]
		}
		if len(a) == 0 || !bytes.Equal(a, b) {
			t.Fatalf("encoding mismatch: input=%x largestAcked=%v decoded=%v reencoded=%x", in, largestAcked, decoded, encoded)
		}

		if g := decodePacketNumber(largestAcked, truncatedNumber(encoded), len(encoded)); g != decoded {
			t.Fatalf("packet encode/decode mismatch: pnum=%v largestAcked=%v encoded=%x got=%v", decoded, largestAcked, encoded, g)
		}
		if l := packetNumberLength(decoded, largestAcked); l != len(encoded) {
			t.Fatalf("packet number length mismatch: pnum=%v largestAcked=%v encoded=%x len=%v", decoded, largestAcked, encoded, l)
		}
	})
}
