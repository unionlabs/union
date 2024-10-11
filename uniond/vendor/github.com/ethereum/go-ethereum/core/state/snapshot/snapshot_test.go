// Copyright 2017 The go-ethereum Authors
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
	crand "crypto/rand"
	"encoding/binary"
	"fmt"
	"math/big"
	"math/rand"
	"testing"
	"time"

	"github.com/VictoriaMetrics/fastcache"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
)

// randomHash generates a random blob of data and returns it as a hash.
func randomHash() common.Hash {
	var hash common.Hash
	if n, err := crand.Read(hash[:]); n != common.HashLength || err != nil {
		panic(err)
	}
	return hash
}

// randomAccount generates a random account and returns it RLP encoded.
func randomAccount() []byte {
	a := &types.StateAccount{
		Balance:  big.NewInt(rand.Int63()),
		Nonce:    rand.Uint64(),
		Root:     randomHash(),
		CodeHash: types.EmptyCodeHash[:],
	}
	data, _ := rlp.EncodeToBytes(a)
	return data
}

// randomAccountSet generates a set of random accounts with the given strings as
// the account address hashes.
func randomAccountSet(hashes ...string) map[common.Hash][]byte {
	accounts := make(map[common.Hash][]byte)
	for _, hash := range hashes {
		accounts[common.HexToHash(hash)] = randomAccount()
	}
	return accounts
}

// randomStorageSet generates a set of random slots with the given strings as
// the slot addresses.
func randomStorageSet(accounts []string, hashes [][]string, nilStorage [][]string) map[common.Hash]map[common.Hash][]byte {
	storages := make(map[common.Hash]map[common.Hash][]byte)
	for index, account := range accounts {
		storages[common.HexToHash(account)] = make(map[common.Hash][]byte)

		if index < len(hashes) {
			hashes := hashes[index]
			for _, hash := range hashes {
				storages[common.HexToHash(account)][common.HexToHash(hash)] = randomHash().Bytes()
			}
		}
		if index < len(nilStorage) {
			nils := nilStorage[index]
			for _, hash := range nils {
				storages[common.HexToHash(account)][common.HexToHash(hash)] = nil
			}
		}
	}
	return storages
}

// Tests that if a disk layer becomes stale, no active external references will
// be returned with junk data. This version of the test flattens every diff layer
// to check internal corner case around the bottom-most memory accumulator.
func TestDiskLayerExternalInvalidationFullFlatten(t *testing.T) {
	// Create an empty base layer and a snapshot tree out of it
	base := &diskLayer{
		diskdb: rawdb.NewMemoryDatabase(),
		root:   common.HexToHash("0x01"),
		cache:  fastcache.New(1024 * 500),
	}
	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			base.root: base,
		},
	}
	// Retrieve a reference to the base and commit a diff on top
	ref := snaps.Snapshot(base.root)

	accounts := map[common.Hash][]byte{
		common.HexToHash("0xa1"): randomAccount(),
	}
	if err := snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil, accounts, nil); err != nil {
		t.Fatalf("failed to create a diff layer: %v", err)
	}
	if n := len(snaps.layers); n != 2 {
		t.Errorf("pre-cap layer count mismatch: have %d, want %d", n, 2)
	}
	// Commit the diff layer onto the disk and ensure it's persisted
	if err := snaps.Cap(common.HexToHash("0x02"), 0); err != nil {
		t.Fatalf("failed to merge diff layer onto disk: %v", err)
	}
	// Since the base layer was modified, ensure that data retrievals on the external reference fail
	if acc, err := ref.Account(common.HexToHash("0x01")); err != ErrSnapshotStale {
		t.Errorf("stale reference returned account: %#x (err: %v)", acc, err)
	}
	if slot, err := ref.Storage(common.HexToHash("0xa1"), common.HexToHash("0xb1")); err != ErrSnapshotStale {
		t.Errorf("stale reference returned storage slot: %#x (err: %v)", slot, err)
	}
	if n := len(snaps.layers); n != 1 {
		t.Errorf("post-cap layer count mismatch: have %d, want %d", n, 1)
		fmt.Println(snaps.layers)
	}
}

