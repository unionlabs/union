/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
 * Modifications copyright (C) 2017 Andy Kimball and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package batchskl

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"testing"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

// iterAdapter adapts the new Iterator API which returns the key and value from
// positioning methods (Seek*, First, Last, Next, Prev) to the old API which
// returned a boolean corresponding to Valid. Only used by test code.
type iterAdapter struct {
	Iterator
}

func (i *iterAdapter) verify(key *base.InternalKey) bool {
	valid := key != nil
	if valid != i.Valid() {
		panic(fmt.Sprintf("inconsistent valid: %t != %t", valid, i.Valid()))
	}
	if valid {
		if base.InternalCompare(bytes.Compare, *key, i.Key()) != 0 {
			panic(fmt.Sprintf("inconsistent key: %s != %s", *key, i.Key()))
		}
	}
	return valid
}

func (i *iterAdapter) SeekGE(key []byte) bool {
	return i.verify(i.Iterator.SeekGE(key, base.SeekGEFlagsNone))
}

func (i *iterAdapter) SeekLT(key []byte) bool {
	return i.verify(i.Iterator.SeekLT(key))
}

func (i *iterAdapter) First() bool {
	return i.verify(i.Iterator.First())
}

func (i *iterAdapter) Last() bool {
	return i.verify(i.Iterator.Last())
}

func (i *iterAdapter) Next() bool {
	return i.verify(i.Iterator.Next())
}

func (i *iterAdapter) Prev() bool {
	return i.verify(i.Iterator.Prev())
}

func (i *iterAdapter) Key() base.InternalKey {
	return *i.Iterator.Key()
}

// length iterates over skiplist to give exact size.
func length(s *Skiplist) int {
	count := 0

	it := iterAdapter{s.NewIter(nil, nil)}
	for valid := it.First(); valid; valid = it.Next() {
		count++
	}

	return count
}

// length iterates over skiplist in reverse order to give exact size.
func lengthRev(s *Skiplist) int {
	count := 0

	it := iterAdapter{s.NewIter(nil, nil)}
	for valid := it.Last(); valid; valid = it.Prev() {
		count++
	}

	return count
}

func makeKey(s string) []byte {
	return []byte(s)
}

type testStorage struct {
	data []byte
}

func (d *testStorage) add(key string) uint32 {
	offset := uint32(len(d.data))
	d.data = append(d.data, uint8(base.InternalKeyKindSet))
	var buf [binary.MaxVarintLen64]byte
	n := binary.PutUvarint(buf[:], uint64(len(key)))
	d.data = append(d.data, buf[:n]...)
	d.data = append(d.data, key...)
	return offset
}

func (d *testStorage) addBytes(key []byte) uint32 {
	offset := uint32(len(d.data))
	d.data = append(d.data, uint8(base.InternalKeyKindSet))
	var buf [binary.MaxVarintLen64]byte
	n := binary.PutUvarint(buf[:], uint64(len(key)))
	d.data = append(d.data, buf[:n]...)
	d.data = append(d.data, key...)
	return offset
}

func newTestSkiplist(storage *testStorage) *Skiplist {
	return NewSkiplist(&storage.data, base.DefaultComparer.Compare,
		base.DefaultComparer.AbbreviatedKey)
}

func TestEmpty(t *testing.T) {
	key := makeKey("aaa")
	l := newTestSkiplist(&testStorage{})
	it := iterAdapter{l.NewIter(nil, nil)}

	require.False(t, it.Valid())

	it.First()
	require.False(t, it.Valid())

	it.Last()
	require.False(t, it.Valid())

	require.False(t, it.SeekGE(key))
	require.False(t, it.Valid())
}

