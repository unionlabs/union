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
	"encoding/binary"
	"fmt"
	"math/rand"
	"testing"

	"github.com/VictoriaMetrics/fastcache"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
)

// TestAccountIteratorBasics tests some simple single-layer(diff and disk) iteration
func TestAccountIteratorBasics(t *testing.T) {
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
	diffLayer := newDiffLayer(emptyLayer(), common.Hash{}, copyDestructs(destructs), copyAccounts(accounts), copyStorage(storage))
	it := diffLayer.AccountIterator(common.Hash{})
	verifyIterator(t, 100, it, verifyNothing) // Nil is allowed for single layer iterator

	diskLayer := diffToDisk(diffLayer)
	it = diskLayer.AccountIterator(common.Hash{})
	verifyIterator(t, 100, it, verifyNothing) // Nil is allowed for single layer iterator
}

// TestStorageIteratorBasics tests some simple single-layer(diff and disk) iteration for storage
func TestStorageIteratorBasics(t *testing.T) {
	var (
		nilStorage = make(map[common.Hash]int)
		accounts   = make(map[common.Hash][]byte)
		storage    = make(map[common.Hash]map[common.Hash][]byte)
	)
	// Fill some random data
	for i := 0; i < 10; i++ {
		h := randomHash()
		accounts[h] = randomAccount()

		accStorage := make(map[common.Hash][]byte)
		value := make([]byte, 32)

		var nilstorage int
		for i := 0; i < 100; i++ {
			crand.Read(value)
			if rand.Intn(2) == 0 {
				accStorage[randomHash()] = common.CopyBytes(value)
			} else {
				accStorage[randomHash()] = nil // delete slot
				nilstorage += 1
			}
		}
		storage[h] = accStorage
		nilStorage[h] = nilstorage
	}
	// Add some (identical) layers on top
	diffLayer := newDiffLayer(emptyLayer(), common.Hash{}, nil, copyAccounts(accounts), copyStorage(storage))
	for account := range accounts {
		it, _ := diffLayer.StorageIterator(account, common.Hash{})
		verifyIterator(t, 100, it, verifyNothing) // Nil is allowed for single layer iterator
	}

	diskLayer := diffToDisk(diffLayer)
	for account := range accounts {
		it, _ := diskLayer.StorageIterator(account, common.Hash{})
		verifyIterator(t, 100-nilStorage[account], it, verifyNothing) // Nil is allowed for single layer iterator
	}
}

type testIterator struct {
	values []byte
}

func newTestIterator(values ...byte) *testIterator {
	return &testIterator{values}
}

func (ti *testIterator) Seek(common.Hash) {
	panic("implement me")
}

func (ti *testIterator) Next() bool {
	ti.values = ti.values[1:]
	return len(ti.values) > 0
}

func (ti *testIterator) Error() error {
	return nil
}

func (ti *testIterator) Hash() common.Hash {
	return common.BytesToHash([]byte{ti.values[0]})
}

func (ti *testIterator) Account() []byte {
	return nil
}

func (ti *testIterator) Slot() []byte {
	return nil
}

func (ti *testIterator) Release() {}

func TestFastIteratorBasics(t *testing.T) {
	type testCase struct {
		lists   [][]byte
		expKeys []byte
	}
	for i, tc := range []testCase{
		{lists: [][]byte{{0, 1, 8}, {1, 2, 8}, {2, 9}, {4},
			{7, 14, 15}, {9, 13, 15, 16}},
			expKeys: []byte{0, 1, 2, 4, 7, 8, 9, 13, 14, 15, 16}},
		{lists: [][]byte{{0, 8}, {1, 2, 8}, {7, 14, 15}, {8, 9},
			{9, 10}, {10, 13, 15, 16}},
			expKeys: []byte{0, 1, 2, 7, 8, 9, 10, 13, 14, 15, 16}},
	} {
		var iterators []*weightedIterator
		for i, data := range tc.lists {
			it := newTestIterator(data...)
			iterators = append(iterators, &weightedIterator{it, i})
		}
		fi := &fastIterator{
			iterators: iterators,
			initiated: false,
		}
		count := 0
		for fi.Next() {
			if got, exp := fi.Hash()[31], tc.expKeys[count]; exp != got {
				t.Errorf("tc %d, [%d]: got %d exp %d", i, count, got, exp)
			}
			count++
		}
	}
}

type verifyContent int