// Tests that if a disk layer becomes stale, no active external references will
// be returned with junk data. This version of the test retains the bottom diff
// layer to check the usual mode of operation where the accumulator is retained.
func TestDiskLayerExternalInvalidationPartialFlatten(t *testing.T) {
	// Create an empty base layer and a snapshot tree out of it
	base := &diskLayer{
		diskdb: rawdb.NewMemoryDatabase(),
		root:   common.HexToHash("0x01"),
		cache:  fastcache.New(1024 * 500),
	}
	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			base.root: base,
		},
	}
	// Retrieve a reference to the base and commit two diffs on top
	ref := snaps.Snapshot(base.root)

	accounts := map[common.Hash][]byte{
		common.HexToHash("0xa1"): randomAccount(),
	}
	if err := snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil, accounts, nil); err != nil {
		t.Fatalf("failed to create a diff layer: %v", err)
	}
	if err := snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil, accounts, nil); err != nil {
		t.Fatalf("failed to create a diff layer: %v", err)
	}
	if n := len(snaps.layers); n != 3 {
		t.Errorf("pre-cap layer count mismatch: have %d, want %d", n, 3)
	}
	// Commit the diff layer onto the disk and ensure it's persisted
	defer func(memcap uint64) { aggregatorMemoryLimit = memcap }(aggregatorMemoryLimit)
	aggregatorMemoryLimit = 0

	if err := snaps.Cap(common.HexToHash("0x03"), 1); err != nil {
		t.Fatalf("failed to merge accumulator onto disk: %v", err)
	}
	// Since the base layer was modified, ensure that data retrievals on the external reference fail
	if acc, err := ref.Account(common.HexToHash("0x01")); err != ErrSnapshotStale {
		t.Errorf("stale reference returned account: %#x (err: %v)", acc, err)
	}
	if slot, err := ref.Storage(common.HexToHash("0xa1"), common.HexToHash("0xb1")); err != ErrSnapshotStale {
		t.Errorf("stale reference returned storage slot: %#x (err: %v)", slot, err)
	}
	if n := len(snaps.layers); n != 2 {
		t.Errorf("post-cap layer count mismatch: have %d, want %d", n, 2)
		fmt.Println(snaps.layers)
	}
}

