// Package generic implements generic versions of each of the metric types. They
// can be embedded by other implementations, and converted to specific formats
// as necessary.
package generic

import (
	"fmt"
	"io"
	"math"
	"sync"
	"sync/atomic"

	"github.com/VividCortex/gohistogram"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/internal/lv"
)

// Counter is an in-memory implementation of a Counter.
type Counter struct {
	bits uint64 // bits has to be the first word in order to be 64-aligned on 32-bit
	Name string
	lvs  lv.LabelValues
}

// NewCounter returns a new, usable Counter.
func NewCounter(name string) *Counter {
	return &Counter{
		Name: name,
	}
}

// With implements Counter.
func (c *Counter) With(labelValues ...string) metrics.Counter {
	return &Counter{
		Name: c.Name,
		bits: atomic.LoadUint64(&c.bits),
		lvs:  c.lvs.With(labelValues...),
	}
}

// Add implements Counter.
func (c *Counter) Add(delta float64) {
	for {
		var (
			old  = atomic.LoadUint64(&c.bits)
			newf = math.Float64frombits(old) + delta
			new  = math.Float64bits(newf)
		)
		if atomic.CompareAndSwapUint64(&c.bits, old, new) {
			break
		}
	}
}

// Value returns the current value of the counter.
func (c *Counter) Value() float64 {
	return math.Float64frombits(atomic.LoadUint64(&c.bits))
}

// ValueReset returns the current value of the counter, and resets it to zero.
// This is useful for metrics backends whose counter aggregations expect deltas,
// like Graphite.
func (c *Counter) ValueReset() float64 {
	for {
		var (
			old  = atomic.LoadUint64(&c.bits)
			newf = 0.0
			new  = math.Float64bits(newf)
		)
		if atomic.CompareAndSwapUint64(&c.bits, old, new) {
			return math.Float64frombits(old)
		}
	}
}

// LabelValues returns the set of label values attached to the counter.
func (c *Counter) LabelValues() []string {
	return c.lvs
}

// Gauge is an in-memory implementation of a Gauge.
type Gauge struct {
	bits uint64 // bits has to be the first word in order to be 64-aligned on 32-bit
	Name string
	lvs  lv.LabelValues
}

// NewGauge returns a new, usable Gauge.
func NewGauge(name string) *Gauge {
	return &Gauge{
		Name: name,
	}
}

// With implements Gauge.
func (g *Gauge) With(labelValues ...string) metrics.Gauge {
	return &Gauge{
		Name: g.Name,
		bits: atomic.LoadUint64(&g.bits),
		lvs:  g.lvs.With(labelValues...),
	}
}

// Set implements Gauge.
func (g *Gauge) Set(value float64) {
	atomic.StoreUint64(&g.bits, math.Float64bits(value))
}

// Add implements metrics.Gauge.
func (g *Gauge) Add(delta float64) {
	for {
		var (
			old  = atomic.LoadUint64(&g.bits)
			newf = math.Float64frombits(old) + delta
			new  = math.Float64bits(newf)
		)
		if atomic.CompareAndSwapUint64(&g.bits, old, new) {
			break
		}
	}
}

// Value returns the current value of the gauge.
func (g *Gauge) Value() float64 {
	return math.Float64frombits(atomic.LoadUint64(&g.bits))
}

// LabelValues returns the set of label values attached to the gauge.
func (g *Gauge) LabelValues() []string {
	return g.lvs
}

// Histogram is an in-memory implementation of a streaming histogram, based on
// VividCortex/gohistogram. It dynamically computes quantiles, so it's not
// suitable for aggregation.
type Histogram struct {
	Name string
	lvs  lv.LabelValues
	h    *safeHistogram
}

// NewHistogram returns a numeric histogram based on VividCortex/gohistogram. A
// good default value for buckets is 50.
func NewHistogram(name string, buckets int) *Histogram {
	return &Histogram{
		Name: name,
		h:    &safeHistogram{Histogram: gohistogram.NewHistogram(buckets)},
	}
}

// With implements Histogram.
func (h *Histogram) With(labelValues ...string) metrics.Histogram {
	return &Histogram{
		Name: h.Name,
		lvs:  h.lvs.With(labelValues...),
		h:    h.h,
	}
}

// Observe implements Histogram.
func (h *Histogram) Observe(value float64) {
	h.h.Lock()
	defer h.h.Unlock()
	h.h.Add(value)
}

// Quantile returns the value of the quantile q, 0.0 < q < 1.0.
func (h *Histogram) Quantile(q float64) float64 {
	h.h.RLock()
	defer h.h.RUnlock()
	return h.h.Quantile(q)
}

// LabelValues returns the set of label values attached to the histogram.
func (h *Histogram) LabelValues() []string {
	return h.lvs
}

// Print writes a string representation of the histogram to the passed writer.
// Useful for printing to a terminal.
func (h *Histogram) Print(w io.Writer) {
	h.h.RLock()
	defer h.h.RUnlock()
	fmt.Fprint(w, h.h.String())
}

// safeHistogram exists as gohistogram.Histogram is not goroutine-safe.
type safeHistogram struct {
	sync.RWMutex
	gohistogram.Histogram
}

// Bucket is a range in a histogram which aggregates observations.
type Bucket struct {
	From, To, Count int64
}

// Quantile is a pair of a quantile (0..100) and its observed maximum value.
type Quantile struct {
	Quantile int // 0..100
	Value    int64
}

// SimpleHistogram is an in-memory implementation of a Histogram. It only tracks
// an approximate moving average, so is likely too naïve for many use cases.
type SimpleHistogram struct {
	mtx sync.RWMutex
	lvs lv.LabelValues
	avg float64
	n   uint64
}

// NewSimpleHistogram returns a SimpleHistogram, ready for observations.
func NewSimpleHistogram() *SimpleHistogram {
	return &SimpleHistogram{}
}

// With implements Histogram.
func (h *SimpleHistogram) With(labelValues ...string) metrics.Histogram {
	return &SimpleHistogram{
		lvs: h.lvs.With(labelValues...),
		avg: h.avg,
		n:   h.n,
	}
}

// Observe implements Histogram.
func (h *SimpleHistogram) Observe(value float64) {
	h.mtx.Lock()
	defer h.mtx.Unlock()
	h.n++
	h.avg -= h.avg / float64(h.n)
	h.avg += value / float64(h.n)
}

// ApproximateMovingAverage returns the approximate moving average of observations.
func (h *SimpleHistogram) ApproximateMovingAverage() float64 {
	h.mtx.RLock()
	defer h.mtx.RUnlock()
	return h.avg
}

// LabelValues returns the set of label values attached to the histogram.
func (h *SimpleHistogram) LabelValues() []string {
	return h.lvs
}
