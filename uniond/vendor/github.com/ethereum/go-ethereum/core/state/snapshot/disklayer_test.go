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
	"testing"

	"github.com/VictoriaMetrics/fastcache"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/ethdb/memorydb"
	"github.com/ethereum/go-ethereum/rlp"
)

// reverse reverses the contents of a byte slice. It's used to update random accs
// with deterministic changes.
func reverse(blob []byte) []byte {
	res := make([]byte, len(blob))
	for i, b := range blob {
		res[len(blob)-1-i] = b
	}
	return res
}

// Tests that merging something into a disk layer persists it into the database
// and invalidates any previously written and cached values.
func TestDiskMerge(t *testing.T) {
	// Create some accounts in the disk layer
	db := memorydb.New()

	var (
		accNoModNoCache     = common.Hash{0x1}
		accNoModCache       = common.Hash{0x2}
		accModNoCache       = common.Hash{0x3}
		accModCache         = common.Hash{0x4}
		accDelNoCache       = common.Hash{0x5}
		accDelCache         = common.Hash{0x6}
		conNoModNoCache     = common.Hash{0x7}
		conNoModNoCacheSlot = common.Hash{0x70}
		conNoModCache       = common.Hash{0x8}
		conNoModCacheSlot   = common.Hash{0x80}
		conModNoCache       = common.Hash{0x9}
		conModNoCacheSlot   = common.Hash{0x90}
		conModCache         = common.Hash{0xa}
		conModCacheSlot     = common.Hash{0xa0}
		conDelNoCache       = common.Hash{0xb}
		conDelNoCacheSlot   = common.Hash{0xb0}
		conDelCache         = common.Hash{0xc}
		conDelCacheSlot     = common.Hash{0xc0}
		conNukeNoCache      = common.Hash{0xd}
		conNukeNoCacheSlot  = common.Hash{0xd0}
		conNukeCache        = common.Hash{0xe}
		conNukeCacheSlot    = common.Hash{0xe0}
		baseRoot            = randomHash()
		diffRoot            = randomHash()
	)

	rawdb.WriteAccountSnapshot(db, accNoModNoCache, accNoModNoCache[:])
	rawdb.WriteAccountSnapshot(db, accNoModCache, accNoModCache[:])
	rawdb.WriteAccountSnapshot(db, accModNoCache, accModNoCache[:])
	rawdb.WriteAccountSnapshot(db, accModCache, accModCache[:])
	rawdb.WriteAccountSnapshot(db, accDelNoCache, accDelNoCache[:])
	rawdb.WriteAccountSnapshot(db, accDelCache, accDelCache[:])

	rawdb.WriteAccountSnapshot(db, conNoModNoCache, conNoModNoCache[:])
	rawdb.WriteStorageSnapshot(db, conNoModNoCache, conNoModNoCacheSlot, conNoModNoCacheSlot[:])
	rawdb.WriteAccountSnapshot(db, conNoModCache, conNoModCache[:])
	rawdb.WriteStorageSnapshot(db, conNoModCache, conNoModCacheSlot, conNoModCacheSlot[:])
	rawdb.WriteAccountSnapshot(db, conModNoCache, conModNoCache[:])
	rawdb.WriteStorageSnapshot(db, conModNoCache, conModNoCacheSlot, conModNoCacheSlot[:])
	rawdb.WriteAccountSnapshot(db, conModCache, conModCache[:])
	rawdb.WriteStorageSnapshot(db, conModCache, conModCacheSlot, conModCacheSlot[:])
	rawdb.WriteAccountSnapshot(db, conDelNoCache, conDelNoCache[:])
	rawdb.WriteStorageSnapshot(db, conDelNoCache, conDelNoCacheSlot, conDelNoCacheSlot[:])
	rawdb.WriteAccountSnapshot(db, conDelCache, conDelCache[:])
	rawdb.WriteStorageSnapshot(db, conDelCache, conDelCacheSlot, conDelCacheSlot[:])

	rawdb.WriteAccountSnapshot(db, conNukeNoCache, conNukeNoCache[:])
	rawdb.WriteStorageSnapshot(db, conNukeNoCache, conNukeNoCacheSlot, conNukeNoCacheSlot[:])
	rawdb.WriteAccountSnapshot(db, conNukeCache, conNukeCache[:])
	rawdb.WriteStorageSnapshot(db, conNukeCache, conNukeCacheSlot, conNukeCacheSlot[:])

	rawdb.WriteSnapshotRoot(db, baseRoot)

	// Create a disk layer based on the above and cache in some data
	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			baseRoot: &diskLayer{
				diskdb: db,
				cache:  fastcache.New(500 * 1024),
				root:   baseRoot,
			},
		},
	}
	base := snaps.Snapshot(baseRoot)
	base.AccountRLP(accNoModCache)
	base.AccountRLP(accModCache)
	base.AccountRLP(accDelCache)
	base.Storage(conNoModCache, conNoModCacheSlot)
	base.Storage(conModCache, conModCacheSlot)
	base.Storage(conDelCache, conDelCacheSlot)
	base.Storage(conNukeCache, conNukeCacheSlot)

	// Modify or delete some accounts, flatten everything onto disk
	if err := snaps.Update(diffRoot, baseRoot, map[common.Hash]struct{}{
		accDelNoCache:  {},
		accDelCache:    {},
		conNukeNoCache: {},
		conNukeCache:   {},
	}, map[common.Hash][]byte{
		accModNoCache: reverse(accModNoCache[:]),
		accModCache:   reverse(accModCache[:]),
	}, map[common.Hash]map[common.Hash][]byte{
		conModNoCache: {conModNoCacheSlot: reverse(conModNoCacheSlot[:])},
		conModCache:   {conModCacheSlot: reverse(conModCacheSlot[:])},
		conDelNoCache: {conDelNoCacheSlot: nil},
		conDelCache:   {conDelCacheSlot: nil},
	}); err != nil {
		t.Fatalf("failed to update snapshot tree: %v", err)
	}
	if err := snaps.Cap(diffRoot, 0); err != nil {
		t.Fatalf("failed to flatten snapshot tree: %v", err)
	}
	// Retrieve all the data through the disk layer and validate it
	base = snaps.Snapshot(diffRoot)
	if _, ok := base.(*diskLayer); !ok {
		t.Fatalf("update not flattend into the disk layer")
	}

	// assertAccount ensures that an account matches the given blob.
	assertAccount := func(account common.Hash, data []byte) {
		t.Helper()
		blob, err := base.AccountRLP(account)
		if err != nil {
			t.Errorf("account access (%x) failed: %v", account, err)
		} else if !bytes.Equal(blob, data) {
			t.Errorf("account access (%x) mismatch: have %x, want %x", account, blob, data)
		}
	}
	assertAccount(accNoModNoCache, accNoModNoCache[:])
	assertAccount(accNoModCache, accNoModCache[:])
	assertAccount(accModNoCache, reverse(accModNoCache[:]))
	assertAccount(accModCache, reverse(accModCache[:]))
	assertAccount(accDelNoCache, nil)
	assertAccount(accDelCache, nil)

	// assertStorage ensures that a storage slot matches the given blob.
	assertStorage := func(account common.Hash, slot common.Hash, data []byte) {
		t.Helper()
		blob, err := base.Storage(account, slot)
		if err != nil {
			t.Errorf("storage access (%x:%x) failed: %v", account, slot, err)
		} else if !bytes.Equal(blob, data) {
			t.Errorf("storage access (%x:%x) mismatch: have %x, want %x", account, slot, blob, data)
		}
	}
	assertStorage(conNoModNoCache, conNoModNoCacheSlot, conNoModNoCacheSlot[:])
	assertStorage(conNoModCache, conNoModCacheSlot, conNoModCacheSlot[:])
	assertStorage(conModNoCache, conModNoCacheSlot, reverse(conModNoCacheSlot[:]))
	assertStorage(conModCache, conModCacheSlot, reverse(conModCacheSlot[:]))
	assertStorage(conDelNoCache, conDelNoCacheSlot, nil)
	assertStorage(conDelCache, conDelCacheSlot, nil)
	assertStorage(conNukeNoCache, conNukeNoCacheSlot, nil)
	assertStorage(conNukeCache, conNukeCacheSlot, nil)

	// Retrieve all the data directly from the database and validate it

	// assertDatabaseAccount ensures that an account from the database matches the given blob.
	assertDatabaseAccount := func(account common.Hash, data []byte) {
		t.Helper()
		if blob := rawdb.ReadAccountSnapshot(db, account); !bytes.Equal(blob, data) {
			t.Errorf("account database access (%x) mismatch: have %x, want %x", account, blob, data)
		}
	}
	assertDatabaseAccount(accNoModNoCache, accNoModNoCache[:])
	assertDatabaseAccount(accNoModCache, accNoModCache[:])
	assertDatabaseAccount(accModNoCache, reverse(accModNoCache[:]))
	assertDatabaseAccount(accModCache, reverse(accModCache[:]))
	assertDatabaseAccount(accDelNoCache, nil)
	assertDatabaseAccount(accDelCache, nil)

	// assertDatabaseStorage ensures that a storage slot from the database matches the given blob.
	assertDatabaseStorage := func(account common.Hash, slot common.Hash, data []byte) {
		t.Helper()
		if blob := rawdb.ReadStorageSnapshot(db, account, slot); !bytes.Equal(blob, data) {
			t.Errorf("storage database access (%x:%x) mismatch: have %x, want %x", account, slot, blob, data)
		}
	}
	assertDatabaseStorage(conNoModNoCache, conNoModNoCacheSlot, conNoModNoCacheSlot[:])
	assertDatabaseStorage(conNoModCache, conNoModCacheSlot, conNoModCacheSlot[:])
	assertDatabaseStorage(conModNoCache, conModNoCacheSlot, reverse(conModNoCacheSlot[:]))
	assertDatabaseStorage(conModCache, conModCacheSlot, reverse(conModCacheSlot[:]))
	assertDatabaseStorage(conDelNoCache, conDelNoCacheSlot, nil)
	assertDatabaseStorage(conDelCache, conDelCacheSlot, nil)
	assertDatabaseStorage(conNukeNoCache, conNukeNoCacheSlot, nil)
	assertDatabaseStorage(conNukeCache, conNukeCacheSlot, nil)
}

