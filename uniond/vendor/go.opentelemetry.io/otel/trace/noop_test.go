// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package trace

import (
	"context"
	"testing"
)

func TestNewNoopTracerProvider(t *testing.T) {
	got, want := NewNoopTracerProvider(), noopTracerProvider{}
	if got != want {
		t.Errorf("NewNoopTracerProvider() returned %#v, want %#v", got, want)
	}
}

func TestNoopTracerProviderTracer(t *testing.T) {
	tp := NewNoopTracerProvider()
	got, want := tp.Tracer(""), noopTracer{}
	if got != want {
		t.Errorf("noopTracerProvider.Tracer() returned %#v, want %#v", got, want)
	}
}

func TestNoopTracerStart(t *testing.T) {
	ctx := context.Background()
	tracer := NewNoopTracerProvider().Tracer("test instrumentation")

	var span Span
	ctx, span = tracer.Start(ctx, "span name")
	got, ok := span.(noopSpan)
	if !ok {
		t.Fatalf("noopTracer.Start() returned a non-noopSpan: %#v", span)
	}
	want := noopSpan{}
	if got != want {
		t.Errorf("noopTracer.Start() returned %#v, want %#v", got, want)
	}
	got, ok = SpanFromContext(ctx).(noopSpan)
	if !ok {
		t.Fatal("noopTracer.Start() did not set span as current in returned context")
	}
	if got != want {
		t.Errorf("noopTracer.Start() current span in returned context set to %#v, want %#v", got, want)
	}
}

func TestNoopSpan(t *testing.T) {
	tracer := NewNoopTracerProvider().Tracer("test instrumentation")
	_, s := tracer.Start(context.Background(), "test span")
	span := s.(noopSpan)

	if got, want := span.SpanContext(), (SpanContext{}); !assertSpanContextEqual(got, want) {
		t.Errorf("span.SpanContext() returned %#v, want %#v", got, want)
	}

	if got, want := span.IsRecording(), false; got != want {
		t.Errorf("span.IsRecording() returned %#v, want %#v", got, want)
	}
}

func TestNonRecordingSpanTracerStart(t *testing.T) {
	tid, err := TraceIDFromHex("01000000000000000000000000000000")
	if err != nil {
		t.Fatalf("failure creating TraceID: %s", err.Error())
	}
	sid, err := SpanIDFromHex("0200000000000000")
	if err != nil {
		t.Fatalf("failure creating SpanID: %s", err.Error())
	}
	sc := NewSpanContext(SpanContextConfig{TraceID: tid, SpanID: sid})

	ctx := ContextWithSpanContext(context.Background(), sc)
	_, span := NewNoopTracerProvider().Tracer("test instrumentation").Start(ctx, "span1")

	if got, want := span.SpanContext(), sc; !assertSpanContextEqual(got, want) {
		t.Errorf("SpanContext not carried by nonRecordingSpan. got %#v, want %#v", got, want)
	}
}
