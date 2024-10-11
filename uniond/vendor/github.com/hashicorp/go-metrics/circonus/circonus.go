// Circonus Metrics Sink

package circonus

import (
	"strings"

	cgm "github.com/circonus-labs/circonus-gometrics"
	"github.com/hashicorp/go-metrics"
)

// CirconusSink provides an interface to forward metrics to Circonus with
// automatic check creation and metric management
type CirconusSink struct {
	metrics *cgm.CirconusMetrics
}

// Config options for CirconusSink
// See https://github.com/circonus-labs/circonus-gometrics for configuration options
type Config cgm.Config

// NewCirconusSink - create new metric sink for circonus
//
// one of the following must be supplied:
//   - API Token - search for an existing check or create a new check
//   - API Token + Check Id - the check identified by check id will be used
//   - API Token + Check Submission URL - the check identified by the submission url will be used
//   - Check Submission URL - the check identified by the submission url will be used
//     metric management will be *disabled*
//
// Note: If submission url is supplied w/o an api token, the public circonus ca cert will be used
// to verify the broker for metrics submission.
func NewCirconusSink(cc *Config) (*CirconusSink, error) {
	cfg := cgm.Config{}
	if cc != nil {
		cfg = cgm.Config(*cc)
	}

	metrics, err := cgm.NewCirconusMetrics(&cfg)
	if err != nil {
		return nil, err
	}

	return &CirconusSink{
		metrics: metrics,
	}, nil
}

// Start submitting metrics to Circonus (flush every SubmitInterval)
func (s *CirconusSink) Start() {
	s.metrics.Start()
}

// Flush manually triggers metric submission to Circonus
func (s *CirconusSink) Flush() {
	s.metrics.Flush()
}

// SetGauge sets value for a gauge metric
func (s *CirconusSink) SetGauge(key []string, val float32) {
	flatKey := s.flattenKey(key)
	s.metrics.SetGauge(flatKey, int64(val))
}

// SetGaugeWithLabels sets value for a gauge metric with the given labels
func (s *CirconusSink) SetGaugeWithLabels(key []string, val float32, labels []metrics.Label) {
	flatKey := s.flattenKeyLabels(key, labels)
	s.metrics.SetGauge(flatKey, int64(val))
}

// SetPrecisionGauge sets value for a gauge metric with float64 precision
func (s *CirconusSink) SetPrecisionGauge(key []string, val float64) {
	flatKey := s.flattenKey(key)
	s.metrics.SetGauge(flatKey, val)
}

// SetPrecisionGaugeWithLabels sets value for a gauge metric with the given labels with float64 precision
func (s *CirconusSink) SetPrecisionGaugeWithLabels(key []string, val float64, labels []metrics.Label) {
	flatKey := s.flattenKeyLabels(key, labels)
	s.metrics.SetGauge(flatKey, val)
}

// EmitKey is not implemented in circonus
func (s *CirconusSink) EmitKey(key []string, val float32) {
	// NOP
}

// IncrCounter increments a counter metric
func (s *CirconusSink) IncrCounter(key []string, val float32) {
	flatKey := s.flattenKey(key)
	s.metrics.IncrementByValue(flatKey, uint64(val))
}

// IncrCounterWithLabels increments a counter metric with the given labels
func (s *CirconusSink) IncrCounterWithLabels(key []string, val float32, labels []metrics.Label) {
	flatKey := s.flattenKeyLabels(key, labels)
	s.metrics.IncrementByValue(flatKey, uint64(val))
}

// AddSample adds a sample to a histogram metric
func (s *CirconusSink) AddSample(key []string, val float32) {
	flatKey := s.flattenKey(key)
	s.metrics.RecordValue(flatKey, float64(val))
}

// AddSampleWithLabels adds a sample to a histogram metric with the given labels
func (s *CirconusSink) AddSampleWithLabels(key []string, val float32, labels []metrics.Label) {
	flatKey := s.flattenKeyLabels(key, labels)
	s.metrics.RecordValue(flatKey, float64(val))
}

// Shutdown blocks while flushing metrics to the backend.
func (s *CirconusSink) Shutdown() {
	// The version of circonus metrics in go.mod (v2.3.1), and the current
	// version (v3.4.6) do not support a shutdown operation. Instead we call
	// Flush which blocks until metrics are submitted to storage, and then exit
	// as the README examples do.
	s.metrics.Flush()
}

// Flattens key to Circonus metric name
func (s *CirconusSink) flattenKey(parts []string) string {
	joined := strings.Join(parts, "`")
	return strings.Map(func(r rune) rune {
		switch r {
		case ' ':
			return '_'
		default:
			return r
		}
	}, joined)
}

// Flattens the key along with labels for formatting, removes spaces
func (s *CirconusSink) flattenKeyLabels(parts []string, labels []metrics.Label) string {
	for _, label := range labels {
		parts = append(parts, label.Value)
	}
	return s.flattenKey(parts)
}
