// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package global

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/metric"
	metricnoop "go.opentelemetry.io/otel/metric/noop"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"
	tracenoop "go.opentelemetry.io/otel/trace/noop"
)

type nonComparableErrorHandler struct {
	ErrorHandler

	nonComparable func() //nolint:structcheck,unused  // This is not called.
}

type nonComparableTracerProvider struct {
	trace.TracerProvider

	nonComparable func() //nolint:structcheck,unused  // This is not called.
}

type nonComparableMeterProvider struct {
	metric.MeterProvider

	nonComparable func() //nolint:structcheck,unused  // This is not called.
}

type fnErrHandler func(error)

func (f fnErrHandler) Handle(err error) { f(err) }

var noopEH = fnErrHandler(func(error) {})

func TestSetErrorHandler(t *testing.T) {
	t.Run("Set With default is a noop", func(t *testing.T) {
		ResetForTest(t)
		SetErrorHandler(GetErrorHandler())

		eh, ok := GetErrorHandler().(*ErrDelegator)
		if !ok {
			t.Fatal("Global ErrorHandler should be the default ErrorHandler")
		}

		if eh.delegate.Load() != nil {
			t.Fatal("ErrorHandler should not delegate when setting itself")
		}
	})

	t.Run("First Set() should replace the delegate", func(t *testing.T) {
		ResetForTest(t)

		SetErrorHandler(noopEH)

		_, ok := GetErrorHandler().(*ErrDelegator)
		if ok {
			t.Fatal("Global ErrorHandler was not changed")
		}
	})

	t.Run("Set() should delegate existing ErrorHandlers", func(t *testing.T) {
		ResetForTest(t)

		eh := GetErrorHandler()
		SetErrorHandler(noopEH)

		errDel, ok := eh.(*ErrDelegator)
		if !ok {
			t.Fatal("Wrong ErrorHandler returned")
		}

		if errDel.delegate.Load() == nil {
			t.Fatal("The ErrDelegator should have a delegate")
		}
	})

	t.Run("non-comparable types should not panic", func(t *testing.T) {
		ResetForTest(t)

		eh := nonComparableErrorHandler{}
		assert.NotPanics(t, func() { SetErrorHandler(eh) }, "delegate")
		assert.NotPanics(t, func() { SetErrorHandler(eh) }, "replacement")
	})
}

func TestSetTracerProvider(t *testing.T) {
	t.Run("Set With default is a noop", func(t *testing.T) {
		ResetForTest(t)
		SetTracerProvider(TracerProvider())

		tp, ok := TracerProvider().(*tracerProvider)
		if !ok {
			t.Fatal("Global TracerProvider should be the default tracer provider")
		}

		if tp.delegate != nil {
			t.Fatal("tracer provider should not delegate when setting itself")
		}
	})

	t.Run("First Set() should replace the delegate", func(t *testing.T) {
		ResetForTest(t)

		SetTracerProvider(tracenoop.NewTracerProvider())

		_, ok := TracerProvider().(*tracerProvider)
		if ok {
			t.Fatal("Global TracerProvider was not changed")
		}
	})

	t.Run("Set() should delegate existing TracerProviders", func(t *testing.T) {
		ResetForTest(t)

		tp := TracerProvider()
		SetTracerProvider(tracenoop.NewTracerProvider())

		ntp := tp.(*tracerProvider)

		if ntp.delegate == nil {
			t.Fatal("The delegated tracer providers should have a delegate")
		}
	})

	t.Run("non-comparable types should not panic", func(t *testing.T) {
		ResetForTest(t)

		tp := nonComparableTracerProvider{}
		SetTracerProvider(tp)
		assert.NotPanics(t, func() { SetTracerProvider(tp) })
	})
}

func TestSetTextMapPropagator(t *testing.T) {
	t.Run("Set With default is a noop", func(t *testing.T) {
		ResetForTest(t)
		SetTextMapPropagator(TextMapPropagator())

		tmp, ok := TextMapPropagator().(*textMapPropagator)
		if !ok {
			t.Fatal("Global TextMapPropagator should be the default propagator")
		}

		if tmp.delegate != nil {
			t.Fatal("TextMapPropagator should not delegate when setting itself")
		}
	})

	t.Run("First Set() should replace the delegate", func(t *testing.T) {
		ResetForTest(t)

		SetTextMapPropagator(propagation.TraceContext{})

		_, ok := TextMapPropagator().(*textMapPropagator)
		if ok {
			t.Fatal("Global TextMapPropagator was not changed")
		}
	})

	t.Run("Set() should delegate existing propagators", func(t *testing.T) {
		ResetForTest(t)

		p := TextMapPropagator()
		SetTextMapPropagator(propagation.TraceContext{})

		np := p.(*textMapPropagator)

		if np.delegate == nil {
			t.Fatal("The delegated TextMapPropagators should have a delegate")
		}
	})

	t.Run("non-comparable types should not panic", func(t *testing.T) {
		ResetForTest(t)

		// A composite TextMapPropagator is not comparable.
		prop := propagation.NewCompositeTextMapPropagator(propagation.TraceContext{})
		SetTextMapPropagator(prop)
		assert.NotPanics(t, func() { SetTextMapPropagator(prop) })
	})
}

func TestSetMeterProvider(t *testing.T) {
	t.Run("Set With default is a noop", func(t *testing.T) {
		ResetForTest(t)

		SetMeterProvider(MeterProvider())

		mp, ok := MeterProvider().(*meterProvider)
		if !ok {
			t.Fatal("Global MeterProvider should be the default meter provider")
		}

		if mp.delegate != nil {
			t.Fatal("meter provider should not delegate when setting itself")
		}
	})

	t.Run("First Set() should replace the delegate", func(t *testing.T) {
		ResetForTest(t)

		SetMeterProvider(metricnoop.NewMeterProvider())

		_, ok := MeterProvider().(*meterProvider)
		if ok {
			t.Fatal("Global MeterProvider was not changed")
		}
	})

	t.Run("Set() should delegate existing Meter Providers", func(t *testing.T) {
		ResetForTest(t)

		mp := MeterProvider()

		SetMeterProvider(metricnoop.NewMeterProvider())

		dmp := mp.(*meterProvider)

		if dmp.delegate == nil {
			t.Fatal("The delegated meter providers should have a delegate")
		}
	})

	t.Run("non-comparable types should not panic", func(t *testing.T) {
		ResetForTest(t)

		mp := nonComparableMeterProvider{}
		SetMeterProvider(mp)
		assert.NotPanics(t, func() { SetMeterProvider(mp) })
	})
}
