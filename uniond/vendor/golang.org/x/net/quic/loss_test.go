// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"fmt"
	"testing"
	"time"
)

func TestLossAntiAmplificationLimit(t *testing.T) {
	test := newLossTest(t, serverSide, lossTestOpts{})
	test.datagramReceived(1200)
	t.Logf("# consume anti-amplification capacity in a mix of packets")
	test.send(initialSpace, 0, sentPacket{
		size:         1200,
		ackEliciting: true,
		inFlight:     true,
	})
	test.send(initialSpace, 1, sentPacket{
		size:         1200,
		ackEliciting: false,
		inFlight:     false,
	})
	test.send(initialSpace, 2, sentPacket{
		size:         1200,
		ackEliciting: false,
		inFlight:     true,
	})
	t.Logf("# send blocked by anti-amplification limit")
	test.wantSendLimit(ccBlocked)

	t.Logf("# receiving a datagram unblocks server")
	test.datagramReceived(100)
	test.wantSendLimit(ccOK)

	t.Logf("# validating client address removes anti-amplification limit")
	test.validateClientAddress()
	test.wantSendLimit(ccOK)
}

func TestLossRTTSampleNotGenerated(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.send(initialSpace, 0, 1)
	test.send(initialSpace, 2, sentPacket{
		ackEliciting: false,
		inFlight:     false,
	})
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(initialSpace, 1)
	test.wantVar("latest_rtt", 10*time.Millisecond)
	t.Logf("# smoothed_rtt = latest_rtt")
	test.wantVar("smoothed_rtt", 10*time.Millisecond)
	t.Logf("# rttvar = latest_rtt / 2")
	test.wantVar("rttvar", 5*time.Millisecond)

	// "...an ACK frame SHOULD NOT be used to update RTT estimates if
	// it does not newly acknowledge the largest acknowledged packet."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.1-6
	t.Logf("# acks for older packets do not generate an RTT sample")
	test.advance(1 * time.Millisecond)
	test.ack(initialSpace, 1*time.Millisecond, i64range[packetNumber]{0, 2})
	test.wantAck(initialSpace, 0)
	test.wantVar("smoothed_rtt", 10*time.Millisecond)

	// "An RTT sample MUST NOT be generated on receiving an ACK frame
	// that does not newly acknowledge at least one ack-eliciting packet."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.1-7
	t.Logf("# acks for non-ack-eliciting packets do not generate an RTT sample")
	test.advance(1 * time.Millisecond)
	test.ack(initialSpace, 1*time.Millisecond, i64range[packetNumber]{0, 3})
	test.wantAck(initialSpace, 2)
	test.wantVar("smoothed_rtt", 10*time.Millisecond)
}

func TestLossMinRTT(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})

	// "min_rtt MUST be set to the latest_rtt on the first RTT sample."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.2-2
	t.Logf("# min_rtt set on first sample")
	test.send(initialSpace, 0)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	test.wantVar("min_rtt", 10*time.Millisecond)

	// "min_rtt MUST be set to the lesser of min_rtt and latest_rtt [...]
	// on all other samples."
	t.Logf("# min_rtt does not increase")
	test.send(initialSpace, 1)
	test.advance(20 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 2})
	test.wantAck(initialSpace, 1)
	test.wantVar("min_rtt", 10*time.Millisecond)

	t.Logf("# min_rtt decreases")
	test.send(initialSpace, 2)
	test.advance(5 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 3})
	test.wantAck(initialSpace, 2)
	test.wantVar("min_rtt", 5*time.Millisecond)
}

func TestLossMinRTTAfterCongestion(t *testing.T) {
	// "Endpoints SHOULD set the min_rtt to the newest RTT sample
	// after persistent congestion is established."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.2-5
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	t.Logf("# establish initial RTT sample")
	test.send(initialSpace, 0, testSentPacketSize(1200))
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	test.wantVar("min_rtt", 10*time.Millisecond)

	t.Logf("# send two packets spanning persistent congestion duration")
	test.send(initialSpace, 1, testSentPacketSize(1200))
	t.Logf("# 2000ms >> persistent congestion duration")
	test.advance(2000 * time.Millisecond)
	test.wantPTOExpired()
	test.send(initialSpace, 2, testSentPacketSize(1200))

	t.Logf("# trigger loss of previous packets")
	test.advance(10 * time.Millisecond)
	test.send(initialSpace, 3, testSentPacketSize(1200))
	test.advance(20 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{3, 4})
	test.wantAck(initialSpace, 3)
	test.wantLoss(initialSpace, 1, 2)
	t.Logf("# persistent congestion detected")

	test.send(initialSpace, 4, testSentPacketSize(1200))
	test.advance(20 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{4, 5})
	test.wantAck(initialSpace, 4)

	t.Logf("# min_rtt set from first sample after persistent congestion")
	test.wantVar("min_rtt", 20*time.Millisecond)
}

func TestLossInitialRTTSample(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.setMaxAckDelay(2 * time.Millisecond)
	t.Logf("# initial smoothed_rtt and rtt values")
	test.wantVar("smoothed_rtt", 333*time.Millisecond)
	test.wantVar("rttvar", 333*time.Millisecond/2)

	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.3-11
	t.Logf("# first RTT sample")
	test.send(initialSpace, 0)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	test.wantVar("latest_rtt", 10*time.Millisecond)
	t.Logf("# smoothed_rtt = latest_rtt")
	test.wantVar("smoothed_rtt", 10*time.Millisecond)
	t.Logf("# rttvar = latest_rtt / 2")
	test.wantVar("rttvar", 5*time.Millisecond)
}

