// Copyright 2020 The go-ethereum Authors
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

// Tests that abnormal program termination (i.e.crash) and restart doesn't leave
// the database in some strange state with gaps in the chain, nor with block data
// dangling in the future.

package core

import (
	"math/big"
	"path"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/params"
)

// Tests a recovery for a short canonical chain where a recent block was already
// committed to disk and then the process crashed. In this case we expect the full
// chain to be rolled back to the committed block, but the chain data itself left
// in the database for replaying.
func TestShortRepair(t *testing.T)              { testShortRepair(t, false) }
func TestShortRepairWithSnapshots(t *testing.T) { testShortRepair(t, true) }

func testShortRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 8,
		expSidechainBlocks: 0,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a short canonical chain where the fast sync pivot point was
// already committed, after which the process crashed. In this case we expect the full
// chain to be rolled back to the committed block, but the chain data itself left in
// the database for replaying.
func TestShortSnapSyncedRepair(t *testing.T)              { testShortSnapSyncedRepair(t, false) }
func TestShortSnapSyncedRepairWithSnapshots(t *testing.T) { testShortSnapSyncedRepair(t, true) }

func testShortSnapSyncedRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 8,
		expSidechainBlocks: 0,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a short canonical chain where the fast sync pivot point was
// not yet committed, but the process crashed. In this case we expect the chain to
// detect that it was fast syncing and not delete anything, since we can just pick
// up directly where we left off.
func TestShortSnapSyncingRepair(t *testing.T)              { testShortSnapSyncingRepair(t, false) }
func TestShortSnapSyncingRepairWithSnapshots(t *testing.T) { testShortSnapSyncingRepair(t, true) }

func testShortSnapSyncingRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//
	// Frozen: none
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 8,
		expSidechainBlocks: 0,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a shorter side chain, where a
// recent block was already committed to disk and then the process crashed. In this
// test scenario the side chain is below the committed block. In this case we expect
// the canonical chain to be rolled back to the committed block, but the chain data
// itself left in the database for replaying.
func TestShortOldForkedRepair(t *testing.T)              { testShortOldForkedRepair(t, false) }
func TestShortOldForkedRepairWithSnapshots(t *testing.T) { testShortOldForkedRepair(t, true) }

func testShortOldForkedRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 8,
		expSidechainBlocks: 3,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a shorter side chain, where
// the fast sync pivot point was already committed to disk and then the process
// crashed. In this test scenario the side chain is below the committed block. In
// this case we expect the canonical chain to be rolled back to the committed block,
// but the chain data itself left in the database for replaying.
func TestShortOldForkedSnapSyncedRepair(t *testing.T) {
	testShortOldForkedSnapSyncedRepair(t, false)
}
func TestShortOldForkedSnapSyncedRepairWithSnapshots(t *testing.T) {
	testShortOldForkedSnapSyncedRepair(t, true)
}

func testShortOldForkedSnapSyncedRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 8,
		expSidechainBlocks: 3,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a shorter side chain, where
// the fast sync pivot point was not yet committed, but the process crashed. In this
// test scenario the side chain is below the committed block. In this case we expect
// the chain to detect that it was fast syncing and not delete anything, since we
// can just pick up directly where we left off.
func TestShortOldForkedSnapSyncingRepair(t *testing.T) {
	testShortOldForkedSnapSyncingRepair(t, false)
}
func TestShortOldForkedSnapSyncingRepairWithSnapshots(t *testing.T) {
	testShortOldForkedSnapSyncingRepair(t, true)
}

func testShortOldForkedSnapSyncingRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen: none
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 8,
		expSidechainBlocks: 3,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a shorter side chain, where a
// recent block was already committed to disk and then the process crashed. In this
// test scenario the side chain reaches above the committed block. In this case we
// expect the canonical chain to be rolled back to the committed block, but the
// chain data itself left in the database for replaying.
func TestShortNewlyForkedRepair(t *testing.T)              { testShortNewlyForkedRepair(t, false) }
func TestShortNewlyForkedRepairWithSnapshots(t *testing.T) { testShortNewlyForkedRepair(t, true) }

func testShortNewlyForkedRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3->S4->S5->S6
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    6,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 8,
		expSidechainBlocks: 6,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a shorter side chain, where
