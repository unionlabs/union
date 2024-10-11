// Copyright 2014 The go-ethereum Authors
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
	"fmt"
	"math/rand"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

func TestEmptyIterator(t *testing.T) {
	trie := NewEmpty(NewDatabase(rawdb.NewMemoryDatabase(), nil))
	iter := trie.MustNodeIterator(nil)

	seen := make(map[string]struct{})
	for iter.Next(true) {
		seen[string(iter.Path())] = struct{}{}
	}
	if len(seen) != 0 {
		t.Fatal("Unexpected trie node iterated")
	}
}

func TestIterator(t *testing.T) {
	db := NewDatabase(rawdb.NewMemoryDatabase(), nil)
	trie := NewEmpty(db)
	vals := []struct{ k, v string }{
		{"do", "verb"},
		{"ether", "wookiedoo"},
		{"horse", "stallion"},
		{"shaman", "horse"},
		{"doge", "coin"},
		{"dog", "puppy"},
		{"somethingveryoddindeedthis is", "myothernodedata"},
	}
	all := make(map[string]string)
	for _, val := range vals {
		all[val.k] = val.v
		trie.MustUpdate([]byte(val.k), []byte(val.v))
	}
	root, nodes, _ := trie.Commit(false)
	db.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)

	trie, _ = New(TrieID(root), db)
	found := make(map[string]string)
	it := NewIterator(trie.MustNodeIterator(nil))
	for it.Next() {
		found[string(it.Key)] = string(it.Value)
	}

	for k, v := range all {
		if found[k] != v {
			t.Errorf("iterator value mismatch for %s: got %q want %q", k, found[k], v)
		}
	}
}

type kv struct {
	k, v []byte
	t    bool
}

func (k *kv) cmp(other *kv) int {
	return bytes.Compare(k.k, other.k)
}

func TestIteratorLargeData(t *testing.T) {
	trie := NewEmpty(NewDatabase(rawdb.NewMemoryDatabase(), nil))
	vals := make(map[string]*kv)

	for i := byte(0); i < 255; i++ {
		value := &kv{common.LeftPadBytes([]byte{i}, 32), []byte{i}, false}
		value2 := &kv{common.LeftPadBytes([]byte{10, i}, 32), []byte{i}, false}
		trie.MustUpdate(value.k, value.v)
		trie.MustUpdate(value2.k, value2.v)
		vals[string(value.k)] = value
		vals[string(value2.k)] = value2
	}

	it := NewIterator(trie.MustNodeIterator(nil))
	for it.Next() {
		vals[string(it.Key)].t = true
	}

	var untouched []*kv
	for _, value := range vals {
		if !value.t {
			untouched = append(untouched, value)
		}
	}

	if len(untouched) > 0 {
		t.Errorf("Missed %d nodes", len(untouched))
		for _, value := range untouched {
			t.Error(value)
		}
	}
}

type iterationElement struct {
	hash common.Hash
	path []byte
	blob []byte
}

// Tests that the node iterator indeed walks over the entire database contents.
func TestNodeIteratorCoverage(t *testing.T) {
	testNodeIteratorCoverage(t, rawdb.HashScheme)
	testNodeIteratorCoverage(t, rawdb.PathScheme)
}

