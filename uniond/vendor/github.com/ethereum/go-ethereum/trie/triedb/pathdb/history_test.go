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
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>

package pathdb

import (
	"bytes"
	"fmt"
	"reflect"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie/testutil"
	"github.com/ethereum/go-ethereum/trie/triestate"
)

// randomStateSet generates a random state change set.
func randomStateSet(n int) *triestate.Set {
	var (
		accounts = make(map[common.Address][]byte)
		storages = make(map[common.Address]map[common.Hash][]byte)
	)
	for i := 0; i < n; i++ {
		addr := testutil.RandomAddress()
		storages[addr] = make(map[common.Hash][]byte)
		for j := 0; j < 3; j++ {
			v, _ := rlp.EncodeToBytes(common.TrimLeftZeroes(testutil.RandBytes(32)))
			storages[addr][testutil.RandomHash()] = v
		}
		account := generateAccount(types.EmptyRootHash)
		accounts[addr] = types.SlimAccountRLP(account)
	}
	return triestate.New(accounts, storages, nil)
}

func makeHistory() *history {
	return newHistory(testutil.RandomHash(), types.EmptyRootHash, 0, randomStateSet(3))
}

func makeHistories(n int) []*history {
	var (
		parent = types.EmptyRootHash
		result []*history
	)
	for i := 0; i < n; i++ {
		root := testutil.RandomHash()
		h := newHistory(root, parent, uint64(i), randomStateSet(3))
		parent = root
		result = append(result, h)
	}
	return result
}

func TestEncodeDecodeHistory(t *testing.T) {
	var (
		m   meta
		dec history
		obj = makeHistory()
	)
	// check if meta data can be correctly encode/decode
	blob := obj.meta.encode()
	if err := m.decode(blob); err != nil {
		t.Fatalf("Failed to decode %v", err)
	}
	if !reflect.DeepEqual(&m, obj.meta) {
		t.Fatal("meta is mismatched")
	}

	// check if account/storage data can be correctly encode/decode
	accountData, storageData, accountIndexes, storageIndexes := obj.encode()
	if err := dec.decode(accountData, storageData, accountIndexes, storageIndexes); err != nil {
		t.Fatalf("Failed to decode, err: %v", err)
	}
	if !compareSet(dec.accounts, obj.accounts) {
		t.Fatal("account data is mismatched")
	}
	if !compareStorages(dec.storages, obj.storages) {
		t.Fatal("storage data is mismatched")
	}
	if !compareList(dec.accountList, obj.accountList) {
		t.Fatal("account list is mismatched")
	}
	if !compareStorageList(dec.storageList, obj.storageList) {
		t.Fatal("storage list is mismatched")
	}
}

func checkHistory(t *testing.T, db ethdb.KeyValueReader, freezer *rawdb.ResettableFreezer, id uint64, root common.Hash, exist bool) {
	blob := rawdb.ReadStateHistoryMeta(freezer, id)
	if exist && len(blob) == 0 {
		t.Fatalf("Failed to load trie history, %d", id)
	}
	if !exist && len(blob) != 0 {
		t.Fatalf("Unexpected trie history, %d", id)
	}
	if exist && rawdb.ReadStateID(db, root) == nil {
		t.Fatalf("Root->ID mapping is not found, %d", id)
	}
	if !exist && rawdb.ReadStateID(db, root) != nil {
		t.Fatalf("Unexpected root->ID mapping, %d", id)
	}
}

func checkHistoriesInRange(t *testing.T, db ethdb.KeyValueReader, freezer *rawdb.ResettableFreezer, from, to uint64, roots []common.Hash, exist bool) {
	for i, j := from, 0; i <= to; i, j = i+1, j+1 {
		checkHistory(t, db, freezer, i, roots[j], exist)
	}
}

func TestTruncateHeadHistory(t *testing.T) {
	var (
		roots      []common.Hash
		hs         = makeHistories(10)
		db         = rawdb.NewMemoryDatabase()
		freezer, _ = openFreezer(t.TempDir(), false)
	)
	defer freezer.Close()

	for i := 0; i < len(hs); i++ {
		accountData, storageData, accountIndex, storageIndex := hs[i].encode()
		rawdb.WriteStateHistory(freezer, uint64(i+1), hs[i].meta.encode(), accountIndex, storageIndex, accountData, storageData)
		rawdb.WriteStateID(db, hs[i].meta.root, uint64(i+1))
		roots = append(roots, hs[i].meta.root)
	}
	for size := len(hs); size > 0; size-- {
		pruned, err := truncateFromHead(db, freezer, uint64(size-1))
		if err != nil {
			t.Fatalf("Failed to truncate from head %v", err)
		}
		if pruned != 1 {
			t.Error("Unexpected pruned items", "want", 1, "got", pruned)
		}
		checkHistoriesInRange(t, db, freezer, uint64(size), uint64(10), roots[size-1:], false)
		checkHistoriesInRange(t, db, freezer, uint64(1), uint64(size-1), roots[:size-1], true)
	}
}

