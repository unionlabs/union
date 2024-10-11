// Copyright 2023 The go-ethereum Authors
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

package blobpool

import (
	"container/heap"
	mrand "math/rand"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/params"
	"github.com/holiman/uint256"
)

var rand = mrand.New(mrand.NewSource(1))

// verifyHeapInternals verifies that all accounts present in the index are also
// present in the heap and internals are consistent across various indices.
func verifyHeapInternals(t *testing.T, evict *evictHeap) {
	t.Helper()

	// Ensure that all accounts are present in the heap and no extras
	seen := make(map[common.Address]struct{})
	for i, addr := range evict.addrs {
		seen[addr] = struct{}{}
		if _, ok := (*evict.metas)[addr]; !ok {
			t.Errorf("heap contains unexpected address at slot %d: %v", i, addr)
		}
	}
	for addr := range *evict.metas {
		if _, ok := seen[addr]; !ok {
			t.Errorf("heap is missing required address %v", addr)
		}
	}
	if len(evict.addrs) != len(*evict.metas) {
		t.Errorf("heap size %d mismatches metadata size %d", len(evict.addrs), len(*evict.metas))
	}
	// Ensure that all accounts are present in the heap order index and no extras
	have := make([]common.Address, len(evict.index))
	for addr, i := range evict.index {
		have[i] = addr
	}
	if len(have) != len(evict.addrs) {
		t.Errorf("heap index size %d mismatches heap size %d", len(have), len(evict.addrs))
	}
	for i := 0; i < len(have) && i < len(evict.addrs); i++ {
		if have[i] != evict.addrs[i] {
			t.Errorf("heap index for slot %d mismatches: have %v, want %v", i, have[i], evict.addrs[i])
		}
	}
}

// Tests that the price heap can correctly sort its set of transactions based on
// an input base- and blob fee.
func TestPriceHeapSorting(t *testing.T) {
	tests := []struct {
		execTips []uint64
		execFees []uint64
		blobFees []uint64

		basefee uint64
		blobfee uint64

		order []int
	}{
		// If everything is above the basefee and blobfee, order by miner tip
		{
			execTips: []uint64{1, 0, 2},
			execFees: []uint64{1, 2, 3},
			blobFees: []uint64{3, 2, 1},
			basefee:  0,
			blobfee:  0,
			order:    []int{1, 0, 2},
		},
		// If only basefees are used (blob fee matches with network), return the
		// ones the furthest below the current basefee, splitting same ones with
		// the tip. Anything above the basefee should be split by tip.
		{
			execTips: []uint64{100, 50, 100, 50, 1, 2, 3},
			execFees: []uint64{1000, 1000, 500, 500, 2000, 2000, 2000},
			blobFees: []uint64{0, 0, 0, 0, 0, 0, 0},
			basefee:  1999,
			blobfee:  0,
			order:    []int{3, 2, 1, 0, 4, 5, 6},
		},
		// If only blobfees are used (base fee matches with network), return the
		// ones the furthest below the current blobfee, splitting same ones with
		// the tip. Anything above the blobfee should be split by tip.
		{
			execTips: []uint64{100, 50, 100, 50, 1, 2, 3},
			execFees: []uint64{0, 0, 0, 0, 0, 0, 0},
			blobFees: []uint64{1000, 1000, 500, 500, 2000, 2000, 2000},
			basefee:  0,
			blobfee:  1999,
			order:    []int{3, 2, 1, 0, 4, 5, 6},
		},
		// If both basefee and blobfee is specified, sort by the larger distance
		// of the two from the current network conditions, splitting same (loglog)
		// ones via the tip.
		//
		// Basefee: 1000
		// Blobfee: 100
		//
		// Tx #0: (800, 80) - 2 jumps below both => priority -1
		// Tx #1: (630, 63) - 4 jumps below both => priority -2
		// Tx #2: (800, 63) - 2 jumps below basefee, 4 jumps below blobfee => priority -2 (blob penalty dominates)
		// Tx #3: (630, 80) - 4 jumps below basefee, 2 jumps below blobfee => priority -2 (base penalty dominates)
		//
		// Txs 1, 2, 3 share the same priority, split via tip, prefer 0 as the best
		{
			execTips: []uint64{1, 2, 3, 4},
			execFees: []uint64{800, 630, 800, 630},
			blobFees: []uint64{80, 63, 63, 80},
			basefee:  1000,
			blobfee:  100,
			order:    []int{1, 2, 3, 0},
		},
	}
	for i, tt := range tests {
		// Create an index of the transactions
		index := make(map[common.Address][]*blobTxMeta)
		for j := byte(0); j < byte(len(tt.execTips)); j++ {
			addr := common.Address{j}

			var (
				execTip = uint256.NewInt(tt.execTips[j])
				execFee = uint256.NewInt(tt.execFees[j])
				blobFee = uint256.NewInt(tt.blobFees[j])

				basefeeJumps = dynamicFeeJumps(execFee)
				blobfeeJumps = dynamicFeeJumps(blobFee)
			)
			index[addr] = []*blobTxMeta{{
				id:                   uint64(j),
				size:                 128 * 1024,
				nonce:                0,
				execTipCap:           execTip,
				execFeeCap:           execFee,
				blobFeeCap:           blobFee,
				basefeeJumps:         basefeeJumps,
				blobfeeJumps:         blobfeeJumps,
				evictionExecTip:      execTip,
				evictionExecFeeJumps: basefeeJumps,
				evictionBlobFeeJumps: blobfeeJumps,
			}}
		}
		// Create a price heap and check the pop order
		priceheap := newPriceHeap(uint256.NewInt(tt.basefee), uint256.NewInt(tt.blobfee), &index)
		verifyHeapInternals(t, priceheap)

		for j := 0; j < len(tt.order); j++ {
			if next := heap.Pop(priceheap); int(next.(common.Address)[0]) != tt.order[j] {
				t.Errorf("test %d, item %d: order mismatch: have %d, want %d", i, j, next.(common.Address)[0], tt.order[j])
			} else {
				delete(index, next.(common.Address)) // remove to simulate a correct pool for the test
			}
			verifyHeapInternals(t, priceheap)
		}
	}
}