// the fast sync pivot point was already committed to disk and then the process
// crashed. In this test scenario the side chain reaches above the committed block.
// In this case we expect the canonical chain to be rolled back to the committed
// block, but the chain data itself left in the database for replaying.
func TestShortNewlyForkedSnapSyncedRepair(t *testing.T) {
	testShortNewlyForkedSnapSyncedRepair(t, false)
}
func TestShortNewlyForkedSnapSyncedRepairWithSnapshots(t *testing.T) {
	testShortNewlyForkedSnapSyncedRepair(t, true)
}

func testShortNewlyForkedSnapSyncedRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3->S4->S5->S6
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    6,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 8,
		expSidechainBlocks: 6,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a shorter side chain, where
// the fast sync pivot point was not yet committed, but the process crashed. In
// this test scenario the side chain reaches above the committed block. In this
// case we expect the chain to detect that it was fast syncing and not delete
// anything, since we can just pick up directly where we left off.
func TestShortNewlyForkedSnapSyncingRepair(t *testing.T) {
	testShortNewlyForkedSnapSyncingRepair(t, false)
}
func TestShortNewlyForkedSnapSyncingRepairWithSnapshots(t *testing.T) {
	testShortNewlyForkedSnapSyncingRepair(t, true)
}

func testShortNewlyForkedSnapSyncingRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6
	//
	// Frozen: none
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3->S4->S5->S6
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    6,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 8,
		expSidechainBlocks: 6,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a longer side chain, where a
// recent block was already committed to disk and then the process crashed. In this
// case we expect the canonical chain to be rolled back to the committed block, but
// the chain data itself left in the database for replaying.
func TestShortReorgedRepair(t *testing.T)              { testShortReorgedRepair(t, false) }
func TestShortReorgedRepairWithSnapshots(t *testing.T) { testShortReorgedRepair(t, true) }

func testShortReorgedRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    10,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 8,
		expSidechainBlocks: 10,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a longer side chain, where
// the fast sync pivot point was already committed to disk and then the process
// crashed. In this case we expect the canonical chain to be rolled back to the
// committed block, but the chain data itself left in the database for replaying.
func TestShortReorgedSnapSyncedRepair(t *testing.T) {
	testShortReorgedSnapSyncedRepair(t, false)
}
func TestShortReorgedSnapSyncedRepairWithSnapshots(t *testing.T) {
	testShortReorgedSnapSyncedRepair(t, true)
}

func testShortReorgedSnapSyncedRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    10,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 8,
		expSidechainBlocks: 10,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a short canonical chain and a longer side chain, where
// the fast sync pivot point was not yet committed, but the process crashed. In
// this case we expect the chain to detect that it was fast syncing and not delete
// anything, since we can just pick up directly where we left off.
func TestShortReorgedSnapSyncingRepair(t *testing.T) {
	testShortReorgedSnapSyncingRepair(t, false)
}
func TestShortReorgedSnapSyncingRepairWithSnapshots(t *testing.T) {
	testShortReorgedSnapSyncingRepair(t, true)
}

func testShortReorgedSnapSyncingRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Frozen: none
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Expected head header    : C8
	// Expected head fast block: C8
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    10,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 8,
		expSidechainBlocks: 10,
		expFrozen:          0,
		expHeadHeader:      8,
		expHeadFastBlock:   8,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks where a recent
// block - newer than the ancient limit - was already committed to disk and then
// the process crashed. In this case we expect the chain to be rolled back to the
// committed block, with everything afterwards kept as fast sync data.
func TestLongShallowRepair(t *testing.T)              { testLongShallowRepair(t, false) }
func TestLongShallowRepairWithSnapshots(t *testing.T) { testLongShallowRepair(t, true) }

func testLongShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks where a recent
// block - older than the ancient limit - was already committed to disk and then
// the process crashed. In this case we expect the chain to be rolled back to the
// committed block, with everything afterwards deleted.
func TestLongDeepRepair(t *testing.T)              { testLongDeepRepair(t, false) }
func TestLongDeepRepairWithSnapshots(t *testing.T) { testLongDeepRepair(t, true) }

func testLongDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C4
	// Expected head fast block: C4
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks where the fast
// sync pivot point - newer than the ancient limit - was already committed, after
// which the process crashed. In this case we expect the chain to be rolled back
// to the committed block, with everything afterwards kept as fast sync data.
func TestLongSnapSyncedShallowRepair(t *testing.T) {
	testLongSnapSyncedShallowRepair(t, false)
}
func TestLongSnapSyncedShallowRepairWithSnapshots(t *testing.T) {
	testLongSnapSyncedShallowRepair(t, true)
}

func testLongSnapSyncedShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks where the fast
// sync pivot point - older than the ancient limit - was already committed, after
// which the process crashed. In this case we expect the chain to be rolled back
// to the committed block, with everything afterwards deleted.
func TestLongSnapSyncedDeepRepair(t *testing.T)              { testLongSnapSyncedDeepRepair(t, false) }
func TestLongSnapSyncedDeepRepairWithSnapshots(t *testing.T) { testLongSnapSyncedDeepRepair(t, true) }

func testLongSnapSyncedDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C4
	// Expected head fast block: C4
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks where the fast
// sync pivot point - older than the ancient limit - was not yet committed, but the
// process crashed. In this case we expect the chain to detect that it was fast
// syncing and not delete anything, since we can just pick up directly where we
// left off.
func TestLongSnapSyncingShallowRepair(t *testing.T) {
	testLongSnapSyncingShallowRepair(t, false)
}
func TestLongSnapSyncingShallowRepairWithSnapshots(t *testing.T) {
	testLongSnapSyncingShallowRepair(t, true)
}

func testLongSnapSyncingShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks where the fast
// sync pivot point - newer than the ancient limit - was not yet committed, but the
// process crashed. In this case we expect the chain to detect that it was fast
// syncing and not delete anything, since we can just pick up directly where we
// left off.
func TestLongSnapSyncingDeepRepair(t *testing.T)              { testLongSnapSyncingDeepRepair(t, false) }
func TestLongSnapSyncingDeepRepairWithSnapshots(t *testing.T) { testLongSnapSyncingDeepRepair(t, true) }

func testLongSnapSyncingDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Expected in leveldb:
	//   C8)->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24
	//
	// Expected head header    : C24
	// Expected head fast block: C24
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 24,
		expSidechainBlocks: 0,
		expFrozen:          9,
		expHeadHeader:      24,
		expHeadFastBlock:   24,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where a recent block - newer than the ancient limit - was already
// committed to disk and then the process crashed. In this test scenario the side
// chain is below the committed block. In this case we expect the chain to be
// rolled back to the committed block, with everything afterwards kept as fast
// sync data; the side chain completely nuked by the freezer.
func TestLongOldForkedShallowRepair(t *testing.T) {
	testLongOldForkedShallowRepair(t, false)
}
func TestLongOldForkedShallowRepairWithSnapshots(t *testing.T) {
	testLongOldForkedShallowRepair(t, true)
}

func testLongOldForkedShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where a recent block - older than the ancient limit - was already
// committed to disk and then the process crashed. In this test scenario the side
// chain is below the committed block. In this case we expect the canonical chain
// to be rolled back to the committed block, with everything afterwards deleted;
// the side chain completely nuked by the freezer.
func TestLongOldForkedDeepRepair(t *testing.T)              { testLongOldForkedDeepRepair(t, false) }
func TestLongOldForkedDeepRepairWithSnapshots(t *testing.T) { testLongOldForkedDeepRepair(t, true) }

func testLongOldForkedDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C4
	// Expected head fast block: C4
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was already committed to disk and then the process crashed. In this test scenario
// the side chain is below the committed block. In this case we expect the chain
// to be rolled back to the committed block, with everything afterwards kept as
// fast sync data; the side chain completely nuked by the freezer.
func TestLongOldForkedSnapSyncedShallowRepair(t *testing.T) {
	testLongOldForkedSnapSyncedShallowRepair(t, false)
}
func TestLongOldForkedSnapSyncedShallowRepairWithSnapshots(t *testing.T) {
	testLongOldForkedSnapSyncedShallowRepair(t, true)
}

func testLongOldForkedSnapSyncedShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was already committed to disk and then the process crashed. In this test scenario
// the side chain is below the committed block. In this case we expect the canonical
// chain to be rolled back to the committed block, with everything afterwards deleted;
// the side chain completely nuked by the freezer.
func TestLongOldForkedSnapSyncedDeepRepair(t *testing.T) {
	testLongOldForkedSnapSyncedDeepRepair(t, false)
}
func TestLongOldForkedSnapSyncedDeepRepairWithSnapshots(t *testing.T) {
	testLongOldForkedSnapSyncedDeepRepair(t, true)
}

func testLongOldForkedSnapSyncedDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C4
	// Expected head fast block: C4
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was not yet committed, but the process crashed. In this test scenario the side
// chain is below the committed block. In this case we expect the chain to detect
// that it was fast syncing and not delete anything. The side chain is completely
// nuked by the freezer.
func TestLongOldForkedSnapSyncingShallowRepair(t *testing.T) {
	testLongOldForkedSnapSyncingShallowRepair(t, false)
}
func TestLongOldForkedSnapSyncingShallowRepairWithSnapshots(t *testing.T) {
	testLongOldForkedSnapSyncingShallowRepair(t, true)
}

func testLongOldForkedSnapSyncingShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was not yet committed, but the process crashed. In this test scenario the side
// chain is below the committed block. In this case we expect the chain to detect
// that it was fast syncing and not delete anything. The side chain is completely
// nuked by the freezer.
func TestLongOldForkedSnapSyncingDeepRepair(t *testing.T) {
	testLongOldForkedSnapSyncingDeepRepair(t, false)
}
func TestLongOldForkedSnapSyncingDeepRepairWithSnapshots(t *testing.T) {
	testLongOldForkedSnapSyncingDeepRepair(t, true)
}

func testLongOldForkedSnapSyncingDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Expected in leveldb:
	//   C8)->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24
	//
	// Expected head header    : C24
	// Expected head fast block: C24
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 24,
		expSidechainBlocks: 0,
		expFrozen:          9,
		expHeadHeader:      24,
		expHeadFastBlock:   24,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where a recent block - newer than the ancient limit - was already
// committed to disk and then the process crashed. In this test scenario the side
// chain is above the committed block. In this case we expect the chain to be
// rolled back to the committed block, with everything afterwards kept as fast
// sync data; the side chain completely nuked by the freezer.
func TestLongNewerForkedShallowRepair(t *testing.T) {
	testLongNewerForkedShallowRepair(t, false)
}
func TestLongNewerForkedShallowRepairWithSnapshots(t *testing.T) {
	testLongNewerForkedShallowRepair(t, true)
}

func testLongNewerForkedShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where a recent block - older than the ancient limit - was already
// committed to disk and then the process crashed. In this test scenario the side
// chain is above the committed block. In this case we expect the canonical chain
// to be rolled back to the committed block, with everything afterwards deleted;
// the side chain completely nuked by the freezer.
func TestLongNewerForkedDeepRepair(t *testing.T)              { testLongNewerForkedDeepRepair(t, false) }
func TestLongNewerForkedDeepRepairWithSnapshots(t *testing.T) { testLongNewerForkedDeepRepair(t, true) }

func testLongNewerForkedDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C4
	// Expected head fast block: C4
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was already committed to disk and then the process crashed. In this test scenario
// the side chain is above the committed block. In this case we expect the chain
// to be rolled back to the committed block, with everything afterwards kept as fast
// sync data; the side chain completely nuked by the freezer.
func TestLongNewerForkedSnapSyncedShallowRepair(t *testing.T) {
	testLongNewerForkedSnapSyncedShallowRepair(t, false)
}
func TestLongNewerForkedSnapSyncedShallowRepairWithSnapshots(t *testing.T) {
	testLongNewerForkedSnapSyncedShallowRepair(t, true)
}

func testLongNewerForkedSnapSyncedShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was already committed to disk and then the process crashed. In this test scenario
// the side chain is above the committed block. In this case we expect the canonical
// chain to be rolled back to the committed block, with everything afterwards deleted;
// the side chain completely nuked by the freezer.
func TestLongNewerForkedSnapSyncedDeepRepair(t *testing.T) {
	testLongNewerForkedSnapSyncedDeepRepair(t, false)
}
func TestLongNewerForkedSnapSyncedDeepRepairWithSnapshots(t *testing.T) {
	testLongNewerForkedSnapSyncedDeepRepair(t, true)
}

func testLongNewerForkedSnapSyncedDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C4
	// Expected head fast block: C4
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was not yet committed, but the process crashed. In this test scenario the side
// chain is above the committed block. In this case we expect the chain to detect
// that it was fast syncing and not delete anything. The side chain is completely
// nuked by the freezer.
func TestLongNewerForkedSnapSyncingShallowRepair(t *testing.T) {
	testLongNewerForkedSnapSyncingShallowRepair(t, false)
}
func TestLongNewerForkedSnapSyncingShallowRepairWithSnapshots(t *testing.T) {
	testLongNewerForkedSnapSyncingShallowRepair(t, true)
}

func testLongNewerForkedSnapSyncingShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was not yet committed, but the process crashed. In this test scenario the side
// chain is above the committed block. In this case we expect the chain to detect
// that it was fast syncing and not delete anything. The side chain is completely
// nuked by the freezer.
func TestLongNewerForkedSnapSyncingDeepRepair(t *testing.T) {
	testLongNewerForkedSnapSyncingDeepRepair(t, false)
}
func TestLongNewerForkedSnapSyncingDeepRepairWithSnapshots(t *testing.T) {
	testLongNewerForkedSnapSyncingDeepRepair(t, true)
}

func testLongNewerForkedSnapSyncingDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Expected in leveldb:
	//   C8)->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24
	//
	// Expected head header    : C24
	// Expected head fast block: C24
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 24,
		expSidechainBlocks: 0,
		expFrozen:          9,
		expHeadHeader:      24,
		expHeadFastBlock:   24,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a longer side
// chain, where a recent block - newer than the ancient limit - was already committed
// to disk and then the process crashed. In this case we expect the chain to be
// rolled back to the committed block, with everything afterwards kept as fast sync
// data. The side chain completely nuked by the freezer.
func TestLongReorgedShallowRepair(t *testing.T)              { testLongReorgedShallowRepair(t, false) }
func TestLongReorgedShallowRepairWithSnapshots(t *testing.T) { testLongReorgedShallowRepair(t, true) }

func testLongReorgedShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12->S13->S14->S15->S16->S17->S18->S19->S20->S21->S22->S23->S24->S25->S26
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a longer side
// chain, where a recent block - older than the ancient limit - was already committed
// to disk and then the process crashed. In this case we expect the canonical chains
// to be rolled back to the committed block, with everything afterwards deleted. The
// side chain completely nuked by the freezer.
func TestLongReorgedDeepRepair(t *testing.T)              { testLongReorgedDeepRepair(t, false) }
func TestLongReorgedDeepRepairWithSnapshots(t *testing.T) { testLongReorgedDeepRepair(t, true) }

func testLongReorgedDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12->S13->S14->S15->S16->S17->S18->S19->S20->S21->S22->S23->S24->S25->S26
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : none
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C4
	// Expected head fast block: C4
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a longer
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was already committed to disk and then the process crashed. In this case we
// expect the chain to be rolled back to the committed block, with everything
// afterwards kept as fast sync data. The side chain completely nuked by the
// freezer.
func TestLongReorgedSnapSyncedShallowRepair(t *testing.T) {
	testLongReorgedSnapSyncedShallowRepair(t, false)
}
func TestLongReorgedSnapSyncedShallowRepairWithSnapshots(t *testing.T) {
	testLongReorgedSnapSyncedShallowRepair(t, true)
}

func testLongReorgedSnapSyncedShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12->S13->S14->S15->S16->S17->S18->S19->S20->S21->S22->S23->S24->S25->S26
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a longer
// side chain, where the fast sync pivot point - older than the ancient limit -
// was already committed to disk and then the process crashed. In this case we
// expect the canonical chains to be rolled back to the committed block, with
// everything afterwards deleted. The side chain completely nuked by the freezer.
func TestLongReorgedSnapSyncedDeepRepair(t *testing.T) {
	testLongReorgedSnapSyncedDeepRepair(t, false)
}
func TestLongReorgedSnapSyncedDeepRepairWithSnapshots(t *testing.T) {
	testLongReorgedSnapSyncedDeepRepair(t, true)
}

func testLongReorgedSnapSyncedDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12->S13->S14->S15->S16->S17->S18->S19->S20->S21->S22->S23->S24->S25->S26
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C4
	// Expected head fast block: C4
	// Expected head block     : C4
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a longer
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was not yet committed, but the process crashed. In this case we expect the
// chain to detect that it was fast syncing and not delete anything, since we
// can just pick up directly where we left off.
func TestLongReorgedSnapSyncingShallowRepair(t *testing.T) {
	testLongReorgedSnapSyncingShallowRepair(t, false)
}
func TestLongReorgedSnapSyncingShallowRepairWithSnapshots(t *testing.T) {
	testLongReorgedSnapSyncingShallowRepair(t, true)
}

func testLongReorgedSnapSyncingShallowRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12->S13->S14->S15->S16->S17->S18->S19->S20->S21->S22->S23->S24->S25->S26
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18
	//
	// Expected head header    : C18
	// Expected head fast block: C18
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 18,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      18,
		expHeadFastBlock:   18,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a recovery for a long canonical chain with frozen blocks and a longer
// side chain, where the fast sync pivot point - older than the ancient limit -
// was not yet committed, but the process crashed. In this case we expect the
// chain to detect that it was fast syncing and not delete anything, since we
// can just pick up directly where we left off.
func TestLongReorgedSnapSyncingDeepRepair(t *testing.T) {
	testLongReorgedSnapSyncingDeepRepair(t, false)
}
func TestLongReorgedSnapSyncingDeepRepairWithSnapshots(t *testing.T) {
	testLongReorgedSnapSyncingDeepRepair(t, true)
}

func testLongReorgedSnapSyncingDeepRepair(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10->S11->S12->S13->S14->S15->S16->S17->S18->S19->S20->S21->S22->S23->S24->S25->S26
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G
	// Pivot : C4
	//
	// CRASH
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Expected in leveldb:
	//   C8)->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24
	//
	// Expected head header    : C24
	// Expected head fast block: C24
	// Expected head block     : G
	testRepair(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		expCanonicalBlocks: 24,
		expSidechainBlocks: 0,
		expFrozen:          9,
		expHeadHeader:      24,
		expHeadFastBlock:   24,
		expHeadBlock:       0,
	}, snapshots)
}

func testRepair(t *testing.T, tt *rewindTest, snapshots bool) {
	for _, scheme := range []string{rawdb.HashScheme, rawdb.PathScheme} {
		testRepairWithScheme(t, tt, snapshots, scheme)
	}
}

