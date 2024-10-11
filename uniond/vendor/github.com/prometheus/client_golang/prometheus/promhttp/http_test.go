// Copyright 2016 The Prometheus Authors
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

package promhttp

import (
	"bytes"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/klauspost/compress/zstd"
	dto "github.com/prometheus/client_model/go"

	"github.com/prometheus/client_golang/prometheus"
)

type errorCollector struct{}

const (
	acceptHeader    = "Accept"
	acceptTextPlain = "text/plain"
)

func (e errorCollector) Describe(ch chan<- *prometheus.Desc) {
	ch <- prometheus.NewDesc("invalid_metric", "not helpful", nil, nil)
}

func (e errorCollector) Collect(ch chan<- prometheus.Metric) {
	ch <- prometheus.NewInvalidMetric(
		prometheus.NewDesc("invalid_metric", "not helpful", nil, nil),
		errors.New("collect error"),
	)
}

type blockingCollector struct {
	CollectStarted, Block chan struct{}
}

func (b blockingCollector) Describe(ch chan<- *prometheus.Desc) {
	ch <- prometheus.NewDesc("dummy_desc", "not helpful", nil, nil)
}

func (b blockingCollector) Collect(ch chan<- prometheus.Metric) {
	select {
	case b.CollectStarted <- struct{}{}:
	default:
	}
	// Collects nothing, just waits for a channel receive.
	<-b.Block
}

type mockTransactionGatherer struct {
	g             prometheus.Gatherer
	gatherInvoked int
	doneInvoked   int
}

func (g *mockTransactionGatherer) Gather() (_ []*dto.MetricFamily, done func(), err error) {
	g.gatherInvoked++
	mfs, err := g.g.Gather()
	return mfs, func() { g.doneInvoked++ }, err
}

func readCompressedBody(r io.Reader, comp Compression) (string, error) {
	switch comp {
	case Gzip:
		reader, err := gzip.NewReader(r)
		if err != nil {
			return "", err
		}
		defer reader.Close()
		got, err := io.ReadAll(reader)
		return string(got), err
	case Zstd:
		reader, err := zstd.NewReader(r)
		if err != nil {
			return "", err
		}
		defer reader.Close()
		got, err := io.ReadAll(reader)
		return string(got), err
	}
	return "", fmt.Errorf("Unsupported compression")
}

