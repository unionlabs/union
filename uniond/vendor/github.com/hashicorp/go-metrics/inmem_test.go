package metrics

import (
	"math"
	"net/url"
	"strings"
	"testing"
	"time"
)

func TestInmemSink(t *testing.T) {
	inm := NewInmemSink(10*time.Millisecond, 50*time.Millisecond)

	data := inm.Data()
	if len(data) != 1 {
		t.Fatalf("bad: %v", data)
	}

	// Add data points
	inm.SetGauge([]string{"foo", "bar"}, 42)
	inm.SetGaugeWithLabels([]string{"foo", "bar"}, 23, []Label{{"a", "b"}})
	inm.EmitKey([]string{"foo", "bar"}, 42)
	inm.IncrCounter([]string{"foo", "bar"}, 20)
	inm.IncrCounter([]string{"foo", "bar"}, 22)
	inm.IncrCounterWithLabels([]string{"foo", "bar"}, 20, []Label{{"a", "b"}})
	inm.IncrCounterWithLabels([]string{"foo", "bar"}, 22, []Label{{"a", "b"}})
	inm.AddSample([]string{"foo", "bar"}, 20)
	inm.AddSample([]string{"foo", "bar"}, 22)
	inm.AddSampleWithLabels([]string{"foo", "bar"}, 23, []Label{{"a", "b"}})

	data = inm.Data()
	if len(data) != 1 {
		t.Fatalf("bad: %v", data)
	}

	intvM := data[0]
	intvM.RLock()

	if time.Now().Sub(intvM.Interval) > 10*time.Millisecond {
		t.Fatalf("interval too old")
	}
	if intvM.Gauges["foo.bar"].Value != 42 {
		t.Fatalf("bad val: %v", intvM.Gauges)
	}
	if intvM.Gauges["foo.bar;a=b"].Value != 23 {
		t.Fatalf("bad val: %v", intvM.Gauges)
	}
	if intvM.Points["foo.bar"][0] != 42 {
		t.Fatalf("bad val: %v", intvM.Points)
	}

	for _, agg := range []SampledValue{intvM.Counters["foo.bar"], intvM.Counters["foo.bar;a=b"]} {
		if agg.Count != 2 {
			t.Fatalf("bad val: %v", agg)
		}
		if agg.Rate != 4200 {
			t.Fatalf("bad val: %v", agg.Rate)
		}
		if agg.Sum != 42 {
			t.Fatalf("bad val: %v", agg)
		}
		if agg.SumSq != 884 {
			t.Fatalf("bad val: %v", agg)
		}
		if agg.Min != 20 {
			t.Fatalf("bad val: %v", agg)
		}
		if agg.Max != 22 {
			t.Fatalf("bad val: %v", agg)
		}
		if agg.AggregateSample.Mean() != 21 {
			t.Fatalf("bad val: %v", agg)
		}
		if agg.AggregateSample.Stddev() != math.Sqrt(2) {
			t.Fatalf("bad val: %v", agg)
		}

		if agg.LastUpdated.IsZero() {
			t.Fatalf("agg.LastUpdated is not set: %v", agg)
		}

		diff := time.Now().Sub(agg.LastUpdated).Seconds()
		if diff > 1 {
			t.Fatalf("time diff too great: %f", diff)
		}
	}

	if _, ok := intvM.Samples["foo.bar"]; !ok {
		t.Fatalf("missing sample")
	}

	if _, ok := intvM.Samples["foo.bar;a=b"]; !ok {
		t.Fatalf("missing sample")
	}

	intvM.RUnlock()

	for i := 1; i < 10; i++ {
		time.Sleep(10 * time.Millisecond)
		inm.SetGauge([]string{"foo", "bar"}, 42)
		data = inm.Data()
		if len(data) != min(i+1, 5) {
			t.Fatalf("bad: %v", data)
		}
	}

	// Should not exceed 5 intervals!
	time.Sleep(10 * time.Millisecond)
	inm.SetGauge([]string{"foo", "bar"}, 42)
	data = inm.Data()
	if len(data) != 5 {
		t.Fatalf("bad: %v", data)
	}
}

func TestNewInmemSinkFromURL(t *testing.T) {
	for _, tc := range []struct {
		desc           string
		input          string
		expectErr      string
		expectInterval time.Duration
		expectRetain   time.Duration
	}{
		{
			desc:           "interval and duration are set via query params",
			input:          "inmem://?interval=11s&retain=22s",
			expectInterval: duration(t, "11s"),
			expectRetain:   duration(t, "22s"),
		},
		{
			desc:      "interval is required",
			input:     "inmem://?retain=22s",
			expectErr: "Bad 'interval' param",
		},
		{
			desc:      "interval must be a duration",
			input:     "inmem://?retain=30s&interval=HIYA",
			expectErr: "Bad 'interval' param",
		},
		{
			desc:      "retain is required",
			input:     "inmem://?interval=30s",
			expectErr: "Bad 'retain' param",
		},
		{
			desc:      "retain must be a valid duration",
			input:     "inmem://?interval=30s&retain=HELLO",
			expectErr: "Bad 'retain' param",
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			u, err := url.Parse(tc.input)
			if err != nil {
				t.Fatalf("error parsing URL: %s", err)
			}
			ms, err := NewInmemSinkFromURL(u)
			if tc.expectErr != "" {
				if !strings.Contains(err.Error(), tc.expectErr) {
					t.Fatalf("expected err: %q, to contain: %q", err, tc.expectErr)
				}
			} else {
				if err != nil {
					t.Fatalf("unexpected err: %s", err)
				}
				is := ms.(*InmemSink)
				if is.interval != tc.expectInterval {
					t.Fatalf("expected interval %s, got: %s", tc.expectInterval, is.interval)
				}
				if is.retain != tc.expectRetain {
					t.Fatalf("expected retain %s, got: %s", tc.expectRetain, is.retain)
				}
			}
		})
	}
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func duration(t *testing.T, s string) time.Duration {
	dur, err := time.ParseDuration(s)
	if err != nil {
		t.Fatalf("error parsing duration: %s", err)
	}
	return dur
}