func testRepairWithScheme(t *testing.T, tt *rewindTest, snapshots bool, scheme string) {
	// It's hard to follow the test case, visualize the input
	//log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))
	// fmt.Println(tt.dump(true))

	// Create a temporary persistent database
	datadir := t.TempDir()
	ancient := path.Join(datadir, "ancient")

	db, err := rawdb.Open(rawdb.OpenOptions{
		Directory:         datadir,
		AncientsDirectory: ancient,
		Ephemeral:         true,
	})
	if err != nil {
		t.Fatalf("Failed to create persistent database: %v", err)
	}
	defer db.Close() // Might double close, should be fine

	// Initialize a fresh chain
	var (
		gspec = &Genesis{
			BaseFee: big.NewInt(params.InitialBaseFee),
			Config:  params.AllEthashProtocolChanges,
		}
		engine = ethash.NewFullFaker()
		config = &CacheConfig{
			TrieCleanLimit: 256,
			TrieDirtyLimit: 256,
			TrieTimeLimit:  5 * time.Minute,
			SnapshotLimit:  0, // Disable snapshot by default
			StateScheme:    scheme,
		}
	)
	defer engine.Close()
	if snapshots {
		config.SnapshotLimit = 256
		config.SnapshotWait = true
	}
	chain, err := NewBlockChain(db, config, gspec, nil, engine, vm.Config{}, nil, nil)
	if err != nil {
		t.Fatalf("Failed to create chain: %v", err)
	}
	// If sidechain blocks are needed, make a light chain and import it
	var sideblocks types.Blocks
	if tt.sidechainBlocks > 0 {
		sideblocks, _ = GenerateChain(gspec.Config, gspec.ToBlock(), engine, rawdb.NewMemoryDatabase(), tt.sidechainBlocks, func(i int, b *BlockGen) {
			b.SetCoinbase(common.Address{0x01})
		})
		if _, err := chain.InsertChain(sideblocks); err != nil {
			t.Fatalf("Failed to import side chain: %v", err)
		}
	}
	canonblocks, _ := GenerateChain(gspec.Config, gspec.ToBlock(), engine, rawdb.NewMemoryDatabase(), tt.canonicalBlocks, func(i int, b *BlockGen) {
		b.SetCoinbase(common.Address{0x02})
		b.SetDifficulty(big.NewInt(1000000))
	})
	if _, err := chain.InsertChain(canonblocks[:tt.commitBlock]); err != nil {
		t.Fatalf("Failed to import canonical chain start: %v", err)
	}
	if tt.commitBlock > 0 {
		if err := chain.triedb.Commit(canonblocks[tt.commitBlock-1].Root(), false); err != nil {
			t.Fatalf("Failed to flush trie state: %v", err)
		}
		if snapshots {
			if err := chain.snaps.Cap(canonblocks[tt.commitBlock-1].Root(), 0); err != nil {
				t.Fatalf("Failed to flatten snapshots: %v", err)
			}
		}
	}
	if _, err := chain.InsertChain(canonblocks[tt.commitBlock:]); err != nil {
		t.Fatalf("Failed to import canonical chain tail: %v", err)
	}
	// Force run a freeze cycle
	type freezer interface {
		Freeze(threshold uint64) error
		Ancients() (uint64, error)
	}
	db.(freezer).Freeze(tt.freezeThreshold)

	// Set the simulated pivot block
	if tt.pivotBlock != nil {
		rawdb.WriteLastPivotNumber(db, *tt.pivotBlock)
	}
	// Pull the plug on the database, simulating a hard crash
	chain.triedb.Close()
	db.Close()
	chain.stopWithoutSaving()

	// Start a new blockchain back up and see where the repair leads us
	db, err = rawdb.Open(rawdb.OpenOptions{
		Directory:         datadir,
		AncientsDirectory: ancient,
		Ephemeral:         true,
	})
	if err != nil {
		t.Fatalf("Failed to reopen persistent database: %v", err)
	}
	defer db.Close()

	newChain, err := NewBlockChain(db, config, gspec, nil, engine, vm.Config{}, nil, nil)
	if err != nil {
		t.Fatalf("Failed to recreate chain: %v", err)
	}
	defer newChain.Stop()

	// Iterate over all the remaining blocks and ensure there are no gaps
	verifyNoGaps(t, newChain, true, canonblocks)
	verifyNoGaps(t, newChain, false, sideblocks)
	verifyCutoff(t, newChain, true, canonblocks, tt.expCanonicalBlocks)
	verifyCutoff(t, newChain, false, sideblocks, tt.expSidechainBlocks)

	if head := newChain.CurrentHeader(); head.Number.Uint64() != tt.expHeadHeader {
		t.Errorf("Head header mismatch: have %d, want %d", head.Number, tt.expHeadHeader)
	}
	if head := newChain.CurrentSnapBlock(); head.Number.Uint64() != tt.expHeadFastBlock {
		t.Errorf("Head fast block mismatch: have %d, want %d", head.Number, tt.expHeadFastBlock)
	}
	if head := newChain.CurrentBlock(); head.Number.Uint64() != tt.expHeadBlock {
		t.Errorf("Head block mismatch: have %d, want %d", head.Number, tt.expHeadBlock)
	}
	if frozen, err := db.(freezer).Ancients(); err != nil {
		t.Errorf("Failed to retrieve ancient count: %v\n", err)
	} else if int(frozen) != tt.expFrozen {
		t.Errorf("Frozen block count mismatch: have %d, want %d", frozen, tt.expFrozen)
	}
}

// TestIssue23496 tests scenario described in https://github.com/ethereum/go-ethereum/pull/23496#issuecomment-926393893
// Credits to @zzyalbert for finding the issue.
//
// Local chain owns these blocks:
// G  B1  B2  B3  B4
// B1: state committed
// B2: snapshot disk layer
// B3: state committed
// B4: head block
//
// Crash happens without fully persisting snapshot and in-memory states,
// chain rewinds itself to the B1 (skip B3 in order to recover snapshot)
// In this case the snapshot layer of B3 is not created because of existent
// state.
func TestIssue23496(t *testing.T) {
	testIssue23496(t, rawdb.HashScheme)
	testIssue23496(t, rawdb.PathScheme)
}