func TestLossSmoothedRTTIgnoresMaxAckDelayBeforeHandshakeConfirmed(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.setMaxAckDelay(1 * time.Millisecond)
	test.send(initialSpace, 0)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	smoothedRTT := 10 * time.Millisecond
	rttvar := 5 * time.Millisecond

	// "[...] an endpoint [...] SHOULD ignore the peer's max_ack_delay
	// until the handshake is confirmed [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.3-7.2
	t.Logf("# subsequent RTT sample")
	test.send(handshakeSpace, 0)
	test.advance(20 * time.Millisecond)
	test.ack(handshakeSpace, 10*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(handshakeSpace, 0)
	test.wantVar("latest_rtt", 20*time.Millisecond)
	t.Logf("# ack_delay > max_ack_delay")
	t.Logf("# handshake not confirmed, so ignore max_ack_delay")
	t.Logf("# adjusted_rtt = latest_rtt - ackDelay")
	adjustedRTT := 10 * time.Millisecond
	t.Logf("# smoothed_rtt = 7/8 * smoothed_rtt + 1/8 * adjusted_rtt")
	smoothedRTT = (7*smoothedRTT + adjustedRTT) / 8
	test.wantVar("smoothed_rtt", smoothedRTT)
	rttvarSample := abs(smoothedRTT - adjustedRTT)
	t.Logf("# rttvar_sample = abs(smoothed_rtt - adjusted_rtt) = %v", rttvarSample)
	t.Logf("# rttvar = 3/4 * rttvar + 1/4 * rttvar_sample")
	rttvar = (3*rttvar + rttvarSample) / 4
	test.wantVar("rttvar", rttvar)
}

func TestLossSmoothedRTTUsesMaxAckDelayAfterHandshakeConfirmed(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.setMaxAckDelay(25 * time.Millisecond)
	test.send(initialSpace, 0)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	smoothedRTT := 10 * time.Millisecond
	rttvar := 5 * time.Millisecond

	test.confirmHandshake()

	// "[...] an endpoint [...] MUST use the lesser of the acknowledgment
	// delay and the peer's max_ack_delay after the handshake is confirmed [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.3-7.3
	t.Logf("# subsequent RTT sample")
	test.send(handshakeSpace, 0)
	test.advance(50 * time.Millisecond)
	test.ack(handshakeSpace, 40*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(handshakeSpace, 0)
	test.wantVar("latest_rtt", 50*time.Millisecond)
	t.Logf("# ack_delay > max_ack_delay")
	t.Logf("# handshake confirmed, so adjusted_rtt clamps to max_ack_delay")
	t.Logf("# adjusted_rtt = max_ack_delay")
	adjustedRTT := 25 * time.Millisecond
	rttvarSample := abs(smoothedRTT - adjustedRTT)
	t.Logf("# rttvar_sample = abs(smoothed_rtt - adjusted_rtt) = %v", rttvarSample)
	t.Logf("# rttvar = 3/4 * rttvar + 1/4 * rttvar_sample")
	rttvar = (3*rttvar + rttvarSample) / 4
	test.wantVar("rttvar", rttvar)
	t.Logf("# smoothed_rtt = 7/8 * smoothed_rtt + 1/8 * adjusted_rtt")
	smoothedRTT = (7*smoothedRTT + adjustedRTT) / 8
	test.wantVar("smoothed_rtt", smoothedRTT)
}

func TestLossAckDelayReducesRTTBelowMinRTT(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.send(initialSpace, 0)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	smoothedRTT := 10 * time.Millisecond
	rttvar := 5 * time.Millisecond

	// "[...] an endpoint [...] MUST NOT subtract the acknowledgment delay
	// from the RTT sample if the resulting value is smaller than the min_rtt."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.3-7.4
	t.Logf("# subsequent RTT sample")
	test.send(handshakeSpace, 0)
	test.advance(12 * time.Millisecond)
	test.ack(handshakeSpace, 4*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(handshakeSpace, 0)
	test.wantVar("latest_rtt", 12*time.Millisecond)
	t.Logf("# latest_rtt - ack_delay < min_rtt, so adjusted_rtt = latest_rtt")
	adjustedRTT := 12 * time.Millisecond
	rttvarSample := abs(smoothedRTT - adjustedRTT)
	t.Logf("# rttvar_sample = abs(smoothed_rtt - adjusted_rtt) = %v", rttvarSample)
	t.Logf("# rttvar = 3/4 * rttvar + 1/4 * rttvar_sample")
	rttvar = (3*rttvar + rttvarSample) / 4
	test.wantVar("rttvar", rttvar)
	t.Logf("# smoothed_rtt = 7/8 * smoothed_rtt + 1/8 * adjusted_rtt")
	smoothedRTT = (7*smoothedRTT + adjustedRTT) / 8
	test.wantVar("smoothed_rtt", smoothedRTT)
}

func TestLossPacketThreshold(t *testing.T) {
	// "[...] the packet was sent kPacketThreshold packets before an
	// acknowledged packet [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.1.1
	test := newLossTest(t, clientSide, lossTestOpts{})
	t.Logf("# acking a packet triggers loss of packets sent kPacketThreshold earlier")
	test.send(appDataSpace, 0, 1, 2, 3, 4, 5, 6)
	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{4, 5})
	test.wantAck(appDataSpace, 4)
	test.wantLoss(appDataSpace, 0, 1)
}

func TestLossOutOfOrderAcks(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})
	t.Logf("# out of order acks, no loss")
	test.send(appDataSpace, 0, 1, 2)
	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{2, 3})
	test.wantAck(appDataSpace, 2)

	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(appDataSpace, 1)

	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(appDataSpace, 0)
}

func TestLossSendAndAck(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.send(appDataSpace, 0, 1, 2)
	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{0, 3})
	test.wantAck(appDataSpace, 0, 1, 2)
	// Redundant ACK doesn't trigger more ACK events.
	// (If we did get an extra ACK, the test cleanup would notice and complain.)
	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{0, 3})
}

func TestLossAckEveryOtherPacket(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.send(appDataSpace, 0, 1, 2, 3, 4, 5, 6)
	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(appDataSpace, 0)

	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{2, 3})
	test.wantAck(appDataSpace, 2)

	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{4, 5})
	test.wantAck(appDataSpace, 4)
	test.wantLoss(appDataSpace, 1)

	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{6, 7})
	test.wantAck(appDataSpace, 6)
	test.wantLoss(appDataSpace, 3)
}

func TestLossMultipleSpaces(t *testing.T) {
	// "Loss detection is separate per packet number space [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6-3
	test := newLossTest(t, clientSide, lossTestOpts{})
	t.Logf("# send packets in different spaces")
	test.send(initialSpace, 0, 1, 2)
	test.send(handshakeSpace, 0, 1, 2)
	test.send(appDataSpace, 0, 1, 2)

	t.Logf("# ack one packet in each space")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(initialSpace, 1)

	test.ack(handshakeSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(handshakeSpace, 1)

	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(appDataSpace, 1)

	t.Logf("# send more packets")
	test.send(initialSpace, 3, 4, 5)
	test.send(handshakeSpace, 3, 4, 5)
	test.send(appDataSpace, 3, 4, 5)

	t.Logf("# ack the last packet, triggering loss")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{5, 6})
	test.wantAck(initialSpace, 5)
	test.wantLoss(initialSpace, 0, 2)

	test.ack(handshakeSpace, 0*time.Millisecond, i64range[packetNumber]{5, 6})
	test.wantAck(handshakeSpace, 5)
	test.wantLoss(handshakeSpace, 0, 2)

	test.ack(appDataSpace, 0*time.Millisecond, i64range[packetNumber]{5, 6})
	test.wantAck(appDataSpace, 5)
	test.wantLoss(appDataSpace, 0, 2)
}

func TestLossTimeThresholdFirstPacketLost(t *testing.T) {
	// "[...] the packet [...] was sent long enough in the past."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.1-3.2
	test := newLossTest(t, clientSide, lossTestOpts{})
	t.Logf("# packet 0 lost after time threshold passes")
	test.send(initialSpace, 0, 1)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(initialSpace, 1)

	t.Logf("# latest_rtt == smoothed_rtt")
	test.wantVar("smoothed_rtt", 10*time.Millisecond)
	test.wantVar("latest_rtt", 10*time.Millisecond)
	t.Logf("# timeout = 9/8 * max(smoothed_rtt, latest_rtt) - time_since_packet_sent")
	test.wantTimeout(((10 * time.Millisecond * 9) / 8) - 10*time.Millisecond)

	test.advanceToLossTimer()
	test.wantLoss(initialSpace, 0)
}

