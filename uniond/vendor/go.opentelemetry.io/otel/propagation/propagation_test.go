// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package propagation_test

import (
	"context"
	"slices"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/propagation"
)

type ctxKeyType uint

var ctxKey ctxKeyType

type carrier []string

func (c *carrier) Keys() []string { return nil }

func (c *carrier) Get(string) string { return "" }

func (c *carrier) Set(setter, _ string) {
	*c = append(*c, setter)
}

type propagator struct {
	Name string
}

func (p propagator) Inject(ctx context.Context, carrier propagation.TextMapCarrier) {
	carrier.Set(p.Name, "")
}

func (p propagator) Extract(ctx context.Context, carrier propagation.TextMapCarrier) context.Context {
	v := ctx.Value(ctxKey)
	if v == nil {
		ctx = context.WithValue(ctx, ctxKey, []string{p.Name})
	} else {
		orig := v.([]string)
		ctx = context.WithValue(ctx, ctxKey, append(orig, p.Name))
	}
	return ctx
}

func (p propagator) Fields() []string { return []string{p.Name} }

func TestCompositeTextMapPropagatorFields(t *testing.T) {
	a, b1, b2 := propagator{"a"}, propagator{"b"}, propagator{"b"}

	want := map[string]struct{}{
		"a": {},
		"b": {},
	}
	got := propagation.NewCompositeTextMapPropagator(a, b1, b2).Fields()
	if len(got) != len(want) {
		t.Fatalf("invalid fields from composite: %v (want %v)", got, want)
	}
	for _, v := range got {
		if _, ok := want[v]; !ok {
			t.Errorf("invalid field returned from composite: %q", v)
		}
	}
}

func TestCompositeTextMapPropagatorInject(t *testing.T) {
	a, b := propagator{"a"}, propagator{"b"}

	c := make(carrier, 0, 2)
	propagation.NewCompositeTextMapPropagator(a, b).Inject(context.Background(), &c)

	if got := strings.Join([]string(c), ","); got != "a,b" {
		t.Errorf("invalid inject order: %s", got)
	}
}

func TestCompositeTextMapPropagatorExtract(t *testing.T) {
	a, b := propagator{"a"}, propagator{"b"}

	ctx := context.Background()
	ctx = propagation.NewCompositeTextMapPropagator(a, b).Extract(ctx, nil)

	v := ctx.Value(ctxKey)
	if v == nil {
		t.Fatal("no composite extraction")
	}
	if got := strings.Join(v.([]string), ","); got != "a,b" {
		t.Errorf("invalid extract order: %s", got)
	}
}

func TestMapCarrierGet(t *testing.T) {
	carrier := propagation.MapCarrier{
		"foo": "bar",
		"baz": "qux",
	}

	assert.Equal(t, carrier.Get("foo"), "bar")
	assert.Equal(t, carrier.Get("baz"), "qux")
}

func TestMapCarrierSet(t *testing.T) {
	carrier := make(propagation.MapCarrier)
	carrier.Set("foo", "bar")
	carrier.Set("baz", "qux")

	assert.Equal(t, carrier["foo"], "bar")
	assert.Equal(t, carrier["baz"], "qux")
}

func TestMapCarrierKeys(t *testing.T) {
	carrier := propagation.MapCarrier{
		"foo": "bar",
		"baz": "qux",
	}

	keys := carrier.Keys()
	slices.Sort(keys)
	assert.Equal(t, []string{"baz", "foo"}, keys)
}
