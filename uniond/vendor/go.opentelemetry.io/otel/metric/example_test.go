// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package metric_test

import (
	"context"
	"database/sql"
	"fmt"
	"net/http"
	"runtime"
	"time"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
	semconv "go.opentelemetry.io/otel/semconv/v1.25.0"
)

var meter = otel.Meter("my-service-meter")

func ExampleMeter_synchronous() {
	// Create a histogram using the global MeterProvider.
	workDuration, err := meter.Int64Histogram(
		"workDuration",
		metric.WithUnit("ms"))
	if err != nil {
		fmt.Println("Failed to register instrument")
		panic(err)
	}

	startTime := time.Now()
	ctx := context.Background()
	// Do work
	// ...
	workDuration.Record(ctx, time.Since(startTime).Milliseconds())
}

func ExampleMeter_asynchronous_single() {
	_, err := meter.Int64ObservableGauge(
		"DiskUsage",
		metric.WithUnit("By"),
		metric.WithInt64Callback(func(_ context.Context, obsrv metric.Int64Observer) error {
			// Do the real work here to get the real disk usage. For example,
			//
			//   usage, err := GetDiskUsage(diskID)
			//   if err != nil {
			//   	if retryable(err) {
			//   		// Retry the usage measurement.
			//   	} else {
			//   		return err
			//   	}
			//   }
			//
			// For demonstration purpose, a static value is used here.
			usage := 75000
			obsrv.Observe(int64(usage), metric.WithAttributes(attribute.Int("disk.id", 3)))
			return nil
		}),
	)
	if err != nil {
		fmt.Println("failed to register instrument")
		panic(err)
	}
}

func ExampleMeter_asynchronous_multiple() {
	// This is just a sample of memory stats to record from the Memstats
	heapAlloc, err := meter.Int64ObservableUpDownCounter("heapAllocs")
	if err != nil {
		fmt.Println("failed to register updown counter for heapAllocs")
		panic(err)
	}
	gcCount, err := meter.Int64ObservableCounter("gcCount")
	if err != nil {
		fmt.Println("failed to register counter for gcCount")
		panic(err)
	}

	_, err = meter.RegisterCallback(
		func(_ context.Context, o metric.Observer) error {
			memStats := &runtime.MemStats{}
			// This call does work
			runtime.ReadMemStats(memStats)

			o.ObserveInt64(heapAlloc, int64(memStats.HeapAlloc))
			o.ObserveInt64(gcCount, int64(memStats.NumGC))

			return nil
		},
		heapAlloc,
		gcCount,
	)
	if err != nil {
		fmt.Println("Failed to register callback")
		panic(err)
	}
}

// Counters can be used to measure a non-negative, increasing value.
//
// Here's how you might report the number of calls for an HTTP handler.
func ExampleMeter_counter() {
	apiCounter, err := meter.Int64Counter(
		"api.counter",
		metric.WithDescription("Number of API calls."),
		metric.WithUnit("{call}"),
	)
	if err != nil {
		panic(err)
	}
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		apiCounter.Add(r.Context(), 1)

		// do some work in an API call
	})
}

// UpDown counters can increment and decrement, allowing you to observe
// a cumulative value that goes up or down.
//
// Here's how you might report the number of items of some collection.
func ExampleMeter_upDownCounter() {
	var err error
	itemsCounter, err := meter.Int64UpDownCounter(
		"items.counter",
		metric.WithDescription("Number of items."),
		metric.WithUnit("{item}"),
	)
	if err != nil {
		panic(err)
	}

	_ = func() {
		// code that adds an item to the collection
		itemsCounter.Add(context.Background(), 1)
	}

	_ = func() {
		// code that removes an item from the collection
		itemsCounter.Add(context.Background(), -1)
	}
}

// Histograms are used to measure a distribution of values over time.
//
// Here's how you might report a distribution of response times for an HTTP handler.
func ExampleMeter_histogram() {
	histogram, err := meter.Float64Histogram(
		"task.duration",
		metric.WithDescription("The duration of task execution."),
		metric.WithUnit("s"),
		metric.WithExplicitBucketBoundaries(.005, .01, .025, .05, .075, .1, .25, .5, .75, 1, 2.5, 5, 7.5, 10),
	)
	if err != nil {
		panic(err)
	}
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()

		// do some work in an API call

		duration := time.Since(start)
		histogram.Record(r.Context(), duration.Seconds())
	})
}

// Observable counters can be used to measure an additive, non-negative,
// monotonically increasing value.
//
// Here's how you might report time since the application started.
func ExampleMeter_observableCounter() {
	start := time.Now()
	if _, err := meter.Float64ObservableCounter(
		"uptime",
		metric.WithDescription("The duration since the application started."),
		metric.WithUnit("s"),
		metric.WithFloat64Callback(func(_ context.Context, o metric.Float64Observer) error {
			o.Observe(float64(time.Since(start).Seconds()))
			return nil
		}),
	); err != nil {
		panic(err)
	}
}

// Observable UpDown counters can increment and decrement, allowing you to measure
// an additive, non-negative, non-monotonically increasing cumulative value.
//
// Here's how you might report some database metrics.
func ExampleMeter_observableUpDownCounter() {
	// The function registers asynchronous metrics for the provided db.
	// Make sure to unregister metric.Registration before closing the provided db.
	_ = func(db *sql.DB, meter metric.Meter, poolName string) (metric.Registration, error) {
		max, err := meter.Int64ObservableUpDownCounter(
			"db.client.connections.max",
			metric.WithDescription("The maximum number of open connections allowed."),
			metric.WithUnit("{connection}"),
		)
		if err != nil {
			return nil, err
		}

		waitTime, err := meter.Int64ObservableUpDownCounter(
			"db.client.connections.wait_time",
			metric.WithDescription("The time it took to obtain an open connection from the pool."),
			metric.WithUnit("ms"),
		)
		if err != nil {
			return nil, err
		}

		reg, err := meter.RegisterCallback(
			func(_ context.Context, o metric.Observer) error {
				stats := db.Stats()
				o.ObserveInt64(max, int64(stats.MaxOpenConnections))
				o.ObserveInt64(waitTime, int64(stats.WaitDuration))
				return nil
			},
			max,
			waitTime,
		)
		if err != nil {
			return nil, err
		}
		return reg, nil
	}
}

// Observable Gauges should be used to measure non-additive values.
//
// Here's how you might report memory usage of the heap objects used
// in application.
func ExampleMeter_observableGauge() {
	if _, err := meter.Int64ObservableGauge(
		"memory.heap",
		metric.WithDescription(
			"Memory usage of the allocated heap objects.",
		),
		metric.WithUnit("By"),
		metric.WithInt64Callback(func(_ context.Context, o metric.Int64Observer) error {
			var m runtime.MemStats
			runtime.ReadMemStats(&m)
			o.Observe(int64(m.HeapAlloc))
			return nil
		}),
	); err != nil {
		panic(err)
	}
}

// You can add Attributes by using the [WithAttributeSet] and [WithAttributes] options.
//
// Here's how you might add the HTTP status code attribute to your recordings.
func ExampleMeter_attributes() {
	apiCounter, err := meter.Int64UpDownCounter(
		"api.finished.counter",
		metric.WithDescription("Number of finished API calls."),
		metric.WithUnit("{call}"),
	)
	if err != nil {
		panic(err)
	}
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		// do some work in an API call and set the response HTTP status code
		statusCode := http.StatusOK

		apiCounter.Add(r.Context(), 1,
			metric.WithAttributes(semconv.HTTPResponseStatusCode(statusCode)))
	})
}
