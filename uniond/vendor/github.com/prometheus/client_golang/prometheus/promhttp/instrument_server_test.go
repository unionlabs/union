// Copyright 2017 The Prometheus Authors
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
	"context"
	"io"
	"log"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/prometheus/client_golang/prometheus"
)

func TestLabelCheck(t *testing.T) {
	scenarios := map[string]struct {
		metricName    string // Defaults to "c".
		varLabels     []string
		constLabels   []string
		curriedLabels []string
		dynamicLabels []string
		ok            bool
	}{
		"empty": {
			varLabels:     []string{},
			constLabels:   []string{},
			curriedLabels: []string{},
			ok:            true,
		},
		"code as single var label": {
			varLabels:     []string{"code"},
			constLabels:   []string{},
			curriedLabels: []string{},
			ok:            true,
		},
		"method as single var label": {
			varLabels:     []string{"method"},
			constLabels:   []string{},
			curriedLabels: []string{},
			ok:            true,
		},
		"code and method as var labels": {
			varLabels:     []string{"method", "code"},
			constLabels:   []string{},
			curriedLabels: []string{},
			ok:            true,
		},
		"valid case with all labels used": {
			varLabels:     []string{"code", "method"},
			constLabels:   []string{"foo", "bar"},
			curriedLabels: []string{"dings", "bums"},
			dynamicLabels: []string{"dyn", "amics"},
			ok:            true,
		},
		"all labels used with an invalid const label name": {
			varLabels:     []string{"code", "method"},
			constLabels:   []string{"in-valid", "bar"},
			curriedLabels: []string{"dings", "bums"},
			dynamicLabels: []string{"dyn", "amics"},
			ok:            false,
		},
		"unsupported var label": {
			varLabels:     []string{"foo"},
			constLabels:   []string{},
			curriedLabels: []string{},
			ok:            false,
		},
		"mixed var labels": {
			varLabels:     []string{"method", "foo", "code"},
			constLabels:   []string{},
			curriedLabels: []string{},
			ok:            false,
		},
		"unsupported var label but curried": {
			varLabels:     []string{},
			constLabels:   []string{},
			curriedLabels: []string{"foo"},
			ok:            true,
		},
		"mixed var labels but unsupported curried": {
			varLabels:     []string{"code", "method"},
			constLabels:   []string{},
			curriedLabels: []string{"foo"},
			ok:            true,
		},
		"supported label as const and curry": {
			varLabels:     []string{},
			constLabels:   []string{"code"},
			curriedLabels: []string{"method"},
			ok:            true,
		},
		"supported label as const and dynamic": {
			varLabels:     []string{},
			constLabels:   []string{"code"},
			dynamicLabels: []string{"method"},
			ok:            true,
		},
		"supported label as curried and dynamic": {
			varLabels:     []string{},
			curriedLabels: []string{"code"},
			dynamicLabels: []string{"method"},
			ok:            true,
		},
		"supported label as const and curry with unsupported as var": {
			varLabels:     []string{"foo"},
			constLabels:   []string{"code"},
			curriedLabels: []string{"method"},
			ok:            false,
		},
		"invalid name and otherwise empty": {
			metricName:    "in-valid",
			varLabels:     []string{},
			constLabels:   []string{},
			curriedLabels: []string{},
			ok:            false,
		},
		"invalid name with all the otherwise valid labels": {
			metricName:    "in-valid",
			varLabels:     []string{"code", "method"},
			constLabels:   []string{"foo", "bar"},
			curriedLabels: []string{"dings", "bums"},
			dynamicLabels: []string{"dyn", "amics"},
			ok:            false,
		},
	}

	for name, sc := range scenarios {
		t.Run(name, func(t *testing.T) {
			metricName := sc.metricName
			if metricName == "" {
				metricName = "c"
			}
			constLabels := prometheus.Labels{}
			for _, l := range sc.constLabels {
				constLabels[l] = "dummy"
			}
			labelNames := append(append(sc.varLabels, sc.curriedLabels...), sc.dynamicLabels...)
			c := prometheus.V2.NewCounterVec(
				prometheus.CounterVecOpts{
					CounterOpts: prometheus.CounterOpts{
						Name:        metricName,
						Help:        "c help",
						ConstLabels: constLabels,
					},
					VariableLabels: prometheus.UnconstrainedLabels(labelNames),
				},
			)
			o := prometheus.ObserverVec(prometheus.V2.NewHistogramVec(
				prometheus.HistogramVecOpts{
					HistogramOpts: prometheus.HistogramOpts{
						Name:        metricName,
						Help:        "c help",
						ConstLabels: constLabels,
					},
					VariableLabels: prometheus.UnconstrainedLabels(labelNames),
				},
			))
			for _, l := range sc.curriedLabels {
				c = c.MustCurryWith(prometheus.Labels{l: "dummy"})
				o = o.MustCurryWith(prometheus.Labels{l: "dummy"})
			}
			opts := []Option{}
			for _, l := range sc.dynamicLabels {
				opts = append(opts, WithLabelFromCtx(l,
					func(_ context.Context) string {
						return "foo"
					},
				))
			}

			func() {
				defer func() {
					if err := recover(); err != nil {
						if sc.ok {
							t.Error("unexpected panic:", err)
						}
					} else if !sc.ok {
						t.Error("expected panic")
					}
				}()
				InstrumentHandlerCounter(c, nil, opts...)
			}()
			func() {
				defer func() {
					if err := recover(); err != nil {
						if sc.ok {
							t.Error("unexpected panic:", err)
						}
					} else if !sc.ok {
						t.Error("expected panic")
					}
				}()
				InstrumentHandlerDuration(o, nil, opts...)
			}()
			if sc.ok {
				// Test if wantCode and wantMethod were detected correctly.
				var wantCode, wantMethod bool
				for _, l := range sc.varLabels {
					if l == "code" {
						wantCode = true
					}
					if l == "method" {
						wantMethod = true
					}
				}
				// Curry the dynamic labels since this is done normally behind the scenes for the check
				for _, l := range sc.dynamicLabels {
					c = c.MustCurryWith(prometheus.Labels{l: "dummy"})
					o = o.MustCurryWith(prometheus.Labels{l: "dummy"})
				}
				gotCode, gotMethod := checkLabels(c)
				if gotCode != wantCode {
					t.Errorf("wanted code=%t for counter, got code=%t", wantCode, gotCode)
				}
				if gotMethod != wantMethod {
					t.Errorf("wanted method=%t for counter, got method=%t", wantMethod, gotMethod)
				}
				gotCode, gotMethod = checkLabels(o)
				if gotCode != wantCode {
					t.Errorf("wanted code=%t for observer, got code=%t", wantCode, gotCode)
				}
				if gotMethod != wantMethod {
					t.Errorf("wanted method=%t for observer, got method=%t", wantMethod, gotMethod)
				}
			}
		})
	}
}

