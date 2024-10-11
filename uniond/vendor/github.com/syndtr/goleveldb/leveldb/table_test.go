// Copyright (c) 2019, Suryandaru Triandana <syndtr@gmail.com>
// All rights reserved.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package leveldb

import (
	"encoding/binary"
	"math/rand"
	"reflect"
	"testing"

	"github.com/onsi/gomega"
	"github.com/syndtr/goleveldb/leveldb/storage"
	"github.com/syndtr/goleveldb/leveldb/testutil"
)

func TestGetOverlaps(t *testing.T) {
	gomega.RegisterTestingT(t)
	stor := testutil.NewStorage()
	defer stor.Close()
	s, err := newSession(stor, nil)
	if err != nil {
		t.Fatal(err)
	}

	v := newVersion(s)
	v.newStaging()

	tmp := make([]byte, 4)
	mik := func(i uint64, typ keyType, ukey bool) []byte {
		if i == 0 {
			return nil
		}
		binary.BigEndian.PutUint32(tmp, uint32(i))
		if ukey {
			key := make([]byte, 4)
			copy(key, tmp)
			return key
		}
		return []byte(makeInternalKey(nil, tmp, 0, typ))
	}

	rec := &sessionRecord{}
	for i, f := range []struct {
		min   uint64
		max   uint64
		level int
	}{
		// Overlapped level 0 files
		{1, 8, 0},
		{4, 5, 0},
		{6, 10, 0},
		// Non-overlapped level 1 files
		{2, 3, 1},
		{8, 10, 1},
		{13, 13, 1},
		{20, 100, 1},
	} {
		rec.addTable(f.level, int64(i), 1, mik(f.min, keyTypeVal, false), mik(f.max, keyTypeVal, false))
	}
	vs := v.newStaging()
	vs.commit(rec)
	v = vs.finish(false)

	for i, x := range []struct {
		min      uint64
		max      uint64
		level    int
		expected []int64
	}{
		// Level0 cases
		{0, 0, 0, []int64{2, 1, 0}},
		{1, 0, 0, []int64{2, 1, 0}},
		{0, 10, 0, []int64{2, 1, 0}},
		{2, 7, 0, []int64{2, 1, 0}},

		// Level1 cases
		{1, 1, 1, nil},
		{0, 100, 1, []int64{3, 4, 5, 6}},
		{5, 0, 1, []int64{4, 5, 6}},
		{5, 4, 1, nil}, // invalid search space
		{1, 13, 1, []int64{3, 4, 5}},
		{2, 13, 1, []int64{3, 4, 5}},
		{3, 13, 1, []int64{3, 4, 5}},
		{4, 13, 1, []int64{4, 5}},
		{4, 19, 1, []int64{4, 5}},
		{4, 20, 1, []int64{4, 5, 6}},
		{4, 100, 1, []int64{4, 5, 6}},
		{4, 105, 1, []int64{4, 5, 6}},
	} {
		tf := v.levels[x.level]
		res := tf.getOverlaps(nil, s.icmp, mik(x.min, keyTypeSeek, true), mik(x.max, keyTypeSeek, true), x.level == 0)

		var fnums []int64
		for _, f := range res {
			fnums = append(fnums, f.fd.Num)
		}
		if !reflect.DeepEqual(x.expected, fnums) {
			t.Errorf("case %d failed, expected %v, got %v", i, x.expected, fnums)
		}
	}
}

func BenchmarkGetOverlapLevel0(b *testing.B) {
	benchmarkGetOverlap(b, 0, 500000)
}

func BenchmarkGetOverlapNonLevel0(b *testing.B) {
	benchmarkGetOverlap(b, 1, 500000)
}

func benchmarkGetOverlap(b *testing.B, level int, size int) {
	stor := storage.NewMemStorage()
	defer stor.Close()
	s, err := newSession(stor, nil)
	if err != nil {
		b.Fatal(err)
	}

	v := newVersion(s)
	v.newStaging()

	tmp := make([]byte, 4)
	mik := func(i uint64, typ keyType, ukey bool) []byte {
		if i == 0 {
			return nil
		}
		binary.BigEndian.PutUint32(tmp, uint32(i))
		if ukey {
			key := make([]byte, 4)
			copy(key, tmp)
			return key
		}
		return []byte(makeInternalKey(nil, tmp, 0, typ))
	}

	rec := &sessionRecord{}
	for i := 1; i <= size; i++ {
		min := mik(uint64(2*i), keyTypeVal, false)
		max := mik(uint64(2*i+1), keyTypeVal, false)
		rec.addTable(level, int64(i), 1, min, max)
	}
	vs := v.newStaging()
	vs.commit(rec)
	v = vs.finish(false)

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		files := v.levels[level]
		start := rand.Intn(size)
		end := rand.Intn(size-start) + start
		files.getOverlaps(nil, s.icmp, mik(uint64(2*start), keyTypeVal, true), mik(uint64(2*end), keyTypeVal, true), level == 0)
	}
}