const (
	verifyNothing verifyContent = iota
	verifyAccount
	verifyStorage
)

func verifyIterator(t *testing.T, expCount int, it Iterator, verify verifyContent) {
	t.Helper()

	var (
		count = 0
		last  = common.Hash{}
	)
	for it.Next() {
		hash := it.Hash()
		if bytes.Compare(last[:], hash[:]) >= 0 {
			t.Errorf("wrong order: %x >= %x", last, hash)
		}
		count++
		if verify == verifyAccount && len(it.(AccountIterator).Account()) == 0 {
			t.Errorf("iterator returned nil-value for hash %x", hash)
		} else if verify == verifyStorage && len(it.(StorageIterator).Slot()) == 0 {
			t.Errorf("iterator returned nil-value for hash %x", hash)
		}
		last = hash
	}
	if count != expCount {
		t.Errorf("iterator count mismatch: have %d, want %d", count, expCount)
	}
	if err := it.Error(); err != nil {
		t.Errorf("iterator failed: %v", err)
	}
}

// TestAccountIteratorTraversal tests some simple multi-layer iteration.
func TestAccountIteratorTraversal(t *testing.T) {
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
	// Stack three diff layers on top with various overlaps
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil,
		randomAccountSet("0xaa", "0xee", "0xff", "0xf0"), nil)

	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil,
		randomAccountSet("0xbb", "0xdd", "0xf0"), nil)

	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), nil,
		randomAccountSet("0xcc", "0xf0", "0xff"), nil)

	// Verify the single and multi-layer iterators
	head := snaps.Snapshot(common.HexToHash("0x04"))

	verifyIterator(t, 3, head.(snapshot).AccountIterator(common.Hash{}), verifyNothing)
	verifyIterator(t, 7, head.(*diffLayer).newBinaryAccountIterator(), verifyAccount)

	it, _ := snaps.AccountIterator(common.HexToHash("0x04"), common.Hash{})
	verifyIterator(t, 7, it, verifyAccount)
	it.Release()

	// Test after persist some bottom-most layers into the disk,
	// the functionalities still work.
	limit := aggregatorMemoryLimit
	defer func() {
		aggregatorMemoryLimit = limit
	}()
	aggregatorMemoryLimit = 0 // Force pushing the bottom-most layer into disk
	snaps.Cap(common.HexToHash("0x04"), 2)
	verifyIterator(t, 7, head.(*diffLayer).newBinaryAccountIterator(), verifyAccount)

	it, _ = snaps.AccountIterator(common.HexToHash("0x04"), common.Hash{})
	verifyIterator(t, 7, it, verifyAccount)
	it.Release()
}

func TestStorageIteratorTraversal(t *testing.T) {
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
	// Stack three diff layers on top with various overlaps
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x01", "0x02", "0x03"}}, nil))

	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x04", "0x05", "0x06"}}, nil))

	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x01", "0x02", "0x03"}}, nil))

	// Verify the single and multi-layer iterators
	head := snaps.Snapshot(common.HexToHash("0x04"))

	diffIter, _ := head.(snapshot).StorageIterator(common.HexToHash("0xaa"), common.Hash{})
	verifyIterator(t, 3, diffIter, verifyNothing)
	verifyIterator(t, 6, head.(*diffLayer).newBinaryStorageIterator(common.HexToHash("0xaa")), verifyStorage)

	it, _ := snaps.StorageIterator(common.HexToHash("0x04"), common.HexToHash("0xaa"), common.Hash{})
	verifyIterator(t, 6, it, verifyStorage)
	it.Release()

	// Test after persist some bottom-most layers into the disk,
	// the functionalities still work.
	limit := aggregatorMemoryLimit
	defer func() {
		aggregatorMemoryLimit = limit
	}()
	aggregatorMemoryLimit = 0 // Force pushing the bottom-most layer into disk
	snaps.Cap(common.HexToHash("0x04"), 2)
	verifyIterator(t, 6, head.(*diffLayer).newBinaryStorageIterator(common.HexToHash("0xaa")), verifyStorage)

	it, _ = snaps.StorageIterator(common.HexToHash("0x04"), common.HexToHash("0xaa"), common.Hash{})
	verifyIterator(t, 6, it, verifyStorage)
	it.Release()
}

