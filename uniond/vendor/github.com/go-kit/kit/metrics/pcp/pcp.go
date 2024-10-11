package pcp

import (
	"github.com/performancecopilot/speed/v4"

	"github.com/go-kit/kit/metrics"
)

// Reporter encapsulates a speed client.
type Reporter struct {
	c *speed.PCPClient
}

// NewReporter creates a new Reporter instance. The first parameter is the
// application name and is used to create the speed client. Hence it should be a
// valid speed parameter name and should not contain spaces or the path
// separator for your operating system.
func NewReporter(appname string) (*Reporter, error) {
	c, err := speed.NewPCPClient(appname)
	if err != nil {
		return nil, err
	}

	return &Reporter{c}, nil
}

// Start starts the underlying speed client so it can start reporting registered
// metrics to your PCP installation.
func (r *Reporter) Start() { r.c.MustStart() }

// Stop stops the underlying speed client so it can stop reporting registered
// metrics to your PCP installation.
func (r *Reporter) Stop() { r.c.MustStop() }

// Counter implements metrics.Counter via a single dimensional speed.Counter.
type Counter struct {
	c speed.Counter
}

// NewCounter creates a new Counter. This requires a name parameter and can
// optionally take a couple of description strings, that are used to create the
// underlying speed.Counter and are reported by PCP.
func (r *Reporter) NewCounter(name string, desc ...string) (*Counter, error) {
	c, err := speed.NewPCPCounter(0, name, desc...)
	if err != nil {
		return nil, err
	}

	r.c.MustRegister(c)
	return &Counter{c}, nil
}

// With is a no-op.
func (c *Counter) With(labelValues ...string) metrics.Counter { return c }

// Add increments Counter. speed.Counters only take int64, so delta is converted
// to int64 before observation.
func (c *Counter) Add(delta float64) { c.c.Inc(int64(delta)) }

// Gauge implements metrics.Gauge via a single dimensional speed.Gauge.
type Gauge struct {
	g speed.Gauge
}

// NewGauge creates a new Gauge. This requires a name parameter and can
// optionally take a couple of description strings, that are used to create the
// underlying speed.Gauge and are reported by PCP.
func (r *Reporter) NewGauge(name string, desc ...string) (*Gauge, error) {
	g, err := speed.NewPCPGauge(0, name, desc...)
	if err != nil {
		return nil, err
	}

	r.c.MustRegister(g)
	return &Gauge{g}, nil
}

// With is a no-op.
func (g *Gauge) With(labelValues ...string) metrics.Gauge { return g }

// Set sets the value of the gauge.
func (g *Gauge) Set(value float64) { g.g.Set(value) }

// Add adds a value to the gauge.
func (g *Gauge) Add(delta float64) { g.g.Inc(delta) }

// Histogram wraps a speed Histogram.
type Histogram struct {
	h speed.Histogram
}

// NewHistogram creates a new Histogram. The minimum observeable value is 0. The
// maximum observeable value is 3600000000 (3.6e9).
//
// The required parameters are a metric name, the minimum and maximum observable
// values, and a metric unit for the units of the observed values.
//
// Optionally, it can also take a couple of description strings.
func (r *Reporter) NewHistogram(name string, min, max int64, unit speed.MetricUnit, desc ...string) (*Histogram, error) {
	h, err := speed.NewPCPHistogram(name, min, max, 5, unit, desc...)
	if err != nil {
		return nil, err
	}

	r.c.MustRegister(h)
	return &Histogram{h}, nil
}

// With is a no-op.
func (h *Histogram) With(labelValues ...string) metrics.Histogram { return h }

// Observe observes a value.
//
// This converts float64 value to int64 before observation, as the Histogram in
// speed is backed using codahale/hdrhistogram, which only observes int64
// values. Additionally, the value is interpreted in the metric unit used to
// construct the histogram.
func (h *Histogram) Observe(value float64) { h.h.MustRecord(int64(value)) }

// Mean returns the mean of the values observed so far by the Histogram.
func (h *Histogram) Mean() float64 { return h.h.Mean() }

// Percentile returns a percentile value for the given percentile
// between 0 and 100 for all values observed by the histogram.
func (h *Histogram) Percentile(p float64) int64 { return h.h.Percentile(p) }