func TestLossTimeThreshold(t *testing.T) {
	// "The time threshold is:
	// max(kTimeThreshold * max(smoothed_rtt, latest_rtt), kGranularity)"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.1.2-2
	for _, tc := range []struct {
		name        string
		initialRTT  time.Duration
		latestRTT   time.Duration
		wantTimeout time.Duration
	}{{
		name:        "rtt increasing",
		initialRTT:  10 * time.Millisecond,
		latestRTT:   20 * time.Millisecond,
		wantTimeout: 20 * time.Millisecond * 9 / 8,
	}, {
		name:        "rtt decreasing",
		initialRTT:  10 * time.Millisecond,
		latestRTT:   5 * time.Millisecond,
		wantTimeout: ((7*10*time.Millisecond + 5*time.Millisecond) / 8) * 9 / 8,
	}, {
		name:        "rtt less than timer granularity",
		initialRTT:  500 * time.Microsecond,
		latestRTT:   500 * time.Microsecond,
		wantTimeout: 1 * time.Millisecond,
	}} {
		t.Run(tc.name, func(t *testing.T) {
			test := newLossTest(t, clientSide, lossTestOpts{})
			t.Logf("# first ack establishes smoothed_rtt")
			test.send(initialSpace, 0)
			test.advance(tc.initialRTT)
			test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
			test.wantAck(initialSpace, 0)

			t.Logf("# ack of packet 2 starts loss timer for packet 1")
			test.send(initialSpace, 1, 2)
			test.advance(tc.latestRTT)
			test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{2, 3})
			test.wantAck(initialSpace, 2)

			t.Logf("# smoothed_rtt = %v", test.c.rtt.smoothedRTT)
			t.Logf("# latest_rtt = %v", test.c.rtt.latestRTT)
			t.Logf("# timeout = max(9/8 * max(smoothed_rtt, latest_rtt), 1ms)")
			t.Logf("#           (measured since packet 1 sent)")
			test.wantTimeout(tc.wantTimeout - tc.latestRTT)

			t.Logf("# advancing to the loss time causes loss of packet 1")
			test.advanceToLossTimer()
			test.wantLoss(initialSpace, 1)
		})
	}
}

func TestLossPTONotAckEliciting(t *testing.T) {
	// "When an ack-eliciting packet is transmitted,
	// the sender schedules a timer for the PTO period [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-1
	test := newLossTest(t, clientSide, lossTestOpts{})
	t.Logf("# PTO timer for first packet")
	test.send(initialSpace, 0)
	test.wantVar("smoothed_rtt", 333*time.Millisecond) // initial value
	test.wantVar("rttvar", 333*time.Millisecond/2)     // initial value
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms)")
	test.wantTimeout(999 * time.Millisecond)

	t.Logf("# sending a non-ack-eliciting packet doesn't adjust PTO")
	test.advance(333 * time.Millisecond)
	test.send(initialSpace, 1, sentPacket{
		ackEliciting: false,
	})
	test.wantVar("smoothed_rtt", 333*time.Millisecond) // unchanged
	test.wantVar("rttvar", 333*time.Millisecond/2)     // unchanged
	test.wantTimeout(666 * time.Millisecond)
}

func TestLossPTOMaxAckDelay(t *testing.T) {
	// "When the PTO is armed for Initial or Handshake packet number spaces,
	// the max_ack_delay in the PTO period computation is set to 0 [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-4
	test := newLossTest(t, clientSide, lossTestOpts{})
	t.Logf("# PTO timer for first packet")
	test.send(initialSpace, 0)
	test.wantVar("smoothed_rtt", 333*time.Millisecond) // initial value
	test.wantVar("rttvar", 333*time.Millisecond/2)     // initial value
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms)")
	test.wantTimeout(999 * time.Millisecond)

	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)

	t.Logf("# PTO timer for handshake packet")
	test.send(handshakeSpace, 0)
	test.wantVar("smoothed_rtt", 10*time.Millisecond)
	test.wantVar("rttvar", 5*time.Millisecond)
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms)")
	test.wantTimeout(30 * time.Millisecond)

	test.advance(10 * time.Millisecond)
	test.ack(handshakeSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(handshakeSpace, 0)
	test.confirmHandshake()

	t.Logf("# PTO timer for appdata packet")
	test.send(appDataSpace, 0)
	test.wantVar("smoothed_rtt", 10*time.Millisecond)
	test.wantVar("rttvar", 3750*time.Microsecond)
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms) + max_ack_delay (25ms)")
	test.wantTimeout(50 * time.Millisecond)
}

func TestLossPTOUnderTimerGranularity(t *testing.T) {
	// "The PTO period MUST be at least kGranularity [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-5
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.send(initialSpace, 0)
	test.advance(10 * time.Microsecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)

	test.send(initialSpace, 1)
	test.wantVar("smoothed_rtt", 10*time.Microsecond)
	test.wantVar("rttvar", 5*time.Microsecond)
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms)")
	test.wantTimeout(10*time.Microsecond + 1*time.Millisecond)
}

func TestLossPTOMultipleSpaces(t *testing.T) {
	// "[...] the timer MUST be set to the earlier value of the Initial and Handshake
	// packet number spaces."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-6
	test := newLossTest(t, clientSide, lossTestOpts{})
	t.Logf("# PTO timer for first packet")
	test.send(initialSpace, 0)
	test.wantVar("smoothed_rtt", 333*time.Millisecond) // initial value
	test.wantVar("rttvar", 333*time.Millisecond/2)     // initial value
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms)")
	test.wantTimeout(999 * time.Millisecond)

	t.Logf("# Initial and Handshake packets in flight, first takes precedence")
	test.advance(333 * time.Millisecond)
	test.send(handshakeSpace, 0)
	test.wantTimeout(666 * time.Millisecond)

	t.Logf("# Initial packet acked, Handshake PTO timer armed")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	test.wantTimeout(999 * time.Millisecond)

	t.Logf("# send Initial, earlier Handshake PTO takes precedence")
	test.advance(333 * time.Millisecond)
	test.send(initialSpace, 1)
	test.wantTimeout(666 * time.Millisecond)
}

func TestLossPTOHandshakeConfirmation(t *testing.T) {
	// "An endpoint MUST NOT set its PTO timer for the Application Data
	// packet number space until the handshake is confirmed."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-7
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.send(initialSpace, 0)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)

	test.send(handshakeSpace, 0)
	test.ack(handshakeSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(handshakeSpace, 0)

	test.send(appDataSpace, 0)
	test.wantNoTimeout()
}

func TestLossPTOBackoffDoubles(t *testing.T) {
	// "When a PTO timer expires, the PTO backoff MUST be increased,
	// resulting in the PTO period being set to twice its current value."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-9
	test := newLossTest(t, serverSide, lossTestOpts{})
	test.datagramReceived(1200)
	test.send(initialSpace, 0)
	test.wantVar("smoothed_rtt", 333*time.Millisecond) // initial value
	test.wantVar("rttvar", 333*time.Millisecond/2)     // initial value
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms)")
	test.wantTimeout(999 * time.Millisecond)

	t.Logf("# wait for PTO timer expiration")
	test.advanceToLossTimer()
	test.wantPTOExpired()
	test.wantNoTimeout()

	t.Logf("# PTO timer doubles")
	test.send(initialSpace, 1)
	test.wantTimeout(2 * 999 * time.Millisecond)
	test.advanceToLossTimer()
	test.wantPTOExpired()
	test.wantNoTimeout()

	t.Logf("# PTO timer doubles again")
	test.send(initialSpace, 2)
	test.wantTimeout(4 * 999 * time.Millisecond)
	test.advanceToLossTimer()
	test.wantPTOExpired()
	test.wantNoTimeout()
}

