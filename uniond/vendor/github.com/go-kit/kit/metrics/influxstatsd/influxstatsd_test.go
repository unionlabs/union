package influxstatsd

import (
	"testing"

	"github.com/go-kit/kit/metrics/teststat"
	"github.com/go-kit/log"
)

func TestCounter(t *testing.T) {
	prefix, name := "abc.", "def"
	label, value := "label", "value"
	regex := `^` + prefix + name + "," + label + `=` + value + `:([0-9\.]+)\|c$`
	d := New(prefix, log.NewNopLogger())
	counter := d.NewCounter(name, 1.0).With(label, value)
	valuef := teststat.SumLines(d, regex)
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
	label, value := "xyz", "abc"
	regex := `^` + prefix + name + `,hostname=foohost,` + label + `=` + value + `:([0-9\.]+)\|g$`
	d := New(prefix, log.NewNopLogger(), "hostname", "foohost")
	gauge := d.NewGauge(name).With(label, value)
	valuef := teststat.LastLine(d, regex)
	if err := teststat.TestGauge(gauge, valuef); err != nil {
		t.Fatal(err)
	}
}

// InfluxStatsD histograms just emit all observations. So, we collect them into
// a generic histogram, and run the statistics test on that.

func TestHistogram(t *testing.T) {
	prefix, name := "influxstatsd.", "histogram_test"
	label, value := "abc", "def"
	regex := `^` + prefix + name + "," + label + `=` + value + `:([0-9\.]+)\|h$`
	d := New(prefix, log.NewNopLogger())
	histogram := d.NewHistogram(name, 1.0).With(label, value)
	quantiles := teststat.Quantiles(d, regex, 50) // no |@0.X
	if err := teststat.TestHistogram(histogram, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}

func TestHistogramSampled(t *testing.T) {
	prefix, name := "influxstatsd.", "sampled_histogram_test"
	label, value := "foo", "bar"
	regex := `^` + prefix + name + "," + label + `=` + value + `:([0-9\.]+)\|h\|@0\.01[0]*$`
	d := New(prefix, log.NewNopLogger())
	histogram := d.NewHistogram(name, 0.01).With(label, value)
	quantiles := teststat.Quantiles(d, regex, 50)
	if err := teststat.TestHistogram(histogram, quantiles, 0.02); err != nil {
		t.Fatal(err)
	}
}

func TestTiming(t *testing.T) {
	prefix, name := "influxstatsd.", "timing_test"
	label, value := "wiggle", "bottom"
	regex := `^` + prefix + name + "," + label + `=` + value + `:([0-9\.]+)\|ms$`
	d := New(prefix, log.NewNopLogger())
	histogram := d.NewTiming(name, 1.0).With(label, value)
	quantiles := teststat.Quantiles(d, regex, 50) // no |@0.X
	if err := teststat.TestHistogram(histogram, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}

func TestTimingSampled(t *testing.T) {
	prefix, name := "influxstatsd.", "sampled_timing_test"
	label, value := "internal", "external"
	regex := `^` + prefix + name + "," + label + `=` + value + `:([0-9\.]+)\|ms\|@0.03[0]*$`
	d := New(prefix, log.NewNopLogger())
	histogram := d.NewTiming(name, 0.03).With(label, value)
	quantiles := teststat.Quantiles(d, regex, 50)
	if err := teststat.TestHistogram(histogram, quantiles, 0.02); err != nil {
		t.Fatal(err)
	}
}
