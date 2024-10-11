// Copyright 2015 The go-ethereum Authors
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

package state

import (
	"bytes"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/triedb/hashdb"
	"github.com/ethereum/go-ethereum/trie/triedb/pathdb"
)

// testAccount is the data associated with an account used by the state tests.
type testAccount struct {
	address common.Address
	balance *big.Int
	nonce   uint64
	code    []byte
}

// makeTestState create a sample test state to test node-wise reconstruction.
func makeTestState(scheme string) (ethdb.Database, Database, *trie.Database, common.Hash, []*testAccount) {
	// Create an empty state
	config := &trie.Config{Preimages: true}
	if scheme == rawdb.PathScheme {
		config.PathDB = pathdb.Defaults
	} else {
		config.HashDB = hashdb.Defaults
	}
	db := rawdb.NewMemoryDatabase()
	nodeDb := trie.NewDatabase(db, config)
	sdb := NewDatabaseWithNodeDB(db, nodeDb)
	state, _ := New(types.EmptyRootHash, sdb, nil)

	// Fill it with some arbitrary data
	var accounts []*testAccount
	for i := byte(0); i < 96; i++ {
		obj := state.GetOrNewStateObject(common.BytesToAddress([]byte{i}))
		acc := &testAccount{address: common.BytesToAddress([]byte{i})}

		obj.AddBalance(big.NewInt(int64(11 * i)))
		acc.balance = big.NewInt(int64(11 * i))

		obj.SetNonce(uint64(42 * i))
		acc.nonce = uint64(42 * i)

		if i%3 == 0 {
			obj.SetCode(crypto.Keccak256Hash([]byte{i, i, i, i, i}), []byte{i, i, i, i, i})
			acc.code = []byte{i, i, i, i, i}
		}
		if i%5 == 0 {
			for j := byte(0); j < 5; j++ {
				hash := crypto.Keccak256Hash([]byte{i, i, i, i, i, j, j})
				obj.SetState(hash, hash)
			}
		}
		accounts = append(accounts, acc)
	}
	root, _ := state.Commit(0, false)

	// Return the generated state
	return db, sdb, nodeDb, root, accounts
}

// checkStateAccounts cross references a reconstructed state with an expected
// account array.
func checkStateAccounts(t *testing.T, db ethdb.Database, scheme string, root common.Hash, accounts []*testAccount) {
	var config trie.Config
	if scheme == rawdb.PathScheme {
		config.PathDB = pathdb.Defaults
	}
	// Check root availability and state contents
	state, err := New(root, NewDatabaseWithConfig(db, &config), nil)
	if err != nil {
		t.Fatalf("failed to create state trie at %x: %v", root, err)
	}
	if err := checkStateConsistency(db, scheme, root); err != nil {
		t.Fatalf("inconsistent state trie at %x: %v", root, err)
	}
	for i, acc := range accounts {
		if balance := state.GetBalance(acc.address); balance.Cmp(acc.balance) != 0 {
			t.Errorf("account %d: balance mismatch: have %v, want %v", i, balance, acc.balance)
		}
		if nonce := state.GetNonce(acc.address); nonce != acc.nonce {
			t.Errorf("account %d: nonce mismatch: have %v, want %v", i, nonce, acc.nonce)
		}
		if code := state.GetCode(acc.address); !bytes.Equal(code, acc.code) {
			t.Errorf("account %d: code mismatch: have %x, want %x", i, code, acc.code)
		}
	}
}

// checkStateConsistency checks that all data of a state root is present.
func checkStateConsistency(db ethdb.Database, scheme string, root common.Hash) error {
	config := &trie.Config{Preimages: true}
	if scheme == rawdb.PathScheme {
		config.PathDB = pathdb.Defaults
	}
	state, err := New(root, NewDatabaseWithConfig(db, config), nil)
	if err != nil {
		return err
	}
	it := newNodeIterator(state)
	for it.Next() {
	}
	return it.Error
}

