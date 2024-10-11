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

package trie

import (
	"sort"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/pb"
)

func TestGet(t *testing.T) {
	trie := NewTrie()
	trie.Add([]byte("hello"), 1)
	trie.Add([]byte("hello"), 3)
	trie.Add([]byte("hello"), 4)
	trie.Add([]byte("hel"), 20)
	trie.Add([]byte("he"), 20)
	trie.Add([]byte("badger"), 30)

	trie.Add(nil, 10)
	require.Equal(t, map[uint64]struct{}{10: {}}, trie.Get([]byte("A")))

	ids := trie.Get([]byte("hel"))
	require.Equal(t, 2, len(ids))
	require.Equal(t, map[uint64]struct{}{10: {}, 20: {}}, ids)

	ids = trie.Get([]byte("badger"))
	require.Equal(t, 2, len(ids))
	require.Equal(t, map[uint64]struct{}{10: {}, 30: {}}, ids)

	ids = trie.Get([]byte("hello"))
	require.Equal(t, 5, len(ids))
	require.Equal(t, map[uint64]struct{}{10: {}, 1: {}, 3: {}, 4: {}, 20: {}}, ids)

	trie.Add([]byte{}, 11)
	require.Equal(t, map[uint64]struct{}{10: {}, 11: {}}, trie.Get([]byte("A")))
}

func TestTrieDelete(t *testing.T) {
	trie := NewTrie()
	t.Logf("Num nodes: %d", numNodes(trie.root))
	require.Equal(t, 1, numNodes(trie.root))

	trie.Add([]byte("hello"), 1)
	trie.Add([]byte("hello"), 3)
	trie.Add([]byte("hello"), 4)
	trie.Add(nil, 5)

	t.Logf("Num nodes: %d", numNodes(trie.root))

	require.NoError(t, trie.Delete([]byte("hello"), 4))
	t.Logf("Num nodes: %d", numNodes(trie.root))

	require.Equal(t, map[uint64]struct{}{5: {}, 1: {}, 3: {}}, trie.Get([]byte("hello")))

	require.NoError(t, trie.Delete(nil, 5))
	t.Logf("Num nodes: %d", numNodes(trie.root))
	require.Equal(t, map[uint64]struct{}{1: {}, 3: {}}, trie.Get([]byte("hello")))

	require.NoError(t, trie.Delete([]byte("hello"), 1))
	require.NoError(t, trie.Delete([]byte("hello"), 3))
	require.NoError(t, trie.Delete([]byte("hello"), 4))
	require.NoError(t, trie.Delete([]byte("hello"), 5))
	require.NoError(t, trie.Delete([]byte("hello"), 6))

	require.Equal(t, 1, numNodes(trie.root))
	t.Logf("Num nodes: %d", numNodes(trie.root))

	require.Equal(t, true, trie.root.isEmpty())
	require.Equal(t, map[uint64]struct{}{}, trie.Get([]byte("hello")))
}

func TestParseIgnoreBytes(t *testing.T) {
	out, err := parseIgnoreBytes("1")
	require.NoError(t, err)
	require.Equal(t, []bool{false, true}, out)

	out, err = parseIgnoreBytes("0")
	require.NoError(t, err)
	require.Equal(t, []bool{true}, out)

	out, err = parseIgnoreBytes("0, 3 - 5, 7")
	require.NoError(t, err)
	require.Equal(t, []bool{true, false, false, true, true, true, false, true}, out)
}

func TestPrefixMatchWithHoles(t *testing.T) {
	trie := NewTrie()

	add := func(prefix, ignore string, id uint64) {
		m := pb.Match{
			Prefix:      []byte(prefix),
			IgnoreBytes: ignore,
		}
		require.NoError(t, trie.AddMatch(m, id))
	}

	add("", "", 1)
	add("aaaa", "", 2)
	add("aaaaaa", "2-10", 3)
	add("aaaaaaaaa", "0, 4 - 6, 8", 4)

	get := func(k string) []uint64 {
		var ids []uint64
		m := trie.Get([]byte(k))
		for id := range m {
			ids = append(ids, id)
		}
		sort.Slice(ids, func(i, j int) bool {
			return ids[i] < ids[j]
		})
		return ids
	}

	// Everything matches 1
	require.Equal(t, []uint64{1}, get(""))
	require.Equal(t, []uint64{1}, get("aax"))

	// aaaaa would match 2, but not 3 because 3's length is 6.
	require.Equal(t, []uint64{1, 2}, get("aaaaa"))

	// aa and enough length is sufficient to match 3.
	require.Equal(t, []uint64{1, 3}, get("aabbbbbbbb"))

	// has differences in the right place to match 4.
	require.Equal(t, []uint64{1, 4}, get("baaabbbabba"))

	// Even with differences matches everything.
	require.Equal(t, []uint64{1, 2, 3, 4}, get("aaaabbbabba"))

	t.Logf("Num nodes: %d", numNodes(trie.root))

	del := func(prefix, ignore string, id uint64) {
		m := pb.Match{
			Prefix:      []byte(prefix),
			IgnoreBytes: ignore,
		}
		require.NoError(t, trie.DeleteMatch(m, id))
	}

	del("aaaaaaaaa", "0, 4 - 6, 8", 5)
	t.Logf("Num nodes: %d", numNodes(trie.root))

	del("aaaaaaaaa", "0, 4 - 6, 8", 4)
	t.Logf("Num nodes: %d", numNodes(trie.root))

	del("aaaaaa", "2-10", 3)
	t.Logf("Num nodes: %d", numNodes(trie.root))

	del("aaaa", "", 2)
	t.Logf("Num nodes: %d", numNodes(trie.root))

	del("", "", 1)
	t.Logf("Num nodes: %d", numNodes(trie.root))

	del("abracadabra", "", 4)
	t.Logf("Num nodes: %d", numNodes(trie.root))

	require.Equal(t, 1, numNodes(trie.root))
}
