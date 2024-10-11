// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestThroughputMetric(t *testing.T) {
	m1 := ThroughputMetric{
		Bytes:        10,
		WorkDuration: time.Millisecond,
		IdleDuration: 9 * time.Millisecond,
	}
	var m2 ThroughputMetric
	m2.Merge(m1)
	require.Equal(t, m1, m2)
	m2.Merge(m1)
	doubleM1 := ThroughputMetric{
		Bytes:        2 * m1.Bytes,
		WorkDuration: 2 * m1.WorkDuration,
		IdleDuration: 2 * m1.IdleDuration,
	}
	require.Equal(t, doubleM1, m2)
	require.EqualValues(t, 10*100, m1.Rate())
	require.EqualValues(t, 10*1000, m1.PeakRate())
}

func TestThroughputMetric_Subtract(t *testing.T) {
	m1 := ThroughputMetric{
		Bytes:        10,
		WorkDuration: time.Millisecond,
		IdleDuration: 9 * time.Millisecond,
	}
	m2 := ThroughputMetric{
		Bytes:        100,
		WorkDuration: time.Millisecond,
		IdleDuration: 90 * time.Millisecond,
	}

	m2.Subtract(m1)
	require.Equal(t, int64(90), m2.Bytes)
	require.Equal(t, 0*time.Millisecond, m2.WorkDuration)
	require.Equal(t, 81*time.Millisecond, m2.IdleDuration)
}

func TestGaugeSampleMetric(t *testing.T) {
	g1 := GaugeSampleMetric{}
	g1.AddSample(10)
	g1.AddSample(20)
	g2 := GaugeSampleMetric{}
	g2.Merge(g1)
	g2.AddSample(60)
	require.EqualValues(t, 30, g2.Mean())
	require.EqualValues(t, 3, g2.count)
	require.EqualValues(t, 15, g1.Mean())
	require.EqualValues(t, 2, g1.count)
}

func TestGaugeSampleMetricSubtract(t *testing.T) {
	g1 := GaugeSampleMetric{}
	g2 := GaugeSampleMetric{}
	g1.AddSample(10)
	g1.AddSample(20)
	g1.AddSample(0)

	g2.AddSample(10)

	g1.Subtract(g2)

	require.Equal(t, int64(20), g1.sampleSum)
	require.Equal(t, int64(2), g1.count)

}
