// Copyright 2018 The Prometheus Authors
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

package testutil

import (
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/prometheus/common/expfmt"

	"github.com/prometheus/client_golang/prometheus"
)

type untypedCollector struct{}

func (u untypedCollector) Describe(c chan<- *prometheus.Desc) {
	c <- prometheus.NewDesc("name", "help", nil, nil)
}

func (u untypedCollector) Collect(c chan<- prometheus.Metric) {
	c <- prometheus.MustNewConstMetric(
		prometheus.NewDesc("name", "help", nil, nil),
		prometheus.UntypedValue,
		2001,
	)
}

func TestToFloat64(t *testing.T) {
	gaugeWithAValueSet := prometheus.NewGauge(prometheus.GaugeOpts{})
	gaugeWithAValueSet.Set(3.14)

	counterVecWithOneElement := prometheus.NewCounterVec(prometheus.CounterOpts{}, []string{"foo"})
	counterVecWithOneElement.WithLabelValues("bar").Inc()

	counterVecWithTwoElements := prometheus.NewCounterVec(prometheus.CounterOpts{}, []string{"foo"})
	counterVecWithTwoElements.WithLabelValues("bar").Add(42)
	counterVecWithTwoElements.WithLabelValues("baz").Inc()

	histogramVecWithOneElement := prometheus.NewHistogramVec(prometheus.HistogramOpts{}, []string{"foo"})
	histogramVecWithOneElement.WithLabelValues("bar").Observe(2.7)

	scenarios := map[string]struct {
		collector prometheus.Collector
		panics    bool
		want      float64
	}{
		"simple counter": {
			collector: prometheus.NewCounter(prometheus.CounterOpts{}),
			panics:    false,
			want:      0,
		},
		"simple gauge": {
			collector: prometheus.NewGauge(prometheus.GaugeOpts{}),
			panics:    false,
			want:      0,
		},
		"simple untyped": {
			collector: untypedCollector{},
			panics:    false,
			want:      2001,
		},
		"simple histogram": {
			collector: prometheus.NewHistogram(prometheus.HistogramOpts{}),
			panics:    true,
		},
		"simple summary": {
			collector: prometheus.NewSummary(prometheus.SummaryOpts{}),
			panics:    true,
		},
		"simple gauge with an actual value set": {
			collector: gaugeWithAValueSet,
			panics:    false,
			want:      3.14,
		},
		"counter vec with zero elements": {
			collector: prometheus.NewCounterVec(prometheus.CounterOpts{}, nil),
			panics:    true,
		},
		"counter vec with one element": {
			collector: counterVecWithOneElement,
			panics:    false,
			want:      1,
		},
		"counter vec with two elements": {
			collector: counterVecWithTwoElements,
			panics:    true,
		},
		"histogram vec with one element": {
			collector: histogramVecWithOneElement,
			panics:    true,
		},
	}

	for n, s := range scenarios {
		t.Run(n, func(t *testing.T) {
			defer func() {
				r := recover()
				if r == nil && s.panics {
					t.Error("expected panic")
				} else if r != nil && !s.panics {
					t.Error("unexpected panic: ", r)
				}
				// Any other combination is the expected outcome.
			}()
			if got := ToFloat64(s.collector); got != s.want {
				t.Errorf("want %f, got %f", s.want, got)
			}
		})
	}
}

func TestCollectAndCompare(t *testing.T) {
	const metadata = `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter
	`

	c := prometheus.NewCounter(prometheus.CounterOpts{
		Name: "some_total",
		Help: "A value that represents a counter.",
		ConstLabels: prometheus.Labels{
			"label1": "value1",
		},
	})
	c.Inc()

	expected := `

		some_total{ label1 = "value1" } 1
	`

	if err := CollectAndCompare(c, strings.NewReader(metadata+expected), "some_total"); err != nil {
		t.Errorf("unexpected collecting result:\n%s", err)
	}
}

func TestCollectAndCompareNoLabel(t *testing.T) {
	const metadata = `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter
	`

	c := prometheus.NewCounter(prometheus.CounterOpts{
		Name: "some_total",
		Help: "A value that represents a counter.",
	})
	c.Inc()

	expected := `

		some_total 1
	`

	if err := CollectAndCompare(c, strings.NewReader(metadata+expected), "some_total"); err != nil {
		t.Errorf("unexpected collecting result:\n%s", err)
	}
}