// TestBasic tests seeks and adds.
func TestBasic(t *testing.T) {
	d := &testStorage{}
	l := newTestSkiplist(d)
	it := iterAdapter{l.NewIter(nil, nil)}

	// Try adding values.
	require.Nil(t, l.Add(d.add("key1")))
	require.Nil(t, l.Add(d.add("key2")))
	require.Nil(t, l.Add(d.add("key3")))

	require.True(t, it.SeekGE(makeKey("key")))
	require.EqualValues(t, "key1", it.Key().UserKey)

	require.True(t, it.SeekGE(makeKey("key1")))
	require.EqualValues(t, "key1", it.Key().UserKey)

	require.True(t, it.SeekGE(makeKey("key2")))
	require.EqualValues(t, "key2", it.Key().UserKey)

	require.True(t, it.SeekGE(makeKey("key3")))
	require.EqualValues(t, "key3", it.Key().UserKey)

	require.True(t, it.SeekGE(makeKey("key2")))
	require.True(t, it.SeekGE(makeKey("key3")))
}

func TestSkiplistAdd(t *testing.T) {
	d := &testStorage{}
	l := newTestSkiplist(d)
	it := iterAdapter{l.NewIter(nil, nil)}

	// Add empty key.
	require.Nil(t, l.Add(d.add("")))
	require.EqualValues(t, []byte(nil), it.Key().UserKey)
	require.True(t, it.First())
	require.EqualValues(t, []byte{}, it.Key().UserKey)

	// Add to empty list.
	require.Nil(t, l.Add(d.add("00002")))
	require.True(t, it.SeekGE(makeKey("00002")))
	require.EqualValues(t, "00002", it.Key().UserKey)

	// Add first element in non-empty list.
	require.Nil(t, l.Add(d.add("00001")))
	require.True(t, it.SeekGE(makeKey("00001")))
	require.EqualValues(t, "00001", it.Key().UserKey)

	// Add last element in non-empty list.
	require.Nil(t, l.Add(d.add("00004")))
	require.True(t, it.SeekGE(makeKey("00004")))
	require.EqualValues(t, "00004", it.Key().UserKey)

	// Add element in middle of list.
	require.Nil(t, l.Add(d.add("00003")))
	require.True(t, it.SeekGE(makeKey("00003")))
	require.EqualValues(t, "00003", it.Key().UserKey)

	// Try to add element that already exists.
	require.Nil(t, l.Add(d.add("00002")))
	require.Equal(t, 6, length(l))
	require.Equal(t, 6, lengthRev(l))
}

func TestSkiplistAdd_Overflow(t *testing.T) {
	// Regression test for cockroachdb/pebble#1258. The length of the nodes buffer
	// cannot exceed the maximum allowable size.
	d := &testStorage{}
	l := newTestSkiplist(d)

	// Simulate a full nodes slice. This speeds up the test significantly, as
	// opposed to adding data to the list.
	l.nodes = make([]byte, maxNodesSize)

	// Adding a new node to the list would overflow the nodes slice. Note that it
	// is the size of a new node struct that is relevant here, rather than the
	// size of the data being added to the list.
	err := l.Add(d.add("too much!"))
	require.Error(t, err)
	require.True(t, errors.Is(err, ErrTooManyRecords))
}

// TestIteratorNext tests a basic iteration over all nodes from the beginning.
func TestIteratorNext(t *testing.T) {
	const n = 100
	d := &testStorage{}
	l := newTestSkiplist(d)
	it := iterAdapter{l.NewIter(nil, nil)}

	require.False(t, it.Valid())

	it.First()
	require.False(t, it.Valid())

	for i := n - 1; i >= 0; i-- {
		require.Nil(t, l.Add(d.add(fmt.Sprintf("%05d", i))))
	}

	it.First()
	for i := 0; i < n; i++ {
		require.True(t, it.Valid())
		require.EqualValues(t, fmt.Sprintf("%05d", i), it.Key().UserKey)
		it.Next()
	}
	require.False(t, it.Valid())
}

// // TestIteratorPrev tests a basic iteration over all nodes from the end.
func TestIteratorPrev(t *testing.T) {
	const n = 100
	d := &testStorage{}
	l := newTestSkiplist(d)
	it := iterAdapter{l.NewIter(nil, nil)}

	require.False(t, it.Valid())

	it.Last()
	require.False(t, it.Valid())

	for i := 0; i < n; i++ {
		l.Add(d.add(fmt.Sprintf("%05d", i)))
	}

	it.Last()
	for i := n - 1; i >= 0; i-- {
		require.True(t, it.Valid())
		require.EqualValues(t, fmt.Sprintf("%05d", i), string(it.Key().UserKey))
		it.Prev()
	}
	require.False(t, it.Valid())
}

