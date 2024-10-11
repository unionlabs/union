// Copyright 2019 The go-ethereum Authors
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

package snapshot

import (
	"bytes"
	crand "crypto/rand"
	"math/rand"
	"testing"

	"github.com/VictoriaMetrics/fastcache"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethdb/memorydb"
)

func copyDestructs(destructs map[common.Hash]struct{}) map[common.Hash]struct{} {
	copy := make(map[common.Hash]struct{})
	for hash := range destructs {
		copy[hash] = struct{}{}
	}
	return copy
}

func copyAccounts(accounts map[common.Hash][]byte) map[common.Hash][]byte {
	copy := make(map[common.Hash][]byte)
	for hash, blob := range accounts {
		copy[hash] = blob
	}
	return copy
}

func copyStorage(storage map[common.Hash]map[common.Hash][]byte) map[common.Hash]map[common.Hash][]byte {
	copy := make(map[common.Hash]map[common.Hash][]byte)
	for accHash, slots := range storage {
		copy[accHash] = make(map[common.Hash][]byte)
		for slotHash, blob := range slots {
			copy[accHash][slotHash] = blob
		}
	}
	return copy
}

// TestMergeBasics tests some simple merges
func TestMergeBasics(t *testing.T) {
	var (
		destructs = make(map[common.Hash]struct{})
		accounts  = make(map[common.Hash][]byte)
		storage   = make(map[common.Hash]map[common.Hash][]byte)
	)
	// Fill up a parent
	for i := 0; i < 100; i++ {
		h := randomHash()
		data := randomAccount()

		accounts[h] = data
		if rand.Intn(4) == 0 {
			destructs[h] = struct{}{}
		}
		if rand.Intn(2) == 0 {
			accStorage := make(map[common.Hash][]byte)
			value := make([]byte, 32)
			crand.Read(value)
			accStorage[randomHash()] = value
			storage[h] = accStorage
		}
	}
	// Add some (identical) layers on top
	parent := newDiffLayer(emptyLayer(), common.Hash{}, copyDestructs(destructs), copyAccounts(accounts), copyStorage(storage))
	child := newDiffLayer(parent, common.Hash{}, copyDestructs(destructs), copyAccounts(accounts), copyStorage(storage))
	child = newDiffLayer(child, common.Hash{}, copyDestructs(destructs), copyAccounts(accounts), copyStorage(storage))
	child = newDiffLayer(child, common.Hash{}, copyDestructs(destructs), copyAccounts(accounts), copyStorage(storage))
	child = newDiffLayer(child, common.Hash{}, copyDestructs(destructs), copyAccounts(accounts), copyStorage(storage))
	// And flatten
	merged := (child.flatten()).(*diffLayer)

	{ // Check account lists
		if have, want := len(merged.accountList), 0; have != want {
			t.Errorf("accountList wrong: have %v, want %v", have, want)
		}
		if have, want := len(merged.AccountList()), len(accounts); have != want {
			t.Errorf("AccountList() wrong: have %v, want %v", have, want)
		}
		if have, want := len(merged.accountList), len(accounts); have != want {
			t.Errorf("accountList [2] wrong: have %v, want %v", have, want)
		}
	}
	{ // Check account drops
		if have, want := len(merged.destructSet), len(destructs); have != want {
			t.Errorf("accountDrop wrong: have %v, want %v", have, want)
		}
	}
	{ // Check storage lists
		i := 0
		for aHash, sMap := range storage {
			if have, want := len(merged.storageList), i; have != want {
				t.Errorf("[1] storageList wrong: have %v, want %v", have, want)
			}
			list, _ := merged.StorageList(aHash)
			if have, want := len(list), len(sMap); have != want {
				t.Errorf("[2] StorageList() wrong: have %v, want %v", have, want)
			}
			if have, want := len(merged.storageList[aHash]), len(sMap); have != want {
				t.Errorf("storageList wrong: have %v, want %v", have, want)
			}
			i++
		}
	}
}

