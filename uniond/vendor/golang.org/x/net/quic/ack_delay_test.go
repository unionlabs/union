// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"math"
	"testing"
	"time"
)

func TestAckDelayFromDuration(t *testing.T) {
	for _, test := range []struct {
		d                time.Duration
		ackDelayExponent uint8
		want             unscaledAckDelay
	}{{
		d:                8 * time.Microsecond,
		ackDelayExponent: 3,
		want:             1,
	}, {
		d:                1 * time.Nanosecond,
		ackDelayExponent: 3,
		want:             0, // rounds to zero
	}, {
		d:                3 * (1 << 20) * time.Microsecond,
		ackDelayExponent: 20,
		want:             3,
	}} {
		got := unscaledAckDelayFromDuration(test.d, test.ackDelayExponent)
		if got != test.want {
			t.Errorf("unscaledAckDelayFromDuration(%v, %v) = %v, want %v",
				test.d, test.ackDelayExponent, got, test.want)
		}
	}
}

func TestAckDelayToDuration(t *testing.T) {
	for _, test := range []struct {
		d                unscaledAckDelay
		ackDelayExponent uint8
		want             time.Duration
	}{{
		d:                1,
		ackDelayExponent: 3,
		want:             8 * time.Microsecond,
	}, {
		d:                0,
		ackDelayExponent: 3,
		want:             0,
	}, {
		d:                3,
		ackDelayExponent: 20,
		want:             3 * (1 << 20) * time.Microsecond,
	}, {
		d:                math.MaxInt64 / 1000,
		ackDelayExponent: 0,
		want:             (math.MaxInt64 / 1000) * time.Microsecond,
	}, {
		d:                (math.MaxInt64 / 1000) + 1,
		ackDelayExponent: 0,
		want:             0, // return 0 on overflow
	}, {
		d:                math.MaxInt64 / 1000 / 8,
		ackDelayExponent: 3,
		want:             (math.MaxInt64 / 1000 / 8) * 8 * time.Microsecond,
	}, {
		d:                (math.MaxInt64 / 1000 / 8) + 1,
		ackDelayExponent: 3,
		want:             0, // return 0 on overflow
	}} {
		got := test.d.Duration(test.ackDelayExponent)
		if got != test.want {
			t.Errorf("unscaledAckDelay(%v).Duration(%v) = %v, want %v",
				test.d, test.ackDelayExponent, int64(got), int64(test.want))
		}
	}
}
