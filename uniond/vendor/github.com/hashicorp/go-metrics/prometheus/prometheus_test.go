package prometheus

import (
	"errors"
	"fmt"
	"log"
	"net/http"
	"net/http/httptest"
	"net/url"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/golang/protobuf/proto"
	dto "github.com/prometheus/client_model/go"

	"github.com/hashicorp/go-metrics"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/common/expfmt"
)

const (
	TestHostname = "test_hostname"
)

func TestNewPrometheusSinkFrom(t *testing.T) {
	reg := prometheus.NewRegistry()

	sink, err := NewPrometheusSinkFrom(PrometheusOpts{
		Registerer: reg,
	})

	if err != nil {
		t.Fatalf("err = %v, want nil", err)
	}

	//check if register has a sink by unregistering it.
	ok := reg.Unregister(sink)
	if !ok {
		t.Fatalf("Unregister(sink) = false, want true")
	}
}

func TestNewPrometheusSink(t *testing.T) {
	sink, err := NewPrometheusSink()
	if err != nil {
		t.Fatalf("err = %v, want nil", err)
	}

	//check if register has a sink by unregistering it.
	ok := prometheus.Unregister(sink)
	if !ok {
		t.Fatalf("Unregister(sink) = false, want true")
	}
}

// TestMultiplePrometheusSink tests registering multiple sinks on the same registerer with different descriptors
func TestMultiplePrometheusSink(t *testing.T) {
	gaugeDef := GaugeDefinition{
		Name: []string{"my", "test", "gauge"},
		Help: "A gauge for testing? How helpful!",
	}

	cfg := PrometheusOpts{
		Expiration:         5 * time.Second,
		GaugeDefinitions:   append([]GaugeDefinition{}, gaugeDef),
		SummaryDefinitions: append([]SummaryDefinition{}),
		CounterDefinitions: append([]CounterDefinition{}),
		Name:               "sink1",
	}

	sink1, err := NewPrometheusSinkFrom(cfg)
	if err != nil {
		t.Fatalf("err = %v, want nil", err)
	}

	reg := prometheus.DefaultRegisterer
	if reg == nil {
		t.Fatalf("Expected default register to be non nil, got nil.")
	}

	gaugeDef2 := GaugeDefinition{
		Name: []string{"my2", "test", "gauge"},
		Help: "A gauge for testing? How helpful!",
	}

	cfg2 := PrometheusOpts{
		Expiration:         15 * time.Second,
		GaugeDefinitions:   append([]GaugeDefinition{}, gaugeDef2),
		SummaryDefinitions: append([]SummaryDefinition{}),
		CounterDefinitions: append([]CounterDefinition{}),
		// commenting out the name to point out that the default name will be used here instead
		// Name:               "sink2",
	}

	sink2, err := NewPrometheusSinkFrom(cfg2)
	if err != nil {
		t.Fatalf("err = %v, want nil", err)
	}
	//check if register has a sink by unregistering it.
	ok := reg.Unregister(sink1)
	if !ok {
		t.Fatalf("Unregister(sink) = false, want true")
	}

	//check if register has a sink by unregistering it.
	ok = reg.Unregister(sink2)
	if !ok {
		t.Fatalf("Unregister(sink) = false, want true")
	}
}

