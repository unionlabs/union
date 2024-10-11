/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
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

package table

import (
	"sort"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/y"
)

type SimpleIterator struct {
	keys     [][]byte
	vals     [][]byte
	idx      int
	reversed bool
}

var (
	closeCount int
)

func (s *SimpleIterator) Close() error { closeCount++; return nil }

func (s *SimpleIterator) Next() {
	if !s.reversed {
		s.idx++
	} else {
		s.idx--
	}
}

func (s *SimpleIterator) Rewind() {
	if !s.reversed {
		s.idx = 0
	} else {
		s.idx = len(s.keys) - 1
	}
}

func (s *SimpleIterator) Seek(key []byte) {
	key = y.KeyWithTs(key, 0)
	if !s.reversed {
		s.idx = sort.Search(len(s.keys), func(i int) bool {
			return y.CompareKeys(s.keys[i], key) >= 0
		})
	} else {
		n := len(s.keys)
		s.idx = n - 1 - sort.Search(n, func(i int) bool {
			return y.CompareKeys(s.keys[n-1-i], key) <= 0
		})
	}
}

func (s *SimpleIterator) Key() []byte { return s.keys[s.idx] }
func (s *SimpleIterator) Value() y.ValueStruct {
	return y.ValueStruct{
		Value:    s.vals[s.idx],
		UserMeta: 55,
		Meta:     0,
	}
}
func (s *SimpleIterator) Valid() bool {
	return s.idx >= 0 && s.idx < len(s.keys)
}

var _ y.Iterator = &SimpleIterator{}

func newSimpleIterator(keys []string, vals []string, reversed bool) *SimpleIterator {
	k := make([][]byte, len(keys))
	v := make([][]byte, len(vals))
	y.AssertTrue(len(keys) == len(vals))
	for i := 0; i < len(keys); i++ {
		k[i] = y.KeyWithTs([]byte(keys[i]), 0)
		v[i] = []byte(vals[i])
	}
	return &SimpleIterator{
		keys:     k,
		vals:     v,
		idx:      -1,
		reversed: reversed,
	}
}

func getAll(it y.Iterator) ([]string, []string) {
	var keys, vals []string
	for ; it.Valid(); it.Next() {
		k := it.Key()
		keys = append(keys, string(y.ParseKey(k)))
		v := it.Value()
		vals = append(vals, string(v.Value))
	}
	return keys, vals
}

func closeAndCheck(t *testing.T, it y.Iterator, expected int) {
	closeCount = 0
	it.Close()
	require.EqualValues(t, expected, closeCount)
}

func TestSimpleIterator(t *testing.T) {
	keys := []string{"1", "2", "3"}
	vals := []string{"v1", "v2", "v3"}
	it := newSimpleIterator(keys, vals, false)
	it.Rewind()
	k, v := getAll(it)
	require.EqualValues(t, keys, k)
	require.EqualValues(t, vals, v)

	closeAndCheck(t, it, 1)
}

func reversed(a []string) []string {
	var out []string
	for i := len(a) - 1; i >= 0; i-- {
		out = append(out, a[i])
	}
	return out
}

func TestMergeSingle(t *testing.T) {
	keys := []string{"1", "2", "3"}
	vals := []string{"v1", "v2", "v3"}
	it := newSimpleIterator(keys, vals, false)
	mergeIt := NewMergeIterator([]y.Iterator{it}, false)
	mergeIt.Rewind()
	k, v := getAll(mergeIt)
	require.EqualValues(t, keys, k)
	require.EqualValues(t, vals, v)
	closeAndCheck(t, mergeIt, 1)
}

func TestMergeSingleReversed(t *testing.T) {
	keys := []string{"1", "2", "3"}
	vals := []string{"v1", "v2", "v3"}
	it := newSimpleIterator(keys, vals, true)
	mergeIt := NewMergeIterator([]y.Iterator{it}, true)
	mergeIt.Rewind()
	k, v := getAll(mergeIt)
	require.EqualValues(t, reversed(keys), k)
	require.EqualValues(t, reversed(vals), v)
	closeAndCheck(t, mergeIt, 1)
}

