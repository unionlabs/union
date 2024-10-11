// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package metric // import "go.opentelemetry.io/otel/metric"

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/metric/embedded"
)

func TestFloat64ObservableConfiguration(t *testing.T) {
	const (
		token  float64 = 43
		desc           = "Instrument description."
		uBytes         = "By"
	)

	run := func(got float64ObservableConfig) func(*testing.T) {
		return func(t *testing.T) {
			assert.Equal(t, desc, got.Description(), "description")
			assert.Equal(t, uBytes, got.Unit(), "unit")

			// Functions are not comparable.
			cBacks := got.Callbacks()
			require.Len(t, cBacks, 1, "callbacks")
			o := &float64Observer{}
			err := cBacks[0](context.Background(), o)
			require.NoError(t, err)
			assert.Equal(t, token, o.got, "callback not set")
		}
	}

	cback := func(ctx context.Context, obsrv Float64Observer) error {
		obsrv.Observe(token)
		return nil
	}

	t.Run("Float64ObservableCounter", run(
		NewFloat64ObservableCounterConfig(
			WithDescription(desc),
			WithUnit(uBytes),
			WithFloat64Callback(cback),
		),
	))

	t.Run("Float64ObservableUpDownCounter", run(
		NewFloat64ObservableUpDownCounterConfig(
			WithDescription(desc),
			WithUnit(uBytes),
			WithFloat64Callback(cback),
		),
	))

	t.Run("Float64ObservableGauge", run(
		NewFloat64ObservableGaugeConfig(
			WithDescription(desc),
			WithUnit(uBytes),
			WithFloat64Callback(cback),
		),
	))
}

type float64ObservableConfig interface {
	Description() string
	Unit() string
	Callbacks() []Float64Callback
}

type float64Observer struct {
	embedded.Float64Observer
	Observable
	got float64
}

func (o *float64Observer) Observe(v float64, _ ...ObserveOption) {
	o.got = v
}
