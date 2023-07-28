# package metrics

`package metrics` provides a set of uniform interfaces for service instrumentation.
It has
 [counters](http://prometheus.io/docs/concepts/metric_types/#counter),
 [gauges](http://prometheus.io/docs/concepts/metric_types/#gauge), and
 [histograms](http://prometheus.io/docs/concepts/metric_types/#histogram),
and provides adapters to popular metrics packages, like
 [expvar](https://golang.org/pkg/expvar),
 [StatsD](https://github.com/etsy/statsd), and
 [Prometheus](https://prometheus.io).

## Rationale

Code instrumentation is absolutely essential to achieve
 [observability](https://speakerdeck.com/mattheath/observability-in-micro-service-architectures)
 into a distributed system.
Metrics and instrumentation tools have coalesced around a few well-defined idioms.
`package metrics` provides a common, minimal interface those idioms for service authors.

## Usage

A simple counter, exported via expvar.

```go
import (
	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/expvar"
)

func main() {
	var myCount metrics.Counter
	myCount = expvar.NewCounter("my_count")
	myCount.Add(1)
}
```

A histogram for request duration,
 exported via a Prometheus summary with dynamically-computed quantiles.

```go
import (
	"time"

	stdprometheus "github.com/prometheus/client_golang/prometheus"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/prometheus"
)

func main() {
	var dur metrics.Histogram = prometheus.NewSummaryFrom(stdprometheus.SummaryOpts{
		Namespace: "myservice",
		Subsystem: "api",
		Name:     "request_duration_seconds",
		Help:     "Total time spent serving requests.",
	}, []string{})
	// ...
}

func handleRequest(dur metrics.Histogram) {
	defer func(begin time.Time) { dur.Observe(time.Since(begin).Seconds()) }(time.Now())
	// handle request
}
```

A gauge for the number of goroutines currently running, exported via StatsD.

```go
import (
	"context"
	"net"
	"os"
	"runtime"
	"time"

	"github.com/go-kit/kit/metrics"
	"github.com/go-kit/kit/metrics/statsd"
)

func main() {
	statsd := statsd.New("foo_svc.", log.NewNopLogger())
	report := time.NewTicker(5 * time.Second)
	defer report.Stop()
	go statsd.SendLoop(context.Background(), report.C, "tcp", "statsd.internal:8125")
	goroutines := statsd.NewGauge("goroutine_count")
	go exportGoroutines(goroutines)
	// ...
}

func exportGoroutines(g metrics.Gauge) {
	for range time.Tick(time.Second) {
		g.Set(float64(runtime.NumGoroutine()))
	}
}
```

For more information, see [the package documentation](https://godoc.org/github.com/go-kit/kit/metrics).
