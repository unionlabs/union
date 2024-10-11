package prometheus

import (
	"io/ioutil"
	"math"
	"math/rand"
	"net/http"
	"net/http/httptest"
	"reflect"
	"regexp"
	"strconv"
	"strings"
	"testing"

	"github.com/go-kit/kit/metrics/teststat"
	stdprometheus "github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

func TestCounter(t *testing.T) {
	s := httptest.NewServer(promhttp.HandlerFor(stdprometheus.DefaultGatherer, promhttp.HandlerOpts{}))
	defer s.Close()

	scrape := func() string {
		resp, _ := http.Get(s.URL)
		buf, _ := ioutil.ReadAll(resp.Body)
		return string(buf)
	}

	namespace, subsystem, name := "ns", "ss", "foo"
	re := regexp.MustCompile(namespace + `_` + subsystem + `_` + name + `{alpha="alpha-value",beta="beta-value"} ([0-9\.]+)`)

	counter := NewCounterFrom(stdprometheus.CounterOpts{
		Namespace: namespace,
		Subsystem: subsystem,
		Name:      name,
		Help:      "This is the help string.",
	}, []string{"alpha", "beta"}).With("beta", "beta-value", "alpha", "alpha-value") // order shouldn't matter

	value := func() float64 {
		matches := re.FindStringSubmatch(scrape())
		f, _ := strconv.ParseFloat(matches[1], 64)
		return f
	}

	if err := teststat.TestCounter(counter, value); err != nil {
		t.Fatal(err)
	}
}

func TestGauge(t *testing.T) {
	s := httptest.NewServer(promhttp.HandlerFor(stdprometheus.DefaultGatherer, promhttp.HandlerOpts{}))
	defer s.Close()

	scrape := func() string {
		resp, _ := http.Get(s.URL)
		buf, _ := ioutil.ReadAll(resp.Body)
		return string(buf)
	}

	namespace, subsystem, name := "aaa", "bbb", "ccc"
	re := regexp.MustCompile(namespace + `_` + subsystem + `_` + name + `{foo="bar"} ([0-9\.]+)`)

	gauge := NewGaugeFrom(stdprometheus.GaugeOpts{
		Namespace: namespace,
		Subsystem: subsystem,
		Name:      name,
		Help:      "This is a different help string.",
	}, []string{"foo"}).With("foo", "bar")

	value := func() []float64 {
		matches := re.FindStringSubmatch(scrape())
		f, _ := strconv.ParseFloat(matches[1], 64)
		return []float64{f}
	}

	if err := teststat.TestGauge(gauge, value); err != nil {
		t.Fatal(err)
	}
}

func TestSummary(t *testing.T) {
	s := httptest.NewServer(promhttp.HandlerFor(stdprometheus.DefaultGatherer, promhttp.HandlerOpts{}))
	defer s.Close()

	scrape := func() string {
		resp, _ := http.Get(s.URL)
		buf, _ := ioutil.ReadAll(resp.Body)
		return string(buf)
	}

	namespace, subsystem, name := "test", "prometheus", "summary"
	re50 := regexp.MustCompile(namespace + `_` + subsystem + `_` + name + `{a="a",b="b",quantile="0.5"} ([0-9\.]+)`)
	re90 := regexp.MustCompile(namespace + `_` + subsystem + `_` + name + `{a="a",b="b",quantile="0.9"} ([0-9\.]+)`)
	re99 := regexp.MustCompile(namespace + `_` + subsystem + `_` + name + `{a="a",b="b",quantile="0.99"} ([0-9\.]+)`)

	summary := NewSummaryFrom(stdprometheus.SummaryOpts{
		Namespace:  namespace,
		Subsystem:  subsystem,
		Name:       name,
		Help:       "This is the help string for the summary.",
		Objectives: map[float64]float64{0.5: 0.05, 0.9: 0.01, 0.99: 0.001},
	}, []string{"a", "b"}).With("b", "b").With("a", "a")

	quantiles := func() (float64, float64, float64, float64) {
		buf := scrape()
		match50 := re50.FindStringSubmatch(buf)
		p50, _ := strconv.ParseFloat(match50[1], 64)
		match90 := re90.FindStringSubmatch(buf)
		p90, _ := strconv.ParseFloat(match90[1], 64)
		match99 := re99.FindStringSubmatch(buf)
		p99, _ := strconv.ParseFloat(match99[1], 64)
		p95 := p90 + ((p99 - p90) / 2) // Prometheus, y u no p95??? :< #yolo
		return p50, p90, p95, p99
	}

	if err := teststat.TestHistogram(summary, quantiles, 0.01); err != nil {
		t.Fatal(err)
	}
}

func TestHistogram(t *testing.T) {
	// Prometheus reports histograms as a count of observations that fell into
	// each predefined bucket, with the bucket value representing a global upper
	// limit. That is, the count monotonically increases over the buckets. This
	// requires a different strategy to test.

	s := httptest.NewServer(promhttp.HandlerFor(stdprometheus.DefaultGatherer, promhttp.HandlerOpts{}))
	defer s.Close()

	scrape := func() string {
		resp, _ := http.Get(s.URL)
		buf, _ := ioutil.ReadAll(resp.Body)
		return string(buf)
	}

	namespace, subsystem, name := "test", "prometheus", "histogram"
	re := regexp.MustCompile(namespace + `_` + subsystem + `_` + name + `_bucket{x="1",le="([0-9]+|\+Inf)"} ([0-9\.]+)`)

	numStdev := 3
	bucketMin := (teststat.Mean - (numStdev * teststat.Stdev))
	bucketMax := (teststat.Mean + (numStdev * teststat.Stdev))
	if bucketMin < 0 {
		bucketMin = 0
	}
	bucketCount := 10
	bucketDelta := (bucketMax - bucketMin) / bucketCount
	buckets := []float64{}
	for i := bucketMin; i <= bucketMax; i += bucketDelta {
		buckets = append(buckets, float64(i))
	}

	histogram := NewHistogramFrom(stdprometheus.HistogramOpts{
		Namespace: namespace,
		Subsystem: subsystem,
		Name:      name,
		Help:      "This is the help string for the histogram.",
		Buckets:   buckets,
	}, []string{"x"}).With("x", "1")

	// Can't TestHistogram, because Prometheus Histograms don't dynamically
	// compute quantiles. Instead, they fill up buckets. So, let's populate the
	// histogram kind of manually.
	teststat.PopulateNormalHistogram(histogram, rand.Int())

	// Then, we use ExpectedObservationsLessThan to validate.
	for _, line := range strings.Split(scrape(), "\n") {
		match := re.FindStringSubmatch(line)
		if match == nil {
			continue
		}

		bucket, _ := strconv.ParseInt(match[1], 10, 64)
		have, _ := strconv.ParseFloat(match[2], 64)

		want := teststat.ExpectedObservationsLessThan(bucket)
		if match[1] == "+Inf" {
			want = int64(teststat.Count) // special case
		}

		// Unfortunately, we observe experimentally that Prometheus is quite
		// imprecise at the extremes. I'm setting a very high tolerance for now.
		// It would be great to dig in and figure out whether that's a problem
		// with my Expected calculation, or in Prometheus.
		tolerance := 0.25
		if delta := math.Abs(float64(want) - float64(have)); (delta / float64(want)) > tolerance {
			t.Errorf("Bucket %d: want %d, have %d (%.1f%%)", bucket, want, int(have), (100.0 * delta / float64(want)))
		}
	}
}

func TestInconsistentLabelCardinality(t *testing.T) {
	defer func() {
		x := recover()
		if x == nil {
			t.Fatal("expected panic, got none")
		}
		err, ok := x.(error)
		if !ok {
			t.Fatalf("expected error, got %s", reflect.TypeOf(x))
		}
		if want, have := "inconsistent label cardinality", err.Error(); !strings.HasPrefix(have, want) {
			t.Fatalf("want %q, have %q", want, have)
		}
	}()

	NewCounterFrom(stdprometheus.CounterOpts{
		Namespace: "test",
		Subsystem: "inconsistent_label_cardinality",
		Name:      "foobar",
		Help:      "This is the help string for the metric.",
	}, []string{"a", "b"}).With(
		"a", "1", "b", "2", "c", "KABOOM!",
	).Add(123)
}
