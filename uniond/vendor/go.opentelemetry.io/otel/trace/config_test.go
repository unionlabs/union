// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package trace

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/attribute"
)

func TestNewSpanConfig(t *testing.T) {
	k1v1 := attribute.String("key1", "value1")
	k1v2 := attribute.String("key1", "value2")
	k2v2 := attribute.String("key2", "value2")

	timestamp0 := time.Unix(0, 0)
	timestamp1 := time.Unix(0, 0)

	link1 := Link{
		SpanContext: SpanContext{traceID: TraceID([16]byte{1, 1}), spanID: SpanID{3}},
		Attributes:  []attribute.KeyValue{k1v1},
	}
	link2 := Link{
		SpanContext: SpanContext{traceID: TraceID([16]byte{1, 1}), spanID: SpanID{3}},
		Attributes:  []attribute.KeyValue{k1v2, k2v2},
	}

	tests := []struct {
		options  []SpanStartOption
		expected SpanConfig
	}{
		{
			// No non-zero-values should be set.
			[]SpanStartOption{},
			SpanConfig{},
		},
		{
			[]SpanStartOption{
				WithAttributes(k1v1),
			},
			SpanConfig{
				attributes: []attribute.KeyValue{k1v1},
			},
		},
		{
			// Multiple calls should append not overwrite.
			[]SpanStartOption{
				WithAttributes(k1v1),
				WithAttributes(k1v2),
				WithAttributes(k2v2),
			},
			SpanConfig{
				// No uniqueness is guaranteed by the API.
				attributes: []attribute.KeyValue{k1v1, k1v2, k2v2},
			},
		},
		{
			[]SpanStartOption{
				WithAttributes(k1v1, k1v2, k2v2),
			},
			SpanConfig{
				// No uniqueness is guaranteed by the API.
				attributes: []attribute.KeyValue{k1v1, k1v2, k2v2},
			},
		},
		{
			[]SpanStartOption{
				WithTimestamp(timestamp0),
			},
			SpanConfig{
				timestamp: timestamp0,
			},
		},
		{
			[]SpanStartOption{
				// Multiple calls overwrites with last-one-wins.
				WithTimestamp(timestamp0),
				WithTimestamp(timestamp1),
			},
			SpanConfig{
				timestamp: timestamp1,
			},
		},
		{
			[]SpanStartOption{
				WithLinks(link1),
			},
			SpanConfig{
				links: []Link{link1},
			},
		},
		{
			[]SpanStartOption{
				// Multiple calls should append not overwrite.
				WithLinks(link1),
				WithLinks(link1, link2),
			},
			SpanConfig{
				// No uniqueness is guaranteed by the API.
				links: []Link{link1, link1, link2},
			},
		},
		{
			[]SpanStartOption{
				WithNewRoot(),
			},
			SpanConfig{
				newRoot: true,
			},
		},
		{
			[]SpanStartOption{
				// Multiple calls should not change NewRoot state.
				WithNewRoot(),
				WithNewRoot(),
			},
			SpanConfig{
				newRoot: true,
			},
		},
		{
			[]SpanStartOption{
				WithSpanKind(SpanKindConsumer),
			},
			SpanConfig{
				spanKind: SpanKindConsumer,
			},
		},
		{
			[]SpanStartOption{
				// Multiple calls overwrites with last-one-wins.
				WithSpanKind(SpanKindClient),
				WithSpanKind(SpanKindConsumer),
			},
			SpanConfig{
				spanKind: SpanKindConsumer,
			},
		},
		{
			// Everything should work together.
			[]SpanStartOption{
				WithAttributes(k1v1),
				WithTimestamp(timestamp0),
				WithLinks(link1, link2),
				WithNewRoot(),
				WithSpanKind(SpanKindConsumer),
			},
			SpanConfig{
				attributes: []attribute.KeyValue{k1v1},
				timestamp:  timestamp0,
				links:      []Link{link1, link2},
				newRoot:    true,
				spanKind:   SpanKindConsumer,
			},
		},
	}
	for _, test := range tests {
		assert.Equal(t, test.expected, NewSpanStartConfig(test.options...))
	}
}

func TestEndSpanConfig(t *testing.T) {
	timestamp := time.Unix(0, 0)

	tests := []struct {
		options  []SpanEndOption
		expected SpanConfig
	}{
		{
			[]SpanEndOption{},
			SpanConfig{},
		},
		{
			[]SpanEndOption{
				WithStackTrace(true),
			},
			SpanConfig{
				stackTrace: true,
			},
		},
		{
			[]SpanEndOption{
				WithTimestamp(timestamp),
			},
			SpanConfig{
				timestamp: timestamp,
			},
		},
	}
	for _, test := range tests {
		assert.Equal(t, test.expected, NewSpanEndConfig(test.options...))
	}
}

func TestTracerConfig(t *testing.T) {
	v1 := "semver:0.0.1"
	v2 := "semver:1.0.0"
	schemaURL := "https://opentelemetry.io/schemas/1.2.0"
	attrs := attribute.NewSet(
		attribute.String("user", "alice"),
		attribute.Bool("admin", true),
	)

	c := NewTracerConfig(
		// Multiple calls should overwrite.
		WithInstrumentationVersion(v1),
		WithInstrumentationVersion(v2),
		WithSchemaURL(schemaURL),
		WithInstrumentationAttributes(attrs.ToSlice()...),
	)

	assert.Equal(t, v2, c.InstrumentationVersion(), "instrumentation version")
	assert.Equal(t, schemaURL, c.SchemaURL(), "schema URL")
	assert.Equal(t, attrs, c.InstrumentationAttributes(), "instrumentation attributes")
}

// Save benchmark results to a file level var to avoid the compiler optimizing
// away the actual work.
var (
	tracerConfig TracerConfig
	spanConfig   SpanConfig
	eventConfig  EventConfig
)

func BenchmarkNewTracerConfig(b *testing.B) {
	opts := []TracerOption{
		WithInstrumentationVersion("testing version"),
		WithSchemaURL("testing URL"),
	}

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		tracerConfig = NewTracerConfig(opts...)
	}
}

func BenchmarkNewSpanStartConfig(b *testing.B) {
	opts := []SpanStartOption{
		WithAttributes(attribute.Bool("key", true)),
		WithTimestamp(time.Now()),
		WithLinks(Link{}),
		WithNewRoot(),
		WithSpanKind(SpanKindClient),
	}

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		spanConfig = NewSpanStartConfig(opts...)
	}
}

func BenchmarkNewSpanEndConfig(b *testing.B) {
	opts := []SpanEndOption{
		WithTimestamp(time.Now()),
		WithStackTrace(true),
	}

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		spanConfig = NewSpanEndConfig(opts...)
	}
}

func BenchmarkNewEventConfig(b *testing.B) {
	opts := []EventOption{
		WithAttributes(attribute.Bool("key", true)),
		WithTimestamp(time.Now()),
		WithStackTrace(true),
	}

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		eventConfig = NewEventConfig(opts...)
	}
}