func TestMergeMore(t *testing.T) {
	it := newSimpleIterator([]string{"1", "3", "7"}, []string{"a1", "a3", "a7"}, false)
	it2 := newSimpleIterator([]string{"2", "3", "5"}, []string{"b2", "b3", "b5"}, false)
	it3 := newSimpleIterator([]string{"1"}, []string{"c1"}, false)
	it4 := newSimpleIterator([]string{"1", "7", "9"}, []string{"d1", "d7", "d9"}, false)
	t.Run("forward", func(t *testing.T) {
		expectedKeys := []string{"1", "2", "3", "5", "7", "9"}
		expectedVals := []string{"a1", "b2", "a3", "b5", "a7", "d9"}
		t.Run("no duplicates", func(t *testing.T) {
			mergeIt := NewMergeIterator([]y.Iterator{it, it2, it3, it4}, false)
			mergeIt.Rewind()
			k, v := getAll(mergeIt)
			require.EqualValues(t, expectedKeys, k)
			require.EqualValues(t, expectedVals, v)
			closeAndCheck(t, mergeIt, 4)
		})
		t.Run("duplicates", func(t *testing.T) {
			it5 := newSimpleIterator(
				[]string{"1", "1", "3", "7"},
				[]string{"a1", "a1-1", "a3", "a7"},
				false)
			mergeIt := NewMergeIterator([]y.Iterator{it5, it2, it3, it4}, false)
			expectedKeys := []string{"1", "2", "3", "5", "7", "9"}
			expectedVals := []string{"a1", "b2", "a3", "b5", "a7", "d9"}
			mergeIt.Rewind()
			k, v := getAll(mergeIt)
			require.EqualValues(t, expectedKeys, k)
			require.EqualValues(t, expectedVals, v)
			closeAndCheck(t, mergeIt, 4)
		})
	})
	t.Run("reverse", func(t *testing.T) {
		it.reversed = true
		it2.reversed = true
		it3.reversed = true
		it4.reversed = true
		t.Run("no duplicates", func(t *testing.T) {
			mergeIt := NewMergeIterator([]y.Iterator{it, it2, it3, it4}, true)
			expectedKeys := []string{"9", "7", "5", "3", "2", "1"}
			expectedVals := []string{"d9", "a7", "b5", "a3", "b2", "a1"}
			mergeIt.Rewind()
			k, v := getAll(mergeIt)
			require.EqualValues(t, expectedKeys, k)
			require.EqualValues(t, expectedVals, v)
			closeAndCheck(t, mergeIt, 4)
		})
		t.Run("duplicates", func(t *testing.T) {
			it5 := newSimpleIterator(
				[]string{"1", "1", "3", "7"},
				[]string{"a1", "a1-1", "a3", "a7"},
				true)
			mergeIt := NewMergeIterator([]y.Iterator{it5, it2, it3, it4}, true)
			expectedKeys := []string{"9", "7", "5", "3", "2", "1"}
			expectedVals := []string{"d9", "a7", "b5", "a3", "b2", "a1-1"}
			mergeIt.Rewind()
			k, v := getAll(mergeIt)
			require.EqualValues(t, expectedKeys, k)
			require.EqualValues(t, expectedVals, v)
			closeAndCheck(t, mergeIt, 4)
		})
	})
}

// Ensure MergeIterator satisfies the Iterator interface
func TestMergeIteratorNested(t *testing.T) {
	keys := []string{"1", "2", "3"}
	vals := []string{"v1", "v2", "v3"}
	it := newSimpleIterator(keys, vals, false)
	mergeIt := NewMergeIterator([]y.Iterator{it}, false)
	mergeIt2 := NewMergeIterator([]y.Iterator{mergeIt}, false)
	mergeIt2.Rewind()
	k, v := getAll(mergeIt2)
	require.EqualValues(t, keys, k)
	require.EqualValues(t, vals, v)
	closeAndCheck(t, mergeIt2, 1)
}

func TestMergeIteratorSeek(t *testing.T) {
	it := newSimpleIterator([]string{"1", "3", "7"}, []string{"a1", "a3", "a7"}, false)
	it2 := newSimpleIterator([]string{"2", "3", "5"}, []string{"b2", "b3", "b5"}, false)
	it3 := newSimpleIterator([]string{"1"}, []string{"c1"}, false)
	it4 := newSimpleIterator([]string{"1", "7", "9"}, []string{"d1", "d7", "d9"}, false)
	mergeIt := NewMergeIterator([]y.Iterator{it, it2, it3, it4}, false)
	mergeIt.Seek([]byte("4"))
	k, v := getAll(mergeIt)
	require.EqualValues(t, []string{"5", "7", "9"}, k)
	require.EqualValues(t, []string{"b5", "a7", "d9"}, v)
	closeAndCheck(t, mergeIt, 4)
}

func TestMergeIteratorSeekReversed(t *testing.T) {
	it := newSimpleIterator([]string{"1", "3", "7"}, []string{"a1", "a3", "a7"}, true)
	it2 := newSimpleIterator([]string{"2", "3", "5"}, []string{"b2", "b3", "b5"}, true)
	it3 := newSimpleIterator([]string{"1"}, []string{"c1"}, true)
	it4 := newSimpleIterator([]string{"1", "7", "9"}, []string{"d1", "d7", "d9"}, true)
	mergeIt := NewMergeIterator([]y.Iterator{it, it2, it3, it4}, true)
	mergeIt.Seek([]byte("5"))
	k, v := getAll(mergeIt)
	require.EqualValues(t, []string{"5", "3", "2", "1"}, k)
	require.EqualValues(t, []string{"b5", "a3", "b2", "a1"}, v)
	closeAndCheck(t, mergeIt, 4)
}

func TestMergeIteratorSeekInvalid(t *testing.T) {
	it := newSimpleIterator([]string{"1", "3", "7"}, []string{"a1", "a3", "a7"}, false)
	it2 := newSimpleIterator([]string{"2", "3", "5"}, []string{"b2", "b3", "b5"}, false)
	it3 := newSimpleIterator([]string{"1"}, []string{"c1"}, false)
	it4 := newSimpleIterator([]string{"1", "7", "9"}, []string{"d1", "d7", "d9"}, false)
	mergeIt := NewMergeIterator([]y.Iterator{it, it2, it3, it4}, false)
	mergeIt.Seek([]byte("f"))
	require.False(t, mergeIt.Valid())
	closeAndCheck(t, mergeIt, 4)
}

