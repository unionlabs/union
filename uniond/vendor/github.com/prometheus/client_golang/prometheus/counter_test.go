// Copyright 2014 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package prometheus

import (
	"fmt"
	"math"
	"strings"
	"testing"
	"time"

	dto "github.com/prometheus/client_model/go"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func TestCounterAdd(t *testing.T) {
	now := time.Now()

	counter := NewCounter(CounterOpts{
		Name:        "test",
		Help:        "test help",
		ConstLabels: Labels{"a": "1", "b": "2"},
		now:         func() time.Time { return now },
	}).(*counter)
	counter.Inc()
	if expected, got := 0.0, math.Float64frombits(counter.valBits); expected != got {
		t.Errorf("Expected %f, got %f.", expected, got)
	}
	if expected, got := uint64(1), counter.valInt; expected != got {
		t.Errorf("Expected %d, got %d.", expected, got)
	}
	counter.Add(42)
	if expected, got := 0.0, math.Float64frombits(counter.valBits); expected != got {
		t.Errorf("Expected %f, got %f.", expected, got)
	}
	if expected, got := uint64(43), counter.valInt; expected != got {
		t.Errorf("Expected %d, got %d.", expected, got)
	}

	counter.Add(24.42)
	if expected, got := 24.42, math.Float64frombits(counter.valBits); expected != got {
		t.Errorf("Expected %f, got %f.", expected, got)
	}
	if expected, got := uint64(43), counter.valInt; expected != got {
		t.Errorf("Expected %d, got %d.", expected, got)
	}

	if expected, got := "counter cannot decrease in value", decreaseCounter(counter).Error(); expected != got {
		t.Errorf("Expected error %q, got %q.", expected, got)
	}

	m := &dto.Metric{}
	counter.Write(m)

	expected := &dto.Metric{
		Label: []*dto.LabelPair{
			{Name: proto.String("a"), Value: proto.String("1")},
			{Name: proto.String("b"), Value: proto.String("2")},
		},
		Counter: &dto.Counter{
			Value:            proto.Float64(67.42),
			CreatedTimestamp: timestamppb.New(now),
		},
	}
	if !proto.Equal(expected, m) {
		t.Errorf("expected %q, got %q", expected, m)
	}
}

func decreaseCounter(c *counter) (err error) {
	defer func() {
		if e := recover(); e != nil {
			err = e.(error)
		}
	}()
	c.Add(-1)
	return nil
}

func TestCounterVecGetMetricWithInvalidLabelValues(t *testing.T) {
	testCases := []struct {
		desc   string
		labels Labels
	}{
		{
			desc:   "non utf8 label value",
			labels: Labels{"a": "\xFF"},
		},
		{
			desc:   "not enough label values",
			labels: Labels{},
		},
		{
			desc:   "too many label values",
			labels: Labels{"a": "1", "b": "2"},
		},
	}

	for _, test := range testCases {
		counterVec := NewCounterVec(CounterOpts{
			Name: "test",
		}, []string{"a"})

		labelValues := make([]string, 0, len(test.labels))
		for _, val := range test.labels {
			labelValues = append(labelValues, val)
		}

		expectPanic(t, func() {
			counterVec.WithLabelValues(labelValues...)
		}, fmt.Sprintf("WithLabelValues: expected panic because: %s", test.desc))
		expectPanic(t, func() {
			counterVec.With(test.labels)
		}, fmt.Sprintf("WithLabelValues: expected panic because: %s", test.desc))

		if _, err := counterVec.GetMetricWithLabelValues(labelValues...); err == nil {
			t.Errorf("GetMetricWithLabelValues: expected error because: %s", test.desc)
		}
		if _, err := counterVec.GetMetricWith(test.labels); err == nil {
			t.Errorf("GetMetricWith: expected error because: %s", test.desc)
		}
	}
}

func expectPanic(t *testing.T, op func(), errorMsg string) {
	defer func() {
		if err := recover(); err == nil {
			t.Error(errorMsg)
		}
	}()

	op()
}

func TestCounterAddInf(t *testing.T) {
	now := time.Now()

	counter := NewCounter(CounterOpts{
		Name: "test",
		Help: "test help",
		now:  func() time.Time { return now },
	}).(*counter)

	counter.Inc()
	if expected, got := 0.0, math.Float64frombits(counter.valBits); expected != got {
		t.Errorf("Expected %f, got %f.", expected, got)
	}
	if expected, got := uint64(1), counter.valInt; expected != got {
		t.Errorf("Expected %d, got %d.", expected, got)
	}

	counter.Add(math.Inf(1))
	if expected, got := math.Inf(1), math.Float64frombits(counter.valBits); expected != got {
		t.Errorf("valBits expected %f, got %f.", expected, got)
	}
	if expected, got := uint64(1), counter.valInt; expected != got {
		t.Errorf("valInts expected %d, got %d.", expected, got)
	}

	counter.Inc()
	if expected, got := math.Inf(1), math.Float64frombits(counter.valBits); expected != got {
		t.Errorf("Expected %f, got %f.", expected, got)
	}
	if expected, got := uint64(2), counter.valInt; expected != got {
		t.Errorf("Expected %d, got %d.", expected, got)
	}

	m := &dto.Metric{}
	counter.Write(m)

	expected := &dto.Metric{
		Counter: &dto.Counter{
			Value:            proto.Float64(math.Inf(1)),
			CreatedTimestamp: timestamppb.New(now),
		},
	}

	if !proto.Equal(expected, m) {
		t.Errorf("expected %q, got %q", expected, m)
	}
}

func TestCounterAddLarge(t *testing.T) {
	now := time.Now()

	counter := NewCounter(CounterOpts{
		Name: "test",
		Help: "test help",
		now:  func() time.Time { return now },
	}).(*counter)

	// large overflows the underlying type and should therefore be stored in valBits.
	large := math.Nextafter(float64(math.MaxUint64), 1e20)
	counter.Add(large)
	if expected, got := large, math.Float64frombits(counter.valBits); expected != got {
		t.Errorf("valBits expected %f, got %f.", expected, got)
	}
	if expected, got := uint64(0), counter.valInt; expected != got {
		t.Errorf("valInts expected %d, got %d.", expected, got)
	}

	m := &dto.Metric{}
	counter.Write(m)

	expected := &dto.Metric{
		Counter: &dto.Counter{
			Value:            proto.Float64(large),
			CreatedTimestamp: timestamppb.New(now),
		},
	}

	if !proto.Equal(expected, m) {
		t.Errorf("expected %q, got %q", expected, m)
	}
}

func TestCounterAddSmall(t *testing.T) {
	now := time.Now()

	counter := NewCounter(CounterOpts{
		Name: "test",
		Help: "test help",
		now:  func() time.Time { return now },
	}).(*counter)

	small := 0.000000000001
	counter.Add(small)
	if expected, got := small, math.Float64frombits(counter.valBits); expected != got {
		t.Errorf("valBits expected %f, got %f.", expected, got)
	}
	if expected, got := uint64(0), counter.valInt; expected != got {
		t.Errorf("valInts expected %d, got %d.", expected, got)
	}

	m := &dto.Metric{}
	counter.Write(m)

	expected := &dto.Metric{
		Counter: &dto.Counter{
			Value:            proto.Float64(small),
			CreatedTimestamp: timestamppb.New(now),
		},
	}

	if !proto.Equal(expected, m) {
		t.Errorf("expected %q, got %q", expected, m)
	}
}

func TestCounterExemplar(t *testing.T) {
	now := time.Now()

	counter := NewCounter(CounterOpts{
		Name: "test",
		Help: "test help",
		now:  func() time.Time { return now },
	}).(*counter)

	ts := timestamppb.New(now)
	if err := ts.CheckValid(); err != nil {
		t.Fatal(err)
	}
	expectedExemplar := &dto.Exemplar{
		Label: []*dto.LabelPair{
			{Name: proto.String("foo"), Value: proto.String("bar")},
		},
		Value:     proto.Float64(42),
		Timestamp: ts,
	}

	counter.AddWithExemplar(42, Labels{"foo": "bar"})
	if expected, got := expectedExemplar.String(), counter.exemplar.Load().(*dto.Exemplar).String(); expected != got {
		t.Errorf("expected exemplar %s, got %s.", expected, got)
	}

	addExemplarWithInvalidLabel := func() (err error) {
		defer func() {
			if e := recover(); e != nil {
				err = e.(error)
			}
		}()
		// Should panic because of invalid label name.
		counter.AddWithExemplar(42, Labels{":o)": "smile"})
		return nil
	}
	if addExemplarWithInvalidLabel() == nil {
		t.Error("adding exemplar with invalid label succeeded")
	}

	addExemplarWithOversizedLabels := func() (err error) {
		defer func() {
			if e := recover(); e != nil {
				err = e.(error)
			}
		}()
		// Should panic because of 129 runes.
		counter.AddWithExemplar(42, Labels{
			"abcdefghijklmnopqrstuvwxyz": "26+16 characters",
			"x1234567":                   "8+15 characters",
			"z":                          strings.Repeat("x", 63),
		})
		return nil
	}
	if addExemplarWithOversizedLabels() == nil {
		t.Error("adding exemplar with oversized labels succeeded")
	}
}

func TestCounterVecCreatedTimestampWithDeletes(t *testing.T) {
	now := time.Now()

	counterVec := NewCounterVec(CounterOpts{
		Name: "test",
		Help: "test help",
		now:  func() time.Time { return now },
	}, []string{"label"})

	// First use of "With" should populate CT.
	counterVec.WithLabelValues("1")
	expected := map[string]time.Time{"1": now}

	now = now.Add(1 * time.Hour)
	expectCTsForMetricVecValues(t, counterVec.MetricVec, dto.MetricType_COUNTER, expected)

	// Two more labels at different times.
	counterVec.WithLabelValues("2")
	expected["2"] = now

	now = now.Add(1 * time.Hour)

	counterVec.WithLabelValues("3")
	expected["3"] = now

	now = now.Add(1 * time.Hour)
	expectCTsForMetricVecValues(t, counterVec.MetricVec, dto.MetricType_COUNTER, expected)

	// Recreate metric instance should reset created timestamp to now.
	counterVec.DeleteLabelValues("1")
	counterVec.WithLabelValues("1")
	expected["1"] = now

	now = now.Add(1 * time.Hour)
	expectCTsForMetricVecValues(t, counterVec.MetricVec, dto.MetricType_COUNTER, expected)
}

func expectCTsForMetricVecValues(t testing.TB, vec *MetricVec, typ dto.MetricType, ctsPerLabelValue map[string]time.Time) {
	t.Helper()

	for val, ct := range ctsPerLabelValue {
		var metric dto.Metric
		m, err := vec.GetMetricWithLabelValues(val)
		if err != nil {
			t.Fatal(err)
		}

		if err := m.Write(&metric); err != nil {
			t.Fatal(err)
		}

		var gotTs time.Time
		switch typ {
		case dto.MetricType_COUNTER:
			gotTs = metric.Counter.CreatedTimestamp.AsTime()
		case dto.MetricType_HISTOGRAM:
			gotTs = metric.Histogram.CreatedTimestamp.AsTime()
		case dto.MetricType_SUMMARY:
			gotTs = metric.Summary.CreatedTimestamp.AsTime()
		default:
			t.Fatalf("unknown metric type %v", typ)
		}

		if !gotTs.Equal(ct) {
			t.Errorf("expected created timestamp for %s with label value %q: %s, got %s", typ, val, ct, gotTs)
		}
	}
}
