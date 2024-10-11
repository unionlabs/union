// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package trie

import (
	"bytes"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

var (
	tiny = []struct{ k, v string }{
		{"k1", "v1"},
		{"k2", "v2"},
		{"k3", "v3"},
	}
	nonAligned = []struct{ k, v string }{
		{"do", "verb"},
		{"ether", "wookiedoo"},
		{"horse", "stallion"},
		{"shaman", "horse"},
		{"doge", "coin"},
		{"dog", "puppy"},
		{"somethingveryoddindeedthis is", "myothernodedata"},
	}
	standard = []struct{ k, v string }{
		{string(randBytes(32)), "verb"},
		{string(randBytes(32)), "wookiedoo"},
		{string(randBytes(32)), "stallion"},
		{string(randBytes(32)), "horse"},
		{string(randBytes(32)), "coin"},
		{string(randBytes(32)), "puppy"},
		{string(randBytes(32)), "myothernodedata"},
	}
)

func TestTrieTracer(t *testing.T) {
	testTrieTracer(t, tiny)
	testTrieTracer(t, nonAligned)
	testTrieTracer(t, standard)
}

// Tests if the trie diffs are tracked correctly. Tracer should capture
// all non-leaf dirty nodes, no matter the node is embedded or not.
func testTrieTracer(t *testing.T, vals []struct{ k, v string }) {
	db := NewDatabase(rawdb.NewMemoryDatabase(), nil)
	trie := NewEmpty(db)

	// Determine all new nodes are tracked
	for _, val := range vals {
		trie.MustUpdate([]byte(val.k), []byte(val.v))
	}
	insertSet := copySet(trie.tracer.inserts) // copy before commit
	deleteSet := copySet(trie.tracer.deletes) // copy before commit
	root, nodes, _ := trie.Commit(false)
	db.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)

	seen := setKeys(iterNodes(db, root))
	if !compareSet(insertSet, seen) {
		t.Fatal("Unexpected insertion set")
	}
	if !compareSet(deleteSet, nil) {
		t.Fatal("Unexpected deletion set")
	}

	// Determine all deletions are tracked
	trie, _ = New(TrieID(root), db)
	for _, val := range vals {
		trie.MustDelete([]byte(val.k))
	}
	insertSet, deleteSet = copySet(trie.tracer.inserts), copySet(trie.tracer.deletes)
	if !compareSet(insertSet, nil) {
		t.Fatal("Unexpected insertion set")
	}
	if !compareSet(deleteSet, seen) {
		t.Fatal("Unexpected deletion set")
	}
}

// Test that after inserting a new batch of nodes and deleting them immediately,
// the trie tracer should be cleared normally as no operation happened.
func TestTrieTracerNoop(t *testing.T) {
	testTrieTracerNoop(t, tiny)
	testTrieTracerNoop(t, nonAligned)
	testTrieTracerNoop(t, standard)
}

func testTrieTracerNoop(t *testing.T, vals []struct{ k, v string }) {
	trie := NewEmpty(NewDatabase(rawdb.NewMemoryDatabase(), nil))
	for _, val := range vals {
		trie.MustUpdate([]byte(val.k), []byte(val.v))
	}
	for _, val := range vals {
		trie.MustDelete([]byte(val.k))
	}
	if len(trie.tracer.inserts) != 0 {
		t.Fatal("Unexpected insertion set")
	}
	if len(trie.tracer.deletes) != 0 {
		t.Fatal("Unexpected deletion set")
	}
}

// Tests if the accessList is correctly tracked.
func TestAccessList(t *testing.T) {
	testAccessList(t, tiny)
	testAccessList(t, nonAligned)
	testAccessList(t, standard)
}

func testAccessList(t *testing.T, vals []struct{ k, v string }) {
	var (
		db   = NewDatabase(rawdb.NewMemoryDatabase(), nil)
		trie = NewEmpty(db)
		orig = trie.Copy()
	)
	// Create trie from scratch
	for _, val := range vals {
		trie.MustUpdate([]byte(val.k), []byte(val.v))
	}
	root, nodes, _ := trie.Commit(false)
	db.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)

	trie, _ = New(TrieID(root), db)
	if err := verifyAccessList(orig, trie, nodes); err != nil {
		t.Fatalf("Invalid accessList %v", err)
	}

	// Update trie
	parent := root
	trie, _ = New(TrieID(root), db)
	orig = trie.Copy()
	for _, val := range vals {
		trie.MustUpdate([]byte(val.k), randBytes(32))
	}
	root, nodes, _ = trie.Commit(false)
	db.Update(root, parent, 0, trienode.NewWithNodeSet(nodes), nil)

	trie, _ = New(TrieID(root), db)
	if err := verifyAccessList(orig, trie, nodes); err != nil {
		t.Fatalf("Invalid accessList %v", err)
	}

	// Add more new nodes
	parent = root
	trie, _ = New(TrieID(root), db)
	orig = trie.Copy()
	var keys []string
	for i := 0; i < 30; i++ {
		key := randBytes(32)
		keys = append(keys, string(key))
		trie.MustUpdate(key, randBytes(32))
	}
	root, nodes, _ = trie.Commit(false)
	db.Update(root, parent, 0, trienode.NewWithNodeSet(nodes), nil)

	trie, _ = New(TrieID(root), db)
	if err := verifyAccessList(orig, trie, nodes); err != nil {
		t.Fatalf("Invalid accessList %v", err)
	}

	// Partial deletions
	parent = root
	trie, _ = New(TrieID(root), db)
	orig = trie.Copy()
	for _, key := range keys {
		trie.MustUpdate([]byte(key), nil)
	}
	root, nodes, _ = trie.Commit(false)
	db.Update(root, parent, 0, trienode.NewWithNodeSet(nodes), nil)

	trie, _ = New(TrieID(root), db)
	if err := verifyAccessList(orig, trie, nodes); err != nil {
		t.Fatalf("Invalid accessList %v", err)
	}

	// Delete all
	parent = root
	trie, _ = New(TrieID(root), db)
	orig = trie.Copy()
	for _, val := range vals {
		trie.MustUpdate([]byte(val.k), nil)
	}
	root, nodes, _ = trie.Commit(false)
	db.Update(root, parent, 0, trienode.NewWithNodeSet(nodes), nil)

	trie, _ = New(TrieID(root), db)
	if err := verifyAccessList(orig, trie, nodes); err != nil {
		t.Fatalf("Invalid accessList %v", err)
	}
}