// Tests that if a diff layer becomes stale, no active external references will
// be returned with junk data. This version of the test retains the bottom diff
// layer to check the usual mode of operation where the accumulator is retained.
func TestDiffLayerExternalInvalidationPartialFlatten(t *testing.T) {
	// Un-commenting this triggers the bloom set to be deterministic. The values below
	// were used to trigger the flaw described in https://github.com/ethereum/go-ethereum/issues/27254.
	// bloomDestructHasherOffset, bloomAccountHasherOffset, bloomStorageHasherOffset = 14, 24, 5

	// Create an empty base layer and a snapshot tree out of it
	base := &diskLayer{
		diskdb: rawdb.NewMemoryDatabase(),
		root:   common.HexToHash("0x01"),
		cache:  fastcache.New(1024 * 500),
	}
	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			base.root: base,
		},
	}
	// Commit three diffs on top and retrieve a reference to the bottommost
	accounts := map[common.Hash][]byte{
		common.HexToHash("0xa1"): randomAccount(),
	}
	if err := snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil, accounts, nil); err != nil {
		t.Fatalf("failed to create a diff layer: %v", err)
	}
	if err := snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil, accounts, nil); err != nil {
		t.Fatalf("failed to create a diff layer: %v", err)
	}
	if err := snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), nil, accounts, nil); err != nil {
		t.Fatalf("failed to create a diff layer: %v", err)
	}
	if n := len(snaps.layers); n != 4 {
		t.Errorf("pre-cap layer count mismatch: have %d, want %d", n, 4)
	}
	ref := snaps.Snapshot(common.HexToHash("0x02"))

	// Doing a Cap operation with many allowed layers should be a no-op
	exp := len(snaps.layers)
	if err := snaps.Cap(common.HexToHash("0x04"), 2000); err != nil {
		t.Fatalf("failed to flatten diff layer into accumulator: %v", err)
	}
	if got := len(snaps.layers); got != exp {
		t.Errorf("layers modified, got %d exp %d", got, exp)
	}
	// Flatten the diff layer into the bottom accumulator
	if err := snaps.Cap(common.HexToHash("0x04"), 1); err != nil {
		t.Fatalf("failed to flatten diff layer into accumulator: %v", err)
	}
	// Since the accumulator diff layer was modified, ensure that data retrievals on the external reference fail
	if acc, err := ref.Account(common.HexToHash("0x01")); err != ErrSnapshotStale {
		t.Errorf("stale reference returned account: %#x (err: %v)", acc, err)
	}
	if slot, err := ref.Storage(common.HexToHash("0xa1"), common.HexToHash("0xb1")); err != ErrSnapshotStale {
		t.Errorf("stale reference returned storage slot: %#x (err: %v)", slot, err)
	}
	if n := len(snaps.layers); n != 3 {
		t.Errorf("post-cap layer count mismatch: have %d, want %d", n, 3)
		fmt.Println(snaps.layers)
	}
}

// TestPostCapBasicDataAccess tests some functionality regarding capping/flattening.
func TestPostCapBasicDataAccess(t *testing.T) {
	// setAccount is a helper to construct a random account entry and assign it to
	// an account slot in a snapshot
	setAccount := func(accKey string) map[common.Hash][]byte {
		return map[common.Hash][]byte{
			common.HexToHash(accKey): randomAccount(),
		}
	}
	// Create a starting base layer and a snapshot tree out of it
	base := &diskLayer{
		diskdb: rawdb.NewMemoryDatabase(),
		root:   common.HexToHash("0x01"),
		cache:  fastcache.New(1024 * 500),
	}
	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			base.root: base,
		},
	}
	// The lowest difflayer
	snaps.Update(common.HexToHash("0xa1"), common.HexToHash("0x01"), nil, setAccount("0xa1"), nil)
	snaps.Update(common.HexToHash("0xa2"), common.HexToHash("0xa1"), nil, setAccount("0xa2"), nil)
	snaps.Update(common.HexToHash("0xb2"), common.HexToHash("0xa1"), nil, setAccount("0xb2"), nil)

	snaps.Update(common.HexToHash("0xa3"), common.HexToHash("0xa2"), nil, setAccount("0xa3"), nil)
	snaps.Update(common.HexToHash("0xb3"), common.HexToHash("0xb2"), nil, setAccount("0xb3"), nil)

	// checkExist verifies if an account exists in a snapshot
	checkExist := func(layer *diffLayer, key string) error {
		if data, _ := layer.Account(common.HexToHash(key)); data == nil {
			return fmt.Errorf("expected %x to exist, got nil", common.HexToHash(key))
		}
		return nil
	}
	// shouldErr checks that an account access errors as expected
	shouldErr := func(layer *diffLayer, key string) error {
		if data, err := layer.Account(common.HexToHash(key)); err == nil {
			return fmt.Errorf("expected error, got data %x", data)
		}
		return nil
	}
	// check basics
	snap := snaps.Snapshot(common.HexToHash("0xb3")).(*diffLayer)

	if err := checkExist(snap, "0xa1"); err != nil {
		t.Error(err)
	}
	if err := checkExist(snap, "0xb2"); err != nil {
		t.Error(err)
	}
	if err := checkExist(snap, "0xb3"); err != nil {
		t.Error(err)
	}
	// Cap to a bad root should fail
	if err := snaps.Cap(common.HexToHash("0x1337"), 0); err == nil {
		t.Errorf("expected error, got none")
	}
	// Now, merge the a-chain
	snaps.Cap(common.HexToHash("0xa3"), 0)

	// At this point, a2 got merged into a1. Thus, a1 is now modified, and as a1 is
	// the parent of b2, b2 should no longer be able to iterate into parent.

	// These should still be accessible
	if err := checkExist(snap, "0xb2"); err != nil {
		t.Error(err)
	}
	if err := checkExist(snap, "0xb3"); err != nil {
		t.Error(err)
	}
	// But these would need iteration into the modified parent
	if err := shouldErr(snap, "0xa1"); err != nil {
		t.Error(err)
	}
	if err := shouldErr(snap, "0xa2"); err != nil {
		t.Error(err)
	}
	if err := shouldErr(snap, "0xa3"); err != nil {
		t.Error(err)
	}
	// Now, merge it again, just for fun. It should now error, since a3
	// is a disk layer
	if err := snaps.Cap(common.HexToHash("0xa3"), 0); err == nil {
		t.Error("expected error capping the disk layer, got none")
	}
}

