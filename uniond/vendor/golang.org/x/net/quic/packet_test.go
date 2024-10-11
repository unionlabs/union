// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"reflect"
	"strings"
	"testing"
)

func TestPacketHeader(t *testing.T) {
	for _, test := range []struct {
		name         string
		packet       []byte
		isLongHeader bool
		packetType   packetType
		dstConnID    []byte
	}{{
		// Initial packet from https://www.rfc-editor.org/rfc/rfc9001#section-a.1
		// (truncated)
		name: "rfc9001_a1",
		packet: unhex(`
			c000000001088394c8f03e5157080000 449e7b9aec34d1b1c98dd7689fb8ec11
		`),
		isLongHeader: true,
		packetType:   packetTypeInitial,
		dstConnID:    unhex(`8394c8f03e515708`),
	}, {
		// Initial packet from https://www.rfc-editor.org/rfc/rfc9001#section-a.3
		// (truncated)
		name: "rfc9001_a3",
		packet: unhex(`
			cf000000010008f067a5502a4262b500 4075c0d95a482cd0991cd25b0aac406a
		`),
		isLongHeader: true,
		packetType:   packetTypeInitial,
		dstConnID:    []byte{},
	}, {
		// Retry packet from https://www.rfc-editor.org/rfc/rfc9001#section-a.4
		name: "rfc9001_a4",
		packet: unhex(`
			ff000000010008f067a5502a4262b574 6f6b656e04a265ba2eff4d829058fb3f
			0f2496ba
		`),
		isLongHeader: true,
		packetType:   packetTypeRetry,
		dstConnID:    []byte{},
	}, {
		// Short header packet from https://www.rfc-editor.org/rfc/rfc9001#section-a.5
		name: "rfc9001_a5",
		packet: unhex(`
			4cfe4189655e5cd55c41f69080575d7999c25a5bfb
		`),
		isLongHeader: false,
		packetType:   packetType1RTT,
		dstConnID:    unhex(`fe4189655e5cd55c`),
	}, {
		// Version Negotiation packet.
		name: "version_negotiation",
		packet: unhex(`
			80 00000000 01ff0001020304
		`),
		isLongHeader: true,
		packetType:   packetTypeVersionNegotiation,
		dstConnID:    []byte{0xff},
	}, {
		// Too-short packet.
		name: "truncated_after_connid_length",
		packet: unhex(`
			cf0000000105
		`),
		isLongHeader: true,
		packetType:   packetTypeInitial,
		dstConnID:    nil,
	}, {
		// Too-short packet.
		name: "truncated_after_version",
		packet: unhex(`
			cf00000001
		`),
		isLongHeader: true,
		packetType:   packetTypeInitial,
		dstConnID:    nil,
	}, {
		// Much too short packet.
		name: "truncated_in_version",
		packet: unhex(`
			cf000000
		`),
		isLongHeader: true,
		packetType:   packetTypeInvalid,
		dstConnID:    nil,
	}} {
		t.Run(test.name, func(t *testing.T) {
			if got, want := isLongHeader(test.packet[0]), test.isLongHeader; got != want {
				t.Errorf("packet %x:\nisLongHeader(packet) = %v, want %v", test.packet, got, want)
			}
			if got, want := getPacketType(test.packet), test.packetType; got != want {
				t.Errorf("packet %x:\ngetPacketType(packet) = %v, want %v", test.packet, got, want)
			}
			gotConnID, gotOK := dstConnIDForDatagram(test.packet)
			wantConnID, wantOK := test.dstConnID, test.dstConnID != nil
			if !bytes.Equal(gotConnID, wantConnID) || gotOK != wantOK {
				t.Errorf("packet %x:\ndstConnIDForDatagram(packet) = {%x}, %v; want {%x}, %v", test.packet, gotConnID, gotOK, wantConnID, wantOK)
			}
		})
	}
}

