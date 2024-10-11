package sentry

import (
	"fmt"
	"sync"
	"testing"
)

func TestFixedRateSampler(t *testing.T) {
	ctx := NewTestContext(ClientOptions{})
	rootSpan := StartSpan(ctx, "root")

	t.Run("UniformRate", func(t *testing.T) {
		// The sample decision for the root span should observe the configured
		// rate.
		tests := []struct {
			Rate      float64
			Tolerance float64
		}{
			{0.0, 0.0},
			{0.25, 0.1},
			{0.5, 0.1},
			{0.75, 0.1},
			{1.0, 0.0},
		}
		for _, tt := range tests {
			tt := tt
			t.Run(fmt.Sprint(tt.Rate), func(t *testing.T) {
				got := repeatedSample(func(ctx SamplingContext) float64 { return tt.Rate }, SamplingContext{Span: rootSpan}, 10000)
				if got < tt.Rate*(1-tt.Tolerance) || got > tt.Rate*(1+tt.Tolerance) {
					t.Errorf("got rootSpan sample rate %.2f, want %.2f±%.0f%%", got, tt.Rate, 100*tt.Tolerance)
				}
			})
		}
	})

	t.Run("Concurrency", func(t *testing.T) {
		// This test is for use with -race to catch data races.
		var wg sync.WaitGroup
		for i := 0; i < 32; i++ {
			wg.Add(1)
			go func() {
				defer wg.Done()
				repeatedSample(func(ctx SamplingContext) float64 { return 0.5 }, SamplingContext{Span: rootSpan}, 10000)
			}()
		}
		wg.Wait()
	})
}

func repeatedSample(sampler TracesSampler, ctx SamplingContext, count int) (observedRate float64) {
	var n float64
	for i := 0; i < count; i++ {
		sampleRate := sampler.Sample(ctx)
		if rng.Float64() < sampleRate {
			n++
		}
	}
	return n / float64(count)
}