// TestAccountIteratorTraversalValues tests some multi-layer iteration, where we
// also expect the correct values to show up.
func TestAccountIteratorTraversalValues(t *testing.T) {
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
	// Create a batch of account sets to seed subsequent layers with
	var (
		a = make(map[common.Hash][]byte)
		b = make(map[common.Hash][]byte)
		c = make(map[common.Hash][]byte)
		d = make(map[common.Hash][]byte)
		e = make(map[common.Hash][]byte)
		f = make(map[common.Hash][]byte)
		g = make(map[common.Hash][]byte)
		h = make(map[common.Hash][]byte)
	)
	for i := byte(2); i < 0xff; i++ {
		a[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 0, i))
		if i > 20 && i%2 == 0 {
			b[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 1, i))
		}
		if i%4 == 0 {
			c[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 2, i))
		}
		if i%7 == 0 {
			d[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 3, i))
		}
		if i%8 == 0 {
			e[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 4, i))
		}
		if i > 50 || i < 85 {
			f[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 5, i))
		}
		if i%64 == 0 {
			g[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 6, i))
		}
		if i%128 == 0 {
			h[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 7, i))
		}
	}
	// Assemble a stack of snapshots from the account layers
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil, a, nil)
	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil, b, nil)
	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), nil, c, nil)
	snaps.Update(common.HexToHash("0x05"), common.HexToHash("0x04"), nil, d, nil)
	snaps.Update(common.HexToHash("0x06"), common.HexToHash("0x05"), nil, e, nil)
	snaps.Update(common.HexToHash("0x07"), common.HexToHash("0x06"), nil, f, nil)
	snaps.Update(common.HexToHash("0x08"), common.HexToHash("0x07"), nil, g, nil)
	snaps.Update(common.HexToHash("0x09"), common.HexToHash("0x08"), nil, h, nil)

	it, _ := snaps.AccountIterator(common.HexToHash("0x09"), common.Hash{})
	head := snaps.Snapshot(common.HexToHash("0x09"))
	for it.Next() {
		hash := it.Hash()
		want, err := head.AccountRLP(hash)
		if err != nil {
			t.Fatalf("failed to retrieve expected account: %v", err)
		}
		if have := it.Account(); !bytes.Equal(want, have) {
			t.Fatalf("hash %x: account mismatch: have %x, want %x", hash, have, want)
		}
	}
	it.Release()

	// Test after persist some bottom-most layers into the disk,
	// the functionalities still work.
	limit := aggregatorMemoryLimit
	defer func() {
		aggregatorMemoryLimit = limit
	}()
	aggregatorMemoryLimit = 0 // Force pushing the bottom-most layer into disk
	snaps.Cap(common.HexToHash("0x09"), 2)

	it, _ = snaps.AccountIterator(common.HexToHash("0x09"), common.Hash{})
	for it.Next() {
		hash := it.Hash()
		want, err := head.AccountRLP(hash)
		if err != nil {
			t.Fatalf("failed to retrieve expected account: %v", err)
		}
		if have := it.Account(); !bytes.Equal(want, have) {
			t.Fatalf("hash %x: account mismatch: have %x, want %x", hash, have, want)
		}
	}
	it.Release()
}