func TestLabels(t *testing.T) {
	scenarios := map[string]struct {
		varLabels    []string
		reqMethod    string
		respStatus   int
		extraMethods []string
		wantLabels   prometheus.Labels
		ok           bool
	}{
		"empty": {
			varLabels:  []string{},
			wantLabels: prometheus.Labels{},
			reqMethod:  "GET",
			respStatus: 200,
			ok:         true,
		},
		"code as single var label": {
			varLabels:  []string{"code"},
			reqMethod:  "GET",
			respStatus: 200,
			wantLabels: prometheus.Labels{"code": "200"},
			ok:         true,
		},
		"code as single var label and out-of-range code": {
			varLabels:  []string{"code"},
			reqMethod:  "GET",
			respStatus: 99,
			wantLabels: prometheus.Labels{"code": "unknown"},
			ok:         true,
		},
		"code as single var label and in-range but unrecognized code": {
			varLabels:  []string{"code"},
			reqMethod:  "GET",
			respStatus: 308,
			wantLabels: prometheus.Labels{"code": "308"},
			ok:         true,
		},
		"method as single var label": {
			varLabels:  []string{"method"},
			reqMethod:  "GET",
			respStatus: 200,
			wantLabels: prometheus.Labels{"method": "get"},
			ok:         true,
		},
		"method as single var label and unknown method": {
			varLabels:  []string{"method"},
			reqMethod:  "CUSTOM_METHOD",
			respStatus: 200,
			wantLabels: prometheus.Labels{"method": "unknown"},
			ok:         true,
		},
		"code and method as var labels": {
			varLabels:  []string{"method", "code"},
			reqMethod:  "GET",
			respStatus: 200,
			wantLabels: prometheus.Labels{"method": "get", "code": "200"},
			ok:         true,
		},
		"method as single var label with extra methods specified": {
			varLabels:    []string{"method"},
			reqMethod:    "CUSTOM_METHOD",
			respStatus:   200,
			extraMethods: []string{"CUSTOM_METHOD", "CUSTOM_METHOD_1"},
			wantLabels:   prometheus.Labels{"method": "custom_method"},
			ok:           true,
		},
		"all labels used with an unknown method and out-of-range code": {
			varLabels:  []string{"code", "method"},
			reqMethod:  "CUSTOM_METHOD",
			respStatus: 99,
			wantLabels: prometheus.Labels{"method": "unknown", "code": "unknown"},
			ok:         false,
		},
	}
	checkLabels := func(labels []string) (gotCode, gotMethod bool) {
		for _, label := range labels {
			switch label {
			case "code":
				gotCode = true
			case "method":
				gotMethod = true
			default:
				panic("metric partitioned with non-supported labels for this test")
			}
		}
		return
	}
	equalLabels := func(gotLabels, wantLabels prometheus.Labels) bool {
		if len(gotLabels) != len(wantLabels) {
			return false
		}
		for ln, lv := range gotLabels {
			olv, ok := wantLabels[ln]
			if !ok {
				return false
			}
			if olv != lv {
				return false
			}
		}
		return true
	}

	for name, sc := range scenarios {
		t.Run(name, func(t *testing.T) {
			if sc.ok {
				gotCode, gotMethod := checkLabels(sc.varLabels)
				gotLabels := labels(gotCode, gotMethod, sc.reqMethod, sc.respStatus, sc.extraMethods...)
				if !equalLabels(gotLabels, sc.wantLabels) {
					t.Errorf("wanted labels=%v for counter, got code=%v", sc.wantLabels, gotLabels)
				}
			}
		})
	}
}