func testNodeIteratorCoverage(t *testing.T, scheme string) {
	// Create some arbitrary test trie to iterate
	db, nodeDb, trie, _ := makeTestTrie(scheme)

	// Gather all the node hashes found by the iterator
	var elements = make(map[common.Hash]iterationElement)
	for it := trie.MustNodeIterator(nil); it.Next(true); {
		if it.Hash() != (common.Hash{}) {
			elements[it.Hash()] = iterationElement{
				hash: it.Hash(),
				path: common.CopyBytes(it.Path()),
				blob: common.CopyBytes(it.NodeBlob()),
			}
		}
	}
	// Cross check the hashes and the database itself
	reader, err := nodeDb.Reader(trie.Hash())
	if err != nil {
		t.Fatalf("state is not available %x", trie.Hash())
	}
	for _, element := range elements {
		if blob, err := reader.Node(common.Hash{}, element.path, element.hash); err != nil {
			t.Errorf("failed to retrieve reported node %x: %v", element.hash, err)
		} else if !bytes.Equal(blob, element.blob) {
			t.Errorf("node blob is different, want %v got %v", element.blob, blob)
		}
	}
	var (
		count int
		it    = db.NewIterator(nil, nil)
	)
	for it.Next() {
		res, _, _ := isTrieNode(nodeDb.Scheme(), it.Key(), it.Value())
		if !res {
			continue
		}
		count += 1
		if elem, ok := elements[crypto.Keccak256Hash(it.Value())]; !ok {
			t.Error("state entry not reported")
		} else if !bytes.Equal(it.Value(), elem.blob) {
			t.Errorf("node blob is different, want %v got %v", elem.blob, it.Value())
		}
	}
	it.Release()
	if count != len(elements) {
		t.Errorf("state entry is mismatched %d %d", count, len(elements))
	}
}

type kvs struct{ k, v string }

var testdata1 = []kvs{
	{"barb", "ba"},
	{"bard", "bc"},
	{"bars", "bb"},
	{"bar", "b"},
	{"fab", "z"},
	{"food", "ab"},
	{"foos", "aa"},
	{"foo", "a"},
}

var testdata2 = []kvs{
	{"aardvark", "c"},
	{"bar", "b"},
	{"barb", "bd"},
	{"bars", "be"},
	{"fab", "z"},
	{"foo", "a"},
	{"foos", "aa"},
	{"food", "ab"},
	{"jars", "d"},
}

func TestIteratorSeek(t *testing.T) {
	trie := NewEmpty(NewDatabase(rawdb.NewMemoryDatabase(), nil))
	for _, val := range testdata1 {
		trie.MustUpdate([]byte(val.k), []byte(val.v))
	}

	// Seek to the middle.
	it := NewIterator(trie.MustNodeIterator([]byte("fab")))
	if err := checkIteratorOrder(testdata1[4:], it); err != nil {
		t.Fatal(err)
	}

	// Seek to a non-existent key.
	it = NewIterator(trie.MustNodeIterator([]byte("barc")))
	if err := checkIteratorOrder(testdata1[1:], it); err != nil {
		t.Fatal(err)
	}

	// Seek beyond the end.
	it = NewIterator(trie.MustNodeIterator([]byte("z")))
	if err := checkIteratorOrder(nil, it); err != nil {
		t.Fatal(err)
	}
}

func checkIteratorOrder(want []kvs, it *Iterator) error {
	for it.Next() {
		if len(want) == 0 {
			return fmt.Errorf("didn't expect any more values, got key %q", it.Key)
		}
		if !bytes.Equal(it.Key, []byte(want[0].k)) {
			return fmt.Errorf("wrong key: got %q, want %q", it.Key, want[0].k)
		}
		want = want[1:]
	}
	if len(want) > 0 {
		return fmt.Errorf("iterator ended early, want key %q", want[0])
	}
	return nil
}

func TestDifferenceIterator(t *testing.T) {
	dba := NewDatabase(rawdb.NewMemoryDatabase(), nil)
	triea := NewEmpty(dba)
	for _, val := range testdata1 {
		triea.MustUpdate([]byte(val.k), []byte(val.v))
	}
	rootA, nodesA, _ := triea.Commit(false)
	dba.Update(rootA, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodesA), nil)
	triea, _ = New(TrieID(rootA), dba)

	dbb := NewDatabase(rawdb.NewMemoryDatabase(), nil)
	trieb := NewEmpty(dbb)
	for _, val := range testdata2 {
		trieb.MustUpdate([]byte(val.k), []byte(val.v))
	}
	rootB, nodesB, _ := trieb.Commit(false)
	dbb.Update(rootB, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodesB), nil)
	trieb, _ = New(TrieID(rootB), dbb)

	found := make(map[string]string)
	di, _ := NewDifferenceIterator(triea.MustNodeIterator(nil), trieb.MustNodeIterator(nil))
	it := NewIterator(di)
	for it.Next() {
		found[string(it.Key)] = string(it.Value)
	}

	all := []struct{ k, v string }{
		{"aardvark", "c"},
		{"barb", "bd"},
		{"bars", "be"},
		{"jars", "d"},
	}
	for _, item := range all {
		if found[item.k] != item.v {
			t.Errorf("iterator value mismatch for %s: got %v want %v", item.k, found[item.k], item.v)
		}
	}
	if len(found) != len(all) {
		t.Errorf("iterator count mismatch: got %d values, want %d", len(found), len(all))
	}
}

