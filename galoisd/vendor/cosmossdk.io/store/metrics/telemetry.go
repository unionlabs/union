package metrics

import (
	"time"

	"github.com/hashicorp/go-metrics"
)

// StoreMetrics defines the set of metrics for the store package
type StoreMetrics interface {
	MeasureSince(keys ...string)
}

var (
	_ StoreMetrics = Metrics{}
	_ StoreMetrics = NoOpMetrics{}
)

// Metrics defines the metrics wrapper for the store package
type Metrics struct {
	Labels []metrics.Label
}

// NewMetrics returns a new instance of the Metrics with labels set by the node operator
func NewMetrics(labels [][]string) Metrics {
	gatherer := Metrics{}

	if numGlobalLables := len(labels); numGlobalLables > 0 {
		parsedGlobalLabels := make([]metrics.Label, numGlobalLables)
		for i, gl := range labels {
			parsedGlobalLabels[i] = metrics.Label{Name: gl[0], Value: gl[1]}
		}

		gatherer.Labels = parsedGlobalLabels
	}

	return gatherer
}

// MeasureSince provides a wrapper functionality for emitting a time measure
// metric with global labels (if any).
func (m Metrics) MeasureSince(keys ...string) {
	start := time.Now()
	metrics.MeasureSinceWithLabels(keys, start.UTC(), m.Labels)
}

// NoOpMetrics is a no-op implementation of the StoreMetrics interface
type NoOpMetrics struct{}

// NewNoOpMetrics returns a new instance of the NoOpMetrics
func NewNoOpMetrics() NoOpMetrics {
	return NoOpMetrics{}
}

// MeasureSince is a no-op implementation of the StoreMetrics interface to avoid time.Now() calls
func (m NoOpMetrics) MeasureSince(keys ...string) {}
