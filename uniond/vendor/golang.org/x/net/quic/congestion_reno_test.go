// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"testing"
	"time"
)

func TestRenoInitialCongestionWindow(t *testing.T) {
	// https://www.rfc-editor.org/rfc/rfc9002#section-7.2-1
	for _, test := range []struct {
		maxDatagramSize int
		wantWindow      int
	}{{
		// "[...] ten times the maximum datagram size [...]"
		maxDatagramSize: 1200,
		wantWindow:      12000,
	}, {
		// [...] limiting the window to the larger of 14,720 bytes [...]"
		maxDatagramSize: 1500,
		wantWindow:      14720,
	}, {
		// [...] or twice the maximum datagram size."
		maxDatagramSize: 15000,
		wantWindow:      30000,
	}} {
		c := newReno(test.maxDatagramSize)
		if got, want := c.congestionWindow, test.wantWindow; got != want {
			t.Errorf("newReno(max_datagram_size=%v): congestion_window = %v, want %v",
				test.maxDatagramSize, got, want)
		}
	}
}

func TestRenoSlowStartWindowIncreases(t *testing.T) {
	// "[...] the congestion window increases by the number of bytes acknowledged [...]"
	// https://www.rfc-editor.org/rfc/rfc9002#section-7.3.1-2
	test := newRenoTest(t, 1200)

	p0 := test.packetSent(initialSpace, 1200)
	test.wantVar("congestion_window", 12000)
	test.packetAcked(initialSpace, p0)
	test.packetBatchEnd(initialSpace)
	test.wantVar("congestion_window", 12000+1200)

	p1 := test.packetSent(handshakeSpace, 600)
	p2 := test.packetSent(handshakeSpace, 300)
	test.packetAcked(handshakeSpace, p1)
	test.packetAcked(handshakeSpace, p2)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("congestion_window", 12000+1200+600+300)
}

func TestRenoSlowStartToRecovery(t *testing.T) {
	// "The sender MUST exit slow start and enter a recovery period
	// when a packet is lost [...]"
	// https://www.rfc-editor.org/rfc/rfc9002#section-7.3.1-3
	test := newRenoTest(t, 1200)

	p0 := test.packetSent(initialSpace, 1200)
	p1 := test.packetSent(initialSpace, 1200)
	p2 := test.packetSent(initialSpace, 1200)
	p3 := test.packetSent(initialSpace, 1200)
	test.wantVar("congestion_window", 12000)

	t.Logf("# ACK triggers packet loss, sender enters recovery")
	test.advance(1 * time.Millisecond)
	test.packetAcked(initialSpace, p3)
	test.packetLost(initialSpace, p0)
	test.packetBatchEnd(initialSpace)

	// "[...] set the slow start threshold to half the value of
	// the congestion window when loss is detected."
	// https://www.rfc-editor.org/rfc/rfc9002#section-7.3.2-2
	test.wantVar("slow_start_threshold", 6000)

	t.Logf("# packet loss in recovery does not change congestion window")
	test.packetLost(initialSpace, p1)
	test.packetBatchEnd(initialSpace)

	t.Logf("# ack of packet from before recovery does not change congestion window")
	test.packetAcked(initialSpace, p2)
	test.packetBatchEnd(initialSpace)

	p4 := test.packetSent(initialSpace, 1200)
	test.packetAcked(initialSpace, p4)
	test.packetBatchEnd(initialSpace)

	// "The congestion window MUST be set to the reduced value of
	// the slow start threshold before exiting the recovery period."
	// https://www.rfc-editor.org/rfc/rfc9002#section-7.3.2-2
	test.wantVar("congestion_window", 6000)
}

