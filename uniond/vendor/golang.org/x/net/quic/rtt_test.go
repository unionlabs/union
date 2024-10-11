// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"testing"
	"time"
)

func TestRTTMinRTT(t *testing.T) {
	var (
		handshakeConfirmed = false
		ackDelay           = 0 * time.Millisecond
		maxAckDelay        = 25 * time.Millisecond
		now                = time.Date(2000, 1, 1, 0, 0, 0, 0, time.UTC)
	)
	rtt := &rttState{}
	rtt.init()

	// "min_rtt MUST be set to the latest_rtt on the first RTT sample."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.2-2
	rtt.updateSample(now, handshakeConfirmed, initialSpace, 10*time.Millisecond, ackDelay, maxAckDelay)
	if got, want := rtt.latestRTT, 10*time.Millisecond; got != want {
		t.Errorf("on first sample: latest_rtt = %v, want %v", got, want)
	}
	if got, want := rtt.minRTT, 10*time.Millisecond; got != want {
		t.Errorf("on first sample: min_rtt = %v, want %v", got, want)
	}

	// "min_rtt MUST be set to the lesser of min_rtt and latest_rtt [...]
	// on all other samples."
	rtt.updateSample(now, handshakeConfirmed, initialSpace, 20*time.Millisecond, ackDelay, maxAckDelay)
	if got, want := rtt.latestRTT, 20*time.Millisecond; got != want {
		t.Errorf("on increasing sample: latest_rtt = %v, want %v", got, want)
	}
	if got, want := rtt.minRTT, 10*time.Millisecond; got != want {
		t.Errorf("on increasing sample: min_rtt = %v, want %v (no change)", got, want)
	}

	rtt.updateSample(now, handshakeConfirmed, initialSpace, 5*time.Millisecond, ackDelay, maxAckDelay)
	if got, want := rtt.latestRTT, 5*time.Millisecond; got != want {
		t.Errorf("on new minimum: latest_rtt = %v, want %v", got, want)
	}
	if got, want := rtt.minRTT, 5*time.Millisecond; got != want {
		t.Errorf("on new minimum: min_rtt = %v, want %v", got, want)
	}

	// "Endpoints SHOULD set the min_rtt to the newest RTT sample
	// after persistent congestion is established."
	// https://www.rfc-editor.org/rfc/rfc9002.html#section-5.2-5
	rtt.updateSample(now, handshakeConfirmed, initialSpace, 15*time.Millisecond, ackDelay, maxAckDelay)
	if got, want := rtt.latestRTT, 15*time.Millisecond; got != want {
		t.Errorf("on increasing sample: latest_rtt = %v, want %v", got, want)
	}
	if got, want := rtt.minRTT, 5*time.Millisecond; got != want {
		t.Errorf("on increasing sample: min_rtt = %v, want %v (no change)", got, want)
	}
	rtt.establishPersistentCongestion()
	if got, want := rtt.minRTT, 15*time.Millisecond; got != want {
		t.Errorf("after persistent congestion: min_rtt = %v, want %v", got, want)
	}
}