func makeInstrumentedHandler(handler http.HandlerFunc, opts ...Option) (http.Handler, *prometheus.Registry) {
	reg := prometheus.NewRegistry()

	inFlightGauge := prometheus.NewGauge(prometheus.GaugeOpts{
		Name: "in_flight_requests",
		Help: "A gauge of requests currently being served by the wrapped handler.",
	})

	counter := prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "api_requests_total",
			Help: "A counter for requests to the wrapped handler.",
		},
		[]string{"code", "method"},
	)

	histVec := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:        "response_duration_seconds",
			Help:        "A histogram of request latencies.",
			Buckets:     prometheus.DefBuckets,
			ConstLabels: prometheus.Labels{"handler": "api"},
		},
		[]string{"method"},
	)

	writeHeaderVec := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:        "write_header_duration_seconds",
			Help:        "A histogram of time to first write latencies.",
			Buckets:     prometheus.DefBuckets,
			ConstLabels: prometheus.Labels{"handler": "api"},
		},
		[]string{},
	)

	responseSize := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "push_request_size_bytes",
			Help:    "A histogram of request sizes for requests.",
			Buckets: []float64{200, 500, 900, 1500},
		},
		[]string{},
	)

	reg.MustRegister(inFlightGauge, counter, histVec, responseSize, writeHeaderVec)

	return InstrumentHandlerInFlight(inFlightGauge,
		InstrumentHandlerCounter(counter,
			InstrumentHandlerDuration(histVec,
				InstrumentHandlerTimeToWriteHeader(writeHeaderVec,
					InstrumentHandlerResponseSize(responseSize, handler, opts...),
					opts...),
				opts...),
			opts...),
	), reg
}

func TestMiddlewareAPI(t *testing.T) {
	chain, reg := makeInstrumentedHandler(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte("OK"))
	})

	r, _ := http.NewRequest("GET", "www.example.com", nil)
	w := httptest.NewRecorder()
	chain.ServeHTTP(w, r)

	assetMetricAndExemplars(t, reg, 5, nil)
}

func TestMiddlewareAPI_WithExemplars(t *testing.T) {
	exemplar := prometheus.Labels{"traceID": "example situation observed by this metric"}

	chain, reg := makeInstrumentedHandler(func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte("OK"))
	}, WithExemplarFromContext(func(_ context.Context) prometheus.Labels { return exemplar }))

	r, _ := http.NewRequest("GET", "www.example.com", nil)
	w := httptest.NewRecorder()
	chain.ServeHTTP(w, r)

	assetMetricAndExemplars(t, reg, 5, labelsToLabelPair(exemplar))
}

func TestInstrumentTimeToFirstWrite(t *testing.T) {
	var i int
	dobs := &responseWriterDelegator{
		ResponseWriter: httptest.NewRecorder(),
		observeWriteHeader: func(status int) {
			i = status
		},
	}
	d := newDelegator(dobs, nil)

	d.WriteHeader(http.StatusOK)

	if i != http.StatusOK {
		t.Fatalf("failed to execute observeWriteHeader")
	}
}

// testResponseWriter is an http.ResponseWriter that also implements
// http.CloseNotifier, http.Flusher, and io.ReaderFrom.
type testResponseWriter struct {
	closeNotifyCalled, flushCalled, readFromCalled bool
}

