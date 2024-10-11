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

// Tests that setting the chain head backwards doesn't leave the database in some
// strange state with gaps in the chain, nor with block data dangling in the future.

package core

import (
	"fmt"
	"math/big"
	"path"
	"strings"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/triedb/hashdb"
	"github.com/ethereum/go-ethereum/trie/triedb/pathdb"
)

// rewindTest is a test case for chain rollback upon user request.
type rewindTest struct {
	canonicalBlocks int     // Number of blocks to generate for the canonical chain (heavier)
	sidechainBlocks int     // Number of blocks to generate for the side chain (lighter)
	freezeThreshold uint64  // Block number until which to move things into the freezer
	commitBlock     uint64  // Block number for which to commit the state to disk
	pivotBlock      *uint64 // Pivot block number in case of fast sync

	setheadBlock       uint64 // Block number to set head back to
	expCanonicalBlocks int    // Number of canonical blocks expected to remain in the database (excl. genesis)
	expSidechainBlocks int    // Number of sidechain blocks expected to remain in the database (excl. genesis)
	expFrozen          int    // Number of canonical blocks expected to be in the freezer (incl. genesis)
	expHeadHeader      uint64 // Block number of the expected head header
	expHeadFastBlock   uint64 // Block number of the expected head fast sync block
	expHeadBlock       uint64 // Block number of the expected head full block
}

//nolint:unused
func (tt *rewindTest) dump(crash bool) string {
	buffer := new(strings.Builder)

	fmt.Fprint(buffer, "Chain:\n  G")
	for i := 0; i < tt.canonicalBlocks; i++ {
		fmt.Fprintf(buffer, "->C%d", i+1)
	}
	fmt.Fprint(buffer, " (HEAD)\n")
	if tt.sidechainBlocks > 0 {
		fmt.Fprintf(buffer, "  └")
		for i := 0; i < tt.sidechainBlocks; i++ {
			fmt.Fprintf(buffer, "->S%d", i+1)
		}
		fmt.Fprintf(buffer, "\n")
	}
	fmt.Fprintf(buffer, "\n")

	if tt.canonicalBlocks > int(tt.freezeThreshold) {
		fmt.Fprint(buffer, "Frozen:\n  G")
		for i := 0; i < tt.canonicalBlocks-int(tt.freezeThreshold); i++ {
			fmt.Fprintf(buffer, "->C%d", i+1)
		}
		fmt.Fprintf(buffer, "\n\n")
	} else {
		fmt.Fprintf(buffer, "Frozen: none\n")
	}
	fmt.Fprintf(buffer, "Commit: G")
	if tt.commitBlock > 0 {
		fmt.Fprintf(buffer, ", C%d", tt.commitBlock)
	}
	fmt.Fprint(buffer, "\n")

	if tt.pivotBlock == nil {
		fmt.Fprintf(buffer, "Pivot : none\n")
	} else {
		fmt.Fprintf(buffer, "Pivot : C%d\n", *tt.pivotBlock)
	}
	if crash {
		fmt.Fprintf(buffer, "\nCRASH\n\n")
	} else {
		fmt.Fprintf(buffer, "\nSetHead(%d)\n\n", tt.setheadBlock)
	}
	fmt.Fprintf(buffer, "------------------------------\n\n")

	if tt.expFrozen > 0 {
		fmt.Fprint(buffer, "Expected in freezer:\n  G")
		for i := 0; i < tt.expFrozen-1; i++ {
			fmt.Fprintf(buffer, "->C%d", i+1)
		}
		fmt.Fprintf(buffer, "\n\n")
	}
	if tt.expFrozen > 0 {
		if tt.expFrozen >= tt.expCanonicalBlocks {
			fmt.Fprintf(buffer, "Expected in leveldb: none\n")
		} else {
			fmt.Fprintf(buffer, "Expected in leveldb:\n  C%d)", tt.expFrozen-1)
			for i := tt.expFrozen - 1; i < tt.expCanonicalBlocks; i++ {
				fmt.Fprintf(buffer, "->C%d", i+1)
			}
			fmt.Fprint(buffer, "\n")
			if tt.expSidechainBlocks > tt.expFrozen {
				fmt.Fprintf(buffer, "  └")
				for i := tt.expFrozen - 1; i < tt.expSidechainBlocks; i++ {
					fmt.Fprintf(buffer, "->S%d", i+1)
				}
				fmt.Fprintf(buffer, "\n")
			}
		}
	} else {
		fmt.Fprint(buffer, "Expected in leveldb:\n  G")
		for i := tt.expFrozen; i < tt.expCanonicalBlocks; i++ {
			fmt.Fprintf(buffer, "->C%d", i+1)
		}
		fmt.Fprint(buffer, "\n")
		if tt.expSidechainBlocks > tt.expFrozen {
			fmt.Fprintf(buffer, "  └")
			for i := tt.expFrozen; i < tt.expSidechainBlocks; i++ {
				fmt.Fprintf(buffer, "->S%d", i+1)
			}
			fmt.Fprintf(buffer, "\n")
		}
	}
	fmt.Fprintf(buffer, "\n")
	fmt.Fprintf(buffer, "Expected head header    : C%d\n", tt.expHeadHeader)
	fmt.Fprintf(buffer, "Expected head fast block: C%d\n", tt.expHeadFastBlock)
	if tt.expHeadBlock == 0 {
		fmt.Fprintf(buffer, "Expected head block     : G\n")
	} else {
		fmt.Fprintf(buffer, "Expected head block     : C%d\n", tt.expHeadBlock)
	}
	return buffer.String()
}

