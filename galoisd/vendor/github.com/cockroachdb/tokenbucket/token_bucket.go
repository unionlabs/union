// Copyright 2023 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package tokenbucket

import (
	"context"
	"time"
)

// Tokens are abstract units (usually units of work).
type Tokens float64

// TokensPerSecond is the rate of token replenishment.
type TokensPerSecond float64

// TokenBucket implements the basic accounting for a token bucket.
//
// A token bucket has a rate of replenishment and a burst limit. Tokens are
// replenished over time, up to the burst limit.
//
// The token bucket keeps track of the current amount and updates it as time
// passes. The bucket can go into debt (i.e. negative current amount).
type TokenBucket struct {
	rate  TokensPerSecond
	burst Tokens
	nowFn func() time.Time

	current     Tokens
	lastUpdated time.Time

	exhaustedStart  time.Time
	exhaustedMicros int64
}

// Init the token bucket.
func (tb *TokenBucket) Init(rate TokensPerSecond, burst Tokens) {
	tb.InitWithNowFn(rate, burst, func() time.Time {
		return time.Now()
	})
}

// Init the token bucket with a custom "Now" function.
// Note that Wait/WaitCtx cannot be used with a custom time function.
func (tb *TokenBucket) InitWithNowFn(rate TokensPerSecond, burst Tokens, nowFn func() time.Time) {
	*tb = TokenBucket{
		rate:        rate,
		burst:       burst,
		nowFn:       nowFn,
		current:     burst,
		lastUpdated: nowFn(),
	}
}

// Update moves the time forward, accounting for the replenishment since the
// last update.
func (tb *TokenBucket) Update() {
	now := tb.nowFn()
	if since := now.Sub(tb.lastUpdated); since > 0 {
		tb.current += Tokens(float64(tb.rate) * since.Seconds())

		if tb.current > tb.burst {
			tb.current = tb.burst
		}
		tb.lastUpdated = now
		tb.updateExhaustedMicros()
	}
}

// UpdateConfig updates the rate and burst limits. The change in burst will be
// applied to the current token quantity. For example, if the RateLimiter
// currently had 5 available tokens and the burst is updated from 10 to 20, the
// amount will increase to 15. Similarly, if the burst is decreased by 10, the
// current quota will decrease accordingly, potentially putting the limiter into
// debt.
func (tb *TokenBucket) UpdateConfig(rate TokensPerSecond, burst Tokens) {
	tb.Update()

	burstDelta := burst - tb.burst
	tb.rate = rate
	tb.burst = burst

	tb.current += burstDelta
	tb.updateExhaustedMicros()
}

// Reset resets the current tokens to whatever the burst is.
func (tb *TokenBucket) Reset() {
	tb.current = tb.burst
	tb.updateExhaustedMicros()
}

// Adjust returns tokens to the bucket (positive delta) or accounts for a debt
// of tokens (negative delta).
func (tb *TokenBucket) Adjust(delta Tokens) {
	tb.Update()
	tb.current += delta
	if tb.current > tb.burst {
		tb.current = tb.burst
	}
	tb.updateExhaustedMicros()
}

// TryToFulfill either removes the given amount if is available, or returns a
// time after which the request should be retried.
func (tb *TokenBucket) TryToFulfill(amount Tokens) (fulfilled bool, tryAgainAfter time.Duration) {
	tb.Update()

	// Deal with the case where the request is larger than the burst size. In
	// this case we'll allow the acquisition to complete if and when the current
	// value is equal to the burst. If the acquisition succeeds, it will put the
	// limiter into debt.
	want := amount
	if want > tb.burst {
		want = tb.burst
	}
	if delta := want - tb.current; delta > 0 {
		// Compute the time it will take to get to the needed capacity.
		timeDelta := time.Duration((float64(delta) * float64(time.Second)) / float64(tb.rate))
		if timeDelta < time.Nanosecond {
			timeDelta = time.Nanosecond
		}
		return false, timeDelta
	}

	tb.current -= amount
	tb.updateExhaustedMicros()
	return true, 0
}

// Wait removes the given amount, waiting as long as necessary.
func (tb *TokenBucket) Wait(amount Tokens) {
	_ = tb.WaitCtx(context.Background(), amount)
}

// WaitCtx removes the given amount, waiting as long as necessary or until the
// context is canceled.
func (tb *TokenBucket) WaitCtx(ctx context.Context, amount Tokens) error {
	// We want to check for context cancelation even if we don't need to wait.
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}
	for {
		fulfilled, tryAgainAfter := tb.TryToFulfill(amount)
		if fulfilled {
			return nil
		}
		select {
		case <-time.After(tryAgainAfter):
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}

// Exhausted returns the cumulative duration over which this token bucket was
// exhausted. Exported only for metrics.
func (tb *TokenBucket) Exhausted() time.Duration {
	var ongoingExhaustionMicros int64
	if !tb.exhaustedStart.IsZero() {
		ongoingExhaustionMicros = tb.nowFn().Sub(tb.exhaustedStart).Microseconds()
	}
	return time.Duration(tb.exhaustedMicros+ongoingExhaustionMicros) * time.Microsecond
}

// Available returns the currently available tokens (can be -ve). Exported only
// for metrics.
func (tb *TokenBucket) Available() Tokens {
	return tb.current
}

func (tb *TokenBucket) updateExhaustedMicros() {
	now := tb.nowFn()
	if tb.current <= 0 && tb.exhaustedStart.IsZero() {
		tb.exhaustedStart = now
	}
	if tb.current > 0 && !tb.exhaustedStart.IsZero() {
		tb.exhaustedMicros += now.Sub(tb.exhaustedStart).Microseconds()
		tb.exhaustedStart = time.Time{}
	}
}

// TestingInternalParameters returns the refill rate (configured), burst tokens
// (configured), and number of available tokens where available <= burst. It's
// used in tests.
func (tb *TokenBucket) TestingInternalParameters() (rate TokensPerSecond, burst, available Tokens) {
	tb.Update()
	return tb.rate, tb.burst, tb.current
}
