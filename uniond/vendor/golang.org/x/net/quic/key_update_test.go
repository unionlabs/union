// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"testing"
)

func TestKeyUpdatePeerUpdates(t *testing.T) {
	tc := newTestConn(t, serverSide)
	tc.handshake()
	tc.ignoreFrames = nil // ignore nothing

	// Peer initiates a key update.
	tc.sendKeyNumber = 1
	tc.sendKeyPhaseBit = true
	tc.writeFrames(packetType1RTT, debugFramePing{})

	// We update to the new key.
	tc.advanceToTimer()
	tc.wantFrameType("conn ACKs last packet",
		packetType1RTT, debugFrameAck{})
	tc.wantFrame("first packet after a key update is always ack-eliciting",
		packetType1RTT, debugFramePing{})
	if got, want := tc.lastPacket.keyNumber, 1; got != want {
		t.Errorf("after key rotation, conn sent packet with key %v, want %v", got, want)
	}
	if !tc.lastPacket.keyPhaseBit {
		t.Errorf("after key rotation, conn failed to change Key Phase bit")
	}
	tc.wantIdle("conn has nothing to send")

	// Peer's ACK of a packet we sent in the new phase completes the update.
	tc.writeAckForAll()

	// Peer initiates a second key update.
	tc.sendKeyNumber = 2
	tc.sendKeyPhaseBit = false
	tc.writeFrames(packetType1RTT, debugFramePing{})

	// We update to the new key.
	tc.advanceToTimer()
	tc.wantFrameType("conn ACKs last packet",
		packetType1RTT, debugFrameAck{})
	tc.wantFrame("first packet after a key update is always ack-eliciting",
		packetType1RTT, debugFramePing{})
	if got, want := tc.lastPacket.keyNumber, 2; got != want {
		t.Errorf("after key rotation, conn sent packet with key %v, want %v", got, want)
	}
	if tc.lastPacket.keyPhaseBit {
		t.Errorf("after second key rotation, conn failed to change Key Phase bit")
	}
	tc.wantIdle("conn has nothing to send")
}

func TestKeyUpdateAcceptPreviousPhaseKeys(t *testing.T) {
	// "An endpoint SHOULD retain old keys for some time after
	// unprotecting a packet sent using the new keys."
	// https://www.rfc-editor.org/rfc/rfc9001#section-6.1-8
	tc := newTestConn(t, serverSide)
	tc.handshake()
	tc.ignoreFrames = nil // ignore nothing

	// Peer initiates a key update, skipping one packet number.
	pnum0 := tc.peerNextPacketNum[appDataSpace]
	tc.peerNextPacketNum[appDataSpace]++
	tc.sendKeyNumber = 1
	tc.sendKeyPhaseBit = true
	tc.writeFrames(packetType1RTT, debugFramePing{})

	// We update to the new key.
	// This ACK is not delayed, because we've skipped a packet number.
	tc.wantFrame("conn ACKs last packet",
		packetType1RTT, debugFrameAck{
			ranges: []i64range[packetNumber]{
				{0, pnum0},
				{pnum0 + 1, pnum0 + 2},
			},
		})
	tc.wantFrame("first packet after a key update is always ack-eliciting",
		packetType1RTT, debugFramePing{})
	if got, want := tc.lastPacket.keyNumber, 1; got != want {
		t.Errorf("after key rotation, conn sent packet with key %v, want %v", got, want)
	}
	if !tc.lastPacket.keyPhaseBit {
		t.Errorf("after key rotation, conn failed to change Key Phase bit")
	}
	tc.wantIdle("conn has nothing to send")

	// We receive the previously-skipped packet in the earlier key phase.
	tc.peerNextPacketNum[appDataSpace] = pnum0
	tc.sendKeyNumber = 0
	tc.sendKeyPhaseBit = false
	tc.writeFrames(packetType1RTT, debugFramePing{})

	// We ack the reordered packet immediately, still in the new key phase.
	tc.wantFrame("conn ACKs reordered packet",
		packetType1RTT, debugFrameAck{
			ranges: []i64range[packetNumber]{
				{0, pnum0 + 2},
			},
		})
	tc.wantIdle("packet is not ack-eliciting")
	if got, want := tc.lastPacket.keyNumber, 1; got != want {
		t.Errorf("after key rotation, conn sent packet with key %v, want %v", got, want)
	}
	if !tc.lastPacket.keyPhaseBit {
		t.Errorf("after key rotation, conn failed to change Key Phase bit")
	}
}