func TestStorageIteratorTraversalValues(t *testing.T) {
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
	wrapStorage := func(storage map[common.Hash][]byte) map[common.Hash]map[common.Hash][]byte {
		return map[common.Hash]map[common.Hash][]byte{
			common.HexToHash("0xaa"): storage,
		}
	}
	// Create a batch of storage sets to seed subsequent layers with
	var (
		a = make(map[common.Hash][]byte)
		b = make(map[common.Hash][]byte)
		c = make(map[common.Hash][]byte)
		d = make(map[common.Hash][]byte)
		e = make(map[common.Hash][]byte)
		f = make(map[common.Hash][]byte)
		g = make(map[common.Hash][]byte)
		h = make(map[common.Hash][]byte)
	)
	for i := byte(2); i < 0xff; i++ {
		a[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 0, i))
		if i > 20 && i%2 == 0 {
			b[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 1, i))
		}
		if i%4 == 0 {
			c[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 2, i))
		}
		if i%7 == 0 {
			d[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 3, i))
		}
		if i%8 == 0 {
			e[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 4, i))
		}
		if i > 50 || i < 85 {
			f[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 5, i))
		}
		if i%64 == 0 {
			g[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 6, i))
		}
		if i%128 == 0 {
			h[common.Hash{i}] = []byte(fmt.Sprintf("layer-%d, key %d", 7, i))
		}
	}
	// Assemble a stack of snapshots from the account layers
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil, randomAccountSet("0xaa"), wrapStorage(a))
	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil, randomAccountSet("0xaa"), wrapStorage(b))
	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), nil, randomAccountSet("0xaa"), wrapStorage(c))
	snaps.Update(common.HexToHash("0x05"), common.HexToHash("0x04"), nil, randomAccountSet("0xaa"), wrapStorage(d))
	snaps.Update(common.HexToHash("0x06"), common.HexToHash("0x05"), nil, randomAccountSet("0xaa"), wrapStorage(e))
	snaps.Update(common.HexToHash("0x07"), common.HexToHash("0x06"), nil, randomAccountSet("0xaa"), wrapStorage(e))
	snaps.Update(common.HexToHash("0x08"), common.HexToHash("0x07"), nil, randomAccountSet("0xaa"), wrapStorage(g))
	snaps.Update(common.HexToHash("0x09"), common.HexToHash("0x08"), nil, randomAccountSet("0xaa"), wrapStorage(h))

	it, _ := snaps.StorageIterator(common.HexToHash("0x09"), common.HexToHash("0xaa"), common.Hash{})
	head := snaps.Snapshot(common.HexToHash("0x09"))
	for it.Next() {
		hash := it.Hash()
		want, err := head.Storage(common.HexToHash("0xaa"), hash)
		if err != nil {
			t.Fatalf("failed to retrieve expected storage slot: %v", err)
		}
		if have := it.Slot(); !bytes.Equal(want, have) {
			t.Fatalf("hash %x: slot mismatch: have %x, want %x", hash, have, want)
		}
	}
	it.Release()

	// Test after persist some bottom-most layers into the disk,
	// the functionalities still work.
	limit := aggregatorMemoryLimit
	defer func() {
		aggregatorMemoryLimit = limit
	}()
	aggregatorMemoryLimit = 0 // Force pushing the bottom-most layer into disk
	snaps.Cap(common.HexToHash("0x09"), 2)

	it, _ = snaps.StorageIterator(common.HexToHash("0x09"), common.HexToHash("0xaa"), common.Hash{})
	for it.Next() {
		hash := it.Hash()
		want, err := head.Storage(common.HexToHash("0xaa"), hash)
		if err != nil {
			t.Fatalf("failed to retrieve expected slot: %v", err)
		}
		if have := it.Slot(); !bytes.Equal(want, have) {
			t.Fatalf("hash %x: slot mismatch: have %x, want %x", hash, have, want)
		}
	}
	it.Release()
}

// This testcase is notorious, all layers contain the exact same 200 accounts.
func TestAccountIteratorLargeTraversal(t *testing.T) {
	// Create a custom account factory to recreate the same addresses
	makeAccounts := func(num int) map[common.Hash][]byte {
		accounts := make(map[common.Hash][]byte)
		for i := 0; i < num; i++ {
			h := common.Hash{}
			binary.BigEndian.PutUint64(h[:], uint64(i+1))
			accounts[h] = randomAccount()
		}
		return accounts
	}
	// Build up a large stack of snapshots
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
	for i := 1; i < 128; i++ {
		snaps.Update(common.HexToHash(fmt.Sprintf("0x%02x", i+1)), common.HexToHash(fmt.Sprintf("0x%02x", i)), nil, makeAccounts(200), nil)
	}
	// Iterate the entire stack and ensure everything is hit only once
	head := snaps.Snapshot(common.HexToHash("0x80"))
	verifyIterator(t, 200, head.(snapshot).AccountIterator(common.Hash{}), verifyNothing)
	verifyIterator(t, 200, head.(*diffLayer).newBinaryAccountIterator(), verifyAccount)

	it, _ := snaps.AccountIterator(common.HexToHash("0x80"), common.Hash{})
	verifyIterator(t, 200, it, verifyAccount)
	it.Release()

	// Test after persist some bottom-most layers into the disk,
	// the functionalities still work.
	limit := aggregatorMemoryLimit
	defer func() {
		aggregatorMemoryLimit = limit
	}()
	aggregatorMemoryLimit = 0 // Force pushing the bottom-most layer into disk
	snaps.Cap(common.HexToHash("0x80"), 2)

	verifyIterator(t, 200, head.(*diffLayer).newBinaryAccountIterator(), verifyAccount)

	it, _ = snaps.AccountIterator(common.HexToHash("0x80"), common.Hash{})
	verifyIterator(t, 200, it, verifyAccount)
	it.Release()
}

