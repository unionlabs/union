// Copyright 2023 The go-ethereum Authors
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

package pathdb

import (
	"bytes"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/trie/trienode"
	"github.com/ethereum/go-ethereum/trie/triestate"
	"golang.org/x/exp/slices"
)

// testHasher is a test utility for computing root hash of a batch of state
// elements. The hash algorithm is to sort all the elements in lexicographical
// order, concat the key and value in turn, and perform hash calculation on
// the concatenated bytes. Except the root hash, a nodeset will be returned
// once Commit is called, which contains all the changes made to hasher.
type testHasher struct {
	owner   common.Hash            // owner identifier
	root    common.Hash            // original root
	dirties map[common.Hash][]byte // dirty states
	cleans  map[common.Hash][]byte // clean states
}

// newTestHasher constructs a hasher object with provided states.
func newTestHasher(owner common.Hash, root common.Hash, cleans map[common.Hash][]byte) (*testHasher, error) {
	if cleans == nil {
		cleans = make(map[common.Hash][]byte)
	}
	if got, _ := hash(cleans); got != root {
		return nil, fmt.Errorf("state root mismatched, want: %x, got: %x", root, got)
	}
	return &testHasher{
		owner:   owner,
		root:    root,
		dirties: make(map[common.Hash][]byte),
		cleans:  cleans,
	}, nil
}

// Get returns the value for key stored in the trie.
func (h *testHasher) Get(key []byte) ([]byte, error) {
	hash := common.BytesToHash(key)
	val, ok := h.dirties[hash]
	if ok {
		return val, nil
	}
	return h.cleans[hash], nil
}

// Update associates key with value in the trie.
func (h *testHasher) Update(key, value []byte) error {
	h.dirties[common.BytesToHash(key)] = common.CopyBytes(value)
	return nil
}

// Delete removes any existing value for key from the trie.
func (h *testHasher) Delete(key []byte) error {
	h.dirties[common.BytesToHash(key)] = nil
	return nil
}

// Commit computes the new hash of the states and returns the set with all
// state changes.
func (h *testHasher) Commit(collectLeaf bool) (common.Hash, *trienode.NodeSet, error) {
	var (
		nodes = make(map[common.Hash][]byte)
		set   = trienode.NewNodeSet(h.owner)
	)
	for hash, val := range h.cleans {
		nodes[hash] = val
	}
	for hash, val := range h.dirties {
		nodes[hash] = val
		if bytes.Equal(val, h.cleans[hash]) {
			continue
		}
		if len(val) == 0 {
			set.AddNode(hash.Bytes(), trienode.NewDeleted())
		} else {
			set.AddNode(hash.Bytes(), trienode.New(crypto.Keccak256Hash(val), val))
		}
	}
	root, blob := hash(nodes)

	// Include the dirty root node as well.
	if root != types.EmptyRootHash && root != h.root {
		set.AddNode(nil, trienode.New(root, blob))
	}
	if root == types.EmptyRootHash && h.root != types.EmptyRootHash {
		set.AddNode(nil, trienode.NewDeleted())
	}
	return root, set, nil
}

// hash performs the hash computation upon the provided states.
func hash(states map[common.Hash][]byte) (common.Hash, []byte) {
	var hs []common.Hash
	for hash := range states {
		hs = append(hs, hash)
	}
	slices.SortFunc(hs, common.Hash.Cmp)

	var input []byte
	for _, hash := range hs {
		if len(states[hash]) == 0 {
			continue
		}
		input = append(input, hash.Bytes()...)
		input = append(input, states[hash]...)
	}
	if len(input) == 0 {
		return types.EmptyRootHash, nil
	}
	return crypto.Keccak256Hash(input), input
}

type hashLoader struct {
	accounts map[common.Hash][]byte
	storages map[common.Hash]map[common.Hash][]byte
}

func newHashLoader(accounts map[common.Hash][]byte, storages map[common.Hash]map[common.Hash][]byte) *hashLoader {
	return &hashLoader{
		accounts: accounts,
		storages: storages,
	}
}

// OpenTrie opens the main account trie.
func (l *hashLoader) OpenTrie(root common.Hash) (triestate.Trie, error) {
	return newTestHasher(common.Hash{}, root, l.accounts)
}

// OpenStorageTrie opens the storage trie of an account.
func (l *hashLoader) OpenStorageTrie(stateRoot common.Hash, addrHash, root common.Hash) (triestate.Trie, error) {
	return newTestHasher(addrHash, root, l.storages[addrHash])
}
