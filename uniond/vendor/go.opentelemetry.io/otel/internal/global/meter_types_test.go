// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package global // import "go.opentelemetry.io/otel/internal/global"

import (
	"context"

	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/metric/embedded"
)

type testMeterProvider struct {
	embedded.MeterProvider

	count int
}

func (p *testMeterProvider) Meter(name string, opts ...metric.MeterOption) metric.Meter {
	p.count++

	return &testMeter{}
}

type testMeter struct {
	embedded.Meter

	afCount   int
	afUDCount int
	afGauge   int

	aiCount   int
	aiUDCount int
	aiGauge   int

	sfCount   int
	sfUDCount int
	sfHist    int
	sfGauge   int

	siCount   int
	siUDCount int
	siHist    int
	siGauge   int

	callbacks []metric.Callback
}

func (m *testMeter) Int64Counter(name string, options ...metric.Int64CounterOption) (metric.Int64Counter, error) {
	m.siCount++
	return &testCountingIntInstrument{}, nil
}

func (m *testMeter) Int64UpDownCounter(name string, options ...metric.Int64UpDownCounterOption) (metric.Int64UpDownCounter, error) {
	m.siUDCount++
	return &testCountingIntInstrument{}, nil
}

func (m *testMeter) Int64Histogram(name string, options ...metric.Int64HistogramOption) (metric.Int64Histogram, error) {
	m.siHist++
	return &testCountingIntInstrument{}, nil
}

func (m *testMeter) Int64Gauge(name string, options ...metric.Int64GaugeOption) (metric.Int64Gauge, error) {
	m.siGauge++
	return &testCountingIntInstrument{}, nil
}

func (m *testMeter) Int64ObservableCounter(name string, options ...metric.Int64ObservableCounterOption) (metric.Int64ObservableCounter, error) {
	m.aiCount++
	return &testCountingIntInstrument{}, nil
}

func (m *testMeter) Int64ObservableUpDownCounter(name string, options ...metric.Int64ObservableUpDownCounterOption) (metric.Int64ObservableUpDownCounter, error) {
	m.aiUDCount++
	return &testCountingIntInstrument{}, nil
}

func (m *testMeter) Int64ObservableGauge(name string, options ...metric.Int64ObservableGaugeOption) (metric.Int64ObservableGauge, error) {
	m.aiGauge++
	return &testCountingIntInstrument{}, nil
}

func (m *testMeter) Float64Counter(name string, options ...metric.Float64CounterOption) (metric.Float64Counter, error) {
	m.sfCount++
	return &testCountingFloatInstrument{}, nil
}

func (m *testMeter) Float64UpDownCounter(name string, options ...metric.Float64UpDownCounterOption) (metric.Float64UpDownCounter, error) {
	m.sfUDCount++
	return &testCountingFloatInstrument{}, nil
}

func (m *testMeter) Float64Histogram(name string, options ...metric.Float64HistogramOption) (metric.Float64Histogram, error) {
	m.sfHist++
	return &testCountingFloatInstrument{}, nil
}

func (m *testMeter) Float64Gauge(name string, options ...metric.Float64GaugeOption) (metric.Float64Gauge, error) {
	m.sfGauge++
	return &testCountingFloatInstrument{}, nil
}

func (m *testMeter) Float64ObservableCounter(name string, options ...metric.Float64ObservableCounterOption) (metric.Float64ObservableCounter, error) {
	m.afCount++
	return &testCountingFloatInstrument{}, nil
}

func (m *testMeter) Float64ObservableUpDownCounter(name string, options ...metric.Float64ObservableUpDownCounterOption) (metric.Float64ObservableUpDownCounter, error) {
	m.afUDCount++
	return &testCountingFloatInstrument{}, nil
}

func (m *testMeter) Float64ObservableGauge(name string, options ...metric.Float64ObservableGaugeOption) (metric.Float64ObservableGauge, error) {
	m.afGauge++
	return &testCountingFloatInstrument{}, nil
}

// RegisterCallback captures the function that will be called during Collect.
func (m *testMeter) RegisterCallback(f metric.Callback, i ...metric.Observable) (metric.Registration, error) {
	m.callbacks = append(m.callbacks, f)
	return testReg{
		f: func(idx int) func() {
			return func() { m.callbacks[idx] = nil }
		}(len(m.callbacks) - 1),
	}, nil
}

type testReg struct {
	embedded.Registration

	f func()
}

func (r testReg) Unregister() error {
	r.f()
	return nil
}

// This enables async collection.
func (m *testMeter) collect() {
	ctx := context.Background()
	o := observationRecorder{ctx: ctx}
	for _, f := range m.callbacks {
		if f == nil {
			// Unregister.
			continue
		}
		_ = f(ctx, o)
	}
}

type observationRecorder struct {
	embedded.Observer

	ctx context.Context
}

func (o observationRecorder) ObserveFloat64(i metric.Float64Observable, value float64, _ ...metric.ObserveOption) {
	iImpl, ok := i.(*testCountingFloatInstrument)
	if ok {
		iImpl.observe()
	}
}

func (o observationRecorder) ObserveInt64(i metric.Int64Observable, value int64, _ ...metric.ObserveOption) {
	iImpl, ok := i.(*testCountingIntInstrument)
	if ok {
		iImpl.observe()
	}
}
