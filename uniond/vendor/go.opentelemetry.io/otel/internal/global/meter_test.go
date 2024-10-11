// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package global // import "go.opentelemetry.io/otel/internal/global"

import (
	"context"
	"fmt"
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/metric/noop"
)

func TestMeterProviderConcurrentSafe(t *testing.T) {
	mp := &meterProvider{}
	done := make(chan struct{})
	finish := make(chan struct{})
	go func() {
		defer close(done)
		for i := 0; ; i++ {
			mp.Meter(fmt.Sprintf("a%d", i))
			select {
			case <-finish:
				return
			default:
			}
		}
	}()

	mp.setDelegate(noop.NewMeterProvider())
	close(finish)
	<-done
}

var zeroCallback metric.Callback = func(ctx context.Context, or metric.Observer) error {
	return nil
}

func TestMeterConcurrentSafe(t *testing.T) {
	mtr := &meter{}

	wg := &sync.WaitGroup{}
	wg.Add(1)
	done := make(chan struct{})
	finish := make(chan struct{})
	go func() {
		defer close(done)
		for i, once := 0, false; ; i++ {
			name := fmt.Sprintf("a%d", i)
			_, _ = mtr.Float64ObservableCounter(name)
			_, _ = mtr.Float64ObservableUpDownCounter(name)
			_, _ = mtr.Float64ObservableGauge(name)
			_, _ = mtr.Int64ObservableCounter(name)
			_, _ = mtr.Int64ObservableUpDownCounter(name)
			_, _ = mtr.Int64ObservableGauge(name)
			_, _ = mtr.Float64Counter(name)
			_, _ = mtr.Float64UpDownCounter(name)
			_, _ = mtr.Float64Histogram(name)
			_, _ = mtr.Float64Gauge(name)
			_, _ = mtr.Int64Counter(name)
			_, _ = mtr.Int64UpDownCounter(name)
			_, _ = mtr.Int64Histogram(name)
			_, _ = mtr.Int64Gauge(name)
			_, _ = mtr.RegisterCallback(zeroCallback)
			if !once {
				wg.Done()
				once = true
			}
			select {
			case <-finish:
				return
			default:
			}
		}
	}()

	wg.Wait()
	mtr.setDelegate(noop.NewMeterProvider())
	close(finish)
	<-done
}

func TestUnregisterConcurrentSafe(t *testing.T) {
	mtr := &meter{}
	reg, err := mtr.RegisterCallback(zeroCallback)
	require.NoError(t, err)

	wg := &sync.WaitGroup{}
	wg.Add(1)
	done := make(chan struct{})
	finish := make(chan struct{})
	go func() {
		defer close(done)
		for i, once := 0, false; ; i++ {
			_ = reg.Unregister()
			if !once {
				wg.Done()
				once = true
			}
			select {
			case <-finish:
				return
			default:
			}
		}
	}()
	_ = reg.Unregister()

	wg.Wait()
	mtr.setDelegate(noop.NewMeterProvider())
	close(finish)
	<-done
}

func testSetupAllInstrumentTypes(t *testing.T, m metric.Meter) (metric.Float64Counter, metric.Float64ObservableCounter) {
	afcounter, err := m.Float64ObservableCounter("test_Async_Counter")
	require.NoError(t, err)
	_, err = m.Float64ObservableUpDownCounter("test_Async_UpDownCounter")
	assert.NoError(t, err)
	_, err = m.Float64ObservableGauge("test_Async_Gauge")
	assert.NoError(t, err)

	_, err = m.Int64ObservableCounter("test_Async_Counter")
	assert.NoError(t, err)
	_, err = m.Int64ObservableUpDownCounter("test_Async_UpDownCounter")
	assert.NoError(t, err)
	_, err = m.Int64ObservableGauge("test_Async_Gauge")
	assert.NoError(t, err)

	_, err = m.RegisterCallback(func(ctx context.Context, obs metric.Observer) error {
		obs.ObserveFloat64(afcounter, 3)
		return nil
	}, afcounter)
	require.NoError(t, err)

	sfcounter, err := m.Float64Counter("test_Sync_Counter")
	require.NoError(t, err)
	_, err = m.Float64UpDownCounter("test_Sync_UpDownCounter")
	assert.NoError(t, err)
	_, err = m.Float64Histogram("test_Sync_Histogram")
	assert.NoError(t, err)
	_, err = m.Float64Gauge("test_Sync_Gauge")
	assert.NoError(t, err)

	_, err = m.Int64Counter("test_Sync_Counter")
	assert.NoError(t, err)
	_, err = m.Int64UpDownCounter("test_Sync_UpDownCounter")
	assert.NoError(t, err)
	_, err = m.Int64Histogram("test_Sync_Histogram")
	assert.NoError(t, err)
	_, err = m.Int64Gauge("test_Sync_Gauge")
	assert.NoError(t, err)

	return sfcounter, afcounter
}