func TestRTTInitialRTT(t *testing.T) {
	var (
		handshakeConfirmed = false
		ackDelay           = 0 * time.Millisecond
		maxAckDelay        = 25 * time.Millisecond
		now                = time.Date(2000, 1, 1, 0, 0, 0, 0, time.UTC)
	)
	rtt := &rttState{}
	rtt.init()

	// "When no previous RTT is available,
	// the initial RTT SHOULD be set to 333 milliseconds."
	// https://www.rfc-editor.org/rfc/rfc9002#section-6.2.2-1
	if got, want := rtt.smoothedRTT, 333*time.Millisecond; got != want {
		t.Errorf("initial smoothed_rtt = %v, want %v", got, want)
	}
	if got, want := rtt.rttvar, 333*time.Millisecond/2; got != want {
		t.Errorf("initial rttvar = %v, want %v", got, want)
	}

	rtt.updateSample(now, handshakeConfirmed, initialSpace, 10*time.Millisecond, ackDelay, maxAckDelay)
	smoothedRTT := 10 * time.Millisecond
	if got, want := rtt.smoothedRTT, smoothedRTT; got != want {
		t.Errorf("after first rtt sample of 10ms, smoothed_rtt = %v, want %v", got, want)
	}
	rttvar := 5 * time.Millisecond
	if got, want := rtt.rttvar, rttvar; got != want {
		t.Errorf("after first rtt sample of 10ms, rttvar = %v, want %v", got, want)
	}

	// "[...] MAY ignore the acknowledgment delay for Initial packets [...]"
	// https://www.rfc-editor.org/rfc/rfc9002#section-5.3-7.1
	ackDelay = 1 * time.Millisecond
	rtt.updateSample(now, handshakeConfirmed, initialSpace, 10*time.Millisecond, ackDelay, maxAckDelay)
	adjustedRTT := 10 * time.Millisecond
	smoothedRTT = (7*smoothedRTT + adjustedRTT) / 8
	if got, want := rtt.smoothedRTT, smoothedRTT; got != want {
		t.Errorf("smoothed_rtt = %v, want %v", got, want)
	}
	rttvarSample := abs(smoothedRTT - adjustedRTT)
	rttvar = (3*rttvar + rttvarSample) / 4
	if got, want := rtt.rttvar, rttvar; got != want {
		t.Errorf("rttvar = %v, want %v", got, want)
	}

	// "[...] SHOULD ignore the peer's max_ack_delay until the handshake is confirmed [...]"
	// https://www.rfc-editor.org/rfc/rfc9002#section-5.3-7.2
	ackDelay = 30 * time.Millisecond
	maxAckDelay = 25 * time.Millisecond
	rtt.updateSample(now, handshakeConfirmed, handshakeSpace, 40*time.Millisecond, ackDelay, maxAckDelay)
	adjustedRTT = 10 * time.Millisecond // latest_rtt (40ms) - ack_delay (30ms)
	smoothedRTT = (7*smoothedRTT + adjustedRTT) / 8
	if got, want := rtt.smoothedRTT, smoothedRTT; got != want {
		t.Errorf("smoothed_rtt = %v, want %v", got, want)
	}
	rttvarSample = abs(smoothedRTT - adjustedRTT)
	rttvar = (3*rttvar + rttvarSample) / 4
	if got, want := rtt.rttvar, rttvar; got != want {
		t.Errorf("rttvar = %v, want %v", got, want)
	}

	// "[...] MUST use the lesser of the acknowledgment delay and
	// the peer's max_ack_delay after the handshake is confirmed [...]"
	// https://www.rfc-editor.org/rfc/rfc9002#section-5.3-7.3
	ackDelay = 30 * time.Millisecond
	maxAckDelay = 25 * time.Millisecond
	handshakeConfirmed = true
	rtt.updateSample(now, handshakeConfirmed, handshakeSpace, 40*time.Millisecond, ackDelay, maxAckDelay)
	adjustedRTT = 15 * time.Millisecond // latest_rtt (40ms) - max_ack_delay (25ms)
	rttvarSample = abs(smoothedRTT - adjustedRTT)
	rttvar = (3*rttvar + rttvarSample) / 4
	if got, want := rtt.rttvar, rttvar; got != want {
		t.Errorf("rttvar = %v, want %v", got, want)
	}
	smoothedRTT = (7*smoothedRTT + adjustedRTT) / 8
	if got, want := rtt.smoothedRTT, smoothedRTT; got != want {
		t.Errorf("smoothed_rtt = %v, want %v", got, want)
	}

	// "[...] MUST NOT subtract the acknowledgment delay from
	// the RTT sample if the resulting value is smaller than the min_rtt."
	// https://www.rfc-editor.org/rfc/rfc9002#section-5.3-7.4
	ackDelay = 25 * time.Millisecond
	maxAckDelay = 25 * time.Millisecond
	handshakeConfirmed = true
	rtt.updateSample(now, handshakeConfirmed, handshakeSpace, 30*time.Millisecond, ackDelay, maxAckDelay)
	if got, want := rtt.minRTT, 10*time.Millisecond; got != want {
		t.Errorf("min_rtt = %v, want %v", got, want)
	}
	// latest_rtt (30ms) - ack_delay (25ms) = 5ms, which is less than min_rtt (10ms)
	adjustedRTT = 30 * time.Millisecond // latest_rtt
	rttvarSample = abs(smoothedRTT - adjustedRTT)
	rttvar = (3*rttvar + rttvarSample) / 4
	if got, want := rtt.rttvar, rttvar; got != want {
		t.Errorf("rttvar = %v, want %v", got, want)
	}
	smoothedRTT = (7*smoothedRTT + adjustedRTT) / 8
	if got, want := rtt.smoothedRTT, smoothedRTT; got != want {
		t.Errorf("smoothed_rtt = %v, want %v", got, want)
	}
}
