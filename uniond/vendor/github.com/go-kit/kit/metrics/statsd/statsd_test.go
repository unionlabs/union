package statsd

import (
	"testing"

	"github.com/go-kit/kit/metrics/teststat"
	"github.com/go-kit/log"
)

func TestCounter(t *testing.T) {
	prefix, name := "abc.", "def"
	label, value := "label", "value" // ignored
	regex := `^` + prefix + name + `:([0-9\.]+)\|c$`
	s := New(prefix, log.NewNopLogger())
	counter := s.NewCounter(name, 1.0).With(label, value)
	valuef := teststat.SumLines(s, regex)
	if err := teststat.TestCounter(counter, valuef); err != nil {
		t.Fatal(err)
	}
}

func TestCounterSampled(t *testing.T) {
	// This will involve multiplying the observed sum by the inverse of the
	// sample rate and checking against the expected value within some
	// tolerance.
	t.Skip("TODO")
}

func TestGauge(t *testing.T) {
	prefix, name := "ghi.", "jkl"
	label, value := "xyz", "abc" // ignored
	regex := `^` + prefix + name + `:([0-9\.]+)\|g$`
	s := New(prefix, log.NewNopLogger())
	gauge := s.NewGauge(name).With(label, value)
	valuef := teststat.LastLine(s, regex)
	if err := teststat.TestGauge(gauge, valuef); err != nil {
		t.Fatal(err)
	}
}

// StatsD timings just emit all observations. So, we collect them into a generic
// histogram, and run the statistics test on that.

func TestTiming(t *testing.T) {
	prefix, name := "statsd.", "timing_test"
	label, value := "abc", "def" // ignored
	regex := `^` + prefix + name + `:([0-9\.]+)\|ms$`
	s := New(prefix, log.NewNopLogger())
	timing := s.NewTiming(name, 1.0).With(label, value)
	quantiles := teststat.Quantiles(s, regex, 50) // no |@0.X
	if err := teststat.TestHistogram(timing, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}

func TestTimingSampled(t *testing.T) {
	prefix, name := "statsd.", "sampled_timing_test"
	label, value := "foo", "bar" // ignored
	regex := `^` + prefix + name + `:([0-9\.]+)\|ms\|@0\.01[0]*$`
	s := New(prefix, log.NewNopLogger())
	timing := s.NewTiming(name, 0.01).With(label, value)
	quantiles := teststat.Quantiles(s, regex, 50)
	if err := teststat.TestHistogram(timing, quantiles, 0.02); err != nil {
		t.Fatal(err)
	}
}