func TestLossPTOBackoffResetOnAck(t *testing.T) {
	// "The PTO backoff factor is reset when an acknowledgment is received [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-9
	test := newLossTest(t, serverSide, lossTestOpts{})
	test.datagramReceived(1200)

	t.Logf("# first ack establishes smoothed_rtt = 10ms")
	test.send(initialSpace, 0)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	t.Logf("# set rttvar for simplicity")
	test.setRTTVar(0)

	t.Logf("# send packet 1 and wait for PTO")
	test.send(initialSpace, 1)
	test.wantTimeout(11 * time.Millisecond)
	test.advanceToLossTimer()
	test.wantPTOExpired()
	test.wantNoTimeout()

	t.Logf("# send packet 2 & 3, PTO doubles")
	test.send(initialSpace, 2, 3)
	test.wantTimeout(22 * time.Millisecond)

	test.advance(10 * time.Millisecond)
	t.Logf("# check remaining PTO (22ms - 10ms elapsed)")
	test.wantTimeout(12 * time.Millisecond)

	t.Logf("# ACK to packet 2 resets PTO")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 3})
	test.wantAck(initialSpace, 1)
	test.wantAck(initialSpace, 2)

	t.Logf("# check remaining PTO (11ms - 10ms elapsed)")
	test.wantTimeout(1 * time.Millisecond)
}

func TestLossPTOBackoffNotResetOnClientInitialAck(t *testing.T) {
	// "[...] a client does not reset the PTO backoff factor on
	// receiving acknowledgments in Initial packets."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-9
	test := newLossTest(t, clientSide, lossTestOpts{})

	t.Logf("# first ack establishes smoothed_rtt = 10ms")
	test.send(initialSpace, 0)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	t.Logf("# set rttvar for simplicity")
	test.setRTTVar(0)

	t.Logf("# send packet 1 and wait for PTO")
	test.send(initialSpace, 1)
	test.wantTimeout(11 * time.Millisecond)
	test.advanceToLossTimer()
	test.wantPTOExpired()
	test.wantNoTimeout()

	t.Logf("# send more packets, PTO doubles")
	test.send(initialSpace, 2, 3)
	test.send(handshakeSpace, 0)
	test.wantTimeout(22 * time.Millisecond)

	test.advance(10 * time.Millisecond)
	t.Logf("# check remaining PTO (22ms - 10ms elapsed)")
	test.wantTimeout(12 * time.Millisecond)

	// TODO: Is this right? 6.2.1-9 says we don't reset the PTO *backoff*, not the PTO.
	// 6.2.1-8 says we reset the PTO timer when an ack-eliciting packet is sent *or
	// acknowledged*, but the pseudocode in appendix A doesn't appear to do the latter.
	t.Logf("# ACK to Initial packet does not reset PTO for client")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 3})
	test.wantAck(initialSpace, 1)
	test.wantAck(initialSpace, 2)
	t.Logf("# check remaining PTO (22ms - 10ms elapsed)")
	test.wantTimeout(12 * time.Millisecond)

	t.Logf("# ACK to handshake packet does reset PTO")
	test.ack(handshakeSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(handshakeSpace, 0)
	t.Logf("# check remaining PTO (12ms - 10ms elapsed)")
	test.wantTimeout(1 * time.Millisecond)
}

func TestLossPTONotSetWhenLossTimerSet(t *testing.T) {
	// "The PTO timer MUST NOT be set if a timer is set
	// for time threshold loss detection [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.1-12
	test := newLossTest(t, serverSide, lossTestOpts{})
	test.datagramReceived(1200)
	t.Logf("# PTO timer set for first packets sent")
	test.send(initialSpace, 0, 1)
	test.wantVar("smoothed_rtt", 333*time.Millisecond) // initial value
	test.wantVar("rttvar", 333*time.Millisecond/2)     // initial value
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms)")
	test.wantTimeout(999 * time.Millisecond)

	t.Logf("# ack of packet 1 starts loss timer for 0, PTO overidden")
	test.advance(333 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(initialSpace, 1)

	t.Logf("# latest_rtt == smoothed_rtt")
	test.wantVar("smoothed_rtt", 333*time.Millisecond)
	test.wantVar("latest_rtt", 333*time.Millisecond)
	t.Logf("# timeout = 9/8 * max(smoothed_rtt, latest_rtt) - time_since_packet_sent")
	test.wantTimeout(((333 * time.Millisecond * 9) / 8) - 333*time.Millisecond)
}

