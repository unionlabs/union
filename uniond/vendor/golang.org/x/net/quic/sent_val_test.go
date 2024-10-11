// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import "testing"

func TestSentVal(t *testing.T) {
	for _, test := range []struct {
		name              string
		f                 func(*sentVal)
		wantIsSet         bool
		wantShouldSend    bool
		wantIsReceived    bool
		wantShouldSendPTO bool
	}{{
		name:              "zero value",
		f:                 func(*sentVal) {},
		wantIsSet:         false,
		wantShouldSend:    false,
		wantShouldSendPTO: false,
		wantIsReceived:    false,
	}, {
		name:              "v.set()",
		f:                 (*sentVal).set,
		wantIsSet:         true,
		wantShouldSend:    true,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}, {
		name: "v.setSent(0)",
		f: func(v *sentVal) {
			v.setSent(0)
		},
		wantIsSet:         true,
		wantShouldSend:    false,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}, {
		name: "sent.set()",
		f: func(v *sentVal) {
			v.setSent(0)
			v.set()
		},
		wantIsSet:         true,
		wantShouldSend:    false,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}, {
		name: "sent.setUnsent()",
		f: func(v *sentVal) {
			v.setSent(0)
			v.setUnsent()
		},
		wantIsSet:         true,
		wantShouldSend:    true,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}, {
		name: "set.clear()",
		f: func(v *sentVal) {
			v.set()
			v.clear()
		},
		wantIsSet:         false,
		wantShouldSend:    false,
		wantShouldSendPTO: false,
		wantIsReceived:    false,
	}, {
		name:              "v.setReceived()",
		f:                 (*sentVal).setReceived,
		wantIsSet:         true,
		wantShouldSend:    false,
		wantShouldSendPTO: false,
		wantIsReceived:    true,
	}, {
		name: "v.ackOrLoss(!pnum, true)",
		f: func(v *sentVal) {
			v.setSent(1)
			v.ackOrLoss(0, packetAcked) // ack different packet containing the val
		},
		wantIsSet:         true,
		wantShouldSend:    false,
		wantShouldSendPTO: false,
		wantIsReceived:    true,
	}, {
		name: "v.ackOrLoss(!pnum, packetLost)",
		f: func(v *sentVal) {
			v.setSent(1)
			v.ackOrLoss(0, packetLost) // lose different packet containing the val
		},
		wantIsSet:         true,
		wantShouldSend:    false,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}, {
		name: "v.ackOrLoss(pnum, packetLost)",
		f: func(v *sentVal) {
			v.setSent(1)
			v.ackOrLoss(1, packetLost) // lose same packet containing the val
		},
		wantIsSet:         true,
		wantShouldSend:    true,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}, {
		name: "v.ackLatestOrLoss(!pnum, packetAcked)",
		f: func(v *sentVal) {
			v.setSent(1)
			v.ackLatestOrLoss(0, packetAcked) // ack different packet containing the val
		},
		wantIsSet:         true,
		wantShouldSend:    false,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}, {
		name: "v.ackLatestOrLoss(pnum, packetAcked)",
		f: func(v *sentVal) {
			v.setSent(1)
			v.ackLatestOrLoss(1, packetAcked) // ack same packet containing the val
		},
		wantIsSet:         true,
		wantShouldSend:    false,
		wantShouldSendPTO: false,
		wantIsReceived:    true,
	}, {
		name: "v.ackLatestOrLoss(!pnum, packetLost)",
		f: func(v *sentVal) {
			v.setSent(1)
			v.ackLatestOrLoss(0, packetLost) // lose different packet containing the val
		},
		wantIsSet:         true,
		wantShouldSend:    false,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}, {
		name: "v.ackLatestOrLoss(pnum, packetLost)",
		f: func(v *sentVal) {
			v.setSent(1)
			v.ackLatestOrLoss(1, packetLost) // lose same packet containing the val
		},
		wantIsSet:         true,
		wantShouldSend:    true,
		wantShouldSendPTO: true,
		wantIsReceived:    false,
	}} {
		var v sentVal
		test.f(&v)
		if got, want := v.isSet(), test.wantIsSet; got != want {
			t.Errorf("%v: v.isSet() = %v, want %v", test.name, got, want)
		}
		if got, want := v.shouldSend(), test.wantShouldSend; got != want {
			t.Errorf("%v: v.shouldSend() = %v, want %v", test.name, got, want)
		}
		if got, want := v.shouldSendPTO(false), test.wantShouldSend; got != want {
			t.Errorf("%v: v.shouldSendPTO(false) = %v, want %v", test.name, got, want)
		}
		if got, want := v.shouldSendPTO(true), test.wantShouldSendPTO; got != want {
			t.Errorf("%v: v.shouldSendPTO(true) = %v, want %v", test.name, got, want)
		}
		if got, want := v.isReceived(), test.wantIsReceived; got != want {
			t.Errorf("%v: v.isReceived() = %v, want %v", test.name, got, want)
		}
	}
}