// Tests that an empty state is not scheduled for syncing.
func TestEmptyStateSync(t *testing.T) {
	dbA := trie.NewDatabase(rawdb.NewMemoryDatabase(), nil)
	dbB := trie.NewDatabase(rawdb.NewMemoryDatabase(), &trie.Config{PathDB: pathdb.Defaults})

	sync := NewStateSync(types.EmptyRootHash, rawdb.NewMemoryDatabase(), nil, dbA.Scheme())
	if paths, nodes, codes := sync.Missing(1); len(paths) != 0 || len(nodes) != 0 || len(codes) != 0 {
		t.Errorf("content requested for empty state: %v, %v, %v", nodes, paths, codes)
	}
	sync = NewStateSync(types.EmptyRootHash, rawdb.NewMemoryDatabase(), nil, dbB.Scheme())
	if paths, nodes, codes := sync.Missing(1); len(paths) != 0 || len(nodes) != 0 || len(codes) != 0 {
		t.Errorf("content requested for empty state: %v, %v, %v", nodes, paths, codes)
	}
}

// Tests that given a root hash, a state can sync iteratively on a single thread,
// requesting retrieval tasks and returning all of them in one go.
func TestIterativeStateSyncIndividual(t *testing.T) {
	testIterativeStateSync(t, 1, false, false, rawdb.HashScheme)
	testIterativeStateSync(t, 1, false, false, rawdb.PathScheme)
}
func TestIterativeStateSyncBatched(t *testing.T) {
	testIterativeStateSync(t, 100, false, false, rawdb.HashScheme)
	testIterativeStateSync(t, 100, false, false, rawdb.PathScheme)
}
func TestIterativeStateSyncIndividualFromDisk(t *testing.T) {
	testIterativeStateSync(t, 1, true, false, rawdb.HashScheme)
	testIterativeStateSync(t, 1, true, false, rawdb.PathScheme)
}
func TestIterativeStateSyncBatchedFromDisk(t *testing.T) {
	testIterativeStateSync(t, 100, true, false, rawdb.HashScheme)
	testIterativeStateSync(t, 100, true, false, rawdb.PathScheme)
}
func TestIterativeStateSyncIndividualByPath(t *testing.T) {
	testIterativeStateSync(t, 1, false, true, rawdb.HashScheme)
	testIterativeStateSync(t, 1, false, true, rawdb.PathScheme)
}
func TestIterativeStateSyncBatchedByPath(t *testing.T) {
	testIterativeStateSync(t, 100, false, true, rawdb.HashScheme)
	testIterativeStateSync(t, 100, false, true, rawdb.PathScheme)
}

// stateElement represents the element in the state trie(bytecode or trie node).
type stateElement struct {
	path     string
	hash     common.Hash
	code     common.Hash
	syncPath trie.SyncPath
}