func TestHandlerErrorHandling(t *testing.T) {
	// Create a registry that collects a MetricFamily with two elements,
	// another with one, and reports an error. Further down, we'll use the
	// same registry in the HandlerOpts.
	reg := prometheus.NewRegistry()

	cnt := prometheus.NewCounter(prometheus.CounterOpts{
		Name: "the_count",
		Help: "Ah-ah-ah! Thunder and lightning!",
	})
	reg.MustRegister(cnt)

	cntVec := prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name:        "name",
			Help:        "docstring",
			ConstLabels: prometheus.Labels{"constname": "constvalue"},
		},
		[]string{"labelname"},
	)
	cntVec.WithLabelValues("val1").Inc()
	cntVec.WithLabelValues("val2").Inc()
	reg.MustRegister(cntVec)

	reg.MustRegister(errorCollector{})

	logBuf := &bytes.Buffer{}
	logger := log.New(logBuf, "", 0)

	writer := httptest.NewRecorder()
	request, _ := http.NewRequest("GET", "/", nil)
	request.Header.Add("Accept", "test/plain")

	mReg := &mockTransactionGatherer{g: reg}
	errorHandler := HandlerForTransactional(mReg, HandlerOpts{
		ErrorLog:      logger,
		ErrorHandling: HTTPErrorOnError,
		Registry:      reg,
	})
	continueHandler := HandlerForTransactional(mReg, HandlerOpts{
		ErrorLog:      logger,
		ErrorHandling: ContinueOnError,
		Registry:      reg,
	})
	panicHandler := HandlerForTransactional(mReg, HandlerOpts{
		ErrorLog:      logger,
		ErrorHandling: PanicOnError,
		Registry:      reg,
	})
	// Expect gatherer not touched.
	if got := mReg.gatherInvoked; got != 0 {
		t.Fatalf("unexpected number of gather invokes, want 0, got %d", got)
	}
	if got := mReg.doneInvoked; got != 0 {
		t.Fatalf("unexpected number of done invokes, want 0, got %d", got)
	}

	wantMsg := `error gathering metrics: error collecting metric Desc{fqName: "invalid_metric", help: "not helpful", constLabels: {}, variableLabels: {}}: collect error
`
	wantErrorBody := `An error has occurred while serving metrics:

error collecting metric Desc{fqName: "invalid_metric", help: "not helpful", constLabels: {}, variableLabels: {}}: collect error
`
	wantOKBody1 := `# HELP name docstring
# TYPE name counter
name{constname="constvalue",labelname="val1"} 1
name{constname="constvalue",labelname="val2"} 1
# HELP promhttp_metric_handler_errors_total Total number of internal errors encountered by the promhttp metric handler.
# TYPE promhttp_metric_handler_errors_total counter
promhttp_metric_handler_errors_total{cause="encoding"} 0
promhttp_metric_handler_errors_total{cause="gathering"} 1
# HELP the_count Ah-ah-ah! Thunder and lightning!
# TYPE the_count counter
the_count 0
`
	// It might happen that counting the gathering error makes it to the
	// promhttp_metric_handler_errors_total counter before it is gathered
	// itself. Thus, we have to bodies that are acceptable for the test.
	wantOKBody2 := `# HELP name docstring
# TYPE name counter
name{constname="constvalue",labelname="val1"} 1
name{constname="constvalue",labelname="val2"} 1
# HELP promhttp_metric_handler_errors_total Total number of internal errors encountered by the promhttp metric handler.
# TYPE promhttp_metric_handler_errors_total counter
promhttp_metric_handler_errors_total{cause="encoding"} 0
promhttp_metric_handler_errors_total{cause="gathering"} 2
# HELP the_count Ah-ah-ah! Thunder and lightning!
# TYPE the_count counter
the_count 0
`

	errorHandler.ServeHTTP(writer, request)
	if got := mReg.gatherInvoked; got != 1 {
		t.Fatalf("unexpected number of gather invokes, want 1, got %d", got)
	}
	if got := mReg.doneInvoked; got != 1 {
		t.Fatalf("unexpected number of done invokes, want 1, got %d", got)
	}
	if got, want := writer.Code, http.StatusInternalServerError; got != want {
		t.Errorf("got HTTP status code %d, want %d", got, want)
	}
	if got, want := logBuf.String(), wantMsg; got != want {
		t.Errorf("got log buf %q, want %q", got, want)
	}
	if got, want := writer.Body.String(), wantErrorBody; got != want {
		t.Errorf("got body %q, want %q", got, want)
	}

	logBuf.Reset()
	writer.Body.Reset()
	writer.Code = http.StatusOK

	continueHandler.ServeHTTP(writer, request)

	if got := mReg.gatherInvoked; got != 2 {
		t.Fatalf("unexpected number of gather invokes, want 2, got %d", got)
	}
	if got := mReg.doneInvoked; got != 2 {
		t.Fatalf("unexpected number of done invokes, want 2, got %d", got)
	}
	if got, want := writer.Code, http.StatusOK; got != want {
		t.Errorf("got HTTP status code %d, want %d", got, want)
	}
	if got, want := logBuf.String(), wantMsg; got != want {
		t.Errorf("got log buf %q, want %q", got, want)
	}
	if got := writer.Body.String(); got != wantOKBody1 && got != wantOKBody2 {
		t.Errorf("got body %q, want either %q or %q", got, wantOKBody1, wantOKBody2)
	}

	defer func() {
		if err := recover(); err == nil {
			t.Error("expected panic from panicHandler")
		}
		if got := mReg.gatherInvoked; got != 3 {
			t.Fatalf("unexpected number of gather invokes, want 3, got %d", got)
		}
		if got := mReg.doneInvoked; got != 3 {
			t.Fatalf("unexpected number of done invokes, want 3, got %d", got)
		}
	}()
	panicHandler.ServeHTTP(writer, request)
}

