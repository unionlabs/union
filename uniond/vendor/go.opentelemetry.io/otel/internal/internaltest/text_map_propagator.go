// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/internaltest/text_map_propagator.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internaltest // import "go.opentelemetry.io/otel/internal/internaltest"

import (
	"context"
	"fmt"
	"strconv"
	"strings"
	"testing"

	"go.opentelemetry.io/otel/propagation"
)

type ctxKeyType string

type state struct {
	Injections  uint64
	Extractions uint64
}

func newState(encoded string) state {
	if encoded == "" {
		return state{}
	}
	s0, s1, _ := strings.Cut(encoded, ",")
	injects, _ := strconv.ParseUint(s0, 10, 64)
	extracts, _ := strconv.ParseUint(s1, 10, 64)
	return state{
		Injections:  injects,
		Extractions: extracts,
	}
}

func (s state) String() string {
	return fmt.Sprintf("%d,%d", s.Injections, s.Extractions)
}

// TextMapPropagator is a propagation.TextMapPropagator used for testing.
type TextMapPropagator struct {
	name   string
	ctxKey ctxKeyType
}

var _ propagation.TextMapPropagator = (*TextMapPropagator)(nil)

// NewTextMapPropagator returns a new TextMapPropagator for testing. It will
// use name as the key it injects into a TextMapCarrier when Inject is called.
func NewTextMapPropagator(name string) *TextMapPropagator {
	return &TextMapPropagator{name: name, ctxKey: ctxKeyType(name)}
}

func (p *TextMapPropagator) stateFromContext(ctx context.Context) state {
	if v := ctx.Value(p.ctxKey); v != nil {
		if s, ok := v.(state); ok {
			return s
		}
	}
	return state{}
}

func (p *TextMapPropagator) stateFromCarrier(carrier propagation.TextMapCarrier) state {
	return newState(carrier.Get(p.name))
}

// Inject sets cross-cutting concerns for p from ctx into carrier.
func (p *TextMapPropagator) Inject(ctx context.Context, carrier propagation.TextMapCarrier) {
	s := p.stateFromContext(ctx)
	s.Injections++
	carrier.Set(p.name, s.String())
}

// InjectedN tests if p has made n injections to carrier.
func (p *TextMapPropagator) InjectedN(t *testing.T, carrier *TextMapCarrier, n int) bool {
	if actual := p.stateFromCarrier(carrier).Injections; actual != uint64(n) {
		t.Errorf("TextMapPropagator{%q} injected %d times, not %d", p.name, actual, n)
		return false
	}
	return true
}

// Extract reads cross-cutting concerns for p from carrier into ctx.
func (p *TextMapPropagator) Extract(ctx context.Context, carrier propagation.TextMapCarrier) context.Context {
	s := p.stateFromCarrier(carrier)
	s.Extractions++
	return context.WithValue(ctx, p.ctxKey, s)
}

// ExtractedN tests if p has made n extractions from the lineage of ctx.
// nolint (context is not first arg)
func (p *TextMapPropagator) ExtractedN(t *testing.T, ctx context.Context, n int) bool {
	if actual := p.stateFromContext(ctx).Extractions; actual != uint64(n) {
		t.Errorf("TextMapPropagator{%q} extracted %d time, not %d", p.name, actual, n)
		return false
	}
	return true
}

// Fields returns the name of p as the key who's value is set with Inject.
func (p *TextMapPropagator) Fields() []string { return []string{p.name} }
