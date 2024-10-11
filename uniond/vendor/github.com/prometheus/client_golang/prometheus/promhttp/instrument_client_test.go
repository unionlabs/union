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
	"log"
	"net/http"
	"net/http/httptest"
	"reflect"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/testutil"

	dto "github.com/prometheus/client_model/go"
	"google.golang.org/protobuf/proto"
)

func makeInstrumentedClient(opts ...Option) (*http.Client, *prometheus.Registry) {
	client := http.DefaultClient
	client.Timeout = 1 * time.Second

	reg := prometheus.NewRegistry()

	inFlightGauge := prometheus.NewGauge(prometheus.GaugeOpts{
		Name: "client_in_flight_requests",
		Help: "A gauge of in-flight requests for the wrapped client.",
	})

	counter := prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "client_api_requests_total",
			Help: "A counter for requests from the wrapped client.",
		},
		[]string{"code", "method"},
	)

	dnsLatencyVec := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "dns_duration_seconds",
			Help:    "Trace dns latency histogram.",
			Buckets: []float64{.005, .01, .025, .05},
		},
		[]string{"event"},
	)

	tlsLatencyVec := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "tls_duration_seconds",
			Help:    "Trace tls latency histogram.",
			Buckets: []float64{.05, .1, .25, .5},
		},
		[]string{"event"},
	)

	histVec := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "request_duration_seconds",
			Help:    "A histogram of request latencies.",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"method"},
	)

	reg.MustRegister(counter, tlsLatencyVec, dnsLatencyVec, histVec, inFlightGauge)

	trace := &InstrumentTrace{
		DNSStart: func(t float64) {
			dnsLatencyVec.WithLabelValues("dns_start").Observe(t)
		},
		DNSDone: func(t float64) {
			dnsLatencyVec.WithLabelValues("dns_done").Observe(t)
		},
		TLSHandshakeStart: func(t float64) {
			tlsLatencyVec.WithLabelValues("tls_handshake_start").Observe(t)
		},
		TLSHandshakeDone: func(t float64) {
			tlsLatencyVec.WithLabelValues("tls_handshake_done").Observe(t)
		},
	}

	client.Transport = InstrumentRoundTripperInFlight(inFlightGauge,
		InstrumentRoundTripperCounter(counter,
			InstrumentRoundTripperTrace(trace,
				InstrumentRoundTripperDuration(histVec, http.DefaultTransport, opts...),
			),
			opts...),
	)
	return client, reg
}

func labelsToLabelPair(l prometheus.Labels) []*dto.LabelPair {
	ret := make([]*dto.LabelPair, 0, len(l))
	for k, v := range l {
		ret = append(ret, &dto.LabelPair{Name: proto.String(k), Value: proto.String(v)})
	}
	sort.Slice(ret, func(i, j int) bool {
		return *ret[i].Name < *ret[j].Name
	})
	return ret
}

func assetMetricAndExemplars(
	t *testing.T,
	reg *prometheus.Registry,
	expectedNumMetrics int,
	expectedExemplar []*dto.LabelPair,
) {
	t.Helper()

	mfs, err := reg.Gather()
	if err != nil {
		t.Fatal(err)
	}
	if want, got := expectedNumMetrics, len(mfs); want != got {
		t.Fatalf("unexpected number of metric families gathered, want %d, got %d", want, got)
	}

	for _, mf := range mfs {
		if len(mf.Metric) == 0 {
			t.Errorf("metric family %s must not be empty", mf.GetName())
		}
		for _, m := range mf.GetMetric() {
			if c := m.GetCounter(); c != nil {
				if len(expectedExemplar) == 0 {
					if c.Exemplar != nil {
						t.Errorf("expected no exemplar on the counter %v%v, got %v", mf.GetName(), m.Label, c.Exemplar.String())
					}
					continue
				}

				if c.Exemplar == nil {
					t.Errorf("expected exemplar %v on the counter %v%v, got none", expectedExemplar, mf.GetName(), m.Label)
					continue
				}
				if got := c.Exemplar.Label; !reflect.DeepEqual(expectedExemplar, got) {
					t.Errorf("expected exemplar %v on the counter %v%v, got %v", expectedExemplar, mf.GetName(), m.Label, got)
				}
				continue
			}
			if h := m.GetHistogram(); h != nil {
				found := false
				for _, b := range h.GetBucket() {
					if len(expectedExemplar) == 0 {
						if b.Exemplar != nil {
							t.Errorf("expected no exemplar on histogram %v%v bkt %v, got %v", mf.GetName(), m.Label, b.GetUpperBound(), b.Exemplar.String())
						}
						continue
					}

					if b.Exemplar == nil {
						continue
					}
					if got := b.Exemplar.Label; !reflect.DeepEqual(expectedExemplar, got) {
						t.Errorf("expected exemplar %v on the histogram %v%v on bkt %v, got %v", expectedExemplar, mf.GetName(), m.Label, b.GetUpperBound(), got)
						continue
					}
					found = true
					break
				}

				if len(expectedExemplar) > 0 && !found {
					t.Errorf("expected exemplar %v on at least one bucket of the histogram %v%v, got none", expectedExemplar, mf.GetName(), m.Label)
				}
			}
		}
	}
}

func TestClientMiddlewareAPI(t *testing.T) {
	client, reg := makeInstrumentedClient()
	backend := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer backend.Close()

	resp, err := client.Get(backend.URL)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()

	assetMetricAndExemplars(t, reg, 3, nil)
}