func TestDefinitions(t *testing.T) {
	gaugeDef := GaugeDefinition{
		Name: []string{"my", "test", "gauge"},
		Help: "A gauge for testing? How helpful!",
	}
	summaryDef := SummaryDefinition{
		Name: []string{"my", "test", "summary"},
		Help: "A summary for testing? How helpful!",
	}
	counterDef := CounterDefinition{
		Name: []string{"my", "test", "counter"},
		Help: "A counter for testing? How helpful!",
	}

	// PrometheusSink config w/ definitions for each metric type
	cfg := PrometheusOpts{
		Expiration:         5 * time.Second,
		GaugeDefinitions:   append([]GaugeDefinition{}, gaugeDef),
		SummaryDefinitions: append([]SummaryDefinition{}, summaryDef),
		CounterDefinitions: append([]CounterDefinition{}, counterDef),
	}
	sink, err := NewPrometheusSinkFrom(cfg)
	if err != nil {
		t.Fatalf("err = %v, want nil", err)
	}
	defer prometheus.Unregister(sink)

	// We can't just len(x) where x is a sync.Map, so we range over the single item and assert the name in our metric
	// definition matches the key we have for the map entry. Should fail if any metrics exist that aren't defined, or if
	// the defined metrics don't exist.
	sink.gauges.Range(func(key, value interface{}) bool {
		name, _ := flattenKey(gaugeDef.Name, gaugeDef.ConstLabels)
		if name != key {
			t.Fatalf("expected my_test_gauge, got #{name}")
		}
		return true
	})
	sink.summaries.Range(func(key, value interface{}) bool {
		name, _ := flattenKey(summaryDef.Name, summaryDef.ConstLabels)
		if name != key {
			t.Fatalf("expected my_test_summary, got #{name}")
		}
		return true
	})
	sink.counters.Range(func(key, value interface{}) bool {
		name, _ := flattenKey(counterDef.Name, counterDef.ConstLabels)
		if name != key {
			t.Fatalf("expected my_test_counter, got #{name}")
		}
		return true
	})

	// Set a bunch of values
	sink.SetGauge(gaugeDef.Name, 42)
	sink.AddSample(summaryDef.Name, 42)
	sink.IncrCounter(counterDef.Name, 1)

	// Prometheus panic should not be propagated
	sink.IncrCounter(counterDef.Name, -1)

	// Test that the expiry behavior works as expected. First pick a time which
	// is after all the actual updates above.
	timeAfterUpdates := time.Now()
	// Buffer the chan to make sure it doesn't block. We expect only 3 metrics to
	// be produced but give some extra room as this will hang the test if we don't
	// have a big enough buffer.
	ch := make(chan prometheus.Metric, 10)

	// Collect the metrics as if it's some time in the future, way beyond the 5
	// second expiry.
	sink.collectAtTime(ch, timeAfterUpdates.Add(10*time.Second))

	// We should see all the metrics desired Expiry behavior
	expectedNum := 3
	for i := 0; i < expectedNum; i++ {
		select {
		case m := <-ch:
			// m is a prometheus.Metric the only thing we can do is Write it to a
			// protobuf type and read from there.
			var pb dto.Metric
			if err := m.Write(&pb); err != nil {
				t.Fatalf("unexpected error reading metric: %s", err)
			}
			desc := m.Desc().String()
			switch {
			case pb.Counter != nil:
				if !strings.Contains(desc, counterDef.Help) {
					t.Fatalf("expected counter to include correct help=%s, but was %s", counterDef.Help, m.Desc().String())
				}
				// Counters should _not_ reset. We could assert not nil too but that
				// would be a bug in prometheus client code so assume it's never nil...
				if *pb.Counter.Value != float64(1) {
					t.Fatalf("expected defined counter to have value 42 after expiring, got %f", *pb.Counter.Value)
				}
			case pb.Gauge != nil:
				if !strings.Contains(desc, gaugeDef.Help) {
					t.Fatalf("expected gauge to include correct help=%s, but was %s", gaugeDef.Help, m.Desc().String())
				}
				// Gauges should _not_ reset. We could assert not nil too but that
				// would be a bug in prometheus client code so assume it's never nil...
				if *pb.Gauge.Value != float64(42) {
					t.Fatalf("expected defined gauge to have value 42 after expiring, got %f", *pb.Gauge.Value)
				}
			case pb.Summary != nil:
				if !strings.Contains(desc, summaryDef.Help) {
					t.Fatalf("expected summary to include correct help=%s, but was %s", summaryDef.Help, m.Desc().String())
				}
				// Summaries should not be reset. Previous behavior here did attempt to
				// reset them by calling Observe(NaN) which results in all values being
				// set to NaN but doesn't actually clear the time window of data
				// predictably so future observations could also end up as NaN until the
				// NaN sample has aged out of the window. Since the summary is already
				// aging out a fixed time window (we fix it a 10 seconds currently for
				// all summaries and it's not affected by Expiration option), there's no
				// point in trying to reset it after "expiry".
				if *pb.Summary.SampleSum != float64(42) {
					t.Fatalf("expected defined summary sum to have value 42 after expiring, got %f", *pb.Summary.SampleSum)
				}
			default:
				t.Fatalf("unexpected metric type %v", pb)
			}
		case <-time.After(100 * time.Millisecond):
			t.Fatalf("Timed out waiting to collect expected metric. Got %d, want %d", i, expectedNum)
		}
	}
}

