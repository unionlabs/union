// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package propagation_test

import (
	"context"
	"net/http"
	"testing"

	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"
)

func BenchmarkInject(b *testing.B) {
	var t propagation.TraceContext

	injectSubBenchmarks(b, func(ctx context.Context, b *testing.B) {
		h := http.Header{}
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			t.Inject(ctx, propagation.HeaderCarrier(h))
		}
	})
}

func injectSubBenchmarks(b *testing.B, fn func(context.Context, *testing.B)) {
	b.Run("SampledSpanContext", func(b *testing.B) {
		b.ReportAllocs()
		sc := trace.NewSpanContext(trace.SpanContextConfig{
			TraceID:    traceID,
			SpanID:     spanID,
			TraceFlags: trace.FlagsSampled,
		})
		ctx := trace.ContextWithRemoteSpanContext(context.Background(), sc)
		fn(ctx, b)
	})

	b.Run("WithoutSpanContext", func(b *testing.B) {
		b.ReportAllocs()
		ctx := context.Background()
		fn(ctx, b)
	})
}

func BenchmarkExtract(b *testing.B) {
	extractSubBenchmarks(b, func(b *testing.B, req *http.Request) {
		var propagator propagation.TraceContext
		ctx := context.Background()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			propagator.Extract(ctx, propagation.HeaderCarrier(req.Header))
		}
	})
}

func extractSubBenchmarks(b *testing.B, fn func(*testing.B, *http.Request)) {
	b.Run("Sampled", func(b *testing.B) {
		req, _ := http.NewRequest("GET", "http://example.com", nil)
		req.Header.Set("traceparent", "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
		b.ReportAllocs()

		fn(b, req)
	})

	b.Run("BogusVersion", func(b *testing.B) {
		req, _ := http.NewRequest("GET", "http://example.com", nil)
		req.Header.Set("traceparent", "qw-00000000000000000000000000000000-0000000000000000-01")
		b.ReportAllocs()
		fn(b, req)
	})

	b.Run("FutureAdditionalData", func(b *testing.B) {
		req, _ := http.NewRequest("GET", "http://example.com", nil)
		req.Header.Set("traceparent", "02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09-XYZxsf09")
		b.ReportAllocs()
		fn(b, req)
	})
}
