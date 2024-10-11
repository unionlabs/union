// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package baggage

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestContextWithList(t *testing.T) {
	ctx := context.Background()
	l := List{"foo": {Value: "1"}}

	nCtx := ContextWithList(ctx, l)
	assert.Equal(t, baggageState{list: l}, nCtx.Value(baggageKey))
	assert.Nil(t, ctx.Value(baggageKey))
}

func TestClearContextOfList(t *testing.T) {
	l := List{"foo": {Value: "1"}}

	ctx := context.Background()
	ctx = context.WithValue(ctx, baggageKey, l)

	nCtx := ContextWithList(ctx, nil)
	nL, ok := nCtx.Value(baggageKey).(baggageState)
	require.True(t, ok, "wrong type stored in context")
	assert.Nil(t, nL.list)
	assert.Equal(t, l, ctx.Value(baggageKey))
}

func TestListFromContext(t *testing.T) {
	ctx := context.Background()
	assert.Nil(t, ListFromContext(ctx))

	l := List{"foo": {Value: "1"}}
	ctx = context.WithValue(ctx, baggageKey, baggageState{list: l})
	assert.Equal(t, l, ListFromContext(ctx))
}

func TestContextWithSetHook(t *testing.T) {
	var called bool
	f := func(ctx context.Context, list List) context.Context {
		called = true
		return ctx
	}

	ctx := context.Background()
	ctx = ContextWithSetHook(ctx, f)
	assert.False(t, called, "SetHookFunc called when setting hook")
	ctx = ContextWithList(ctx, nil)
	assert.True(t, called, "SetHookFunc not called when setting List")

	// Ensure resetting the hook works.
	called = false
	ctx = ContextWithSetHook(ctx, f)
	assert.False(t, called, "SetHookFunc called when re-setting hook")
	ContextWithList(ctx, nil)
	assert.True(t, called, "SetHookFunc not called when re-setting List")
}

func TestContextWithGetHook(t *testing.T) {
	var called bool
	f := func(ctx context.Context, list List) List {
		called = true
		return list
	}

	ctx := context.Background()
	ctx = ContextWithGetHook(ctx, f)
	assert.False(t, called, "GetHookFunc called when setting hook")
	_ = ListFromContext(ctx)
	assert.True(t, called, "GetHookFunc not called when getting List")

	// Ensure resetting the hook works.
	called = false
	ctx = ContextWithGetHook(ctx, f)
	assert.False(t, called, "GetHookFunc called when re-setting hook")
	_ = ListFromContext(ctx)
	assert.True(t, called, "GetHookFunc not called when re-getting List")
}
