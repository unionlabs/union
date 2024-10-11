// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package global

import (
	"context"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/trace"
	"go.opentelemetry.io/otel/trace/embedded"
	"go.opentelemetry.io/otel/trace/noop"
)

type fnTracerProvider struct {
	embedded.TracerProvider

	tracer func(string, ...trace.TracerOption) trace.Tracer
}

func (fn fnTracerProvider) Tracer(instrumentationName string, opts ...trace.TracerOption) trace.Tracer {
	return fn.tracer(instrumentationName, opts...)
}

type fnTracer struct {
	embedded.Tracer

	start func(ctx context.Context, spanName string, opts ...trace.SpanStartOption) (context.Context, trace.Span)
}

func (fn fnTracer) Start(ctx context.Context, spanName string, opts ...trace.SpanStartOption) (context.Context, trace.Span) {
	return fn.start(ctx, spanName, opts...)
}

func TestTraceProviderDelegation(t *testing.T) {
	ResetForTest(t)

	// Map of tracers to expected span names.
	expected := map[string][]string{
		"pre":      {"span2"},
		"post":     {"span3"},
		"fromSpan": {"span4"},
	}

	ctx := context.Background()
	gtp := TracerProvider()
	tracer1 := gtp.Tracer("pre")
	// This is started before an SDK was registered and should be dropped.
	_, span1 := tracer1.Start(ctx, "span1")

	SetTracerProvider(fnTracerProvider{
		tracer: func(name string, opts ...trace.TracerOption) trace.Tracer {
			spans, ok := expected[name]
			assert.Truef(t, ok, "invalid tracer: %s", name)
			return fnTracer{
				start: func(ctx context.Context, spanName string, opts ...trace.SpanStartOption) (context.Context, trace.Span) {
					if ok {
						if len(spans) == 0 {
							t.Errorf("unexpected span: %s", spanName)
						} else {
							var want string
							want, spans = spans[0], spans[1:]
							assert.Equal(t, want, spanName)
						}
					}
					return noop.NewTracerProvider().Tracer(name).Start(ctx, spanName)
				},
			}
		},
	})

	// This span was started before initialization, it is expected to be dropped.
	span1.End()

	// The existing Tracer should have been configured to now use the configured SDK.
	_, span2 := tracer1.Start(ctx, "span2")
	span2.End()

	// The global TracerProvider should now create Tracers that also use the newly configured SDK.
	tracer2 := gtp.Tracer("post")
	_, span3 := tracer2.Start(ctx, "span3")
	span3.End()

	// The noop-span should still provide access to a usable TracerProvider.
	_, span4 := span1.TracerProvider().Tracer("fromSpan").Start(ctx, "span4")
	span4.End()
}

func TestTraceProviderDelegates(t *testing.T) {
	ResetForTest(t)

	// Retrieve the placeholder TracerProvider.
	gtp := TracerProvider()

	// Configure it with a spy.
	called := false
	SetTracerProvider(fnTracerProvider{
		tracer: func(name string, opts ...trace.TracerOption) trace.Tracer {
			called = true
			assert.Equal(t, "abc", name)
			return noop.NewTracerProvider().Tracer("")
		},
	})

	gtp.Tracer("abc", trace.WithInstrumentationVersion("xyz"))
	assert.True(t, called, "expected configured TraceProvider to be called")
}

func TestTraceProviderDelegatesConcurrentSafe(t *testing.T) {
	ResetForTest(t)

	// Retrieve the placeholder TracerProvider.
	gtp := TracerProvider()

	done := make(chan struct{})
	quit := make(chan struct{})
	go func() {
		defer close(done)
		for {
			select {
			case <-time.After(1 * time.Millisecond):
				gtp.Tracer("abc", trace.WithInstrumentationVersion("xyz"))
			case <-quit:
				return
			}
		}
	}()

	// Wait for the goroutine to make some calls before installing the provider.
	<-time.After(100 * time.Millisecond)

	// Configure it with a spy.
	called := int32(0)
	SetTracerProvider(fnTracerProvider{
		tracer: func(name string, opts ...trace.TracerOption) trace.Tracer {
			newVal := atomic.AddInt32(&called, 1)
			assert.Equal(t, "abc", name)
			if newVal == 10 {
				// Signal the goroutine to finish.
				close(quit)
			}
			return noop.NewTracerProvider().Tracer("")
		},
	})

	// Wait for the go routine to finish
	<-done

	assert.LessOrEqual(t, int32(10), atomic.LoadInt32(&called), "expected configured TraceProvider to be called")
}

func TestTracerDelegatesConcurrentSafe(t *testing.T) {
	ResetForTest(t)

	// Retrieve the placeholder TracerProvider.
	gtp := TracerProvider()
	tracer := gtp.Tracer("abc", trace.WithInstrumentationVersion("xyz"))

	done := make(chan struct{})
	quit := make(chan struct{})
	go func() {
		defer close(done)
		for {
			select {
			case <-time.After(1 * time.Millisecond):
				tracer.Start(context.Background(), "name")
			case <-quit:
				return
			}
		}
	}()

	// Wait for the goroutine to make some calls before installing the provider.
	<-time.After(100 * time.Millisecond)

	// Configure it with a spy.
	called := int32(0)
	SetTracerProvider(fnTracerProvider{
		tracer: func(name string, opts ...trace.TracerOption) trace.Tracer {
			assert.Equal(t, "abc", name)
			return fnTracer{
				start: func(ctx context.Context, spanName string, opts ...trace.SpanStartOption) (context.Context, trace.Span) {
					newVal := atomic.AddInt32(&called, 1)
					assert.Equal(t, "name", spanName)
					if newVal == 10 {
						// Signal the goroutine to finish.
						close(quit)
					}
					return noop.NewTracerProvider().Tracer("").Start(ctx, spanName)
				},
			}
		},
	})

	// Wait for the go routine to finish
	<-done

	assert.LessOrEqual(t, int32(10), atomic.LoadInt32(&called), "expected configured TraceProvider to be called")
}

func TestTraceProviderDelegatesSameInstance(t *testing.T) {
	ResetForTest(t)

	// Retrieve the placeholder TracerProvider.
	gtp := TracerProvider()
	tracer := gtp.Tracer("abc", trace.WithInstrumentationVersion("xyz"))
	assert.Same(t, tracer, gtp.Tracer("abc", trace.WithInstrumentationVersion("xyz")))
	assert.Same(t, tracer, gtp.Tracer("abc", trace.WithInstrumentationVersion("xyz")))

	SetTracerProvider(fnTracerProvider{
		tracer: func(name string, opts ...trace.TracerOption) trace.Tracer {
			return noop.NewTracerProvider().Tracer("")
		},
	})

	assert.NotSame(t, tracer, gtp.Tracer("abc", trace.WithInstrumentationVersion("xyz")))
}

func TestSpanContextPropagatedWithNonRecordingSpan(t *testing.T) {
	ResetForTest(t)

	sc := trace.NewSpanContext(trace.SpanContextConfig{
		TraceID:    [16]byte{0x01},
		SpanID:     [8]byte{0x01},
		TraceFlags: trace.FlagsSampled,
		Remote:     true,
	})
	ctx := trace.ContextWithSpanContext(context.Background(), sc)
	_, span := TracerProvider().Tracer("test").Start(ctx, "test")

	assert.Equal(t, sc, span.SpanContext())
	assert.False(t, span.IsRecording())
}