func TestLossDiscardingKeysResetsTimers(t *testing.T) {
	// "When Initial or Handshake keys are discarded,
	// the PTO and loss detection timers MUST be reset"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.2-3
	test := newLossTest(t, clientSide, lossTestOpts{})

	t.Logf("# handshake packet sent 1ms after initial")
	test.send(initialSpace, 0, 1)
	test.advance(1 * time.Millisecond)
	test.send(handshakeSpace, 0, 1)
	test.advance(9 * time.Millisecond)

	t.Logf("# ack of Initial packet 2 starts loss timer for packet 1")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(initialSpace, 1)

	test.advance(1 * time.Millisecond)
	t.Logf("# smoothed_rtt = %v", 10*time.Millisecond)
	t.Logf("# latest_rtt = %v", 10*time.Millisecond)
	t.Logf("# timeout = max(9/8 * max(smoothed_rtt, latest_rtt), 1ms)")
	t.Logf("#           (measured since Initial packet 1 sent)")
	test.wantTimeout((10 * time.Millisecond * 9 / 8) - 11*time.Millisecond)

	t.Logf("# ack of Handshake packet 2 starts loss timer for packet 1")
	test.ack(handshakeSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(handshakeSpace, 1)

	t.Logf("# dropping Initial keys sets timer to Handshake timeout")
	test.discardKeys(initialSpace)
	test.wantTimeout((10 * time.Millisecond * 9 / 8) - 10*time.Millisecond)
}

func TestLossNoPTOAtAntiAmplificationLimit(t *testing.T) {
	// "If no additional data can be sent [because the server is at the
	// anti-amplification limit], the server's PTO timer MUST NOT be armed [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.2.1-1
	test := newLossTest(t, serverSide, lossTestOpts{
		maxDatagramSize: 1 << 20, // large initial congestion window
	})
	test.datagramReceived(1200)
	test.send(initialSpace, 0, sentPacket{
		ackEliciting: true,
		inFlight:     true,
		size:         1200,
	})
	test.wantTimeout(999 * time.Millisecond)

	t.Logf("PTO timer should be disabled when at the anti-amplification limit")
	test.send(initialSpace, 1, sentPacket{
		ackEliciting: false,
		inFlight:     true,
		size:         2 * 1200,
	})
	test.wantNoTimeout()

	// "When the server receives a datagram from the client, the amplification
	// limit is increased and the server resets the PTO timer."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.2.1-2
	t.Logf("PTO timer should be reset when datagrams are received")
	test.datagramReceived(1200)
	test.wantTimeout(999 * time.Millisecond)

	// "If the PTO timer is then set to a time in the past, it is executed immediately."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.2.1-2
	test.send(initialSpace, 2, sentPacket{
		ackEliciting: true,
		inFlight:     true,
		size:         3 * 1200,
	})
	test.wantNoTimeout()
	t.Logf("resetting expired PTO timer should exeute immediately")
	test.advance(1000 * time.Millisecond)
	test.datagramReceived(1200)
	test.wantPTOExpired()
	test.wantNoTimeout()
}

func TestLossClientSetsPTOWhenHandshakeUnacked(t *testing.T) {
	// "[...] the client MUST set the PTO timer if the client has not
	// received an acknowledgment for any of its Handshake packets and
	// the handshake is not confirmed [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.2.2.1-3
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.send(initialSpace, 0)

	test.wantVar("smoothed_rtt", 333*time.Millisecond) // initial value
	test.wantVar("rttvar", 333*time.Millisecond/2)     // initial value
	t.Logf("# PTO = smoothed_rtt + max(4*rttvar, 1ms)")
	test.wantTimeout(999 * time.Millisecond)

	test.advance(333 * time.Millisecond)
	test.wantTimeout(666 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	t.Logf("# PTO timer set for a client before handshake ack even if no packets in flight")
	test.wantTimeout(999 * time.Millisecond)

	test.advance(333 * time.Millisecond)
	test.wantTimeout(666 * time.Millisecond)
}

func TestLossKeysDiscarded(t *testing.T) {
	// "The sender MUST discard all recovery state associated with
	// [packets in number spaces with discarded keys] and MUST remove
	// them from the count of bytes in flight."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-6.4-1
	test := newLossTest(t, clientSide, lossTestOpts{})
	test.send(initialSpace, 0, testSentPacketSize(1200))
	test.send(handshakeSpace, 0, testSentPacketSize(600))
	test.wantVar("bytes_in_flight", 1800)

	test.discardKeys(initialSpace)
	test.wantVar("bytes_in_flight", 600)

	test.discardKeys(handshakeSpace)
	test.wantVar("bytes_in_flight", 0)
}

func TestLossInitialCongestionWindow(t *testing.T) {
	// "Endpoints SHOULD use an initial congestion window of [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.2-1

	// "[...] 10 times the maximum datagram size [...]"
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	t.Logf("# congestion_window = 10*max_datagram_size (1200)")
	test.wantVar("congestion_window", 12000)

	// "[...] while limiting the window to the larger of 14720 bytes [...]"
	test = newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1500,
	})
	t.Logf("# congestion_window limited to 14720 bytes")
	test.wantVar("congestion_window", 14720)

	// "[...] or twice the maximum datagram size."
	test = newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 10000,
	})
	t.Logf("# congestion_window limited to 2*max_datagram_size (10000)")
	test.wantVar("congestion_window", 20000)

	for _, tc := range []struct {
		maxDatagramSize  int
		wantInitialBurst int
	}{{
		// "[...] 10 times the maximum datagram size [...]"
		maxDatagramSize:  1200,
		wantInitialBurst: 12000,
	}, {
		// "[...] while limiting the window to the larger of 14720 bytes [...]"
		maxDatagramSize:  1500,
		wantInitialBurst: 14720,
	}, {
		// "[...] or twice the maximum datagram size."
		maxDatagramSize:  10000,
		wantInitialBurst: 20000,
	}} {
		t.Run(fmt.Sprintf("max_datagram_size=%v", tc.maxDatagramSize), func(t *testing.T) {
			test := newLossTest(t, clientSide, lossTestOpts{
				maxDatagramSize: tc.maxDatagramSize,
			})

			var num packetNumber
			window := tc.wantInitialBurst
			for window >= tc.maxDatagramSize {
				t.Logf("# %v bytes of initial congestion window remain", window)
				test.send(initialSpace, num, sentPacket{
					ackEliciting: true,
					inFlight:     true,
					size:         tc.maxDatagramSize,
				})
				window -= tc.maxDatagramSize
				num++
			}
			t.Logf("# congestion window (%v) < max_datagram_size, congestion control blocks send", window)
			test.wantSendLimit(ccLimited)
		})
	}
}

func TestLossBytesInFlight(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	t.Logf("# sent packets are added to bytes_in_flight")
	test.wantVar("bytes_in_flight", 0)
	test.send(initialSpace, 0, testSentPacketSize(1200))
	test.wantVar("bytes_in_flight", 1200)
	test.send(initialSpace, 1, testSentPacketSize(800))
	test.wantVar("bytes_in_flight", 2000)

	t.Logf("# acked packets are removed from bytes_in_flight")
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{1, 2})
	test.wantAck(initialSpace, 1)
	test.wantVar("bytes_in_flight", 1200)

	t.Logf("# lost packets are removed from bytes_in_flight")
	test.advanceToLossTimer()
	test.wantLoss(initialSpace, 0)
	test.wantVar("bytes_in_flight", 0)
}

func TestLossCongestionWindowLimit(t *testing.T) {
	// "An endpoint MUST NOT send a packet if it would cause bytes_in_flight
	// [...] to be larger than the congestion window [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7-7
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	t.Logf("# consume the initial congestion window")
	test.send(initialSpace, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, testSentPacketSize(1200))
	test.wantSendLimit(ccLimited)

	t.Logf("# give the pacer bucket time to refill")
	test.advance(333 * time.Millisecond) // initial RTT

	t.Logf("# sending limited by congestion window, not the pacer")
	test.wantVar("congestion_window", 12000)
	test.wantVar("bytes_in_flight", 12000)
	test.wantVar("pacer_bucket", 12000)
	test.wantSendLimit(ccLimited)

	t.Logf("# receiving an ack opens up the congestion window")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	test.wantSendLimit(ccOK)
}

func TestLossCongestionStates(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	t.Logf("# consume the initial congestion window")
	test.send(initialSpace, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, testSentPacketSize(1200))
	test.wantSendLimit(ccLimited)
	test.wantVar("congestion_window", 12000)

	// "While a sender is in slow start, the congestion window
	// increases by the number of bytes acknowledged [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.3.1-2
	test.advance(333 * time.Millisecond)
	t.Logf("# congestion window increases by number of bytes acked (1200)")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	test.wantVar("congestion_window", 13200) // 12000 + 1200

	t.Logf("# congestion window increases by number of bytes acked (2400)")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 3})
	test.wantAck(initialSpace, 1, 2)
	test.wantVar("congestion_window", 15600) // 12000 + 3*1200

	// TODO: ECN-CE count

	// "The sender MUST exit slow start and enter a recovery period
	// when a packet is lost [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.3.1-3
	t.Logf("# loss of a packet triggers entry to a recovery period")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{6, 7})
	test.wantAck(initialSpace, 6)
	test.wantLoss(initialSpace, 3)

	// "On entering a recovery period, a sender MUST set the slow start
	// threshold to half the value of the congestion window when loss is detected."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.3.2-2
	t.Logf("# slow_start_threshold = congestion_window / 2")
	test.wantVar("slow_start_threshold", 7800) // 15600/2

	// "[...] a single packet can be sent prior to reduction [of the congestion window]."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.3.2-3
	test.send(initialSpace, 10, testSentPacketSize(1200))

	// "The congestion window MUST be set to the reduced value of the slow start
	// threshold before exiting the recovery period."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.3.2-2
	t.Logf("# congestion window reduced to slow start threshold")
	test.wantVar("congestion_window", 7800)

	t.Logf("# acks for packets sent before recovery started do not affect congestion")
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 10})
	test.wantAck(initialSpace, 4, 5, 7, 8, 9)
	test.wantVar("slow_start_threshold", 7800)
	test.wantVar("congestion_window", 7800)

	// "A recovery period ends and the sender enters congestion avoidance when
	// a packet sent during the recovery period is acknowledged."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.3.2-5
	t.Logf("# recovery ends and congestion avoidance begins when packet 10 is acked")
	test.advance(333 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 11})
	test.wantAck(initialSpace, 10)

	// "[...] limit the increase to the congestion window to at most one
	// maximum datagram size for each congestion window that is acknowledged."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.3.3-2
	t.Logf("# after processing acks for one congestion window's worth of data...")
	test.send(initialSpace, 11, 12, 13, 14, 15, 16, testSentPacketSize(1200))
	test.advance(333 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 17})
	test.wantAck(initialSpace, 11, 12, 13, 14, 15, 16)
	t.Logf("# ...congestion window increases by max_datagram_size")
	test.wantVar("congestion_window", 9000) // 7800 + 1200

	// "The sender exits congestion avoidance and enters a recovery period
	// when a packet is lost [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.3.3-3
	test.send(initialSpace, 17, 18, 19, 20, 21, testSentPacketSize(1200))
	test.advance(333 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{18, 21})
	test.wantAck(initialSpace, 18, 19, 20)
	test.wantLoss(initialSpace, 17)
	t.Logf("# slow_start_threshold = congestion_window / 2")
	test.wantVar("slow_start_threshold", 4500)
}

