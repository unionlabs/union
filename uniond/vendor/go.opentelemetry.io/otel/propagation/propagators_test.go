// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package propagation_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"
)

const (
	traceIDStr = "4bf92f3577b34da6a3ce929d0e0e4736"
	spanIDStr  = "00f067aa0ba902b7"
)

var (
	traceID = mustTraceIDFromHex(traceIDStr)
	spanID  = mustSpanIDFromHex(spanIDStr)
)

func mustTraceIDFromHex(s string) (t trace.TraceID) {
	var err error
	t, err = trace.TraceIDFromHex(s)
	if err != nil {
		panic(err)
	}
	return
}

func mustSpanIDFromHex(s string) (t trace.SpanID) {
	var err error
	t, err = trace.SpanIDFromHex(s)
	if err != nil {
		panic(err)
	}
	return
}

type outOfThinAirPropagator struct {
	t *testing.T
}

var _ propagation.TextMapPropagator = outOfThinAirPropagator{}

func (p outOfThinAirPropagator) Extract(ctx context.Context, carrier propagation.TextMapCarrier) context.Context {
	sc := trace.NewSpanContext(trace.SpanContextConfig{
		TraceID:    traceID,
		SpanID:     spanID,
		TraceFlags: 0,
	})
	require.True(p.t, sc.IsValid())
	return trace.ContextWithRemoteSpanContext(ctx, sc)
}

func (outOfThinAirPropagator) Inject(context.Context, propagation.TextMapCarrier) {}

func (outOfThinAirPropagator) Fields() []string {
	return nil
}

type nilCarrier struct{}

var _ propagation.TextMapCarrier = nilCarrier{}

func (nilCarrier) Keys() []string {
	return nil
}

func (nilCarrier) Get(key string) string {
	return ""
}

func (nilCarrier) Set(key string, value string) {}

func TestMultiplePropagators(t *testing.T) {
	ootaProp := outOfThinAirPropagator{t: t}
	ns := nilCarrier{}
	testProps := []propagation.TextMapPropagator{
		propagation.TraceContext{},
	}
	bg := context.Background()
	// sanity check of oota propagator, ensuring that it really
	// generates the valid span context out of thin air
	{
		ctx := ootaProp.Extract(bg, ns)
		sc := trace.SpanContextFromContext(ctx)
		require.True(t, sc.IsValid(), "oota prop failed sanity check")
		require.True(t, sc.IsRemote(), "oota prop is remote")
	}
	// sanity check for real propagators, ensuring that they
	// really are not putting any valid span context into an empty
	// go context in absence of the HTTP headers.
	for _, prop := range testProps {
		ctx := prop.Extract(bg, ns)
		sc := trace.SpanContextFromContext(ctx)
		require.Falsef(t, sc.IsValid(), "%#v failed sanity check", prop)
		require.Falsef(t, sc.IsRemote(), "%#v prop set a remote", prop)
	}
	for _, prop := range testProps {
		props := propagation.NewCompositeTextMapPropagator(ootaProp, prop)
		ctx := props.Extract(bg, ns)
		sc := trace.SpanContextFromContext(ctx)
		assert.Truef(t, sc.IsRemote(), "%#v prop is remote", prop)
		assert.Truef(t, sc.IsValid(), "%#v clobbers span context", prop)
	}
}