// TestAccountIteratorFlattening tests what happens when we
// - have a live iterator on child C (parent C1 -> C2 .. CN)
// - flattens C2 all the way into CN
// - continues iterating
func TestAccountIteratorFlattening(t *testing.T) {
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
	// Create a stack of diffs on top
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil,
		randomAccountSet("0xaa", "0xee", "0xff", "0xf0"), nil)

	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil,
		randomAccountSet("0xbb", "0xdd", "0xf0"), nil)

	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), nil,
		randomAccountSet("0xcc", "0xf0", "0xff"), nil)

	// Create an iterator and flatten the data from underneath it
	it, _ := snaps.AccountIterator(common.HexToHash("0x04"), common.Hash{})
	defer it.Release()

	if err := snaps.Cap(common.HexToHash("0x04"), 1); err != nil {
		t.Fatalf("failed to flatten snapshot stack: %v", err)
	}
	//verifyIterator(t, 7, it)
}

func TestAccountIteratorSeek(t *testing.T) {
	// Create a snapshot stack with some initial data
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
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil,
		randomAccountSet("0xaa", "0xee", "0xff", "0xf0"), nil)

	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil,
		randomAccountSet("0xbb", "0xdd", "0xf0"), nil)

	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), nil,
		randomAccountSet("0xcc", "0xf0", "0xff"), nil)

	// Account set is now
	// 02: aa, ee, f0, ff
	// 03: aa, bb, dd, ee, f0 (, f0), ff
	// 04: aa, bb, cc, dd, ee, f0 (, f0), ff (, ff)
	// Construct various iterators and ensure their traversal is correct
	it, _ := snaps.AccountIterator(common.HexToHash("0x02"), common.HexToHash("0xdd"))
	defer it.Release()
	verifyIterator(t, 3, it, verifyAccount) // expected: ee, f0, ff

	it, _ = snaps.AccountIterator(common.HexToHash("0x02"), common.HexToHash("0xaa"))
	defer it.Release()
	verifyIterator(t, 4, it, verifyAccount) // expected: aa, ee, f0, ff

	it, _ = snaps.AccountIterator(common.HexToHash("0x02"), common.HexToHash("0xff"))
	defer it.Release()
	verifyIterator(t, 1, it, verifyAccount) // expected: ff

	it, _ = snaps.AccountIterator(common.HexToHash("0x02"), common.HexToHash("0xff1"))
	defer it.Release()
	verifyIterator(t, 0, it, verifyAccount) // expected: nothing

	it, _ = snaps.AccountIterator(common.HexToHash("0x04"), common.HexToHash("0xbb"))
	defer it.Release()
	verifyIterator(t, 6, it, verifyAccount) // expected: bb, cc, dd, ee, f0, ff

	it, _ = snaps.AccountIterator(common.HexToHash("0x04"), common.HexToHash("0xef"))
	defer it.Release()
	verifyIterator(t, 2, it, verifyAccount) // expected: f0, ff

	it, _ = snaps.AccountIterator(common.HexToHash("0x04"), common.HexToHash("0xf0"))
	defer it.Release()
	verifyIterator(t, 2, it, verifyAccount) // expected: f0, ff

	it, _ = snaps.AccountIterator(common.HexToHash("0x04"), common.HexToHash("0xff"))
	defer it.Release()
	verifyIterator(t, 1, it, verifyAccount) // expected: ff

	it, _ = snaps.AccountIterator(common.HexToHash("0x04"), common.HexToHash("0xff1"))
	defer it.Release()
	verifyIterator(t, 0, it, verifyAccount) // expected: nothing
}

