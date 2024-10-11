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

package bloombits

import (
	"context"
	"math/rand"
	"sync/atomic"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
)

const testSectionSize = 4096

// Tests that wildcard filter rules (nil) can be specified and are handled well.
func TestMatcherWildcards(t *testing.T) {
	t.Parallel()
	matcher := NewMatcher(testSectionSize, [][][]byte{
		{common.Address{}.Bytes(), common.Address{0x01}.Bytes()}, // Default address is not a wildcard
		{common.Hash{}.Bytes(), common.Hash{0x01}.Bytes()},       // Default hash is not a wildcard
		{common.Hash{0x01}.Bytes()},                              // Plain rule, sanity check
		{common.Hash{0x01}.Bytes(), nil},                         // Wildcard suffix, drop rule
		{nil, common.Hash{0x01}.Bytes()},                         // Wildcard prefix, drop rule
		{nil, nil},                                               // Wildcard combo, drop rule
		{},                                                       // Inited wildcard rule, drop rule
		nil,                                                      // Proper wildcard rule, drop rule
	})
	if len(matcher.filters) != 3 {
		t.Fatalf("filter system size mismatch: have %d, want %d", len(matcher.filters), 3)
	}
	if len(matcher.filters[0]) != 2 {
		t.Fatalf("address clause size mismatch: have %d, want %d", len(matcher.filters[0]), 2)
	}
	if len(matcher.filters[1]) != 2 {
		t.Fatalf("combo topic clause size mismatch: have %d, want %d", len(matcher.filters[1]), 2)
	}
	if len(matcher.filters[2]) != 1 {
		t.Fatalf("singletone topic clause size mismatch: have %d, want %d", len(matcher.filters[2]), 1)
	}
}

// Tests the matcher pipeline on a single continuous workflow without interrupts.
func TestMatcherContinuous(t *testing.T) {
	t.Parallel()
	testMatcherDiffBatches(t, [][]bloomIndexes{{{10, 20, 30}}}, 0, 100000, false, 75)
	testMatcherDiffBatches(t, [][]bloomIndexes{{{32, 3125, 100}}, {{40, 50, 10}}}, 0, 100000, false, 81)
	testMatcherDiffBatches(t, [][]bloomIndexes{{{4, 8, 11}, {7, 8, 17}}, {{9, 9, 12}, {15, 20, 13}}, {{18, 15, 15}, {12, 10, 4}}}, 0, 10000, false, 36)
}

// Tests the matcher pipeline on a constantly interrupted and resumed work pattern
// with the aim of ensuring data items are requested only once.
func TestMatcherIntermittent(t *testing.T) {
	t.Parallel()
	testMatcherDiffBatches(t, [][]bloomIndexes{{{10, 20, 30}}}, 0, 100000, true, 75)
	testMatcherDiffBatches(t, [][]bloomIndexes{{{32, 3125, 100}}, {{40, 50, 10}}}, 0, 100000, true, 81)
	testMatcherDiffBatches(t, [][]bloomIndexes{{{4, 8, 11}, {7, 8, 17}}, {{9, 9, 12}, {15, 20, 13}}, {{18, 15, 15}, {12, 10, 4}}}, 0, 10000, true, 36)
}

// Tests the matcher pipeline on random input to hopefully catch anomalies.
func TestMatcherRandom(t *testing.T) {
	t.Parallel()
	for i := 0; i < 10; i++ {
		testMatcherBothModes(t, makeRandomIndexes([]int{1}, 50), 0, 10000, 0)
		testMatcherBothModes(t, makeRandomIndexes([]int{3}, 50), 0, 10000, 0)
		testMatcherBothModes(t, makeRandomIndexes([]int{2, 2, 2}, 20), 0, 10000, 0)
		testMatcherBothModes(t, makeRandomIndexes([]int{5, 5, 5}, 50), 0, 10000, 0)
		testMatcherBothModes(t, makeRandomIndexes([]int{4, 4, 4}, 20), 0, 10000, 0)
	}
}

// Tests that the matcher can properly find matches if the starting block is
// shifted from a multiple of 8. This is needed to cover an optimisation with
// bitset matching https://github.com/ethereum/go-ethereum/issues/15309.
func TestMatcherShifted(t *testing.T) {
	t.Parallel()
	// Block 0 always matches in the tests, skip ahead of first 8 blocks with the
	// start to get a potential zero byte in the matcher bitset.

	// To keep the second bitset byte zero, the filter must only match for the first
	// time in block 16, so doing an all-16 bit filter should suffice.

	// To keep the starting block non divisible by 8, block number 9 is the first
	// that would introduce a shift and not match block 0.
	testMatcherBothModes(t, [][]bloomIndexes{{{16, 16, 16}}}, 9, 64, 0)
}

// Tests that matching on everything doesn't crash (special case internally).
func TestWildcardMatcher(t *testing.T) {
	t.Parallel()
	testMatcherBothModes(t, nil, 0, 10000, 0)
}

// makeRandomIndexes generates a random filter system, composed of multiple filter
// criteria, each having one bloom list component for the address and arbitrarily
// many topic bloom list components.
func makeRandomIndexes(lengths []int, max int) [][]bloomIndexes {
	res := make([][]bloomIndexes, len(lengths))
	for i, topics := range lengths {
		res[i] = make([]bloomIndexes, topics)
		for j := 0; j < topics; j++ {
			for k := 0; k < len(res[i][j]); k++ {
				res[i][j][k] = uint(rand.Intn(max-1) + 2)
			}
		}
	}
	return res
}

