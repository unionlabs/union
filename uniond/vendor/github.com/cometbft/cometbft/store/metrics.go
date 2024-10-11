package store

import (
	"github.com/go-kit/kit/metrics"
)

const (
	// MetricsSubsystem is a subsystem shared by all metrics exposed by this
	// package.
	MetricsSubsystem = "store"
)

//go:generate go run ../scripts/metricsgen -struct=Metrics

// Metrics contains metrics exposed by this package.
type Metrics struct {
	// The duration of accesses to the state store labeled by which method
	// was called on the store.
	BlockStoreAccessDurationSeconds metrics.Histogram `metrics_bucketsizes:"0.0002, 10, 5" metrics_buckettype:"exp" metrics_labels:"method"`
}
