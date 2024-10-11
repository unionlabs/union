// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package noop // import "go.opentelemetry.io/otel/trace/noop"

import (
	"context"
	"reflect"
	"testing"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/trace"
)

func TestImplementationNoPanics(t *testing.T) {
	// Check that if type has an embedded interface and that interface has
	// methods added to it than the No-Op implementation implements them.
	t.Run("TracerProvider", assertAllExportedMethodNoPanic(
		reflect.ValueOf(TracerProvider{}),
		reflect.TypeOf((*trace.TracerProvider)(nil)).Elem(),
	))
	t.Run("Tracer", assertAllExportedMethodNoPanic(
		reflect.ValueOf(Tracer{}),
		reflect.TypeOf((*trace.Tracer)(nil)).Elem(),
	))
	t.Run("Span", assertAllExportedMethodNoPanic(
		reflect.ValueOf(Span{}),
		reflect.TypeOf((*trace.Span)(nil)).Elem(),
	))
}

func assertAllExportedMethodNoPanic(rVal reflect.Value, rType reflect.Type) func(*testing.T) {
	return func(t *testing.T) {
		for n := 0; n < rType.NumMethod(); n++ {
			mType := rType.Method(n)
			if !mType.IsExported() {
				t.Logf("ignoring unexported %s", mType.Name)
				continue
			}
			m := rVal.MethodByName(mType.Name)
			if !m.IsValid() {
				t.Errorf("unknown method for %s: %s", rVal.Type().Name(), mType.Name)
			}

			numIn := mType.Type.NumIn()
			if mType.Type.IsVariadic() {
				numIn--
			}
			args := make([]reflect.Value, numIn)
			ctx := context.Background()
			for i := range args {
				aType := mType.Type.In(i)
				if aType.Name() == "Context" {
					// Do not panic on a nil context.
					args[i] = reflect.ValueOf(ctx)
				} else {
					args[i] = reflect.New(aType).Elem()
				}
			}

			assert.NotPanicsf(t, func() {
				_ = m.Call(args)
			}, "%s.%s", rVal.Type().Name(), mType.Name)
		}
	}
}

func TestNewTracerProvider(t *testing.T) {
	tp := NewTracerProvider()
	assert.Equal(t, tp, TracerProvider{})
	tracer := tp.Tracer("")
	assert.Equal(t, tracer, Tracer{})
}

func TestTracerStartPropagatesSpanContext(t *testing.T) {
	tracer := NewTracerProvider().Tracer("")
	spanCtx := trace.SpanContext{}

	ctx := trace.ContextWithSpanContext(context.Background(), spanCtx)
	ctx, span := tracer.Start(ctx, "test_span")
	assert.Equal(t, spanCtx, trace.SpanContextFromContext(ctx), "empty span context not set in context")
	assert.IsType(t, Span{}, span, "non-noop span returned")
	assert.Equal(t, spanCtx, span.SpanContext(), "empty span context not returned from span")
	assert.False(t, span.IsRecording(), "empty span context returned recording span")

	spanCtx = spanCtx.WithTraceID(trace.TraceID([16]byte{1}))
	spanCtx = spanCtx.WithSpanID(trace.SpanID([8]byte{1}))
	ctx = trace.ContextWithSpanContext(context.Background(), spanCtx)
	ctx, span = tracer.Start(ctx, "test_span")
	assert.Equal(t, spanCtx, trace.SpanContextFromContext(ctx), "non-empty span context not set in context")
	assert.Equal(t, spanCtx, span.SpanContext(), "non-empty span context not returned from span")
	assert.False(t, span.IsRecording(), "non-empty span context returned recording span")

	rSpan := recordingSpan{Span: Span{sc: spanCtx}}
	ctx = trace.ContextWithSpan(context.Background(), rSpan)
	ctx, span = tracer.Start(ctx, "test_span")
	assert.Equal(t, spanCtx, trace.SpanContextFromContext(ctx), "recording span's span context not set in context")
	assert.IsType(t, Span{}, span, "non-noop span returned")
	assert.Equal(t, spanCtx, span.SpanContext(), "recording span's span context not returned from span")
	assert.False(t, span.IsRecording(), "recording span returned")
}

type recordingSpan struct{ Span }

func (recordingSpan) IsRecording() bool { return true }