func TestStorageIteratorSeek(t *testing.T) {
	// Create a snapshot stack with some initial data
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
	// Stack three diff layers on top with various overlaps
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x01", "0x03", "0x05"}}, nil))

	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x02", "0x05", "0x06"}}, nil))

	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x01", "0x05", "0x08"}}, nil))

	// Account set is now
	// 02: 01, 03, 05
	// 03: 01, 02, 03, 05 (, 05), 06
	// 04: 01(, 01), 02, 03, 05(, 05, 05), 06, 08
	// Construct various iterators and ensure their traversal is correct
	it, _ := snaps.StorageIterator(common.HexToHash("0x02"), common.HexToHash("0xaa"), common.HexToHash("0x01"))
	defer it.Release()
	verifyIterator(t, 3, it, verifyStorage) // expected: 01, 03, 05

	it, _ = snaps.StorageIterator(common.HexToHash("0x02"), common.HexToHash("0xaa"), common.HexToHash("0x02"))
	defer it.Release()
	verifyIterator(t, 2, it, verifyStorage) // expected: 03, 05

	it, _ = snaps.StorageIterator(common.HexToHash("0x02"), common.HexToHash("0xaa"), common.HexToHash("0x5"))
	defer it.Release()
	verifyIterator(t, 1, it, verifyStorage) // expected: 05

	it, _ = snaps.StorageIterator(common.HexToHash("0x02"), common.HexToHash("0xaa"), common.HexToHash("0x6"))
	defer it.Release()
	verifyIterator(t, 0, it, verifyStorage) // expected: nothing

	it, _ = snaps.StorageIterator(common.HexToHash("0x04"), common.HexToHash("0xaa"), common.HexToHash("0x01"))
	defer it.Release()
	verifyIterator(t, 6, it, verifyStorage) // expected: 01, 02, 03, 05, 06, 08

	it, _ = snaps.StorageIterator(common.HexToHash("0x04"), common.HexToHash("0xaa"), common.HexToHash("0x05"))
	defer it.Release()
	verifyIterator(t, 3, it, verifyStorage) // expected: 05, 06, 08

	it, _ = snaps.StorageIterator(common.HexToHash("0x04"), common.HexToHash("0xaa"), common.HexToHash("0x08"))
	defer it.Release()
	verifyIterator(t, 1, it, verifyStorage) // expected: 08

	it, _ = snaps.StorageIterator(common.HexToHash("0x04"), common.HexToHash("0xaa"), common.HexToHash("0x09"))
	defer it.Release()
	verifyIterator(t, 0, it, verifyStorage) // expected: nothing
}

// TestAccountIteratorDeletions tests that the iterator behaves correct when there are
// deleted accounts (where the Account() value is nil). The iterator
// should not output any accounts or nil-values for those cases.
func TestAccountIteratorDeletions(t *testing.T) {
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
	// Stack three diff layers on top with various overlaps
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"),
		nil, randomAccountSet("0x11", "0x22", "0x33"), nil)

	deleted := common.HexToHash("0x22")
	destructed := map[common.Hash]struct{}{
		deleted: {},
	}
	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"),
		destructed, randomAccountSet("0x11", "0x33"), nil)

	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"),
		nil, randomAccountSet("0x33", "0x44", "0x55"), nil)

	// The output should be 11,33,44,55
	it, _ := snaps.AccountIterator(common.HexToHash("0x04"), common.Hash{})
	// Do a quick check
	verifyIterator(t, 4, it, verifyAccount)
	it.Release()

	// And a more detailed verification that we indeed do not see '0x22'
	it, _ = snaps.AccountIterator(common.HexToHash("0x04"), common.Hash{})
	defer it.Release()
	for it.Next() {
		hash := it.Hash()
		if it.Account() == nil {
			t.Errorf("iterator returned nil-value for hash %x", hash)
		}
		if hash == deleted {
			t.Errorf("expected deleted elem %x to not be returned by iterator", deleted)
		}
	}
}

