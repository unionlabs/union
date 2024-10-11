// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import "testing"

func TestSentPacket(t *testing.T) {
	frames := []any{
		byte(frameTypePing),
		byte(frameTypeStreamBase),
		uint64(1),
		i64range[int64]{1 << 20, 1<<20 + 1024},
	}
	// Record sent frames.
	sent := newSentPacket()
	for _, f := range frames {
		switch f := f.(type) {
		case byte:
			sent.appendAckElicitingFrame(f)
		case uint64:
			sent.appendInt(f)
		case i64range[int64]:
			sent.appendOffAndSize(f.start, int(f.size()))
		}
	}
	// Read the record.
	for i, want := range frames {
		if done := sent.done(); done {
			t.Fatalf("before consuming contents, sent.done() = true, want false")
		}
		switch want := want.(type) {
		case byte:
			if got := sent.next(); got != want {
				t.Fatalf("%v: sent.next() = %v, want %v", i, got, want)
			}
		case uint64:
			if got := sent.nextInt(); got != want {
				t.Fatalf("%v: sent.nextInt() = %v, want %v", i, got, want)
			}
		case i64range[int64]:
			if start, end := sent.nextRange(); start != want.start || end != want.end {
				t.Fatalf("%v: sent.nextRange() = [%v,%v), want %v", i, start, end, want)
			}
		}
	}
	if done := sent.done(); !done {
		t.Fatalf("after consuming contents, sent.done() = false, want true")
	}
}
