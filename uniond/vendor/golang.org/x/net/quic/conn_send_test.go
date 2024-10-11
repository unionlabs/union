// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"testing"
	"time"
)

func TestAckElicitingAck(t *testing.T) {
	// "A receiver that sends only non-ack-eliciting packets [...] might not receive
	// an acknowledgment for a long period of time.
	// [...] a receiver could send a [...] ack-eliciting frame occasionally [...]
	// to elicit an ACK from the peer."
	// https://www.rfc-editor.org/rfc/rfc9000#section-13.2.4-2
	//
	// Send a bunch of ack-eliciting packets, verify that the conn doesn't just
	// send ACKs in response.
	tc := newTestConn(t, clientSide, permissiveTransportParameters)
	tc.handshake()
	const count = 100
	for i := 0; i < count; i++ {
		tc.advance(1 * time.Millisecond)
		tc.writeFrames(packetType1RTT,
			debugFramePing{},
		)
		got, _ := tc.readFrame()
		switch got.(type) {
		case debugFrameAck:
			continue
		case debugFramePing:
			return
		}
	}
	t.Errorf("after sending %v PINGs, got no ack-eliciting response", count)
}

func TestSendPacketNumberSize(t *testing.T) {
	tc := newTestConn(t, clientSide, permissiveTransportParameters)
	tc.handshake()

	recvPing := func() *testPacket {
		t.Helper()
		tc.conn.ping(appDataSpace)
		p := tc.readPacket()
		if p == nil {
			t.Fatalf("want packet containing PING, got none")
		}
		return p
	}

	// Desynchronize the packet numbers the conn is sending and the ones it is receiving,
	// by having the conn send a number of unacked packets.
	for i := 0; i < 16; i++ {
		recvPing()
	}

	// Establish the maximum packet number the conn has received an ACK for.
	maxAcked := recvPing().num
	tc.writeAckForAll()

	// Make the conn send a sequence of packets.
	// Check that the packet number is encoded with two bytes once the difference between the
	// current packet and the max acked one is sufficiently large.
	for want := maxAcked + 1; want < maxAcked+0x100; want++ {
		p := recvPing()
		if p.num != want {
			t.Fatalf("received packet number %v, want %v", p.num, want)
		}
		gotPnumLen := int(p.header&0x03) + 1
		wantPnumLen := 1
		if p.num-maxAcked >= 0x80 {
			wantPnumLen = 2
		}
		if gotPnumLen != wantPnumLen {
			t.Fatalf("packet number 0x%x encoded with %v bytes, want %v (max acked = %v)", p.num, gotPnumLen, wantPnumLen, maxAcked)
		}
	}
}