func TestIteratorSeekGE(t *testing.T) {
	const n = 1000
	d := &testStorage{}
	l := newTestSkiplist(d)
	it := iterAdapter{l.NewIter(nil, nil)}

	require.False(t, it.Valid())
	it.First()
	require.False(t, it.Valid())
	// 1000, 1010, 1020, ..., 1990.
	for i := n - 1; i >= 0; i-- {
		require.Nil(t, l.Add(d.add(fmt.Sprintf("%05d", i*10+1000))))
	}

	require.True(t, it.SeekGE(makeKey("")))
	require.True(t, it.Valid())
	require.EqualValues(t, "01000", it.Key().UserKey)

	require.True(t, it.SeekGE(makeKey("01000")))
	require.True(t, it.Valid())
	require.EqualValues(t, "01000", it.Key().UserKey)

	require.True(t, it.SeekGE(makeKey("01005")))
	require.True(t, it.Valid())
	require.EqualValues(t, "01010", it.Key().UserKey)

	require.True(t, it.SeekGE(makeKey("01010")))
	require.True(t, it.Valid())
	require.EqualValues(t, "01010", it.Key().UserKey)

	require.False(t, it.SeekGE(makeKey("99999")))
	require.False(t, it.Valid())

	// Test seek for empty key.
	require.Nil(t, l.Add(d.add("")))
	require.True(t, it.SeekGE([]byte{}))
	require.True(t, it.Valid())

	require.True(t, it.SeekGE(makeKey("")))
	require.True(t, it.Valid())
}

func TestIteratorSeekLT(t *testing.T) {
	const n = 100
	d := &testStorage{}
	l := newTestSkiplist(d)
	it := iterAdapter{l.NewIter(nil, nil)}

	require.False(t, it.Valid())
	it.First()
	require.False(t, it.Valid())
	// 1000, 1010, 1020, ..., 1990.
	for i := n - 1; i >= 0; i-- {
		require.Nil(t, l.Add(d.add(fmt.Sprintf("%05d", i*10+1000))))
	}

	require.False(t, it.SeekLT(makeKey("")))
	require.False(t, it.Valid())

	require.False(t, it.SeekLT(makeKey("01000")))
	require.False(t, it.Valid())

	require.True(t, it.SeekLT(makeKey("01001")))
	require.EqualValues(t, "01000", it.Key().UserKey)
	require.True(t, it.Valid())

	require.True(t, it.SeekLT(makeKey("01005")))
	require.EqualValues(t, "01000", it.Key().UserKey)
	require.True(t, it.Valid())

	require.True(t, it.SeekLT(makeKey("01991")))
	require.EqualValues(t, "01990", it.Key().UserKey)
	require.True(t, it.Valid())

	require.True(t, it.SeekLT(makeKey("99999")))
	require.True(t, it.Valid())
	require.EqualValues(t, "01990", it.Key().UserKey)

	// Test seek for empty key.
	require.Nil(t, l.Add(d.add("")))
	require.False(t, it.SeekLT([]byte{}))
	require.False(t, it.Valid())
	require.True(t, it.SeekLT(makeKey("\x01")))
	require.True(t, it.Valid())
	require.EqualValues(t, "", it.Key().UserKey)
}