func TestLossMinimumCongestionWindow(t *testing.T) {
	// "The RECOMMENDED [minimum congestion window] is 2 * max_datagram_size."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.2-4
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	test.send(initialSpace, 0, 1, 2, 3, testSentPacketSize(1200))
	test.wantVar("congestion_window", 12000)

	t.Logf("# enter recovery")
	test.advance(333 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{3, 4})
	test.wantAck(initialSpace, 3)
	test.wantLoss(initialSpace, 0)
	test.wantVar("congestion_window", 6000)

	t.Logf("# enter congestion avoidance and return to recovery")
	test.send(initialSpace, 4, 5, 6, 7)
	test.advance(333 * time.Millisecond)
	test.wantLoss(initialSpace, 1, 2)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{7, 8})
	test.wantAck(initialSpace, 7)
	test.wantLoss(initialSpace, 4)
	test.wantVar("congestion_window", 3000)

	t.Logf("# enter congestion avoidance and return to recovery")
	test.send(initialSpace, 8, 9, 10, 11)
	test.advance(333 * time.Millisecond)
	test.wantLoss(initialSpace, 5, 6)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{11, 12})
	test.wantAck(initialSpace, 11)
	test.wantLoss(initialSpace, 8)
	t.Logf("# congestion window does not fall below 2*max_datagram_size")
	test.wantVar("congestion_window", 2400)

	t.Logf("# enter congestion avoidance and return to recovery")
	test.send(initialSpace, 12, 13, 14, 15)
	test.advance(333 * time.Millisecond)
	test.wantLoss(initialSpace, 9, 10)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{15, 16})
	test.wantAck(initialSpace, 15)
	test.wantLoss(initialSpace, 12)
	t.Logf("# congestion window does not fall below 2*max_datagram_size")
	test.wantVar("congestion_window", 2400)
}

func TestLossPersistentCongestion(t *testing.T) {
	// "When persistent congestion is declared, the sender's congestion
	// window MUST be reduced to the minimum congestion window [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.6.2-6
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	test.send(initialSpace, 0, testSentPacketSize(1200))
	test.c.cc.setUnderutilized(nil, true)

	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)

	t.Logf("# set rttvar for simplicity")
	test.setRTTVar(0)
	test.wantVar("smoothed_rtt", 10*time.Millisecond)
	t.Logf("# persistent congestion duration = 3*(smoothed_rtt + timerGranularity + max_ack_delay)")
	t.Logf("# persistent congestion duration = 108ms")

	t.Logf("# sending packets 1-5 over 108ms")
	test.send(initialSpace, 1, testSentPacketSize(1200))

	test.advance(11 * time.Millisecond) // total 11ms
	test.wantPTOExpired()
	test.send(initialSpace, 2, testSentPacketSize(1200))

	test.advance(22 * time.Millisecond) // total 33ms
	test.wantPTOExpired()
	test.send(initialSpace, 3, testSentPacketSize(1200))

	test.advance(44 * time.Millisecond) // total 77ms
	test.wantPTOExpired()
	test.send(initialSpace, 4, testSentPacketSize(1200))

	test.advance(31 * time.Millisecond) // total 108ms
	test.send(initialSpace, 5, testSentPacketSize(1200))
	t.Logf("# 108ms between packets 1-5")

	test.wantVar("congestion_window", 12000)
	t.Logf("# triggering loss of packets 1-5")
	test.send(initialSpace, 6, 7, 8, testSentPacketSize(1200))
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{8, 9})
	test.wantAck(initialSpace, 8)
	test.wantLoss(initialSpace, 1, 2, 3, 4, 5)

	t.Logf("# lost packets spanning persistent congestion duration")
	t.Logf("# congestion_window = 2 * max_datagram_size (minimum)")
	test.wantVar("congestion_window", 2400)
}

func TestLossSimplePersistentCongestion(t *testing.T) {
	// Simpler version of TestLossPersistentCongestion which acts as a
	// base for subsequent tests.
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})

	t.Logf("# establish initial RTT sample")
	test.send(initialSpace, 0, testSentPacketSize(1200))
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)

	t.Logf("# send two packets spanning persistent congestion duration")
	test.send(initialSpace, 1, testSentPacketSize(1200))
	t.Logf("# 2000ms >> persistent congestion duration")
	test.advance(2000 * time.Millisecond)
	test.wantPTOExpired()
	test.send(initialSpace, 2, testSentPacketSize(1200))

	t.Logf("# trigger loss of previous packets")
	test.advance(10 * time.Millisecond)
	test.send(initialSpace, 3, testSentPacketSize(1200))
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{3, 4})
	test.wantAck(initialSpace, 3)
	test.wantLoss(initialSpace, 1, 2)

	t.Logf("# persistent congestion detected")
	test.wantVar("congestion_window", 2400)
}

func TestLossPersistentCongestionAckElicitingPackets(t *testing.T) {
	// "These two packets MUST be ack-eliciting [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.6.2-3
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})

	t.Logf("# establish initial RTT sample")
	test.send(initialSpace, 0, testSentPacketSize(1200))
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)

	t.Logf("# send two packets spanning persistent congestion duration")
	test.send(initialSpace, 1, testSentPacketSize(1200))
	t.Logf("# 2000ms >> persistent congestion duration")
	test.advance(2000 * time.Millisecond)
	test.wantPTOExpired()
	test.send(initialSpace, 2, sentPacket{
		inFlight:     true,
		ackEliciting: false,
		size:         1200,
	})
	test.send(initialSpace, 3, testSentPacketSize(1200)) // PTO probe

	t.Logf("# trigger loss of previous packets")
	test.advance(10 * time.Millisecond)
	test.send(initialSpace, 4, testSentPacketSize(1200))
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{3, 5})
	test.wantAck(initialSpace, 3)
	test.wantAck(initialSpace, 4)
	test.wantLoss(initialSpace, 1, 2)

	t.Logf("# persistent congestion not detected: packet 2 is not ack-eliciting")
	test.wantVar("congestion_window", (12000+1200+1200-1200)/2)
}

