package datadog

import (
	"fmt"
	"strings"

	"github.com/DataDog/datadog-go/statsd"
	"github.com/hashicorp/go-metrics"
)

// DogStatsdSink provides a MetricSink that can be used
// with a dogstatsd server. It utilizes the Dogstatsd client at github.com/DataDog/datadog-go/statsd
type DogStatsdSink struct {
	client            *statsd.Client
	hostName          string
	propagateHostname bool
}

// NewDogStatsdSink is used to create a new DogStatsdSink with sane defaults
func NewDogStatsdSink(addr string, hostName string) (*DogStatsdSink, error) {
	client, err := statsd.New(addr)
	if err != nil {
		return nil, err
	}
	sink := &DogStatsdSink{
		client:            client,
		hostName:          hostName,
		propagateHostname: false,
	}
	return sink, nil
}

// SetTags sets common tags on the Dogstatsd Client that will be sent
// along with all dogstatsd packets.
// Ref: http://docs.datadoghq.com/guides/dogstatsd/#tags
func (s *DogStatsdSink) SetTags(tags []string) {
	s.client.Tags = tags
}

// EnableHostnamePropagation forces a Dogstatsd `host` tag with the value specified by `s.HostName`
// Since the go-metrics package has its own mechanism for attaching a hostname to metrics,
// setting the `propagateHostname` flag ensures that `s.HostName` overrides the host tag naively set by the DogStatsd server
func (s *DogStatsdSink) EnableHostNamePropagation() {
	s.propagateHostname = true
}

func (s *DogStatsdSink) flattenKey(parts []string) string {
	joined := strings.Join(parts, ".")
	return strings.Map(sanitize, joined)
}

func sanitize(r rune) rune {
	switch r {
	case ':':
		fallthrough
	case ' ':
		return '_'
	default:
		return r
	}
}

func (s *DogStatsdSink) parseKey(key []string) ([]string, []metrics.Label) {
	// Since DogStatsd supports dimensionality via tags on metric keys, this sink's approach is to splice the hostname out of the key in favor of a `host` tag
	// The `host` tag is either forced here, or set downstream by the DogStatsd server

	var labels []metrics.Label
	hostName := s.hostName

	// Splice the hostname out of the key
	for i, el := range key {
		if el == hostName {
			// We need an intermediate key to prevent clobbering the
			// original backing array that other sinks might be consuming.
			tempKey := append([]string{}, key[:i]...)
			key = append(tempKey, key[i+1:]...)
			break
		}
	}

	if s.propagateHostname {
		labels = append(labels, metrics.Label{"host", hostName})
	}
	return key, labels
}

// Implementation of methods in the MetricSink interface

func (s *DogStatsdSink) SetGauge(key []string, val float32) {
	s.SetGaugeWithLabels(key, val, nil)
}

func (s *DogStatsdSink) SetPrecisionGauge(key []string, val float64) {
	s.SetPrecisionGaugeWithLabels(key, val, nil)
}

func (s *DogStatsdSink) IncrCounter(key []string, val float32) {
	s.IncrCounterWithLabels(key, val, nil)
}

// EmitKey is not implemented since DogStatsd does not provide a metric type that holds an
// arbitrary number of values
func (s *DogStatsdSink) EmitKey(key []string, val float32) {
}

func (s *DogStatsdSink) AddSample(key []string, val float32) {
	s.AddSampleWithLabels(key, val, nil)
}

// The following ...WithLabels methods correspond to Datadog's Tag extension to Statsd.
// http://docs.datadoghq.com/guides/dogstatsd/#tags
func (s *DogStatsdSink) SetGaugeWithLabels(key []string, val float32, labels []metrics.Label) {
	flatKey, tags := s.getFlatkeyAndCombinedLabels(key, labels)
	rate := 1.0
	s.client.Gauge(flatKey, float64(val), tags, rate)
}

// The following ...WithLabels methods correspond to Datadog's Tag extension to Statsd.
// http://docs.datadoghq.com/guides/dogstatsd/#tags
func (s *DogStatsdSink) SetPrecisionGaugeWithLabels(key []string, val float64, labels []metrics.Label) {
	flatKey, tags := s.getFlatkeyAndCombinedLabels(key, labels)
	rate := 1.0
	s.client.Gauge(flatKey, val, tags, rate)
}

func (s *DogStatsdSink) IncrCounterWithLabels(key []string, val float32, labels []metrics.Label) {
	flatKey, tags := s.getFlatkeyAndCombinedLabels(key, labels)
	rate := 1.0
	s.client.Count(flatKey, int64(val), tags, rate)
}

func (s *DogStatsdSink) AddSampleWithLabels(key []string, val float32, labels []metrics.Label) {
	flatKey, tags := s.getFlatkeyAndCombinedLabels(key, labels)
	rate := 1.0
	s.client.TimeInMilliseconds(flatKey, float64(val), tags, rate)
}

// Shutdown disables further metric collection, blocks to flush data, and tears down the sink.
func (s *DogStatsdSink) Shutdown() {
	s.client.Close()
}

func (s *DogStatsdSink) getFlatkeyAndCombinedLabels(key []string, labels []metrics.Label) (string, []string) {
	key, parsedLabels := s.parseKey(key)
	flatKey := s.flattenKey(key)
	labels = append(labels, parsedLabels...)

	var tags []string
	for _, label := range labels {
		label.Name = strings.Map(sanitize, label.Name)
		label.Value = strings.Map(sanitize, label.Value)
		if label.Value != "" {
			tags = append(tags, fmt.Sprintf("%s:%s", label.Name, label.Value))
		} else {
			tags = append(tags, label.Name)
		}
	}

	return flatKey, tags
}
