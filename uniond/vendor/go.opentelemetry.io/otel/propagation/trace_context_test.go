// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package propagation_test

import (
	"context"
	"net/http"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"
)

var (
	traceparent = http.CanonicalHeaderKey("traceparent")
	tracestate  = http.CanonicalHeaderKey("tracestate")

	prop = propagation.TraceContext{}
)

type testcase struct {
	name   string
	header http.Header
	sc     trace.SpanContext
}

func TestExtractValidTraceContext(t *testing.T) {
	stateStr := "key1=value1,key2=value2"
	state, err := trace.ParseTraceState(stateStr)
	require.NoError(t, err)

	tests := []testcase{
		{
			name: "not sampled",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID: traceID,
				SpanID:  spanID,
				Remote:  true,
			}),
		},
		{
			name: "sampled",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID:    traceID,
				SpanID:     spanID,
				TraceFlags: trace.FlagsSampled,
				Remote:     true,
			}),
		},
		{
			name: "valid tracestate",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00"},
				tracestate:  []string{stateStr},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID:    traceID,
				SpanID:     spanID,
				TraceState: state,
				Remote:     true,
			}),
		},
		{
			name: "invalid tracestate preserves traceparent",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00"},
				tracestate:  []string{"invalid$@#=invalid"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID: traceID,
				SpanID:  spanID,
				Remote:  true,
			}),
		},
		{
			name: "future version not sampled",
			header: http.Header{
				traceparent: []string{"02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID: traceID,
				SpanID:  spanID,
				Remote:  true,
			}),
		},
		{
			name: "future version sampled",
			header: http.Header{
				traceparent: []string{"02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID:    traceID,
				SpanID:     spanID,
				TraceFlags: trace.FlagsSampled,
				Remote:     true,
			}),
		},
		{
			name: "future version sample bit set",
			header: http.Header{
				traceparent: []string{"02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-09"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID:    traceID,
				SpanID:     spanID,
				TraceFlags: trace.FlagsSampled,
				Remote:     true,
			}),
		},
		{
			name: "future version sample bit not set",
			header: http.Header{
				traceparent: []string{"02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-08"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID: traceID,
				SpanID:  spanID,
				Remote:  true,
			}),
		},
		{
			name: "future version additional data",
			header: http.Header{
				traceparent: []string{"02-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00-XYZxsf09"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID: traceID,
				SpanID:  spanID,
				Remote:  true,
			}),
		},
		{
			name: "B3 format ending in dash",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00-"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID: traceID,
				SpanID:  spanID,
				Remote:  true,
			}),
		},
		{
			name: "future version B3 format ending in dash",
			header: http.Header{
				traceparent: []string{"03-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00-"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID: traceID,
				SpanID:  spanID,
				Remote:  true,
			}),
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			ctx := context.Background()
			ctx = prop.Extract(ctx, propagation.HeaderCarrier(tc.header))
			assert.Equal(t, tc.sc, trace.SpanContextFromContext(ctx))
		})
	}
}

func TestExtractInvalidTraceContextFromHTTPReq(t *testing.T) {
	tests := []struct {
		name   string
		header string
	}{
		{
			name:   "wrong version length",
			header: "0000-00000000000000000000000000000000-0000000000000000-01",
		},
		{
			name:   "wrong trace ID length",
			header: "00-ab00000000000000000000000000000000-cd00000000000000-01",
		},
		{
			name:   "wrong span ID length",
			header: "00-ab000000000000000000000000000000-cd0000000000000000-01",
		},
		{
			name:   "wrong trace flag length",
			header: "00-ab000000000000000000000000000000-cd00000000000000-0100",
		},
		{
			name:   "bogus version",
			header: "qw-00000000000000000000000000000000-0000000000000000-01",
		},
		{
			name:   "bogus trace ID",
			header: "00-qw000000000000000000000000000000-cd00000000000000-01",
		},
		{
			name:   "bogus span ID",
			header: "00-ab000000000000000000000000000000-qw00000000000000-01",
		},
		{
			name:   "bogus trace flag",
			header: "00-ab000000000000000000000000000000-cd00000000000000-qw",
		},
		{
			name:   "upper case version",
			header: "A0-00000000000000000000000000000000-0000000000000000-01",
		},
		{
			name:   "upper case trace ID",
			header: "00-AB000000000000000000000000000000-cd00000000000000-01",
		},
		{
			name:   "upper case span ID",
			header: "00-ab000000000000000000000000000000-CD00000000000000-01",
		},
		{
			name:   "upper case trace flag",
			header: "00-ab000000000000000000000000000000-cd00000000000000-A1",
		},
		{
			name:   "zero trace ID and span ID",
			header: "00-00000000000000000000000000000000-0000000000000000-01",
		},
		{
			name:   "trace-flag unused bits set",
			header: "00-ab000000000000000000000000000000-cd00000000000000-09",
		},
		{
			name:   "missing options",
			header: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7",
		},
		{
			name:   "empty options",
			header: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-",
		},
	}

	empty := trace.SpanContext{}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			h := http.Header{traceparent: []string{tt.header}}
			ctx := context.Background()
			ctx = prop.Extract(ctx, propagation.HeaderCarrier(h))

			// Failure to extract needs to result in no SpanContext being set.
			// This cannot be directly measured, but we can check that an
			// zero-value SpanContext is returned from SpanContextFromContext.
			assert.Equal(t, empty, trace.SpanContextFromContext(ctx))
		})
	}
}

func TestInjectValidTraceContext(t *testing.T) {
	stateStr := "key1=value1,key2=value2"
	state, err := trace.ParseTraceState(stateStr)
	require.NoError(t, err)

	tests := []testcase{
		{
			name: "not sampled",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID: traceID,
				SpanID:  spanID,
				Remote:  true,
			}),
		},
		{
			name: "sampled",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID:    traceID,
				SpanID:     spanID,
				TraceFlags: trace.FlagsSampled,
				Remote:     true,
			}),
		},
		{
			name: "unsupported trace flag bits dropped",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID:    traceID,
				SpanID:     spanID,
				TraceFlags: 0xff,
				Remote:     true,
			}),
		},
		{
			name: "with tracestate",
			header: http.Header{
				traceparent: []string{"00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00"},
				tracestate:  []string{stateStr},
			},
			sc: trace.NewSpanContext(trace.SpanContextConfig{
				TraceID:    traceID,
				SpanID:     spanID,
				TraceState: state,
				Remote:     true,
			}),
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			ctx := context.Background()
			ctx = trace.ContextWithRemoteSpanContext(ctx, tc.sc)

			h := http.Header{}
			prop.Inject(ctx, propagation.HeaderCarrier(h))
			assert.Equal(t, tc.header, h)
		})
	}
}

func TestInvalidSpanContextDropped(t *testing.T) {
	invalidSC := trace.SpanContext{}
	require.False(t, invalidSC.IsValid())
	ctx := trace.ContextWithRemoteSpanContext(context.Background(), invalidSC)

	header := http.Header{}
	propagation.TraceContext{}.Inject(ctx, propagation.HeaderCarrier(header))
	assert.Equal(t, "", header.Get("traceparent"), "injected invalid SpanContext")
}

func TestTraceContextFields(t *testing.T) {
	expected := []string{"traceparent", "tracestate"}
	assert.Equal(t, expected, propagation.TraceContext{}.Fields())
}
