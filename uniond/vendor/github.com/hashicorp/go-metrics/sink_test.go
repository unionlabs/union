package metrics

import (
	"reflect"
	"strings"
	"sync"
	"testing"
)

type MockSink struct {
	lock sync.Mutex

	shutdown      bool
	keys          [][]string
	vals          []float32
	precisionVals []float64
	labels        [][]Label
}

func (m *MockSink) getKeys() [][]string {
	m.lock.Lock()
	defer m.lock.Unlock()

	return m.keys
}

func (m *MockSink) SetGauge(key []string, val float32) {
	m.SetGaugeWithLabels(key, val, nil)
}
func (m *MockSink) SetGaugeWithLabels(key []string, val float32, labels []Label) {
	m.lock.Lock()
	defer m.lock.Unlock()

	m.keys = append(m.keys, key)
	m.vals = append(m.vals, val)
	m.labels = append(m.labels, labels)
}
func (m *MockSink) SetPrecisionGauge(key []string, val float64) {
	m.SetPrecisionGaugeWithLabels(key, val, nil)
}
func (m *MockSink) SetPrecisionGaugeWithLabels(key []string, val float64, labels []Label) {
	m.lock.Lock()
	defer m.lock.Unlock()

	m.keys = append(m.keys, key)
	m.precisionVals = append(m.precisionVals, val)
	m.labels = append(m.labels, labels)
}
func (m *MockSink) EmitKey(key []string, val float32) {
	m.lock.Lock()
	defer m.lock.Unlock()

	m.keys = append(m.keys, key)
	m.vals = append(m.vals, val)
	m.labels = append(m.labels, nil)
}
func (m *MockSink) IncrCounter(key []string, val float32) {
	m.IncrCounterWithLabels(key, val, nil)
}
func (m *MockSink) IncrCounterWithLabels(key []string, val float32, labels []Label) {
	m.lock.Lock()
	defer m.lock.Unlock()

	m.keys = append(m.keys, key)
	m.vals = append(m.vals, val)
	m.labels = append(m.labels, labels)
}
func (m *MockSink) AddSample(key []string, val float32) {
	m.AddSampleWithLabels(key, val, nil)
}
func (m *MockSink) AddSampleWithLabels(key []string, val float32, labels []Label) {
	m.lock.Lock()
	defer m.lock.Unlock()

	m.keys = append(m.keys, key)
	m.vals = append(m.vals, val)
	m.labels = append(m.labels, labels)
}
func (m *MockSink) Shutdown() {
	m.lock.Lock()
	defer m.lock.Unlock()

	m.shutdown = true
}

func TestFanoutSink_Gauge(t *testing.T) {
	m1 := &MockSink{}
	m2 := &MockSink{}
	fh := &FanoutSink{m1, m2}

	k := []string{"test"}
	v := float32(42.0)
	fh.SetGauge(k, v)

	if !reflect.DeepEqual(m1.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m2.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m1.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m2.vals[0], v) {
		t.Fatalf("val not equal")
	}
}

func TestFanoutSink_Gauge_Labels(t *testing.T) {
	m1 := &MockSink{}
	m2 := &MockSink{}
	fh := &FanoutSink{m1, m2}

	k := []string{"test"}
	v := float32(42.0)
	l := []Label{{"a", "b"}}
	fh.SetGaugeWithLabels(k, v, l)

	if !reflect.DeepEqual(m1.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m2.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m1.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m2.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m1.labels[0], l) {
		t.Fatalf("labels not equal")
	}
	if !reflect.DeepEqual(m2.labels[0], l) {
		t.Fatalf("labels not equal")
	}
}

func TestFanoutSink_Key(t *testing.T) {
	m1 := &MockSink{}
	m2 := &MockSink{}
	fh := &FanoutSink{m1, m2}

	k := []string{"test"}
	v := float32(42.0)
	fh.EmitKey(k, v)

	if !reflect.DeepEqual(m1.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m2.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m1.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m2.vals[0], v) {
		t.Fatalf("val not equal")
	}
}

func TestFanoutSink_Counter(t *testing.T) {
	m1 := &MockSink{}
	m2 := &MockSink{}
	fh := &FanoutSink{m1, m2}

	k := []string{"test"}
	v := float32(42.0)
	fh.IncrCounter(k, v)

	if !reflect.DeepEqual(m1.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m2.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m1.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m2.vals[0], v) {
		t.Fatalf("val not equal")
	}
}

func TestFanoutSink_Counter_Labels(t *testing.T) {
	m1 := &MockSink{}
	m2 := &MockSink{}
	fh := &FanoutSink{m1, m2}

	k := []string{"test"}
	v := float32(42.0)
	l := []Label{{"a", "b"}}
	fh.IncrCounterWithLabels(k, v, l)

	if !reflect.DeepEqual(m1.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m2.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m1.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m2.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m1.labels[0], l) {
		t.Fatalf("labels not equal")
	}
	if !reflect.DeepEqual(m2.labels[0], l) {
		t.Fatalf("labels not equal")
	}
}

func TestFanoutSink_Sample(t *testing.T) {
	m1 := &MockSink{}
	m2 := &MockSink{}
	fh := &FanoutSink{m1, m2}

	k := []string{"test"}
	v := float32(42.0)
	fh.AddSample(k, v)

	if !reflect.DeepEqual(m1.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m2.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m1.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m2.vals[0], v) {
		t.Fatalf("val not equal")
	}
}

func TestFanoutSink_Sample_Labels(t *testing.T) {
	m1 := &MockSink{}
	m2 := &MockSink{}
	fh := &FanoutSink{m1, m2}

	k := []string{"test"}
	v := float32(42.0)
	l := []Label{{"a", "b"}}
	fh.AddSampleWithLabels(k, v, l)

	if !reflect.DeepEqual(m1.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m2.keys[0], k) {
		t.Fatalf("key not equal")
	}
	if !reflect.DeepEqual(m1.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m2.vals[0], v) {
		t.Fatalf("val not equal")
	}
	if !reflect.DeepEqual(m1.labels[0], l) {
		t.Fatalf("labels not equal")
	}
	if !reflect.DeepEqual(m2.labels[0], l) {
		t.Fatalf("labels not equal")
	}
}

func TestNewMetricSinkFromURL(t *testing.T) {
	for _, tc := range []struct {
		desc      string
		input     string
		expect    reflect.Type
		expectErr string
	}{
		{
			desc:   "statsd scheme yields a StatsdSink",
			input:  "statsd://someserver:123",
			expect: reflect.TypeOf(&StatsdSink{}),
		},
		{
			desc:   "statsite scheme yields a StatsiteSink",
			input:  "statsite://someserver:123",
			expect: reflect.TypeOf(&StatsiteSink{}),
		},
		{
			desc:   "inmem scheme yields an InmemSink",
			input:  "inmem://?interval=30s&retain=30s",
			expect: reflect.TypeOf(&InmemSink{}),
		},
		{
			desc:      "unknown scheme yields an error",
			input:     "notasink://whatever",
			expectErr: "unrecognized sink name: \"notasink\"",
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			ms, err := NewMetricSinkFromURL(tc.input)
			if tc.expectErr != "" {
				if !strings.Contains(err.Error(), tc.expectErr) {
					t.Fatalf("expected err: %q to contain: %q", err, tc.expectErr)
				}
			} else {
				if err != nil {
					t.Fatalf("unexpected err: %s", err)
				}
				got := reflect.TypeOf(ms)
				if got != tc.expect {
					t.Fatalf("expected return type to be %v, got: %v", tc.expect, got)
				}
			}
		})
	}
}
