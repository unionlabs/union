// Package expvar provides expvar backends for metrics.
// Label values are not supported.
package expvar

import (
	"expvar"
	"sync"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/generic"
)

// Counter implements the counter metric with an expvar float.
// Label values are not supported.
type Counter struct {
	f *expvar.Float
}

// NewCounter creates an expvar Float with the given name, and returns an object
// that implements the Counter interface.
func NewCounter(name string) *Counter {
	return &Counter{
		f: expvar.NewFloat(name),
	}
}

// With is a no-op.
func (c *Counter) With(labelValues ...string) metrics.Counter { return c }

// Add implements Counter.
func (c *Counter) Add(delta float64) { c.f.Add(delta) }

// Gauge implements the gauge metric with an expvar float.
// Label values are not supported.
type Gauge struct {
	f *expvar.Float
}

// NewGauge creates an expvar Float with the given name, and returns an object
// that implements the Gauge interface.
func NewGauge(name string) *Gauge {
	return &Gauge{
		f: expvar.NewFloat(name),
	}
}

// With is a no-op.
func (g *Gauge) With(labelValues ...string) metrics.Gauge { return g }

// Set implements Gauge.
func (g *Gauge) Set(value float64) { g.f.Set(value) }

// Add implements metrics.Gauge.
func (g *Gauge) Add(delta float64) { g.f.Add(delta) }

// Histogram implements the histogram metric with a combination of the generic
// Histogram object and several expvar Floats, one for each of the 50th, 90th,
// 95th, and 99th quantiles of observed values, with the quantile attached to
// the name as a suffix. Label values are not supported.
type Histogram struct {
	mtx sync.Mutex
	h   *generic.Histogram
	p50 *expvar.Float
	p90 *expvar.Float
	p95 *expvar.Float
	p99 *expvar.Float
}

// NewHistogram returns a Histogram object with the given name and number of
// buckets in the underlying histogram object. 50 is a good default number of
// buckets.
func NewHistogram(name string, buckets int) *Histogram {
	return &Histogram{
		h:   generic.NewHistogram(name, buckets),
		p50: expvar.NewFloat(name + ".p50"),
		p90: expvar.NewFloat(name + ".p90"),
		p95: expvar.NewFloat(name + ".p95"),
		p99: expvar.NewFloat(name + ".p99"),
	}
}

// With is a no-op.
func (h *Histogram) With(labelValues ...string) metrics.Histogram { return h }

// Observe implements Histogram.
func (h *Histogram) Observe(value float64) {
	h.mtx.Lock()
	defer h.mtx.Unlock()
	h.h.Observe(value)
	h.p50.Set(h.h.Quantile(0.50))
	h.p90.Set(h.h.Quantile(0.90))
	h.p95.Set(h.h.Quantile(0.95))
	h.p99.Set(h.h.Quantile(0.99))
}