func MockGetHostname() string {
	return TestHostname
}

func fakeServer(q chan string) *httptest.Server {
	handler := func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(202)
		w.Header().Set("Content-Type", "application/json")
		defer r.Body.Close()
		dec := expfmt.NewDecoder(r.Body, expfmt.FmtProtoDelim)
		m := &dto.MetricFamily{}
		dec.Decode(m)
		expectedm := &dto.MetricFamily{
			Name: proto.String("default_one_two"),
			Help: proto.String("default_one_two"),
			Type: dto.MetricType_GAUGE.Enum(),
			Metric: []*dto.Metric{
				&dto.Metric{
					Label: []*dto.LabelPair{
						&dto.LabelPair{
							Name:  proto.String("host"),
							Value: proto.String(MockGetHostname()),
						},
					},
					Gauge: &dto.Gauge{
						Value: proto.Float64(42),
					},
				},
			},
		}
		if !reflect.DeepEqual(m, expectedm) {
			msg := fmt.Sprintf("Unexpected samples extracted, got: %+v, want: %+v", m, expectedm)
			q <- errors.New(msg).Error()
		} else {
			q <- "ok"
		}
	}

	return httptest.NewServer(http.HandlerFunc(handler))
}

func TestSetGauge(t *testing.T) {
	q := make(chan string)
	server := fakeServer(q)
	defer server.Close()
	u, err := url.Parse(server.URL)
	if err != nil {
		log.Fatal(err)
	}
	host := u.Hostname() + ":" + u.Port()
	sink, err := NewPrometheusPushSink(host, time.Second, "pushtest")
	metricsConf := metrics.DefaultConfig("default")
	metricsConf.HostName = MockGetHostname()
	metricsConf.EnableHostnameLabel = true
	metrics.NewGlobal(metricsConf, sink)
	metrics.SetGauge([]string{"one", "two"}, 42)
	response := <-q
	if response != "ok" {
		t.Fatal(response)
	}
}

func TestSetPrecisionGauge(t *testing.T) {
	q := make(chan string)
	server := fakeServer(q)
	defer server.Close()
	u, err := url.Parse(server.URL)
	if err != nil {
		log.Fatal(err)
	}
	host := u.Hostname() + ":" + u.Port()
	sink, err := NewPrometheusPushSink(host, time.Second, "pushtest")
	metricsConf := metrics.DefaultConfig("default")
	metricsConf.HostName = MockGetHostname()
	metricsConf.EnableHostnameLabel = true
	metrics.NewGlobal(metricsConf, sink)
	metrics.SetPrecisionGauge([]string{"one", "two"}, 42)
	response := <-q
	if response != "ok" {
		t.Fatal(response)
	}
}

func TestDefinitionsWithLabels(t *testing.T) {
	gaugeDef := GaugeDefinition{
		Name: []string{"my", "test", "gauge"},
		Help: "A gauge for testing? How helpful!",
	}
	summaryDef := SummaryDefinition{
		Name: []string{"my", "test", "summary"},
		Help: "A summary for testing? How helpful!",
	}
	counterDef := CounterDefinition{
		Name: []string{"my", "test", "counter"},
		Help: "A counter for testing? How helpful!",
	}

	// PrometheusSink config w/ definitions for each metric type
	cfg := PrometheusOpts{
		Expiration:         5 * time.Second,
		GaugeDefinitions:   append([]GaugeDefinition{}, gaugeDef),
		SummaryDefinitions: append([]SummaryDefinition{}, summaryDef),
		CounterDefinitions: append([]CounterDefinition{}, counterDef),
	}
	sink, err := NewPrometheusSinkFrom(cfg)
	if err != nil {
		t.Fatalf("err =%#v, want nil", err)
	}
	defer prometheus.Unregister(sink)
	if len(sink.help) != 3 {
		t.Fatalf("Expected len(sink.help) to be 3, was %d: %#v", len(sink.help), sink.help)
	}

	sink.SetGaugeWithLabels(gaugeDef.Name, 42.0, []metrics.Label{
		{Name: "version", Value: "some info"},
	})
	sink.gauges.Range(func(key, value interface{}) bool {
		localGauge := *value.(*gauge)
		if !strings.Contains(localGauge.Desc().String(), gaugeDef.Help) {
			t.Fatalf("expected gauge to include correct help=%s, but was %s", gaugeDef.Help, localGauge.Desc().String())
		}
		return true
	})

	sink.AddSampleWithLabels(summaryDef.Name, 42.0, []metrics.Label{
		{Name: "version", Value: "some info"},
	})
	sink.summaries.Range(func(key, value interface{}) bool {
		metric := *value.(*summary)
		if !strings.Contains(metric.Desc().String(), summaryDef.Help) {
			t.Fatalf("expected gauge to include correct help=%s, but was %s", summaryDef.Help, metric.Desc().String())
		}
		return true
	})

	sink.IncrCounterWithLabels(counterDef.Name, 42.0, []metrics.Label{
		{Name: "version", Value: "some info"},
	})
	sink.counters.Range(func(key, value interface{}) bool {
		metric := *value.(*counter)
		if !strings.Contains(metric.Desc().String(), counterDef.Help) {
			t.Fatalf("expected gauge to include correct help=%s, but was %s", counterDef.Help, metric.Desc().String())
		}
		return true
	})

	// Prometheus panic should not be propagated
	sink.IncrCounterWithLabels(counterDef.Name, -1, []metrics.Label{
		{Name: "version", Value: "some info"},
	})
}

