// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"testing"
	"time"
)

func TestPacerStartup(t *testing.T) {
	p := &pacerTest{
		cwnd:             10000,
		rtt:              100 * time.Millisecond,
		timerGranularity: 1 * time.Millisecond,
	}
	p.init(t)
	t.Logf("# initial burst permits sending ten packets")
	for i := 0; i < 10; i++ {
		p.sendPacket(1000)
	}

	t.Logf("# empty bucket allows for one more packet")
	p.sendPacket(1000)

	t.Logf("# sending 1000 byte packets with 8ms interval:")
	t.Logf("#   (smoothed_rtt * packet_size / congestion_window) / 1.25")
	t.Logf("#   (100ms * 1000 / 10000) / 1.25 = 8ms")
	p.wantSendDelay(8 * time.Millisecond)
	p.advance(8 * time.Millisecond)
	p.sendPacket(1000)
	p.wantSendDelay(8 * time.Millisecond)

	t.Logf("# accumulate enough window for two packets")
	p.advance(16 * time.Millisecond)
	p.sendPacket(1000)
	p.sendPacket(1000)
	p.wantSendDelay(8 * time.Millisecond)

	t.Logf("# window does not grow to more than burst limit")
	p.advance(1 * time.Second)
	for i := 0; i < 11; i++ {
		p.sendPacket(1000)
	}
	p.wantSendDelay(8 * time.Millisecond)
}

func TestPacerTimerGranularity(t *testing.T) {
	p := &pacerTest{
		cwnd:             10000,
		rtt:              100 * time.Millisecond,
		timerGranularity: 1 * time.Millisecond,
	}
	p.init(t)
	t.Logf("# consume initial burst")
	for i := 0; i < 11; i++ {
		p.sendPacket(1000)
	}
	p.wantSendDelay(8 * time.Millisecond)

	t.Logf("# small advance in time does not permit sending")
	p.advance(4 * time.Millisecond)
	p.wantSendDelay(4 * time.Millisecond)

	t.Logf("# advancing to within timerGranularity of next send permits send")
	p.advance(3 * time.Millisecond)
	p.wantSendDelay(0)

	t.Logf("# early send adds skipped delay (1ms) to next send (8ms)")
	p.sendPacket(1000)
	p.wantSendDelay(9 * time.Millisecond)
}

func TestPacerChangingRate(t *testing.T) {
	p := &pacerTest{
		cwnd:             10000,
		rtt:              100 * time.Millisecond,
		timerGranularity: 0,
	}
	p.init(t)
	t.Logf("# consume initial burst")
	for i := 0; i < 11; i++ {
		p.sendPacket(1000)
	}
	p.wantSendDelay(8 * time.Millisecond)
	p.advance(8 * time.Millisecond)

	t.Logf("# set congestion window to 20000, 1000 byte interval is 4ms")
	p.cwnd = 20000
	p.sendPacket(1000)
	p.wantSendDelay(4 * time.Millisecond)
	p.advance(4 * time.Millisecond)

	t.Logf("# set rtt to 200ms, 1000 byte interval is 8ms")
	p.rtt = 200 * time.Millisecond
	p.sendPacket(1000)
	p.wantSendDelay(8 * time.Millisecond)
	p.advance(8 * time.Millisecond)

	t.Logf("# set congestion window to 40000, 1000 byte interval is 4ms")
	p.cwnd = 40000
	p.advance(8 * time.Millisecond)
	p.sendPacket(1000)
	p.sendPacket(1000)
	p.sendPacket(1000)
	p.wantSendDelay(4 * time.Millisecond)
}

func TestPacerTimeReverses(t *testing.T) {
	p := &pacerTest{
		cwnd:             10000,
		rtt:              100 * time.Millisecond,
		timerGranularity: 0,
	}
	p.init(t)
	t.Logf("# consume initial burst")
	for i := 0; i < 11; i++ {
		p.sendPacket(1000)
	}
	p.wantSendDelay(8 * time.Millisecond)
	t.Logf("# reverse time")
	p.advance(-4 * time.Millisecond)
	p.sendPacket(1000)
	p.wantSendDelay(8 * time.Millisecond)
	p.advance(8 * time.Millisecond)
	p.sendPacket(1000)
	p.wantSendDelay(8 * time.Millisecond)
}

func TestPacerZeroRTT(t *testing.T) {
	p := &pacerTest{
		cwnd:             10000,
		rtt:              0,
		timerGranularity: 0,
	}
	p.init(t)
	t.Logf("# with rtt 0, the pacer does not limit sending")
	for i := 0; i < 20; i++ {
		p.sendPacket(1000)
	}
	p.advance(1 * time.Second)
	for i := 0; i < 20; i++ {
		p.sendPacket(1000)
	}
}

func TestPacerZeroCongestionWindow(t *testing.T) {
	p := &pacerTest{
		cwnd:             10000,
		rtt:              100 * time.Millisecond,
		timerGranularity: 0,
	}
	p.init(t)
	p.cwnd = 0
	t.Logf("# with cwnd 0, the pacer does not limit sending")
	for i := 0; i < 20; i++ {
		p.sendPacket(1000)
	}
}

type pacerTest struct {
	t                *testing.T
	p                pacerState
	timerGranularity time.Duration
	cwnd             int
	rtt              time.Duration
	now              time.Time
}

func newPacerTest(t *testing.T, congestionWindow int, rtt time.Duration) *pacerTest {
	p := &pacerTest{
		now:  time.Date(2000, 1, 1, 0, 0, 0, 0, time.UTC),
		cwnd: congestionWindow,
		rtt:  rtt,
	}
	p.p.init(p.now, congestionWindow, p.timerGranularity)
	return p
}

func (p *pacerTest) init(t *testing.T) {
	p.t = t
	p.now = time.Date(2000, 1, 1, 0, 0, 0, 0, time.UTC)
	p.p.init(p.now, p.cwnd, p.timerGranularity)
	t.Logf("# initial congestion window: %v", p.cwnd)
	t.Logf("# timer granularity: %v", p.timerGranularity)
}

func (p *pacerTest) advance(d time.Duration) {
	p.t.Logf("advance time %v", d)
	p.now = p.now.Add(d)
	p.p.advance(p.now, p.cwnd, p.rtt)
}

func (p *pacerTest) sendPacket(size int) {
	if canSend, next := p.p.canSend(p.now); !canSend {
		p.t.Fatalf("ERROR: pacer unexpectedly blocked send, delay=%v", next.Sub(p.now))
	}
	p.t.Logf("send packet of size %v", size)
	p.p.packetSent(p.now, size, p.cwnd, p.rtt)
}

func (p *pacerTest) wantSendDelay(want time.Duration) {
	wantCanSend := want == 0
	gotCanSend, next := p.p.canSend(p.now)
	var got time.Duration
	if !gotCanSend {
		got = next.Sub(p.now)
	}
	p.t.Logf("# pacer send delay: %v", got)
	if got != want || gotCanSend != wantCanSend {
		p.t.Fatalf("ERROR: pacer send delay = %v (can send: %v); want %v, %v", got, gotCanSend, want, wantCanSend)
	}
}

func (p *pacerTest) sendDelay() time.Duration {
	canSend, next := p.p.canSend(p.now)
	if canSend {
		return 0
	}
	return next.Sub(p.now)
}