// TestSnaphots tests the functionality for retrieving the snapshot
// with given head root and the desired depth.
func TestSnaphots(t *testing.T) {
	// setAccount is a helper to construct a random account entry and assign it to
	// an account slot in a snapshot
	setAccount := func(accKey string) map[common.Hash][]byte {
		return map[common.Hash][]byte{
			common.HexToHash(accKey): randomAccount(),
		}
	}
	makeRoot := func(height uint64) common.Hash {
		var buffer [8]byte
		binary.BigEndian.PutUint64(buffer[:], height)
		return common.BytesToHash(buffer[:])
	}
	// Create a starting base layer and a snapshot tree out of it
	base := &diskLayer{
		diskdb: rawdb.NewMemoryDatabase(),
		root:   makeRoot(1),
		cache:  fastcache.New(1024 * 500),
	}
	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			base.root: base,
		},
	}
	// Construct the snapshots with 129 layers, flattening whatever's above that
	var (
		last = common.HexToHash("0x01")
		head common.Hash
	)
	for i := 0; i < 129; i++ {
		head = makeRoot(uint64(i + 2))
		snaps.Update(head, last, nil, setAccount(fmt.Sprintf("%d", i+2)), nil)
		last = head
		snaps.Cap(head, 128) // 130 layers (128 diffs + 1 accumulator + 1 disk)
	}
	var cases = []struct {
		headRoot     common.Hash
		limit        int
		nodisk       bool
		expected     int
		expectBottom common.Hash
	}{
		{head, 0, false, 0, common.Hash{}},
		{head, 64, false, 64, makeRoot(129 + 2 - 64)},
		{head, 128, false, 128, makeRoot(3)}, // Normal diff layers, no accumulator
		{head, 129, true, 129, makeRoot(2)},  // All diff layers, including accumulator
		{head, 130, false, 130, makeRoot(1)}, // All diff layers + disk layer
	}
	for i, c := range cases {
		layers := snaps.Snapshots(c.headRoot, c.limit, c.nodisk)
		if len(layers) != c.expected {
			t.Errorf("non-overflow test %d: returned snapshot layers are mismatched, want %v, got %v", i, c.expected, len(layers))
		}
		if len(layers) == 0 {
			continue
		}
		bottommost := layers[len(layers)-1]
		if bottommost.Root() != c.expectBottom {
			t.Errorf("non-overflow test %d: snapshot mismatch, want %v, get %v", i, c.expectBottom, bottommost.Root())
		}
	}
	// Above we've tested the normal capping, which leaves the accumulator live.
	// Test that if the bottommost accumulator diff layer overflows the allowed
	// memory limit, the snapshot tree gets capped to one less layer.
	// Commit the diff layer onto the disk and ensure it's persisted
	defer func(memcap uint64) { aggregatorMemoryLimit = memcap }(aggregatorMemoryLimit)
	aggregatorMemoryLimit = 0

	snaps.Cap(head, 128) // 129 (128 diffs + 1 overflown accumulator + 1 disk)

	cases = []struct {
		headRoot     common.Hash
		limit        int
		nodisk       bool
		expected     int
		expectBottom common.Hash
	}{
		{head, 0, false, 0, common.Hash{}},
		{head, 64, false, 64, makeRoot(129 + 2 - 64)},
		{head, 128, false, 128, makeRoot(3)}, // All diff layers, accumulator was flattened
		{head, 129, true, 128, makeRoot(3)},  // All diff layers, accumulator was flattened
		{head, 130, false, 129, makeRoot(2)}, // All diff layers + disk layer
	}
	for i, c := range cases {
		layers := snaps.Snapshots(c.headRoot, c.limit, c.nodisk)
		if len(layers) != c.expected {
			t.Errorf("overflow test %d: returned snapshot layers are mismatched, want %v, got %v", i, c.expected, len(layers))
		}
		if len(layers) == 0 {
			continue
		}
		bottommost := layers[len(layers)-1]
		if bottommost.Root() != c.expectBottom {
			t.Errorf("overflow test %d: snapshot mismatch, want %v, get %v", i, c.expectBottom, bottommost.Root())
		}
	}
}