func TestUnionIterator(t *testing.T) {
	dba := NewDatabase(rawdb.NewMemoryDatabase(), nil)
	triea := NewEmpty(dba)
	for _, val := range testdata1 {
		triea.MustUpdate([]byte(val.k), []byte(val.v))
	}
	rootA, nodesA, _ := triea.Commit(false)
	dba.Update(rootA, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodesA), nil)
	triea, _ = New(TrieID(rootA), dba)

	dbb := NewDatabase(rawdb.NewMemoryDatabase(), nil)
	trieb := NewEmpty(dbb)
	for _, val := range testdata2 {
		trieb.MustUpdate([]byte(val.k), []byte(val.v))
	}
	rootB, nodesB, _ := trieb.Commit(false)
	dbb.Update(rootB, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodesB), nil)
	trieb, _ = New(TrieID(rootB), dbb)

	di, _ := NewUnionIterator([]NodeIterator{triea.MustNodeIterator(nil), trieb.MustNodeIterator(nil)})
	it := NewIterator(di)

	all := []struct{ k, v string }{
		{"aardvark", "c"},
		{"barb", "ba"},
		{"barb", "bd"},
		{"bard", "bc"},
		{"bars", "bb"},
		{"bars", "be"},
		{"bar", "b"},
		{"fab", "z"},
		{"food", "ab"},
		{"foos", "aa"},
		{"foo", "a"},
		{"jars", "d"},
	}

	for i, kv := range all {
		if !it.Next() {
			t.Errorf("Iterator ends prematurely at element %d", i)
		}
		if kv.k != string(it.Key) {
			t.Errorf("iterator value mismatch for element %d: got key %s want %s", i, it.Key, kv.k)
		}
		if kv.v != string(it.Value) {
			t.Errorf("iterator value mismatch for element %d: got value %s want %s", i, it.Value, kv.v)
		}
	}
	if it.Next() {
		t.Errorf("Iterator returned extra values.")
	}
}

func TestIteratorNoDups(t *testing.T) {
	tr := NewEmpty(NewDatabase(rawdb.NewMemoryDatabase(), nil))
	for _, val := range testdata1 {
		tr.MustUpdate([]byte(val.k), []byte(val.v))
	}
	checkIteratorNoDups(t, tr.MustNodeIterator(nil), nil)
}

// This test checks that nodeIterator.Next can be retried after inserting missing trie nodes.
func TestIteratorContinueAfterError(t *testing.T) {
	testIteratorContinueAfterError(t, false, rawdb.HashScheme)
	testIteratorContinueAfterError(t, true, rawdb.HashScheme)
	testIteratorContinueAfterError(t, false, rawdb.PathScheme)
	testIteratorContinueAfterError(t, true, rawdb.PathScheme)
}

