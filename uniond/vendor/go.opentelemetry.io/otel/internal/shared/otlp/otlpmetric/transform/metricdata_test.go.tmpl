// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/otlp/otlpmetric/transform/metricdata_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package transform

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/sdk/instrumentation"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
	"go.opentelemetry.io/otel/sdk/resource"
	semconv "go.opentelemetry.io/otel/semconv/v1.25.0"
	cpb "go.opentelemetry.io/proto/otlp/common/v1"
	mpb "go.opentelemetry.io/proto/otlp/metrics/v1"
	rpb "go.opentelemetry.io/proto/otlp/resource/v1"
)

type unknownAggT struct {
	metricdata.Aggregation
}

var (
	// Sat Jan 01 2000 00:00:00 GMT+0000.
	start = time.Date(2000, time.January, 0o1, 0, 0, 0, 0, time.FixedZone("GMT", 0))
	end   = start.Add(30 * time.Second)

	alice = attribute.NewSet(attribute.String("user", "alice"))
	bob   = attribute.NewSet(attribute.String("user", "bob"))

	filterAlice = []attribute.KeyValue{attribute.String("user", "filter alice")}
	filterBob   = []attribute.KeyValue{attribute.String("user", "filter bob")}

	pbAlice = &cpb.KeyValue{Key: "user", Value: &cpb.AnyValue{
		Value: &cpb.AnyValue_StringValue{StringValue: "alice"},
	}}
	pbBob = &cpb.KeyValue{Key: "user", Value: &cpb.AnyValue{
		Value: &cpb.AnyValue_StringValue{StringValue: "bob"},
	}}

	pbFilterAlice = &cpb.KeyValue{Key: "user", Value: &cpb.AnyValue{
		Value: &cpb.AnyValue_StringValue{StringValue: "filter alice"},
	}}
	pbFilterBob = &cpb.KeyValue{Key: "user", Value: &cpb.AnyValue{
		Value: &cpb.AnyValue_StringValue{StringValue: "filter bob"},
	}}

	spanIDA  = []byte{0, 0, 0, 0, 0, 0, 0, 1}
	spanIDB  = []byte{0, 0, 0, 0, 0, 0, 0, 2}
	traceIDA = []byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1}
	traceIDB = []byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2}

	exemplarInt64A = metricdata.Exemplar[int64]{
		FilteredAttributes: filterAlice,
		Time:               end,
		Value:              -10,
		SpanID:             spanIDA,
		TraceID:            traceIDA,
	}
	exemplarFloat64A = metricdata.Exemplar[float64]{
		FilteredAttributes: filterAlice,
		Time:               end,
		Value:              -10.0,
		SpanID:             spanIDA,
		TraceID:            traceIDA,
	}
	exemplarInt64B = metricdata.Exemplar[int64]{
		FilteredAttributes: filterBob,
		Time:               end,
		Value:              12,
		SpanID:             spanIDB,
		TraceID:            traceIDB,
	}
	exemplarFloat64B = metricdata.Exemplar[float64]{
		FilteredAttributes: filterBob,
		Time:               end,
		Value:              12.0,
		SpanID:             spanIDB,
		TraceID:            traceIDB,
	}

	pbExemplarInt64A = &mpb.Exemplar{
		FilteredAttributes: []*cpb.KeyValue{pbFilterAlice},
		TimeUnixNano:       uint64(end.UnixNano()),
		Value: &mpb.Exemplar_AsInt{
			AsInt: -10,
		},
		SpanId:  spanIDA,
		TraceId: traceIDA,
	}
	pbExemplarInt64B = &mpb.Exemplar{
		FilteredAttributes: []*cpb.KeyValue{pbFilterBob},
		TimeUnixNano:       uint64(end.UnixNano()),
		Value: &mpb.Exemplar_AsInt{
			AsInt: 12,
		},
		SpanId:  spanIDB,
		TraceId: traceIDB,
	}
	pbExemplarFloat64A = &mpb.Exemplar{
		FilteredAttributes: []*cpb.KeyValue{pbFilterAlice},
		TimeUnixNano:       uint64(end.UnixNano()),
		Value: &mpb.Exemplar_AsDouble{
			AsDouble: -10.0,
		},
		SpanId:  spanIDA,
		TraceId: traceIDA,
	}
	pbExemplarFloat64B = &mpb.Exemplar{
		FilteredAttributes: []*cpb.KeyValue{pbFilterBob},
		TimeUnixNano:       uint64(end.UnixNano()),
		Value: &mpb.Exemplar_AsDouble{
			AsDouble: 12.0,
		},
		SpanId:  spanIDB,
		TraceId: traceIDB,
	}

	minA, maxA, sumA = 2.0, 4.0, 90.0
	minB, maxB, sumB = 4.0, 150.0, 234.0
	otelHDPInt64     = []metricdata.HistogramDataPoint[int64]{
		{
			Attributes:   alice,
			StartTime:    start,
			Time:         end,
			Count:        30,
			Bounds:       []float64{1, 5},
			BucketCounts: []uint64{0, 30, 0},
			Min:          metricdata.NewExtrema(int64(minA)),
			Max:          metricdata.NewExtrema(int64(maxA)),
			Sum:          int64(sumA),
			Exemplars:    []metricdata.Exemplar[int64]{exemplarInt64A},
		}, {
			Attributes:   bob,
			StartTime:    start,
			Time:         end,
			Count:        3,
			Bounds:       []float64{1, 5},
			BucketCounts: []uint64{0, 1, 2},
			Min:          metricdata.NewExtrema(int64(minB)),
			Max:          metricdata.NewExtrema(int64(maxB)),
			Sum:          int64(sumB),
			Exemplars:    []metricdata.Exemplar[int64]{exemplarInt64B},
		},
	}
	otelHDPFloat64 = []metricdata.HistogramDataPoint[float64]{
		{
			Attributes:   alice,
			StartTime:    start,
			Time:         end,
			Count:        30,
			Bounds:       []float64{1, 5},
			BucketCounts: []uint64{0, 30, 0},
			Min:          metricdata.NewExtrema(minA),
			Max:          metricdata.NewExtrema(maxA),
			Sum:          sumA,
			Exemplars:    []metricdata.Exemplar[float64]{exemplarFloat64A},
		}, {
			Attributes:   bob,
			StartTime:    start,
			Time:         end,
			Count:        3,
			Bounds:       []float64{1, 5},
			BucketCounts: []uint64{0, 1, 2},
			Min:          metricdata.NewExtrema(minB),
			Max:          metricdata.NewExtrema(maxB),
			Sum:          sumB,
			Exemplars:    []metricdata.Exemplar[float64]{exemplarFloat64B},
		},
	}

	otelEBucketA = metricdata.ExponentialBucket{
		Offset: 5,
		Counts: []uint64{0, 5, 0, 5},
	}
	otelEBucketB = metricdata.ExponentialBucket{
		Offset: 3,
		Counts: []uint64{0, 5, 0, 5},
	}
	otelEBucketsC = metricdata.ExponentialBucket{
		Offset: 5,
		Counts: []uint64{0, 1},
	}
	otelEBucketsD = metricdata.ExponentialBucket{
		Offset: 3,
		Counts: []uint64{0, 1},
	}

	otelEHDPInt64 = []metricdata.ExponentialHistogramDataPoint[int64]{
		{
			Attributes:     alice,
			StartTime:      start,
			Time:           end,
			Count:          30,
			Scale:          2,
			ZeroCount:      10,
			PositiveBucket: otelEBucketA,
			NegativeBucket: otelEBucketB,
			ZeroThreshold:  .01,
			Min:            metricdata.NewExtrema(int64(minA)),
			Max:            metricdata.NewExtrema(int64(maxA)),
			Sum:            int64(sumA),
			Exemplars:      []metricdata.Exemplar[int64]{exemplarInt64A},
		}, {
			Attributes:     bob,
			StartTime:      start,
			Time:           end,
			Count:          3,
			Scale:          4,
			ZeroCount:      1,
			PositiveBucket: otelEBucketsC,
			NegativeBucket: otelEBucketsD,
			ZeroThreshold:  .02,
			Min:            metricdata.NewExtrema(int64(minB)),
			Max:            metricdata.NewExtrema(int64(maxB)),
			Sum:            int64(sumB),
			Exemplars:      []metricdata.Exemplar[int64]{exemplarInt64B},
		},
	}
	otelEHDPFloat64 = []metricdata.ExponentialHistogramDataPoint[float64]{
		{
			Attributes:     alice,
			StartTime:      start,
			Time:           end,
			Count:          30,
			Scale:          2,
			ZeroCount:      10,
			PositiveBucket: otelEBucketA,
			NegativeBucket: otelEBucketB,
			ZeroThreshold:  .01,
			Min:            metricdata.NewExtrema(minA),
			Max:            metricdata.NewExtrema(maxA),
			Sum:            sumA,
			Exemplars:      []metricdata.Exemplar[float64]{exemplarFloat64A},
		}, {
			Attributes:     bob,
			StartTime:      start,
			Time:           end,
			Count:          3,
			Scale:          4,
			ZeroCount:      1,
			PositiveBucket: otelEBucketsC,
			NegativeBucket: otelEBucketsD,
			ZeroThreshold:  .02,
			Min:            metricdata.NewExtrema(minB),
			Max:            metricdata.NewExtrema(maxB),
			Sum:            sumB,
			Exemplars:      []metricdata.Exemplar[float64]{exemplarFloat64B},
		},
	}

	pbHDPInt64 = []*mpb.HistogramDataPoint{
		{
			Attributes:        []*cpb.KeyValue{pbAlice},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             30,
			Sum:               &sumA,
			ExplicitBounds:    []float64{1, 5},
			BucketCounts:      []uint64{0, 30, 0},
			Min:               &minA,
			Max:               &maxA,
			Exemplars:         []*mpb.Exemplar{pbExemplarInt64A},
		}, {
			Attributes:        []*cpb.KeyValue{pbBob},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             3,
			Sum:               &sumB,
			ExplicitBounds:    []float64{1, 5},
			BucketCounts:      []uint64{0, 1, 2},
			Min:               &minB,
			Max:               &maxB,
			Exemplars:         []*mpb.Exemplar{pbExemplarInt64B},
		},
	}

	pbHDPFloat64 = []*mpb.HistogramDataPoint{
		{
			Attributes:        []*cpb.KeyValue{pbAlice},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             30,
			Sum:               &sumA,
			ExplicitBounds:    []float64{1, 5},
			BucketCounts:      []uint64{0, 30, 0},
			Min:               &minA,
			Max:               &maxA,
			Exemplars:         []*mpb.Exemplar{pbExemplarFloat64A},
		}, {
			Attributes:        []*cpb.KeyValue{pbBob},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             3,
			Sum:               &sumB,
			ExplicitBounds:    []float64{1, 5},
			BucketCounts:      []uint64{0, 1, 2},
			Min:               &minB,
			Max:               &maxB,
			Exemplars:         []*mpb.Exemplar{pbExemplarFloat64B},
		},
	}

	pbEHDPBA = &mpb.ExponentialHistogramDataPoint_Buckets{
		Offset:       5,
		BucketCounts: []uint64{0, 5, 0, 5},
	}
	pbEHDPBB = &mpb.ExponentialHistogramDataPoint_Buckets{
		Offset:       3,
		BucketCounts: []uint64{0, 5, 0, 5},
	}
	pbEHDPBC = &mpb.ExponentialHistogramDataPoint_Buckets{
		Offset:       5,
		BucketCounts: []uint64{0, 1},
	}
	pbEHDPBD = &mpb.ExponentialHistogramDataPoint_Buckets{
		Offset:       3,
		BucketCounts: []uint64{0, 1},
	}

	pbEHDPInt64 = []*mpb.ExponentialHistogramDataPoint{
		{
			Attributes:        []*cpb.KeyValue{pbAlice},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             30,
			Sum:               &sumA,
			Scale:             2,
			ZeroCount:         10,
			Positive:          pbEHDPBA,
			Negative:          pbEHDPBB,
			Min:               &minA,
			Max:               &maxA,
			Exemplars:         []*mpb.Exemplar{pbExemplarInt64A},
		}, {
			Attributes:        []*cpb.KeyValue{pbBob},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             3,
			Sum:               &sumB,
			Scale:             4,
			ZeroCount:         1,
			Positive:          pbEHDPBC,
			Negative:          pbEHDPBD,
			Min:               &minB,
			Max:               &maxB,
			Exemplars:         []*mpb.Exemplar{pbExemplarInt64B},
		},
	}

	pbEHDPFloat64 = []*mpb.ExponentialHistogramDataPoint{
		{
			Attributes:        []*cpb.KeyValue{pbAlice},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             30,
			Sum:               &sumA,
			Scale:             2,
			ZeroCount:         10,
			Positive:          pbEHDPBA,
			Negative:          pbEHDPBB,
			Min:               &minA,
			Max:               &maxA,
			Exemplars:         []*mpb.Exemplar{pbExemplarFloat64A},
		}, {
			Attributes:        []*cpb.KeyValue{pbBob},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             3,
			Sum:               &sumB,
			Scale:             4,
			ZeroCount:         1,
			Positive:          pbEHDPBC,
			Negative:          pbEHDPBD,
			Min:               &minB,
			Max:               &maxB,
			Exemplars:         []*mpb.Exemplar{pbExemplarFloat64B},
		},
	}

	otelHistInt64 = metricdata.Histogram[int64]{
		Temporality: metricdata.DeltaTemporality,
		DataPoints:  otelHDPInt64,
	}
	otelHistFloat64 = metricdata.Histogram[float64]{
		Temporality: metricdata.DeltaTemporality,
		DataPoints:  otelHDPFloat64,
	}
	invalidTemporality metricdata.Temporality
	otelHistInvalid    = metricdata.Histogram[int64]{
		Temporality: invalidTemporality,
		DataPoints:  otelHDPInt64,
	}

	otelExpoHistInt64 = metricdata.ExponentialHistogram[int64]{
		Temporality: metricdata.DeltaTemporality,
		DataPoints:  otelEHDPInt64,
	}
	otelExpoHistFloat64 = metricdata.ExponentialHistogram[float64]{
		Temporality: metricdata.DeltaTemporality,
		DataPoints:  otelEHDPFloat64,
	}
	otelExpoHistInvalid = metricdata.ExponentialHistogram[int64]{
		Temporality: invalidTemporality,
		DataPoints:  otelEHDPInt64,
	}

	pbHistInt64 = &mpb.Histogram{
		AggregationTemporality: mpb.AggregationTemporality_AGGREGATION_TEMPORALITY_DELTA,
		DataPoints:             pbHDPInt64,
	}

	pbHistFloat64 = &mpb.Histogram{
		AggregationTemporality: mpb.AggregationTemporality_AGGREGATION_TEMPORALITY_DELTA,
		DataPoints:             pbHDPFloat64,
	}

	pbExpoHistInt64 = &mpb.ExponentialHistogram{
		AggregationTemporality: mpb.AggregationTemporality_AGGREGATION_TEMPORALITY_DELTA,
		DataPoints:             pbEHDPInt64,
	}

	pbExpoHistFloat64 = &mpb.ExponentialHistogram{
		AggregationTemporality: mpb.AggregationTemporality_AGGREGATION_TEMPORALITY_DELTA,
		DataPoints:             pbEHDPFloat64,
	}

	quantileValuesA = []metricdata.QuantileValue{
		{
			Quantile: 0.0,
			Value:    0.1,
		},
		{
			Quantile: 0.5,
			Value:    1.0,
		},
		{
			Quantile: 1.0,
			Value:    10.4,
		},
	}
	quantileValuesB = []metricdata.QuantileValue{
		{
			Quantile: 0.0,
			Value:    0.5,
		},
		{
			Quantile: 0.5,
			Value:    3.1,
		},
		{
			Quantile: 1.0,
			Value:    8.3,
		},
	}

	pbQuantileValuesA = []*mpb.SummaryDataPoint_ValueAtQuantile{
		{
			Quantile: 0.0,
			Value:    0.1,
		},
		{
			Quantile: 0.5,
			Value:    1.0,
		},
		{
			Quantile: 1.0,
			Value:    10.4,
		},
	}
	pbQuantileValuesB = []*mpb.SummaryDataPoint_ValueAtQuantile{
		{
			Quantile: 0.0,
			Value:    0.5,
		},
		{
			Quantile: 0.5,
			Value:    3.1,
		},
		{
			Quantile: 1.0,
			Value:    8.3,
		},
	}

	otelSummaryDPts = []metricdata.SummaryDataPoint{
		{
			Attributes:     alice,
			StartTime:      start,
			Time:           end,
			Count:          20,
			Sum:            sumA,
			QuantileValues: quantileValuesA,
		},
		{
			Attributes:     bob,
			StartTime:      start,
			Time:           end,
			Count:          26,
			Sum:            sumB,
			QuantileValues: quantileValuesB,
		},
	}

	otelDPtsInt64 = []metricdata.DataPoint[int64]{
		{
			Attributes: alice,
			StartTime:  start,
			Time:       end,
			Value:      1,
			Exemplars:  []metricdata.Exemplar[int64]{exemplarInt64A},
		},
		{
			Attributes: bob,
			StartTime:  start,
			Time:       end,
			Value:      2,
			Exemplars:  []metricdata.Exemplar[int64]{exemplarInt64B},
		},
	}
	otelDPtsFloat64 = []metricdata.DataPoint[float64]{
		{
			Attributes: alice,
			StartTime:  start,
			Time:       end,
			Value:      1.0,
			Exemplars:  []metricdata.Exemplar[float64]{exemplarFloat64A},
		},
		{
			Attributes: bob,
			StartTime:  start,
			Time:       end,
			Value:      2.0,
			Exemplars:  []metricdata.Exemplar[float64]{exemplarFloat64B},
		},
	}

	pbDPtsInt64 = []*mpb.NumberDataPoint{
		{
			Attributes:        []*cpb.KeyValue{pbAlice},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Value:             &mpb.NumberDataPoint_AsInt{AsInt: 1},
			Exemplars:         []*mpb.Exemplar{pbExemplarInt64A},
		},
		{
			Attributes:        []*cpb.KeyValue{pbBob},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Value:             &mpb.NumberDataPoint_AsInt{AsInt: 2},
			Exemplars:         []*mpb.Exemplar{pbExemplarInt64B},
		},
	}
	pbDPtsFloat64 = []*mpb.NumberDataPoint{
		{
			Attributes:        []*cpb.KeyValue{pbAlice},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Value:             &mpb.NumberDataPoint_AsDouble{AsDouble: 1.0},
			Exemplars:         []*mpb.Exemplar{pbExemplarFloat64A},
		},
		{
			Attributes:        []*cpb.KeyValue{pbBob},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Value:             &mpb.NumberDataPoint_AsDouble{AsDouble: 2.0},
			Exemplars:         []*mpb.Exemplar{pbExemplarFloat64B},
		},
	}

	pbDPtsSummary = []*mpb.SummaryDataPoint{
		{
			Attributes:        []*cpb.KeyValue{pbAlice},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             20,
			Sum:               sumA,
			QuantileValues:    pbQuantileValuesA,
		},
		{
			Attributes:        []*cpb.KeyValue{pbBob},
			StartTimeUnixNano: uint64(start.UnixNano()),
			TimeUnixNano:      uint64(end.UnixNano()),
			Count:             26,
			Sum:               sumB,
			QuantileValues:    pbQuantileValuesB,
		},
	}

	otelSumInt64 = metricdata.Sum[int64]{
		Temporality: metricdata.CumulativeTemporality,
		IsMonotonic: true,
		DataPoints:  otelDPtsInt64,
	}
	otelSumFloat64 = metricdata.Sum[float64]{
		Temporality: metricdata.DeltaTemporality,
		IsMonotonic: false,
		DataPoints:  otelDPtsFloat64,
	}
	otelSumInvalid = metricdata.Sum[float64]{
		Temporality: invalidTemporality,
		IsMonotonic: false,
		DataPoints:  otelDPtsFloat64,
	}

	pbSumInt64 = &mpb.Sum{
		AggregationTemporality: mpb.AggregationTemporality_AGGREGATION_TEMPORALITY_CUMULATIVE,
		IsMonotonic:            true,
		DataPoints:             pbDPtsInt64,
	}
	pbSumFloat64 = &mpb.Sum{
		AggregationTemporality: mpb.AggregationTemporality_AGGREGATION_TEMPORALITY_DELTA,
		IsMonotonic:            false,
		DataPoints:             pbDPtsFloat64,
	}

	otelGaugeInt64         = metricdata.Gauge[int64]{DataPoints: otelDPtsInt64}
	otelGaugeFloat64       = metricdata.Gauge[float64]{DataPoints: otelDPtsFloat64}
	otelGaugeZeroStartTime = metricdata.Gauge[int64]{
		DataPoints: []metricdata.DataPoint[int64]{
			{
				Attributes: alice,
				StartTime:  time.Time{},
				Time:       end,
				Value:      1,
				Exemplars:  []metricdata.Exemplar[int64]{exemplarInt64A},
			},
		},
	}

	pbGaugeInt64         = &mpb.Gauge{DataPoints: pbDPtsInt64}
	pbGaugeFloat64       = &mpb.Gauge{DataPoints: pbDPtsFloat64}
	pbGaugeZeroStartTime = &mpb.Gauge{DataPoints: []*mpb.NumberDataPoint{
		{
			Attributes:        []*cpb.KeyValue{pbAlice},
			StartTimeUnixNano: 0,
			TimeUnixNano:      uint64(end.UnixNano()),
			Value:             &mpb.NumberDataPoint_AsInt{AsInt: 1},
			Exemplars:         []*mpb.Exemplar{pbExemplarInt64A},
		},
	}}

	pbSummary = &mpb.Summary{DataPoints: pbDPtsSummary}

	otelSummary = metricdata.Summary{DataPoints: otelSummaryDPts}

	unknownAgg  unknownAggT
	otelMetrics = []metricdata.Metrics{
		{
			Name:        "int64-gauge",
			Description: "Gauge with int64 values",
			Unit:        "1",
			Data:        otelGaugeInt64,
		},
		{
			Name:        "float64-gauge",
			Description: "Gauge with float64 values",
			Unit:        "1",
			Data:        otelGaugeFloat64,
		},
		{
			Name:        "int64-sum",
			Description: "Sum with int64 values",
			Unit:        "1",
			Data:        otelSumInt64,
		},
		{
			Name:        "float64-sum",
			Description: "Sum with float64 values",
			Unit:        "1",
			Data:        otelSumFloat64,
		},
		{
			Name:        "invalid-sum",
			Description: "Sum with invalid temporality",
			Unit:        "1",
			Data:        otelSumInvalid,
		},
		{
			Name:        "int64-histogram",
			Description: "Histogram",
			Unit:        "1",
			Data:        otelHistInt64,
		},
		{
			Name:        "float64-histogram",
			Description: "Histogram",
			Unit:        "1",
			Data:        otelHistFloat64,
		},
		{
			Name:        "invalid-histogram",
			Description: "Invalid histogram",
			Unit:        "1",
			Data:        otelHistInvalid,
		},
		{
			Name:        "unknown",
			Description: "Unknown aggregation",
			Unit:        "1",
			Data:        unknownAgg,
		},
		{
			Name:        "int64-ExponentialHistogram",
			Description: "Exponential Histogram",
			Unit:        "1",
			Data:        otelExpoHistInt64,
		},
		{
			Name:        "float64-ExponentialHistogram",
			Description: "Exponential Histogram",
			Unit:        "1",
			Data:        otelExpoHistFloat64,
		},
		{
			Name:        "invalid-ExponentialHistogram",
			Description: "Invalid Exponential Histogram",
			Unit:        "1",
			Data:        otelExpoHistInvalid,
		},
		{
			Name:        "zero-time",
			Description: "Gauge with 0 StartTime",
			Unit:        "1",
			Data:        otelGaugeZeroStartTime,
		},
		{
			Name:        "summary",
			Description: "Summary metric",
			Unit:        "1",
			Data:        otelSummary,
		},
	}

	pbMetrics = []*mpb.Metric{
		{
			Name:        "int64-gauge",
			Description: "Gauge with int64 values",
			Unit:        "1",
			Data:        &mpb.Metric_Gauge{Gauge: pbGaugeInt64},
		},
		{
			Name:        "float64-gauge",
			Description: "Gauge with float64 values",
			Unit:        "1",
			Data:        &mpb.Metric_Gauge{Gauge: pbGaugeFloat64},
		},
		{
			Name:        "int64-sum",
			Description: "Sum with int64 values",
			Unit:        "1",
			Data:        &mpb.Metric_Sum{Sum: pbSumInt64},
		},
		{
			Name:        "float64-sum",
			Description: "Sum with float64 values",
			Unit:        "1",
			Data:        &mpb.Metric_Sum{Sum: pbSumFloat64},
		},
		{
			Name:        "int64-histogram",
			Description: "Histogram",
			Unit:        "1",
			Data:        &mpb.Metric_Histogram{Histogram: pbHistInt64},
		},
		{
			Name:        "float64-histogram",
			Description: "Histogram",
			Unit:        "1",
			Data:        &mpb.Metric_Histogram{Histogram: pbHistFloat64},
		},
		{
			Name:        "int64-ExponentialHistogram",
			Description: "Exponential Histogram",
			Unit:        "1",
			Data:        &mpb.Metric_ExponentialHistogram{ExponentialHistogram: pbExpoHistInt64},
		},
		{
			Name:        "float64-ExponentialHistogram",
			Description: "Exponential Histogram",
			Unit:        "1",
			Data:        &mpb.Metric_ExponentialHistogram{ExponentialHistogram: pbExpoHistFloat64},
		},
		{
			Name:        "zero-time",
			Description: "Gauge with 0 StartTime",
			Unit:        "1",
			Data:        &mpb.Metric_Gauge{Gauge: pbGaugeZeroStartTime},
		},
		{
			Name:        "summary",
			Description: "Summary metric",
			Unit:        "1",
			Data:        &mpb.Metric_Summary{Summary: pbSummary},
		},
	}

	otelScopeMetrics = []metricdata.ScopeMetrics{
		{
			Scope: instrumentation.Scope{
				Name:      "test/code/path",
				Version:   "v0.1.0",
				SchemaURL: semconv.SchemaURL,
			},
			Metrics: otelMetrics,
		},
	}

	pbScopeMetrics = []*mpb.ScopeMetrics{
		{
			Scope: &cpb.InstrumentationScope{
				Name:    "test/code/path",
				Version: "v0.1.0",
			},
			Metrics:   pbMetrics,
			SchemaUrl: semconv.SchemaURL,
		},
	}

	otelRes = resource.NewWithAttributes(
		semconv.SchemaURL,
		semconv.ServiceName("test server"),
		semconv.ServiceVersion("v0.1.0"),
	)

	pbRes = &rpb.Resource{
		Attributes: []*cpb.KeyValue{
			{
				Key: "service.name",
				Value: &cpb.AnyValue{
					Value: &cpb.AnyValue_StringValue{StringValue: "test server"},
				},
			},
			{
				Key: "service.version",
				Value: &cpb.AnyValue{
					Value: &cpb.AnyValue_StringValue{StringValue: "v0.1.0"},
				},
			},
		},
	}

	otelResourceMetrics = &metricdata.ResourceMetrics{
		Resource:     otelRes,
		ScopeMetrics: otelScopeMetrics,
	}

	pbResourceMetrics = &mpb.ResourceMetrics{
		Resource:     pbRes,
		ScopeMetrics: pbScopeMetrics,
		SchemaUrl:    semconv.SchemaURL,
	}
)