func TestStorageIteratorDeletions(t *testing.T) {
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
	// Stack three diff layers on top with various overlaps
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x01", "0x03", "0x05"}}, nil))

	snaps.Update(common.HexToHash("0x03"), common.HexToHash("0x02"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x02", "0x04", "0x06"}}, [][]string{{"0x01", "0x03"}}))

	// The output should be 02,04,05,06
	it, _ := snaps.StorageIterator(common.HexToHash("0x03"), common.HexToHash("0xaa"), common.Hash{})
	verifyIterator(t, 4, it, verifyStorage)
	it.Release()

	// The output should be 04,05,06
	it, _ = snaps.StorageIterator(common.HexToHash("0x03"), common.HexToHash("0xaa"), common.HexToHash("0x03"))
	verifyIterator(t, 3, it, verifyStorage)
	it.Release()

	// Destruct the whole storage
	destructed := map[common.Hash]struct{}{
		common.HexToHash("0xaa"): {},
	}
	snaps.Update(common.HexToHash("0x04"), common.HexToHash("0x03"), destructed, nil, nil)

	it, _ = snaps.StorageIterator(common.HexToHash("0x04"), common.HexToHash("0xaa"), common.Hash{})
	verifyIterator(t, 0, it, verifyStorage)
	it.Release()

	// Re-insert the slots of the same account
	snaps.Update(common.HexToHash("0x05"), common.HexToHash("0x04"), nil,
		randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x07", "0x08", "0x09"}}, nil))

	// The output should be 07,08,09
	it, _ = snaps.StorageIterator(common.HexToHash("0x05"), common.HexToHash("0xaa"), common.Hash{})
	verifyIterator(t, 3, it, verifyStorage)
	it.Release()

	// Destruct the whole storage but re-create the account in the same layer
	snaps.Update(common.HexToHash("0x06"), common.HexToHash("0x05"), destructed, randomAccountSet("0xaa"), randomStorageSet([]string{"0xaa"}, [][]string{{"0x11", "0x12"}}, nil))
	it, _ = snaps.StorageIterator(common.HexToHash("0x06"), common.HexToHash("0xaa"), common.Hash{})
	verifyIterator(t, 2, it, verifyStorage) // The output should be 11,12
	it.Release()

	verifyIterator(t, 2, snaps.Snapshot(common.HexToHash("0x06")).(*diffLayer).newBinaryStorageIterator(common.HexToHash("0xaa")), verifyStorage)
}

// BenchmarkAccountIteratorTraversal is a bit a bit notorious -- all layers contain the
// exact same 200 accounts. That means that we need to process 2000 items, but
// only spit out 200 values eventually.
//
// The value-fetching benchmark is easy on the binary iterator, since it never has to reach
// down at any depth for retrieving the values -- all are on the topmost layer
//
// BenchmarkAccountIteratorTraversal/binary_iterator_keys-6         	    2239	    483674 ns/op
// BenchmarkAccountIteratorTraversal/binary_iterator_values-6       	    2403	    501810 ns/op
// BenchmarkAccountIteratorTraversal/fast_iterator_keys-6           	    1923	    677966 ns/op
// BenchmarkAccountIteratorTraversal/fast_iterator_values-6         	    1741	    649967 ns/op
func BenchmarkAccountIteratorTraversal(b *testing.B) {
	// Create a custom account factory to recreate the same addresses
	makeAccounts := func(num int) map[common.Hash][]byte {
		accounts := make(map[common.Hash][]byte)
		for i := 0; i < num; i++ {
			h := common.Hash{}
			binary.BigEndian.PutUint64(h[:], uint64(i+1))
			accounts[h] = randomAccount()
		}
		return accounts
	}
	// Build up a large stack of snapshots
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
	for i := 1; i <= 100; i++ {
		snaps.Update(common.HexToHash(fmt.Sprintf("0x%02x", i+1)), common.HexToHash(fmt.Sprintf("0x%02x", i)), nil, makeAccounts(200), nil)
	}
	// We call this once before the benchmark, so the creation of
	// sorted accountlists are not included in the results.
	head := snaps.Snapshot(common.HexToHash("0x65"))
	head.(*diffLayer).newBinaryAccountIterator()

	b.Run("binary iterator keys", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			got := 0
			it := head.(*diffLayer).newBinaryAccountIterator()
			for it.Next() {
				got++
			}
			if exp := 200; got != exp {
				b.Errorf("iterator len wrong, expected %d, got %d", exp, got)
			}
		}
	})
	b.Run("binary iterator values", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			got := 0
			it := head.(*diffLayer).newBinaryAccountIterator()
			for it.Next() {
				got++
				head.(*diffLayer).accountRLP(it.Hash(), 0)
			}
			if exp := 200; got != exp {
				b.Errorf("iterator len wrong, expected %d, got %d", exp, got)
			}
		}
	})
	b.Run("fast iterator keys", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			it, _ := snaps.AccountIterator(common.HexToHash("0x65"), common.Hash{})
			defer it.Release()

			got := 0
			for it.Next() {
				got++
			}
			if exp := 200; got != exp {
				b.Errorf("iterator len wrong, expected %d, got %d", exp, got)
			}
		}
	})
	b.Run("fast iterator values", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			it, _ := snaps.AccountIterator(common.HexToHash("0x65"), common.Hash{})
			defer it.Release()

			got := 0
			for it.Next() {
				got++
				it.Account()
			}
			if exp := 200; got != exp {
				b.Errorf("iterator len wrong, expected %d, got %d", exp, got)
			}
		}
	})
}