// TestMergeDelete tests some deletion
func TestMergeDelete(t *testing.T) {
	var (
		storage = make(map[common.Hash]map[common.Hash][]byte)
	)
	// Fill up a parent
	h1 := common.HexToHash("0x01")
	h2 := common.HexToHash("0x02")

	flipDrops := func() map[common.Hash]struct{} {
		return map[common.Hash]struct{}{
			h2: {},
		}
	}
	flipAccs := func() map[common.Hash][]byte {
		return map[common.Hash][]byte{
			h1: randomAccount(),
		}
	}
	flopDrops := func() map[common.Hash]struct{} {
		return map[common.Hash]struct{}{
			h1: {},
		}
	}
	flopAccs := func() map[common.Hash][]byte {
		return map[common.Hash][]byte{
			h2: randomAccount(),
		}
	}
	// Add some flipAccs-flopping layers on top
	parent := newDiffLayer(emptyLayer(), common.Hash{}, flipDrops(), flipAccs(), storage)
	child := parent.Update(common.Hash{}, flopDrops(), flopAccs(), storage)
	child = child.Update(common.Hash{}, flipDrops(), flipAccs(), storage)
	child = child.Update(common.Hash{}, flopDrops(), flopAccs(), storage)
	child = child.Update(common.Hash{}, flipDrops(), flipAccs(), storage)
	child = child.Update(common.Hash{}, flopDrops(), flopAccs(), storage)
	child = child.Update(common.Hash{}, flipDrops(), flipAccs(), storage)

	if data, _ := child.Account(h1); data == nil {
		t.Errorf("last diff layer: expected %x account to be non-nil", h1)
	}
	if data, _ := child.Account(h2); data != nil {
		t.Errorf("last diff layer: expected %x account to be nil", h2)
	}
	if _, ok := child.destructSet[h1]; ok {
		t.Errorf("last diff layer: expected %x drop to be missing", h1)
	}
	if _, ok := child.destructSet[h2]; !ok {
		t.Errorf("last diff layer: expected %x drop to be present", h1)
	}
	// And flatten
	merged := (child.flatten()).(*diffLayer)

	if data, _ := merged.Account(h1); data == nil {
		t.Errorf("merged layer: expected %x account to be non-nil", h1)
	}
	if data, _ := merged.Account(h2); data != nil {
		t.Errorf("merged layer: expected %x account to be nil", h2)
	}
	if _, ok := merged.destructSet[h1]; !ok { // Note, drops stay alive until persisted to disk!
		t.Errorf("merged diff layer: expected %x drop to be present", h1)
	}
	if _, ok := merged.destructSet[h2]; !ok { // Note, drops stay alive until persisted to disk!
		t.Errorf("merged diff layer: expected %x drop to be present", h1)
	}
	// If we add more granular metering of memory, we can enable this again,
	// but it's not implemented for now
	//if have, want := merged.memory, child.memory; have != want {
	//	t.Errorf("mem wrong: have %d, want %d", have, want)
	//}
}

// This tests that if we create a new account, and set a slot, and then merge
// it, the lists will be correct.
func TestInsertAndMerge(t *testing.T) {
	// Fill up a parent
	var (
		acc    = common.HexToHash("0x01")
		slot   = common.HexToHash("0x02")
		parent *diffLayer
		child  *diffLayer
	)
	{
		var (
			destructs = make(map[common.Hash]struct{})
			accounts  = make(map[common.Hash][]byte)
			storage   = make(map[common.Hash]map[common.Hash][]byte)
		)
		parent = newDiffLayer(emptyLayer(), common.Hash{}, destructs, accounts, storage)
	}
	{
		var (
			destructs = make(map[common.Hash]struct{})
			accounts  = make(map[common.Hash][]byte)
			storage   = make(map[common.Hash]map[common.Hash][]byte)
		)
		accounts[acc] = randomAccount()
		storage[acc] = make(map[common.Hash][]byte)
		storage[acc][slot] = []byte{0x01}
		child = newDiffLayer(parent, common.Hash{}, destructs, accounts, storage)
	}
	// And flatten
	merged := (child.flatten()).(*diffLayer)
	{ // Check that slot value is present
		have, _ := merged.Storage(acc, slot)
		if want := []byte{0x01}; !bytes.Equal(have, want) {
			t.Errorf("merged slot value wrong: have %x, want %x", have, want)
		}
	}
}

func emptyLayer() *diskLayer {
	return &diskLayer{
		diskdb: memorydb.New(),
		cache:  fastcache.New(500 * 1024),
	}
}

// BenchmarkSearch checks how long it takes to find a non-existing key
// BenchmarkSearch-6   	  200000	     10481 ns/op (1K per layer)
// BenchmarkSearch-6   	  200000	     10760 ns/op (10K per layer)
// BenchmarkSearch-6   	  100000	     17866 ns/op
//
// BenchmarkSearch-6   	  500000	      3723 ns/op (10k per layer, only top-level RLock()
func BenchmarkSearch(b *testing.B) {
	// First, we set up 128 diff layers, with 1K items each
	fill := func(parent snapshot) *diffLayer {
		var (
			destructs = make(map[common.Hash]struct{})
			accounts  = make(map[common.Hash][]byte)
			storage   = make(map[common.Hash]map[common.Hash][]byte)
		)
		for i := 0; i < 10000; i++ {
			accounts[randomHash()] = randomAccount()
		}
		return newDiffLayer(parent, common.Hash{}, destructs, accounts, storage)
	}
	var layer snapshot
	layer = emptyLayer()
	for i := 0; i < 128; i++ {
		layer = fill(layer)
	}
	key := crypto.Keccak256Hash([]byte{0x13, 0x38})
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		layer.AccountRLP(key)
	}
}