// Tests a sethead for a short canonical chain where a recent block was already
// committed to disk and then the sethead called. In this case we expect the full
// chain to be rolled back to the committed block. Everything above the sethead
// point should be deleted. In between the committed block and the requested head
// the data can remain as "fast sync" data to avoid redownloading it.
func TestShortSetHead(t *testing.T)              { testShortSetHead(t, false) }
func TestShortSetHeadWithSnapshots(t *testing.T) { testShortSetHead(t, true) }

func testShortSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : none
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 0,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a short canonical chain where the fast sync pivot point was
// already committed, after which sethead was called. In this case we expect the
// chain to behave like in full sync mode, rolling back to the committed block
// Everything above the sethead point should be deleted. In between the committed
// block and the requested head the data can remain as "fast sync" data to avoid
// redownloading it.
func TestShortSnapSyncedSetHead(t *testing.T)              { testShortSnapSyncedSetHead(t, false) }
func TestShortSnapSyncedSetHeadWithSnapshots(t *testing.T) { testShortSnapSyncedSetHead(t, true) }

func testShortSnapSyncedSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : C4
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 0,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a short canonical chain where the fast sync pivot point was
// not yet committed, but sethead was called. In this case we expect the chain to
// detect that it was fast syncing and delete everything from the new head, since
// we can just pick up fast syncing from there. The head full block should be set
// to the genesis.
func TestShortSnapSyncingSetHead(t *testing.T)              { testShortSnapSyncingSetHead(t, false) }
func TestShortSnapSyncingSetHeadWithSnapshots(t *testing.T) { testShortSnapSyncingSetHead(t, true) }

func testShortSnapSyncingSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//
	// Frozen: none
	// Commit: G
	// Pivot : C4
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 0,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a shorter side chain, where a
// recent block was already committed to disk and then sethead was called. In this
// test scenario the side chain is below the committed block. In this case we expect
// the canonical full chain to be rolled back to the committed block. Everything
// above the sethead point should be deleted. In between the committed block and
// the requested head the data can remain as "fast sync" data to avoid redownloading
// it. The side chain should be left alone as it was shorter.
func TestShortOldForkedSetHead(t *testing.T)              { testShortOldForkedSetHead(t, false) }
func TestShortOldForkedSetHeadWithSnapshots(t *testing.T) { testShortOldForkedSetHead(t, true) }

func testShortOldForkedSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : none
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 3,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a shorter side chain, where
// the fast sync pivot point was already committed to disk and then sethead was
// called. In this test scenario the side chain is below the committed block. In
// this case we expect the canonical full chain to be rolled back to the committed
// block. Everything above the sethead point should be deleted. In between the
// committed block and the requested head the data can remain as "fast sync" data
// to avoid redownloading it. The side chain should be left alone as it was shorter.
func TestShortOldForkedSnapSyncedSetHead(t *testing.T) {
	testShortOldForkedSnapSyncedSetHead(t, false)
}
func TestShortOldForkedSnapSyncedSetHeadWithSnapshots(t *testing.T) {
	testShortOldForkedSnapSyncedSetHead(t, true)
}

func testShortOldForkedSnapSyncedSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : C4
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 3,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a shorter side chain, where
// the fast sync pivot point was not yet committed, but sethead was called. In this
// test scenario the side chain is below the committed block. In this case we expect
// the chain to detect that it was fast syncing and delete everything from the new
// head, since we can just pick up fast syncing from there. The head full block
// should be set to the genesis.
func TestShortOldForkedSnapSyncingSetHead(t *testing.T) {
	testShortOldForkedSnapSyncingSetHead(t, false)
}
func TestShortOldForkedSnapSyncingSetHeadWithSnapshots(t *testing.T) {
	testShortOldForkedSnapSyncingSetHead(t, true)
}

func testShortOldForkedSnapSyncingSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3
	//
	// Frozen: none
	// Commit: G
	// Pivot : C4
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 3,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a shorter side chain, where a
// recent block was already committed to disk and then sethead was called. In this
// test scenario the side chain reaches above the committed block. In this case we
// expect the canonical full chain to be rolled back to the committed block. All
// data above the sethead point should be deleted. In between the committed block
// and the requested head the data can remain as "fast sync" data to avoid having
// to redownload it. The side chain should be truncated to the head set.
//
// The side chain could be left to be if the fork point was before the new head
// we are deleting to, but it would be exceedingly hard to detect that case and
// properly handle it, so we'll trade extra work in exchange for simpler code.
func TestShortNewlyForkedSetHead(t *testing.T)              { testShortNewlyForkedSetHead(t, false) }
func TestShortNewlyForkedSetHeadWithSnapshots(t *testing.T) { testShortNewlyForkedSetHead(t, true) }

func testShortNewlyForkedSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : none
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3->S4->S5->S6->S7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    10,
		sidechainBlocks:    8,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 7,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a shorter side chain, where
// the fast sync pivot point was already committed to disk and then sethead was
// called. In this case we expect the canonical full chain to be rolled back to
// between the committed block and the requested head the data can remain as
// "fast sync" data to avoid having to redownload it. The side chain should be
// truncated to the head set.
//
// The side chain could be left to be if the fork point was before the new head
// we are deleting to, but it would be exceedingly hard to detect that case and
// properly handle it, so we'll trade extra work in exchange for simpler code.
func TestShortNewlyForkedSnapSyncedSetHead(t *testing.T) {
	testShortNewlyForkedSnapSyncedSetHead(t, false)
}
func TestShortNewlyForkedSnapSyncedSetHeadWithSnapshots(t *testing.T) {
	testShortNewlyForkedSnapSyncedSetHead(t, true)
}

func testShortNewlyForkedSnapSyncedSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : C4
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3->S4->S5->S6->S7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    10,
		sidechainBlocks:    8,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 7,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a shorter side chain, where
// the fast sync pivot point was not yet committed, but sethead was called. In
// this test scenario the side chain reaches above the committed block. In this
// case we expect the chain to detect that it was fast syncing and delete
// everything from the new head, since we can just pick up fast syncing from
// there.
//
// The side chain could be left to be if the fork point was before the new head
// we are deleting to, but it would be exceedingly hard to detect that case and
// properly handle it, so we'll trade extra work in exchange for simpler code.
func TestShortNewlyForkedSnapSyncingSetHead(t *testing.T) {
	testShortNewlyForkedSnapSyncingSetHead(t, false)
}
func TestShortNewlyForkedSnapSyncingSetHeadWithSnapshots(t *testing.T) {
	testShortNewlyForkedSnapSyncingSetHead(t, true)
}

func testShortNewlyForkedSnapSyncingSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8
	//
	// Frozen: none
	// Commit: G
	// Pivot : C4
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3->S4->S5->S6->S7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    10,
		sidechainBlocks:    8,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 7,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a longer side chain, where a
// recent block was already committed to disk and then sethead was called. In this
// case we expect the canonical full chain to be rolled back to the committed block.
// All data above the sethead point should be deleted. In between the committed
// block and the requested head the data can remain as "fast sync" data to avoid
// having to redownload it. The side chain should be truncated to the head set.
//
// The side chain could be left to be if the fork point was before the new head
// we are deleting to, but it would be exceedingly hard to detect that case and
// properly handle it, so we'll trade extra work in exchange for simpler code.
func TestShortReorgedSetHead(t *testing.T)              { testShortReorgedSetHead(t, false) }
func TestShortReorgedSetHeadWithSnapshots(t *testing.T) { testShortReorgedSetHead(t, true) }

func testShortReorgedSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : none
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3->S4->S5->S6->S7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    10,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 7,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a longer side chain, where
// the fast sync pivot point was already committed to disk and then sethead was
// called. In this case we expect the canonical full chain to be rolled back to
// the committed block. All data above the sethead point should be deleted. In
// between the committed block and the requested head the data can remain as
// "fast sync" data to avoid having to redownload it. The side chain should be
// truncated to the head set.
//
// The side chain could be left to be if the fork point was before the new head
// we are deleting to, but it would be exceedingly hard to detect that case and
// properly handle it, so we'll trade extra work in exchange for simpler code.
func TestShortReorgedSnapSyncedSetHead(t *testing.T) {
	testShortReorgedSnapSyncedSetHead(t, false)
}
func TestShortReorgedSnapSyncedSetHeadWithSnapshots(t *testing.T) {
	testShortReorgedSnapSyncedSetHead(t, true)
}

func testShortReorgedSnapSyncedSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Frozen: none
	// Commit: G, C4
	// Pivot : C4
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3->S4->S5->S6->S7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    10,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 7,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a short canonical chain and a longer side chain, where
// the fast sync pivot point was not yet committed, but sethead was called. In
// this case we expect the chain to detect that it was fast syncing and delete
// everything from the new head, since we can just pick up fast syncing from
// there.
//
// The side chain could be left to be if the fork point was before the new head
// we are deleting to, but it would be exceedingly hard to detect that case and
// properly handle it, so we'll trade extra work in exchange for simpler code.
func TestShortReorgedSnapSyncingSetHead(t *testing.T) {
	testShortReorgedSnapSyncingSetHead(t, false)
}
func TestShortReorgedSnapSyncingSetHeadWithSnapshots(t *testing.T) {
	testShortReorgedSnapSyncingSetHead(t, true)
}

func testShortReorgedSnapSyncingSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8 (HEAD)
	//   └->S1->S2->S3->S4->S5->S6->S7->S8->S9->S10
	//
	// Frozen: none
	// Commit: G
	// Pivot : C4
	//
	// SetHead(7)
	//
	// ------------------------------
	//
	// Expected in leveldb:
	//   G->C1->C2->C3->C4->C5->C6->C7
	//   └->S1->S2->S3->S4->S5->S6->S7
	//
	// Expected head header    : C7
	// Expected head fast block: C7
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    8,
		sidechainBlocks:    10,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       7,
		expCanonicalBlocks: 7,
		expSidechainBlocks: 7,
		expFrozen:          0,
		expHeadHeader:      7,
		expHeadFastBlock:   7,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks where a recent
// block - newer than the ancient limit - was already committed to disk and then
// sethead was called. In this case we expect the full chain to be rolled back
// to the committed block. Everything above the sethead point should be deleted.
// In between the committed block and the requested head the data can remain as
// "fast sync" data to avoid redownloading it.
func TestLongShallowSetHead(t *testing.T)              { testLongShallowSetHead(t, false) }
func TestLongShallowSetHeadWithSnapshots(t *testing.T) { testLongShallowSetHead(t, true) }

func testLongShallowSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : none
	//
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks where a recent
// block - older than the ancient limit - was already committed to disk and then
// sethead was called. In this case we expect the full chain to be rolled back
// to the committed block. Since the ancient limit was underflown, everything
// needs to be deleted onwards to avoid creating a gap.
func TestLongDeepSetHead(t *testing.T)              { testLongDeepSetHead(t, false) }
func TestLongDeepSetHeadWithSnapshots(t *testing.T) { testLongDeepSetHead(t, true) }

func testLongDeepSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : none
	//
	// SetHead(6)
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
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       6,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks where the fast
// sync pivot point - newer than the ancient limit - was already committed, after
// which sethead was called. In this case we expect the full chain to be rolled
// back to the committed block. Everything above the sethead point should be
// deleted. In between the committed block and the requested head the data can
// remain as "fast sync" data to avoid redownloading it.
func TestLongSnapSyncedShallowSetHead(t *testing.T) {
	testLongSnapSyncedShallowSetHead(t, false)
}
func TestLongSnapSyncedShallowSetHeadWithSnapshots(t *testing.T) {
	testLongSnapSyncedShallowSetHead(t, true)
}

func testLongSnapSyncedShallowSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks where the fast
// sync pivot point - older than the ancient limit - was already committed, after
// which sethead was called. In this case we expect the full chain to be rolled
// back to the committed block. Since the ancient limit was underflown, everything
// needs to be deleted onwards to avoid creating a gap.
func TestLongSnapSyncedDeepSetHead(t *testing.T)              { testLongSnapSyncedDeepSetHead(t, false) }
func TestLongSnapSyncedDeepSetHeadWithSnapshots(t *testing.T) { testLongSnapSyncedDeepSetHead(t, true) }

func testLongSnapSyncedDeepSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G, C4
	// Pivot : C4
	//
	// SetHead(6)
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
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks where the fast
// sync pivot point - newer than the ancient limit - was not yet committed, but
// sethead was called. In this case we expect the chain to detect that it was fast
// syncing and delete everything from the new head, since we can just pick up fast
// syncing from there.
func TestLongSnapSyncingShallowSetHead(t *testing.T) {
	testLongSnapSyncingShallowSetHead(t, false)
}
func TestLongSnapSyncingShallowSetHeadWithSnapshots(t *testing.T) {
	testLongSnapSyncingShallowSetHead(t, true)
}

func testLongSnapSyncingShallowSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18 (HEAD)
	//
	// Frozen:
	//   G->C1->C2
	//
	// Commit: G
	// Pivot : C4
	//
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks where the fast
// sync pivot point - older than the ancient limit - was not yet committed, but
// sethead was called. In this case we expect the chain to detect that it was fast
// syncing and delete everything from the new head, since we can just pick up fast
// syncing from there.
func TestLongSnapSyncingDeepSetHead(t *testing.T) {
	testLongSnapSyncingDeepSetHead(t, false)
}
func TestLongSnapSyncingDeepSetHeadWithSnapshots(t *testing.T) {
	testLongSnapSyncingDeepSetHead(t, true)
}