func testIterativeStateSync(t *testing.T, count int, commit bool, bypath bool, scheme string) {
	// Create a random state to copy
	srcDisk, srcDb, ndb, srcRoot, srcAccounts := makeTestState(scheme)
	if commit {
		ndb.Commit(srcRoot, false)
	}
	srcTrie, _ := trie.New(trie.StateTrieID(srcRoot), ndb)

	// Create a destination state and sync with the scheduler
	dstDb := rawdb.NewMemoryDatabase()
	sched := NewStateSync(srcRoot, dstDb, nil, ndb.Scheme())

	var (
		nodeElements []stateElement
		codeElements []stateElement
	)
	paths, nodes, codes := sched.Missing(count)
	for i := 0; i < len(paths); i++ {
		nodeElements = append(nodeElements, stateElement{
			path:     paths[i],
			hash:     nodes[i],
			syncPath: trie.NewSyncPath([]byte(paths[i])),
		})
	}
	for i := 0; i < len(codes); i++ {
		codeElements = append(codeElements, stateElement{code: codes[i]})
	}
	reader, err := ndb.Reader(srcRoot)
	if err != nil {
		t.Fatalf("state is not existent, %#x", srcRoot)
	}
	for len(nodeElements)+len(codeElements) > 0 {
		var (
			nodeResults = make([]trie.NodeSyncResult, len(nodeElements))
			codeResults = make([]trie.CodeSyncResult, len(codeElements))
		)
		for i, element := range codeElements {
			data, err := srcDb.ContractCode(common.Address{}, element.code)
			if err != nil {
				t.Fatalf("failed to retrieve contract bytecode for hash %x", element.code)
			}
			codeResults[i] = trie.CodeSyncResult{Hash: element.code, Data: data}
		}
		for i, node := range nodeElements {
			if bypath {
				if len(node.syncPath) == 1 {
					data, _, err := srcTrie.GetNode(node.syncPath[0])
					if err != nil {
						t.Fatalf("failed to retrieve node data for path %x: %v", node.syncPath[0], err)
					}
					nodeResults[i] = trie.NodeSyncResult{Path: node.path, Data: data}
				} else {
					var acc types.StateAccount
					if err := rlp.DecodeBytes(srcTrie.MustGet(node.syncPath[0]), &acc); err != nil {
						t.Fatalf("failed to decode account on path %x: %v", node.syncPath[0], err)
					}
					id := trie.StorageTrieID(srcRoot, common.BytesToHash(node.syncPath[0]), acc.Root)
					stTrie, err := trie.New(id, ndb)
					if err != nil {
						t.Fatalf("failed to retriev storage trie for path %x: %v", node.syncPath[1], err)
					}
					data, _, err := stTrie.GetNode(node.syncPath[1])
					if err != nil {
						t.Fatalf("failed to retrieve node data for path %x: %v", node.syncPath[1], err)
					}
					nodeResults[i] = trie.NodeSyncResult{Path: node.path, Data: data}
				}
			} else {
				owner, inner := trie.ResolvePath([]byte(node.path))
				data, err := reader.Node(owner, inner, node.hash)
				if err != nil {
					t.Fatalf("failed to retrieve node data for key %v", []byte(node.path))
				}
				nodeResults[i] = trie.NodeSyncResult{Path: node.path, Data: data}
			}
		}
		for _, result := range codeResults {
			if err := sched.ProcessCode(result); err != nil {
				t.Errorf("failed to process result %v", err)
			}
		}
		for _, result := range nodeResults {
			if err := sched.ProcessNode(result); err != nil {
				t.Errorf("failed to process result %v", err)
			}
		}
		batch := dstDb.NewBatch()
		if err := sched.Commit(batch); err != nil {
			t.Fatalf("failed to commit data: %v", err)
		}
		batch.Write()

		paths, nodes, codes = sched.Missing(count)
		nodeElements = nodeElements[:0]
		for i := 0; i < len(paths); i++ {
			nodeElements = append(nodeElements, stateElement{
				path:     paths[i],
				hash:     nodes[i],
				syncPath: trie.NewSyncPath([]byte(paths[i])),
			})
		}
		codeElements = codeElements[:0]
		for i := 0; i < len(codes); i++ {
			codeElements = append(codeElements, stateElement{
				code: codes[i],
			})
		}
	}
	// Copy the preimages from source db in order to traverse the state.
	srcDb.TrieDB().WritePreimages()
	copyPreimages(srcDisk, dstDb)

	// Cross check that the two states are in sync
	checkStateAccounts(t, dstDb, ndb.Scheme(), srcRoot, srcAccounts)
}

// Tests that the trie scheduler can correctly reconstruct the state even if only
// partial results are returned, and the others sent only later.
func TestIterativeDelayedStateSync(t *testing.T) {
	testIterativeDelayedStateSync(t, rawdb.HashScheme)
	testIterativeDelayedStateSync(t, rawdb.PathScheme)
}