// TODO(peter): test First and Last.
func TestIteratorBounds(t *testing.T) {
	d := &testStorage{}
	l := newTestSkiplist(d)
	for i := 1; i < 10; i++ {
		require.NoError(t, l.Add(d.add(fmt.Sprintf("%05d", i))))
	}

	it := iterAdapter{l.NewIter(makeKey("00003"), makeKey("00007"))}

	// SeekGE within the lower and upper bound succeeds.
	for i := 3; i <= 6; i++ {
		key := makeKey(fmt.Sprintf("%05d", i))
		require.True(t, it.SeekGE(key))
		require.EqualValues(t, string(key), string(it.Key().UserKey))
	}

	// SeekGE before the lower bound still succeeds (only the upper bound is
	// checked).
	for i := 1; i < 3; i++ {
		key := makeKey(fmt.Sprintf("%05d", i))
		require.True(t, it.SeekGE(key))
		require.EqualValues(t, string(key), string(it.Key().UserKey))
	}

	// SeekGE beyond the upper bound fails.
	for i := 7; i < 10; i++ {
		key := makeKey(fmt.Sprintf("%05d", i))
		require.False(t, it.SeekGE(key))
	}

	require.True(t, it.SeekGE(makeKey("00006")))
	require.EqualValues(t, "00006", it.Key().UserKey)

	// Next into the upper bound fails.
	require.False(t, it.Next())

	// SeekLT within the lower and upper bound succeeds.
	for i := 4; i <= 7; i++ {
		key := makeKey(fmt.Sprintf("%05d", i))
		require.True(t, it.SeekLT(key))
		require.EqualValues(t, fmt.Sprintf("%05d", i-1), string(it.Key().UserKey))
	}

	// SeekLT beyond the upper bound still succeeds (only the lower bound is
	// checked).
	for i := 8; i < 9; i++ {
		key := makeKey(fmt.Sprintf("%05d", i))
		require.True(t, it.SeekLT(key))
		require.EqualValues(t, fmt.Sprintf("%05d", i-1), string(it.Key().UserKey))
	}

	// SeekLT before the lower bound fails.
	for i := 1; i < 4; i++ {
		key := makeKey(fmt.Sprintf("%05d", i))
		require.False(t, it.SeekLT(key))
	}

	require.True(t, it.SeekLT(makeKey("00004")))
	require.EqualValues(t, "00003", it.Key().UserKey)

	// Prev into the lower bound fails.
	require.False(t, it.Prev())
}

func randomKey(rng *rand.Rand, b []byte) []byte {
	key := rng.Uint32()
	key2 := rng.Uint32()
	binary.LittleEndian.PutUint32(b, key)
	binary.LittleEndian.PutUint32(b[4:], key2)
	return b
}

// Standard test. Some fraction is read. Some fraction is write. Writes have
// to go through mutex lock.
func BenchmarkReadWrite(b *testing.B) {
	for i := 0; i <= 10; i++ {
		readFrac := float32(i) / 10.0
		b.Run(fmt.Sprintf("frac_%d", i*10), func(b *testing.B) {
			var buf [8]byte
			d := &testStorage{
				data: make([]byte, 0, b.N*10),
			}
			l := newTestSkiplist(d)
			it := l.NewIter(nil, nil)
			rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				key := randomKey(rng, buf[:])
				if rng.Float32() < readFrac {
					_ = it.SeekGE(key, base.SeekGEFlagsNone)
				} else {
					offset := d.addBytes(buf[:])
					_ = l.Add(offset)
				}
			}
			b.StopTimer()
		})
	}
}

func BenchmarkOrderedWrite(b *testing.B) {
	var buf [8]byte
	d := &testStorage{
		data: make([]byte, 0, b.N*10),
	}
	l := newTestSkiplist(d)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		binary.BigEndian.PutUint64(buf[:], uint64(i))
		offset := d.addBytes(buf[:])
		_ = l.Add(offset)
	}
}

func BenchmarkIterNext(b *testing.B) {
	var buf [8]byte
	d := &testStorage{
		data: make([]byte, 0, 64<<10),
	}
	l := newTestSkiplist(d)

	rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))
	for len(d.data)+20 < cap(d.data) {
		key := randomKey(rng, buf[:])
		offset := d.addBytes(key)
		err := l.Add(offset)
		require.NoError(b, err)
	}

	it := l.NewIter(nil, nil)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if !it.Valid() {
			it.First()
		}
		it.Next()
	}
}

func BenchmarkIterPrev(b *testing.B) {
	var buf [8]byte
	d := &testStorage{
		data: make([]byte, 0, 64<<10),
	}
	l := newTestSkiplist(d)

	rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))
	for len(d.data)+20 < cap(d.data) {
		key := randomKey(rng, buf[:])
		offset := d.addBytes(key)
		err := l.Add(offset)
		require.NoError(b, err)
	}

	it := l.NewIter(nil, nil)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if !it.Valid() {
			it.Last()
		}
		it.Prev()
	}
}
