// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package trace // import "go.opentelemetry.io/otel/trace"

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
)

type testSpan struct {
	noopSpan

	ID     byte
	Remote bool
}

func (s testSpan) SpanContext() SpanContext {
	return SpanContext{
		traceID: [16]byte{1},
		spanID:  [8]byte{s.ID},
		remote:  s.Remote,
	}
}

var (
	emptySpan   = noopSpan{}
	localSpan   = testSpan{ID: 1, Remote: false}
	remoteSpan  = testSpan{ID: 1, Remote: true}
	wrappedSpan = nonRecordingSpan{sc: remoteSpan.SpanContext()}
)

func TestSpanFromContext(t *testing.T) {
	testCases := []struct {
		name         string
		context      context.Context
		expectedSpan Span
	}{
		{
			name:         "empty context",
			context:      nil,
			expectedSpan: emptySpan,
		},
		{
			name:         "background context",
			context:      context.Background(),
			expectedSpan: emptySpan,
		},
		{
			name:         "local span",
			context:      ContextWithSpan(context.Background(), localSpan),
			expectedSpan: localSpan,
		},
		{
			name:         "remote span",
			context:      ContextWithSpan(context.Background(), remoteSpan),
			expectedSpan: remoteSpan,
		},
		{
			name:         "wrapped remote span",
			context:      ContextWithRemoteSpanContext(context.Background(), remoteSpan.SpanContext()),
			expectedSpan: wrappedSpan,
		},
		{
			name:         "wrapped local span becomes remote",
			context:      ContextWithRemoteSpanContext(context.Background(), localSpan.SpanContext()),
			expectedSpan: wrappedSpan,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			assert.Equal(t, tc.expectedSpan, SpanFromContext(tc.context))

			// Ensure SpanContextFromContext is just
			// SpanFromContext(…).SpanContext().
			assert.Equal(t, tc.expectedSpan.SpanContext(), SpanContextFromContext(tc.context))

			// Check that SpanFromContext does not produce any heap allocation.
			assert.Equal(t, 0.0, testing.AllocsPerRun(5, func() {
				SpanFromContext(tc.context)
			}), "SpanFromContext allocs")

			// Check that SpanContextFromContext does not produce any heap allocation.
			assert.Equal(t, 0.0, testing.AllocsPerRun(5, func() {
				SpanContextFromContext(tc.context)
			}), "SpanContextFromContext allocs")
		})
	}
}
