package telemetry

import (
	"bytes"
	"encoding/json"
	"fmt"
	"time"

	"github.com/hashicorp/go-metrics"
	metricsprom "github.com/hashicorp/go-metrics/prometheus"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/common/expfmt"
)

// globalLabels defines the set of global labels that will be applied to all
// metrics emitted using the telemetry package function wrappers.
var globalLabels = []metrics.Label{}

// Metrics supported format types.
const (
	FormatDefault    = ""
	FormatPrometheus = "prometheus"
	FormatText       = "text"
)

// Config defines the configuration options for application telemetry.
type Config struct {
	// Prefixed with keys to separate services
	ServiceName string `mapstructure:"service-name"`

	// Enabled enables the application telemetry functionality. When enabled,
	// an in-memory sink is also enabled by default. Operators may also enabled
	// other sinks such as Prometheus.
	Enabled bool `mapstructure:"enabled"`

	// Enable prefixing gauge values with hostname
	EnableHostname bool `mapstructure:"enable-hostname"`

	// Enable adding hostname to labels
	EnableHostnameLabel bool `mapstructure:"enable-hostname-label"`

	// Enable adding service to labels
	EnableServiceLabel bool `mapstructure:"enable-service-label"`

	// PrometheusRetentionTime, when positive, enables a Prometheus metrics sink.
	// It defines the retention duration in seconds.
	PrometheusRetentionTime int64 `mapstructure:"prometheus-retention-time"`

	// GlobalLabels defines a global set of name/value label tuples applied to all
	// metrics emitted using the wrapper functions defined in telemetry package.
	//
	// Example:
	// [["chain_id", "cosmoshub-1"]]
	GlobalLabels [][]string `mapstructure:"global-labels"`
}

// Metrics defines a wrapper around application telemetry functionality. It allows
// metrics to be gathered at any point in time. When creating a Metrics object,
// internally, a global metrics is registered with a set of sinks as configured
// by the operator. In addition to the sinks, when a process gets a SIGUSR1, a
// dump of formatted recent metrics will be sent to STDERR.
type Metrics struct {
	memSink           *metrics.InmemSink
	prometheusEnabled bool
}

// GatherResponse is the response type of registered metrics
type GatherResponse struct {
	Metrics     []byte
	ContentType string
}

// New creates a new instance of Metrics
func New(cfg Config) (_ *Metrics, rerr error) {
	if !cfg.Enabled {
		return nil, nil
	}

	if numGlobalLables := len(cfg.GlobalLabels); numGlobalLables > 0 {
		parsedGlobalLabels := make([]metrics.Label, numGlobalLables)
		for i, gl := range cfg.GlobalLabels {
			parsedGlobalLabels[i] = NewLabel(gl[0], gl[1])
		}

		globalLabels = parsedGlobalLabels
	}

	metricsConf := metrics.DefaultConfig(cfg.ServiceName)
	metricsConf.EnableHostname = cfg.EnableHostname
	metricsConf.EnableHostnameLabel = cfg.EnableHostnameLabel

	memSink := metrics.NewInmemSink(10*time.Second, time.Minute)
	inMemSig := metrics.DefaultInmemSignal(memSink)
	defer func() {
		if rerr != nil {
			inMemSig.Stop()
		}
	}()

	m := &Metrics{memSink: memSink}
	fanout := metrics.FanoutSink{memSink}

	if cfg.PrometheusRetentionTime > 0 {
		m.prometheusEnabled = true
		prometheusOpts := metricsprom.PrometheusOpts{
			Expiration: time.Duration(cfg.PrometheusRetentionTime) * time.Second,
		}

		promSink, err := metricsprom.NewPrometheusSinkFrom(prometheusOpts)
		if err != nil {
			return nil, err
		}

		fanout = append(fanout, promSink)
	}

	if _, err := metrics.NewGlobal(metricsConf, fanout); err != nil {
		return nil, err
	}

	return m, nil
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

func (m *Metrics) gatherPrometheus() (GatherResponse, error) {
	if !m.prometheusEnabled {
		return GatherResponse{}, fmt.Errorf("prometheus metrics are not enabled")
	}

	metricsFamilies, err := prometheus.DefaultGatherer.Gather()
	if err != nil {
		return GatherResponse{}, fmt.Errorf("failed to gather prometheus metrics: %w", err)
	}

	buf := &bytes.Buffer{}
	defer buf.Reset()

	e := expfmt.NewEncoder(buf, expfmt.FmtText)
	for _, mf := range metricsFamilies {
		if err := e.Encode(mf); err != nil {
			return GatherResponse{}, fmt.Errorf("failed to encode prometheus metrics: %w", err)
		}
	}

	return GatherResponse{ContentType: string(expfmt.FmtText), Metrics: buf.Bytes()}, nil
}

func (m *Metrics) gatherGeneric() (GatherResponse, error) {
	summary, err := m.memSink.DisplayMetrics(nil, nil)
	if err != nil {
		return GatherResponse{}, fmt.Errorf("failed to gather in-memory metrics: %w", err)
	}

	content, err := json.Marshal(summary)
	if err != nil {
		return GatherResponse{}, fmt.Errorf("failed to encode in-memory metrics: %w", err)
	}

	return GatherResponse{ContentType: "application/json", Metrics: content}, nil
}