// testMatcherDiffBatches runs the given matches test in single-delivery and also
// in batches delivery mode, verifying that all kinds of deliveries are handled
// correctly within.
func testMatcherDiffBatches(t *testing.T, filter [][]bloomIndexes, start, blocks uint64, intermittent bool, retrievals uint32) {
	singleton := testMatcher(t, filter, start, blocks, intermittent, retrievals, 1)
	batched := testMatcher(t, filter, start, blocks, intermittent, retrievals, 16)

	if singleton != batched {
		t.Errorf("filter = %v blocks = %v intermittent = %v: request count mismatch, %v in singleton vs. %v in batched mode", filter, blocks, intermittent, singleton, batched)
	}
}

// testMatcherBothModes runs the given matcher test in both continuous as well as
// in intermittent mode, verifying that the request counts match each other.
func testMatcherBothModes(t *testing.T, filter [][]bloomIndexes, start, blocks uint64, retrievals uint32) {
	continuous := testMatcher(t, filter, start, blocks, false, retrievals, 16)
	intermittent := testMatcher(t, filter, start, blocks, true, retrievals, 16)

	if continuous != intermittent {
		t.Errorf("filter = %v blocks = %v: request count mismatch, %v in continuous vs. %v in intermittent mode", filter, blocks, continuous, intermittent)
	}
}

// testMatcher is a generic tester to run the given matcher test and return the
// number of requests made for cross validation between different modes.
func testMatcher(t *testing.T, filter [][]bloomIndexes, start, blocks uint64, intermittent bool, retrievals uint32, maxReqCount int) uint32 {
	// Create a new matcher an simulate our explicit random bitsets
	matcher := NewMatcher(testSectionSize, nil)
	matcher.filters = filter

	for _, rule := range filter {
		for _, topic := range rule {
			for _, bit := range topic {
				matcher.addScheduler(bit)
			}
		}
	}
	// Track the number of retrieval requests made
	var requested atomic.Uint32

	// Start the matching session for the filter and the retriever goroutines
	quit := make(chan struct{})
	matches := make(chan uint64, 16)

	session, err := matcher.Start(context.Background(), start, blocks-1, matches)
	if err != nil {
		t.Fatalf("failed to stat matcher session: %v", err)
	}
	startRetrievers(session, quit, &requested, maxReqCount)

	// Iterate over all the blocks and verify that the pipeline produces the correct matches
	for i := start; i < blocks; i++ {
		if expMatch3(filter, i) {
			match, ok := <-matches
			if !ok {
				t.Errorf("filter = %v  blocks = %v  intermittent = %v: expected #%v, results channel closed", filter, blocks, intermittent, i)
				return 0
			}
			if match != i {
				t.Errorf("filter = %v  blocks = %v  intermittent = %v: expected #%v, got #%v", filter, blocks, intermittent, i, match)
			}
			// If we're testing intermittent mode, abort and restart the pipeline
			if intermittent {
				session.Close()
				close(quit)

				quit = make(chan struct{})
				matches = make(chan uint64, 16)

				session, err = matcher.Start(context.Background(), i+1, blocks-1, matches)
				if err != nil {
					t.Fatalf("failed to stat matcher session: %v", err)
				}
				startRetrievers(session, quit, &requested, maxReqCount)
			}
		}
	}
	// Ensure the result channel is torn down after the last block
	match, ok := <-matches
	if ok {
		t.Errorf("filter = %v  blocks = %v  intermittent = %v: expected closed channel, got #%v", filter, blocks, intermittent, match)
	}
	// Clean up the session and ensure we match the expected retrieval count
	session.Close()
	close(quit)

	if retrievals != 0 && requested.Load() != retrievals {
		t.Errorf("filter = %v  blocks = %v  intermittent = %v: request count mismatch, have #%v, want #%v", filter, blocks, intermittent, requested.Load(), retrievals)
	}
	return requested.Load()
}

// startRetrievers starts a batch of goroutines listening for section requests
// and serving them.
func startRetrievers(session *MatcherSession, quit chan struct{}, retrievals *atomic.Uint32, batch int) {
	requests := make(chan chan *Retrieval)

	for i := 0; i < 10; i++ {
		// Start a multiplexer to test multiple threaded execution
		go session.Multiplex(batch, 100*time.Microsecond, requests)

		// Start a services to match the above multiplexer
		go func() {
			for {
				// Wait for a service request or a shutdown
				select {
				case <-quit:
					return

				case request := <-requests:
					task := <-request

					task.Bitsets = make([][]byte, len(task.Sections))
					for i, section := range task.Sections {
						if rand.Int()%4 != 0 { // Handle occasional missing deliveries
							task.Bitsets[i] = generateBitset(task.Bit, section)
							retrievals.Add(1)
						}
					}
					request <- task
				}
			}
		}()
	}
}

// generateBitset generates the rotated bitset for the given bloom bit and section
// numbers.
func generateBitset(bit uint, section uint64) []byte {
	bitset := make([]byte, testSectionSize/8)
	for i := 0; i < len(bitset); i++ {
		for b := 0; b < 8; b++ {
			blockIdx := section*testSectionSize + uint64(i*8+b)
			bitset[i] += bitset[i]
			if (blockIdx % uint64(bit)) == 0 {
				bitset[i]++
			}
		}
	}
	return bitset
}

func expMatch1(filter bloomIndexes, i uint64) bool {
	for _, ii := range filter {
		if (i % uint64(ii)) != 0 {
			return false
		}
	}
	return true
}

func expMatch2(filter []bloomIndexes, i uint64) bool {
	for _, ii := range filter {
		if expMatch1(ii, i) {
			return true
		}
	}
	return false
}

func expMatch3(filter [][]bloomIndexes, i uint64) bool {
	for _, ii := range filter {
		if !expMatch2(ii, i) {
			return false
		}
	}
	return true
}