func testIterativeDelayedStateSync(t *testing.T, scheme string) {
	// Create a random state to copy
	srcDisk, srcDb, ndb, srcRoot, srcAccounts := makeTestState(scheme)

	// Create a destination state and sync with the scheduler
	dstDb := rawdb.NewMemoryDatabase()
	sched := NewStateSync(srcRoot, dstDb, nil, ndb.Scheme())

	var (
		nodeElements []stateElement
		codeElements []stateElement
	)
	paths, nodes, codes := sched.Missing(0)
	for i := 0; i < len(paths); i++ {
		nodeElements = append(nodeElements, stateElement{
			path:     paths[i],
			hash:     nodes[i],
			syncPath: trie.NewSyncPath([]byte(paths[i])),
		})
	}
	for i := 0; i < len(codes); i++ {
		codeElements = append(codeElements, stateElement{code: codes[i]})
	}
	reader, err := ndb.Reader(srcRoot)
	if err != nil {
		t.Fatalf("state is not existent, %#x", srcRoot)
	}
	for len(nodeElements)+len(codeElements) > 0 {
		// Sync only half of the scheduled nodes
		var nodeProcessed int
		var codeProcessed int
		if len(codeElements) > 0 {
			codeResults := make([]trie.CodeSyncResult, len(codeElements)/2+1)
			for i, element := range codeElements[:len(codeResults)] {
				data, err := srcDb.ContractCode(common.Address{}, element.code)
				if err != nil {
					t.Fatalf("failed to retrieve contract bytecode for %x", element.code)
				}
				codeResults[i] = trie.CodeSyncResult{Hash: element.code, Data: data}
			}
			for _, result := range codeResults {
				if err := sched.ProcessCode(result); err != nil {
					t.Fatalf("failed to process result %v", err)
				}
			}
			codeProcessed = len(codeResults)
		}
		if len(nodeElements) > 0 {
			nodeResults := make([]trie.NodeSyncResult, len(nodeElements)/2+1)
			for i, element := range nodeElements[:len(nodeResults)] {
				owner, inner := trie.ResolvePath([]byte(element.path))
				data, err := reader.Node(owner, inner, element.hash)
				if err != nil {
					t.Fatalf("failed to retrieve contract bytecode for %x", element.code)
				}
				nodeResults[i] = trie.NodeSyncResult{Path: element.path, Data: data}
			}
			for _, result := range nodeResults {
				if err := sched.ProcessNode(result); err != nil {
					t.Fatalf("failed to process result %v", err)
				}
			}
			nodeProcessed = len(nodeResults)
		}
		batch := dstDb.NewBatch()
		if err := sched.Commit(batch); err != nil {
			t.Fatalf("failed to commit data: %v", err)
		}
		batch.Write()

		paths, nodes, codes = sched.Missing(0)
		nodeElements = nodeElements[nodeProcessed:]
		for i := 0; i < len(paths); i++ {
			nodeElements = append(nodeElements, stateElement{
				path:     paths[i],
				hash:     nodes[i],
				syncPath: trie.NewSyncPath([]byte(paths[i])),
			})
		}
		codeElements = codeElements[codeProcessed:]
		for i := 0; i < len(codes); i++ {
			codeElements = append(codeElements, stateElement{
				code: codes[i],
			})
		}
	}
	// Copy the preimages from source db in order to traverse the state.
	srcDb.TrieDB().WritePreimages()
	copyPreimages(srcDisk, dstDb)

	// Cross check that the two states are in sync
	checkStateAccounts(t, dstDb, ndb.Scheme(), srcRoot, srcAccounts)
}

// Tests that given a root hash, a trie can sync iteratively on a single thread,
// requesting retrieval tasks and returning all of them in one go, however in a
// random order.
func TestIterativeRandomStateSyncIndividual(t *testing.T) {
	testIterativeRandomStateSync(t, 1, rawdb.HashScheme)
	testIterativeRandomStateSync(t, 1, rawdb.PathScheme)
}
func TestIterativeRandomStateSyncBatched(t *testing.T) {
	testIterativeRandomStateSync(t, 100, rawdb.HashScheme)
	testIterativeRandomStateSync(t, 100, rawdb.PathScheme)
}