// This is to emulate a read from an exporter.
func testCollect(t *testing.T, m metric.Meter) {
	if tMeter, ok := m.(*meter); ok {
		m, ok = tMeter.delegate.Load().(metric.Meter)
		if !ok {
			t.Error("meter was not delegated")
			return
		}
	}
	tMeter, ok := m.(*testMeter)
	if !ok {
		t.Error("collect called on non-test Meter")
		return
	}
	tMeter.collect()
}

func TestMeterProviderDelegatesCalls(t *testing.T) {
	// The global MeterProvider should directly call the underlying MeterProvider
	// if it is set prior to Meter() being called.

	// globalMeterProvider := otel.GetMeterProvider
	globalMeterProvider := &meterProvider{}

	mp := &testMeterProvider{}

	// otel.SetMeterProvider(mp)
	globalMeterProvider.setDelegate(mp)

	assert.Equal(t, 0, mp.count)

	meter := globalMeterProvider.Meter("go.opentelemetry.io/otel/metric/internal/global/meter_test")

	ctr, actr := testSetupAllInstrumentTypes(t, meter)

	ctr.Add(context.Background(), 5)

	testCollect(t, meter) // This is a hacky way to emulate a read from an exporter

	// Calls to Meter() after setDelegate() should be executed by the delegate
	require.IsType(t, &testMeter{}, meter)
	tMeter := meter.(*testMeter)
	assert.Equal(t, 1, tMeter.afCount)
	assert.Equal(t, 1, tMeter.afUDCount)
	assert.Equal(t, 1, tMeter.afGauge)
	assert.Equal(t, 1, tMeter.aiCount)
	assert.Equal(t, 1, tMeter.aiUDCount)
	assert.Equal(t, 1, tMeter.aiGauge)
	assert.Equal(t, 1, tMeter.sfCount)
	assert.Equal(t, 1, tMeter.sfUDCount)
	assert.Equal(t, 1, tMeter.sfHist)
	assert.Equal(t, 1, tMeter.siCount)
	assert.Equal(t, 1, tMeter.siUDCount)
	assert.Equal(t, 1, tMeter.siHist)
	assert.Equal(t, 1, len(tMeter.callbacks))

	// Because the Meter was provided by testmeterProvider it should also return our test instrument
	require.IsType(t, &testCountingFloatInstrument{}, ctr, "the meter did not delegate calls to the meter")
	assert.Equal(t, 1, ctr.(*testCountingFloatInstrument).count)

	require.IsType(t, &testCountingFloatInstrument{}, actr, "the meter did not delegate calls to the meter")
	assert.Equal(t, 1, actr.(*testCountingFloatInstrument).count)

	assert.Equal(t, 1, mp.count)
}

func TestMeterDelegatesCalls(t *testing.T) {
	// The global MeterProvider should directly provide a Meter instance that
	// can be updated.  If the SetMeterProvider is called after a Meter was
	// obtained, but before instruments only the instrument should be generated
	// by the delegated type.

	globalMeterProvider := &meterProvider{}

	mp := &testMeterProvider{}

	assert.Equal(t, 0, mp.count)

	m := globalMeterProvider.Meter("go.opentelemetry.io/otel/metric/internal/global/meter_test")

	globalMeterProvider.setDelegate(mp)

	ctr, actr := testSetupAllInstrumentTypes(t, m)

	ctr.Add(context.Background(), 5)

	testCollect(t, m) // This is a hacky way to emulate a read from an exporter

	// Calls to Meter methods after setDelegate() should be executed by the delegate
	require.IsType(t, &meter{}, m)
	tMeter := m.(*meter).delegate.Load().(*testMeter)
	require.NotNil(t, tMeter)
	assert.Equal(t, 1, tMeter.afCount)
	assert.Equal(t, 1, tMeter.afUDCount)
	assert.Equal(t, 1, tMeter.afGauge)
	assert.Equal(t, 1, tMeter.aiCount)
	assert.Equal(t, 1, tMeter.aiUDCount)
	assert.Equal(t, 1, tMeter.aiGauge)
	assert.Equal(t, 1, tMeter.sfCount)
	assert.Equal(t, 1, tMeter.sfUDCount)
	assert.Equal(t, 1, tMeter.sfHist)
	assert.Equal(t, 1, tMeter.siCount)
	assert.Equal(t, 1, tMeter.siUDCount)
	assert.Equal(t, 1, tMeter.siHist)

	// Because the Meter was provided by testmeterProvider it should also return our test instrument
	require.IsType(t, &testCountingFloatInstrument{}, ctr, "the meter did not delegate calls to the meter")
	assert.Equal(t, 1, ctr.(*testCountingFloatInstrument).count)

	// Because the Meter was provided by testmeterProvider it should also return our test instrument
	require.IsType(t, &testCountingFloatInstrument{}, actr, "the meter did not delegate calls to the meter")
	assert.Equal(t, 1, actr.(*testCountingFloatInstrument).count)

	assert.Equal(t, 1, mp.count)
}