// BenchmarkSearchSlot checks how long it takes to find a non-existing key
// - Number of layers: 128
// - Each layers contains the account, with a couple of storage slots
// BenchmarkSearchSlot-6   	  100000	     14554 ns/op
// BenchmarkSearchSlot-6   	  100000	     22254 ns/op (when checking parent root using mutex)
// BenchmarkSearchSlot-6   	  100000	     14551 ns/op (when checking parent number using atomic)
// With bloom filter:
// BenchmarkSearchSlot-6   	 3467835	       351 ns/op
func BenchmarkSearchSlot(b *testing.B) {
	// First, we set up 128 diff layers, with 1K items each
	accountKey := crypto.Keccak256Hash([]byte{0x13, 0x37})
	storageKey := crypto.Keccak256Hash([]byte{0x13, 0x37})
	accountRLP := randomAccount()
	fill := func(parent snapshot) *diffLayer {
		var (
			destructs = make(map[common.Hash]struct{})
			accounts  = make(map[common.Hash][]byte)
			storage   = make(map[common.Hash]map[common.Hash][]byte)
		)
		accounts[accountKey] = accountRLP

		accStorage := make(map[common.Hash][]byte)
		for i := 0; i < 5; i++ {
			value := make([]byte, 32)
			crand.Read(value)
			accStorage[randomHash()] = value
			storage[accountKey] = accStorage
		}
		return newDiffLayer(parent, common.Hash{}, destructs, accounts, storage)
	}
	var layer snapshot
	layer = emptyLayer()
	for i := 0; i < 128; i++ {
		layer = fill(layer)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		layer.Storage(accountKey, storageKey)
	}
}

// With accountList and sorting
// BenchmarkFlatten-6   	      50	  29890856 ns/op
//
// Without sorting and tracking accountList
// BenchmarkFlatten-6   	     300	   5511511 ns/op
func BenchmarkFlatten(b *testing.B) {
	fill := func(parent snapshot) *diffLayer {
		var (
			destructs = make(map[common.Hash]struct{})
			accounts  = make(map[common.Hash][]byte)
			storage   = make(map[common.Hash]map[common.Hash][]byte)
		)
		for i := 0; i < 100; i++ {
			accountKey := randomHash()
			accounts[accountKey] = randomAccount()

			accStorage := make(map[common.Hash][]byte)
			for i := 0; i < 20; i++ {
				value := make([]byte, 32)
				crand.Read(value)
				accStorage[randomHash()] = value
			}
			storage[accountKey] = accStorage
		}
		return newDiffLayer(parent, common.Hash{}, destructs, accounts, storage)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		var layer snapshot
		layer = emptyLayer()
		for i := 1; i < 128; i++ {
			layer = fill(layer)
		}
		b.StartTimer()

		for i := 1; i < 128; i++ {
			dl, ok := layer.(*diffLayer)
			if !ok {
				break
			}
			layer = dl.flatten()
		}
		b.StopTimer()
	}
}

// This test writes ~324M of diff layers to disk, spread over
// - 128 individual layers,
// - each with 200 accounts
// - containing 200 slots
//
// BenchmarkJournal-6   	       1	1471373923 ns/ops
// BenchmarkJournal-6   	       1	1208083335 ns/op // bufio writer
func BenchmarkJournal(b *testing.B) {
	fill := func(parent snapshot) *diffLayer {
		var (
			destructs = make(map[common.Hash]struct{})
			accounts  = make(map[common.Hash][]byte)
			storage   = make(map[common.Hash]map[common.Hash][]byte)
		)
		for i := 0; i < 200; i++ {
			accountKey := randomHash()
			accounts[accountKey] = randomAccount()

			accStorage := make(map[common.Hash][]byte)
			for i := 0; i < 200; i++ {
				value := make([]byte, 32)
				crand.Read(value)
				accStorage[randomHash()] = value
			}
			storage[accountKey] = accStorage
		}
		return newDiffLayer(parent, common.Hash{}, destructs, accounts, storage)
	}
	layer := snapshot(emptyLayer())
	for i := 1; i < 128; i++ {
		layer = fill(layer)
	}
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		layer.Journal(new(bytes.Buffer))
	}
}
