package metrics

import (
	"io/ioutil"
	"log"
	"reflect"
	"sync/atomic"
	"testing"
	"time"
)

func TestDefaultConfig(t *testing.T) {
	conf := DefaultConfig("service")
	if conf.ServiceName != "service" {
		t.Fatalf("Bad name")
	}
	if conf.HostName == "" {
		t.Fatalf("missing hostname")
	}
	if !conf.EnableHostname || !conf.EnableRuntimeMetrics {
		t.Fatalf("expect true")
	}
	if conf.EnableTypePrefix {
		t.Fatalf("expect false")
	}
	if conf.TimerGranularity != time.Millisecond {
		t.Fatalf("bad granularity")
	}
	if conf.ProfileInterval != time.Second {
		t.Fatalf("bad interval")
	}
}

func Test_GlobalMetrics(t *testing.T) {
	var tests = []struct {
		desc string
		key  []string
		val  float32
		fn   func([]string, float32)
	}{
		{"SetGauge", []string{"test"}, 42, SetGauge},
		{"EmitKey", []string{"test"}, 42, EmitKey},
		{"IncrCounter", []string{"test"}, 42, IncrCounter},
		{"AddSample", []string{"test"}, 42, AddSample},
	}

	for _, tt := range tests {
		t.Run(tt.desc, func(t *testing.T) {
			s := &MockSink{}
			globalMetrics.Store(&Metrics{Config: Config{FilterDefault: true}, sink: s})
			tt.fn(tt.key, tt.val)
			if got, want := s.keys[0], tt.key; !reflect.DeepEqual(got, want) {
				t.Fatalf("got key %s want %s", got, want)
			}
			if got, want := s.vals[0], tt.val; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %v want %v", got, want)
			}
		})
	}
}

func Test_GlobalMetrics_Labels(t *testing.T) {
	labels := []Label{{"a", "b"}}
	var tests = []struct {
		desc   string
		key    []string
		val    float32
		fn     func([]string, float32, []Label)
		labels []Label
	}{
		{"SetGaugeWithLabels", []string{"test"}, 42, SetGaugeWithLabels, labels},
		{"IncrCounterWithLabels", []string{"test"}, 42, IncrCounterWithLabels, labels},
		{"AddSampleWithLabels", []string{"test"}, 42, AddSampleWithLabels, labels},
	}

	for _, tt := range tests {
		t.Run(tt.desc, func(t *testing.T) {
			s := &MockSink{}
			globalMetrics.Store(&Metrics{Config: Config{FilterDefault: true}, sink: s})
			tt.fn(tt.key, tt.val, tt.labels)
			if got, want := s.keys[0], tt.key; !reflect.DeepEqual(got, want) {
				t.Fatalf("got key %s want %s", got, want)
			}
			if got, want := s.vals[0], tt.val; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %v want %v", got, want)
			}
			if got, want := s.labels[0], tt.labels; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %s want %s", got, want)
			}
		})
	}
}

func Test_GlobalPrecisionMetrics_Labels(t *testing.T) {
	labels := []Label{{"a", "b"}}
	var tests = []struct {
		desc   string
		key    []string
		val    float64
		fn     func([]string, float64, []Label)
		labels []Label
	}{
		{"SetPrecisionGaugeWithLabels", []string{"test"}, 42, SetPrecisionGaugeWithLabels, labels},
	}

	for _, tt := range tests {
		t.Run(tt.desc, func(t *testing.T) {
			s := &MockSink{}
			globalMetrics.Store(&Metrics{Config: Config{FilterDefault: true}, sink: s})
			tt.fn(tt.key, tt.val, tt.labels)
			if got, want := s.keys[0], tt.key; !reflect.DeepEqual(got, want) {
				t.Fatalf("got key %s want %s", got, want)
			}
			if got, want := s.precisionVals[0], tt.val; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %v want %v", got, want)
			}
			if got, want := s.labels[0], tt.labels; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %s want %s", got, want)
			}
		})
	}
}

func Test_GlobalMetrics_DefaultLabels(t *testing.T) {
	config := Config{
		HostName:            "host1",
		ServiceName:         "redis",
		EnableHostnameLabel: true,
		EnableServiceLabel:  true,
		FilterDefault:       true,
	}
	labels := []Label{
		{"host", config.HostName},
		{"service", config.ServiceName},
	}
	var tests = []struct {
		desc   string
		key    []string
		val    float32
		fn     func([]string, float32, []Label)
		labels []Label
	}{
		{"SetGaugeWithLabels", []string{"test"}, 42, SetGaugeWithLabels, labels},
		{"IncrCounterWithLabels", []string{"test"}, 42, IncrCounterWithLabels, labels},
		{"AddSampleWithLabels", []string{"test"}, 42, AddSampleWithLabels, labels},
	}

	for _, tt := range tests {
		t.Run(tt.desc, func(t *testing.T) {
			s := &MockSink{}
			globalMetrics.Store(&Metrics{Config: config, sink: s})
			tt.fn(tt.key, tt.val, nil)
			if got, want := s.keys[0], tt.key; !reflect.DeepEqual(got, want) {
				t.Fatalf("got key %s want %s", got, want)
			}
			if got, want := s.vals[0], tt.val; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %v want %v", got, want)
			}
			if got, want := s.labels[0], tt.labels; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %s want %s", got, want)
			}
		})
	}
}