func testLongSnapSyncingDeepSetHead(t *testing.T, snapshots bool) {
	// Chain:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8->C9->C10->C11->C12->C13->C14->C15->C16->C17->C18->C19->C20->C21->C22->C23->C24 (HEAD)
	//
	// Frozen:
	//   G->C1->C2->C3->C4->C5->C6->C7->C8
	//
	// Commit: G
	// Pivot : C4
	//
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    0,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          7,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter side
// chain, where a recent block - newer than the ancient limit - was already committed
// to disk and then sethead was called. In this case we expect the canonical full
// chain to be rolled back to the committed block. Everything above the sethead point
// should be deleted. In between the committed block and the requested head the data
// can remain as "fast sync" data to avoid redownloading it. The side chain is nuked
// by the freezer.
func TestLongOldForkedShallowSetHead(t *testing.T) {
	testLongOldForkedShallowSetHead(t, false)
}
func TestLongOldForkedShallowSetHeadWithSnapshots(t *testing.T) {
	testLongOldForkedShallowSetHead(t, true)
}

func testLongOldForkedShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter side
// chain, where a recent block - older than the ancient limit - was already committed
// to disk and then sethead was called. In this case we expect the canonical full
// chain to be rolled back to the committed block. Since the ancient limit was
// underflown, everything needs to be deleted onwards to avoid creating a gap. The
// side chain is nuked by the freezer.
func TestLongOldForkedDeepSetHead(t *testing.T)              { testLongOldForkedDeepSetHead(t, false) }
func TestLongOldForkedDeepSetHeadWithSnapshots(t *testing.T) { testLongOldForkedDeepSetHead(t, true) }

func testLongOldForkedDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
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
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       6,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was already committed to disk and then sethead was called. In this test scenario
// the side chain is below the committed block. In this case we expect the canonical
// full chain to be rolled back to the committed block. Everything above the
// sethead point should be deleted. In between the committed block and the
// requested head the data can remain as "fast sync" data to avoid redownloading
// it. The side chain is nuked by the freezer.
func TestLongOldForkedSnapSyncedShallowSetHead(t *testing.T) {
	testLongOldForkedSnapSyncedShallowSetHead(t, false)
}
func TestLongOldForkedSnapSyncedShallowSetHeadWithSnapshots(t *testing.T) {
	testLongOldForkedSnapSyncedShallowSetHead(t, true)
}

func testLongOldForkedSnapSyncedShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was already committed to disk and then sethead was called. In this test scenario
// the side chain is below the committed block. In this case we expect the canonical
// full chain to be rolled back to the committed block. Since the ancient limit was
// underflown, everything needs to be deleted onwards to avoid creating a gap. The
// side chain is nuked by the freezer.
func TestLongOldForkedSnapSyncedDeepSetHead(t *testing.T) {
	testLongOldForkedSnapSyncedDeepSetHead(t, false)
}
func TestLongOldForkedSnapSyncedDeepSetHeadWithSnapshots(t *testing.T) {
	testLongOldForkedSnapSyncedDeepSetHead(t, true)
}

func testLongOldForkedSnapSyncedDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was not yet committed, but sethead was called. In this test scenario the side
// chain is below the committed block. In this case we expect the chain to detect
// that it was fast syncing and delete everything from the new head, since we can
// just pick up fast syncing from there. The side chain is completely nuked by the
// freezer.
func TestLongOldForkedSnapSyncingShallowSetHead(t *testing.T) {
	testLongOldForkedSnapSyncingShallowSetHead(t, false)
}
func TestLongOldForkedSnapSyncingShallowSetHeadWithSnapshots(t *testing.T) {
	testLongOldForkedSnapSyncingShallowSetHead(t, true)
}

func testLongOldForkedSnapSyncingShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was not yet committed, but sethead was called. In this test scenario the side
// chain is below the committed block. In this case we expect the chain to detect
// that it was fast syncing and delete everything from the new head, since we can
// just pick up fast syncing from there. The side chain is completely nuked by the
// freezer.
func TestLongOldForkedSnapSyncingDeepSetHead(t *testing.T) {
	testLongOldForkedSnapSyncingDeepSetHead(t, false)
}
func TestLongOldForkedSnapSyncingDeepSetHeadWithSnapshots(t *testing.T) {
	testLongOldForkedSnapSyncingDeepSetHead(t, true)
}

func testLongOldForkedSnapSyncingDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    3,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          7,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where a recent block - newer than the ancient limit - was already
// committed to disk and then sethead was called. In this test scenario the side
// chain is above the committed block. In this case the freezer will delete the
// sidechain since it's dangling, reverting to TestLongShallowSetHead.
func TestLongNewerForkedShallowSetHead(t *testing.T) {
	testLongNewerForkedShallowSetHead(t, false)
}
func TestLongNewerForkedShallowSetHeadWithSnapshots(t *testing.T) {
	testLongNewerForkedShallowSetHead(t, true)
}

func testLongNewerForkedShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where a recent block - older than the ancient limit - was already
// committed to disk and then sethead was called. In this test scenario the side
// chain is above the committed block. In this case the freezer will delete the
// sidechain since it's dangling, reverting to TestLongDeepSetHead.
func TestLongNewerForkedDeepSetHead(t *testing.T) {
	testLongNewerForkedDeepSetHead(t, false)
}
func TestLongNewerForkedDeepSetHeadWithSnapshots(t *testing.T) {
	testLongNewerForkedDeepSetHead(t, true)
}

func testLongNewerForkedDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
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
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       6,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was already committed to disk and then sethead was called. In this test scenario
// the side chain is above the committed block. In this case the freezer will delete
// the sidechain since it's dangling, reverting to TestLongSnapSyncedShallowSetHead.
func TestLongNewerForkedSnapSyncedShallowSetHead(t *testing.T) {
	testLongNewerForkedSnapSyncedShallowSetHead(t, false)
}
func TestLongNewerForkedSnapSyncedShallowSetHeadWithSnapshots(t *testing.T) {
	testLongNewerForkedSnapSyncedShallowSetHead(t, true)
}

func testLongNewerForkedSnapSyncedShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was already committed to disk and then sethead was called. In this test scenario
// the side chain is above the committed block. In this case the freezer will delete
// the sidechain since it's dangling, reverting to TestLongSnapSyncedDeepSetHead.
func TestLongNewerForkedSnapSyncedDeepSetHead(t *testing.T) {
	testLongNewerForkedSnapSyncedDeepSetHead(t, false)
}
func TestLongNewerForkedSnapSyncedDeepSetHeadWithSnapshots(t *testing.T) {
	testLongNewerForkedSnapSyncedDeepSetHead(t, true)
}

func testLongNewerForkedSnapSyncedDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
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
	// Expected head block     : C
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was not yet committed, but sethead was called. In this test scenario the side
// chain is above the committed block. In this case the freezer will delete the
// sidechain since it's dangling, reverting to TestLongSnapSyncinghallowSetHead.
func TestLongNewerForkedSnapSyncingShallowSetHead(t *testing.T) {
	testLongNewerForkedSnapSyncingShallowSetHead(t, false)
}
func TestLongNewerForkedSnapSyncingShallowSetHeadWithSnapshots(t *testing.T) {
	testLongNewerForkedSnapSyncingShallowSetHead(t, true)
}

func testLongNewerForkedSnapSyncingShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a shorter
// side chain, where the fast sync pivot point - older than the ancient limit -
// was not yet committed, but sethead was called. In this test scenario the side
// chain is above the committed block. In this case the freezer will delete the
// sidechain since it's dangling, reverting to TestLongSnapSyncingDeepSetHead.
func TestLongNewerForkedSnapSyncingDeepSetHead(t *testing.T) {
	testLongNewerForkedSnapSyncingDeepSetHead(t, false)
}
func TestLongNewerForkedSnapSyncingDeepSetHeadWithSnapshots(t *testing.T) {
	testLongNewerForkedSnapSyncingDeepSetHead(t, true)
}

func testLongNewerForkedSnapSyncingDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    12,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          7,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a longer side
// chain, where a recent block - newer than the ancient limit - was already committed
// to disk and then sethead was called. In this case the freezer will delete the
// sidechain since it's dangling, reverting to TestLongShallowSetHead.
func TestLongReorgedShallowSetHead(t *testing.T)              { testLongReorgedShallowSetHead(t, false) }
func TestLongReorgedShallowSetHeadWithSnapshots(t *testing.T) { testLongReorgedShallowSetHead(t, true) }

func testLongReorgedShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a longer side
// chain, where a recent block - older than the ancient limit - was already committed
// to disk and then sethead was called. In this case the freezer will delete the
// sidechain since it's dangling, reverting to TestLongDeepSetHead.
func TestLongReorgedDeepSetHead(t *testing.T)              { testLongReorgedDeepSetHead(t, false) }
func TestLongReorgedDeepSetHeadWithSnapshots(t *testing.T) { testLongReorgedDeepSetHead(t, true) }

func testLongReorgedDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
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
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         nil,
		setheadBlock:       6,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a longer
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was already committed to disk and then sethead was called. In this case the
// freezer will delete the sidechain since it's dangling, reverting to
// TestLongSnapSyncedShallowSetHead.
func TestLongReorgedSnapSyncedShallowSetHead(t *testing.T) {
	testLongReorgedSnapSyncedShallowSetHead(t, false)
}
func TestLongReorgedSnapSyncedShallowSetHeadWithSnapshots(t *testing.T) {
	testLongReorgedSnapSyncedShallowSetHead(t, true)
}

func testLongReorgedSnapSyncedShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : C4
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a longer
// side chain, where the fast sync pivot point - older than the ancient limit -
// was already committed to disk and then sethead was called. In this case the
// freezer will delete the sidechain since it's dangling, reverting to
// TestLongSnapSyncedDeepSetHead.
func TestLongReorgedSnapSyncedDeepSetHead(t *testing.T) {
	testLongReorgedSnapSyncedDeepSetHead(t, false)
}
func TestLongReorgedSnapSyncedDeepSetHeadWithSnapshots(t *testing.T) {
	testLongReorgedSnapSyncedDeepSetHead(t, true)
}

func testLongReorgedSnapSyncedDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
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
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        4,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 4,
		expSidechainBlocks: 0,
		expFrozen:          5,
		expHeadHeader:      4,
		expHeadFastBlock:   4,
		expHeadBlock:       4,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a longer
// side chain, where the fast sync pivot point - newer than the ancient limit -
// was not yet committed, but sethead was called. In this case we expect the
// chain to detect that it was fast syncing and delete everything from the new
// head, since we can just pick up fast syncing from there. The side chain is
// completely nuked by the freezer.
func TestLongReorgedSnapSyncingShallowSetHead(t *testing.T) {
	testLongReorgedSnapSyncingShallowSetHead(t, false)
}
func TestLongReorgedSnapSyncingShallowSetHeadWithSnapshots(t *testing.T) {
	testLongReorgedSnapSyncingShallowSetHead(t, true)
}

func testLongReorgedSnapSyncingShallowSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2
	//
	// Expected in leveldb:
	//   C2)->C3->C4->C5->C6
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    18,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          3,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       0,
	}, snapshots)
}

