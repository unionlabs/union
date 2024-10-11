package provider

import (
	stdprometheus "github.com/prometheus/client_golang/prometheus"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/prometheus"
)

type prometheusProvider struct {
	namespace string
	subsystem string
}

// NewPrometheusProvider returns a Provider that produces Prometheus metrics.
// Namespace and subsystem are applied to all produced metrics.
func NewPrometheusProvider(namespace, subsystem string) Provider {
	return &prometheusProvider{
		namespace: namespace,
		subsystem: subsystem,
	}
}

// NewCounter implements Provider via prometheus.NewCounterFrom, i.e. the
// counter is registered. The metric's namespace and subsystem are taken from
// the Provider. Help is set to the name of the metric, and no const label names
// are set.
func (p *prometheusProvider) NewCounter(name string) metrics.Counter {
	return prometheus.NewCounterFrom(stdprometheus.CounterOpts{
		Namespace: p.namespace,
		Subsystem: p.subsystem,
		Name:      name,
		Help:      name,
	}, []string{})
}

// NewGauge implements Provider via prometheus.NewGaugeFrom, i.e. the gauge is
// registered. The metric's namespace and subsystem are taken from the Provider.
// Help is set to the name of the metric, and no const label names are set.
func (p *prometheusProvider) NewGauge(name string) metrics.Gauge {
	return prometheus.NewGaugeFrom(stdprometheus.GaugeOpts{
		Namespace: p.namespace,
		Subsystem: p.subsystem,
		Name:      name,
		Help:      name,
	}, []string{})
}

// NewHistogram implements Provider via prometheus.NewSummaryFrom, i.e. the summary
// is registered. The metric's namespace and subsystem are taken from the
// Provider. Help is set to the name of the metric, and no const label names are
// set. Buckets are ignored.
func (p *prometheusProvider) NewHistogram(name string, _ int) metrics.Histogram {
	return prometheus.NewSummaryFrom(stdprometheus.SummaryOpts{
		Namespace: p.namespace,
		Subsystem: p.subsystem,
		Name:      name,
		Help:      name,
	}, []string{})
}

// Stop implements Provider, but is a no-op.
func (p *prometheusProvider) Stop() {}
