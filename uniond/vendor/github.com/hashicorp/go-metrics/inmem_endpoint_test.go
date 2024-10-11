package metrics

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/pascaldekloe/goe/verify"
)

func TestDisplayMetrics(t *testing.T) {
	interval := 10 * time.Millisecond
	inm := NewInmemSink(interval, 50*time.Millisecond)

	// Add data points
	inm.SetGauge([]string{"foo", "bar"}, 42)
	inm.SetGaugeWithLabels([]string{"foo", "bar"}, 23, []Label{{"a", "b"}})
	inm.EmitKey([]string{"foo", "bar"}, 42)
	inm.IncrCounter([]string{"foo", "bar"}, 20)
	inm.IncrCounter([]string{"foo", "bar"}, 22)
	inm.IncrCounterWithLabels([]string{"foo", "bar"}, 20, []Label{{"a", "b"}})
	inm.IncrCounterWithLabels([]string{"foo", "bar"}, 40, []Label{{"a", "b"}})
	inm.AddSample([]string{"foo", "bar"}, 20)
	inm.AddSample([]string{"foo", "bar"}, 24)
	inm.AddSampleWithLabels([]string{"foo", "bar"}, 23, []Label{{"a", "b"}})
	inm.AddSampleWithLabels([]string{"foo", "bar"}, 33, []Label{{"a", "b"}})

	data := inm.Data()
	if len(data) != 1 {
		t.Fatalf("bad: %v", data)
	}

	expected := MetricsSummary{
		Timestamp: data[0].Interval.Round(time.Second).UTC().String(),
		Gauges: []GaugeValue{
			{
				Name:          "foo.bar",
				Hash:          "foo.bar",
				Value:         float32(42),
				DisplayLabels: map[string]string{},
			},
			{
				Name:          "foo.bar",
				Hash:          "foo.bar;a=b",
				Value:         float32(23),
				DisplayLabels: map[string]string{"a": "b"},
			},
		},
		Points: []PointValue{
			{
				Name:   "foo.bar",
				Points: []float32{42},
			},
		},
		Counters: []SampledValue{
			{
				Name: "foo.bar",
				Hash: "foo.bar",
				AggregateSample: &AggregateSample{
					Count: 2,
					Min:   20,
					Max:   22,
					Sum:   42,
					SumSq: 884,
					Rate:  4200,
				},
				Mean:   21,
				Stddev: 1.4142135623730951,
			},
			{
				Name: "foo.bar",
				Hash: "foo.bar;a=b",
				AggregateSample: &AggregateSample{
					Count: 2,
					Min:   20,
					Max:   40,
					Sum:   60,
					SumSq: 2000,
					Rate:  6000,
				},
				Mean:          30,
				Stddev:        14.142135623730951,
				DisplayLabels: map[string]string{"a": "b"},
			},
		},
		Samples: []SampledValue{
			{
				Name: "foo.bar",
				Hash: "foo.bar",
				AggregateSample: &AggregateSample{
					Count: 2,
					Min:   20,
					Max:   24,
					Sum:   44,
					SumSq: 976,
					Rate:  4400,
				},
				Mean:   22,
				Stddev: 2.8284271247461903,
			},
			{
				Name: "foo.bar",
				Hash: "foo.bar;a=b",
				AggregateSample: &AggregateSample{
					Count: 2,
					Min:   23,
					Max:   33,
					Sum:   56,
					SumSq: 1618,
					Rate:  5600,
				},
				Mean:          28,
				Stddev:        7.0710678118654755,
				DisplayLabels: map[string]string{"a": "b"},
			},
		},
	}

	raw, err := inm.DisplayMetrics(nil, nil)
	if err != nil {
		t.Fatalf("err: %v", err)
	}
	result := raw.(MetricsSummary)

	// Ignore the LastUpdated field, we don't export that anyway
	for i, got := range result.Counters {
		expected.Counters[i].LastUpdated = got.LastUpdated
	}
	for i, got := range result.Samples {
		expected.Samples[i].LastUpdated = got.LastUpdated
	}

	verify.Values(t, "all", result, expected)
}

func TestDisplayMetrics_RaceSetGauge(t *testing.T) {
	interval := 200 * time.Millisecond
	inm := NewInmemSink(interval, 10*interval)
	result := make(chan float32)

	go func() {
		for {
			time.Sleep(150 * time.Millisecond)
			inm.SetGauge([]string{"foo", "bar"}, float32(42))
		}
	}()

	go func() {
		start := time.Now()
		var summary MetricsSummary
		// test for twenty intervals
		for time.Now().Sub(start) < 20*interval {
			time.Sleep(100 * time.Millisecond)
			raw, _ := inm.DisplayMetrics(nil, nil)
			summary = raw.(MetricsSummary)
		}
		// save result
		for _, g := range summary.Gauges {
			if g.Name == "foo.bar" {
				result <- g.Value
			}
		}
		close(result)
	}()

	got := <-result
	verify.Values(t, "all", got, float32(42))
}