func TestInstrumentMetricHandler(t *testing.T) {
	reg := prometheus.NewRegistry()
	mReg := &mockTransactionGatherer{g: reg}
	handler := InstrumentMetricHandler(reg, HandlerForTransactional(mReg, HandlerOpts{}))
	// Do it again to test idempotency.
	InstrumentMetricHandler(reg, HandlerForTransactional(mReg, HandlerOpts{}))
	writer := httptest.NewRecorder()
	request, _ := http.NewRequest("GET", "/", nil)
	request.Header.Add(acceptHeader, acceptTextPlain)

	handler.ServeHTTP(writer, request)
	if got := mReg.gatherInvoked; got != 1 {
		t.Fatalf("unexpected number of gather invokes, want 1, got %d", got)
	}
	if got := mReg.doneInvoked; got != 1 {
		t.Fatalf("unexpected number of done invokes, want 1, got %d", got)
	}

	if got, want := writer.Code, http.StatusOK; got != want {
		t.Errorf("got HTTP status code %d, want %d", got, want)
	}

	if got, want := writer.Header().Get(contentEncodingHeader), ""; got != want {
		t.Errorf("got HTTP content encoding header %s, want %s", got, want)
	}

	want := "promhttp_metric_handler_requests_in_flight 1\n"
	if got := writer.Body.String(); !strings.Contains(got, want) {
		t.Errorf("got body %q, does not contain %q", got, want)
	}
	want = "promhttp_metric_handler_requests_total{code=\"200\"} 0\n"
	if got := writer.Body.String(); !strings.Contains(got, want) {
		t.Errorf("got body %q, does not contain %q", got, want)
	}

	for i := 0; i < 100; i++ {
		writer.Body.Reset()
		handler.ServeHTTP(writer, request)

		if got, want := mReg.gatherInvoked, i+2; got != want {
			t.Fatalf("unexpected number of gather invokes, want %d, got %d", want, got)
		}
		if got, want := mReg.doneInvoked, i+2; got != want {
			t.Fatalf("unexpected number of done invokes, want %d, got %d", want, got)
		}
		if got, want := writer.Code, http.StatusOK; got != want {
			t.Errorf("got HTTP status code %d, want %d", got, want)
		}

		want := "promhttp_metric_handler_requests_in_flight 1\n"
		if got := writer.Body.String(); !strings.Contains(got, want) {
			t.Errorf("got body %q, does not contain %q", got, want)
		}
		want = fmt.Sprintf("promhttp_metric_handler_requests_total{code=\"200\"} %d\n", i+1)
		if got := writer.Body.String(); !strings.Contains(got, want) {
			t.Errorf("got body %q, does not contain %q", got, want)
		}
	}
}

func TestHandlerMaxRequestsInFlight(t *testing.T) {
	reg := prometheus.NewRegistry()
	handler := HandlerFor(reg, HandlerOpts{MaxRequestsInFlight: 1})
	w1 := httptest.NewRecorder()
	w2 := httptest.NewRecorder()
	w3 := httptest.NewRecorder()
	request, _ := http.NewRequest("GET", "/", nil)
	request.Header.Add(acceptHeader, acceptTextPlain)

	c := blockingCollector{Block: make(chan struct{}), CollectStarted: make(chan struct{}, 1)}
	reg.MustRegister(c)

	rq1Done := make(chan struct{})
	go func() {
		handler.ServeHTTP(w1, request)
		close(rq1Done)
	}()
	<-c.CollectStarted

	handler.ServeHTTP(w2, request)

	if got, want := w2.Code, http.StatusServiceUnavailable; got != want {
		t.Errorf("got HTTP status code %d, want %d", got, want)
	}
	if got, want := w2.Body.String(), "Limit of concurrent requests reached (1), try again later.\n"; got != want {
		t.Errorf("got body %q, want %q", got, want)
	}

	close(c.Block)
	<-rq1Done

	handler.ServeHTTP(w3, request)

	if got, want := w3.Code, http.StatusOK; got != want {
		t.Errorf("got HTTP status code %d, want %d", got, want)
	}
}

func TestHandlerTimeout(t *testing.T) {
	reg := prometheus.NewRegistry()
	handler := HandlerFor(reg, HandlerOpts{Timeout: time.Millisecond})
	w := httptest.NewRecorder()

	request, _ := http.NewRequest("GET", "/", nil)
	request.Header.Add("Accept", "test/plain")

	c := blockingCollector{Block: make(chan struct{}), CollectStarted: make(chan struct{}, 1)}
	reg.MustRegister(c)

	handler.ServeHTTP(w, request)

	if got, want := w.Code, http.StatusServiceUnavailable; got != want {
		t.Errorf("got HTTP status code %d, want %d", got, want)
	}
	if got, want := w.Body.String(), "Exceeded configured timeout of 1ms.\n"; got != want {
		t.Errorf("got body %q, want %q", got, want)
	}

	close(c.Block) // To not leak a goroutine.
}