// Tests a sethead for a long canonical chain with frozen blocks and a longer
// side chain, where the fast sync pivot point - older than the ancient limit -
// was not yet committed, but sethead was called. In this case we expect the
// chain to detect that it was fast syncing and delete everything from the new
// head, since we can just pick up fast syncing from there. The side chain is
// completely nuked by the freezer.
func TestLongReorgedSnapSyncingDeepSetHead(t *testing.T) {
	testLongReorgedSnapSyncingDeepSetHead(t, false)
}
func TestLongReorgedSnapSyncingDeepSetHeadWithSnapshots(t *testing.T) {
	testLongReorgedSnapSyncingDeepSetHead(t, true)
}

func testLongReorgedSnapSyncingDeepSetHead(t *testing.T, snapshots bool) {
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
	// SetHead(6)
	//
	// ------------------------------
	//
	// Expected in freezer:
	//   G->C1->C2->C3->C4->C5->C6
	//
	// Expected in leveldb: none
	//
	// Expected head header    : C6
	// Expected head fast block: C6
	// Expected head block     : G
	testSetHead(t, &rewindTest{
		canonicalBlocks:    24,
		sidechainBlocks:    26,
		freezeThreshold:    16,
		commitBlock:        0,
		pivotBlock:         uint64ptr(4),
		setheadBlock:       6,
		expCanonicalBlocks: 6,
		expSidechainBlocks: 0,
		expFrozen:          7,
		expHeadHeader:      6,
		expHeadFastBlock:   6,
		expHeadBlock:       0,
	}, snapshots)
}

