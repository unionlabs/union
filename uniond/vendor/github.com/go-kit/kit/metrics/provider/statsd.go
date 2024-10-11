package provider

import (
	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/statsd"
)

type statsdProvider struct {
	s    *statsd.Statsd
	stop func()
}

// NewStatsdProvider wraps the given Statsd object and stop func and returns a
// Provider that produces Statsd metrics. A typical stop function would be
// ticker.Stop from the ticker passed to the SendLoop helper method.
func NewStatsdProvider(s *statsd.Statsd, stop func()) Provider {
	return &statsdProvider{
		s:    s,
		stop: stop,
	}
}

// NewCounter implements Provider.
func (p *statsdProvider) NewCounter(name string) metrics.Counter {
	return p.s.NewCounter(name, 1.0)
}

// NewGauge implements Provider.
func (p *statsdProvider) NewGauge(name string) metrics.Gauge {
	return p.s.NewGauge(name)
}

// NewHistogram implements Provider, returning a StatsD Timing that accepts
// observations in milliseconds. The sample rate is fixed at 1.0. The bucket
// parameter is ignored.
func (p *statsdProvider) NewHistogram(name string, _ int) metrics.Histogram {
	return p.s.NewTiming(name, 1.0)
}

// Stop implements Provider, invoking the stop function passed at construction.
func (p *statsdProvider) Stop() {
	p.stop()
}
