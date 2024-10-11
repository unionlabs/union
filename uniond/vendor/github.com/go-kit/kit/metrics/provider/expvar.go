package provider

import (
	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/expvar"
)

type expvarProvider struct{}

// NewExpvarProvider returns a Provider that produces expvar metrics.
func NewExpvarProvider() Provider {
	return expvarProvider{}
}

// NewCounter implements Provider.
func (p expvarProvider) NewCounter(name string) metrics.Counter {
	return expvar.NewCounter(name)
}

// NewGauge implements Provider.
func (p expvarProvider) NewGauge(name string) metrics.Gauge {
	return expvar.NewGauge(name)
}

// NewHistogram implements Provider.
func (p expvarProvider) NewHistogram(name string, buckets int) metrics.Histogram {
	return expvar.NewHistogram(name, buckets)
}

// Stop implements Provider, but is a no-op.
func (p expvarProvider) Stop() {}
