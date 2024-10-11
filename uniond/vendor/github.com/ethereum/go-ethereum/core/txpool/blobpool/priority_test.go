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
	"testing"

	"github.com/holiman/uint256"
)

// Tests that the priority fees are calculated correctly as the log2 of the fee
// jumps needed to go from the base fee to the tx's fee cap.
func TestPriorityCalculation(t *testing.T) {
	tests := []struct {
		basefee uint64
		txfee   uint64
		result  int
	}{
		{basefee: 7, txfee: 10, result: 2},                          // 3.02 jumps, 4 ceil, 2 log2
		{basefee: 17_200_000_000, txfee: 17_200_000_000, result: 0}, // 0 jumps, special case 0 log2
		{basefee: 9_853_941_692, txfee: 11_085_092_510, result: 0},  // 0.99 jumps, 1 ceil, 0 log2
		{basefee: 11_544_106_391, txfee: 10_356_781_100, result: 0}, // -0.92 jumps, -1 floor, 0 log2
		{basefee: 17_200_000_000, txfee: 7, result: -7},             // -183.57 jumps, -184 floor, -7 log2
		{basefee: 7, txfee: 17_200_000_000, result: 7},              // 183.57 jumps, 184 ceil, 7 log2
	}
	for i, tt := range tests {
		var (
			baseJumps = dynamicFeeJumps(uint256.NewInt(tt.basefee))
			feeJumps  = dynamicFeeJumps(uint256.NewInt(tt.txfee))
		)
		if prio := evictionPriority1D(baseJumps, feeJumps); prio != tt.result {
			t.Errorf("test %d priority mismatch: have %d, want %d", i, prio, tt.result)
		}
	}
}

// Benchmarks how many dynamic fee jump values can be done.
func BenchmarkDynamicFeeJumpCalculation(b *testing.B) {
	fees := make([]*uint256.Int, b.N)
	for i := 0; i < b.N; i++ {
		fees[i] = uint256.NewInt(rand.Uint64())
	}
	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		dynamicFeeJumps(fees[i])
	}
}

// Benchmarks how many priority recalculations can be done.
func BenchmarkPriorityCalculation(b *testing.B) {
	// The basefee and blob fee is constant for all transactions across a block,
	// so we can assume theit absolute jump counts can be pre-computed.
	basefee := uint256.NewInt(17_200_000_000)  // 17.2 Gwei is the 22.03.2023 zero-emission basefee, random number
	blobfee := uint256.NewInt(123_456_789_000) // Completely random, no idea what this will be

	basefeeJumps := dynamicFeeJumps(basefee)
	blobfeeJumps := dynamicFeeJumps(blobfee)

	// The transaction's fee cap and blob fee cap are constant across the life
	// of the transaction, so we can pre-calculate and cache them.
	txBasefeeJumps := make([]float64, b.N)
	txBlobfeeJumps := make([]float64, b.N)
	for i := 0; i < b.N; i++ {
		txBasefeeJumps[i] = dynamicFeeJumps(uint256.NewInt(rand.Uint64()))
		txBlobfeeJumps[i] = dynamicFeeJumps(uint256.NewInt(rand.Uint64()))
	}
	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		evictionPriority(basefeeJumps, txBasefeeJumps[i], blobfeeJumps, txBlobfeeJumps[i])
	}
}