func TestKeyUpdateRejectPacketFromPriorPhase(t *testing.T) {
	// "Packets with higher packet numbers MUST be protected with either
	// the same or newer packet protection keys than packets with lower packet numbers."
	// https://www.rfc-editor.org/rfc/rfc9001#section-6.4-2
	tc := newTestConn(t, serverSide)
	tc.handshake()
	tc.ignoreFrames = nil // ignore nothing

	// Peer initiates a key update.
	tc.sendKeyNumber = 1
	tc.sendKeyPhaseBit = true
	tc.writeFrames(packetType1RTT, debugFramePing{})

	// We update to the new key.
	tc.advanceToTimer()
	tc.wantFrameType("conn ACKs last packet",
		packetType1RTT, debugFrameAck{})
	tc.wantFrame("first packet after a key update is always ack-eliciting",
		packetType1RTT, debugFramePing{})
	if got, want := tc.lastPacket.keyNumber, 1; got != want {
		t.Errorf("after key rotation, conn sent packet with key %v, want %v", got, want)
	}
	if !tc.lastPacket.keyPhaseBit {
		t.Errorf("after key rotation, conn failed to change Key Phase bit")
	}
	tc.wantIdle("conn has nothing to send")

	// Peer sends an ack-eliciting packet using the prior phase keys.
	// We fail to unprotect the packet and ignore it.
	skipped := tc.peerNextPacketNum[appDataSpace]
	tc.sendKeyNumber = 0
	tc.sendKeyPhaseBit = false
	tc.writeFrames(packetType1RTT, debugFramePing{})

	// Peer sends an ack-eliciting packet using the current phase keys.
	tc.sendKeyNumber = 1
	tc.sendKeyPhaseBit = true
	tc.writeFrames(packetType1RTT, debugFramePing{})

	// We ack the peer's packets, not including the one sent with the wrong keys.
	tc.wantFrame("conn ACKs packets, not including packet sent with wrong keys",
		packetType1RTT, debugFrameAck{
			ranges: []i64range[packetNumber]{
				{0, skipped},
				{skipped + 1, skipped + 2},
			},
		})
}

func TestKeyUpdateLocallyInitiated(t *testing.T) {
	const updateAfter = 4 // initiate key update after 1-RTT packet 4
	tc := newTestConn(t, serverSide)
	tc.conn.keysAppData.updateAfter = updateAfter
	tc.handshake()

	for {
		tc.writeFrames(packetType1RTT, debugFramePing{})
		tc.advanceToTimer()
		tc.wantFrameType("conn ACKs last packet",
			packetType1RTT, debugFrameAck{})
		if tc.lastPacket.num > updateAfter {
			break
		}
		if got, want := tc.lastPacket.keyNumber, 0; got != want {
			t.Errorf("before key update, conn sent packet with key %v, want %v", got, want)
		}
		if tc.lastPacket.keyPhaseBit {
			t.Errorf("before key update, keyPhaseBit is set, want unset")
		}
	}
	if got, want := tc.lastPacket.keyNumber, 1; got != want {
		t.Errorf("after key update, conn sent packet with key %v, want %v", got, want)
	}
	if !tc.lastPacket.keyPhaseBit {
		t.Errorf("after key update, keyPhaseBit is unset, want set")
	}
	tc.wantFrame("first packet after a key update is always ack-eliciting",
		packetType1RTT, debugFramePing{})
	tc.wantIdle("no more frames")

	// Peer sends another packet using the prior phase keys.
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.advanceToTimer()
	tc.wantFrameType("conn ACKs packet in prior phase",
		packetType1RTT, debugFrameAck{})
	tc.wantIdle("packet is not ack-eliciting")
	if got, want := tc.lastPacket.keyNumber, 1; got != want {
		t.Errorf("after key update, conn sent packet with key %v, want %v", got, want)
	}

	// Peer updates to the next phase.
	tc.sendKeyNumber = 1
	tc.sendKeyPhaseBit = true
	tc.writeAckForAll()
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.advanceToTimer()
	tc.wantFrameType("conn ACKs packet in current phase",
		packetType1RTT, debugFrameAck{})
	tc.wantIdle("packet is not ack-eliciting")
	if got, want := tc.lastPacket.keyNumber, 1; got != want {
		t.Errorf("after key update, conn sent packet with key %v, want %v", got, want)
	}

	// Peer initiates its own update.
	tc.sendKeyNumber = 2
	tc.sendKeyPhaseBit = false
	tc.writeFrames(packetType1RTT, debugFramePing{})
	tc.advanceToTimer()
	tc.wantFrameType("conn ACKs packet in current phase",
		packetType1RTT, debugFrameAck{})
	tc.wantFrame("first packet after a key update is always ack-eliciting",
		packetType1RTT, debugFramePing{})
	if got, want := tc.lastPacket.keyNumber, 2; got != want {
		t.Errorf("after peer key update, conn sent packet with key %v, want %v", got, want)
	}
	if tc.lastPacket.keyPhaseBit {
		t.Errorf("after peer key update, keyPhaseBit is unset, want set")
	}
}
