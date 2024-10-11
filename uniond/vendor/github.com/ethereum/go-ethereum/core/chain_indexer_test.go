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

package core

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"math/rand"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
)

// Runs multiple tests with randomized parameters.
func TestChainIndexerSingle(t *testing.T) {
	for i := 0; i < 10; i++ {
		testChainIndexer(t, 1)
	}
}

// Runs multiple tests with randomized parameters and different number of
// chain backends.
func TestChainIndexerWithChildren(t *testing.T) {
	for i := 2; i < 8; i++ {
		testChainIndexer(t, i)
	}
}

// testChainIndexer runs a test with either a single chain indexer or a chain of
// multiple backends. The section size and required confirmation count parameters
// are randomized.
func testChainIndexer(t *testing.T, count int) {
	db := rawdb.NewMemoryDatabase()
	defer db.Close()

	// Create a chain of indexers and ensure they all report empty
	backends := make([]*testChainIndexBackend, count)
	for i := 0; i < count; i++ {
		var (
			sectionSize = uint64(rand.Intn(100) + 1)
			confirmsReq = uint64(rand.Intn(10))
		)
		backends[i] = &testChainIndexBackend{t: t, processCh: make(chan uint64)}
		backends[i].indexer = NewChainIndexer(db, rawdb.NewTable(db, string([]byte{byte(i)})), backends[i], sectionSize, confirmsReq, 0, fmt.Sprintf("indexer-%d", i))

		if sections, _, _ := backends[i].indexer.Sections(); sections != 0 {
			t.Fatalf("Canonical section count mismatch: have %v, want %v", sections, 0)
		}
		if i > 0 {
			backends[i-1].indexer.AddChildIndexer(backends[i].indexer)
		}
	}
	defer backends[0].indexer.Close() // parent indexer shuts down children
	// notify pings the root indexer about a new head or reorg, then expect
	// processed blocks if a section is processable
	notify := func(headNum, failNum uint64, reorg bool) {
		backends[0].indexer.newHead(headNum, reorg)
		if reorg {
			for _, backend := range backends {
				headNum = backend.reorg(headNum)
				backend.assertSections()
			}
			return
		}
		var cascade bool
		for _, backend := range backends {
			headNum, cascade = backend.assertBlocks(headNum, failNum)
			if !cascade {
				break
			}
			backend.assertSections()
		}
	}
	// inject inserts a new random canonical header into the database directly
	inject := func(number uint64) {
		header := &types.Header{Number: big.NewInt(int64(number)), Extra: big.NewInt(rand.Int63()).Bytes()}
		if number > 0 {
			header.ParentHash = rawdb.ReadCanonicalHash(db, number-1)
		}
		rawdb.WriteHeader(db, header)
		rawdb.WriteCanonicalHash(db, header.Hash(), number)
	}
	// Start indexer with an already existing chain
	for i := uint64(0); i <= 100; i++ {
		inject(i)
	}
	notify(100, 100, false)

	// Add new blocks one by one
	for i := uint64(101); i <= 1000; i++ {
		inject(i)
		notify(i, i, false)
	}
	// Do a reorg
	notify(500, 500, true)

	// Create new fork
	for i := uint64(501); i <= 1000; i++ {
		inject(i)
		notify(i, i, false)
	}
	for i := uint64(1001); i <= 1500; i++ {
		inject(i)
	}
	// Failed processing scenario where less blocks are available than notified
	notify(2000, 1500, false)

	// Notify about a reorg (which could have caused the missing blocks if happened during processing)
	notify(1500, 1500, true)

	// Create new fork
	for i := uint64(1501); i <= 2000; i++ {
		inject(i)
		notify(i, i, false)
	}
}

// testChainIndexBackend implements ChainIndexerBackend
type testChainIndexBackend struct {
	t                          *testing.T
	indexer                    *ChainIndexer
	section, headerCnt, stored uint64
	processCh                  chan uint64
}

// assertSections verifies if a chain indexer has the correct number of section.
func (b *testChainIndexBackend) assertSections() {
	// Keep trying for 3 seconds if it does not match
	var sections uint64
	for i := 0; i < 300; i++ {
		sections, _, _ = b.indexer.Sections()
		if sections == b.stored {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	b.t.Fatalf("Canonical section count mismatch: have %v, want %v", sections, b.stored)
}

// assertBlocks expects processing calls after new blocks have arrived. If the
// failNum < headNum then we are simulating a scenario where a reorg has happened
// after the processing has started and the processing of a section fails.
func (b *testChainIndexBackend) assertBlocks(headNum, failNum uint64) (uint64, bool) {
	var sections uint64
	if headNum >= b.indexer.confirmsReq {
		sections = (headNum + 1 - b.indexer.confirmsReq) / b.indexer.sectionSize
		if sections > b.stored {
			// expect processed blocks
			for expectd := b.stored * b.indexer.sectionSize; expectd < sections*b.indexer.sectionSize; expectd++ {
				if expectd > failNum {
					// rolled back after processing started, no more process calls expected
					// wait until updating is done to make sure that processing actually fails
					var updating bool
					for i := 0; i < 300; i++ {
						b.indexer.lock.Lock()
						updating = b.indexer.knownSections > b.indexer.storedSections
						b.indexer.lock.Unlock()
						if !updating {
							break
						}
						time.Sleep(10 * time.Millisecond)
					}
					if updating {
						b.t.Fatalf("update did not finish")
					}
					sections = expectd / b.indexer.sectionSize
					break
				}
				select {
				case <-time.After(10 * time.Second):
					b.t.Fatalf("Expected processed block #%d, got nothing", expectd)
				case processed := <-b.processCh:
					if processed != expectd {
						b.t.Errorf("Expected processed block #%d, got #%d", expectd, processed)
					}
				}
			}
			b.stored = sections
		}
	}
	if b.stored == 0 {
		return 0, false
	}
	return b.stored*b.indexer.sectionSize - 1, true
}

func (b *testChainIndexBackend) reorg(headNum uint64) uint64 {
	firstChanged := (headNum + 1) / b.indexer.sectionSize
	if firstChanged < b.stored {
		b.stored = firstChanged
	}
	return b.stored * b.indexer.sectionSize
}

func (b *testChainIndexBackend) Reset(ctx context.Context, section uint64, prevHead common.Hash) error {
	b.section = section
	b.headerCnt = 0
	return nil
}

func (b *testChainIndexBackend) Process(ctx context.Context, header *types.Header) error {
	b.headerCnt++
	if b.headerCnt > b.indexer.sectionSize {
		b.t.Error("Processing too many headers")
	}
	//t.processCh <- header.Number.Uint64()
	select {
	case <-time.After(10 * time.Second):
		b.t.Error("Unexpected call to Process")
		// Can't use Fatal since this is not the test's goroutine.
		// Returning error stops the chainIndexer's updateLoop
		return errors.New("Unexpected call to Process")
	case b.processCh <- header.Number.Uint64():
	}
	return nil
}

func (b *testChainIndexBackend) Commit() error {
	if b.headerCnt != b.indexer.sectionSize {
		b.t.Error("Not enough headers processed")
	}
	return nil
}

func (b *testChainIndexBackend) Prune(threshold uint64) error {
	return nil
}
