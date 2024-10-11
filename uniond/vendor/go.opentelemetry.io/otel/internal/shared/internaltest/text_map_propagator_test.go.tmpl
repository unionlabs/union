// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/internaltest/text_map_propagator_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internaltest

import (
	"context"
	"testing"
)

func TestTextMapPropagatorInjectExtract(t *testing.T) {
	name := "testing"
	ctx := context.Background()
	carrier := NewTextMapCarrier(map[string]string{name: value})
	propagator := NewTextMapPropagator(name)

	propagator.Inject(ctx, carrier)
	// Carrier value overridden with state.
	if carrier.SetKeyValue(t, name, "1,0") {
		// Ensure nothing has been extracted yet.
		propagator.ExtractedN(t, ctx, 0)
		// Test the injection was counted.
		propagator.InjectedN(t, carrier, 1)
	}

	ctx = propagator.Extract(ctx, carrier)
	v := ctx.Value(ctxKeyType(name))
	if v == nil {
		t.Error("TextMapPropagator.Extract failed to extract state")
	}
	if s, ok := v.(state); !ok {
		t.Error("TextMapPropagator.Extract did not extract proper state")
	} else if s.Extractions != 1 {
		t.Error("TextMapPropagator.Extract did not increment state.Extractions")
	}
	if carrier.GotKey(t, name) {
		// Test the extraction was counted.
		propagator.ExtractedN(t, ctx, 1)
		// Ensure no additional injection was recorded.
		propagator.InjectedN(t, carrier, 1)
	}
}

func TestTextMapPropagatorFields(t *testing.T) {
	name := "testing"
	propagator := NewTextMapPropagator(name)
	if got := propagator.Fields(); len(got) != 1 {
		t.Errorf("TextMapPropagator.Fields returned %d fields, want 1", len(got))
	} else if got[0] != name {
		t.Errorf("TextMapPropagator.Fields returned %q, want %q", got[0], name)
	}
}

func TestNewStateEmpty(t *testing.T) {
	if want, got := (state{}), newState(""); got != want {
		t.Errorf("newState(\"\") returned %v, want %v", got, want)
	}
}