func TestCollectAndCompareNoHelp(t *testing.T) {
	const metadata = `
		# TYPE some_total counter
	`

	c := prometheus.NewCounter(prometheus.CounterOpts{
		Name: "some_total",
	})
	c.Inc()

	expected := `

		some_total 1
	`

	if err := CollectAndCompare(c, strings.NewReader(metadata+expected), "some_total"); err != nil {
		t.Errorf("unexpected collecting result:\n%s", err)
	}
}

func TestCollectAndCompareHistogram(t *testing.T) {
	inputs := []struct {
		name        string
		c           prometheus.Collector
		metadata    string
		expect      string
		observation float64
	}{
		{
			name: "Testing Histogram Collector",
			c: prometheus.NewHistogram(prometheus.HistogramOpts{
				Name:    "some_histogram",
				Help:    "An example of a histogram",
				Buckets: []float64{1, 2, 3},
			}),
			metadata: `
				# HELP some_histogram An example of a histogram
				# TYPE some_histogram histogram
			`,
			expect: `
				some_histogram{le="1"} 0
				some_histogram{le="2"} 0
				some_histogram{le="3"} 1
        			some_histogram_bucket{le="+Inf"} 1
        			some_histogram_sum 2.5
        			some_histogram_count 1

			`,
			observation: 2.5,
		},
		{
			name: "Testing HistogramVec Collector",
			c: prometheus.NewHistogramVec(prometheus.HistogramOpts{
				Name:    "some_histogram",
				Help:    "An example of a histogram",
				Buckets: []float64{1, 2, 3},
			}, []string{"test"}),

			metadata: `
				# HELP some_histogram An example of a histogram
				# TYPE some_histogram histogram
			`,
			expect: `
            			some_histogram_bucket{test="test",le="1"} 0
            			some_histogram_bucket{test="test",le="2"} 0
            			some_histogram_bucket{test="test",le="3"} 1
            			some_histogram_bucket{test="test",le="+Inf"} 1
            			some_histogram_sum{test="test"} 2.5
           		 	some_histogram_count{test="test"} 1
		
			`,
			observation: 2.5,
		},
	}

	for _, input := range inputs {
		switch collector := input.c.(type) {
		case prometheus.Histogram:
			collector.Observe(input.observation)
		case *prometheus.HistogramVec:
			collector.WithLabelValues("test").Observe(input.observation)
		default:
			t.Fatalf("unsupported collector tested")

		}

		t.Run(input.name, func(t *testing.T) {
			if err := CollectAndCompare(input.c, strings.NewReader(input.metadata+input.expect)); err != nil {
				t.Errorf("unexpected collecting result:\n%s", err)
			}
		})

	}
}

func TestNoMetricFilter(t *testing.T) {
	const metadata = `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter
	`

	c := prometheus.NewCounter(prometheus.CounterOpts{
		Name: "some_total",
		Help: "A value that represents a counter.",
		ConstLabels: prometheus.Labels{
			"label1": "value1",
		},
	})
	c.Inc()

	expected := `
		some_total{label1="value1"} 1
	`

	if err := CollectAndCompare(c, strings.NewReader(metadata+expected)); err != nil {
		t.Errorf("unexpected collecting result:\n%s", err)
	}
}

func TestMetricNotFound(t *testing.T) {
	const metadata = `
		# HELP some_other_metric A value that represents a counter.
		# TYPE some_other_metric counter
	`

	c := prometheus.NewCounter(prometheus.CounterOpts{
		Name: "some_total",
		Help: "A value that represents a counter.",
		ConstLabels: prometheus.Labels{
			"label1": "value1",
		},
	})

	c.Inc()

	expected := `
		some_other_metric{label1="value1"} 1
	`

	expectedError := `-# HELP some_total A value that represents a counter.
-# TYPE some_total counter
-some_total{label1="value1"} 1
+# HELP some_other_metric A value that represents a counter.
+# TYPE some_other_metric counter
+some_other_metric{label1="value1"} 1
 `

	err := CollectAndCompare(c, strings.NewReader(metadata+expected))
	if err == nil {
		t.Error("Expected error, got no error.")
	}

	if err.Error() != expectedError {
		t.Errorf("Expected\n%#+v\nGot:\n%#+v", expectedError, err.Error())
	}
}