func testIterativeRandomStateSync(t *testing.T, count int, scheme string) {
	// Create a random state to copy
	srcDisk, srcDb, ndb, srcRoot, srcAccounts := makeTestState(scheme)

	// Create a destination state and sync with the scheduler
	dstDb := rawdb.NewMemoryDatabase()
	sched := NewStateSync(srcRoot, dstDb, nil, ndb.Scheme())

	nodeQueue := make(map[string]stateElement)
	codeQueue := make(map[common.Hash]struct{})
	paths, nodes, codes := sched.Missing(count)
	for i, path := range paths {
		nodeQueue[path] = stateElement{
			path:     path,
			hash:     nodes[i],
			syncPath: trie.NewSyncPath([]byte(path)),
		}
	}
	for _, hash := range codes {
		codeQueue[hash] = struct{}{}
	}
	reader, err := ndb.Reader(srcRoot)
	if err != nil {
		t.Fatalf("state is not existent, %#x", srcRoot)
	}
	for len(nodeQueue)+len(codeQueue) > 0 {
		// Fetch all the queued nodes in a random order
		if len(codeQueue) > 0 {
			results := make([]trie.CodeSyncResult, 0, len(codeQueue))
			for hash := range codeQueue {
				data, err := srcDb.ContractCode(common.Address{}, hash)
				if err != nil {
					t.Fatalf("failed to retrieve node data for %x", hash)
				}
				results = append(results, trie.CodeSyncResult{Hash: hash, Data: data})
			}
			for _, result := range results {
				if err := sched.ProcessCode(result); err != nil {
					t.Fatalf("failed to process result %v", err)
				}
			}
		}
		if len(nodeQueue) > 0 {
			results := make([]trie.NodeSyncResult, 0, len(nodeQueue))
			for path, element := range nodeQueue {
				owner, inner := trie.ResolvePath([]byte(element.path))
				data, err := reader.Node(owner, inner, element.hash)
				if err != nil {
					t.Fatalf("failed to retrieve node data for %x %v %v", element.hash, []byte(element.path), element.path)
				}
				results = append(results, trie.NodeSyncResult{Path: path, Data: data})
			}
			for _, result := range results {
				if err := sched.ProcessNode(result); err != nil {
					t.Fatalf("failed to process result %v", err)
				}
			}
		}
		batch := dstDb.NewBatch()
		if err := sched.Commit(batch); err != nil {
			t.Fatalf("failed to commit data: %v", err)
		}
		batch.Write()

		nodeQueue = make(map[string]stateElement)
		codeQueue = make(map[common.Hash]struct{})
		paths, nodes, codes := sched.Missing(count)
		for i, path := range paths {
			nodeQueue[path] = stateElement{
				path:     path,
				hash:     nodes[i],
				syncPath: trie.NewSyncPath([]byte(path)),
			}
		}
		for _, hash := range codes {
			codeQueue[hash] = struct{}{}
		}
	}
	// Copy the preimages from source db in order to traverse the state.
	srcDb.TrieDB().WritePreimages()
	copyPreimages(srcDisk, dstDb)

	// Cross check that the two states are in sync
	checkStateAccounts(t, dstDb, ndb.Scheme(), srcRoot, srcAccounts)
}

// Tests that the trie scheduler can correctly reconstruct the state even if only
// partial results are returned (Even those randomly), others sent only later.
func TestIterativeRandomDelayedStateSync(t *testing.T) {
	testIterativeRandomDelayedStateSync(t, rawdb.HashScheme)
	testIterativeRandomDelayedStateSync(t, rawdb.PathScheme)
}

