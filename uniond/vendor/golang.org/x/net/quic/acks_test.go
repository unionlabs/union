// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"slices"
	"testing"
	"time"
)

func TestAcksDisallowDuplicate(t *testing.T) {
	// Don't process a packet that we've seen before.
	acks := ackState{}
	now := time.Now()
	receive := []packetNumber{0, 1, 2, 4, 7, 6, 9}
	seen := map[packetNumber]bool{}
	for i, pnum := range receive {
		acks.receive(now, appDataSpace, pnum, true)
		seen[pnum] = true
		for ppnum := packetNumber(0); ppnum < 11; ppnum++ {
			if got, want := acks.shouldProcess(ppnum), !seen[ppnum]; got != want {
				t.Fatalf("after receiving %v: acks.shouldProcess(%v) = %v, want %v", receive[:i+1], ppnum, got, want)
			}
		}
	}
}

func TestAcksDisallowDiscardedAckRanges(t *testing.T) {
	// Don't process a packet with a number in a discarded range.
	acks := ackState{}
	now := time.Now()
	for pnum := packetNumber(0); ; pnum += 2 {
		acks.receive(now, appDataSpace, pnum, true)
		send, _ := acks.acksToSend(now)
		for ppnum := packetNumber(0); ppnum < packetNumber(send.min()); ppnum++ {
			if acks.shouldProcess(ppnum) {
				t.Fatalf("after limiting ack ranges to %v: acks.shouldProcess(%v) (in discarded range) = true, want false", send, ppnum)
			}
		}
		if send.min() > 10 {
			break
		}
	}
}

