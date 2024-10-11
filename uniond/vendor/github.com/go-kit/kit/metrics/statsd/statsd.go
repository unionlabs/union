// Package statsd provides a StatsD backend for package metrics. StatsD has no
// concept of arbitrary key-value tagging, so label values are not supported,
// and With is a no-op on all metrics.
//
// This package batches observations and emits them on some schedule to the
// remote server. This is useful even if you connect to your StatsD server over
// UDP. Emitting one network packet per observation can quickly overwhelm even
// the fastest internal network.
package statsd

import (
	"context"
	"fmt"
	"io"
	"time"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/internal/lv"
	"github.com/go-kit/kit/metrics/internal/ratemap"
	"github.com/go-kit/kit/util/conn"
	"github.com/go-kit/log"
)

// Statsd receives metrics observations and forwards them to a StatsD server.
// Create a Statsd object, use it to create metrics, and pass those metrics as
// dependencies to the components that will use them.
//
// All metrics are buffered until WriteTo is called. Counters and gauges are
// aggregated into a single observation per timeseries per write. Timings are
// buffered but not aggregated.
//
// To regularly report metrics to an io.Writer, use the WriteLoop helper method.
// To send to a StatsD server, use the SendLoop helper method.
type Statsd struct {
	prefix string
	rates  *ratemap.RateMap

	// The observations are collected in an N-dimensional vector space, even
	// though they only take advantage of a single dimension (name). This is an
	// implementation detail born purely from convenience. It would be more
	// accurate to collect them in a map[string][]float64, but we already have
	// this nice data structure and helper methods.
	counters *lv.Space
	gauges   *lv.Space
	timings  *lv.Space

	logger log.Logger
}

// New returns a Statsd object that may be used to create metrics. Prefix is
// applied to all created metrics. Callers must ensure that regular calls to
// WriteTo are performed, either manually or with one of the helper methods.
func New(prefix string, logger log.Logger) *Statsd {
	return &Statsd{
		prefix:   prefix,
		rates:    ratemap.New(),
		counters: lv.NewSpace(),
		gauges:   lv.NewSpace(),
		timings:  lv.NewSpace(),
		logger:   logger,
	}
}

// NewCounter returns a counter, sending observations to this Statsd object.
func (s *Statsd) NewCounter(name string, sampleRate float64) *Counter {
	s.rates.Set(s.prefix+name, sampleRate)
	return &Counter{
		name: s.prefix + name,
		obs:  s.counters.Observe,
	}
}

// NewGauge returns a gauge, sending observations to this Statsd object.
func (s *Statsd) NewGauge(name string) *Gauge {
	return &Gauge{
		name: s.prefix + name,
		obs:  s.gauges.Observe,
		add:  s.gauges.Add,
	}
}

// NewTiming returns a histogram whose observations are interpreted as
// millisecond durations, and are forwarded to this Statsd object.
func (s *Statsd) NewTiming(name string, sampleRate float64) *Timing {
	s.rates.Set(s.prefix+name, sampleRate)
	return &Timing{
		name: s.prefix + name,
		obs:  s.timings.Observe,
	}
}

// WriteLoop is a helper method that invokes WriteTo to the passed writer every
// time the passed channel fires. This method blocks until ctx is canceled,
// so clients probably want to run it in its own goroutine. For typical
// usage, create a time.Ticker and pass its C channel to this method.
func (s *Statsd) WriteLoop(ctx context.Context, c <-chan time.Time, w io.Writer) {
	for {
		select {
		case <-c:
			if _, err := s.WriteTo(w); err != nil {
				s.logger.Log("during", "WriteTo", "err", err)
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
func (s *Statsd) SendLoop(ctx context.Context, c <-chan time.Time, network, address string) {
	s.WriteLoop(ctx, c, conn.NewDefaultManager(network, address, s.logger))
}

// WriteTo flushes the buffered content of the metrics to the writer, in
// StatsD format. WriteTo abides best-effort semantics, so observations are
// lost if there is a problem with the write. Clients should be sure to call
// WriteTo regularly, ideally through the WriteLoop or SendLoop helper methods.
func (s *Statsd) WriteTo(w io.Writer) (count int64, err error) {
	var n int

	s.counters.Reset().Walk(func(name string, _ lv.LabelValues, values []float64) bool {
		n, err = fmt.Fprintf(w, "%s:%f|c%s\n", name, sum(values), sampling(s.rates.Get(name)))
		if err != nil {
			return false
		}
		count += int64(n)
		return true
	})
	if err != nil {
		return count, err
	}

	s.gauges.Reset().Walk(func(name string, _ lv.LabelValues, values []float64) bool {
		n, err = fmt.Fprintf(w, "%s:%f|g\n", name, last(values))
		if err != nil {
			return false
		}
		count += int64(n)
		return true
	})
	if err != nil {
		return count, err
	}

	s.timings.Reset().Walk(func(name string, _ lv.LabelValues, values []float64) bool {
		sampleRate := s.rates.Get(name)
		for _, value := range values {
			n, err = fmt.Fprintf(w, "%s:%f|ms%s\n", name, value, sampling(sampleRate))
			if err != nil {
				return false
			}
			count += int64(n)
		}
		return true
	})
	if err != nil {
		return count, err
	}

	return count, err
}

func sum(a []float64) float64 {
	var v float64
	for _, f := range a {
		v += f
	}
	return v
}

func last(a []float64) float64 {
	return a[len(a)-1]
}

func sampling(r float64) string {
	var sv string
	if r < 1.0 {
		sv = fmt.Sprintf("|@%f", r)
	}
	return sv
}

type observeFunc func(name string, lvs lv.LabelValues, value float64)

// Counter is a StatsD counter. Observations are forwarded to a Statsd object,
// and aggregated (summed) per timeseries.
type Counter struct {
	name string
	obs  observeFunc
}

// With is a no-op.
func (c *Counter) With(...string) metrics.Counter {
	return c
}

// Add implements metrics.Counter.
func (c *Counter) Add(delta float64) {
	c.obs(c.name, lv.LabelValues{}, delta)
}

// Gauge is a StatsD gauge. Observations are forwarded to a Statsd object, and
// aggregated (the last observation selected) per timeseries.
type Gauge struct {
	name string
	obs  observeFunc
	add  observeFunc
}

// With is a no-op.
func (g *Gauge) With(...string) metrics.Gauge {
	return g
}

// Set implements metrics.Gauge.
func (g *Gauge) Set(value float64) {
	g.obs(g.name, lv.LabelValues{}, value)
}

// Add implements metrics.Gauge.
func (g *Gauge) Add(delta float64) {
	g.add(g.name, lv.LabelValues{}, delta)
}

// Timing is a StatsD timing, or metrics.Histogram. Observations are
// forwarded to a Statsd object, and collected (but not aggregated) per
// timeseries.
type Timing struct {
	name string
	obs  observeFunc
}

// With is a no-op.
func (t *Timing) With(...string) metrics.Histogram {
	return t
}

// Observe implements metrics.Histogram. Value is interpreted as milliseconds.
func (t *Timing) Observe(value float64) {
	t.obs(t.name, lv.LabelValues{}, value)
}