func TestInstrumentMetricHandlerWithCompression(t *testing.T) {
	reg := prometheus.NewRegistry()
	mReg := &mockTransactionGatherer{g: reg}
	handler := InstrumentMetricHandler(reg, HandlerForTransactional(mReg, HandlerOpts{DisableCompression: false}))
	compression := Zstd
	writer := httptest.NewRecorder()
	request, _ := http.NewRequest("GET", "/", nil)
	request.Header.Add(acceptHeader, acceptTextPlain)
	request.Header.Add(acceptEncodingHeader, string(compression))

	handler.ServeHTTP(writer, request)
	if got := mReg.gatherInvoked; got != 1 {
		t.Fatalf("unexpected number of gather invokes, want 1, got %d", got)
	}
	if got := mReg.doneInvoked; got != 1 {
		t.Fatalf("unexpected number of done invokes, want 1, got %d", got)
	}

	if got, want := writer.Code, http.StatusOK; got != want {
		t.Errorf("got HTTP status code %d, want %d", got, want)
	}

	if got, want := writer.Header().Get(contentEncodingHeader), string(compression); got != want {
		t.Errorf("got HTTP content encoding header %s, want %s", got, want)
	}

	body, err := readCompressedBody(writer.Body, compression)
	want := "promhttp_metric_handler_requests_in_flight 1\n"
	if got := body; !strings.Contains(got, want) {
		t.Errorf("got body %q, does not contain %q, err: %v", got, want, err)
	}

	want = "promhttp_metric_handler_requests_total{code=\"200\"} 0\n"
	if got := body; !strings.Contains(got, want) {
		t.Errorf("got body %q, does not contain %q, err: %v", got, want, err)
	}

	for i := 0; i < 100; i++ {
		writer.Body.Reset()
		handler.ServeHTTP(writer, request)

		if got, want := mReg.gatherInvoked, i+2; got != want {
			t.Fatalf("unexpected number of gather invokes, want %d, got %d", want, got)
		}
		if got, want := mReg.doneInvoked, i+2; got != want {
			t.Fatalf("unexpected number of done invokes, want %d, got %d", want, got)
		}
		if got, want := writer.Code, http.StatusOK; got != want {
			t.Errorf("got HTTP status code %d, want %d", got, want)
		}
		body, err := readCompressedBody(writer.Body, compression)

		want := "promhttp_metric_handler_requests_in_flight 1\n"
		if got := body; !strings.Contains(got, want) {
			t.Errorf("got body %q, does not contain %q, err: %v", got, want, err)
		}

		want = fmt.Sprintf("promhttp_metric_handler_requests_total{code=\"200\"} %d\n", i+1)
		if got := body; !strings.Contains(got, want) {
			t.Errorf("got body %q, does not contain %q, err: %v", got, want, err)
		}
	}

	// Test with Zstd
	compression = Zstd
	request.Header.Set(acceptEncodingHeader, string(compression))

	handler.ServeHTTP(writer, request)

	if got, want := writer.Code, http.StatusOK; got != want {
		t.Errorf("got HTTP status code %d, want %d", got, want)
	}

	if got, want := writer.Header().Get(contentEncodingHeader), string(compression); got != want {
		t.Errorf("got HTTP content encoding header %s, want %s", got, want)
	}

	body, err = readCompressedBody(writer.Body, compression)
	want = "promhttp_metric_handler_requests_in_flight 1\n"
	if got := body; !strings.Contains(got, want) {
		t.Errorf("got body %q, does not contain %q, err: %v", got, want, err)
	}

	want = "promhttp_metric_handler_requests_total{code=\"200\"} 101\n"
	if got := body; !strings.Contains(got, want) {
		t.Errorf("got body %q, does not contain %q, err: %v", got, want, err)
	}

	for i := 101; i < 201; i++ {
		writer.Body.Reset()
		handler.ServeHTTP(writer, request)

		if got, want := mReg.gatherInvoked, i+2; got != want {
			t.Fatalf("unexpected number of gather invokes, want %d, got %d", want, got)
		}
		if got, want := mReg.doneInvoked, i+2; got != want {
			t.Fatalf("unexpected number of done invokes, want %d, got %d", want, got)
		}
		if got, want := writer.Code, http.StatusOK; got != want {
			t.Errorf("got HTTP status code %d, want %d", got, want)
		}
		body, err := readCompressedBody(writer.Body, compression)

		want := "promhttp_metric_handler_requests_in_flight 1\n"
		if got := body; !strings.Contains(got, want) {
			t.Errorf("got body %q, does not contain %q, err: %v", got, want, err)
		}

		want = fmt.Sprintf("promhttp_metric_handler_requests_total{code=\"200\"} %d\n", i+1)
		if got := body; !strings.Contains(got, want) {
			t.Errorf("got body %q, does not contain %q, err: %v", got, want, err)
		}
	}
}

