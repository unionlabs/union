// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package global

import (
	"context"
	"testing"

	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/metric/embedded"
	"go.opentelemetry.io/otel/metric/noop"
)

func testFloat64ConcurrentSafe(interact func(float64), setDelegate func(metric.Meter)) {
	done := make(chan struct{})
	finish := make(chan struct{})
	go func() {
		defer close(done)
		for {
			interact(1)
			select {
			case <-finish:
				return
			default:
			}
		}
	}()

	setDelegate(noop.NewMeterProvider().Meter(""))
	close(finish)
	<-done
}

func testInt64ConcurrentSafe(interact func(int64), setDelegate func(metric.Meter)) {
	done := make(chan struct{})
	finish := make(chan struct{})
	go func() {
		defer close(done)
		for {
			interact(1)
			select {
			case <-finish:
				return
			default:
			}
		}
	}()

	setDelegate(noop.NewMeterProvider().Meter(""))
	close(finish)
	<-done
}

func TestAsyncInstrumentSetDelegateConcurrentSafe(t *testing.T) {
	// Float64 Instruments
	t.Run("Float64", func(t *testing.T) {
		t.Run("Counter", func(t *testing.T) {
			delegate := &afCounter{}
			f := func(float64) { _ = delegate.Unwrap() }
			testFloat64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("UpDownCounter", func(t *testing.T) {
			delegate := &afUpDownCounter{}
			f := func(float64) { _ = delegate.Unwrap() }
			testFloat64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("Gauge", func(t *testing.T) {
			delegate := &afGauge{}
			f := func(float64) { _ = delegate.Unwrap() }
			testFloat64ConcurrentSafe(f, delegate.setDelegate)
		})
	})

	// Int64 Instruments

	t.Run("Int64", func(t *testing.T) {
		t.Run("Counter", func(t *testing.T) {
			delegate := &aiCounter{}
			f := func(int64) { _ = delegate.Unwrap() }
			testInt64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("UpDownCounter", func(t *testing.T) {
			delegate := &aiUpDownCounter{}
			f := func(int64) { _ = delegate.Unwrap() }
			testInt64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("Gauge", func(t *testing.T) {
			delegate := &aiGauge{}
			f := func(int64) { _ = delegate.Unwrap() }
			testInt64ConcurrentSafe(f, delegate.setDelegate)
		})
	})
}

func TestSyncInstrumentSetDelegateConcurrentSafe(t *testing.T) {
	// Float64 Instruments
	t.Run("Float64", func(t *testing.T) {
		t.Run("Counter", func(t *testing.T) {
			delegate := &sfCounter{}
			f := func(v float64) { delegate.Add(context.Background(), v) }
			testFloat64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("UpDownCounter", func(t *testing.T) {
			delegate := &sfUpDownCounter{}
			f := func(v float64) { delegate.Add(context.Background(), v) }
			testFloat64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("Histogram", func(t *testing.T) {
			delegate := &sfHistogram{}
			f := func(v float64) { delegate.Record(context.Background(), v) }
			testFloat64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("Gauge", func(t *testing.T) {
			delegate := &sfGauge{}
			f := func(v float64) { delegate.Record(context.Background(), v) }
			testFloat64ConcurrentSafe(f, delegate.setDelegate)
		})
	})

	// Int64 Instruments

	t.Run("Int64", func(t *testing.T) {
		t.Run("Counter", func(t *testing.T) {
			delegate := &siCounter{}
			f := func(v int64) { delegate.Add(context.Background(), v) }
			testInt64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("UpDownCounter", func(t *testing.T) {
			delegate := &siUpDownCounter{}
			f := func(v int64) { delegate.Add(context.Background(), v) }
			testInt64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("Histogram", func(t *testing.T) {
			delegate := &siHistogram{}
			f := func(v int64) { delegate.Record(context.Background(), v) }
			testInt64ConcurrentSafe(f, delegate.setDelegate)
		})

		t.Run("Gauge", func(t *testing.T) {
			delegate := &siGauge{}
			f := func(v int64) { delegate.Record(context.Background(), v) }
			testInt64ConcurrentSafe(f, delegate.setDelegate)
		})
	})
}

type testCountingFloatInstrument struct {
	count int

	metric.Float64Observable
	embedded.Float64Counter
	embedded.Float64UpDownCounter
	embedded.Float64Histogram
	embedded.Float64Gauge
	embedded.Float64ObservableCounter
	embedded.Float64ObservableUpDownCounter
	embedded.Float64ObservableGauge
}

func (i *testCountingFloatInstrument) observe() {
	i.count++
}

func (i *testCountingFloatInstrument) Add(context.Context, float64, ...metric.AddOption) {
	i.count++
}

func (i *testCountingFloatInstrument) Record(context.Context, float64, ...metric.RecordOption) {
	i.count++
}

type testCountingIntInstrument struct {
	count int

	metric.Int64Observable
	embedded.Int64Counter
	embedded.Int64UpDownCounter
	embedded.Int64Histogram
	embedded.Int64Gauge
	embedded.Int64ObservableCounter
	embedded.Int64ObservableUpDownCounter
	embedded.Int64ObservableGauge
}

func (i *testCountingIntInstrument) observe() {
	i.count++
}

func (i *testCountingIntInstrument) Add(context.Context, int64, ...metric.AddOption) {
	i.count++
}

func (i *testCountingIntInstrument) Record(context.Context, int64, ...metric.RecordOption) {
	i.count++
}
