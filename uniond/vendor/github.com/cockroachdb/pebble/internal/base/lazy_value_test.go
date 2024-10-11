// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import (
	"bytes"
	"testing"
	"unsafe"

	"github.com/stretchr/testify/require"
)

type valueFetcherFunc func(
	handle []byte, valLen int32, buf []byte) (val []byte, callerOwned bool, err error)

func (v valueFetcherFunc) Fetch(
	handle []byte, valLen int32, buf []byte,
) (val []byte, callerOwned bool, err error) {
	return v(handle, valLen, buf)
}

func TestLazyValue(t *testing.T) {
	// Both 40 and 48 bytes makes iteration benchmarks like
	// BenchmarkIteratorScan/keys=1000,r-amp=1,key-types=points-only 75%
	// slower.
	require.True(t, unsafe.Sizeof(LazyValue{}) <= 32)

	fooBytes1 := []byte("foo")
	fooLV1 := MakeInPlaceValue(fooBytes1)
	require.Equal(t, 3, fooLV1.Len())
	_, hasAttr := fooLV1.TryGetShortAttribute()
	require.False(t, hasAttr)
	fooLV2, fooBytes2 := fooLV1.Clone(nil, &LazyFetcher{})
	require.Equal(t, 3, fooLV2.Len())
	_, hasAttr = fooLV2.TryGetShortAttribute()
	require.False(t, hasAttr)
	require.Equal(t, fooLV1.InPlaceValue(), fooLV2.InPlaceValue())
	getValue := func(lv LazyValue, expectedCallerOwned bool) []byte {
		v, callerOwned, err := lv.Value(nil)
		require.NoError(t, err)
		require.Equal(t, expectedCallerOwned, callerOwned)
		return v
	}
	require.Equal(t, getValue(fooLV1, false), getValue(fooLV2, false))
	fooBytes2[0] = 'b'
	require.False(t, bytes.Equal(fooLV1.InPlaceValue(), fooLV2.InPlaceValue()))

	for _, callerOwned := range []bool{false, true} {
		numCalls := 0
		fooLV3 := LazyValue{
			ValueOrHandle: []byte("foo-handle"),
			Fetcher: &LazyFetcher{
				Fetcher: valueFetcherFunc(
					func(handle []byte, valLen int32, buf []byte) ([]byte, bool, error) {
						numCalls++
						require.Equal(t, []byte("foo-handle"), handle)
						require.Equal(t, int32(3), valLen)
						return fooBytes1, callerOwned, nil
					}),
				Attribute: AttributeAndLen{ValueLen: 3, ShortAttribute: 7},
			},
		}
		require.Equal(t, []byte("foo"), getValue(fooLV3, callerOwned))
		require.Equal(t, 1, numCalls)
		require.Equal(t, []byte("foo"), getValue(fooLV3, callerOwned))
		require.Equal(t, 1, numCalls)
		require.Equal(t, 3, fooLV3.Len())
		attr, hasAttr := fooLV3.TryGetShortAttribute()
		require.True(t, hasAttr)
		require.Equal(t, ShortAttribute(7), attr)
	}
}
