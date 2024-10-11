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
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

// NewStateSync create a new state trie download scheduler.
func NewStateSync(root common.Hash, database ethdb.KeyValueReader, onLeaf func(keys [][]byte, leaf []byte) error, scheme string) *trie.Sync {
	// Register the storage slot callback if the external callback is specified.
	var onSlot func(keys [][]byte, path []byte, leaf []byte, parent common.Hash, parentPath []byte) error
	if onLeaf != nil {
		onSlot = func(keys [][]byte, path []byte, leaf []byte, parent common.Hash, parentPath []byte) error {
			return onLeaf(keys, leaf)
		}
	}
	// Register the account callback to connect the state trie and the storage
	// trie belongs to the contract.
	var syncer *trie.Sync
	onAccount := func(keys [][]byte, path []byte, leaf []byte, parent common.Hash, parentPath []byte) error {
		if onLeaf != nil {
			if err := onLeaf(keys, leaf); err != nil {
				return err
			}
		}
		var obj types.StateAccount
		if err := rlp.DecodeBytes(leaf, &obj); err != nil {
			return err
		}
		syncer.AddSubTrie(obj.Root, path, parent, parentPath, onSlot)
		syncer.AddCodeEntry(common.BytesToHash(obj.CodeHash), path, parent, parentPath)
		return nil
	}
	syncer = trie.NewSync(root, database, onAccount, scheme)
	return syncer
}