func testSetHead(t *testing.T, tt *rewindTest, snapshots bool) {
	for _, scheme := range []string{rawdb.HashScheme, rawdb.PathScheme} {
		testSetHeadWithScheme(t, tt, snapshots, scheme)
	}
}

func testSetHeadWithScheme(t *testing.T, tt *rewindTest, snapshots bool, scheme string) {
	// It's hard to follow the test case, visualize the input
	// log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))
	// fmt.Println(tt.dump(false))

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
	defer db.Close()

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
			SnapshotLimit:  0, // Disable snapshot
			StateScheme:    scheme,
		}
	)
	if snapshots {
		config.SnapshotLimit = 256
		config.SnapshotWait = true
	}
	chain, err := NewBlockChain(db, config, gspec, nil, engine, vm.Config{}, nil, nil)
	if err != nil {
		t.Fatalf("Failed to create chain: %v", err)
	}
	defer chain.Stop()

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
		chain.triedb.Commit(canonblocks[tt.commitBlock-1].Root(), false)
		if snapshots {
			if err := chain.snaps.Cap(canonblocks[tt.commitBlock-1].Root(), 0); err != nil {
				t.Fatalf("Failed to flatten snapshots: %v", err)
			}
		}
	}
	if _, err := chain.InsertChain(canonblocks[tt.commitBlock:]); err != nil {
		t.Fatalf("Failed to import canonical chain tail: %v", err)
	}
	// Reopen the trie database without persisting in-memory dirty nodes.
	chain.triedb.Close()
	dbconfig := &trie.Config{}
	if scheme == rawdb.PathScheme {
		dbconfig.PathDB = pathdb.Defaults
	} else {
		dbconfig.HashDB = hashdb.Defaults
	}
	chain.triedb = trie.NewDatabase(chain.db, dbconfig)
	chain.stateCache = state.NewDatabaseWithNodeDB(chain.db, chain.triedb)

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
	// Set the head of the chain back to the requested number
	chain.SetHead(tt.setheadBlock)

	// Iterate over all the remaining blocks and ensure there are no gaps
	verifyNoGaps(t, chain, true, canonblocks)
	verifyNoGaps(t, chain, false, sideblocks)
	verifyCutoff(t, chain, true, canonblocks, tt.expCanonicalBlocks)
	verifyCutoff(t, chain, false, sideblocks, tt.expSidechainBlocks)

	if head := chain.CurrentHeader(); head.Number.Uint64() != tt.expHeadHeader {
		t.Errorf("Head header mismatch: have %d, want %d", head.Number, tt.expHeadHeader)
	}
	if head := chain.CurrentSnapBlock(); head.Number.Uint64() != tt.expHeadFastBlock {
		t.Errorf("Head fast block mismatch: have %d, want %d", head.Number, tt.expHeadFastBlock)
	}
	if head := chain.CurrentBlock(); head.Number.Uint64() != tt.expHeadBlock {
		t.Errorf("Head block mismatch: have %d, want %d", head.Number, tt.expHeadBlock)
	}
	if frozen, err := db.(freezer).Ancients(); err != nil {
		t.Errorf("Failed to retrieve ancient count: %v\n", err)
	} else if int(frozen) != tt.expFrozen {
		t.Errorf("Frozen block count mismatch: have %d, want %d", frozen, tt.expFrozen)
	}
}

