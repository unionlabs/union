// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import "time"

// ThroughputMetric is used to measure the byte throughput of some component
// that performs work in a single-threaded manner. The throughput can be
// approximated by Bytes/(WorkDuration+IdleTime). The idle time is represented
// separately, so that the user of this metric could approximate the peak
// throughput as Bytes/WorkTime. The metric is designed to be cumulative (see
// Merge).
type ThroughputMetric struct {
	// Bytes is the processes bytes by the component.
	Bytes int64
	// WorkDuration is the duration that the component spent doing work.
	WorkDuration time.Duration
	// IdleDuration is the duration that the component was idling, waiting for
	// work.
	IdleDuration time.Duration
}

// Merge accumulates the information from another throughput metric.
func (tm *ThroughputMetric) Merge(x ThroughputMetric) {
	tm.Bytes += x.Bytes
	tm.WorkDuration += x.WorkDuration
	tm.IdleDuration += x.IdleDuration
}

// Subtract subtracts the information from another ThroughputMetric
func (tm *ThroughputMetric) Subtract(x ThroughputMetric) {
	tm.Bytes -= x.Bytes
	tm.WorkDuration -= x.WorkDuration
	tm.IdleDuration -= x.IdleDuration
}

// PeakRate returns the approximate peak rate if there was no idling.
func (tm *ThroughputMetric) PeakRate() int64 {
	if tm.Bytes == 0 {
		return 0
	}
	return int64((float64(tm.Bytes) / float64(tm.WorkDuration)) * float64(time.Second))
}

// Rate returns the observed rate.
func (tm *ThroughputMetric) Rate() int64 {
	if tm.Bytes == 0 {
		return 0
	}
	return int64((float64(tm.Bytes) / float64(tm.WorkDuration+tm.IdleDuration)) *
		float64(time.Second))
}

// Utilization returns a percent [0, 1.0] indicating the percent of time
// work was performed.
func (tm *ThroughputMetric) Utilization() float64 {
	if tm.WorkDuration == 0 {
		return 0
	}
	return float64(tm.WorkDuration) / float64(tm.WorkDuration+tm.IdleDuration)
}

// GaugeSampleMetric is used to measure a gauge value (e.g. queue length) by
// accumulating samples of that gauge.
type GaugeSampleMetric struct {
	// The sum of all the samples.
	sampleSum int64
	// The number of samples.
	count int64
}

// AddSample adds the given sample.
func (gsm *GaugeSampleMetric) AddSample(sample int64) {
	gsm.sampleSum += sample
	gsm.count++
}

// Merge accumulates the information from another gauge metric.
func (gsm *GaugeSampleMetric) Merge(x GaugeSampleMetric) {
	gsm.sampleSum += x.sampleSum
	gsm.count += x.count
}

// Subtract subtracts the information from another gauge metric.
func (gsm *GaugeSampleMetric) Subtract(x GaugeSampleMetric) {
	gsm.sampleSum -= x.sampleSum
	gsm.count -= x.count
}

// Mean returns the mean value.
func (gsm *GaugeSampleMetric) Mean() float64 {
	if gsm.count == 0 {
		return 0
	}
	return float64(gsm.sampleSum) / float64(gsm.count)
}