func Test_GlobalPrecisionMetrics_DefaultLabels(t *testing.T) {
	config := Config{
		HostName:            "host1",
		ServiceName:         "redis",
		EnableHostnameLabel: true,
		EnableServiceLabel:  true,
		FilterDefault:       true,
	}
	labels := []Label{
		{"host", config.HostName},
		{"service", config.ServiceName},
	}
	var tests = []struct {
		desc   string
		key    []string
		val    float64
		fn     func([]string, float64, []Label)
		labels []Label
	}{
		{"SetGaugeWithLabels", []string{"test"}, 42, SetPrecisionGaugeWithLabels, labels},
	}

	for _, tt := range tests {
		t.Run(tt.desc, func(t *testing.T) {
			s := &MockSink{}
			globalMetrics.Store(&Metrics{Config: config, sink: s})
			tt.fn(tt.key, tt.val, nil)
			if got, want := s.keys[0], tt.key; !reflect.DeepEqual(got, want) {
				t.Fatalf("got key %s want %s", got, want)
			}
			if got, want := s.precisionVals[0], tt.val; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %v want %v", got, want)
			}
			if got, want := s.labels[0], tt.labels; !reflect.DeepEqual(got, want) {
				t.Fatalf("got val %s want %s", got, want)
			}
		})
	}
}

func Test_GlobalMetrics_MeasureSince(t *testing.T) {
	s := &MockSink{}
	m := &Metrics{sink: s, Config: Config{TimerGranularity: time.Millisecond, FilterDefault: true}}
	globalMetrics.Store(m)

	k := []string{"test"}
	now := time.Now()
	MeasureSince(k, now)

	if !reflect.DeepEqual(s.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if s.vals[0] > 0.1 {
		t.Fatalf("val too large %v", s.vals[0])
	}

	labels := []Label{{"a", "b"}}
	MeasureSinceWithLabels(k, now, labels)
	if got, want := s.keys[1], k; !reflect.DeepEqual(got, want) {
		t.Fatalf("got key %s want %s", got, want)
	}
	if s.vals[1] > 0.1 {
		t.Fatalf("val too large %v", s.vals[0])
	}
	if got, want := s.labels[1], labels; !reflect.DeepEqual(got, want) {
		t.Fatalf("got val %s want %s", got, want)
	}
}

func Test_GlobalMetrics_UpdateFilter(t *testing.T) {
	globalMetrics.Store(&Metrics{Config: Config{
		AllowedPrefixes: []string{"a"},
		BlockedPrefixes: []string{"b"},
		AllowedLabels:   []string{"1"},
		BlockedLabels:   []string{"2"},
	}})
	UpdateFilterAndLabels([]string{"c"}, []string{"d"}, []string{"3"}, []string{"4"})

	m := globalMetrics.Load().(*Metrics)
	if m.AllowedPrefixes[0] != "c" {
		t.Fatalf("bad: %v", m.AllowedPrefixes)
	}
	if m.BlockedPrefixes[0] != "d" {
		t.Fatalf("bad: %v", m.BlockedPrefixes)
	}
	if m.AllowedLabels[0] != "3" {
		t.Fatalf("bad: %v", m.AllowedPrefixes)
	}
	if m.BlockedLabels[0] != "4" {
		t.Fatalf("bad: %v", m.AllowedPrefixes)
	}
	if _, ok := m.allowedLabels["3"]; !ok {
		t.Fatalf("bad: %v", m.allowedLabels)
	}
	if _, ok := m.blockedLabels["4"]; !ok {
		t.Fatalf("bad: %v", m.blockedLabels)
	}
}

func Test_GlobalMetrics_Shutdown(t *testing.T) {
	s := &MockSink{}
	m := &Metrics{sink: s}
	globalMetrics.Store(m)

	Shutdown()

	loaded := globalMetrics.Load()
	metrics, ok := loaded.(*Metrics)
	if !ok {
		t.Fatalf("Expected globalMetrics to contain a Metrics pointer, but found: %v", loaded)
	}
	if metrics == m {
		t.Errorf("Calling shutdown should have replaced the Metrics struct stored in globalMetrics")
	}
	if !s.shutdown {
		t.Errorf("Expected Shutdown to have been called on MockSink")
	}
}

// Benchmark_GlobalMetrics_Direct/direct-8         	 5000000	       278 ns/op
// Benchmark_GlobalMetrics_Direct/atomic.Value-8   	 5000000	       235 ns/op
func Benchmark_GlobalMetrics_Direct(b *testing.B) {
	log.SetOutput(ioutil.Discard)
	s := &MockSink{}
	m := &Metrics{sink: s}
	var v atomic.Value
	v.Store(m)
	k := []string{"test"}
	b.Run("direct", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			m.IncrCounter(k, 1)
		}
	})
	b.Run("atomic.Value", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			v.Load().(*Metrics).IncrCounter(k, 1)
		}
	})
	// do something with m so that the compiler does not optimize this away
	b.Logf("%d", m.lastNumGC)
}