func TestRenoRecoveryToCongestionAvoidance(t *testing.T) {
	// "A sender in congestion avoidance [limits] the increase
	// to the congestion window to at most one maximum datagram size
	// for each congestion window that is acknowledged."
	// https://www.rfc-editor.org/rfc/rfc9002#section-7.3.3-2
	test := newRenoTest(t, 1200)

	p0 := test.packetSent(initialSpace, 1200)
	p1 := test.packetSent(initialSpace, 1200)
	p2 := test.packetSent(initialSpace, 1200)
	test.advance(1 * time.Millisecond)
	test.packetAcked(initialSpace, p1)
	test.packetLost(initialSpace, p0)
	test.packetBatchEnd(initialSpace)

	p3 := test.packetSent(initialSpace, 1000)
	test.advance(1 * time.Millisecond)
	test.packetAcked(initialSpace, p3)
	test.packetBatchEnd(initialSpace)

	test.wantVar("congestion_window", 6000)
	test.wantVar("slow_start_threshold", 6000)
	test.wantVar("congestion_pending_acks", 1000)

	t.Logf("# ack of packet from before recovery does not change congestion window")
	test.packetAcked(initialSpace, p2)
	test.packetBatchEnd(initialSpace)
	test.wantVar("congestion_pending_acks", 1000)

	for i := 0; i < 6; i++ {
		p := test.packetSent(initialSpace, 1000)
		test.packetAcked(initialSpace, p)
	}
	test.packetBatchEnd(initialSpace)
	t.Logf("# congestion window increased by max_datagram_size")
	test.wantVar("congestion_window", 6000+1200)
	test.wantVar("congestion_pending_acks", 1000)
}

func TestRenoMinimumCongestionWindow(t *testing.T) {
	// "The RECOMMENDED [minimum congestion window] is 2 * max_datagram_size."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.2-4
	test := newRenoTest(t, 1200)

	p0 := test.packetSent(handshakeSpace, 1200)
	p1 := test.packetSent(handshakeSpace, 1200)
	test.advance(1 * time.Millisecond)
	test.packetAcked(handshakeSpace, p1)
	test.packetLost(handshakeSpace, p0)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 6000)
	test.wantVar("congestion_window", 6000)

	test.advance(1 * time.Millisecond)
	p2 := test.packetSent(handshakeSpace, 1200)
	p3 := test.packetSent(handshakeSpace, 1200)
	test.advance(1 * time.Millisecond)
	test.packetAcked(handshakeSpace, p3)
	test.packetLost(handshakeSpace, p2)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 3000)
	test.wantVar("congestion_window", 3000)

	p4 := test.packetSent(handshakeSpace, 1200)
	p5 := test.packetSent(handshakeSpace, 1200)
	test.advance(1 * time.Millisecond)
	test.packetAcked(handshakeSpace, p4)
	test.packetLost(handshakeSpace, p5)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 1500)
	test.wantVar("congestion_window", 2400) // minimum

	p6 := test.packetSent(handshakeSpace, 1200)
	p7 := test.packetSent(handshakeSpace, 1200)
	test.advance(1 * time.Millisecond)
	test.packetAcked(handshakeSpace, p7)
	test.packetLost(handshakeSpace, p6)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 1200) // half congestion window
	test.wantVar("congestion_window", 2400)    // minimum
}

func TestRenoSlowStartToCongestionAvoidance(t *testing.T) {
	test := newRenoTest(t, 1200)
	test.setRTT(1*time.Millisecond, 0)

	t.Logf("# enter recovery with persistent congestion")
	p0 := test.packetSent(handshakeSpace, 1200)
	test.advance(1 * time.Second) // larger than persistent congestion duration
	p1 := test.packetSent(handshakeSpace, 1200)
	p2 := test.packetSent(handshakeSpace, 1200)
	test.advance(1 * time.Millisecond)
	test.packetAcked(handshakeSpace, p2)
	test.packetLost(handshakeSpace, p0)
	test.packetLost(handshakeSpace, p1)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 6000)
	test.wantVar("congestion_window", 2400) // minimum in persistent congestion
	test.wantVar("congestion_pending_acks", 0)

	t.Logf("# enter slow start on new ack")
	p3 := test.packetSent(handshakeSpace, 1200)
	test.packetAcked(handshakeSpace, p3)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("congestion_window", 3600)
	test.wantVar("congestion_pending_acks", 0)

	t.Logf("# enter congestion avoidance after reaching slow_start_threshold")
	p4 := test.packetSent(handshakeSpace, 1200)
	p5 := test.packetSent(handshakeSpace, 1200)
	p6 := test.packetSent(handshakeSpace, 1200)
	test.packetAcked(handshakeSpace, p4)
	test.packetAcked(handshakeSpace, p5)
	test.packetAcked(handshakeSpace, p6)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("congestion_window", 6000)
	test.wantVar("congestion_pending_acks", 1200)
}