// Tests that merging something into a disk layer persists it into the database
// and invalidates any previously written and cached values, discarding anything
// after the in-progress generation marker.
func TestDiskPartialMerge(t *testing.T) {
	// Iterate the test a few times to ensure we pick various internal orderings
	// for the data slots as well as the progress marker.
	for i := 0; i < 1024; i++ {
		// Create some accounts in the disk layer
		db := memorydb.New()

		var (
			accNoModNoCache     = randomHash()
			accNoModCache       = randomHash()
			accModNoCache       = randomHash()
			accModCache         = randomHash()
			accDelNoCache       = randomHash()
			accDelCache         = randomHash()
			conNoModNoCache     = randomHash()
			conNoModNoCacheSlot = randomHash()
			conNoModCache       = randomHash()
			conNoModCacheSlot   = randomHash()
			conModNoCache       = randomHash()
			conModNoCacheSlot   = randomHash()
			conModCache         = randomHash()
			conModCacheSlot     = randomHash()
			conDelNoCache       = randomHash()
			conDelNoCacheSlot   = randomHash()
			conDelCache         = randomHash()
			conDelCacheSlot     = randomHash()
			conNukeNoCache      = randomHash()
			conNukeNoCacheSlot  = randomHash()
			conNukeCache        = randomHash()
			conNukeCacheSlot    = randomHash()
			baseRoot            = randomHash()
			diffRoot            = randomHash()
			genMarker           = append(randomHash().Bytes(), randomHash().Bytes()...)
		)

		// insertAccount injects an account into the database if it's after the
		// generator marker, drops the op otherwise. This is needed to seed the
		// database with a valid starting snapshot.
		insertAccount := func(account common.Hash, data []byte) {
			if bytes.Compare(account[:], genMarker) <= 0 {
				rawdb.WriteAccountSnapshot(db, account, data[:])
			}
		}
		insertAccount(accNoModNoCache, accNoModNoCache[:])
		insertAccount(accNoModCache, accNoModCache[:])
		insertAccount(accModNoCache, accModNoCache[:])
		insertAccount(accModCache, accModCache[:])
		insertAccount(accDelNoCache, accDelNoCache[:])
		insertAccount(accDelCache, accDelCache[:])

		// insertStorage injects a storage slot into the database if it's after
		// the  generator marker, drops the op otherwise. This is needed to seed
		// the  database with a valid starting snapshot.
		insertStorage := func(account common.Hash, slot common.Hash, data []byte) {
			if bytes.Compare(append(account[:], slot[:]...), genMarker) <= 0 {
				rawdb.WriteStorageSnapshot(db, account, slot, data[:])
			}
		}
		insertAccount(conNoModNoCache, conNoModNoCache[:])
		insertStorage(conNoModNoCache, conNoModNoCacheSlot, conNoModNoCacheSlot[:])
		insertAccount(conNoModCache, conNoModCache[:])
		insertStorage(conNoModCache, conNoModCacheSlot, conNoModCacheSlot[:])
		insertAccount(conModNoCache, conModNoCache[:])
		insertStorage(conModNoCache, conModNoCacheSlot, conModNoCacheSlot[:])
		insertAccount(conModCache, conModCache[:])
		insertStorage(conModCache, conModCacheSlot, conModCacheSlot[:])
		insertAccount(conDelNoCache, conDelNoCache[:])
		insertStorage(conDelNoCache, conDelNoCacheSlot, conDelNoCacheSlot[:])
		insertAccount(conDelCache, conDelCache[:])
		insertStorage(conDelCache, conDelCacheSlot, conDelCacheSlot[:])

		insertAccount(conNukeNoCache, conNukeNoCache[:])
		insertStorage(conNukeNoCache, conNukeNoCacheSlot, conNukeNoCacheSlot[:])
		insertAccount(conNukeCache, conNukeCache[:])
		insertStorage(conNukeCache, conNukeCacheSlot, conNukeCacheSlot[:])

		rawdb.WriteSnapshotRoot(db, baseRoot)

		// Create a disk layer based on the above using a random progress marker
		// and cache in some data.
		snaps := &Tree{
			layers: map[common.Hash]snapshot{
				baseRoot: &diskLayer{
					diskdb: db,
					cache:  fastcache.New(500 * 1024),
					root:   baseRoot,
				},
			},
		}
		snaps.layers[baseRoot].(*diskLayer).genMarker = genMarker
		base := snaps.Snapshot(baseRoot)

		// assertAccount ensures that an account matches the given blob if it's
		// already covered by the disk snapshot, and errors out otherwise.
		assertAccount := func(account common.Hash, data []byte) {
			t.Helper()
			blob, err := base.AccountRLP(account)
			if bytes.Compare(account[:], genMarker) > 0 && err != ErrNotCoveredYet {
				t.Fatalf("test %d: post-marker (%x) account access (%x) succeeded: %x", i, genMarker, account, blob)
			}
			if bytes.Compare(account[:], genMarker) <= 0 && !bytes.Equal(blob, data) {
				t.Fatalf("test %d: pre-marker (%x) account access (%x) mismatch: have %x, want %x", i, genMarker, account, blob, data)
			}
		}
		assertAccount(accNoModCache, accNoModCache[:])
		assertAccount(accModCache, accModCache[:])
		assertAccount(accDelCache, accDelCache[:])

		// assertStorage ensures that a storage slot matches the given blob if
		// it's already covered by the disk snapshot, and errors out otherwise.
		assertStorage := func(account common.Hash, slot common.Hash, data []byte) {
			t.Helper()
			blob, err := base.Storage(account, slot)
			if bytes.Compare(append(account[:], slot[:]...), genMarker) > 0 && err != ErrNotCoveredYet {
				t.Fatalf("test %d: post-marker (%x) storage access (%x:%x) succeeded: %x", i, genMarker, account, slot, blob)
			}
			if bytes.Compare(append(account[:], slot[:]...), genMarker) <= 0 && !bytes.Equal(blob, data) {
				t.Fatalf("test %d: pre-marker (%x) storage access (%x:%x) mismatch: have %x, want %x", i, genMarker, account, slot, blob, data)
			}
		}
		assertStorage(conNoModCache, conNoModCacheSlot, conNoModCacheSlot[:])
		assertStorage(conModCache, conModCacheSlot, conModCacheSlot[:])
		assertStorage(conDelCache, conDelCacheSlot, conDelCacheSlot[:])
		assertStorage(conNukeCache, conNukeCacheSlot, conNukeCacheSlot[:])

		// Modify or delete some accounts, flatten everything onto disk
		if err := snaps.Update(diffRoot, baseRoot, map[common.Hash]struct{}{
			accDelNoCache:  {},
			accDelCache:    {},
			conNukeNoCache: {},
			conNukeCache:   {},
		}, map[common.Hash][]byte{
			accModNoCache: reverse(accModNoCache[:]),
			accModCache:   reverse(accModCache[:]),
		}, map[common.Hash]map[common.Hash][]byte{
			conModNoCache: {conModNoCacheSlot: reverse(conModNoCacheSlot[:])},
			conModCache:   {conModCacheSlot: reverse(conModCacheSlot[:])},
			conDelNoCache: {conDelNoCacheSlot: nil},
			conDelCache:   {conDelCacheSlot: nil},
		}); err != nil {
			t.Fatalf("test %d: failed to update snapshot tree: %v", i, err)
		}
		if err := snaps.Cap(diffRoot, 0); err != nil {
			t.Fatalf("test %d: failed to flatten snapshot tree: %v", i, err)
		}
		// Retrieve all the data through the disk layer and validate it
		base = snaps.Snapshot(diffRoot)
		if _, ok := base.(*diskLayer); !ok {
			t.Fatalf("test %d: update not flattend into the disk layer", i)
		}
		assertAccount(accNoModNoCache, accNoModNoCache[:])
		assertAccount(accNoModCache, accNoModCache[:])
		assertAccount(accModNoCache, reverse(accModNoCache[:]))
		assertAccount(accModCache, reverse(accModCache[:]))
		assertAccount(accDelNoCache, nil)
		assertAccount(accDelCache, nil)

		assertStorage(conNoModNoCache, conNoModNoCacheSlot, conNoModNoCacheSlot[:])
		assertStorage(conNoModCache, conNoModCacheSlot, conNoModCacheSlot[:])
		assertStorage(conModNoCache, conModNoCacheSlot, reverse(conModNoCacheSlot[:]))
		assertStorage(conModCache, conModCacheSlot, reverse(conModCacheSlot[:]))
		assertStorage(conDelNoCache, conDelNoCacheSlot, nil)
		assertStorage(conDelCache, conDelCacheSlot, nil)
		assertStorage(conNukeNoCache, conNukeNoCacheSlot, nil)
		assertStorage(conNukeCache, conNukeCacheSlot, nil)

		// Retrieve all the data directly from the database and validate it

		// assertDatabaseAccount ensures that an account inside the database matches
		// the given blob if it's already covered by the disk snapshot, and does not
		// exist otherwise.
		assertDatabaseAccount := func(account common.Hash, data []byte) {
			t.Helper()
			blob := rawdb.ReadAccountSnapshot(db, account)
			if bytes.Compare(account[:], genMarker) > 0 && blob != nil {
				t.Fatalf("test %d: post-marker (%x) account database access (%x) succeeded: %x", i, genMarker, account, blob)
			}
			if bytes.Compare(account[:], genMarker) <= 0 && !bytes.Equal(blob, data) {
				t.Fatalf("test %d: pre-marker (%x) account database access (%x) mismatch: have %x, want %x", i, genMarker, account, blob, data)
			}
		}
		assertDatabaseAccount(accNoModNoCache, accNoModNoCache[:])
		assertDatabaseAccount(accNoModCache, accNoModCache[:])
		assertDatabaseAccount(accModNoCache, reverse(accModNoCache[:]))
		assertDatabaseAccount(accModCache, reverse(accModCache[:]))
		assertDatabaseAccount(accDelNoCache, nil)
		assertDatabaseAccount(accDelCache, nil)

		// assertDatabaseStorage ensures that a storage slot inside the database
		// matches the given blob if it's already covered by the disk snapshot,
		// and does not exist otherwise.
		assertDatabaseStorage := func(account common.Hash, slot common.Hash, data []byte) {
			t.Helper()
			blob := rawdb.ReadStorageSnapshot(db, account, slot)
			if bytes.Compare(append(account[:], slot[:]...), genMarker) > 0 && blob != nil {
				t.Fatalf("test %d: post-marker (%x) storage database access (%x:%x) succeeded: %x", i, genMarker, account, slot, blob)
			}
			if bytes.Compare(append(account[:], slot[:]...), genMarker) <= 0 && !bytes.Equal(blob, data) {
				t.Fatalf("test %d: pre-marker (%x) storage database access (%x:%x) mismatch: have %x, want %x", i, genMarker, account, slot, blob, data)
			}
		}
		assertDatabaseStorage(conNoModNoCache, conNoModNoCacheSlot, conNoModNoCacheSlot[:])
		assertDatabaseStorage(conNoModCache, conNoModCacheSlot, conNoModCacheSlot[:])
		assertDatabaseStorage(conModNoCache, conModNoCacheSlot, reverse(conModNoCacheSlot[:]))
		assertDatabaseStorage(conModCache, conModCacheSlot, reverse(conModCacheSlot[:]))
		assertDatabaseStorage(conDelNoCache, conDelNoCacheSlot, nil)
		assertDatabaseStorage(conDelCache, conDelCacheSlot, nil)
		assertDatabaseStorage(conNukeNoCache, conNukeNoCacheSlot, nil)
		assertDatabaseStorage(conNukeCache, conNukeCacheSlot, nil)
	}
}

