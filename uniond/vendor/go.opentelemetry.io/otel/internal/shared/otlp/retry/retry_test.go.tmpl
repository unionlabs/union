// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/otlp/retry/retry_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package retry

import (
	"context"
	"errors"
	"math"
	"sync"
	"testing"
	"time"

	"github.com/cenkalti/backoff/v4"
	"github.com/stretchr/testify/assert"
)

func TestWait(t *testing.T) {
	tests := []struct {
		ctx      context.Context
		delay    time.Duration
		expected error
	}{
		{
			ctx:   context.Background(),
			delay: time.Duration(0),
		},
		{
			ctx:   context.Background(),
			delay: time.Duration(1),
		},
		{
			ctx:   context.Background(),
			delay: time.Duration(-1),
		},
		{
			ctx: func() context.Context {
				ctx, cancel := context.WithCancel(context.Background())
				cancel()
				return ctx
			}(),
			// Ensure the timer and context do not end simultaneously.
			delay:    1 * time.Hour,
			expected: context.Canceled,
		},
	}

	for _, test := range tests {
		err := wait(test.ctx, test.delay)
		if test.expected == nil {
			assert.NoError(t, err)
		} else {
			assert.ErrorIs(t, err, test.expected)
		}
	}
}

func TestNonRetryableError(t *testing.T) {
	ev := func(error) (bool, time.Duration) { return false, 0 }

	reqFunc := Config{
		Enabled:         true,
		InitialInterval: 1 * time.Nanosecond,
		MaxInterval:     1 * time.Nanosecond,
		// Never stop retrying.
		MaxElapsedTime: 0,
	}.RequestFunc(ev)
	ctx := context.Background()
	assert.NoError(t, reqFunc(ctx, func(context.Context) error {
		return nil
	}))
	assert.ErrorIs(t, reqFunc(ctx, func(context.Context) error {
		return assert.AnError
	}), assert.AnError)
}

func TestThrottledRetry(t *testing.T) {
	// Ensure the throttle delay is used by making longer than backoff delay.
	throttleDelay, backoffDelay := time.Second, time.Nanosecond

	ev := func(error) (bool, time.Duration) {
		// Retry everything with a throttle delay.
		return true, throttleDelay
	}

	reqFunc := Config{
		Enabled:         true,
		InitialInterval: backoffDelay,
		MaxInterval:     backoffDelay,
		// Never stop retrying.
		MaxElapsedTime: 0,
	}.RequestFunc(ev)

	origWait := waitFunc
	var done bool
	waitFunc = func(_ context.Context, delay time.Duration) error {
		assert.Equal(t, throttleDelay, delay, "retry not throttled")
		// Try twice to ensure call is attempted again after delay.
		if done {
			return assert.AnError
		}
		done = true
		return nil
	}
	defer func() { waitFunc = origWait }()

	ctx := context.Background()
	assert.ErrorIs(t, reqFunc(ctx, func(context.Context) error {
		return errors.New("not this error")
	}), assert.AnError)
}

func TestBackoffRetry(t *testing.T) {
	ev := func(error) (bool, time.Duration) { return true, 0 }

	delay := time.Nanosecond
	reqFunc := Config{
		Enabled:         true,
		InitialInterval: delay,
		MaxInterval:     delay,
		// Never stop retrying.
		MaxElapsedTime: 0,
	}.RequestFunc(ev)

	origWait := waitFunc
	var done bool
	waitFunc = func(_ context.Context, d time.Duration) error {
		delta := math.Ceil(float64(delay) * backoff.DefaultRandomizationFactor)
		assert.InDelta(t, delay, d, delta, "retry not backoffed")
		// Try twice to ensure call is attempted again after delay.
		if done {
			return assert.AnError
		}
		done = true
		return nil
	}
	t.Cleanup(func() { waitFunc = origWait })

	ctx := context.Background()
	assert.ErrorIs(t, reqFunc(ctx, func(context.Context) error {
		return errors.New("not this error")
	}), assert.AnError)
}

func TestBackoffRetryCanceledContext(t *testing.T) {
	ev := func(error) (bool, time.Duration) { return true, 0 }

	delay := time.Millisecond
	reqFunc := Config{
		Enabled:         true,
		InitialInterval: delay,
		MaxInterval:     delay,
		// Never stop retrying.
		MaxElapsedTime: 10 * time.Millisecond,
	}.RequestFunc(ev)

	ctx, cancel := context.WithCancel(context.Background())
	count := 0
	cancel()
	err := reqFunc(ctx, func(context.Context) error {
		count++
		return assert.AnError
	})

	assert.ErrorIs(t, err, context.Canceled)
	assert.Contains(t, err.Error(), assert.AnError.Error())
	assert.Equal(t, 1, count)
}

func TestThrottledRetryGreaterThanMaxElapsedTime(t *testing.T) {
	// Ensure the throttle delay is used by making longer than backoff delay.
	tDelay, bDelay := time.Hour, time.Nanosecond
	ev := func(error) (bool, time.Duration) { return true, tDelay }
	reqFunc := Config{
		Enabled:         true,
		InitialInterval: bDelay,
		MaxInterval:     bDelay,
		MaxElapsedTime:  tDelay - (time.Nanosecond),
	}.RequestFunc(ev)

	ctx := context.Background()
	assert.Contains(t, reqFunc(ctx, func(context.Context) error {
		return assert.AnError
	}).Error(), "max retry time would elapse: ")
}

func TestMaxElapsedTime(t *testing.T) {
	ev := func(error) (bool, time.Duration) { return true, 0 }
	delay := time.Nanosecond
	reqFunc := Config{
		Enabled: true,
		// InitialInterval > MaxElapsedTime means immediate return.
		InitialInterval: 2 * delay,
		MaxElapsedTime:  delay,
	}.RequestFunc(ev)

	ctx := context.Background()
	assert.Contains(t, reqFunc(ctx, func(context.Context) error {
		return assert.AnError
	}).Error(), "max retry time elapsed: ")
}

func TestRetryNotEnabled(t *testing.T) {
	ev := func(error) (bool, time.Duration) {
		t.Error("evaluated retry when not enabled")
		return false, 0
	}

	reqFunc := Config{}.RequestFunc(ev)
	ctx := context.Background()
	assert.NoError(t, reqFunc(ctx, func(context.Context) error {
		return nil
	}))
	assert.ErrorIs(t, reqFunc(ctx, func(context.Context) error {
		return assert.AnError
	}), assert.AnError)
}

func TestRetryConcurrentSafe(t *testing.T) {
	ev := func(error) (bool, time.Duration) { return true, 0 }
	reqFunc := Config{
		Enabled: true,
	}.RequestFunc(ev)

	var wg sync.WaitGroup
	ctx := context.Background()

	for i := 1; i < 5; i++ {
		wg.Add(1)

		go func() {
			defer wg.Done()

			var done bool
			assert.NoError(t, reqFunc(ctx, func(context.Context) error {
				if !done {
					done = true
					return assert.AnError
				}

				return nil
			}))
		}()
	}

	wg.Wait()
}
