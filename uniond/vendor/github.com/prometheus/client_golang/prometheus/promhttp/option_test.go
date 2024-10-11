// Copyright 2022 The Prometheus Authors
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

	"github.com/prometheus/client_golang/prometheus"
)

type key int

const (
	CtxResolverKey key = iota
)

func ExampleInstrumentHandlerWithExtraMethods() {
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

	// Create the handlers that will be wrapped by the middleware.
	pullHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("Pull"))
	})

	// Specify additional HTTP methods to be added to the label allow list.
	opts := WithExtraMethods("CUSTOM_METHOD")

	// Instrument the handlers with all the metrics, injecting the "handler"
	// label by currying.
	pullChain := InstrumentHandlerDuration(duration.MustCurryWith(prometheus.Labels{"handler": "pull"}),
		InstrumentHandlerCounter(counter, pullHandler, opts),
		opts,
	)

	http.Handle("/metrics", Handler())
	http.Handle("/pull", pullChain)

	if err := http.ListenAndServe(":3000", nil); err != nil {
		log.Fatal(err)
	}
}

func ExampleInstrumentHandlerWithLabelResolver() {
	counter := prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "api_requests_total",
			Help: "A counter for requests to the wrapped handler.",
		},
		[]string{"code", "method", "myheader"},
	)

	// duration is partitioned by the HTTP method, handler and request header
	// value. It uses custom buckets based on the expected request duration.
	// Beware to not have too high cardinality on the values of header. You
	// always should sanitize external inputs.
	duration := prometheus.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "request_duration_seconds",
			Help:    "A histogram of latencies for requests.",
			Buckets: []float64{.25, .5, 1, 2.5, 5, 10},
		},
		[]string{"handler", "method", "myheader"},
	)

	// Create the handlers that will be wrapped by the middleware.
	pullHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("Pull"))
	})

	// Specify additional HTTP methods to be added to the label allow list.
	opts := WithLabelFromCtx("myheader",
		func(ctx context.Context) string {
			return ctx.Value(CtxResolverKey).(string)
		},
	)

	// Instrument the handlers with all the metrics, injecting the "handler"
	// label by currying.
	pullChain := InstrumentHandlerDuration(duration.MustCurryWith(prometheus.Labels{"handler": "pull"}),
		InstrumentHandlerCounter(counter, pullHandler, opts),
		opts,
	)

	middleware := func(next http.HandlerFunc) http.HandlerFunc {
		return func(w http.ResponseWriter, r *http.Request) {
			ctx := context.WithValue(r.Context(), CtxResolverKey, r.Header.Get("x-my-header"))

			next(w, r.WithContext(ctx))
		}
	}

	http.Handle("/metrics", Handler())
	http.Handle("/pull", middleware(pullChain))

	if err := http.ListenAndServe(":3000", nil); err != nil {
		log.Fatal(err)
	}
}