// BenchmarkAccountIteratorLargeBaselayer is a pretty realistic benchmark, where
// the baselayer is a lot larger than the upper layer.
//
// This is heavy on the binary iterator, which in most cases will have to
// call recursively 100 times for the majority of the values
//
// BenchmarkAccountIteratorLargeBaselayer/binary_iterator_(keys)-6         	     514	   1971999 ns/op
// BenchmarkAccountIteratorLargeBaselayer/binary_iterator_(values)-6       	      61	  18997492 ns/op
// BenchmarkAccountIteratorLargeBaselayer/fast_iterator_(keys)-6           	   10000	    114385 ns/op
// BenchmarkAccountIteratorLargeBaselayer/fast_iterator_(values)-6         	    4047	    296823 ns/op
func BenchmarkAccountIteratorLargeBaselayer(b *testing.B) {
	// Create a custom account factory to recreate the same addresses
	makeAccounts := func(num int) map[common.Hash][]byte {
		accounts := make(map[common.Hash][]byte)
		for i := 0; i < num; i++ {
			h := common.Hash{}
			binary.BigEndian.PutUint64(h[:], uint64(i+1))
			accounts[h] = randomAccount()
		}
		return accounts
	}
	// Build up a large stack of snapshots
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
	snaps.Update(common.HexToHash("0x02"), common.HexToHash("0x01"), nil, makeAccounts(2000), nil)
	for i := 2; i <= 100; i++ {
		snaps.Update(common.HexToHash(fmt.Sprintf("0x%02x", i+1)), common.HexToHash(fmt.Sprintf("0x%02x", i)), nil, makeAccounts(20), nil)
	}
	// We call this once before the benchmark, so the creation of
	// sorted accountlists are not included in the results.
	head := snaps.Snapshot(common.HexToHash("0x65"))
	head.(*diffLayer).newBinaryAccountIterator()

	b.Run("binary iterator (keys)", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			got := 0
			it := head.(*diffLayer).newBinaryAccountIterator()
			for it.Next() {
				got++
			}
			if exp := 2000; got != exp {
				b.Errorf("iterator len wrong, expected %d, got %d", exp, got)
			}
		}
	})
	b.Run("binary iterator (values)", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			got := 0
			it := head.(*diffLayer).newBinaryAccountIterator()
			for it.Next() {
				got++
				v := it.Hash()
				head.(*diffLayer).accountRLP(v, 0)
			}
			if exp := 2000; got != exp {
				b.Errorf("iterator len wrong, expected %d, got %d", exp, got)
			}
		}
	})
	b.Run("fast iterator (keys)", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			it, _ := snaps.AccountIterator(common.HexToHash("0x65"), common.Hash{})
			defer it.Release()

			got := 0
			for it.Next() {
				got++
			}
			if exp := 2000; got != exp {
				b.Errorf("iterator len wrong, expected %d, got %d", exp, got)
			}
		}
	})
	b.Run("fast iterator (values)", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			it, _ := snaps.AccountIterator(common.HexToHash("0x65"), common.Hash{})
			defer it.Release()

			got := 0
			for it.Next() {
				it.Account()
				got++
			}
			if exp := 2000; got != exp {
				b.Errorf("iterator len wrong, expected %d, got %d", exp, got)
			}
		}
	})
}

/*
func BenchmarkBinaryAccountIteration(b *testing.B) {
	benchmarkAccountIteration(b, func(snap snapshot) AccountIterator {
		return snap.(*diffLayer).newBinaryAccountIterator()
	})
}

func BenchmarkFastAccountIteration(b *testing.B) {
	benchmarkAccountIteration(b, newFastAccountIterator)
}

func benchmarkAccountIteration(b *testing.B, iterator func(snap snapshot) AccountIterator) {
	// Create a diff stack and randomize the accounts across them
	layers := make([]map[common.Hash][]byte, 128)
	for i := 0; i < len(layers); i++ {
		layers[i] = make(map[common.Hash][]byte)
	}
	for i := 0; i < b.N; i++ {
		depth := rand.Intn(len(layers))
		layers[depth][randomHash()] = randomAccount()
	}
	stack := snapshot(emptyLayer())
	for _, layer := range layers {
		stack = stack.Update(common.Hash{}, layer, nil, nil)
	}
	// Reset the timers and report all the stats
	it := iterator(stack)

	b.ResetTimer()
	b.ReportAllocs()

	for it.Next() {
	}
}
*/