func TestMetricSinkInterface(t *testing.T) {
	var ps *PrometheusSink
	_ = metrics.MetricSink(ps)
	var pps *PrometheusPushSink
	_ = metrics.MetricSink(pps)
}

func Test_flattenKey(t *testing.T) {
	testCases := []struct {
		name               string
		inputParts         []string
		inputLabels        []metrics.Label
		expectedOutputKey  string
		expectedOutputHash string
	}{
		{
			name:       "no replacement needed",
			inputParts: []string{"my", "example", "metric"},
			inputLabels: []metrics.Label{
				{Name: "foo", Value: "bar"},
				{Name: "baz", Value: "buz"},
			},
			expectedOutputKey:  "my_example_metric",
			expectedOutputHash: "my_example_metric;foo=bar;baz=buz",
		},
		{
			name:       "key with whitespace",
			inputParts: []string{" my ", " example ", " metric "},
			inputLabels: []metrics.Label{
				{Name: "foo", Value: "bar"},
				{Name: "baz", Value: "buz"},
			},
			expectedOutputKey:  "_my___example___metric_",
			expectedOutputHash: "_my___example___metric_;foo=bar;baz=buz",
		},
		{
			name:       "key with dot",
			inputParts: []string{".my.", ".example.", ".metric."},
			inputLabels: []metrics.Label{
				{Name: "foo", Value: "bar"},
				{Name: "baz", Value: "buz"},
			},
			expectedOutputKey:  "_my___example___metric_",
			expectedOutputHash: "_my___example___metric_;foo=bar;baz=buz",
		},
		{
			name:       "key with dash",
			inputParts: []string{"-my-", "-example-", "-metric-"},
			inputLabels: []metrics.Label{
				{Name: "foo", Value: "bar"},
				{Name: "baz", Value: "buz"},
			},
			expectedOutputKey:  "_my___example___metric_",
			expectedOutputHash: "_my___example___metric_;foo=bar;baz=buz",
		},
		{
			name:       "key with forward slash",
			inputParts: []string{"/my/", "/example/", "/metric/"},
			inputLabels: []metrics.Label{
				{Name: "foo", Value: "bar"},
				{Name: "baz", Value: "buz"},
			},
			expectedOutputKey:  "_my___example___metric_",
			expectedOutputHash: "_my___example___metric_;foo=bar;baz=buz",
		},
		{
			name:       "key with all restricted",
			inputParts: []string{"/my-", ".example ", "metric"},
			inputLabels: []metrics.Label{
				{Name: "foo", Value: "bar"},
				{Name: "baz", Value: "buz"},
			},
			expectedOutputKey:  "_my___example__metric",
			expectedOutputHash: "_my___example__metric;foo=bar;baz=buz",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(b *testing.T) {
			actualKey, actualHash := flattenKey(tc.inputParts, tc.inputLabels)
			if actualKey != tc.expectedOutputKey {
				t.Fatalf("expected key %s, got %s", tc.expectedOutputKey, actualKey)
			}
			if actualHash != tc.expectedOutputHash {
				t.Fatalf("expected hash %s, got %s", tc.expectedOutputHash, actualHash)
			}
		})
	}
}