func TestRenoPersistentCongestionDurationExceeded(t *testing.T) {
	// "When persistent congestion is declared, the sender's congestion
	// window MUST be reduced to the minimum congestion window [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.6.2-6
	test := newRenoTest(t, 1200)
	test.setRTT(10*time.Millisecond, 3*time.Millisecond)
	test.maxAckDelay = 25 * time.Millisecond

	t.Logf("persistent congesion duration is 3 * (10ms + 4*3ms + 25ms) = 141ms")
	p0 := test.packetSent(handshakeSpace, 1200)
	test.advance(142 * time.Millisecond) // larger than persistent congestion duration
	p1 := test.packetSent(handshakeSpace, 1200)
	p2 := test.packetSent(handshakeSpace, 1200)
	test.packetAcked(handshakeSpace, p2)
	test.packetLost(handshakeSpace, p0)
	test.packetLost(handshakeSpace, p1)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 6000)
	test.wantVar("congestion_window", 2400) // minimum in persistent congestion
}

func TestRenoPersistentCongestionDurationNotExceeded(t *testing.T) {
	test := newRenoTest(t, 1200)
	test.setRTT(10*time.Millisecond, 3*time.Millisecond)
	test.maxAckDelay = 25 * time.Millisecond

	t.Logf("persistent congesion duration is 3 * (10ms + 4*3ms + 25ms) = 141ms")
	p0 := test.packetSent(handshakeSpace, 1200)
	test.advance(140 * time.Millisecond) // smaller than persistent congestion duration
	p1 := test.packetSent(handshakeSpace, 1200)
	p2 := test.packetSent(handshakeSpace, 1200)
	test.packetAcked(handshakeSpace, p2)
	test.packetLost(handshakeSpace, p0)
	test.packetLost(handshakeSpace, p1)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 6000)
	test.wantVar("congestion_window", 6000) // no persistent congestion
}

func TestRenoPersistentCongestionInterveningAck(t *testing.T) {
	// "[...] none of the packets sent between the send times
	// of these two packets are acknowledged [...]"
	// https://www.rfc-editor.org/rfc/rfc9002#section-7.6.2-2.1
	test := newRenoTest(t, 1200)

	test.setRTT(10*time.Millisecond, 3*time.Millisecond)
	test.maxAckDelay = 25 * time.Millisecond

	t.Logf("persistent congesion duration is 3 * (10ms + 4*3ms + 25ms) = 141ms")
	p0 := test.packetSent(handshakeSpace, 1200)
	test.advance(100 * time.Millisecond)
	p1 := test.packetSent(handshakeSpace, 1200)
	test.advance(42 * time.Millisecond)
	p2 := test.packetSent(handshakeSpace, 1200)
	p3 := test.packetSent(handshakeSpace, 1200)
	test.packetAcked(handshakeSpace, p1)
	test.packetAcked(handshakeSpace, p3)
	test.packetLost(handshakeSpace, p0)
	test.packetLost(handshakeSpace, p2)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 6000)
	test.wantVar("congestion_window", 6000) // no persistent congestion
}

func TestRenoPersistentCongestionInterveningLosses(t *testing.T) {
	test := newRenoTest(t, 1200)

	test.setRTT(10*time.Millisecond, 3*time.Millisecond)
	test.maxAckDelay = 25 * time.Millisecond

	t.Logf("persistent congesion duration is 3 * (10ms + 4*3ms + 25ms) = 141ms")
	p0 := test.packetSent(handshakeSpace, 1200)
	test.advance(50 * time.Millisecond)
	p1 := test.packetSent(handshakeSpace, 1200, func(p *sentPacket) {
		p.inFlight = false
		p.ackEliciting = false
	})
	test.advance(50 * time.Millisecond)
	p2 := test.packetSent(handshakeSpace, 1200, func(p *sentPacket) {
		p.ackEliciting = false
	})
	test.advance(42 * time.Millisecond)
	p3 := test.packetSent(handshakeSpace, 1200)
	p4 := test.packetSent(handshakeSpace, 1200)
	test.packetAcked(handshakeSpace, p4)
	test.packetLost(handshakeSpace, p0)
	test.packetLost(handshakeSpace, p1)
	test.packetLost(handshakeSpace, p2)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("congestion_window", 6000) // no persistent congestion yet
	test.packetLost(handshakeSpace, p3)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("congestion_window", 2400) // persistent congestion
}

