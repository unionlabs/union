// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"math"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/testkeys"
)

// Code in this file contains utils for testing. It implements interval block
// property collectors and filters on the suffixes of keys in the format used
// by the testkeys package (eg, 'key@5').

const testKeysBlockPropertyName = `pebble.internal.testkeys.suffixes`

// NewTestKeysBlockPropertyCollector constructs a sstable property collector
// over testkey suffixes.
func NewTestKeysBlockPropertyCollector() BlockPropertyCollector {
	return NewBlockIntervalCollector(
		testKeysBlockPropertyName,
		&testKeysSuffixIntervalCollector{},
		nil)
}

// NewTestKeysBlockPropertyFilter constructs a new block-property filter that excludes
// blocks containing exclusively suffixed keys where all the suffixes fall
// outside of the range [filterMin, filterMax).
//
// The filter only filters based on data derived from the key. The iteration
// results of this block property filter are deterministic for unsuffixed keys
// and keys with suffixes within the range [filterMin, filterMax). For keys with
// suffixes outside the range, iteration is nondeterministic.
func NewTestKeysBlockPropertyFilter(filterMin, filterMax uint64) *BlockIntervalFilter {
	return NewBlockIntervalFilter(testKeysBlockPropertyName, filterMin, filterMax)
}

// NewTestKeysMaskingFilter constructs a TestKeysMaskingFilter that implements
// pebble.BlockPropertyFilterMask for efficient range-key masking using the
// testkeys block property filter. The masking filter wraps a block interval
// filter, and modifies the configured interval when Pebble requests it.
func NewTestKeysMaskingFilter() TestKeysMaskingFilter {
	return TestKeysMaskingFilter{BlockIntervalFilter: NewTestKeysBlockPropertyFilter(0, math.MaxUint64)}
}

// TestKeysMaskingFilter implements BlockPropertyFilterMask and may be used to mask
// point keys with the testkeys-style suffixes (eg, @4) that are masked by range
// keys with testkeys-style suffixes.
type TestKeysMaskingFilter struct {
	*BlockIntervalFilter
}

// SetSuffix implements pebble.BlockPropertyFilterMask.
func (f TestKeysMaskingFilter) SetSuffix(suffix []byte) error {
	ts, err := testkeys.ParseSuffix(suffix)
	if err != nil {
		return err
	}
	f.BlockIntervalFilter.SetInterval(uint64(ts), math.MaxUint64)
	return nil
}

// Intersects implements the BlockPropertyFilter interface.
func (f TestKeysMaskingFilter) Intersects(prop []byte) (bool, error) {
	return f.BlockIntervalFilter.Intersects(prop)
}

var _ DataBlockIntervalCollector = (*testKeysSuffixIntervalCollector)(nil)

// testKeysSuffixIntervalCollector maintains an interval over the timestamps in
// MVCC-like suffixes for keys (e.g. foo@123).
type testKeysSuffixIntervalCollector struct {
	initialized  bool
	lower, upper uint64
}

// Add implements DataBlockIntervalCollector by adding the timestamp(s) in the
// suffix(es) of this record to the current interval.
//
// Note that range sets and unsets may have multiple suffixes. Range key deletes
// do not have a suffix. All other point keys have a single suffix.
func (c *testKeysSuffixIntervalCollector) Add(key base.InternalKey, value []byte) error {
	i := testkeys.Comparer.Split(key.UserKey)
	if i == len(key.UserKey) {
		c.initialized = true
		c.lower, c.upper = 0, math.MaxUint64
		return nil
	}
	ts, err := testkeys.ParseSuffix(key.UserKey[i:])
	if err != nil {
		return err
	}
	uts := uint64(ts)
	if !c.initialized {
		c.lower, c.upper = uts, uts+1
		c.initialized = true
		return nil
	}
	if uts < c.lower {
		c.lower = uts
	}
	if uts >= c.upper {
		c.upper = uts + 1
	}
	return nil
}

// FinishDataBlock implements DataBlockIntervalCollector.
func (c *testKeysSuffixIntervalCollector) FinishDataBlock() (lower, upper uint64, err error) {
	l, u := c.lower, c.upper
	c.lower, c.upper = 0, 0
	c.initialized = false
	return l, u, nil
}
