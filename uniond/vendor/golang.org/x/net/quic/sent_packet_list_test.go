// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import "testing"

func TestSentPacketListSlidingWindow(t *testing.T) {
	// Record 1000 sent packets, acking everything outside the most recent 10.
	list := &sentPacketList{}
	const window = 10
	for i := packetNumber(0); i < 1000; i++ {
		list.add(&sentPacket{num: i})
		if i < window {
			continue
		}
		prev := i - window
		sent := list.num(prev)
		if sent == nil {
			t.Fatalf("packet %v not in list", prev)
		}
		if sent.num != prev {
			t.Fatalf("list.num(%v) = packet %v", prev, sent.num)
		}
		if got := list.nth(0); got != sent {
			t.Fatalf("list.nth(0) != list.num(%v)", prev)
		}
		sent.acked = true
		list.clean()
		if got := list.num(prev); got != nil {
			t.Fatalf("list.num(%v) = packet %v, expected it to be discarded", prev, got.num)
		}
		if got, want := list.start(), prev+1; got != want {
			t.Fatalf("list.start() = %v, want %v", got, want)
		}
		if got, want := list.end(), i+1; got != want {
			t.Fatalf("list.end() = %v, want %v", got, want)
		}
		if got, want := list.size, window; got != want {
			t.Fatalf("list.size = %v, want %v", got, want)
		}
	}
}

func TestSentPacketListGrows(t *testing.T) {
	// Record 1000 sent packets.
	list := &sentPacketList{}
	const count = 1000
	for i := packetNumber(0); i < count; i++ {
		list.add(&sentPacket{num: i})
	}
	if got, want := list.start(), packetNumber(0); got != want {
		t.Fatalf("list.start() = %v, want %v", got, want)
	}
	if got, want := list.end(), packetNumber(count); got != want {
		t.Fatalf("list.end() = %v, want %v", got, want)
	}
	if got, want := list.size, count; got != want {
		t.Fatalf("list.size = %v, want %v", got, want)
	}
	for i := packetNumber(0); i < count; i++ {
		sent := list.num(i)
		if sent == nil {
			t.Fatalf("packet %v not in list", i)
		}
		if sent.num != i {
			t.Fatalf("list.num(%v) = packet %v", i, sent.num)
		}
		if got := list.nth(int(i)); got != sent {
			t.Fatalf("list.nth(%v) != list.num(%v)", int(i), i)
		}
	}
}

func TestSentPacketListCleanAll(t *testing.T) {
	list := &sentPacketList{}
	// Record 10 sent packets.
	const count = 10
	for i := packetNumber(0); i < count; i++ {
		list.add(&sentPacket{num: i})
	}
	// Mark all the packets as acked.
	for i := packetNumber(0); i < count; i++ {
		list.num(i).acked = true
	}
	list.clean()
	if got, want := list.size, 0; got != want {
		t.Fatalf("list.size = %v, want %v", got, want)
	}
	list.add(&sentPacket{num: 10})
	if got, want := list.size, 1; got != want {
		t.Fatalf("list.size = %v, want %v", got, want)
	}
	sent := list.num(10)
	if sent == nil {
		t.Fatalf("packet %v not in list", 10)
	}
	if sent.num != 10 {
		t.Fatalf("list.num(10) = %v", sent.num)
	}
	if got := list.nth(0); got != sent {
		t.Fatalf("list.nth(0) != list.num(10)")
	}
}
