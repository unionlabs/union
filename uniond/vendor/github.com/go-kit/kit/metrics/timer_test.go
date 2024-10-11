package metrics_test

import (
	"math"
	"testing"

	"time"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/generic"
)

func TestTimerFast(t *testing.T) {
	h := generic.NewSimpleHistogram()
	metrics.NewTimer(h).ObserveDuration()

	tolerance := 0.050
	if want, have := 0.000, h.ApproximateMovingAverage(); math.Abs(want-have) > tolerance {
		t.Errorf("want %.3f, have %.3f", want, have)
	}
}

func TestTimerSlow(t *testing.T) {
	h := generic.NewSimpleHistogram()
	timer := metrics.NewTimer(h)
	time.Sleep(250 * time.Millisecond)
	timer.ObserveDuration()

	tolerance := 0.050
	if want, have := 0.250, h.ApproximateMovingAverage(); math.Abs(want-have) > tolerance {
		t.Errorf("want %.3f, have %.3f", want, have)
	}
}

func TestTimerUnit(t *testing.T) {
	for _, tc := range []struct {
		name      string
		unit      time.Duration
		tolerance float64
		want      float64
	}{
		{"Seconds", time.Second, 0.010, 0.100},
		{"Milliseconds", time.Millisecond, 10, 100},
		{"Nanoseconds", time.Nanosecond, 10000000, 100000000},
	} {
		t.Run(tc.name, func(t *testing.T) {
			h := generic.NewSimpleHistogram()
			timer := metrics.NewTimer(h)
			time.Sleep(100 * time.Millisecond)
			timer.Unit(tc.unit)
			timer.ObserveDuration()

			if want, have := tc.want, h.ApproximateMovingAverage(); math.Abs(want-have) > tc.tolerance {
				t.Errorf("want %.3f, have %.3f", want, have)
			}
		})
	}
}
