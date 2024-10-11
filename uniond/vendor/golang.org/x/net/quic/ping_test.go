// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import "testing"

func TestPing(t *testing.T) {
	tc := newTestConn(t, clientSide)
	tc.handshake()

	tc.conn.ping(appDataSpace)
	tc.wantFrame("connection should send a PING frame",
		packetType1RTT, debugFramePing{})

	tc.advanceToTimer()
	tc.wantFrame("on PTO, connection should send another PING frame",
		packetType1RTT, debugFramePing{})

	tc.wantIdle("after sending PTO probe, no additional frames to send")
}

func TestAck(t *testing.T) {
	tc := newTestConn(t, serverSide)
	tc.handshake()

	// Send two packets, to trigger an immediate ACK.
	tc.writeFrames(packetType1RTT,
		debugFramePing{},
	)
	tc.writeFrames(packetType1RTT,
		debugFramePing{},
	)
	tc.wantFrame("connection should respond to ack-eliciting packet with an ACK frame",
		packetType1RTT,
		debugFrameAck{
			ranges: []i64range[packetNumber]{{0, 4}},
		},
	)
}
