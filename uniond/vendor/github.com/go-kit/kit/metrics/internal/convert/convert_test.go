package convert

import (
	"testing"

	"github.com/go-kit/kit/metrics/generic"
	"github.com/go-kit/kit/metrics/teststat"
)

func TestCounterHistogramConversion(t *testing.T) {
	name := "my_counter"
	c := generic.NewCounter(name)
	h := NewCounterAsHistogram(c)
	top := NewHistogramAsCounter(h).With("label", "counter").(histogramCounter)
	mid := top.h.(counterHistogram)
	low := mid.c.(*generic.Counter)
	if want, have := name, low.Name; want != have {
		t.Errorf("Name: want %q, have %q", want, have)
	}
	if err := teststat.TestCounter(top, low.Value); err != nil {
		t.Fatal(err)
	}
}

func TestCounterGaugeConversion(t *testing.T) {
	name := "my_counter"
	c := generic.NewCounter(name)
	g := NewCounterAsGauge(c)
	top := NewGaugeAsCounter(g).With("label", "counter").(gaugeCounter)
	mid := top.g.(counterGauge)
	low := mid.c.(*generic.Counter)
	if want, have := name, low.Name; want != have {
		t.Errorf("Name: want %q, have %q", want, have)
	}
	if err := teststat.TestCounter(top, low.Value); err != nil {
		t.Fatal(err)
	}
}

func TestHistogramGaugeConversion(t *testing.T) {
	name := "my_histogram"
	h := generic.NewHistogram(name, 50)
	g := NewHistogramAsGauge(h)
	top := NewGaugeAsHistogram(g).With("label", "histogram").(gaugeHistogram)
	mid := top.g.(histogramGauge)
	low := mid.h.(*generic.Histogram)
	if want, have := name, low.Name; want != have {
		t.Errorf("Name: want %q, have %q", want, have)
	}
	quantiles := func() (float64, float64, float64, float64) {
		return low.Quantile(0.50), low.Quantile(0.90), low.Quantile(0.95), low.Quantile(0.99)
	}
	if err := teststat.TestHistogram(top, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}