func TestEncodeDecodeVersionNegotiation(t *testing.T) {
	dstConnID := []byte("this is a very long destination connection id")
	srcConnID := []byte("this is a very long source connection id")
	versions := []uint32{1, 0xffffffff}
	got := appendVersionNegotiation([]byte{}, dstConnID, srcConnID, versions...)
	want := bytes.Join([][]byte{{
		0b1100_0000, // header byte
		0, 0, 0, 0,  // Version
		byte(len(dstConnID)),
	}, dstConnID, {
		byte(len(srcConnID)),
	}, srcConnID, {
		0x00, 0x00, 0x00, 0x01,
		0xff, 0xff, 0xff, 0xff,
	}}, nil)
	if !bytes.Equal(got, want) {
		t.Fatalf("appendVersionNegotiation(nil, %x, %x, %v):\ngot  %x\nwant %x",
			dstConnID, srcConnID, versions, got, want)
	}
	gotDst, gotSrc, gotVersionBytes := parseVersionNegotiation(got)
	if got, want := gotDst, dstConnID; !bytes.Equal(got, want) {
		t.Errorf("parseVersionNegotiation: got dstConnID = %x, want %x", got, want)
	}
	if got, want := gotSrc, srcConnID; !bytes.Equal(got, want) {
		t.Errorf("parseVersionNegotiation: got srcConnID = %x, want %x", got, want)
	}
	var gotVersions []uint32
	for len(gotVersionBytes) >= 4 {
		gotVersions = append(gotVersions, binary.BigEndian.Uint32(gotVersionBytes))
		gotVersionBytes = gotVersionBytes[4:]
	}
	if got, want := gotVersions, versions; !reflect.DeepEqual(got, want) {
		t.Errorf("parseVersionNegotiation: got versions = %v, want %v", got, want)
	}
}

func TestParseGenericLongHeaderPacket(t *testing.T) {
	for _, test := range []struct {
		name      string
		packet    []byte
		version   uint32
		dstConnID []byte
		srcConnID []byte
		data      []byte
	}{{
		name: "long header packet",
		packet: unhex(`
			80 01020304 04a1a2a3a4 05b1b2b3b4b5 c1
		`),
		version:   0x01020304,
		dstConnID: unhex(`a1a2a3a4`),
		srcConnID: unhex(`b1b2b3b4b5`),
		data:      unhex(`c1`),
	}, {
		name: "zero everything",
		packet: unhex(`
			80 00000000 00 00
		`),
		version:   0,
		dstConnID: []byte{},
		srcConnID: []byte{},
		data:      []byte{},
	}} {
		t.Run(test.name, func(t *testing.T) {
			p, ok := parseGenericLongHeaderPacket(test.packet)
			if !ok {
				t.Fatalf("parseGenericLongHeaderPacket() = _, false; want true")
			}
			if got, want := p.version, test.version; got != want {
				t.Errorf("version = %v, want %v", got, want)
			}
			if got, want := p.dstConnID, test.dstConnID; !bytes.Equal(got, want) {
				t.Errorf("Destination Connection ID = {%x}, want {%x}", got, want)
			}
			if got, want := p.srcConnID, test.srcConnID; !bytes.Equal(got, want) {
				t.Errorf("Source Connection ID = {%x}, want {%x}", got, want)
			}
			if got, want := p.data, test.data; !bytes.Equal(got, want) {
				t.Errorf("Data = {%x}, want {%x}", got, want)
			}
		})
	}
}

func TestParseGenericLongHeaderPacketErrors(t *testing.T) {
	for _, test := range []struct {
		name   string
		packet []byte
	}{{
		name: "short header packet",
		packet: unhex(`
			00 01020304 04a1a2a3a4 05b1b2b3b4b5 c1
		`),
	}, {
		name: "packet too short",
		packet: unhex(`
			80 000000
		`),
	}, {
		name: "destination id too long",
		packet: unhex(`
			80 00000000 02 00
		`),
	}, {
		name: "source id too long",
		packet: unhex(`
			80 00000000 00 01
		`),
	}} {
		t.Run(test.name, func(t *testing.T) {
			_, ok := parseGenericLongHeaderPacket(test.packet)
			if ok {
				t.Fatalf("parseGenericLongHeaderPacket() = _, true; want false")
			}
		})
	}
}

func unhex(s string) []byte {
	b, err := hex.DecodeString(strings.Map(func(c rune) rune {
		switch c {
		case ' ', '\t', '\n':
			return -1
		}
		return c
	}, s))
	if err != nil {
		panic(err)
	}
	return b
}