func TestMeterDefersDelegations(t *testing.T) {
	// If SetMeterProvider is called after instruments are registered, the
	// instruments should be recreated with the new meter.

	// globalMeterProvider := otel.GetMeterProvider
	globalMeterProvider := &meterProvider{}

	m := globalMeterProvider.Meter("go.opentelemetry.io/otel/metric/internal/global/meter_test")

	ctr, actr := testSetupAllInstrumentTypes(t, m)

	ctr.Add(context.Background(), 5)

	mp := &testMeterProvider{}

	// otel.SetMeterProvider(mp)
	globalMeterProvider.setDelegate(mp)

	testCollect(t, m) // This is a hacky way to emulate a read from an exporter

	// Calls to Meter() before setDelegate() should be the delegated type
	require.IsType(t, &meter{}, m)
	tMeter := m.(*meter).delegate.Load().(*testMeter)
	require.NotNil(t, tMeter)
	assert.Equal(t, 1, tMeter.afCount)
	assert.Equal(t, 1, tMeter.afUDCount)
	assert.Equal(t, 1, tMeter.afGauge)
	assert.Equal(t, 1, tMeter.aiCount)
	assert.Equal(t, 1, tMeter.aiUDCount)
	assert.Equal(t, 1, tMeter.aiGauge)
	assert.Equal(t, 1, tMeter.sfCount)
	assert.Equal(t, 1, tMeter.sfUDCount)
	assert.Equal(t, 1, tMeter.sfHist)
	assert.Equal(t, 1, tMeter.siCount)
	assert.Equal(t, 1, tMeter.siUDCount)
	assert.Equal(t, 1, tMeter.siHist)

	// Because the Meter was a delegate it should return a delegated instrument

	assert.IsType(t, &sfCounter{}, ctr)
	assert.IsType(t, &afCounter{}, actr)
	assert.Equal(t, 1, mp.count)
}

func TestRegistrationDelegation(t *testing.T) {
	// globalMeterProvider := otel.GetMeterProvider
	globalMeterProvider := &meterProvider{}

	m := globalMeterProvider.Meter("go.opentelemetry.io/otel/metric/internal/global/meter_test")
	require.IsType(t, &meter{}, m)
	mImpl := m.(*meter)

	actr, err := m.Float64ObservableCounter("test_Async_Counter")
	require.NoError(t, err)

	var called0 bool
	reg0, err := m.RegisterCallback(func(context.Context, metric.Observer) error {
		called0 = true
		return nil
	}, actr)
	require.NoError(t, err)
	require.Equal(t, 1, mImpl.registry.Len(), "callback not registered")
	// This means reg0 should not be delegated.
	assert.NoError(t, reg0.Unregister())
	assert.Equal(t, 0, mImpl.registry.Len(), "callback not unregistered")

	var called1 bool
	reg1, err := m.RegisterCallback(func(context.Context, metric.Observer) error {
		called1 = true
		return nil
	}, actr)
	require.NoError(t, err)
	require.Equal(t, 1, mImpl.registry.Len(), "second callback not registered")

	var called2 bool
	_, err = m.RegisterCallback(func(context.Context, metric.Observer) error {
		called2 = true
		return nil
	}, actr)
	require.NoError(t, err)
	require.Equal(t, 2, mImpl.registry.Len(), "third callback not registered")

	mp := &testMeterProvider{}
	globalMeterProvider.setDelegate(mp)

	testCollect(t, m) // This is a hacky way to emulate a read from an exporter
	require.False(t, called0, "pre-delegation unregistered callback called")
	require.True(t, called1, "second callback not called")
	require.True(t, called2, "third callback not called")

	assert.NoError(t, reg1.Unregister(), "unregister second callback")
	called1, called2 = false, false // reset called capture
	testCollect(t, m)               // This is a hacky way to emulate a read from an exporter
	assert.False(t, called1, "unregistered second callback called")
	require.True(t, called2, "third callback not called")

	assert.NotPanics(t, func() {
		assert.NoError(t, reg1.Unregister(), "duplicate unregister calls")
	})
}