// Tests that when the bottom-most diff layer is merged into the disk
// layer whether the corresponding generator is persisted correctly.
func TestDiskGeneratorPersistence(t *testing.T) {
	var (
		accOne        = randomHash()
		accTwo        = randomHash()
		accOneSlotOne = randomHash()
		accOneSlotTwo = randomHash()

		accThree     = randomHash()
		accThreeSlot = randomHash()
		baseRoot     = randomHash()
		diffRoot     = randomHash()
		diffTwoRoot  = randomHash()
		genMarker    = append(randomHash().Bytes(), randomHash().Bytes()...)
	)
	// Testing scenario 1, the disk layer is still under the construction.
	db := rawdb.NewMemoryDatabase()

	rawdb.WriteAccountSnapshot(db, accOne, accOne[:])
	rawdb.WriteStorageSnapshot(db, accOne, accOneSlotOne, accOneSlotOne[:])
	rawdb.WriteStorageSnapshot(db, accOne, accOneSlotTwo, accOneSlotTwo[:])
	rawdb.WriteSnapshotRoot(db, baseRoot)

	// Create a disk layer based on all above updates
	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			baseRoot: &diskLayer{
				diskdb:    db,
				cache:     fastcache.New(500 * 1024),
				root:      baseRoot,
				genMarker: genMarker,
			},
		},
	}
	// Modify or delete some accounts, flatten everything onto disk
	if err := snaps.Update(diffRoot, baseRoot, nil, map[common.Hash][]byte{
		accTwo: accTwo[:],
	}, nil); err != nil {
		t.Fatalf("failed to update snapshot tree: %v", err)
	}
	if err := snaps.Cap(diffRoot, 0); err != nil {
		t.Fatalf("failed to flatten snapshot tree: %v", err)
	}
	blob := rawdb.ReadSnapshotGenerator(db)
	var generator journalGenerator
	if err := rlp.DecodeBytes(blob, &generator); err != nil {
		t.Fatalf("Failed to decode snapshot generator %v", err)
	}
	if !bytes.Equal(generator.Marker, genMarker) {
		t.Fatalf("Generator marker is not matched")
	}
	// Test scenario 2, the disk layer is fully generated
	// Modify or delete some accounts, flatten everything onto disk
	if err := snaps.Update(diffTwoRoot, diffRoot, nil, map[common.Hash][]byte{
		accThree: accThree.Bytes(),
	}, map[common.Hash]map[common.Hash][]byte{
		accThree: {accThreeSlot: accThreeSlot.Bytes()},
	}); err != nil {
		t.Fatalf("failed to update snapshot tree: %v", err)
	}
	diskLayer := snaps.layers[snaps.diskRoot()].(*diskLayer)
	diskLayer.genMarker = nil // Construction finished
	if err := snaps.Cap(diffTwoRoot, 0); err != nil {
		t.Fatalf("failed to flatten snapshot tree: %v", err)
	}
	blob = rawdb.ReadSnapshotGenerator(db)
	if err := rlp.DecodeBytes(blob, &generator); err != nil {
		t.Fatalf("Failed to decode snapshot generator %v", err)
	}
	if len(generator.Marker) != 0 {
		t.Fatalf("Failed to update snapshot generator")
	}
}