func testIssue23496(t *testing.T, scheme string) {
	// It's hard to follow the test case, visualize the input
	//log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))

	// Create a temporary persistent database
	datadir := t.TempDir()
	ancient := path.Join(datadir, "ancient")

	db, err := rawdb.Open(rawdb.OpenOptions{
		Directory:         datadir,
		AncientsDirectory: ancient,
	})
	if err != nil {
		t.Fatalf("Failed to create persistent database: %v", err)
	}
	defer db.Close() // Might double close, should be fine

	// Initialize a fresh chain
	var (
		gspec = &Genesis{
			Config:  params.TestChainConfig,
			BaseFee: big.NewInt(params.InitialBaseFee),
		}
		engine = ethash.NewFullFaker()
	)
	chain, err := NewBlockChain(db, DefaultCacheConfigWithScheme(scheme), gspec, nil, engine, vm.Config{}, nil, nil)
	if err != nil {
		t.Fatalf("Failed to create chain: %v", err)
	}
	_, blocks, _ := GenerateChainWithGenesis(gspec, engine, 4, func(i int, b *BlockGen) {
		b.SetCoinbase(common.Address{0x02})
		b.SetDifficulty(big.NewInt(1000000))
	})

	// Insert block B1 and commit the state into disk
	if _, err := chain.InsertChain(blocks[:1]); err != nil {
		t.Fatalf("Failed to import canonical chain start: %v", err)
	}
	chain.triedb.Commit(blocks[0].Root(), false)

	// Insert block B2 and commit the snapshot into disk
	if _, err := chain.InsertChain(blocks[1:2]); err != nil {
		t.Fatalf("Failed to import canonical chain start: %v", err)
	}
	if err := chain.snaps.Cap(blocks[1].Root(), 0); err != nil {
		t.Fatalf("Failed to flatten snapshots: %v", err)
	}

	// Insert block B3 and commit the state into disk
	if _, err := chain.InsertChain(blocks[2:3]); err != nil {
		t.Fatalf("Failed to import canonical chain start: %v", err)
	}
	chain.triedb.Commit(blocks[2].Root(), false)

	// Insert the remaining blocks
	if _, err := chain.InsertChain(blocks[3:]); err != nil {
		t.Fatalf("Failed to import canonical chain tail: %v", err)
	}

	// Pull the plug on the database, simulating a hard crash
	chain.triedb.Close()
	db.Close()
	chain.stopWithoutSaving()

	// Start a new blockchain back up and see where the repair leads us
	db, err = rawdb.Open(rawdb.OpenOptions{
		Directory:         datadir,
		AncientsDirectory: ancient,
		Ephemeral:         true,
	})
	if err != nil {
		t.Fatalf("Failed to reopen persistent database: %v", err)
	}
	defer db.Close()

	chain, err = NewBlockChain(db, DefaultCacheConfigWithScheme(scheme), gspec, nil, engine, vm.Config{}, nil, nil)
	if err != nil {
		t.Fatalf("Failed to recreate chain: %v", err)
	}
	defer chain.Stop()

	if head := chain.CurrentHeader(); head.Number.Uint64() != uint64(4) {
		t.Errorf("Head header mismatch: have %d, want %d", head.Number, 4)
	}
	if head := chain.CurrentSnapBlock(); head.Number.Uint64() != uint64(4) {
		t.Errorf("Head fast block mismatch: have %d, want %d", head.Number, uint64(4))
	}
	expHead := uint64(1)
	if scheme == rawdb.PathScheme {
		expHead = uint64(2)
	}
	if head := chain.CurrentBlock(); head.Number.Uint64() != expHead {
		t.Errorf("Head block mismatch: have %d, want %d", head.Number, expHead)
	}

	// Reinsert B2-B4
	if _, err := chain.InsertChain(blocks[1:]); err != nil {
		t.Fatalf("Failed to import canonical chain tail: %v", err)
	}
	if head := chain.CurrentHeader(); head.Number.Uint64() != uint64(4) {
		t.Errorf("Head header mismatch: have %d, want %d", head.Number, 4)
	}
	if head := chain.CurrentSnapBlock(); head.Number.Uint64() != uint64(4) {
		t.Errorf("Head fast block mismatch: have %d, want %d", head.Number, uint64(4))
	}
	if head := chain.CurrentBlock(); head.Number.Uint64() != uint64(4) {
		t.Errorf("Head block mismatch: have %d, want %d", head.Number, uint64(4))
	}
	if layer := chain.Snapshots().Snapshot(blocks[2].Root()); layer == nil {
		t.Error("Failed to regenerate the snapshot of known state")
	}
}