func testIteratorContinueAfterError(t *testing.T, memonly bool, scheme string) {
	diskdb := rawdb.NewMemoryDatabase()
	tdb := newTestDatabase(diskdb, scheme)

	tr := NewEmpty(tdb)
	for _, val := range testdata1 {
		tr.MustUpdate([]byte(val.k), []byte(val.v))
	}
	root, nodes, _ := tr.Commit(false)
	tdb.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)
	if !memonly {
		tdb.Commit(root, false)
	}
	tr, _ = New(TrieID(root), tdb)
	wantNodeCount := checkIteratorNoDups(t, tr.MustNodeIterator(nil), nil)

	var (
		paths  [][]byte
		hashes []common.Hash
	)
	if memonly {
		for path, n := range nodes.Nodes {
			paths = append(paths, []byte(path))
			hashes = append(hashes, n.Hash)
		}
	} else {
		it := diskdb.NewIterator(nil, nil)
		for it.Next() {
			ok, path, hash := isTrieNode(tdb.Scheme(), it.Key(), it.Value())
			if !ok {
				continue
			}
			paths = append(paths, path)
			hashes = append(hashes, hash)
		}
		it.Release()
	}
	for i := 0; i < 20; i++ {
		// Create trie that will load all nodes from DB.
		tr, _ := New(TrieID(tr.Hash()), tdb)

		// Remove a random node from the database. It can't be the root node
		// because that one is already loaded.
		var (
			rval  []byte
			rpath []byte
			rhash common.Hash
		)
		for {
			if memonly {
				rpath = paths[rand.Intn(len(paths))]
				n := nodes.Nodes[string(rpath)]
				if n == nil {
					continue
				}
				rhash = n.Hash
			} else {
				index := rand.Intn(len(paths))
				rpath = paths[index]
				rhash = hashes[index]
			}
			if rhash != tr.Hash() {
				break
			}
		}
		if memonly {
			tr.reader.banned = map[string]struct{}{string(rpath): {}}
		} else {
			rval = rawdb.ReadTrieNode(diskdb, common.Hash{}, rpath, rhash, tdb.Scheme())
			rawdb.DeleteTrieNode(diskdb, common.Hash{}, rpath, rhash, tdb.Scheme())
		}
		// Iterate until the error is hit.
		seen := make(map[string]bool)
		it := tr.MustNodeIterator(nil)
		checkIteratorNoDups(t, it, seen)
		missing, ok := it.Error().(*MissingNodeError)
		if !ok || missing.NodeHash != rhash {
			t.Fatal("didn't hit missing node, got", it.Error())
		}

		// Add the node back and continue iteration.
		if memonly {
			delete(tr.reader.banned, string(rpath))
		} else {
			rawdb.WriteTrieNode(diskdb, common.Hash{}, rpath, rhash, rval, tdb.Scheme())
		}
		checkIteratorNoDups(t, it, seen)
		if it.Error() != nil {
			t.Fatal("unexpected error", it.Error())
		}
		if len(seen) != wantNodeCount {
			t.Fatal("wrong node iteration count, got", len(seen), "want", wantNodeCount)
		}
	}
}

// Similar to the test above, this one checks that failure to create nodeIterator at a
// certain key prefix behaves correctly when Next is called. The expectation is that Next
// should retry seeking before returning true for the first time.
func TestIteratorContinueAfterSeekError(t *testing.T) {
	testIteratorContinueAfterSeekError(t, false, rawdb.HashScheme)
	testIteratorContinueAfterSeekError(t, true, rawdb.HashScheme)
	testIteratorContinueAfterSeekError(t, false, rawdb.PathScheme)
	testIteratorContinueAfterSeekError(t, true, rawdb.PathScheme)
}

func testIteratorContinueAfterSeekError(t *testing.T, memonly bool, scheme string) {
	// Commit test trie to db, then remove the node containing "bars".
	var (
		barNodePath []byte
		barNodeHash = common.HexToHash("05041990364eb72fcb1127652ce40d8bab765f2bfe53225b1170d276cc101c2e")
	)
	diskdb := rawdb.NewMemoryDatabase()
	triedb := newTestDatabase(diskdb, scheme)
	ctr := NewEmpty(triedb)
	for _, val := range testdata1 {
		ctr.MustUpdate([]byte(val.k), []byte(val.v))
	}
	root, nodes, _ := ctr.Commit(false)
	for path, n := range nodes.Nodes {
		if n.Hash == barNodeHash {
			barNodePath = []byte(path)
			break
		}
	}
	triedb.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)
	if !memonly {
		triedb.Commit(root, false)
	}
	var (
		barNodeBlob []byte
	)
	tr, _ := New(TrieID(root), triedb)
	if memonly {
		tr.reader.banned = map[string]struct{}{string(barNodePath): {}}
	} else {
		barNodeBlob = rawdb.ReadTrieNode(diskdb, common.Hash{}, barNodePath, barNodeHash, triedb.Scheme())
		rawdb.DeleteTrieNode(diskdb, common.Hash{}, barNodePath, barNodeHash, triedb.Scheme())
	}
	// Create a new iterator that seeks to "bars". Seeking can't proceed because
	// the node is missing.
	it := tr.MustNodeIterator([]byte("bars"))
	missing, ok := it.Error().(*MissingNodeError)
	if !ok {
		t.Fatal("want MissingNodeError, got", it.Error())
	} else if missing.NodeHash != barNodeHash {
		t.Fatal("wrong node missing")
	}
	// Reinsert the missing node.
	if memonly {
		delete(tr.reader.banned, string(barNodePath))
	} else {
		rawdb.WriteTrieNode(diskdb, common.Hash{}, barNodePath, barNodeHash, barNodeBlob, triedb.Scheme())
	}
	// Check that iteration produces the right set of values.
	if err := checkIteratorOrder(testdata1[2:], NewIterator(it)); err != nil {
		t.Fatal(err)
	}
}