func TestTruncateTailHistory(t *testing.T) {
	var (
		roots      []common.Hash
		hs         = makeHistories(10)
		db         = rawdb.NewMemoryDatabase()
		freezer, _ = openFreezer(t.TempDir(), false)
	)
	defer freezer.Close()

	for i := 0; i < len(hs); i++ {
		accountData, storageData, accountIndex, storageIndex := hs[i].encode()
		rawdb.WriteStateHistory(freezer, uint64(i+1), hs[i].meta.encode(), accountIndex, storageIndex, accountData, storageData)
		rawdb.WriteStateID(db, hs[i].meta.root, uint64(i+1))
		roots = append(roots, hs[i].meta.root)
	}
	for newTail := 1; newTail < len(hs); newTail++ {
		pruned, _ := truncateFromTail(db, freezer, uint64(newTail))
		if pruned != 1 {
			t.Error("Unexpected pruned items", "want", 1, "got", pruned)
		}
		checkHistoriesInRange(t, db, freezer, uint64(1), uint64(newTail), roots[:newTail], false)
		checkHistoriesInRange(t, db, freezer, uint64(newTail+1), uint64(10), roots[newTail:], true)
	}
}

func TestTruncateTailHistories(t *testing.T) {
	var cases = []struct {
		limit       uint64
		expPruned   int
		maxPruned   uint64
		minUnpruned uint64
		empty       bool
	}{
		{
			1, 9, 9, 10, false,
		},
		{
			0, 10, 10, 0 /* no meaning */, true,
		},
		{
			10, 0, 0, 1, false,
		},
	}
	for i, c := range cases {
		var (
			roots      []common.Hash
			hs         = makeHistories(10)
			db         = rawdb.NewMemoryDatabase()
			freezer, _ = openFreezer(t.TempDir()+fmt.Sprintf("%d", i), false)
		)
		defer freezer.Close()

		for i := 0; i < len(hs); i++ {
			accountData, storageData, accountIndex, storageIndex := hs[i].encode()
			rawdb.WriteStateHistory(freezer, uint64(i+1), hs[i].meta.encode(), accountIndex, storageIndex, accountData, storageData)
			rawdb.WriteStateID(db, hs[i].meta.root, uint64(i+1))
			roots = append(roots, hs[i].meta.root)
		}
		pruned, _ := truncateFromTail(db, freezer, uint64(10)-c.limit)
		if pruned != c.expPruned {
			t.Error("Unexpected pruned items", "want", c.expPruned, "got", pruned)
		}
		if c.empty {
			checkHistoriesInRange(t, db, freezer, uint64(1), uint64(10), roots, false)
		} else {
			tail := 10 - int(c.limit)
			checkHistoriesInRange(t, db, freezer, uint64(1), c.maxPruned, roots[:tail], false)
			checkHistoriesInRange(t, db, freezer, c.minUnpruned, uint64(10), roots[tail:], true)
		}
	}
}

func TestTruncateOutOfRange(t *testing.T) {
	var (
		hs         = makeHistories(10)
		db         = rawdb.NewMemoryDatabase()
		freezer, _ = openFreezer(t.TempDir(), false)
	)
	defer freezer.Close()

	for i := 0; i < len(hs); i++ {
		accountData, storageData, accountIndex, storageIndex := hs[i].encode()
		rawdb.WriteStateHistory(freezer, uint64(i+1), hs[i].meta.encode(), accountIndex, storageIndex, accountData, storageData)
		rawdb.WriteStateID(db, hs[i].meta.root, uint64(i+1))
	}
	truncateFromTail(db, freezer, uint64(len(hs)/2))

	// Ensure of-out-range truncations are rejected correctly.
	head, _ := freezer.Ancients()
	tail, _ := freezer.Tail()

	cases := []struct {
		mode   int
		target uint64
		expErr error
	}{
		{0, head, nil}, // nothing to delete
		{0, head + 1, fmt.Errorf("out of range, tail: %d, head: %d, target: %d", tail, head, head+1)},
		{0, tail - 1, fmt.Errorf("out of range, tail: %d, head: %d, target: %d", tail, head, tail-1)},
		{1, tail, nil}, // nothing to delete
		{1, head + 1, fmt.Errorf("out of range, tail: %d, head: %d, target: %d", tail, head, head+1)},
		{1, tail - 1, fmt.Errorf("out of range, tail: %d, head: %d, target: %d", tail, head, tail-1)},
	}
	for _, c := range cases {
		var gotErr error
		if c.mode == 0 {
			_, gotErr = truncateFromHead(db, freezer, c.target)
		} else {
			_, gotErr = truncateFromTail(db, freezer, c.target)
		}
		if !reflect.DeepEqual(gotErr, c.expErr) {
			t.Errorf("Unexpected error, want: %v, got: %v", c.expErr, gotErr)
		}
	}
}

// openFreezer initializes the freezer instance for storing state histories.
func openFreezer(datadir string, readOnly bool) (*rawdb.ResettableFreezer, error) {
	return rawdb.NewStateFreezer(datadir, readOnly)
}

func compareSet[k comparable](a, b map[k][]byte) bool {
	if len(a) != len(b) {
		return false
	}
	for key, valA := range a {
		valB, ok := b[key]
		if !ok {
			return false
		}
		if !bytes.Equal(valA, valB) {
			return false
		}
	}
	return true
}

func compareList[k comparable](a, b []k) bool {
	if len(a) != len(b) {
		return false
	}
	for i := 0; i < len(a); i++ {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

func compareStorages(a, b map[common.Address]map[common.Hash][]byte) bool {
	if len(a) != len(b) {
		return false
	}
	for h, subA := range a {
		subB, ok := b[h]
		if !ok {
			return false
		}
		if !compareSet(subA, subB) {
			return false
		}
	}
	return true
}

func compareStorageList(a, b map[common.Address][]common.Hash) bool {
	if len(a) != len(b) {
		return false
	}
	for h, la := range a {
		lb, ok := b[h]
		if !ok {
			return false
		}
		if !compareList(la, lb) {
			return false
		}
	}
	return true
}
