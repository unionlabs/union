// Package graphite provides a Graphite backend for metrics. Metrics are batched
// and emitted in the plaintext protocol. For more information, see
// http://graphite.readthedocs.io/en/latest/feeding-carbon.html#the-plaintext-protocol
//
// Graphite does not have a native understanding of metric parameterization, so
// label values not supported. Use distinct metrics for each unique combination
// of label values.
package graphite

import (
	"context"
	"fmt"
	"io"
	"sync"
	"time"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/generic"
	"github.com/go-kit/kit/util/conn"
	"github.com/go-kit/log"
)

// Graphite receives metrics observations and forwards them to a Graphite server.
// Create a Graphite object, use it to create metrics, and pass those metrics as
// dependencies to the components that will use them.
//
// All metrics are buffered until WriteTo is called. Counters and gauges are
// aggregated into a single observation per timeseries per write. Histograms are
// exploded into per-quantile gauges and reported once per write.
//
// To regularly report metrics to an io.Writer, use the WriteLoop helper method.
// To send to a Graphite server, use the SendLoop helper method.
type Graphite struct {
	mtx        sync.RWMutex
	prefix     string
	counters   map[string]*Counter
	gauges     map[string]*Gauge
	histograms map[string]*Histogram
	logger     log.Logger
}

// New returns a Graphite object that may be used to create metrics. Prefix is
// applied to all created metrics. Callers must ensure that regular calls to
// WriteTo are performed, either manually or with one of the helper methods.
func New(prefix string, logger log.Logger) *Graphite {
	return &Graphite{
		prefix:     prefix,
		counters:   map[string]*Counter{},
		gauges:     map[string]*Gauge{},
		histograms: map[string]*Histogram{},
		logger:     logger,
	}
}

// NewCounter returns a counter. Observations are aggregated and emitted once
// per write invocation.
func (g *Graphite) NewCounter(name string) *Counter {
	c := NewCounter(g.prefix + name)
	g.mtx.Lock()
	g.counters[g.prefix+name] = c
	g.mtx.Unlock()
	return c
}

// NewGauge returns a gauge. Observations are aggregated and emitted once per
// write invocation.
func (g *Graphite) NewGauge(name string) *Gauge {
	ga := NewGauge(g.prefix + name)
	g.mtx.Lock()
	g.gauges[g.prefix+name] = ga
	g.mtx.Unlock()
	return ga
}

// NewHistogram returns a histogram. Observations are aggregated and emitted as
// per-quantile gauges, once per write invocation. 50 is a good default value
// for buckets.
func (g *Graphite) NewHistogram(name string, buckets int) *Histogram {
	h := NewHistogram(g.prefix+name, buckets)
	g.mtx.Lock()
	g.histograms[g.prefix+name] = h
	g.mtx.Unlock()
	return h
}

// WriteLoop is a helper method that invokes WriteTo to the passed writer every
// time the passed channel fires. This method blocks until ctx is canceled,
// so clients probably want to run it in its own goroutine. For typical
// usage, create a time.Ticker and pass its C channel to this method.
func (g *Graphite) WriteLoop(ctx context.Context, c <-chan time.Time, w io.Writer) {
	for {
		select {
		case <-c:
			if _, err := g.WriteTo(w); err != nil {
				g.logger.Log("during", "WriteTo", "err", err)
			}
		case <-ctx.Done():
			return
		}
	}
}

// SendLoop is a helper method that wraps WriteLoop, passing a managed
// connection to the network and address. Like WriteLoop, this method blocks
// until ctx is canceled, so clients probably want to start it in its own
// goroutine. For typical usage, create a time.Ticker and pass its C channel to
// this method.
func (g *Graphite) SendLoop(ctx context.Context, c <-chan time.Time, network, address string) {
	g.WriteLoop(ctx, c, conn.NewDefaultManager(network, address, g.logger))
}

// WriteTo flushes the buffered content of the metrics to the writer, in
// Graphite plaintext format. WriteTo abides best-effort semantics, so
// observations are lost if there is a problem with the write. Clients should be
// sure to call WriteTo regularly, ideally through the WriteLoop or SendLoop
// helper methods.
func (g *Graphite) WriteTo(w io.Writer) (count int64, err error) {
	g.mtx.RLock()
	defer g.mtx.RUnlock()
	now := time.Now().Unix()

	for name, c := range g.counters {
		n, err := fmt.Fprintf(w, "%s %f %d\n", name, c.c.ValueReset(), now)
		if err != nil {
			return count, err
		}
		count += int64(n)
	}

	for name, ga := range g.gauges {
		n, err := fmt.Fprintf(w, "%s %f %d\n", name, ga.g.Value(), now)
		if err != nil {
			return count, err
		}
		count += int64(n)
	}

	for name, h := range g.histograms {
		for _, p := range []struct {
			s string
			f float64
		}{
			{"50", 0.50},
			{"90", 0.90},
			{"95", 0.95},
			{"99", 0.99},
		} {
			n, err := fmt.Fprintf(w, "%s.p%s %f %d\n", name, p.s, h.h.Quantile(p.f), now)
			if err != nil {
				return count, err
			}
			count += int64(n)
		}
	}

	return count, err
}

// Counter is a Graphite counter metric.
type Counter struct {
	c *generic.Counter
}

// NewCounter returns a new usable counter metric.
func NewCounter(name string) *Counter {
	return &Counter{generic.NewCounter(name)}
}

// With is a no-op.
func (c *Counter) With(...string) metrics.Counter { return c }

// Add implements counter.
func (c *Counter) Add(delta float64) { c.c.Add(delta) }

// Gauge is a Graphite gauge metric.
type Gauge struct {
	g *generic.Gauge
}

// NewGauge returns a new usable Gauge metric.
func NewGauge(name string) *Gauge {
	return &Gauge{generic.NewGauge(name)}
}

// With is a no-op.
func (g *Gauge) With(...string) metrics.Gauge { return g }

// Set implements gauge.
func (g *Gauge) Set(value float64) { g.g.Set(value) }

// Add implements metrics.Gauge.
func (g *Gauge) Add(delta float64) { g.g.Add(delta) }

// Histogram is a Graphite histogram metric. Observations are bucketed into
// per-quantile gauges.
type Histogram struct {
	h *generic.Histogram
}

// NewHistogram returns a new usable Histogram metric.
func NewHistogram(name string, buckets int) *Histogram {
	return &Histogram{generic.NewHistogram(name, buckets)}
}

// With is a no-op.
func (h *Histogram) With(...string) metrics.Histogram { return h }

// Observe implements histogram.
func (h *Histogram) Observe(value float64) { h.h.Observe(value) }