func TestTransformations(t *testing.T) {
	// Run tests from the "bottom-up" of the metricdata data-types and halt
	// when a failure occurs to ensure the clearest failure message (as
	// opposed to the opposite of testing from the top-down which will obscure
	// errors deep inside the structs).

	// DataPoint types.
	assert.Equal(t, pbHDPInt64, HistogramDataPoints(otelHDPInt64))
	assert.Equal(t, pbHDPFloat64, HistogramDataPoints(otelHDPFloat64))
	assert.Equal(t, pbDPtsInt64, DataPoints[int64](otelDPtsInt64))
	require.Equal(t, pbDPtsFloat64, DataPoints[float64](otelDPtsFloat64))
	assert.Equal(t, pbEHDPInt64, ExponentialHistogramDataPoints(otelEHDPInt64))
	assert.Equal(t, pbEHDPFloat64, ExponentialHistogramDataPoints(otelEHDPFloat64))
	assert.Equal(t, pbEHDPBA, ExponentialHistogramDataPointBuckets(otelEBucketA))
	assert.Equal(t, pbDPtsSummary, SummaryDataPoints(otelSummaryDPts))

	// Aggregations.
	h, err := Histogram(otelHistInt64)
	assert.NoError(t, err)
	assert.Equal(t, &mpb.Metric_Histogram{Histogram: pbHistInt64}, h)
	h, err = Histogram(otelHistFloat64)
	assert.NoError(t, err)
	assert.Equal(t, &mpb.Metric_Histogram{Histogram: pbHistFloat64}, h)
	h, err = Histogram(otelHistInvalid)
	assert.ErrorIs(t, err, errUnknownTemporality)
	assert.Nil(t, h)

	s, err := Sum[int64](otelSumInt64)
	assert.NoError(t, err)
	assert.Equal(t, &mpb.Metric_Sum{Sum: pbSumInt64}, s)
	s, err = Sum[float64](otelSumFloat64)
	assert.NoError(t, err)
	assert.Equal(t, &mpb.Metric_Sum{Sum: pbSumFloat64}, s)
	s, err = Sum[float64](otelSumInvalid)
	assert.ErrorIs(t, err, errUnknownTemporality)
	assert.Nil(t, s)

	assert.Equal(t, &mpb.Metric_Gauge{Gauge: pbGaugeInt64}, Gauge[int64](otelGaugeInt64))
	require.Equal(t, &mpb.Metric_Gauge{Gauge: pbGaugeFloat64}, Gauge[float64](otelGaugeFloat64))

	e, err := ExponentialHistogram(otelExpoHistInt64)
	assert.NoError(t, err)
	assert.Equal(t, &mpb.Metric_ExponentialHistogram{ExponentialHistogram: pbExpoHistInt64}, e)
	e, err = ExponentialHistogram(otelExpoHistFloat64)
	assert.NoError(t, err)
	assert.Equal(t, &mpb.Metric_ExponentialHistogram{ExponentialHistogram: pbExpoHistFloat64}, e)
	e, err = ExponentialHistogram(otelExpoHistInvalid)
	assert.ErrorIs(t, err, errUnknownTemporality)
	assert.Nil(t, e)

	require.Equal(t, &mpb.Metric_Summary{Summary: pbSummary}, Summary(otelSummary))

	// Metrics.
	m, err := Metrics(otelMetrics)
	assert.ErrorIs(t, err, errUnknownTemporality)
	assert.ErrorIs(t, err, errUnknownAggregation)
	require.Equal(t, pbMetrics, m)

	// Scope Metrics.
	sm, err := ScopeMetrics(otelScopeMetrics)
	assert.ErrorIs(t, err, errUnknownTemporality)
	assert.ErrorIs(t, err, errUnknownAggregation)
	require.Equal(t, pbScopeMetrics, sm)

	// Resource Metrics.
	rm, err := ResourceMetrics(otelResourceMetrics)
	assert.ErrorIs(t, err, errUnknownTemporality)
	assert.ErrorIs(t, err, errUnknownAggregation)
	require.Equal(t, pbResourceMetrics, rm)
}