// TestReadStateDuringFlattening tests the scenario that, during the
// bottom diff layers are merging which tags these as stale, the read
// happens via a pre-created top snapshot layer which tries to access
// the state in these stale layers. Ensure this read can retrieve the
// right state back(block until the flattening is finished) instead of
// an unexpected error(snapshot layer is stale).
func TestReadStateDuringFlattening(t *testing.T) {
	// setAccount is a helper to construct a random account entry and assign it to
	// an account slot in a snapshot
	setAccount := func(accKey string) map[common.Hash][]byte {
		return map[common.Hash][]byte{
			common.HexToHash(accKey): randomAccount(),
		}
	}
	// Create a starting base layer and a snapshot tree out of it
	base := &diskLayer{
		diskdb: rawdb.NewMemoryDatabase(),
		root:   common.HexToHash("0x01"),
		cache:  fastcache.New(1024 * 500),
	}
	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			base.root: base,
		},
	}
	// 4 layers in total, 3 diff layers and 1 disk layers
	snaps.Update(common.HexToHash("0xa1"), common.HexToHash("0x01"), nil, setAccount("0xa1"), nil)
	snaps.Update(common.HexToHash("0xa2"), common.HexToHash("0xa1"), nil, setAccount("0xa2"), nil)
	snaps.Update(common.HexToHash("0xa3"), common.HexToHash("0xa2"), nil, setAccount("0xa3"), nil)

	// Obtain the topmost snapshot handler for state accessing
	snap := snaps.Snapshot(common.HexToHash("0xa3"))

	// Register the testing hook to access the state after flattening
	var result = make(chan *types.SlimAccount)
	snaps.onFlatten = func() {
		// Spin up a thread to read the account from the pre-created
		// snapshot handler. It's expected to be blocked.
		go func() {
			account, _ := snap.Account(common.HexToHash("0xa1"))
			result <- account
		}()
		select {
		case res := <-result:
			t.Fatalf("Unexpected return %v", res)
		case <-time.NewTimer(time.Millisecond * 300).C:
		}
	}
	// Cap the snap tree, which will mark the bottom-most layer as stale.
	snaps.Cap(common.HexToHash("0xa3"), 1)
	select {
	case account := <-result:
		if account == nil {
			t.Fatal("Failed to retrieve account")
		}
	case <-time.NewTimer(time.Millisecond * 300).C:
		t.Fatal("Unexpected blocker")
	}
}
