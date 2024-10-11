// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package baggage

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/internal/baggage"
)

func TestContext(t *testing.T) {
	ctx := context.Background()
	assert.Equal(t, Baggage{}, FromContext(ctx))

	b := Baggage{list: baggage.List{"key": baggage.Item{Value: "val"}}}
	ctx = ContextWithBaggage(ctx, b)
	assert.Equal(t, b, FromContext(ctx))

	ctx = ContextWithoutBaggage(ctx)
	assert.Equal(t, Baggage{}, FromContext(ctx))
}