func TestLossNoPersistentCongestionWithoutRTTSample(t *testing.T) {
	// "The persistent congestion period SHOULD NOT start until there
	// is at least one RTT sample."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.6.2-4
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})

	t.Logf("# packets sent before initial RTT sample")
	test.send(initialSpace, 0, testSentPacketSize(1200))
	test.advance(2000 * time.Millisecond)
	test.wantPTOExpired()
	test.send(initialSpace, 1, testSentPacketSize(1200))

	test.advance(10 * time.Millisecond)
	test.send(initialSpace, 2, testSentPacketSize(1200))

	t.Logf("# first ack establishes RTT sample")
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{2, 3})
	test.wantAck(initialSpace, 2)
	test.wantLoss(initialSpace, 0, 1)

	t.Logf("# loss of packets before initial RTT sample does not cause persistent congestion")
	test.wantVar("congestion_window", 12000/2)
}

func TestLossPacerRefillRate(t *testing.T) {
	// "A sender SHOULD pace sending of all in-flight packets based on
	// input from the congestion controller."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.7-1
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	t.Logf("# consume the initial congestion window")
	test.send(initialSpace, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, testSentPacketSize(1200))
	test.wantSendLimit(ccLimited)
	test.wantVar("pacer_bucket", 0)
	test.wantVar("congestion_window", 12000)

	t.Logf("# first RTT sample establishes smoothed_rtt")
	rtt := 100 * time.Millisecond
	test.advance(rtt)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 10})
	test.wantAck(initialSpace, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
	test.wantVar("congestion_window", 24000) // 12000 + 10*1200
	test.wantVar("smoothed_rtt", rtt)

	t.Logf("# advance 1 RTT to let the pacer bucket refill completely")
	test.advance(100 * time.Millisecond)
	t.Logf("# pacer_bucket = initial_congestion_window")
	test.wantVar("pacer_bucket", 12000)

	t.Logf("# consume capacity from the pacer bucket")
	test.send(initialSpace, 10, testSentPacketSize(1200))
	test.wantVar("pacer_bucket", 10800) // 12000 - 1200
	test.send(initialSpace, 11, testSentPacketSize(600))
	test.wantVar("pacer_bucket", 10200) // 10800 - 600
	test.send(initialSpace, 12, testSentPacketSize(600))
	test.wantVar("pacer_bucket", 9600) // 10200 - 600
	test.send(initialSpace, 13, 14, 15, 16, testSentPacketSize(1200))
	test.wantVar("pacer_bucket", 4800) // 9600 - 4*1200

	t.Logf("# advance 1/10 of an RTT, bucket refills")
	test.advance(rtt / 10)
	t.Logf("# pacer_bucket += 1.25 * (1/10) * congestion_window")
	t.Logf("#              += 3000")
	test.wantVar("pacer_bucket", 7800)
}

func TestLossPacerNextSendTime(t *testing.T) {
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	t.Logf("# consume the initial congestion window")
	test.send(initialSpace, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, testSentPacketSize(1200))
	test.wantSendLimit(ccLimited)
	test.wantVar("pacer_bucket", 0)
	test.wantVar("congestion_window", 12000)

	t.Logf("# first RTT sample establishes smoothed_rtt")
	rtt := 100 * time.Millisecond
	test.advance(rtt)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 10})
	test.wantAck(initialSpace, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
	test.wantVar("congestion_window", 24000) // 12000 + 10*1200
	test.wantVar("smoothed_rtt", rtt)

	t.Logf("# advance 1 RTT to let the pacer bucket refill completely")
	test.advance(100 * time.Millisecond)
	t.Logf("# pacer_bucket = initial_congestion_window")
	test.wantVar("pacer_bucket", 12000)

	t.Logf("# consume the refilled pacer bucket")
	test.send(initialSpace, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, testSentPacketSize(1200))
	test.wantSendLimit(ccPaced)

	t.Logf("# refill rate = 1.25 * congestion_window / rtt")
	test.wantSendDelay(rtt / 25) // rtt / (1.25 * 24000 / 1200)

	t.Logf("# no capacity available yet")
	test.advance(rtt / 50)
	test.wantVar("pacer_bucket", -600)
	test.wantSendLimit(ccPaced)

	t.Logf("# capacity available")
	test.advance(rtt / 50)
	test.wantVar("pacer_bucket", 0)
	test.wantSendLimit(ccOK)
}

func TestLossCongestionWindowUnderutilized(t *testing.T) {
	// "When bytes in flight is smaller than the congestion window
	// and sending is not pacing limited [...] the congestion window
	// SHOULD NOT be increased in either slow start or congestion avoidance."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.8-1
	test := newLossTest(t, clientSide, lossTestOpts{
		maxDatagramSize: 1200,
	})
	test.send(initialSpace, 0, testSentPacketSize(1200))
	test.setUnderutilized(true)
	t.Logf("# underutilized: %v", test.c.cc.underutilized)
	test.wantVar("congestion_window", 12000)

	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 1})
	test.wantAck(initialSpace, 0)
	t.Logf("# congestion window does not increase, because window is underutilized")
	test.wantVar("congestion_window", 12000)

	t.Logf("# refill pacer bucket")
	test.advance(10 * time.Millisecond)
	test.wantVar("pacer_bucket", 12000)

	test.send(initialSpace, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, testSentPacketSize(1200))
	test.setUnderutilized(false)
	test.advance(10 * time.Millisecond)
	test.ack(initialSpace, 0*time.Millisecond, i64range[packetNumber]{0, 11})
	test.wantAck(initialSpace, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
	t.Logf("# congestion window increases")
	test.wantVar("congestion_window", 24000)
}

type lossTest struct {
	t      *testing.T
	c      lossState
	now    time.Time
	fates  map[spaceNum]packetFate
	failed bool
}

type lossTestOpts struct {
	maxDatagramSize int
}

func newLossTest(t *testing.T, side connSide, opts lossTestOpts) *lossTest {
	c := &lossTest{
		t:     t,
		now:   time.Date(2000, 1, 1, 0, 0, 0, 0, time.UTC),
		fates: make(map[spaceNum]packetFate),
	}
	maxDatagramSize := 1200
	if opts.maxDatagramSize != 0 {
		maxDatagramSize = opts.maxDatagramSize
	}
	c.c.init(side, maxDatagramSize, c.now)
	t.Cleanup(func() {
		if !c.failed {
			c.checkUnexpectedEvents()
		}
	})
	return c
}

type spaceNum struct {
	space numberSpace
	num   packetNumber
}

func (c *lossTest) checkUnexpectedEvents() {
	c.t.Helper()
	for sn, fate := range c.fates {
		c.t.Errorf("ERROR: unexpected %v: %v %v", fate, sn.space, sn.num)
	}
	if c.c.ptoExpired {
		c.t.Errorf("ERROR: PTO timer unexpectedly expired")
	}
}

func (c *lossTest) setSmoothedRTT(d time.Duration) {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.t.Logf("set smoothed_rtt to %v", d)
	c.c.rtt.smoothedRTT = d
}

func (c *lossTest) setRTTVar(d time.Duration) {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.t.Logf("set rttvar to %v", d)
	c.c.rtt.rttvar = d
}

func (c *lossTest) setUnderutilized(v bool) {
	c.t.Logf("set congestion window underutilized: %v", v)
	c.c.cc.setUnderutilized(nil, v)
}

func (c *lossTest) advance(d time.Duration) {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.t.Logf("advance time %v", d)
	c.now = c.now.Add(d)
	c.c.advance(c.now, c.onAckOrLoss)
}