// Benchmarks reheaping the entire set of accounts in the blob pool.
func BenchmarkPriceHeapReinit1MB(b *testing.B)   { benchmarkPriceHeapReinit(b, 1024*1024) }
func BenchmarkPriceHeapReinit10MB(b *testing.B)  { benchmarkPriceHeapReinit(b, 10*1024*1024) }
func BenchmarkPriceHeapReinit100MB(b *testing.B) { benchmarkPriceHeapReinit(b, 100*1024*1024) }
func BenchmarkPriceHeapReinit1GB(b *testing.B)   { benchmarkPriceHeapReinit(b, 1024*1024*1024) }
func BenchmarkPriceHeapReinit10GB(b *testing.B)  { benchmarkPriceHeapReinit(b, 10*1024*1024*1024) }
func BenchmarkPriceHeapReinit25GB(b *testing.B)  { benchmarkPriceHeapReinit(b, 25*1024*1024*1024) }
func BenchmarkPriceHeapReinit50GB(b *testing.B)  { benchmarkPriceHeapReinit(b, 50*1024*1024*1024) }
func BenchmarkPriceHeapReinit100GB(b *testing.B) { benchmarkPriceHeapReinit(b, 100*1024*1024*1024) }

func benchmarkPriceHeapReinit(b *testing.B, datacap uint64) {
	// Calculate how many unique transactions we can fit into the provided disk
	// data cap
	blobs := datacap / (params.BlobTxBytesPerFieldElement * params.BlobTxFieldElementsPerBlob)

	// Create a random set of transactions with random fees. Use a separate account
	// for each transaction to make it worse case.
	index := make(map[common.Address][]*blobTxMeta)
	for i := 0; i < int(blobs); i++ {
		var addr common.Address
		rand.Read(addr[:])

		var (
			execTip = uint256.NewInt(rand.Uint64())
			execFee = uint256.NewInt(rand.Uint64())
			blobFee = uint256.NewInt(rand.Uint64())

			basefeeJumps = dynamicFeeJumps(execFee)
			blobfeeJumps = dynamicFeeJumps(blobFee)
		)
		index[addr] = []*blobTxMeta{{
			id:                   uint64(i),
			size:                 128 * 1024,
			nonce:                0,
			execTipCap:           execTip,
			execFeeCap:           execFee,
			blobFeeCap:           blobFee,
			basefeeJumps:         basefeeJumps,
			blobfeeJumps:         blobfeeJumps,
			evictionExecTip:      execTip,
			evictionExecFeeJumps: basefeeJumps,
			evictionBlobFeeJumps: blobfeeJumps,
		}}
	}
	// Create a price heap and reinit it over and over
	heap := newPriceHeap(uint256.NewInt(rand.Uint64()), uint256.NewInt(rand.Uint64()), &index)

	basefees := make([]*uint256.Int, b.N)
	blobfees := make([]*uint256.Int, b.N)
	for i := 0; i < b.N; i++ {
		basefees[i] = uint256.NewInt(rand.Uint64())
		blobfees[i] = uint256.NewInt(rand.Uint64())
	}
	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		heap.reinit(basefees[i], blobfees[i], true)
	}
}