func TestNegotiateEncodingWriter(t *testing.T) {
	var defaultCompressions []string

	for _, comp := range defaultCompressionFormats {
		defaultCompressions = append(defaultCompressions, string(comp))
	}

	testCases := []struct {
		name                string
		offeredCompressions []string
		acceptEncoding      string
		expectedCompression string
		err                 error
	}{
		{
			name:                "test without compression enabled",
			offeredCompressions: []string{},
			acceptEncoding:      "",
			expectedCompression: "identity",
			err:                 nil,
		},
		{
			name:                "test with compression enabled with empty accept-encoding header",
			offeredCompressions: defaultCompressions,
			acceptEncoding:      "",
			expectedCompression: "identity",
			err:                 nil,
		},
		{
			name:                "test with gzip compression requested",
			offeredCompressions: defaultCompressions,
			acceptEncoding:      "gzip",
			expectedCompression: "gzip",
			err:                 nil,
		},
		{
			name:                "test with gzip, zstd compression requested",
			offeredCompressions: defaultCompressions,
			acceptEncoding:      "gzip,zstd",
			expectedCompression: "gzip",
			err:                 nil,
		},
		{
			name:                "test with zstd, gzip compression requested",
			offeredCompressions: defaultCompressions,
			acceptEncoding:      "zstd,gzip",
			expectedCompression: "gzip",
			err:                 nil,
		},
	}

	for _, test := range testCases {
		request, _ := http.NewRequest("GET", "/", nil)
		request.Header.Add(acceptEncodingHeader, test.acceptEncoding)
		rr := httptest.NewRecorder()
		_, encodingHeader, _, err := negotiateEncodingWriter(request, rr, test.offeredCompressions)

		if !errors.Is(err, test.err) {
			t.Errorf("got error: %v, expected: %v", err, test.err)
		}

		if encodingHeader != test.expectedCompression {
			t.Errorf("got different compression type: %v, expected: %v", encodingHeader, test.expectedCompression)
		}
	}
}

func BenchmarkCompression(b *testing.B) {
	benchmarks := []struct {
		name            string
		compressionType string
	}{
		{
			name:            "test with gzip compression",
			compressionType: "gzip",
		},
		{
			name:            "test with zstd compression",
			compressionType: "zstd",
		},
		{
			name:            "test with no compression",
			compressionType: "identity",
		},
	}
	sizes := []struct {
		name         string
		metricCount  int
		labelCount   int
		labelLength  int
		metricLength int
	}{
		{
			name:         "small",
			metricCount:  50,
			labelCount:   5,
			labelLength:  5,
			metricLength: 5,
		},
		{
			name:         "medium",
			metricCount:  500,
			labelCount:   10,
			labelLength:  5,
			metricLength: 10,
		},
		{
			name:         "large",
			metricCount:  5000,
			labelCount:   10,
			labelLength:  5,
			metricLength: 10,
		},
		{
			name:         "extra-large",
			metricCount:  50000,
			labelCount:   20,
			labelLength:  5,
			metricLength: 10,
		},
	}

	for _, size := range sizes {
		reg := prometheus.NewRegistry()
		handler := HandlerFor(reg, HandlerOpts{})

		// Generate Metrics
		// Original source: https://github.com/prometheus-community/avalanche/blob/main/metrics/serve.go
		labelKeys := make([]string, size.labelCount)
		for idx := 0; idx < size.labelCount; idx++ {
			labelKeys[idx] = fmt.Sprintf("label_key_%s_%v", strings.Repeat("k", size.labelLength), idx)
		}
		labelValues := make([]string, size.labelCount)
		for idx := 0; idx < size.labelCount; idx++ {
			labelValues[idx] = fmt.Sprintf("label_val_%s_%v", strings.Repeat("v", size.labelLength), idx)
		}
		metrics := make([]*prometheus.GaugeVec, size.metricCount)
		for idx := 0; idx < size.metricCount; idx++ {
			gauge := prometheus.NewGaugeVec(prometheus.GaugeOpts{
				Name: fmt.Sprintf("avalanche_metric_%s_%v_%v", strings.Repeat("m", size.metricLength), 0, idx),
				Help: "A tasty metric morsel",
			}, append([]string{"series_id", "cycle_id"}, labelKeys...))
			reg.MustRegister(gauge)
			metrics[idx] = gauge
		}

		for _, benchmark := range benchmarks {
			b.Run(benchmark.name+"_"+size.name, func(b *testing.B) {
				for i := 0; i < b.N; i++ {
					writer := httptest.NewRecorder()
					request, _ := http.NewRequest("GET", "/", nil)
					request.Header.Add(acceptEncodingHeader, benchmark.compressionType)
					handler.ServeHTTP(writer, request)
				}
			})
		}
	}
}
