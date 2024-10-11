// Package convert provides a way to use Counters, Histograms, or Gauges
// as one of the other types
package convert

import "github.com/go-kit/kit/metrics"

type counterHistogram struct {
	c metrics.Counter
}

// NewCounterAsHistogram returns a Histogram that actually writes the
// value on an underlying Counter
func NewCounterAsHistogram(c metrics.Counter) metrics.Histogram {
	return counterHistogram{c}
}

// With implements Histogram.
func (ch counterHistogram) With(labelValues ...string) metrics.Histogram {
	return counterHistogram{ch.c.With(labelValues...)}
}

// Observe implements histogram.
func (ch counterHistogram) Observe(value float64) {
	ch.c.Add(value)
}

type histogramCounter struct {
	h metrics.Histogram
}

// NewHistogramAsCounter returns a Counter that actually writes the
// value on an underlying Histogram
func NewHistogramAsCounter(h metrics.Histogram) metrics.Counter {
	return histogramCounter{h}
}

// With implements Counter.
func (hc histogramCounter) With(labelValues ...string) metrics.Counter {
	return histogramCounter{hc.h.With(labelValues...)}
}

// Add implements Counter.
func (hc histogramCounter) Add(delta float64) {
	hc.h.Observe(delta)
}

type counterGauge struct {
	c metrics.Counter
}

// NewCounterAsGauge returns a Gauge that actually writes the
// value on an underlying Counter
func NewCounterAsGauge(c metrics.Counter) metrics.Gauge {
	return counterGauge{c}
}

// With implements Gauge.
func (cg counterGauge) With(labelValues ...string) metrics.Gauge {
	return counterGauge{cg.c.With(labelValues...)}
}

// Set implements Gauge.
func (cg counterGauge) Set(value float64) {
	cg.c.Add(value)
}

// Add implements metrics.Gauge.
func (cg counterGauge) Add(delta float64) {
	cg.c.Add(delta)
}

type gaugeCounter struct {
	g metrics.Gauge
}

// NewGaugeAsCounter returns a Counter that actually writes the
// value on an underlying Gauge
func NewGaugeAsCounter(g metrics.Gauge) metrics.Counter {
	return gaugeCounter{g}
}

// With implements Counter.
func (gc gaugeCounter) With(labelValues ...string) metrics.Counter {
	return gaugeCounter{gc.g.With(labelValues...)}
}

// Add implements Counter.
func (gc gaugeCounter) Add(delta float64) {
	gc.g.Set(delta)
}

type histogramGauge struct {
	h metrics.Histogram
}

// NewHistogramAsGauge returns a Gauge that actually writes the
// value on an underlying Histogram
func NewHistogramAsGauge(h metrics.Histogram) metrics.Gauge {
	return histogramGauge{h}
}

// With implements Gauge.
func (hg histogramGauge) With(labelValues ...string) metrics.Gauge {
	return histogramGauge{hg.h.With(labelValues...)}
}

// Set implements Gauge.
func (hg histogramGauge) Set(value float64) {
	hg.h.Observe(value)
}

// Add implements metrics.Gauge.
func (hg histogramGauge) Add(delta float64) {
	hg.h.Observe(delta)
}

type gaugeHistogram struct {
	g metrics.Gauge
}

// NewGaugeAsHistogram returns a Histogram that actually writes the
// value on an underlying Gauge
func NewGaugeAsHistogram(g metrics.Gauge) metrics.Histogram {
	return gaugeHistogram{g}
}

// With implements Histogram.
func (gh gaugeHistogram) With(labelValues ...string) metrics.Histogram {
	return gaugeHistogram{gh.g.With(labelValues...)}
}

// Observe implements histogram.
func (gh gaugeHistogram) Observe(value float64) {
	gh.g.Set(value)
}