// verifyNoGaps checks that there are no gaps after the initial set of blocks in
// the database and errors if found.
func verifyNoGaps(t *testing.T, chain *BlockChain, canonical bool, inserted types.Blocks) {
	t.Helper()

	var end uint64
	for i := uint64(0); i <= uint64(len(inserted)); i++ {
		header := chain.GetHeaderByNumber(i)
		if header == nil && end == 0 {
			end = i
		}
		if header != nil && end > 0 {
			if canonical {
				t.Errorf("Canonical header gap between #%d-#%d", end, i-1)
			} else {
				t.Errorf("Sidechain header gap between #%d-#%d", end, i-1)
			}
			end = 0 // Reset for further gap detection
		}
	}
	end = 0
	for i := uint64(0); i <= uint64(len(inserted)); i++ {
		block := chain.GetBlockByNumber(i)
		if block == nil && end == 0 {
			end = i
		}
		if block != nil && end > 0 {
			if canonical {
				t.Errorf("Canonical block gap between #%d-#%d", end, i-1)
			} else {
				t.Errorf("Sidechain block gap between #%d-#%d", end, i-1)
			}
			end = 0 // Reset for further gap detection
		}
	}
	end = 0
	for i := uint64(1); i <= uint64(len(inserted)); i++ {
		receipts := chain.GetReceiptsByHash(inserted[i-1].Hash())
		if receipts == nil && end == 0 {
			end = i
		}
		if receipts != nil && end > 0 {
			if canonical {
				t.Errorf("Canonical receipt gap between #%d-#%d", end, i-1)
			} else {
				t.Errorf("Sidechain receipt gap between #%d-#%d", end, i-1)
			}
			end = 0 // Reset for further gap detection
		}
	}
}

// verifyCutoff checks that there are no chain data available in the chain after
// the specified limit, but that it is available before.
func verifyCutoff(t *testing.T, chain *BlockChain, canonical bool, inserted types.Blocks, head int) {
	t.Helper()

	for i := 1; i <= len(inserted); i++ {
		if i <= head {
			if header := chain.GetHeader(inserted[i-1].Hash(), uint64(i)); header == nil {
				if canonical {
					t.Errorf("Canonical header   #%2d [%x...] missing before cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				} else {
					t.Errorf("Sidechain header   #%2d [%x...] missing before cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				}
			}
			if block := chain.GetBlock(inserted[i-1].Hash(), uint64(i)); block == nil {
				if canonical {
					t.Errorf("Canonical block    #%2d [%x...] missing before cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				} else {
					t.Errorf("Sidechain block    #%2d [%x...] missing before cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				}
			}
			if receipts := chain.GetReceiptsByHash(inserted[i-1].Hash()); receipts == nil {
				if canonical {
					t.Errorf("Canonical receipts #%2d [%x...] missing before cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				} else {
					t.Errorf("Sidechain receipts #%2d [%x...] missing before cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				}
			}
		} else {
			if header := chain.GetHeader(inserted[i-1].Hash(), uint64(i)); header != nil {
				if canonical {
					t.Errorf("Canonical header   #%2d [%x...] present after cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				} else {
					t.Errorf("Sidechain header   #%2d [%x...] present after cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				}
			}
			if block := chain.GetBlock(inserted[i-1].Hash(), uint64(i)); block != nil {
				if canonical {
					t.Errorf("Canonical block    #%2d [%x...] present after cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				} else {
					t.Errorf("Sidechain block    #%2d [%x...] present after cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				}
			}
			if receipts := chain.GetReceiptsByHash(inserted[i-1].Hash()); receipts != nil {
				if canonical {
					t.Errorf("Canonical receipts #%2d [%x...] present after cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				} else {
					t.Errorf("Sidechain receipts #%2d [%x...] present after cap %d", inserted[i-1].Number(), inserted[i-1].Hash().Bytes()[:3], head)
				}
			}
		}
	}
}

// uint64ptr is a weird helper to allow 1-line constant pointer creation.
func uint64ptr(n uint64) *uint64 {
	return &n
}