// Tests that merging something into a disk layer persists it into the database
// and invalidates any previously written and cached values, discarding anything
// after the in-progress generation marker.
//
// This test case is a tiny specialized case of TestDiskPartialMerge, which tests
// some very specific cornercases that random tests won't ever trigger.
func TestDiskMidAccountPartialMerge(t *testing.T) {
	// TODO(@karalabe) ?
}

// TestDiskSeek tests that seek-operations work on the disk layer
func TestDiskSeek(t *testing.T) {
	// Create some accounts in the disk layer
	db := rawdb.NewMemoryDatabase()
	defer db.Close()

	// Fill even keys [0,2,4...]
	for i := 0; i < 0xff; i += 2 {
		acc := common.Hash{byte(i)}
		rawdb.WriteAccountSnapshot(db, acc, acc[:])
	}
	// Add an 'higher' key, with incorrect (higher) prefix
	highKey := []byte{rawdb.SnapshotAccountPrefix[0] + 1}
	db.Put(highKey, []byte{0xff, 0xff})

	baseRoot := randomHash()
	rawdb.WriteSnapshotRoot(db, baseRoot)

	snaps := &Tree{
		layers: map[common.Hash]snapshot{
			baseRoot: &diskLayer{
				diskdb: db,
				cache:  fastcache.New(500 * 1024),
				root:   baseRoot,
			},
		},
	}
	// Test some different seek positions
	type testcase struct {
		pos    byte
		expkey byte
	}
	var cases = []testcase{
		{0xff, 0x55}, // this should exit immediately without checking key
		{0x01, 0x02},
		{0xfe, 0xfe},
		{0xfd, 0xfe},
		{0x00, 0x00},
	}
	for i, tc := range cases {
		it, err := snaps.AccountIterator(baseRoot, common.Hash{tc.pos})
		if err != nil {
			t.Fatalf("case %d, error: %v", i, err)
		}
		count := 0
		for it.Next() {
			k, v, err := it.Hash()[0], it.Account()[0], it.Error()
			if err != nil {
				t.Fatalf("test %d, item %d, error: %v", i, count, err)
			}
			// First item in iterator should have the expected key
			if count == 0 && k != tc.expkey {
				t.Fatalf("test %d, item %d, got %v exp %v", i, count, k, tc.expkey)
			}
			count++
			if v != k {
				t.Fatalf("test %d, item %d, value wrong, got %v exp %v", i, count, v, k)
			}
		}
	}
}