func (c *lossTest) advanceToLossTimer() {
	c.t.Helper()
	c.checkUnexpectedEvents()
	d := c.c.timer.Sub(c.now)
	c.t.Logf("advance time %v (up to loss timer)", d)
	if d < 0 {
		c.t.Fatalf("loss timer is in the past")
	}
	c.now = c.c.timer
	c.c.advance(c.now, c.onAckOrLoss)
}

type testSentPacketSize int

func (c *lossTest) send(spaceID numberSpace, opts ...any) {
	c.t.Helper()
	c.checkUnexpectedEvents()
	var nums []packetNumber
	prototype := sentPacket{
		ackEliciting: true,
		inFlight:     true,
	}
	for _, o := range opts {
		switch o := o.(type) {
		case sentPacket:
			prototype = o
		case testSentPacketSize:
			prototype.size = int(o)
		case int:
			nums = append(nums, packetNumber(o))
		case packetNumber:
			nums = append(nums, o)
		case i64range[packetNumber]:
			for num := o.start; num < o.end; num++ {
				nums = append(nums, num)
			}
		}
	}
	c.t.Logf("send %v %v", spaceID, nums)
	limit, _ := c.c.sendLimit(c.now)
	if prototype.inFlight && limit != ccOK {
		c.t.Fatalf("congestion control blocks sending packet")
	}
	if !prototype.inFlight && limit == ccBlocked {
		c.t.Fatalf("congestion control blocks sending packet")
	}
	for _, num := range nums {
		sent := &sentPacket{}
		*sent = prototype
		sent.num = num
		c.c.packetSent(c.now, nil, spaceID, sent)
	}
}

func (c *lossTest) datagramReceived(size int) {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.t.Logf("receive %v-byte datagram", size)
	c.c.datagramReceived(c.now, size)
}

func (c *lossTest) ack(spaceID numberSpace, ackDelay time.Duration, rs ...i64range[packetNumber]) {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.c.receiveAckStart()
	var acked rangeset[packetNumber]
	for _, r := range rs {
		c.t.Logf("ack %v delay=%v [%v,%v)", spaceID, ackDelay, r.start, r.end)
		acked.add(r.start, r.end)
	}
	for i, r := range rs {
		c.t.Logf("ack %v delay=%v [%v,%v)", spaceID, ackDelay, r.start, r.end)
		c.c.receiveAckRange(c.now, spaceID, i, r.start, r.end, c.onAckOrLoss)
	}
	c.c.receiveAckEnd(c.now, nil, spaceID, ackDelay, c.onAckOrLoss)
}

func (c *lossTest) onAckOrLoss(space numberSpace, sent *sentPacket, fate packetFate) {
	c.t.Logf("%v %v %v", fate, space, sent.num)
	if _, ok := c.fates[spaceNum{space, sent.num}]; ok {
		c.t.Errorf("ERROR: duplicate %v for %v %v", fate, space, sent.num)
	}
	c.fates[spaceNum{space, sent.num}] = fate
}

func (c *lossTest) confirmHandshake() {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.t.Logf("confirm handshake")
	c.c.confirmHandshake()
}

func (c *lossTest) validateClientAddress() {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.t.Logf("validate client address")
	c.c.validateClientAddress()
}

func (c *lossTest) discardKeys(spaceID numberSpace) {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.t.Logf("discard %s keys", spaceID)
	c.c.discardKeys(c.now, nil, spaceID)
}

func (c *lossTest) setMaxAckDelay(d time.Duration) {
	c.t.Helper()
	c.checkUnexpectedEvents()
	c.t.Logf("set max_ack_delay = %v", d)
	c.c.setMaxAckDelay(d)
}

func (c *lossTest) wantAck(spaceID numberSpace, nums ...packetNumber) {
	c.t.Helper()
	for _, num := range nums {
		if c.fates[spaceNum{spaceID, num}] != packetAcked {
			c.t.Fatalf("expected ack for %v %v\n", spaceID, num)
		}
		delete(c.fates, spaceNum{spaceID, num})
	}
}

func (c *lossTest) wantLoss(spaceID numberSpace, nums ...packetNumber) {
	c.t.Helper()
	for _, num := range nums {
		if c.fates[spaceNum{spaceID, num}] != packetLost {
			c.t.Fatalf("expected loss of %v %v\n", spaceID, num)
		}
		delete(c.fates, spaceNum{spaceID, num})
	}
}

func (c *lossTest) wantPTOExpired() {
	c.t.Helper()
	if !c.c.ptoExpired {
		c.t.Fatalf("expected PTO timer to expire")
	} else {
		c.t.Logf("PTO TIMER EXPIRED")
	}
	c.c.ptoExpired = false
}

func (l ccLimit) String() string {
	switch l {
	case ccOK:
		return "ccOK"
	case ccBlocked:
		return "ccBlocked"
	case ccLimited:
		return "ccLimited"
	case ccPaced:
		return "ccPaced"
	}
	return "BUG"
}

func (c *lossTest) wantSendLimit(want ccLimit) {
	c.t.Helper()
	if got, _ := c.c.sendLimit(c.now); got != want {
		c.t.Fatalf("congestion control send limit is %v, want %v", got, want)
	}
}

func (c *lossTest) wantSendDelay(want time.Duration) {
	c.t.Helper()
	limit, next := c.c.sendLimit(c.now)
	if limit != ccPaced {
		c.t.Fatalf("congestion control limit is %v, want %v", limit, ccPaced)
	}
	got := next.Sub(c.now)
	if got != want {
		c.t.Fatalf("delay until next send is %v, want %v", got, want)
	}
}

func (c *lossTest) wantVar(name string, want any) {
	c.t.Helper()
	var got any
	switch name {
	case "latest_rtt":
		got = c.c.rtt.latestRTT
	case "min_rtt":
		got = c.c.rtt.minRTT
	case "smoothed_rtt":
		got = c.c.rtt.smoothedRTT
	case "rttvar":
		got = c.c.rtt.rttvar
	case "congestion_window":
		got = c.c.cc.congestionWindow
	case "slow_start_threshold":
		got = c.c.cc.slowStartThreshold
	case "bytes_in_flight":
		got = c.c.cc.bytesInFlight
	case "pacer_bucket":
		got = c.c.pacer.bucket
	default:
		c.t.Fatalf("unknown var %q", name)
	}
	if got != want {
		c.t.Fatalf("%v = %v, want %v\n", name, got, want)
	} else {
		c.t.Logf("%v = %v", name, got)
	}
}

func (c *lossTest) wantTimeout(want time.Duration) {
	c.t.Helper()
	if c.c.timer.IsZero() {
		c.t.Fatalf("loss detection timer is not set, want %v", want)
	}
	got := c.c.timer.Sub(c.now)
	if got != want {
		c.t.Fatalf("loss detection timer expires in %v, want %v", got, want)
	}
	c.t.Logf("loss detection timer expires in %v", got)
}

func (c *lossTest) wantNoTimeout() {
	c.t.Helper()
	if !c.c.timer.IsZero() {
		d := c.c.timer.Sub(c.now)
		c.t.Fatalf("loss detection timer expires in %v, want not set", d)
	}
	c.t.Logf("loss detection timer is not set")
}

func (f packetFate) String() string {
	switch f {
	case packetAcked:
		return "ACK"
	case packetLost:
		return "LOSS"
	default:
		panic("unknown packetFate")
	}
}
