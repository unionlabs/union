// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelgrpc

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/metric/embedded"
)

func TestNilInstruments(t *testing.T) {
	mp := meterProvider{}
	c := newConfig([]Option{WithMeterProvider(mp)}, "test")

	ctx := context.Background()
	assert.NotPanics(t, func() { c.rpcDuration.Record(ctx, 0) }, "rpcDuration")
	assert.NotPanics(t, func() { c.rpcRequestSize.Record(ctx, 0) }, "rpcRequestSize")
	assert.NotPanics(t, func() { c.rpcResponseSize.Record(ctx, 0) }, "rpcResponseSize")
	assert.NotPanics(t, func() { c.rpcRequestsPerRPC.Record(ctx, 0) }, "rpcRequestsPerRPC")
	assert.NotPanics(t, func() { c.rpcResponsesPerRPC.Record(ctx, 0) }, "rpcResponsesPerRPC")
}

type meterProvider struct {
	embedded.MeterProvider
}

func (meterProvider) Meter(string, ...metric.MeterOption) metric.Meter {
	return meter{}
}

type meter struct {
	// Panic for non-implemented methods.
	metric.Meter
}

func (meter) Int64Histogram(string, ...metric.Int64HistogramOption) (metric.Int64Histogram, error) {
	return nil, assert.AnError
}

func (meter) Float64Histogram(string, ...metric.Float64HistogramOption) (metric.Float64Histogram, error) {
	return nil, assert.AnError
}