func TestAcksSent(t *testing.T) {
	type packet struct {
		pnum         packetNumber
		ackEliciting bool
	}
	for _, test := range []struct {
		name  string
		space numberSpace

		// ackedPackets and packets are packets that we receive.
		// After receiving all packets in ackedPackets, we send an ack.
		// Then we receive the subsequent packets in packets.
		ackedPackets []packet
		packets      []packet

		wantDelay time.Duration
		wantAcks  rangeset[packetNumber]
	}{{
		name:  "no packets to ack",
		space: initialSpace,
	}, {
		name:  "non-ack-eliciting packets are not acked",
		space: initialSpace,
		packets: []packet{{
			pnum:         0,
			ackEliciting: false,
		}},
	}, {
		name:  "ack-eliciting Initial packets are acked immediately",
		space: initialSpace,
		packets: []packet{{
			pnum:         0,
			ackEliciting: true,
		}},
		wantAcks:  rangeset[packetNumber]{{0, 1}},
		wantDelay: 0,
	}, {
		name:  "ack-eliciting Handshake packets are acked immediately",
		space: handshakeSpace,
		packets: []packet{{
			pnum:         0,
			ackEliciting: true,
		}},
		wantAcks:  rangeset[packetNumber]{{0, 1}},
		wantDelay: 0,
	}, {
		name:  "ack-eliciting AppData packets are acked after max_ack_delay",
		space: appDataSpace,
		packets: []packet{{
			pnum:         0,
			ackEliciting: true,
		}},
		wantAcks:  rangeset[packetNumber]{{0, 1}},
		wantDelay: maxAckDelay - timerGranularity,
	}, {
		name:  "reordered ack-eliciting packets are acked immediately",
		space: appDataSpace,
		ackedPackets: []packet{{
			pnum:         1,
			ackEliciting: true,
		}},
		packets: []packet{{
			pnum:         0,
			ackEliciting: true,
		}},
		wantAcks:  rangeset[packetNumber]{{0, 2}},
		wantDelay: 0,
	}, {
		name:  "gaps in ack-eliciting packets are acked immediately",
		space: appDataSpace,
		packets: []packet{{
			pnum:         1,
			ackEliciting: true,
		}},
		wantAcks:  rangeset[packetNumber]{{1, 2}},
		wantDelay: 0,
	}, {
		name:  "reordered non-ack-eliciting packets are not acked immediately",
		space: appDataSpace,
		ackedPackets: []packet{{
			pnum:         1,
			ackEliciting: true,
		}},
		packets: []packet{{
			pnum:         2,
			ackEliciting: true,
		}, {
			pnum:         0,
			ackEliciting: false,
		}, {
			pnum:         4,
			ackEliciting: false,
		}},
		wantAcks:  rangeset[packetNumber]{{0, 3}, {4, 5}},
		wantDelay: maxAckDelay - timerGranularity,
	}, {
		name:  "immediate ack after two ack-eliciting packets are received",
		space: appDataSpace,
		packets: []packet{{
			pnum:         0,
			ackEliciting: true,
		}, {
			pnum:         1,
			ackEliciting: true,
		}},
		wantAcks:  rangeset[packetNumber]{{0, 2}},
		wantDelay: 0,
	}} {
		t.Run(test.name, func(t *testing.T) {
			acks := ackState{}
			start := time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)
			for _, p := range test.ackedPackets {
				t.Logf("receive %v.%v, ack-eliciting=%v", test.space, p.pnum, p.ackEliciting)
				acks.receive(start, test.space, p.pnum, p.ackEliciting)
			}
			t.Logf("send an ACK frame")
			acks.sentAck()
			for _, p := range test.packets {
				t.Logf("receive %v.%v, ack-eliciting=%v", test.space, p.pnum, p.ackEliciting)
				acks.receive(start, test.space, p.pnum, p.ackEliciting)
			}
			switch {
			case len(test.wantAcks) == 0:
				// No ACK should be sent, even well after max_ack_delay.
				if acks.shouldSendAck(start.Add(10 * maxAckDelay)) {
					t.Errorf("acks.shouldSendAck(T+10*max_ack_delay) = true, want false")
				}
			case test.wantDelay > 0:
				// No ACK should be sent before a delay.
				if acks.shouldSendAck(start.Add(test.wantDelay - 1)) {
					t.Errorf("acks.shouldSendAck(T+%v-1ns) = true, want false", test.wantDelay)
				}
				fallthrough
			default:
				// ACK should be sent after a delay.
				if !acks.shouldSendAck(start.Add(test.wantDelay)) {
					t.Errorf("acks.shouldSendAck(T+%v) = false, want true", test.wantDelay)
				}
			}
			// acksToSend always reports the available packets that can be acked,
			// and the amount of time that has passed since the most recent acked
			// packet was received.
			for _, delay := range []time.Duration{
				0,
				test.wantDelay,
				test.wantDelay + 1,
			} {
				gotNums, gotDelay := acks.acksToSend(start.Add(delay))
				wantDelay := delay
				if len(gotNums) == 0 {
					wantDelay = 0
				}
				if !slices.Equal(gotNums, test.wantAcks) || gotDelay != wantDelay {
					t.Errorf("acks.acksToSend(T+%v) = %v, %v; want %v, %v", delay, gotNums, gotDelay, test.wantAcks, wantDelay)
				}
			}
		})
	}
}

func TestAcksDiscardAfterAck(t *testing.T) {
	acks := ackState{}
	now := time.Now()
	acks.receive(now, appDataSpace, 0, true)
	acks.receive(now, appDataSpace, 2, true)
	acks.receive(now, appDataSpace, 4, true)
	acks.receive(now, appDataSpace, 5, true)
	acks.receive(now, appDataSpace, 6, true)
	acks.handleAck(6) // discards all ranges prior to the one containing packet 6
	acks.receive(now, appDataSpace, 7, true)
	got, _ := acks.acksToSend(now)
	if len(got) != 1 {
		t.Errorf("acks.acksToSend contains ranges prior to last acknowledged ack; got %v, want 1 range", got)
	}
}

func TestAcksLargestSeen(t *testing.T) {
	acks := ackState{}
	now := time.Now()
	acks.receive(now, appDataSpace, 0, true)
	acks.receive(now, appDataSpace, 4, true)
	acks.receive(now, appDataSpace, 1, true)
	if got, want := acks.largestSeen(), packetNumber(4); got != want {
		t.Errorf("acks.largestSeen() = %v, want %v", got, want)
	}
}
