// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"context"
	"crypto/tls"
	"testing"
)

func TestVersionNegotiationServerReceivesUnknownVersion(t *testing.T) {
	config := &Config{
		TLSConfig: newTestTLSConfig(serverSide),
	}
	te := newTestEndpoint(t, config)

	// Packet of unknown contents for some unrecognized QUIC version.
	dstConnID := []byte{1, 2, 3, 4}
	srcConnID := []byte{5, 6, 7, 8}
	pkt := []byte{
		0b1000_0000,
		0x00, 0x00, 0x00, 0x0f,
	}
	pkt = append(pkt, byte(len(dstConnID)))
	pkt = append(pkt, dstConnID...)
	pkt = append(pkt, byte(len(srcConnID)))
	pkt = append(pkt, srcConnID...)
	for len(pkt) < paddedInitialDatagramSize {
		pkt = append(pkt, 0)
	}

	te.write(&datagram{
		b: pkt,
	})
	gotPkt := te.read()
	if gotPkt == nil {
		t.Fatalf("got no response; want Version Negotiation")
	}
	if got := getPacketType(gotPkt); got != packetTypeVersionNegotiation {
		t.Fatalf("got packet type %v; want Version Negotiation", got)
	}
	gotDst, gotSrc, versions := parseVersionNegotiation(gotPkt)
	if got, want := gotDst, srcConnID; !bytes.Equal(got, want) {
		t.Errorf("got Destination Connection ID %x, want %x", got, want)
	}
	if got, want := gotSrc, dstConnID; !bytes.Equal(got, want) {
		t.Errorf("got Source Connection ID %x, want %x", got, want)
	}
	if got, want := versions, []byte{0, 0, 0, 1}; !bytes.Equal(got, want) {
		t.Errorf("got Supported Version %x, want %x", got, want)
	}
}

func TestVersionNegotiationClientAborts(t *testing.T) {
	tc := newTestConn(t, clientSide)
	p := tc.readPacket() // client Initial packet
	tc.endpoint.write(&datagram{
		b: appendVersionNegotiation(nil, p.srcConnID, p.dstConnID, 10),
	})
	tc.wantIdle("connection does not send a CONNECTION_CLOSE")
	if err := tc.conn.waitReady(canceledContext()); err != errVersionNegotiation {
		t.Errorf("conn.waitReady() = %v, want errVersionNegotiation", err)
	}
}

func TestVersionNegotiationClientIgnoresAfterProcessingPacket(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.ignoreFrame(frameTypeAck)
	p := tc.readPacket() // client Initial packet
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	tc.endpoint.write(&datagram{
		b: appendVersionNegotiation(nil, p.srcConnID, p.dstConnID, 10),
	})
	if err := tc.conn.waitReady(canceledContext()); err != context.Canceled {
		t.Errorf("conn.waitReady() = %v, want context.Canceled", err)
	}
	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
		})
	tc.wantFrameType("conn ignores Version Negotiation and continues with handshake",
		packetTypeHandshake, debugFrameCrypto{})
}

func TestVersionNegotiationClientIgnoresMismatchingSourceConnID(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.ignoreFrame(frameTypeAck)
	p := tc.readPacket() // client Initial packet
	tc.endpoint.write(&datagram{
		b: appendVersionNegotiation(nil, p.srcConnID, []byte("mismatch"), 10),
	})
	tc.writeFrames(packetTypeInitial,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelInitial],
		})
	tc.writeFrames(packetTypeHandshake,
		debugFrameCrypto{
			data: tc.cryptoDataIn[tls.QUICEncryptionLevelHandshake],
		})
	tc.wantFrameType("conn ignores Version Negotiation and continues with handshake",
		packetTypeHandshake, debugFrameCrypto{})
}