func testIterativeRandomDelayedStateSync(t *testing.T, scheme string) {
	// Create a random state to copy
	srcDisk, srcDb, ndb, srcRoot, srcAccounts := makeTestState(scheme)

	// Create a destination state and sync with the scheduler
	dstDb := rawdb.NewMemoryDatabase()
	sched := NewStateSync(srcRoot, dstDb, nil, ndb.Scheme())

	nodeQueue := make(map[string]stateElement)
	codeQueue := make(map[common.Hash]struct{})
	paths, nodes, codes := sched.Missing(0)
	for i, path := range paths {
		nodeQueue[path] = stateElement{
			path:     path,
			hash:     nodes[i],
			syncPath: trie.NewSyncPath([]byte(path)),
		}
	}
	for _, hash := range codes {
		codeQueue[hash] = struct{}{}
	}
	reader, err := ndb.Reader(srcRoot)
	if err != nil {
		t.Fatalf("state is not existent, %#x", srcRoot)
	}
	for len(nodeQueue)+len(codeQueue) > 0 {
		// Sync only half of the scheduled nodes, even those in random order
		if len(codeQueue) > 0 {
			results := make([]trie.CodeSyncResult, 0, len(codeQueue)/2+1)
			for hash := range codeQueue {
				delete(codeQueue, hash)

				data, err := srcDb.ContractCode(common.Address{}, hash)
				if err != nil {
					t.Fatalf("failed to retrieve node data for %x", hash)
				}
				results = append(results, trie.CodeSyncResult{Hash: hash, Data: data})

				if len(results) >= cap(results) {
					break
				}
			}
			for _, result := range results {
				if err := sched.ProcessCode(result); err != nil {
					t.Fatalf("failed to process result %v", err)
				}
			}
		}
		if len(nodeQueue) > 0 {
			results := make([]trie.NodeSyncResult, 0, len(nodeQueue)/2+1)
			for path, element := range nodeQueue {
				delete(nodeQueue, path)

				owner, inner := trie.ResolvePath([]byte(element.path))
				data, err := reader.Node(owner, inner, element.hash)
				if err != nil {
					t.Fatalf("failed to retrieve node data for %x", element.hash)
				}
				results = append(results, trie.NodeSyncResult{Path: path, Data: data})

				if len(results) >= cap(results) {
					break
				}
			}
			// Feed the retrieved results back and queue new tasks
			for _, result := range results {
				if err := sched.ProcessNode(result); err != nil {
					t.Fatalf("failed to process result %v", err)
				}
			}
		}
		batch := dstDb.NewBatch()
		if err := sched.Commit(batch); err != nil {
			t.Fatalf("failed to commit data: %v", err)
		}
		batch.Write()

		paths, nodes, codes := sched.Missing(0)
		for i, path := range paths {
			nodeQueue[path] = stateElement{
				path:     path,
				hash:     nodes[i],
				syncPath: trie.NewSyncPath([]byte(path)),
			}
		}
		for _, hash := range codes {
			codeQueue[hash] = struct{}{}
		}
	}
	// Copy the preimages from source db in order to traverse the state.
	srcDb.TrieDB().WritePreimages()
	copyPreimages(srcDisk, dstDb)

	// Cross check that the two states are in sync
	checkStateAccounts(t, dstDb, ndb.Scheme(), srcRoot, srcAccounts)
}

// Tests that at any point in time during a sync, only complete sub-tries are in
// the database.
func TestIncompleteStateSync(t *testing.T) {
	testIncompleteStateSync(t, rawdb.HashScheme)
	testIncompleteStateSync(t, rawdb.PathScheme)
}

