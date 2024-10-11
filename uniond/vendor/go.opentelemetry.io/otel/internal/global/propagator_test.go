// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package global

import (
	"context"
	"testing"

	"go.opentelemetry.io/otel/internal/internaltest"
)

func TestTextMapPropagatorDelegation(t *testing.T) {
	ResetForTest(t)
	ctx := context.Background()
	carrier := internaltest.NewTextMapCarrier(nil)

	// The default should be a noop.
	initial := TextMapPropagator()
	initial.Inject(ctx, carrier)
	ctx = initial.Extract(ctx, carrier)
	if !carrier.GotN(t, 0) || !carrier.SetN(t, 0) {
		return
	}

	// Make sure the delegate woks as expected.
	delegate := internaltest.NewTextMapPropagator("test")
	delegate.Inject(ctx, carrier)
	ctx = delegate.Extract(ctx, carrier)
	if !delegate.InjectedN(t, carrier, 1) || !delegate.ExtractedN(t, ctx, 1) {
		return
	}

	// The initial propagator should use the delegate after it is set as the
	// global.
	SetTextMapPropagator(delegate)
	initial.Inject(ctx, carrier)
	ctx = initial.Extract(ctx, carrier)
	delegate.InjectedN(t, carrier, 2)
	delegate.ExtractedN(t, ctx, 2)
}

func TestTextMapPropagatorDelegationNil(t *testing.T) {
	ResetForTest(t)
	ctx := context.Background()
	carrier := internaltest.NewTextMapCarrier(nil)

	// The default should be a noop.
	initial := TextMapPropagator()
	initial.Inject(ctx, carrier)
	ctx = initial.Extract(ctx, carrier)
	if !carrier.GotN(t, 0) || !carrier.SetN(t, 0) {
		return
	}

	// Delegation to nil should not make a change.
	SetTextMapPropagator(nil)
	initial.Inject(ctx, carrier)
	initial.Extract(ctx, carrier)
	if !carrier.GotN(t, 0) || !carrier.SetN(t, 0) {
		return
	}
}

func TestTextMapPropagatorFields(t *testing.T) {
	ResetForTest(t)
	initial := TextMapPropagator()
	delegate := internaltest.NewTextMapPropagator("test")
	delegateFields := delegate.Fields()

	// Sanity check on the initial Fields.
	if got := initial.Fields(); fieldsEqual(got, delegateFields) {
		t.Fatalf("testing fields (%v) matched Noop fields (%v)", delegateFields, got)
	}
	SetTextMapPropagator(delegate)
	// Check previous returns from global not correctly delegate.
	if got := initial.Fields(); !fieldsEqual(got, delegateFields) {
		t.Errorf("global TextMapPropagator.Fields returned %v instead of delegating, want (%v)", got, delegateFields)
	}
	// Check new calls to global.
	if got := TextMapPropagator().Fields(); !fieldsEqual(got, delegateFields) {
		t.Errorf("global TextMapPropagator.Fields returned %v, want (%v)", got, delegateFields)
	}
}

func fieldsEqual(f1, f2 []string) bool {
	if len(f1) != len(f2) {
		return false
	}
	for i := range f1 {
		if f1[i] != f2[i] {
			return false
		}
	}
	return true
}