// Tests origin values won't be tracked in Iterator or Prover
func TestAccessListLeak(t *testing.T) {
	var (
		db   = NewDatabase(rawdb.NewMemoryDatabase(), nil)
		trie = NewEmpty(db)
	)
	// Create trie from scratch
	for _, val := range standard {
		trie.MustUpdate([]byte(val.k), []byte(val.v))
	}
	root, nodes, _ := trie.Commit(false)
	db.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)

	var cases = []struct {
		op func(tr *Trie)
	}{
		{
			func(tr *Trie) {
				it := tr.MustNodeIterator(nil)
				for it.Next(true) {
				}
			},
		},
		{
			func(tr *Trie) {
				it := NewIterator(tr.MustNodeIterator(nil))
				for it.Next() {
				}
			},
		},
		{
			func(tr *Trie) {
				for _, val := range standard {
					tr.Prove([]byte(val.k), rawdb.NewMemoryDatabase())
				}
			},
		},
	}
	for _, c := range cases {
		trie, _ = New(TrieID(root), db)
		n1 := len(trie.tracer.accessList)
		c.op(trie)
		n2 := len(trie.tracer.accessList)

		if n1 != n2 {
			t.Fatalf("AccessList is leaked, prev %d after %d", n1, n2)
		}
	}
}

// Tests whether the original tree node is correctly deleted after being embedded
// in its parent due to the smaller size of the original tree node.
func TestTinyTree(t *testing.T) {
	var (
		db   = NewDatabase(rawdb.NewMemoryDatabase(), nil)
		trie = NewEmpty(db)
	)
	for _, val := range tiny {
		trie.MustUpdate([]byte(val.k), randBytes(32))
	}
	root, set, _ := trie.Commit(false)
	db.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(set), nil)

	parent := root
	trie, _ = New(TrieID(root), db)
	orig := trie.Copy()
	for _, val := range tiny {
		trie.MustUpdate([]byte(val.k), []byte(val.v))
	}
	root, set, _ = trie.Commit(false)
	db.Update(root, parent, 0, trienode.NewWithNodeSet(set), nil)

	trie, _ = New(TrieID(root), db)
	if err := verifyAccessList(orig, trie, set); err != nil {
		t.Fatalf("Invalid accessList %v", err)
	}
}

func compareSet(setA, setB map[string]struct{}) bool {
	if len(setA) != len(setB) {
		return false
	}
	for key := range setA {
		if _, ok := setB[key]; !ok {
			return false
		}
	}
	return true
}

func forNodes(tr *Trie) map[string][]byte {
	var (
		it    = tr.MustNodeIterator(nil)
		nodes = make(map[string][]byte)
	)
	for it.Next(true) {
		if it.Leaf() {
			continue
		}
		nodes[string(it.Path())] = common.CopyBytes(it.NodeBlob())
	}
	return nodes
}

func iterNodes(db *Database, root common.Hash) map[string][]byte {
	tr, _ := New(TrieID(root), db)
	return forNodes(tr)
}

func forHashedNodes(tr *Trie) map[string][]byte {
	var (
		it    = tr.MustNodeIterator(nil)
		nodes = make(map[string][]byte)
	)
	for it.Next(true) {
		if it.Hash() == (common.Hash{}) {
			continue
		}
		nodes[string(it.Path())] = common.CopyBytes(it.NodeBlob())
	}
	return nodes
}

func diffTries(trieA, trieB *Trie) (map[string][]byte, map[string][]byte, map[string][]byte) {
	var (
		nodesA = forHashedNodes(trieA)
		nodesB = forHashedNodes(trieB)
		inA    = make(map[string][]byte) // hashed nodes in trie a but not b
		inB    = make(map[string][]byte) // hashed nodes in trie b but not a
		both   = make(map[string][]byte) // hashed nodes in both tries but different value
	)
	for path, blobA := range nodesA {
		if blobB, ok := nodesB[path]; ok {
			if bytes.Equal(blobA, blobB) {
				continue
			}
			both[path] = blobA
			continue
		}
		inA[path] = blobA
	}
	for path, blobB := range nodesB {
		if _, ok := nodesA[path]; ok {
			continue
		}
		inB[path] = blobB
	}
	return inA, inB, both
}

func setKeys(set map[string][]byte) map[string]struct{} {
	keys := make(map[string]struct{})
	for k := range set {
		keys[k] = struct{}{}
	}
	return keys
}

func copySet(set map[string]struct{}) map[string]struct{} {
	copied := make(map[string]struct{})
	for k := range set {
		copied[k] = struct{}{}
	}
	return copied
}