func testIncompleteStateSync(t *testing.T, scheme string) {
	// Create a random state to copy
	db, srcDb, ndb, srcRoot, srcAccounts := makeTestState(scheme)

	// isCodeLookup to save some hashing
	var isCode = make(map[common.Hash]struct{})
	for _, acc := range srcAccounts {
		if len(acc.code) > 0 {
			isCode[crypto.Keccak256Hash(acc.code)] = struct{}{}
		}
	}
	isCode[types.EmptyCodeHash] = struct{}{}

	// Create a destination state and sync with the scheduler
	dstDb := rawdb.NewMemoryDatabase()
	sched := NewStateSync(srcRoot, dstDb, nil, ndb.Scheme())

	var (
		addedCodes  []common.Hash
		addedPaths  []string
		addedHashes []common.Hash
	)
	reader, err := ndb.Reader(srcRoot)
	if err != nil {
		t.Fatalf("state is not available %x", srcRoot)
	}
	nodeQueue := make(map[string]stateElement)
	codeQueue := make(map[common.Hash]struct{})
	paths, nodes, codes := sched.Missing(1)
	for i, path := range paths {
		nodeQueue[path] = stateElement{
			path:     path,
			hash:     nodes[i],
			syncPath: trie.NewSyncPath([]byte(path)),
		}
	}
	for _, hash := range codes {
		codeQueue[hash] = struct{}{}
	}
	for len(nodeQueue)+len(codeQueue) > 0 {
		// Fetch a batch of state nodes
		if len(codeQueue) > 0 {
			results := make([]trie.CodeSyncResult, 0, len(codeQueue))
			for hash := range codeQueue {
				data, err := srcDb.ContractCode(common.Address{}, hash)
				if err != nil {
					t.Fatalf("failed to retrieve node data for %x", hash)
				}
				results = append(results, trie.CodeSyncResult{Hash: hash, Data: data})
				addedCodes = append(addedCodes, hash)
			}
			// Process each of the state nodes
			for _, result := range results {
				if err := sched.ProcessCode(result); err != nil {
					t.Fatalf("failed to process result %v", err)
				}
			}
		}
		if len(nodeQueue) > 0 {
			results := make([]trie.NodeSyncResult, 0, len(nodeQueue))
			for path, element := range nodeQueue {
				owner, inner := trie.ResolvePath([]byte(element.path))
				data, err := reader.Node(owner, inner, element.hash)
				if err != nil {
					t.Fatalf("failed to retrieve node data for %x", element.hash)
				}
				results = append(results, trie.NodeSyncResult{Path: path, Data: data})

				if element.hash != srcRoot {
					addedPaths = append(addedPaths, element.path)
					addedHashes = append(addedHashes, element.hash)
				}
			}
			// Process each of the state nodes
			for _, result := range results {
				if err := sched.ProcessNode(result); err != nil {
					t.Fatalf("failed to process result %v", err)
				}
			}
		}
		batch := dstDb.NewBatch()
		if err := sched.Commit(batch); err != nil {
			t.Fatalf("failed to commit data: %v", err)
		}
		batch.Write()

		// Fetch the next batch to retrieve
		nodeQueue = make(map[string]stateElement)
		codeQueue = make(map[common.Hash]struct{})
		paths, nodes, codes := sched.Missing(1)
		for i, path := range paths {
			nodeQueue[path] = stateElement{
				path:     path,
				hash:     nodes[i],
				syncPath: trie.NewSyncPath([]byte(path)),
			}
		}
		for _, hash := range codes {
			codeQueue[hash] = struct{}{}
		}
	}
	// Copy the preimages from source db in order to traverse the state.
	srcDb.TrieDB().WritePreimages()
	copyPreimages(db, dstDb)

	// Sanity check that removing any node from the database is detected
	for _, node := range addedCodes {
		val := rawdb.ReadCode(dstDb, node)
		rawdb.DeleteCode(dstDb, node)
		if err := checkStateConsistency(dstDb, ndb.Scheme(), srcRoot); err == nil {
			t.Errorf("trie inconsistency not caught, missing: %x", node)
		}
		rawdb.WriteCode(dstDb, node, val)
	}
	for i, path := range addedPaths {
		owner, inner := trie.ResolvePath([]byte(path))
		hash := addedHashes[i]
		val := rawdb.ReadTrieNode(dstDb, owner, inner, hash, scheme)
		if val == nil {
			t.Error("missing trie node")
		}
		rawdb.DeleteTrieNode(dstDb, owner, inner, hash, scheme)
		if err := checkStateConsistency(dstDb, scheme, srcRoot); err == nil {
			t.Errorf("trie inconsistency not caught, missing: %v", path)
		}
		rawdb.WriteTrieNode(dstDb, owner, inner, hash, val, scheme)
	}
}

func copyPreimages(srcDb, dstDb ethdb.Database) {
	it := srcDb.NewIterator(rawdb.PreimagePrefix, nil)
	defer it.Release()

	preimages := make(map[common.Hash][]byte)
	for it.Next() {
		hash := it.Key()[len(rawdb.PreimagePrefix):]
		preimages[common.BytesToHash(hash)] = common.CopyBytes(it.Value())
	}
	rawdb.WritePreimages(dstDb, preimages)
}
