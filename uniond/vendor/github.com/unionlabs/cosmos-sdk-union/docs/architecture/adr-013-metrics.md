# ADR 013: Observability

## Changelog

* 20-01-2020: Initial Draft

## Status

Proposed

## Context

Telemetry is paramount into debugging and understanding what the application is doing and how it is
performing. We aim to expose metrics from modules and other core parts of the Cosmos SDK.

In addition, we should aim to support multiple configurable sinks that an operator may choose from.
By default, when telemetry is enabled, the application should track and expose metrics that are
stored in-memory. The operator may choose to enable additional sinks, where we support only
[Prometheus](https://prometheus.io/) for now, as it's battle-tested, simple to setup, open source,
and is rich with ecosystem tooling.

We must also aim to integrate metrics into the Cosmos SDK in the most seamless way possible such that
metrics may be added or removed at will and without much friction. To do this, we will use the
[go-metrics](https://github.com/hashicorp/go-metrics) library.

Finally, operators may enable telemetry along with specific configuration options. If enabled, metrics
will be exposed via `/metrics?format={text|prometheus}` via the API server.

## Decision

We will add an additional configuration block to `app.toml` that defines telemetry settings:

```toml
###############################################################################
###                         Telemetry Configuration                         ###
###############################################################################

[telemetry]

# Prefixed with keys to separate services
service-name = {{ .Telemetry.ServiceName }}

# Enabled enables the application telemetry functionality. When enabled,
# an in-memory sink is also enabled by default. Operators may also enabled
# other sinks such as Prometheus.
enabled = {{ .Telemetry.Enabled }}

# Enable prefixing gauge values with hostname
enable-hostname = {{ .Telemetry.EnableHostname }}

# Enable adding hostname to labels
enable-hostname-label = {{ .Telemetry.EnableHostnameLabel }}

# Enable adding service to labels
enable-service-label = {{ .Telemetry.EnableServiceLabel }}

# PrometheusRetentionTime, when positive, enables a Prometheus metrics sink.
prometheus-retention-time = {{ .Telemetry.PrometheusRetentionTime }}
```

The given configuration allows for two sinks -- in-memory and Prometheus. We create a `Metrics`
type that performs all the bootstrapping for the operator, so capturing metrics becomes seamless.

```go
// Metrics defines a wrapper around application telemetry functionality. It allows
// metrics to be gathered at any point in time. When creating a Metrics object,
// internally, a global metrics is registered with a set of sinks as configured
// by the operator. In addition to the sinks, when a process gets a SIGUSR1, a
// dump of formatted recent metrics will be sent to STDERR.
type Metrics struct {
  memSink           *metrics.InmemSink
  prometheusEnabled bool
}

// Gather collects all registered metrics and returns a GatherResponse where the
// metrics are encoded depending on the type. Metrics are either encoded via
// Prometheus or JSON if in-memory.
func (m *Metrics) Gather(format string) (GatherResponse, error) {
  switch format {
  case FormatPrometheus:
    return m.gatherPrometheus()

  case FormatText:
    return m.gatherGeneric()

  case FormatDefault:
    return m.gatherGeneric()

  default:
    return GatherResponse{}, fmt.Errorf("unsupported metrics format: %s", format)
  }
}
```

In addition, `Metrics` allows us to gather the current set of metrics at any given point in time. An
operator may also choose to send a signal, SIGUSR1, to dump and print formatted metrics to STDERR.

During an application's bootstrapping and construction phase, if `Telemetry.Enabled` is `true`, the
API server will create an instance of a reference to `Metrics` object and will register a metrics
handler accordingly.

```go
func (s *Server) Start(cfg config.Config) error {
  // ...

  if cfg.Telemetry.Enabled {
    m, err := telemetry.New(cfg.Telemetry)
    if err != nil {
      return err
    }

    s.metrics = m
    s.registerMetrics()
  }

  // ...
}

func (s *Server) registerMetrics() {
  metricsHandler := func(w http.ResponseWriter, r *http.Request) {
    format := strings.TrimSpace(r.FormValue("format"))

    gr, err := s.metrics.Gather(format)
    if err != nil {
      rest.WriteErrorResponse(w, http.StatusBadRequest, fmt.Sprintf("failed to gather metrics: %s", err))
      return
    }

    w.Header().Set("Content-Type", gr.ContentType)
    _, _ = w.Write(gr.Metrics)
  }

  s.Router.HandleFunc("/metrics", metricsHandler).Methods("GET")
}
```

Application developers may track counters, gauges, summaries, and key/value metrics. There is no
additional lifting required by modules to leverage profiling metrics. To do so, it's as simple as:

```go
func (k BaseKeeper) MintCoins(ctx sdk.Context, moduleName string, amt sdk.Coins) error {
  defer metrics.MeasureSince(time.Now(), "MintCoins")
  // ...
}
```

## Consequences

### Positive

* Exposure into the performance and behavior of an application

### Negative

### Neutral

## References
