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
	"bytes"
	"container/heap"
	"math"
	"sort"

	"github.com/ethereum/go-ethereum/common"
	"github.com/holiman/uint256"
)

// evictHeap is a helper data structure to keep track of the cheapest bottleneck
// transaction from each account to determine which account to evict from.
//
// The heap internally tracks a slice of cheapest transactions from each account
// and a mapping from addresses to indices for direct removals/udates.
//
// The goal of the heap is to decide which account has the worst bottleneck to
// evict transactions from.
type evictHeap struct {
	metas *map[common.Address][]*blobTxMeta // Pointer to the blob pool's index for price retrievals

	basefeeJumps float64 // Pre-calculated absolute dynamic fee jumps for the base fee
	blobfeeJumps float64 // Pre-calculated absolute dynamic fee jumps for the blob fee

	addrs []common.Address       // Heap of addresses to retrieve the cheapest out of
	index map[common.Address]int // Indices into the heap for replacements
}

// newPriceHeap creates a new heap of cheapest accounts in the blob pool to evict
// from in case of over saturation.
func newPriceHeap(basefee *uint256.Int, blobfee *uint256.Int, index *map[common.Address][]*blobTxMeta) *evictHeap {
	heap := &evictHeap{
		metas: index,
		index: make(map[common.Address]int),
	}
	// Populate the heap in account sort order. Not really needed in practice,
	// but it makes the heap initialization deterministic and less annoying to
	// test in unit tests.
	addrs := make([]common.Address, 0, len(*index))
	for addr := range *index {
		addrs = append(addrs, addr)
	}
	sort.Slice(addrs, func(i, j int) bool { return bytes.Compare(addrs[i][:], addrs[j][:]) < 0 })

	for _, addr := range addrs {
		heap.index[addr] = len(heap.addrs)
		heap.addrs = append(heap.addrs, addr)
	}
	heap.reinit(basefee, blobfee, true)
	return heap
}

// reinit updates the pre-calculated dynamic fee jumps in the price heap and runs
// the sorting algorithm from scratch on the entire heap.
func (h *evictHeap) reinit(basefee *uint256.Int, blobfee *uint256.Int, force bool) {
	// If the update is mostly the same as the old, don't sort pointlessly
	basefeeJumps := dynamicFeeJumps(basefee)
	blobfeeJumps := dynamicFeeJumps(blobfee)

	if !force && math.Abs(h.basefeeJumps-basefeeJumps) < 0.01 && math.Abs(h.blobfeeJumps-blobfeeJumps) < 0.01 { // TODO(karalabe): 0.01 enough, maybe should be smaller? Maybe this optimization is moot?
		return
	}
	// One or both of the dynamic fees jumped, resort the pool
	h.basefeeJumps = basefeeJumps
	h.blobfeeJumps = blobfeeJumps

	heap.Init(h)
}

// Len implements sort.Interface as part of heap.Interface, returning the number
// of accounts in the pool which can be considered for eviction.
func (h *evictHeap) Len() int {
	return len(h.addrs)
}

// Less implements sort.Interface as part of heap.Interface, returning which of
// the two requested accounts has a cheaper bottleneck.
func (h *evictHeap) Less(i, j int) bool {
	txsI := (*(h.metas))[h.addrs[i]]
	txsJ := (*(h.metas))[h.addrs[j]]

	lastI := txsI[len(txsI)-1]
	lastJ := txsJ[len(txsJ)-1]

	prioI := evictionPriority(h.basefeeJumps, lastI.evictionExecFeeJumps, h.blobfeeJumps, lastI.evictionBlobFeeJumps)
	if prioI > 0 {
		prioI = 0
	}
	prioJ := evictionPriority(h.basefeeJumps, lastJ.evictionExecFeeJumps, h.blobfeeJumps, lastJ.evictionBlobFeeJumps)
	if prioJ > 0 {
		prioJ = 0
	}
	if prioI == prioJ {
		return lastI.evictionExecTip.Lt(lastJ.evictionExecTip)
	}
	return prioI < prioJ
}

// Swap implements sort.Interface as part of heap.Interface, maintaining both the
// order of the accounts according to the heap, and the account->item slot mapping
// for replacements.
func (h *evictHeap) Swap(i, j int) {
	h.index[h.addrs[i]], h.index[h.addrs[j]] = h.index[h.addrs[j]], h.index[h.addrs[i]]
	h.addrs[i], h.addrs[j] = h.addrs[j], h.addrs[i]
}

// Push implements heap.Interface, appending an item to the end of the account
// ordering as well as the address to item slot mapping.
func (h *evictHeap) Push(x any) {
	h.index[x.(common.Address)] = len(h.addrs)
	h.addrs = append(h.addrs, x.(common.Address))
}

// Pop implements heap.Interface, removing and returning the last element of the
// heap.
//
// Note, use `heap.Pop`, not `evictHeap.Pop`. This method is used by Go's heap,
// to provide the functionality, it does not embed it.
func (h *evictHeap) Pop() any {
	// Remove the last element from the heap
	size := len(h.addrs)
	addr := h.addrs[size-1]
	h.addrs = h.addrs[:size-1]

	// Unindex the removed element and return
	delete(h.index, addr)
	return addr
}
