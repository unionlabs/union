package metrics

import (
	"reflect"
	"runtime"
	"testing"
	"time"
)

func mockMetric() (*MockSink, *Metrics) {
	m := &MockSink{}
	met := &Metrics{Config: Config{FilterDefault: true}, sink: m}
	return m, met
}

func TestMetrics_SetGauge(t *testing.T) {
	m, met := mockMetric()
	met.SetGauge([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	labels := []Label{{"a", "b"}}
	met.SetGaugeWithLabels([]string{"key"}, float32(1), labels)
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}
	if !reflect.DeepEqual(m.labels[0], labels) {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.HostName = "test"
	met.EnableHostname = true
	met.SetGauge([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "test" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.EnableTypePrefix = true
	met.SetGauge([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "gauge" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.ServiceName = "service"
	met.SetGauge([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "service" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}
}

func TestMetrics_SetPrecisionGauge(t *testing.T) {
	m, met := mockMetric()
	met.SetPrecisionGauge([]string{"key"}, float64(1))
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.precisionVals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	labels := []Label{{"a", "b"}}
	met.SetPrecisionGaugeWithLabels([]string{"key"}, float64(1), labels)
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.precisionVals[0] != 1 {
		t.Fatalf("")
	}
	if !reflect.DeepEqual(m.labels[0], labels) {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.HostName = "test"
	met.EnableHostname = true
	met.SetPrecisionGauge([]string{"key"}, float64(1))
	if m.getKeys()[0][0] != "test" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.precisionVals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.EnableTypePrefix = true
	met.SetPrecisionGauge([]string{"key"}, float64(1))
	if m.getKeys()[0][0] != "gauge" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.precisionVals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.ServiceName = "service"
	met.SetPrecisionGauge([]string{"key"}, float64(1))
	if m.getKeys()[0][0] != "service" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.precisionVals[0] != 1 {
		t.Fatalf("")
	}
}

func TestMetrics_EmitKey(t *testing.T) {
	m, met := mockMetric()
	met.EmitKey([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.EnableTypePrefix = true
	met.EmitKey([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "kv" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.ServiceName = "service"
	met.EmitKey([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "service" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}
}

func TestMetrics_IncrCounter(t *testing.T) {
	m, met := mockMetric()
	met.IncrCounter([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	labels := []Label{{"a", "b"}}
	met.IncrCounterWithLabels([]string{"key"}, float32(1), labels)
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}
	if !reflect.DeepEqual(m.labels[0], labels) {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.EnableTypePrefix = true
	met.IncrCounter([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "counter" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.ServiceName = "service"
	met.IncrCounter([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "service" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}
}

func TestMetrics_AddSample(t *testing.T) {
	m, met := mockMetric()
	met.AddSample([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	labels := []Label{{"a", "b"}}
	met.AddSampleWithLabels([]string{"key"}, float32(1), labels)
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}
	if !reflect.DeepEqual(m.labels[0], labels) {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.EnableTypePrefix = true
	met.AddSample([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "sample" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.ServiceName = "service"
	met.AddSample([]string{"key"}, float32(1))
	if m.getKeys()[0][0] != "service" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] != 1 {
		t.Fatalf("")
	}
}

func TestMetrics_MeasureSince(t *testing.T) {
	m, met := mockMetric()
	met.TimerGranularity = time.Millisecond
	n := time.Now()
	met.MeasureSince([]string{"key"}, n)
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] > 0.1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.TimerGranularity = time.Millisecond
	labels := []Label{{"a", "b"}}
	met.MeasureSinceWithLabels([]string{"key"}, n, labels)
	if m.getKeys()[0][0] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] > 0.1 {
		t.Fatalf("")
	}
	if !reflect.DeepEqual(m.labels[0], labels) {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.TimerGranularity = time.Millisecond
	met.EnableTypePrefix = true
	met.MeasureSince([]string{"key"}, n)
	if m.getKeys()[0][0] != "timer" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] > 0.1 {
		t.Fatalf("")
	}

	m, met = mockMetric()
	met.TimerGranularity = time.Millisecond
	met.ServiceName = "service"
	met.MeasureSince([]string{"key"}, n)
	if m.getKeys()[0][0] != "service" || m.getKeys()[0][1] != "key" {
		t.Fatalf("")
	}
	if m.vals[0] > 0.1 {
		t.Fatalf("")
	}
}

func TestMetrics_EmitRuntimeStats(t *testing.T) {
	runtime.GC()
	m, met := mockMetric()
	met.EmitRuntimeStats()

	if m.getKeys()[0][0] != "runtime" || m.getKeys()[0][1] != "num_goroutines" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[0] <= 1 {
		t.Fatalf("bad val: %v", m.vals)
	}

	if m.getKeys()[1][0] != "runtime" || m.getKeys()[1][1] != "alloc_bytes" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[1] <= 40000 {
		t.Fatalf("bad val: %v", m.vals)
	}

	if m.getKeys()[2][0] != "runtime" || m.getKeys()[2][1] != "sys_bytes" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[2] <= 100000 {
		t.Fatalf("bad val: %v", m.vals)
	}

	if m.getKeys()[3][0] != "runtime" || m.getKeys()[3][1] != "malloc_count" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[3] <= 100 {
		t.Fatalf("bad val: %v", m.vals)
	}

	if m.getKeys()[4][0] != "runtime" || m.getKeys()[4][1] != "free_count" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[4] <= 100 {
		t.Fatalf("bad val: %v", m.vals)
	}

	if m.getKeys()[5][0] != "runtime" || m.getKeys()[5][1] != "heap_objects" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[5] <= 100 {
		t.Fatalf("bad val: %v", m.vals)
	}

	if m.getKeys()[6][0] != "runtime" || m.getKeys()[6][1] != "total_gc_pause_ns" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[6] <= 100 {
		t.Fatalf("bad val: %v\nkeys: %v", m.vals, m.getKeys())
	}

	if m.getKeys()[7][0] != "runtime" || m.getKeys()[7][1] != "total_gc_runs" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[7] < 1 {
		t.Fatalf("bad val: %v", m.vals)
	}

	if m.getKeys()[8][0] != "runtime" || m.getKeys()[8][1] != "gc_pause_ns" {
		t.Fatalf("bad key %v", m.getKeys())
	}
	if m.vals[8] <= 1000 {
		t.Fatalf("bad val: %v", m.vals)
	}
}

func TestInsert(t *testing.T) {
	k := []string{"hi", "bob"}
	exp := []string{"hi", "there", "bob"}
	out := insert(1, "there", k)
	if !reflect.DeepEqual(exp, out) {
		t.Fatalf("bad insert %v %v", exp, out)
	}
}

func TestMetrics_Filter_Blacklist(t *testing.T) {
	m := &MockSink{}
	conf := DefaultConfig("")
	conf.AllowedPrefixes = []string{"service", "debug.thing"}
	conf.BlockedPrefixes = []string{"debug"}
	conf.EnableHostname = false
	met, err := New(conf, m)
	if err != nil {
		t.Fatal(err)
	}

	// Allowed by default
	key := []string{"thing"}
	met.SetGauge(key, 1)
	if !reflect.DeepEqual(m.getKeys()[0], key) {
		t.Fatalf("key doesn't exist %v, %v", m.getKeys()[0], key)
	}
	if m.vals[0] != 1 {
		t.Fatalf("bad val: %v", m.vals[0])
	}

	// Allowed by filter
	key = []string{"service", "thing"}
	met.SetGauge(key, 2)
	if !reflect.DeepEqual(m.getKeys()[1], key) {
		t.Fatalf("key doesn't exist")
	}
	if m.vals[1] != 2 {
		t.Fatalf("bad val: %v", m.vals[1])
	}

	// Allowed by filter, subtree of a blocked entry
	key = []string{"debug", "thing"}
	met.SetGauge(key, 3)
	if !reflect.DeepEqual(m.getKeys()[2], key) {
		t.Fatalf("key doesn't exist")
	}
	if m.vals[2] != 3 {
		t.Fatalf("bad val: %v", m.vals[2])
	}

	// Blocked by filter
	key = []string{"debug", "other-thing"}
	met.SetGauge(key, 4)
	if len(m.getKeys()) != 3 {
		t.Fatalf("key shouldn't exist")
	}
}

func HasElem(s interface{}, elem interface{}) bool {
	arrV := reflect.ValueOf(s)

	if arrV.Kind() == reflect.Slice {
		for i := 0; i < arrV.Len(); i++ {
			if arrV.Index(i).Interface() == elem {
				return true
			}
		}
	}

	return false
}

func TestMetrics_Filter_Whitelist(t *testing.T) {
	m := &MockSink{}
	conf := DefaultConfig("")
	conf.AllowedPrefixes = []string{"service", "debug.thing"}
	conf.BlockedPrefixes = []string{"debug"}
	conf.FilterDefault = false
	conf.EnableHostname = false
	conf.BlockedLabels = []string{"bad_label"}
	met, err := New(conf, m)
	if err != nil {
		t.Fatal(err)
	}

	// Blocked by default
	key := []string{"thing"}
	met.SetGauge(key, 1)
	if len(m.getKeys()) != 0 {
		t.Fatalf("key should not exist")
	}

	// Allowed by filter
	key = []string{"service", "thing"}
	met.SetGauge(key, 2)
	if !reflect.DeepEqual(m.getKeys()[0], key) {
		t.Fatalf("key doesn't exist")
	}
	if m.vals[0] != 2 {
		t.Fatalf("bad val: %v", m.vals[0])
	}

	// Allowed by filter, subtree of a blocked entry
	key = []string{"debug", "thing"}
	met.SetGauge(key, 3)
	if !reflect.DeepEqual(m.getKeys()[1], key) {
		t.Fatalf("key doesn't exist")
	}
	if m.vals[1] != 3 {
		t.Fatalf("bad val: %v", m.vals[1])
	}

	// Blocked by filter
	key = []string{"debug", "other-thing"}
	met.SetGauge(key, 4)
	if len(m.getKeys()) != 2 {
		t.Fatalf("key shouldn't exist")
	}
	// Test blacklisting of labels
	key = []string{"debug", "thing"}
	goodLabel := Label{Name: "good", Value: "should be present"}
	badLabel := Label{Name: "bad_label", Value: "should not be there"}
	labels := []Label{badLabel, goodLabel}
	met.SetGaugeWithLabels(key, 3, labels)
	if !reflect.DeepEqual(m.getKeys()[1], key) {
		t.Fatalf("key doesn't exist")
	}
	if m.vals[2] != 3 {
		t.Fatalf("bad val: %v", m.vals[1])
	}
	if HasElem(m.labels[2], badLabel) {
		t.Fatalf("bad_label should not be present in %v", m.labels[2])
	}
	if !HasElem(m.labels[2], goodLabel) {
		t.Fatalf("good label is not present in %v", m.labels[2])
	}
}

func TestMetrics_Filter_Labels_Whitelist(t *testing.T) {
	m := &MockSink{}
	conf := DefaultConfig("")
	conf.AllowedPrefixes = []string{"service", "debug.thing"}
	conf.BlockedPrefixes = []string{"debug"}
	conf.FilterDefault = false
	conf.EnableHostname = false
	conf.AllowedLabels = []string{"good_label"}
	conf.BlockedLabels = []string{"bad_label"}
	met, err := New(conf, m)
	if err != nil {
		t.Fatal(err)
	}

	// Blocked by default
	key := []string{"thing"}
	key = []string{"debug", "thing"}
	goodLabel := Label{Name: "good_label", Value: "should be present"}
	notReallyGoodLabel := Label{Name: "not_really_good_label", Value: "not whitelisted, but not blacklisted"}
	badLabel := Label{Name: "bad_label", Value: "should not be there"}
	labels := []Label{badLabel, notReallyGoodLabel, goodLabel}
	met.SetGaugeWithLabels(key, 1, labels)

	if HasElem(m.labels[0], badLabel) {
		t.Fatalf("bad_label should not be present in %v", m.labels[0])
	}
	if HasElem(m.labels[0], notReallyGoodLabel) {
		t.Fatalf("not_really_good_label should not be present in %v", m.labels[0])
	}
	if !HasElem(m.labels[0], goodLabel) {
		t.Fatalf("good label is not present in %v", m.labels[0])
	}

	conf.AllowedLabels = nil
	met.UpdateFilterAndLabels(conf.AllowedPrefixes, conf.BlockedLabels, conf.AllowedLabels, conf.BlockedLabels)
	met.SetGaugeWithLabels(key, 1, labels)

	if HasElem(m.labels[1], badLabel) {
		t.Fatalf("bad_label should not be present in %v", m.labels[1])
	}
	// Since no whitelist, not_really_good_label should be there
	if !HasElem(m.labels[1], notReallyGoodLabel) {
		t.Fatalf("not_really_good_label is not present in %v", m.labels[1])
	}
	if !HasElem(m.labels[1], goodLabel) {
		t.Fatalf("good label is not present in %v", m.labels[1])
	}
}

func TestMetrics_Filter_Labels_ModifyArgs(t *testing.T) {
	m := &MockSink{}
	conf := DefaultConfig("")
	conf.FilterDefault = false
	conf.EnableHostname = false
	conf.AllowedLabels = []string{"keep"}
	conf.BlockedLabels = []string{"delete"}
	met, err := New(conf, m)
	if err != nil {
		t.Fatal(err)
	}

	// Blocked by default
	key := []string{"thing"}
	key = []string{"debug", "thing"}
	goodLabel := Label{Name: "keep", Value: "should be kept"}
	badLabel := Label{Name: "delete", Value: "should be deleted"}
	argLabels := []Label{badLabel, goodLabel, badLabel, goodLabel, badLabel, goodLabel, badLabel}
	origLabels := append([]Label{}, argLabels...)
	met.SetGaugeWithLabels(key, 1, argLabels)

	if !reflect.DeepEqual(argLabels, origLabels) {
		t.Fatalf("SetGaugeWithLabels modified the input argument")
	}
}