// Benchmarks overflowing the heap over and over (add and then drop).
func BenchmarkPriceHeapOverflow1MB(b *testing.B)   { benchmarkPriceHeapOverflow(b, 1024*1024) }
func BenchmarkPriceHeapOverflow10MB(b *testing.B)  { benchmarkPriceHeapOverflow(b, 10*1024*1024) }
func BenchmarkPriceHeapOverflow100MB(b *testing.B) { benchmarkPriceHeapOverflow(b, 100*1024*1024) }
func BenchmarkPriceHeapOverflow1GB(b *testing.B)   { benchmarkPriceHeapOverflow(b, 1024*1024*1024) }
func BenchmarkPriceHeapOverflow10GB(b *testing.B)  { benchmarkPriceHeapOverflow(b, 10*1024*1024*1024) }
func BenchmarkPriceHeapOverflow25GB(b *testing.B)  { benchmarkPriceHeapOverflow(b, 25*1024*1024*1024) }
func BenchmarkPriceHeapOverflow50GB(b *testing.B)  { benchmarkPriceHeapOverflow(b, 50*1024*1024*1024) }
func BenchmarkPriceHeapOverflow100GB(b *testing.B) { benchmarkPriceHeapOverflow(b, 100*1024*1024*1024) }

func benchmarkPriceHeapOverflow(b *testing.B, datacap uint64) {
	// Calculate how many unique transactions we can fit into the provided disk
	// data cap
	blobs := datacap / (params.BlobTxBytesPerFieldElement * params.BlobTxFieldElementsPerBlob)

	// Create a random set of transactions with random fees. Use a separate account
	// for each transaction to make it worse case.
	index := make(map[common.Address][]*blobTxMeta)
	for i := 0; i < int(blobs); i++ {
		var addr common.Address
		rand.Read(addr[:])

		var (
			execTip = uint256.NewInt(rand.Uint64())
			execFee = uint256.NewInt(rand.Uint64())
			blobFee = uint256.NewInt(rand.Uint64())

			basefeeJumps = dynamicFeeJumps(execFee)
			blobfeeJumps = dynamicFeeJumps(blobFee)
		)
		index[addr] = []*blobTxMeta{{
			id:                   uint64(i),
			size:                 128 * 1024,
			nonce:                0,
			execTipCap:           execTip,
			execFeeCap:           execFee,
			blobFeeCap:           blobFee,
			basefeeJumps:         basefeeJumps,
			blobfeeJumps:         blobfeeJumps,
			evictionExecTip:      execTip,
			evictionExecFeeJumps: basefeeJumps,
			evictionBlobFeeJumps: blobfeeJumps,
		}}
	}
	// Create a price heap and overflow it over and over
	evict := newPriceHeap(uint256.NewInt(rand.Uint64()), uint256.NewInt(rand.Uint64()), &index)
	var (
		addrs = make([]common.Address, b.N)
		metas = make([]*blobTxMeta, b.N)
	)
	for i := 0; i < b.N; i++ {
		rand.Read(addrs[i][:])

		var (
			execTip = uint256.NewInt(rand.Uint64())
			execFee = uint256.NewInt(rand.Uint64())
			blobFee = uint256.NewInt(rand.Uint64())

			basefeeJumps = dynamicFeeJumps(execFee)
			blobfeeJumps = dynamicFeeJumps(blobFee)
		)
		metas[i] = &blobTxMeta{
			id:                   uint64(int(blobs) + i),
			size:                 128 * 1024,
			nonce:                0,
			execTipCap:           execTip,
			execFeeCap:           execFee,
			blobFeeCap:           blobFee,
			basefeeJumps:         basefeeJumps,
			blobfeeJumps:         blobfeeJumps,
			evictionExecTip:      execTip,
			evictionExecFeeJumps: basefeeJumps,
			evictionBlobFeeJumps: blobfeeJumps,
		}
	}
	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		index[addrs[i]] = []*blobTxMeta{metas[i]}
		heap.Push(evict, addrs[i])

		drop := heap.Pop(evict)
		delete(index, drop.(common.Address))
	}
}
