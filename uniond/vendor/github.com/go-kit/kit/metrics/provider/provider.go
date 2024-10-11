// Package provider provides a factory-like abstraction for metrics backends.
// This package is provided specifically for the needs of the NY Times framework
// Gizmo. Most normal Go kit users shouldn't need to use it.
//
// Normally, if your microservice needs to support different metrics backends,
// you can simply do different construction based on a flag. For example,
//
//    var latency metrics.Histogram
//    var requests metrics.Counter
//    switch *metricsBackend {
//    case "prometheus":
//        latency = prometheus.NewSummaryVec(...)
//        requests = prometheus.NewCounterVec(...)
//    case "statsd":
//        s := statsd.New(...)
//        t := time.NewTicker(5*time.Second)
//        go s.SendLoop(ctx, t.C, "tcp", "statsd.local:8125")
//        latency = s.NewHistogram(...)
//        requests = s.NewCounter(...)
//    default:
//        log.Fatal("unsupported metrics backend %q", *metricsBackend)
//    }
//
package provider

import (
	"github.com/go-kit/kit/metrics"
)

// Provider abstracts over constructors and lifecycle management functions for
// each supported metrics backend. It should only be used by those who need to
// swap out implementations dynamically.
//
// This is primarily useful for intermediating frameworks, and is likely
// unnecessary for most Go kit services. See the package-level doc comment for
// more typical usage instructions.
type Provider interface {
	NewCounter(name string) metrics.Counter
	NewGauge(name string) metrics.Gauge
	NewHistogram(name string, buckets int) metrics.Histogram
	Stop()
}