func TestScrapeAndCompare(t *testing.T) {
	scenarios := map[string]struct {
		want        string
		metricNames []string
		// expectedErr if empty, means no fail is expected for the comparison.
		expectedErr string
	}{
		"empty metric Names": {
			want: `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter

		some_total{ label1 = "value1" } 1
	`,
			metricNames: []string{},
		},
		"one metric": {
			want: `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter

		some_total{ label1 = "value1" } 1
	`,
			metricNames: []string{"some_total"},
		},
		"multiple expected": {
			want: `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter

		some_total{ label1 = "value1" } 1

		# HELP some_total2 A value that represents a counter.
		# TYPE some_total2 counter

		some_total2{ label2 = "value2" } 1
	`,
			metricNames: []string{"some_total2"},
		},
		"expected metric name is not scraped": {
			want: `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter

		some_total{ label1 = "value1" } 1

		# HELP some_total2 A value that represents a counter.
		# TYPE some_total2 counter

		some_total2{ label2 = "value2" } 1
	`,
			metricNames: []string{"some_total3"},
			expectedErr: "expected metric name(s) not found: [some_total3]",
		},
		"one of multiple expected metric names is not scraped": {
			want: `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter

		some_total{ label1 = "value1" } 1

		# HELP some_total2 A value that represents a counter.
		# TYPE some_total2 counter

		some_total2{ label2 = "value2" } 1
	`,
			metricNames: []string{"some_total1", "some_total3"},
			expectedErr: "expected metric name(s) not found: [some_total1 some_total3]",
		},
	}
	for name, scenario := range scenarios {
		t.Run(name, func(t *testing.T) {
			expectedReader := strings.NewReader(scenario.want)

			ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				fmt.Fprintln(w, scenario.want)
			}))
			defer ts.Close()
			if err := ScrapeAndCompare(ts.URL, expectedReader, scenario.metricNames...); err != nil {
				if scenario.expectedErr == "" || err.Error() != scenario.expectedErr {
					t.Errorf("unexpected error happened: %s", err)
				}
			} else if scenario.expectedErr != "" {
				t.Errorf("expected an error but got nil")
			}
		})
	}

	t.Run("fetching fail", func(t *testing.T) {
		err := ScrapeAndCompare("some_url", strings.NewReader("some expectation"), "some_total")
		if err == nil {
			t.Errorf("expected an error but got nil")
		}
		if !strings.HasPrefix(err.Error(), "scraping metrics failed") {
			t.Errorf("unexpected error happened: %s", err)
		}
	})

	t.Run("bad status code", func(t *testing.T) {
		const expected = `
		# HELP some_total A value that represents a counter.
		# TYPE some_total counter

		some_total{ label1 = "value1" } 1
	`

		expectedReader := strings.NewReader(expected)

		ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusBadGateway)
			fmt.Fprintln(w, expected)
		}))
		defer ts.Close()

		err := ScrapeAndCompare(ts.URL, expectedReader, "some_total")
		if err == nil {
			t.Errorf("expected an error but got nil")
		}
		if !strings.HasPrefix(err.Error(), "the scraping target returned a status code other than 200") {
			t.Errorf("unexpected error happened: %s", err)
		}
	})
}

func TestCollectAndCount(t *testing.T) {
	c := prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "some_total",
			Help: "A value that represents a counter.",
		},
		[]string{"foo"},
	)
	if got, want := CollectAndCount(c), 0; got != want {
		t.Errorf("unexpected metric count, got %d, want %d", got, want)
	}
	c.WithLabelValues("bar")
	if got, want := CollectAndCount(c), 1; got != want {
		t.Errorf("unexpected metric count, got %d, want %d", got, want)
	}
	c.WithLabelValues("baz")
	if got, want := CollectAndCount(c), 2; got != want {
		t.Errorf("unexpected metric count, got %d, want %d", got, want)
	}
	if got, want := CollectAndCount(c, "some_total"), 2; got != want {
		t.Errorf("unexpected metric count, got %d, want %d", got, want)
	}
	if got, want := CollectAndCount(c, "some_other_total"), 0; got != want {
		t.Errorf("unexpected metric count, got %d, want %d", got, want)
	}
}

func TestCollectAndFormat(t *testing.T) {
	const expected = `# HELP foo_bar A value that represents the number of bars in foo.
# TYPE foo_bar counter
foo_bar{fizz="bang"} 1
`
	c := prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "foo_bar",
			Help: "A value that represents the number of bars in foo.",
		},
		[]string{"fizz"},
	)
	c.WithLabelValues("bang").Inc()

	got, err := CollectAndFormat(c, expfmt.TypeTextPlain, "foo_bar")
	if err != nil {
		t.Errorf("unexpected error: %s", err.Error())
	}

	gotS := string(got)
	if err != nil {
		t.Errorf("unexpected error: %s", err.Error())
	}

	if gotS != expected {
		t.Errorf("unexpected metric output, got %q, expected %q", gotS, expected)
	}
}
