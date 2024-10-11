// Package multi provides adapters that send observations to multiple metrics
// simultaneously. This is useful if your service needs to emit to multiple
// instrumentation systems at the same time, for example if your organization is
// transitioning from one system to another.
package multi

import "github.com/go-kit/kit/metrics"

// Counter collects multiple individual counters and treats them as a unit.
type Counter []metrics.Counter

// NewCounter returns a multi-counter, wrapping the passed counters.
func NewCounter(c ...metrics.Counter) Counter {
	return Counter(c)
}

// Add implements counter.
func (c Counter) Add(delta float64) {
	for _, counter := range c {
		counter.Add(delta)
	}
}

// With implements counter.
func (c Counter) With(labelValues ...string) metrics.Counter {
	next := make(Counter, len(c))
	for i := range c {
		next[i] = c[i].With(labelValues...)
	}
	return next
}

// Gauge collects multiple individual gauges and treats them as a unit.
type Gauge []metrics.Gauge

// NewGauge returns a multi-gauge, wrapping the passed gauges.
func NewGauge(g ...metrics.Gauge) Gauge {
	return Gauge(g)
}

// Set implements Gauge.
func (g Gauge) Set(value float64) {
	for _, gauge := range g {
		gauge.Set(value)
	}
}

// With implements gauge.
func (g Gauge) With(labelValues ...string) metrics.Gauge {
	next := make(Gauge, len(g))
	for i := range g {
		next[i] = g[i].With(labelValues...)
	}
	return next
}

// Add implements metrics.Gauge.
func (g Gauge) Add(delta float64) {
	for _, gauge := range g {
		gauge.Add(delta)
	}
}

// Histogram collects multiple individual histograms and treats them as a unit.
type Histogram []metrics.Histogram

// NewHistogram returns a multi-histogram, wrapping the passed histograms.
func NewHistogram(h ...metrics.Histogram) Histogram {
	return Histogram(h)
}

// Observe implements Histogram.
func (h Histogram) Observe(value float64) {
	for _, histogram := range h {
		histogram.Observe(value)
	}
}

// With implements histogram.
func (h Histogram) With(labelValues ...string) metrics.Histogram {
	next := make(Histogram, len(h))
	for i := range h {
		next[i] = h[i].With(labelValues...)
	}
	return next
}
