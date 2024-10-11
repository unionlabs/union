// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package replay

import (
	"math"
	"time"

	"github.com/guptarohit/asciigraph"
)

// SampledMetric holds a metric that is sampled at various points of workload
// replay. Samples are collected when a new step in the workload is applied to
// the database, and whenever a compaction completes.
type SampledMetric struct {
	samples []sample
	first   time.Time
}

type sample struct {
	since time.Duration
	value int64
}

func (m *SampledMetric) record(v int64) {
	if m.first.IsZero() {
		m.first = time.Now()
	}
	m.samples = append(m.samples, sample{
		since: time.Since(m.first),
		value: v,
	})
}

// Plot returns an ASCII graph plot of the metric over time, with the provided
// width and height determining the size of the graph and the number of representable discrete x and y
// points. All values are first
// multiplied by the provided scale parameter before graphing.
func (m *SampledMetric) Plot(width, height int, scale float64) string {
	values := m.Values(width)
	for i := range values {
		values[i] *= scale
	}
	return asciigraph.Plot(values, asciigraph.Height(height))
}

// PlotIncreasingPerSec returns an ASCII graph plot of the increasing delta of a
// metric over time, per-second. The provided width and height determine the
// size of the graph and the number of representable discrete x and y points.
// All deltas are multiplied by the provided scale parameter and scaled to
// per-second before graphing.
func (m *SampledMetric) PlotIncreasingPerSec(width, height int, scale float64) string {
	bucketDur, values := m.values(width)
	deltas := make([]float64, width)
	for i := range values {
		if i == 0 {
			deltas[i] = (values[i] * scale) / bucketDur.Seconds()
		} else if values[i] > values[i-1] {
			deltas[i] = (values[i] - values[i-1]) * scale / bucketDur.Seconds()
		}
	}
	return asciigraph.Plot(deltas, asciigraph.Height(height))
}

// Mean calculates the mean value of the metric.
func (m *SampledMetric) Mean() float64 {
	var sum float64
	if len(m.samples) == 0 {
		return 0.0
	}
	for _, s := range m.samples {
		sum += float64(s.value)
	}
	return sum / float64(len(m.samples))
}

// Min calculates the mininum value of the metric.
func (m *SampledMetric) Min() int64 {
	min := int64(math.MaxInt64)
	for _, s := range m.samples {
		if min > s.value {
			min = s.value
		}
	}
	return min
}

// Max calculates the maximum value of the metric.
func (m *SampledMetric) Max() int64 {
	var max int64
	for _, s := range m.samples {
		if max < s.value {
			max = s.value
		}
	}
	return max
}

// Values returns the values of the metric, distributed across n discrete
// buckets that are equally spaced over time. If multiple values fall within a
// bucket, the latest recorded value is used. If no values fall within a bucket,
// the next recorded value is used.
func (m *SampledMetric) Values(n int) []float64 {
	_, values := m.values(n)
	return values
}

func (m *SampledMetric) values(buckets int) (bucketDur time.Duration, values []float64) {
	if len(m.samples) == 0 || buckets < 1 {
		return bucketDur, nil
	}

	values = make([]float64, buckets)
	totalDur := m.samples[len(m.samples)-1].since
	bucketDur = totalDur / time.Duration(buckets)

	for i, b := 0, 0; i < len(m.samples); i++ {
		// Fill any buckets that precede this value with the previous value.
		bi := int(m.samples[i].since / bucketDur)
		if bi == buckets {
			bi = buckets - 1
		}
		if b < bi {
			b++
			for ; b < bi; b++ {
				values[b] = float64(m.samples[i].value)
			}
		}
		values[bi] = float64(m.samples[i].value)
		b = bi
	}
	return bucketDur, values
}
