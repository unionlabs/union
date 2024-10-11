package pcp

import (
	"testing"

	"github.com/performancecopilot/speed/v4"

	"github.com/go-kit/kit/metrics/teststat"
)

func TestCounter(t *testing.T) {
	r, err := NewReporter("test_counter")
	if err != nil {
		t.Fatal(err)
	}

	counter, err := r.NewCounter("speed_counter")
	if err != nil {
		t.Fatal(err)
	}

	counter = counter.With("label values", "not supported").(*Counter)

	value := func() float64 { f := counter.c.Val(); return float64(f) }
	if err := teststat.TestCounter(counter, value); err != nil {
		t.Fatal(err)
	}
}

func TestGauge(t *testing.T) {
	r, err := NewReporter("test_gauge")
	if err != nil {
		t.Fatal(err)
	}

	gauge, err := r.NewGauge("speed_gauge")
	if err != nil {
		t.Fatal(err)
	}

	gauge = gauge.With("label values", "not supported").(*Gauge)

	value := func() []float64 { f := gauge.g.Val(); return []float64{f} }
	if err := teststat.TestGauge(gauge, value); err != nil {
		t.Fatal(err)
	}
}

func TestHistogram(t *testing.T) {
	r, err := NewReporter("test_histogram")
	if err != nil {
		t.Fatal(err)
	}

	histogram, err := r.NewHistogram("speed_histogram", 0, 3600000000, speed.OneUnit)
	if err != nil {
		t.Fatal(err)
	}

	histogram = histogram.With("label values", "not supported").(*Histogram)

	quantiles := func() (float64, float64, float64, float64) {
		p50 := float64(histogram.Percentile(50))
		p90 := float64(histogram.Percentile(90))
		p95 := float64(histogram.Percentile(95))
		p99 := float64(histogram.Percentile(99))
		return p50, p90, p95, p99
	}
	if err := teststat.TestHistogram(histogram, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}