func TestDisplayMetrics_RaceAddSample(t *testing.T) {
	interval := 200 * time.Millisecond
	inm := NewInmemSink(interval, 10*interval)
	result := make(chan float32)

	go func() {
		for {
			time.Sleep(75 * time.Millisecond)
			inm.AddSample([]string{"foo", "bar"}, float32(0.0))
		}
	}()

	go func() {
		start := time.Now()
		var summary MetricsSummary
		// test for twenty intervals
		for time.Now().Sub(start) < 20*interval {
			time.Sleep(100 * time.Millisecond)
			raw, _ := inm.DisplayMetrics(nil, nil)
			summary = raw.(MetricsSummary)
		}
		// save result
		for _, g := range summary.Gauges {
			if g.Name == "foo.bar" {
				result <- g.Value
			}
		}
		close(result)
	}()

	got := <-result
	verify.Values(t, "all", got, float32(0.0))
}

func TestDisplayMetrics_RaceIncrCounter(t *testing.T) {
	interval := 200 * time.Millisecond
	inm := NewInmemSink(interval, 10*interval)
	result := make(chan float32)

	go func() {
		for {
			time.Sleep(75 * time.Millisecond)
			inm.IncrCounter([]string{"foo", "bar"}, float32(0.0))
		}
	}()

	go func() {
		start := time.Now()
		var summary MetricsSummary
		// test for twenty intervals
		for time.Now().Sub(start) < 20*interval {
			time.Sleep(30 * time.Millisecond)
			raw, _ := inm.DisplayMetrics(nil, nil)
			summary = raw.(MetricsSummary)
		}
		// save result for testing
		for _, g := range summary.Gauges {
			if g.Name == "foo.bar" {
				result <- g.Value
			}
		}
		close(result)
	}()

	got := <-result
	verify.Values(t, "all", got, float32(0.0))
}

func TestDisplayMetrics_RaceMetricsSetGauge(t *testing.T) {
	interval := 200 * time.Millisecond
	inm := NewInmemSink(interval, 10*interval)
	met := &Metrics{Config: Config{FilterDefault: true}, sink: inm}
	result := make(chan float32)
	labels := []Label{
		{"name1", "value1"},
		{"name2", "value2"},
	}

	go func() {
		for {
			time.Sleep(75 * time.Millisecond)
			met.SetGaugeWithLabels([]string{"foo", "bar"}, float32(42), labels)
		}
	}()

	go func() {
		start := time.Now()
		var summary MetricsSummary
		// test for twenty intervals
		for time.Now().Sub(start) < 40*interval {
			time.Sleep(150 * time.Millisecond)
			raw, _ := inm.DisplayMetrics(nil, nil)
			summary = raw.(MetricsSummary)
		}
		// save result
		for _, g := range summary.Gauges {
			if g.Name == "foo.bar" {
				result <- g.Value
			}
		}
		close(result)
	}()

	got := <-result
	verify.Values(t, "all", got, float32(42))
}

func TestInmemSink_Stream(t *testing.T) {
	interval := 10 * time.Millisecond
	total := 50 * time.Millisecond
	inm := NewInmemSink(interval, total)

	ctx, cancel := context.WithTimeout(context.Background(), total*2)
	defer cancel()

	chDone := make(chan struct{})

	go func() {
		for i := float32(0); ctx.Err() == nil; i++ {
			inm.SetGaugeWithLabels([]string{"gauge", "foo"}, 20+i, []Label{{"a", "b"}})
			inm.EmitKey([]string{"key", "foo"}, 30+i)
			inm.IncrCounterWithLabels([]string{"counter", "bar"}, 40+i, []Label{{"a", "b"}})
			inm.IncrCounterWithLabels([]string{"counter", "bar"}, 50+i, []Label{{"a", "b"}})
			inm.AddSampleWithLabels([]string{"sample", "bar"}, 60+i, []Label{{"a", "b"}})
			inm.AddSampleWithLabels([]string{"sample", "bar"}, 70+i, []Label{{"a", "b"}})
			time.Sleep(interval / 3)
		}
		close(chDone)
	}()

	resp := httptest.NewRecorder()
	enc := encoder{
		encoder: json.NewEncoder(resp),
		flusher: resp,
	}
	inm.Stream(ctx, enc)

	<-chDone

	decoder := json.NewDecoder(resp.Body)
	var prevGaugeValue float32
	for i := 0; i < 8; i++ {
		var summary MetricsSummary
		if err := decoder.Decode(&summary); err != nil {
			t.Fatalf("expected no error while decoding response %d, got %v", i, err)
		}
		if count := len(summary.Gauges); count != 1 {
			t.Fatalf("expected at least one gauge in response %d, got %v", i, count)
		}
		value := summary.Gauges[0].Value
		// The upper bound of the gauge value is not known, but we can expect it
		// to be less than 50 because it increments by 3 every interval and we run
		// for ~10 intervals.
		if value < 20 || value > 50 {
			t.Fatalf("expected interval %d guage value between 20 and 50, got %v", i, value)
		}
		if value <= prevGaugeValue {
			t.Fatalf("expected interval %d guage value to be greater than previous, %v == %v", i, value, prevGaugeValue)
		}
		prevGaugeValue = value
	}
}

type encoder struct {
	flusher http.Flusher
	encoder *json.Encoder
}

func (e encoder) Encode(metrics interface{}) error {
	if err := e.encoder.Encode(metrics); err != nil {
		fmt.Println("failed to encode metrics summary", "error", err)
		return err
	}
	e.flusher.Flush()
	return nil
}
