// Package metrics provides a framework for application instrumentation. It's
// primarily designed to help you get started with good and robust
// instrumentation, and to help you migrate from a less-capable system like
// Graphite to a more-capable system like Prometheus. If your organization has
// already standardized on an instrumentation system like Prometheus, and has no
// plans to change, it may make sense to use that system's instrumentation
// library directly.
//
// This package provides three core metric abstractions (Counter, Gauge, and
// Histogram) and implementations for almost all common instrumentation
// backends. Each metric has an observation method (Add, Set, or Observe,
// respectively) used to record values, and a With method to "scope" the
// observation by various parameters. For example, you might have a Histogram to
// record request durations, parameterized by the method that's being called.
//
//    var requestDuration metrics.Histogram
//    // ...
//    requestDuration.With("method", "MyMethod").Observe(time.Since(begin))
//
// This allows a single high-level metrics object (requestDuration) to work with
// many code paths somewhat dynamically. The concept of With is fully supported
// in some backends like Prometheus, and not supported in other backends like
// Graphite. So, With may be a no-op, depending on the concrete implementation
// you choose. Please check the implementation to know for sure. For
// implementations that don't provide With, it's necessary to fully parameterize
// each metric in the metric name, e.g.
//
//    // Statsd
//    c := statsd.NewCounter("request_duration_MyMethod_200")
//    c.Add(1)
//
//    // Prometheus
//    c := prometheus.NewCounter(stdprometheus.CounterOpts{
//        Name: "request_duration",
//        ...
//    }, []string{"method", "status_code"})
//    c.With("method", "MyMethod", "status_code", strconv.Itoa(code)).Add(1)
//
// Usage
//
// Metrics are dependencies, and should be passed to the components that need
// them in the same way you'd construct and pass a database handle, or reference
// to another component. Metrics should *not* be created in the global scope.
// Instead, instantiate metrics in your func main, using whichever concrete
// implementation is appropriate for your organization.
//
//    latency := prometheus.NewSummaryFrom(stdprometheus.SummaryOpts{
//        Namespace: "myteam",
//        Subsystem: "foosvc",
//        Name:      "request_latency_seconds",
//        Help:      "Incoming request latency in seconds.",
//    }, []string{"method", "status_code"})
//
// Write your components to take the metrics they will use as parameters to
// their constructors. Use the interface types, not the concrete types. That is,
//
//    // NewAPI takes metrics.Histogram, not *prometheus.Summary
//    func NewAPI(s Store, logger log.Logger, latency metrics.Histogram) *API {
//        // ...
//    }
//
//    func (a *API) ServeFoo(w http.ResponseWriter, r *http.Request) {
//        begin := time.Now()
//        // ...
//        a.latency.Observe(time.Since(begin).Seconds())
//    }
//
// Finally, pass the metrics as dependencies when building your object graph.
// This should happen in func main, not in the global scope.
//
//    api := NewAPI(store, logger, latency)
//    http.ListenAndServe("/", api)
//
// Note that metrics are "write-only" interfaces.
//
// Implementation details
//
// All metrics are safe for concurrent use. Considerable design influence has
// been taken from https://github.com/codahale/metrics and
// https://prometheus.io.
//
// Each telemetry system has different semantics for label values, push vs.
// pull, support for histograms, etc. These properties influence the design of
// their respective packages. This table attempts to summarize the key points of
// distinction.
//
//    SYSTEM      DIM  COUNTERS               GAUGES                 HISTOGRAMS
//    dogstatsd   n    batch, push-aggregate  batch, push-aggregate  native, batch, push-each
//    statsd      1    batch, push-aggregate  batch, push-aggregate  native, batch, push-each
//    graphite    1    batch, push-aggregate  batch, push-aggregate  synthetic, batch, push-aggregate
//    expvar      1    atomic                 atomic                 synthetic, batch, in-place expose
//    influx      n    custom                 custom                 custom
//    prometheus  n    native                 native                 native
//    pcp         1    native                 native                 native
//    cloudwatch  n    batch push-aggregate   batch push-aggregate   synthetic, batch, push-aggregate
//
package metrics
