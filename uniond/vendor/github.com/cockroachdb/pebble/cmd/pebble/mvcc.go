// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"bytes"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/bytealloc"
)

// MVCC encoding and decoding routines adapted from CockroachDB sources. Used
// to perform apples-to-apples benchmarking for CockroachDB's usage of RocksDB.

var mvccComparer = &pebble.Comparer{
	Compare: mvccCompare,

	AbbreviatedKey: func(k []byte) uint64 {
		key, _, ok := mvccSplitKey(k)
		if !ok {
			return 0
		}
		return pebble.DefaultComparer.AbbreviatedKey(key)
	},

	Equal: func(a, b []byte) bool {
		return mvccCompare(a, b) == 0
	},

	Separator: func(dst, a, b []byte) []byte {
		aKey, _, ok := mvccSplitKey(a)
		if !ok {
			return append(dst, a...)
		}
		bKey, _, ok := mvccSplitKey(b)
		if !ok {
			return append(dst, a...)
		}
		// If the keys are the same just return a.
		if bytes.Equal(aKey, bKey) {
			return append(dst, a...)
		}
		n := len(dst)
		// MVCC key comparison uses bytes.Compare on the roachpb.Key, which is the same semantics as
		// pebble.DefaultComparer, so reuse the latter's Separator implementation.
		dst = pebble.DefaultComparer.Separator(dst, aKey, bKey)
		// Did it pick a separator different than aKey -- if it did not we can't do better than a.
		buf := dst[n:]
		if bytes.Equal(aKey, buf) {
			return append(dst[:n], a...)
		}
		// The separator is > aKey, so we only need to add the timestamp sentinel.
		return append(dst, 0)
	},

	Successor: func(dst, a []byte) []byte {
		aKey, _, ok := mvccSplitKey(a)
		if !ok {
			return append(dst, a...)
		}
		n := len(dst)
		// MVCC key comparison uses bytes.Compare on the roachpb.Key, which is the same semantics as
		// pebble.DefaultComparer, so reuse the latter's Successor implementation.
		dst = pebble.DefaultComparer.Successor(dst, aKey)
		// Did it pick a successor different than aKey -- if it did not we can't do better than a.
		buf := dst[n:]
		if bytes.Equal(aKey, buf) {
			return append(dst[:n], a...)
		}
		// The successor is > aKey, so we only need to add the timestamp sentinel.
		return append(dst, 0)
	},

	Split: func(k []byte) int {
		key, _, ok := mvccSplitKey(k)
		if !ok {
			return len(k)
		}
		// This matches the behavior of libroach/KeyPrefix. RocksDB requires that
		// keys generated via a SliceTransform be comparable with normal encoded
		// MVCC keys. Encoded MVCC keys have a suffix indicating the number of
		// bytes of timestamp data. MVCC keys without a timestamp have a suffix of
		// 0. We're careful in EncodeKey to make sure that the user-key always has
		// a trailing 0. If there is no timestamp this falls out naturally. If
		// there is a timestamp we prepend a 0 to the encoded timestamp data.
		return len(key) + 1
	},

	Name: "cockroach_comparator",
}

func mvccSplitKey(mvccKey []byte) (key []byte, ts []byte, ok bool) {
	if len(mvccKey) == 0 {
		return nil, nil, false
	}
	n := len(mvccKey) - 1
	tsLen := int(mvccKey[n])
	if n < tsLen {
		return nil, nil, false
	}
	key = mvccKey[:n-tsLen]
	if tsLen > 0 {
		ts = mvccKey[n-tsLen+1 : len(mvccKey)-1]
	}
	return key, ts, true
}

func mvccCompare(a, b []byte) int {
	// NB: For performance, this routine manually splits the key into the
	// user-key and timestamp components rather than using SplitMVCCKey. Don't
	// try this at home kids: use SplitMVCCKey.

	aEnd := len(a) - 1
	bEnd := len(b) - 1
	if aEnd < 0 || bEnd < 0 {
		// This should never happen unless there is some sort of corruption of
		// the keys. This is a little bizarre, but the behavior exactly matches
		// engine/db.cc:DBComparator.
		return bytes.Compare(a, b)
	}

	// Compute the index of the separator between the key and the timestamp.
	aSep := aEnd - int(a[aEnd])
	bSep := bEnd - int(b[bEnd])
	if aSep < 0 || bSep < 0 {
		// This should never happen unless there is some sort of corruption of
		// the keys. This is a little bizarre, but the behavior exactly matches
		// engine/db.cc:DBComparator.
		return bytes.Compare(a, b)
	}

	// Compare the "user key" part of the key.
	if c := bytes.Compare(a[:aSep], b[:bSep]); c != 0 {
		return c
	}

	// Compare the timestamp part of the key.
	aTS := a[aSep:aEnd]
	bTS := b[bSep:bEnd]
	if len(aTS) == 0 {
		if len(bTS) == 0 {
			return 0
		}
		return -1
	} else if len(bTS) == 0 {
		return 1
	}
	return bytes.Compare(bTS, aTS)
}

// <key>\x00[<wall_time>[<logical>]]<#timestamp-bytes>
func mvccEncode(dst, key []byte, walltime uint64, logical uint32) []byte {
	dst = append(dst, key...)
	dst = append(dst, 0)
	if walltime != 0 || logical != 0 {
		extra := byte(1 + 8)
		dst = encodeUint64Ascending(dst, walltime)
		if logical != 0 {
			dst = encodeUint32Ascending(dst, logical)
			extra += 4
		}
		dst = append(dst, extra)
	}
	return dst
}

func mvccForwardScan(d DB, start, end, ts []byte) (int, int64) {
	it := d.NewIter(&pebble.IterOptions{
		LowerBound: mvccEncode(nil, start, 0, 0),
		UpperBound: mvccEncode(nil, end, 0, 0),
	})
	defer it.Close()

	var data bytealloc.A
	var count int
	var nbytes int64

	for valid := it.First(); valid; valid = it.Next() {
		key, keyTS, _ := mvccSplitKey(it.Key())
		if bytes.Compare(keyTS, ts) <= 0 {
			data, _ = data.Copy(key)
			data, _ = data.Copy(it.Value())
		}
		count++
		nbytes += int64(len(it.Key()) + len(it.Value()))
	}
	return count, nbytes
}

func mvccReverseScan(d DB, start, end, ts []byte) (int, int64) {
	it := d.NewIter(&pebble.IterOptions{
		LowerBound: mvccEncode(nil, start, 0, 0),
		UpperBound: mvccEncode(nil, end, 0, 0),
	})
	defer it.Close()

	var data bytealloc.A
	var count int
	var nbytes int64

	for valid := it.Last(); valid; valid = it.Prev() {
		key, keyTS, _ := mvccSplitKey(it.Key())
		if bytes.Compare(keyTS, ts) <= 0 {
			data, _ = data.Copy(key)
			data, _ = data.Copy(it.Value())
		}
		count++
		nbytes += int64(len(it.Key()) + len(it.Value()))
	}
	return count, nbytes
}

var fauxMVCCMerger = &pebble.Merger{
	Name: "cockroach_merge_operator",
	Merge: func(key, value []byte) (pebble.ValueMerger, error) {
		// This merger is used by the compact benchmark and use the
		// pebble default value merger to concatenate values.
		// It shouldn't materially affect the benchmarks.
		return pebble.DefaultMerger.Merge(key, value)
	},
}