func TestMergeIteratorSeekInvalidReversed(t *testing.T) {
	it := newSimpleIterator([]string{"1", "3", "7"}, []string{"a1", "a3", "a7"}, true)
	it2 := newSimpleIterator([]string{"2", "3", "5"}, []string{"b2", "b3", "b5"}, true)
	it3 := newSimpleIterator([]string{"1"}, []string{"c1"}, true)
	it4 := newSimpleIterator([]string{"1", "7", "9"}, []string{"d1", "d7", "d9"}, true)
	mergeIt := NewMergeIterator([]y.Iterator{it, it2, it3, it4}, true)
	mergeIt.Seek([]byte("0"))
	require.False(t, mergeIt.Valid())
	closeAndCheck(t, mergeIt, 4)
}

func TestMergeIteratorDuplicate(t *testing.T) {
	it1 := newSimpleIterator([]string{"0", "1", "2"}, []string{"a0", "a1", "a2"}, false)
	it2 := newSimpleIterator([]string{"1", "3"}, []string{"b1", "b3"}, false)
	it3 := newSimpleIterator([]string{"0", "1", "2"}, []string{"c0", "c1", "c2"}, false)
	t.Run("forward", func(t *testing.T) {
		t.Run("only duplicates", func(t *testing.T) {
			it := NewMergeIterator([]y.Iterator{it1, it3}, false)
			expectedKeys := []string{"0", "1", "2"}
			expectedVals := []string{"a0", "a1", "a2"}
			it.Rewind()
			k, v := getAll(it)
			require.Equal(t, expectedKeys, k)
			require.Equal(t, expectedVals, v)
		})
		t.Run("one", func(t *testing.T) {
			it := NewMergeIterator([]y.Iterator{it3, it2, it1}, false)
			expectedKeys := []string{"0", "1", "2", "3"}
			expectedVals := []string{"c0", "c1", "c2", "b3"}
			it.Rewind()
			k, v := getAll(it)
			require.Equal(t, expectedKeys, k)
			require.Equal(t, expectedVals, v)
		})
		t.Run("two", func(t *testing.T) {
			it1 := newSimpleIterator([]string{"0", "1", "2"}, []string{"0", "1", "2"}, false)
			it2 := newSimpleIterator([]string{"1"}, []string{"1"}, false)
			it3 := newSimpleIterator([]string{"2"}, []string{"2"}, false)
			it := NewMergeIterator([]y.Iterator{it3, it2, it1}, false)

			var cnt int
			for it.Rewind(); it.Valid(); it.Next() {
				require.EqualValues(t, cnt+48, it.Key()[0])
				cnt++
			}
			require.Equal(t, 3, cnt)
		})
	})

	t.Run("reverse", func(t *testing.T) {
		it1.reversed = true
		it2.reversed = true
		it3.reversed = true

		it := NewMergeIterator([]y.Iterator{it3, it2, it1}, true)

		expectedKeys := []string{"3", "2", "1", "0"}
		expectedVals := []string{"b3", "c2", "c1", "c0"}
		it.Rewind()
		k, v := getAll(it)
		require.Equal(t, expectedKeys, k)
		require.Equal(t, expectedVals, v)
	})
}

func TestMergeDuplicates(t *testing.T) {
	it := newSimpleIterator([]string{"1", "1", "1"}, []string{"a1", "a3", "a7"}, false)
	it2 := newSimpleIterator([]string{"1", "1", "1"}, []string{"b2", "b3", "b5"}, false)
	it3 := newSimpleIterator([]string{"1"}, []string{"c1"}, false)
	it4 := newSimpleIterator([]string{"1", "1", "2"}, []string{"d1", "d7", "d9"}, false)
	t.Run("forward", func(t *testing.T) {
		expectedKeys := []string{"1", "2"}
		expectedVals := []string{"a1", "d9"}
		mergeIt := NewMergeIterator([]y.Iterator{it, it2, it3, it4}, false)
		mergeIt.Rewind()
		k, v := getAll(mergeIt)
		require.EqualValues(t, expectedKeys, k)
		require.EqualValues(t, expectedVals, v)
		closeAndCheck(t, mergeIt, 4)
	})

	t.Run("reverse", func(t *testing.T) {
		it.reversed = true
		it2.reversed = true
		it3.reversed = true
		it4.reversed = true
		expectedKeys := []string{"2", "1"}
		expectedVals := []string{"d9", "a7"}
		mergeIt := NewMergeIterator([]y.Iterator{it, it2, it3, it4}, true)
		mergeIt.Rewind()
		k, v := getAll(mergeIt)
		require.EqualValues(t, expectedKeys, k)
		require.EqualValues(t, expectedVals, v)
		closeAndCheck(t, mergeIt, 4)
	})
}