func TestRenoPersistentCongestionNoRTTSample(t *testing.T) {
	// "[...] a prior RTT sample existed when these two packets were sent."
	// https://www.rfc-editor.org/rfc/rfc9002#section-7.6.2-2.3
	test := newRenoTest(t, 1200)

	t.Logf("first packet sent prior to first RTT sample")
	p0 := test.packetSent(handshakeSpace, 1200)

	test.advance(1 * time.Millisecond)
	test.setRTT(10*time.Millisecond, 3*time.Millisecond)
	test.maxAckDelay = 25 * time.Millisecond

	t.Logf("persistent congesion duration is 3 * (10ms + 4*3ms + 25ms) = 141ms")
	test.advance(142 * time.Millisecond) // larger than persistent congestion duration
	p1 := test.packetSent(handshakeSpace, 1200)
	p2 := test.packetSent(handshakeSpace, 1200)
	test.packetAcked(handshakeSpace, p2)
	test.packetLost(handshakeSpace, p0)
	test.packetLost(handshakeSpace, p1)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 6000)
	test.wantVar("congestion_window", 6000) // no persistent congestion
}

func TestRenoPersistentCongestionPacketNotAckEliciting(t *testing.T) {
	// "These two packets MUST be ack-eliciting [...]"
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-7.6.2-3
	test := newRenoTest(t, 1200)

	t.Logf("first packet set prior to first RTT sample")
	p0 := test.packetSent(handshakeSpace, 1200)

	test.advance(1 * time.Millisecond)
	test.setRTT(10*time.Millisecond, 3*time.Millisecond)
	test.maxAckDelay = 25 * time.Millisecond

	t.Logf("persistent congesion duration is 3 * (10ms + 4*3ms + 25ms) = 141ms")
	test.advance(142 * time.Millisecond) // larger than persistent congestion duration
	p1 := test.packetSent(handshakeSpace, 1200)
	p2 := test.packetSent(handshakeSpace, 1200)
	test.packetAcked(handshakeSpace, p2)
	test.packetLost(handshakeSpace, p0)
	test.packetLost(handshakeSpace, p1)
	test.packetBatchEnd(handshakeSpace)
	test.wantVar("slow_start_threshold", 6000)
	test.wantVar("congestion_window", 6000) // no persistent congestion
}

func TestRenoCanSend(t *testing.T) {
	test := newRenoTest(t, 1200)
	test.wantVar("congestion_window", 12000)

	t.Logf("controller permits sending until congestion window is full")
	var packets []*sentPacket
	for i := 0; i < 10; i++ {
		test.wantVar("bytes_in_flight", i*1200)
		test.wantCanSend(true)
		p := test.packetSent(initialSpace, 1200)
		packets = append(packets, p)
	}
	test.wantVar("bytes_in_flight", 12000)

	t.Logf("controller blocks sending when congestion window is consumed")
	test.wantCanSend(false)

	t.Logf("loss of packet moves to recovery, reduces window")
	test.packetLost(initialSpace, packets[0])
	test.packetAcked(initialSpace, packets[1])
	test.packetBatchEnd(initialSpace)
	test.wantVar("bytes_in_flight", 9600)   // 12000 - 2*1200
	test.wantVar("congestion_window", 6000) // 12000 / 2

	t.Logf("one packet permitted on entry to recovery")
	test.wantCanSend(true)
	test.packetSent(initialSpace, 1200)
	test.wantVar("bytes_in_flight", 10800)
	test.wantCanSend(false)
}

func TestRenoNonAckEliciting(t *testing.T) {
	test := newRenoTest(t, 1200)
	test.wantVar("congestion_window", 12000)

	t.Logf("in-flight packet")
	p0 := test.packetSent(initialSpace, 1200)
	test.wantVar("bytes_in_flight", 1200)
	test.packetAcked(initialSpace, p0)
	test.packetBatchEnd(initialSpace)
	test.wantVar("bytes_in_flight", 0)
	test.wantVar("congestion_window", 12000+1200)

	t.Logf("non-in-flight packet")
	p1 := test.packetSent(initialSpace, 1200, func(p *sentPacket) {
		p.inFlight = false
		p.ackEliciting = false
	})
	test.wantVar("bytes_in_flight", 0)
	test.packetAcked(initialSpace, p1)
	test.packetBatchEnd(initialSpace)
	test.wantVar("bytes_in_flight", 0)
	test.wantVar("congestion_window", 12000+1200)
}

func TestRenoUnderutilizedCongestionWindow(t *testing.T) {
	test := newRenoTest(t, 1200)
	test.setUnderutilized(true)
	test.wantVar("congestion_window", 12000)

	t.Logf("congestion window does not increase when application limited")
	p0 := test.packetSent(initialSpace, 1200)
	test.packetAcked(initialSpace, p0)
	test.wantVar("congestion_window", 12000)
}