func TestClientMiddlewareAPI_WithExemplars(t *testing.T) {
	exemplar := prometheus.Labels{"traceID": "example situation observed by this metric"}

	client, reg := makeInstrumentedClient(WithExemplarFromContext(func(_ context.Context) prometheus.Labels { return exemplar }))
	backend := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer backend.Close()

	resp, err := client.Get(backend.URL)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()

	assetMetricAndExemplars(t, reg, 3, labelsToLabelPair(exemplar))
}

func TestClientMiddlewareAPI_WithRequestContext(t *testing.T) {
	client, reg := makeInstrumentedClient()
	backend := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer backend.Close()

	req, err := http.NewRequest("GET", backend.URL, nil)
	if err != nil {
		t.Fatalf("%v", err)
	}

	// Set a context with a long timeout.
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()
	req = req.WithContext(ctx)

	resp, err := client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()

	mfs, err := reg.Gather()
	if err != nil {
		t.Fatal(err)
	}
	if want, got := 3, len(mfs); want != got {
		t.Fatalf("unexpected number of metric families gathered, want %d, got %d", want, got)
	}
	for _, mf := range mfs {
		if len(mf.Metric) == 0 {
			t.Errorf("metric family %s must not be empty", mf.GetName())
		}
	}

	// make sure counters aren't double-incremented (see #1117)
	expected := `
		# HELP client_api_requests_total A counter for requests from the wrapped client.
		# TYPE client_api_requests_total counter
		client_api_requests_total{code="200",method="get"} 1
	`

	if err := testutil.GatherAndCompare(reg, strings.NewReader(expected),
		"client_api_requests_total",
	); err != nil {
		t.Fatal(err)
	}
}

func TestClientMiddlewareAPIWithRequestContextTimeout(t *testing.T) {
	client, _ := makeInstrumentedClient()

	// Slow testserver responding in 100ms.
	backend := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(100 * time.Millisecond)
		w.WriteHeader(http.StatusOK)
	}))
	defer backend.Close()

	req, err := http.NewRequest("GET", backend.URL, nil)
	if err != nil {
		t.Fatalf("%v", err)
	}

	// Set a context with a short timeout.
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Millisecond)
	defer cancel()
	req = req.WithContext(ctx)

	_, err = client.Do(req)
	if err == nil {
		t.Fatal("did not get timeout error")
	}
	expectedMsg := "context deadline exceeded"
	if !strings.Contains(err.Error(), expectedMsg) {
		t.Fatalf("unexpected error: %q, expect error: %q", err.Error(), expectedMsg)
	}
}

func ExampleInstrumentRoundTripperDuration() {
	client := http.DefaultClient
	client.Timeout = 1 * time.Second

	inFlightGauge := prometheus.NewGauge(prometheus.GaugeOpts{
		Name: "client_in_flight_requests",
		Help: "A gauge of in-flight requests for the wrapped client.",
	})

	counter := prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "client_api_requests_total",
			Help: "A counter for requests from the wrapped client.",
		},
		[]string{"code", "method"},
	)

	// dnsLatencyVec uses custom buckets based on expected dns durations.
	// It has an instance label "event", which is set in the
	// DNSStart and DNSDonehook functions defined in the
	// InstrumentTrace struct below.
	dnsLatencyVec := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "dns_duration_seconds",
			Help:    "Trace dns latency histogram.",
			Buckets: []float64{.005, .01, .025, .05},
		},
		[]string{"event"},
	)

	// tlsLatencyVec uses custom buckets based on expected tls durations.
	// It has an instance label "event", which is set in the
	// TLSHandshakeStart and TLSHandshakeDone hook functions defined in the
	// InstrumentTrace struct below.
	tlsLatencyVec := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "tls_duration_seconds",
			Help:    "Trace tls latency histogram.",
			Buckets: []float64{.05, .1, .25, .5},
		},
		[]string{"event"},
	)

	// histVec has no labels, making it a zero-dimensional ObserverVec.
	histVec := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "request_duration_seconds",
			Help:    "A histogram of request latencies.",
			Buckets: prometheus.DefBuckets,
		},
		[]string{},
	)

	// Register all of the metrics in the standard registry.
	prometheus.MustRegister(counter, tlsLatencyVec, dnsLatencyVec, histVec, inFlightGauge)

	// Define functions for the available httptrace.ClientTrace hook
	// functions that we want to instrument.
	trace := &InstrumentTrace{
		DNSStart: func(t float64) {
			dnsLatencyVec.WithLabelValues("dns_start").Observe(t)
		},
		DNSDone: func(t float64) {
			dnsLatencyVec.WithLabelValues("dns_done").Observe(t)
		},
		TLSHandshakeStart: func(t float64) {
			tlsLatencyVec.WithLabelValues("tls_handshake_start").Observe(t)
		},
		TLSHandshakeDone: func(t float64) {
			tlsLatencyVec.WithLabelValues("tls_handshake_done").Observe(t)
		},
	}

	// Wrap the default RoundTripper with middleware.
	roundTripper := InstrumentRoundTripperInFlight(inFlightGauge,
		InstrumentRoundTripperCounter(counter,
			InstrumentRoundTripperTrace(trace,
				InstrumentRoundTripperDuration(histVec, http.DefaultTransport),
			),
		),
	)

	// Set the RoundTripper on our client.
	client.Transport = roundTripper

	resp, err := client.Get("http://google.com")
	if err != nil {
		log.Printf("error: %v", err)
	}
	defer resp.Body.Close()
}
