// Package discard provides a no-op metrics backend.
package discard

import "github.com/go-kit/kit/metrics"

type counter struct{}

// NewCounter returns a new no-op counter.
func NewCounter() metrics.Counter { return counter{} }

// With implements Counter.
func (c counter) With(labelValues ...string) metrics.Counter { return c }

// Add implements Counter.
func (c counter) Add(delta float64) {}

type gauge struct{}

// NewGauge returns a new no-op gauge.
func NewGauge() metrics.Gauge { return gauge{} }

// With implements Gauge.
func (g gauge) With(labelValues ...string) metrics.Gauge { return g }

// Set implements Gauge.
func (g gauge) Set(value float64) {}

// Add implements metrics.Gauge.
func (g gauge) Add(delta float64) {}

type histogram struct{}

// NewHistogram returns a new no-op histogram.
func NewHistogram() metrics.Histogram { return histogram{} }

// With implements Histogram.
func (h histogram) With(labelValues ...string) metrics.Histogram { return h }

// Observe implements histogram.
func (h histogram) Observe(value float64) {}