func TestRenoDiscardKeys(t *testing.T) {
	test := newRenoTest(t, 1200)

	p0 := test.packetSent(initialSpace, 1200)
	p1 := test.packetSent(handshakeSpace, 1200)
	test.wantVar("bytes_in_flight", 2400)

	test.packetDiscarded(initialSpace, p0)
	test.wantVar("bytes_in_flight", 1200)

	test.packetDiscarded(handshakeSpace, p1)
	test.wantVar("bytes_in_flight", 0)
}

type ccTest struct {
	t           *testing.T
	cc          *ccReno
	rtt         rttState
	maxAckDelay time.Duration
	now         time.Time
	nextNum     [numberSpaceCount]packetNumber
}

func newRenoTest(t *testing.T, maxDatagramSize int) *ccTest {
	test := &ccTest{
		t:   t,
		now: time.Date(2000, 1, 1, 0, 0, 0, 0, time.UTC),
	}
	test.cc = newReno(maxDatagramSize)
	return test
}

func (c *ccTest) setRTT(smoothedRTT, rttvar time.Duration) {
	c.t.Helper()
	c.t.Logf("set smoothed_rtt=%v rttvar=%v", smoothedRTT, rttvar)
	c.rtt.smoothedRTT = smoothedRTT
	c.rtt.rttvar = rttvar
	if c.rtt.firstSampleTime.IsZero() {
		c.rtt.firstSampleTime = c.now
	}
}

func (c *ccTest) setUnderutilized(v bool) {
	c.t.Helper()
	c.t.Logf("set underutilized = %v", v)
	c.cc.setUnderutilized(nil, v)
}

func (c *ccTest) packetSent(space numberSpace, size int, fns ...func(*sentPacket)) *sentPacket {
	c.t.Helper()
	num := c.nextNum[space]
	c.nextNum[space]++
	sent := &sentPacket{
		inFlight:     true,
		ackEliciting: true,
		num:          num,
		size:         size,
		time:         c.now,
	}
	for _, f := range fns {
		f(sent)
	}
	c.t.Logf("packet sent:  num=%v.%v, size=%v", space, sent.num, sent.size)
	c.cc.packetSent(c.now, nil, space, sent)
	return sent
}

func (c *ccTest) advance(d time.Duration) {
	c.t.Helper()
	c.t.Logf("advance time %v", d)
	c.now = c.now.Add(d)
}

func (c *ccTest) packetAcked(space numberSpace, sent *sentPacket) {
	c.t.Helper()
	c.t.Logf("packet acked: num=%v.%v, size=%v", space, sent.num, sent.size)
	c.cc.packetAcked(c.now, sent)
}

func (c *ccTest) packetLost(space numberSpace, sent *sentPacket) {
	c.t.Helper()
	c.t.Logf("packet lost:  num=%v.%v, size=%v", space, sent.num, sent.size)
	c.cc.packetLost(c.now, space, sent, &c.rtt)
}

func (c *ccTest) packetDiscarded(space numberSpace, sent *sentPacket) {
	c.t.Helper()
	c.t.Logf("packet number space discarded: num=%v.%v, size=%v", space, sent.num, sent.size)
	c.cc.packetDiscarded(sent)
}

func (c *ccTest) packetBatchEnd(space numberSpace) {
	c.t.Helper()
	c.t.Logf("(end of batch)")
	c.cc.packetBatchEnd(c.now, nil, space, &c.rtt, c.maxAckDelay)
}

func (c *ccTest) wantCanSend(want bool) {
	if got := c.cc.canSend(); got != want {
		c.t.Fatalf("canSend() = %v, want %v", got, want)
	}
}

func (c *ccTest) wantVar(name string, want int) {
	c.t.Helper()
	var got int
	switch name {
	case "bytes_in_flight":
		got = c.cc.bytesInFlight
	case "congestion_pending_acks":
		got = c.cc.congestionPendingAcks
	case "congestion_window":
		got = c.cc.congestionWindow
	case "slow_start_threshold":
		got = c.cc.slowStartThreshold
	default:
		c.t.Fatalf("unknown var %q", name)
	}
	if got != want {
		c.t.Fatalf("ERROR: %v = %v, want %v", name, got, want)
	}
	c.t.Logf("# %v = %v", name, got)
}
