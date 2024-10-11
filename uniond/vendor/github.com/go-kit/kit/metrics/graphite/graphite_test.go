package graphite

import (
	"bytes"
	"regexp"
	"strconv"
	"testing"

	"github.com/go-kit/kit/metrics/teststat"
	"github.com/go-kit/log"
)

func TestCounter(t *testing.T) {
	prefix, name := "abc.", "def"
	label, value := "label", "value" // ignored for Graphite
	regex := `^` + prefix + name + ` ([0-9\.]+) [0-9]+$`
	g := New(prefix, log.NewNopLogger())
	counter := g.NewCounter(name).With(label, value)
	valuef := teststat.SumLines(g, regex)
	if err := teststat.TestCounter(counter, valuef); err != nil {
		t.Fatal(err)
	}
}

func TestGauge(t *testing.T) {
	prefix, name := "ghi.", "jkl"
	label, value := "xyz", "abc" // ignored for Graphite
	regex := `^` + prefix + name + ` ([0-9\.]+) [0-9]+$`
	g := New(prefix, log.NewNopLogger())
	gauge := g.NewGauge(name).With(label, value)
	valuef := teststat.LastLine(g, regex)
	if err := teststat.TestGauge(gauge, valuef); err != nil {
		t.Fatal(err)
	}
}

func TestHistogram(t *testing.T) {
	// The histogram test is actually like 4 gauge tests.
	prefix, name := "graphite.", "histogram_test"
	label, value := "abc", "def" // ignored for Graphite
	re50 := regexp.MustCompile(prefix + name + `.p50 ([0-9\.]+) [0-9]+`)
	re90 := regexp.MustCompile(prefix + name + `.p90 ([0-9\.]+) [0-9]+`)
	re95 := regexp.MustCompile(prefix + name + `.p95 ([0-9\.]+) [0-9]+`)
	re99 := regexp.MustCompile(prefix + name + `.p99 ([0-9\.]+) [0-9]+`)
	g := New(prefix, log.NewNopLogger())
	histogram := g.NewHistogram(name, 50).With(label, value)
	quantiles := func() (float64, float64, float64, float64) {
		var buf bytes.Buffer
		g.WriteTo(&buf)
		match50 := re50.FindStringSubmatch(buf.String())
		p50, _ := strconv.ParseFloat(match50[1], 64)
		match90 := re90.FindStringSubmatch(buf.String())
		p90, _ := strconv.ParseFloat(match90[1], 64)
		match95 := re95.FindStringSubmatch(buf.String())
		p95, _ := strconv.ParseFloat(match95[1], 64)
		match99 := re99.FindStringSubmatch(buf.String())
		p99, _ := strconv.ParseFloat(match99[1], 64)
		return p50, p90, p95, p99
	}
	if err := teststat.TestHistogram(histogram, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}
