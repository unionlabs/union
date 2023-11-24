// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import "sync/atomic"

// FilterMetrics holds metrics for the filter policy.
type FilterMetrics struct {
	// The number of hits for the filter policy. This is the
	// number of times the filter policy was successfully used to avoid access
	// of a data block.
	Hits int64
	// The number of misses for the filter policy. This is the number of times
	// the filter policy was checked but was unable to filter an access of a data
	// block.
	Misses int64
}

// FilterMetricsTracker is used to keep track of filter metrics. It contains the
// same metrics as FilterMetrics, but they can be updated atomically. An
// instance of FilterMetricsTracker can be passed to a Reader as a ReaderOption.
type FilterMetricsTracker struct {
	// See FilterMetrics.Hits.
	hits atomic.Int64
	// See FilterMetrics.Misses.
	misses atomic.Int64
}

var _ ReaderOption = (*FilterMetricsTracker)(nil)

func (m *FilterMetricsTracker) readerApply(r *Reader) {
	if r.tableFilter != nil {
		r.tableFilter.metrics = m
	}
}

// Load returns the current values as FilterMetrics.
func (m *FilterMetricsTracker) Load() FilterMetrics {
	return FilterMetrics{
		Hits:   m.hits.Load(),
		Misses: m.misses.Load(),
	}
}

// BlockHandle is the file offset and length of a block.
type BlockHandle struct {
	Offset, Length uint64
}

// BlockHandleWithProperties is used for data blocks and first/lower level
// index blocks, since they can be annotated using BlockPropertyCollectors.
type BlockHandleWithProperties struct {
	BlockHandle
	Props []byte
}

type filterWriter interface {
	addKey(key []byte)
	finish() ([]byte, error)
	metaName() string
	policyName() string
}

type tableFilterReader struct {
	policy  FilterPolicy
	metrics *FilterMetricsTracker
}

func newTableFilterReader(policy FilterPolicy) *tableFilterReader {
	return &tableFilterReader{
		policy:  policy,
		metrics: nil,
	}
}

func (f *tableFilterReader) mayContain(data, key []byte) bool {
	mayContain := f.policy.MayContain(TableFilter, data, key)
	if f.metrics != nil {
		if mayContain {
			f.metrics.misses.Add(1)
		} else {
			f.metrics.hits.Add(1)
		}
	}
	return mayContain
}

type tableFilterWriter struct {
	policy FilterPolicy
	writer FilterWriter
	// count is the count of the number of keys added to the filter.
	count int
}

func newTableFilterWriter(policy FilterPolicy) *tableFilterWriter {
	return &tableFilterWriter{
		policy: policy,
		writer: policy.NewWriter(TableFilter),
	}
}

func (f *tableFilterWriter) addKey(key []byte) {
	f.count++
	f.writer.AddKey(key)
}

func (f *tableFilterWriter) finish() ([]byte, error) {
	if f.count == 0 {
		return nil, nil
	}
	return f.writer.Finish(nil), nil
}

func (f *tableFilterWriter) metaName() string {
	return "fullfilter." + f.policy.Name()
}

func (f *tableFilterWriter) policyName() string {
	return f.policy.Name()
}
