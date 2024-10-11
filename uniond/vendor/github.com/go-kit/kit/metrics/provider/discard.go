package provider

import (
	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/discard"
)

type discardProvider struct{}

// NewDiscardProvider returns a provider that produces no-op metrics via the
// discarding backend.
func NewDiscardProvider() Provider { return discardProvider{} }

// NewCounter implements Provider.
func (discardProvider) NewCounter(string) metrics.Counter { return discard.NewCounter() }

// NewGauge implements Provider.
func (discardProvider) NewGauge(string) metrics.Gauge { return discard.NewGauge() }

// NewHistogram implements Provider.
func (discardProvider) NewHistogram(string, int) metrics.Histogram { return discard.NewHistogram() }

// Stop implements Provider.
func (discardProvider) Stop() {}