func checkIteratorNoDups(t *testing.T, it NodeIterator, seen map[string]bool) int {
	if seen == nil {
		seen = make(map[string]bool)
	}
	for it.Next(true) {
		if seen[string(it.Path())] {
			t.Fatalf("iterator visited node path %x twice", it.Path())
		}
		seen[string(it.Path())] = true
	}
	return len(seen)
}

func TestIteratorNodeBlob(t *testing.T) {
	testIteratorNodeBlob(t, rawdb.HashScheme)
	testIteratorNodeBlob(t, rawdb.PathScheme)
}

func testIteratorNodeBlob(t *testing.T, scheme string) {
	var (
		db     = rawdb.NewMemoryDatabase()
		triedb = newTestDatabase(db, scheme)
		trie   = NewEmpty(triedb)
	)
	vals := []struct{ k, v string }{
		{"do", "verb"},
		{"ether", "wookiedoo"},
		{"horse", "stallion"},
		{"shaman", "horse"},
		{"doge", "coin"},
		{"dog", "puppy"},
		{"somethingveryoddindeedthis is", "myothernodedata"},
	}
	all := make(map[string]string)
	for _, val := range vals {
		all[val.k] = val.v
		trie.MustUpdate([]byte(val.k), []byte(val.v))
	}
	root, nodes, _ := trie.Commit(false)
	triedb.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)
	triedb.Commit(root, false)

	var found = make(map[common.Hash][]byte)
	trie, _ = New(TrieID(root), triedb)
	it := trie.MustNodeIterator(nil)
	for it.Next(true) {
		if it.Hash() == (common.Hash{}) {
			continue
		}
		found[it.Hash()] = it.NodeBlob()
	}

	dbIter := db.NewIterator(nil, nil)
	defer dbIter.Release()

	var count int
	for dbIter.Next() {
		ok, _, _ := isTrieNode(triedb.Scheme(), dbIter.Key(), dbIter.Value())
		if !ok {
			continue
		}
		got, present := found[crypto.Keccak256Hash(dbIter.Value())]
		if !present {
			t.Fatal("Miss trie node")
		}
		if !bytes.Equal(got, dbIter.Value()) {
			t.Fatalf("Unexpected trie node want %v got %v", dbIter.Value(), got)
		}
		count += 1
	}
	if count != len(found) {
		t.Fatal("Find extra trie node via iterator")
	}
}

// isTrieNode is a helper function which reports if the provided
// database entry belongs to a trie node or not. Note in tests
// only single layer trie is used, namely storage trie is not
// considered at all.
func isTrieNode(scheme string, key, val []byte) (bool, []byte, common.Hash) {
	var (
		path []byte
		hash common.Hash
	)
	if scheme == rawdb.HashScheme {
		ok := rawdb.IsLegacyTrieNode(key, val)
		if !ok {
			return false, nil, common.Hash{}
		}
		hash = common.BytesToHash(key)
	} else {
		ok, remain := rawdb.ResolveAccountTrieNodeKey(key)
		if !ok {
			return false, nil, common.Hash{}
		}
		path = common.CopyBytes(remain)
		hash = crypto.Keccak256Hash(val)
	}
	return true, path, hash
}
