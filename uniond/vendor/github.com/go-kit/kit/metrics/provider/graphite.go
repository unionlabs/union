package provider

import (
	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/graphite"
)

type graphiteProvider struct {
	g    *graphite.Graphite
	stop func()
}

// NewGraphiteProvider wraps the given Graphite object and stop func and returns
// a Provider that produces Graphite metrics. A typical stop function would be
// ticker.Stop from the ticker passed to the SendLoop helper method.
func NewGraphiteProvider(g *graphite.Graphite, stop func()) Provider {
	return &graphiteProvider{
		g:    g,
		stop: stop,
	}
}

// NewCounter implements Provider.
func (p *graphiteProvider) NewCounter(name string) metrics.Counter {
	return p.g.NewCounter(name)
}

// NewGauge implements Provider.
func (p *graphiteProvider) NewGauge(name string) metrics.Gauge {
	return p.g.NewGauge(name)
}

// NewHistogram implements Provider.
func (p *graphiteProvider) NewHistogram(name string, buckets int) metrics.Histogram {
	return p.g.NewHistogram(name, buckets)
}

// Stop implements Provider, invoking the stop function passed at construction.
func (p *graphiteProvider) Stop() {
	p.stop()
}
