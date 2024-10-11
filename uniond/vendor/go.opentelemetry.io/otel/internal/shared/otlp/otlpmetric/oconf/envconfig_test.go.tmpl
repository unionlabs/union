// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/otlp/otlpmetric/oconf/envconfig_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package oconf

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/metric/metricdata"
)

func TestWithEnvTemporalityPreference(t *testing.T) {
	origReader := DefaultEnvOptionsReader.GetEnv
	tests := []struct {
		name     string
		envValue string
		want     map[metric.InstrumentKind]metricdata.Temporality
	}{
		{
			name:     "default do not set the selector",
			envValue: "",
		},
		{
			name:     "non-normative do not set the selector",
			envValue: "non-normative",
		},
		{
			name:     "cumulative",
			envValue: "cumulative",
			want: map[metric.InstrumentKind]metricdata.Temporality{
				metric.InstrumentKindCounter:                 metricdata.CumulativeTemporality,
				metric.InstrumentKindHistogram:               metricdata.CumulativeTemporality,
				metric.InstrumentKindUpDownCounter:           metricdata.CumulativeTemporality,
				metric.InstrumentKindObservableCounter:       metricdata.CumulativeTemporality,
				metric.InstrumentKindObservableUpDownCounter: metricdata.CumulativeTemporality,
				metric.InstrumentKindObservableGauge:         metricdata.CumulativeTemporality,
			},
		},
		{
			name:     "delta",
			envValue: "delta",
			want: map[metric.InstrumentKind]metricdata.Temporality{
				metric.InstrumentKindCounter:                 metricdata.DeltaTemporality,
				metric.InstrumentKindHistogram:               metricdata.DeltaTemporality,
				metric.InstrumentKindUpDownCounter:           metricdata.CumulativeTemporality,
				metric.InstrumentKindObservableCounter:       metricdata.DeltaTemporality,
				metric.InstrumentKindObservableUpDownCounter: metricdata.CumulativeTemporality,
				metric.InstrumentKindObservableGauge:         metricdata.CumulativeTemporality,
			},
		},
		{
			name:     "lowmemory",
			envValue: "lowmemory",
			want: map[metric.InstrumentKind]metricdata.Temporality{
				metric.InstrumentKindCounter:                 metricdata.DeltaTemporality,
				metric.InstrumentKindHistogram:               metricdata.DeltaTemporality,
				metric.InstrumentKindUpDownCounter:           metricdata.CumulativeTemporality,
				metric.InstrumentKindObservableCounter:       metricdata.CumulativeTemporality,
				metric.InstrumentKindObservableUpDownCounter: metricdata.CumulativeTemporality,
				metric.InstrumentKindObservableGauge:         metricdata.CumulativeTemporality,
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			DefaultEnvOptionsReader.GetEnv = func(key string) string {
				if key == "OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE" {
					return tt.envValue
				}
				return origReader(key)
			}
			cfg := Config{}
			cfg = ApplyGRPCEnvConfigs(cfg)

			if tt.want == nil {
				// There is no function set, the SDK's default is used.
				assert.Nil(t, cfg.Metrics.TemporalitySelector)
				return
			}

			require.NotNil(t, cfg.Metrics.TemporalitySelector)
			for ik, want := range tt.want {
				assert.Equal(t, want, cfg.Metrics.TemporalitySelector(ik))
			}
		})
	}
	DefaultEnvOptionsReader.GetEnv = origReader
}

func TestWithEnvAggPreference(t *testing.T) {
	origReader := DefaultEnvOptionsReader.GetEnv
	tests := []struct {
		name     string
		envValue string
		want     map[metric.InstrumentKind]metric.Aggregation
	}{
		{
			name:     "default do not set the selector",
			envValue: "",
		},
		{
			name:     "non-normative do not set the selector",
			envValue: "non-normative",
		},
		{
			name:     "explicit_bucket_histogram",
			envValue: "explicit_bucket_histogram",
			want: map[metric.InstrumentKind]metric.Aggregation{
				metric.InstrumentKindCounter:                 metric.DefaultAggregationSelector(metric.InstrumentKindCounter),
				metric.InstrumentKindHistogram:               metric.DefaultAggregationSelector(metric.InstrumentKindHistogram),
				metric.InstrumentKindUpDownCounter:           metric.DefaultAggregationSelector(metric.InstrumentKindUpDownCounter),
				metric.InstrumentKindObservableCounter:       metric.DefaultAggregationSelector(metric.InstrumentKindObservableCounter),
				metric.InstrumentKindObservableUpDownCounter: metric.DefaultAggregationSelector(metric.InstrumentKindObservableUpDownCounter),
				metric.InstrumentKindObservableGauge:         metric.DefaultAggregationSelector(metric.InstrumentKindObservableGauge),
			},
		},
		{
			name:     "base2_exponential_bucket_histogram",
			envValue: "base2_exponential_bucket_histogram",
			want: map[metric.InstrumentKind]metric.Aggregation{
				metric.InstrumentKindCounter: metric.DefaultAggregationSelector(metric.InstrumentKindCounter),
				metric.InstrumentKindHistogram: metric.AggregationBase2ExponentialHistogram{
					MaxSize:  160,
					MaxScale: 20,
					NoMinMax: false,
				},
				metric.InstrumentKindUpDownCounter:           metric.DefaultAggregationSelector(metric.InstrumentKindUpDownCounter),
				metric.InstrumentKindObservableCounter:       metric.DefaultAggregationSelector(metric.InstrumentKindObservableCounter),
				metric.InstrumentKindObservableUpDownCounter: metric.DefaultAggregationSelector(metric.InstrumentKindObservableUpDownCounter),
				metric.InstrumentKindObservableGauge:         metric.DefaultAggregationSelector(metric.InstrumentKindObservableGauge),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			DefaultEnvOptionsReader.GetEnv = func(key string) string {
				if key == "OTEL_EXPORTER_OTLP_METRICS_DEFAULT_HISTOGRAM_AGGREGATION" {
					return tt.envValue
				}
				return origReader(key)
			}
			cfg := Config{}
			cfg = ApplyGRPCEnvConfigs(cfg)

			if tt.want == nil {
				// There is no function set, the SDK's default is used.
				assert.Nil(t, cfg.Metrics.AggregationSelector)
				return
			}

			require.NotNil(t, cfg.Metrics.AggregationSelector)
			for ik, want := range tt.want {
				assert.Equal(t, want, cfg.Metrics.AggregationSelector(ik))
			}
		})
	}
	DefaultEnvOptionsReader.GetEnv = origReader
}