func (t *testResponseWriter) Header() http.Header       { return nil }
func (t *testResponseWriter) Write([]byte) (int, error) { return 0, nil }
func (t *testResponseWriter) WriteHeader(int)           {}
func (t *testResponseWriter) CloseNotify() <-chan bool {
	t.closeNotifyCalled = true
	return nil
}
func (t *testResponseWriter) Flush() { t.flushCalled = true }
func (t *testResponseWriter) ReadFrom(io.Reader) (int64, error) {
	t.readFromCalled = true
	return 0, nil
}

// testFlusher is an http.ResponseWriter that also implements http.Flusher.
type testFlusher struct {
	flushCalled bool
}

func (t *testFlusher) Header() http.Header       { return nil }
func (t *testFlusher) Write([]byte) (int, error) { return 0, nil }
func (t *testFlusher) WriteHeader(int)           {}
func (t *testFlusher) Flush()                    { t.flushCalled = true }

func TestInterfaceUpgrade(t *testing.T) {
	w := &testResponseWriter{}
	d := newDelegator(w, nil)
	//nolint:staticcheck // Ignore SA1019. http.CloseNotifier is deprecated but we keep it here to not break existing users.
	d.(http.CloseNotifier).CloseNotify()
	if !w.closeNotifyCalled {
		t.Error("CloseNotify not called")
	}
	d.(http.Flusher).Flush()
	if !w.flushCalled {
		t.Error("Flush not called")
	}
	d.(io.ReaderFrom).ReadFrom(nil)
	if !w.readFromCalled {
		t.Error("ReadFrom not called")
	}
	if _, ok := d.(http.Hijacker); ok {
		t.Error("delegator unexpectedly implements http.Hijacker")
	}

	f := &testFlusher{}
	d = newDelegator(f, nil)
	//nolint:staticcheck // Ignore SA1019. http.CloseNotifier is deprecated but we keep it here to not break existing users.
	if _, ok := d.(http.CloseNotifier); ok {
		t.Error("delegator unexpectedly implements http.CloseNotifier")
	}
	d.(http.Flusher).Flush()
	if !w.flushCalled {
		t.Error("Flush not called")
	}
	if _, ok := d.(io.ReaderFrom); ok {
		t.Error("delegator unexpectedly implements io.ReaderFrom")
	}
	if _, ok := d.(http.Hijacker); ok {
		t.Error("delegator unexpectedly implements http.Hijacker")
	}
}

func ExampleInstrumentHandlerDuration() {
	inFlightGauge := prometheus.NewGauge(prometheus.GaugeOpts{
		Name: "in_flight_requests",
		Help: "A gauge of requests currently being served by the wrapped handler.",
	})

	counter := prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "api_requests_total",
			Help: "A counter for requests to the wrapped handler.",
		},
		[]string{"code", "method"},
	)

	// duration is partitioned by the HTTP method and handler. It uses custom
	// buckets based on the expected request duration.
	duration := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "request_duration_seconds",
			Help:    "A histogram of latencies for requests.",
			Buckets: []float64{.25, .5, 1, 2.5, 5, 10},
		},
		[]string{"handler", "method"},
	)

	// responseSize has no labels, making it a zero-dimensional
	// ObserverVec.
	responseSize := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "response_size_bytes",
			Help:    "A histogram of response sizes for requests.",
			Buckets: []float64{200, 500, 900, 1500},
		},
		[]string{},
	)

	// Create the handlers that will be wrapped by the middleware.
	pushHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("Push"))
	})
	pullHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("Pull"))
	})

	// Register all of the metrics in the standard registry.
	prometheus.MustRegister(inFlightGauge, counter, duration, responseSize)

	// Instrument the handlers with all the metrics, injecting the "handler"
	// label by currying.
	pushChain := InstrumentHandlerInFlight(inFlightGauge,
		InstrumentHandlerDuration(duration.MustCurryWith(prometheus.Labels{"handler": "push"}),
			InstrumentHandlerCounter(counter,
				InstrumentHandlerResponseSize(responseSize, pushHandler),
			),
		),
	)
	pullChain := InstrumentHandlerInFlight(inFlightGauge,
		InstrumentHandlerDuration(duration.MustCurryWith(prometheus.Labels{"handler": "pull"}),
			InstrumentHandlerCounter(counter,
				InstrumentHandlerResponseSize(responseSize, pullHandler),
			),
		),
	)

	http.Handle("/metrics", Handler())
	http.Handle("/push", pushChain)
	http.Handle("/pull", pullChain)

	if err := http.ListenAndServe(":3000", nil); err != nil {
		log.Fatal(err)
	}
}
