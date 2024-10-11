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
	"math"
	"math/bits"

	"github.com/holiman/uint256"
)

// log2_1_125 is used in the eviction priority calculation.
var log2_1_125 = math.Log2(1.125)

// evictionPriority calculates the eviction priority based on the algorithm
// described in the BlobPool docs for both fee components.
//
// This method takes about 8ns on a very recent laptop CPU, recalculating about
// 125 million transaction priority values per second.
func evictionPriority(basefeeJumps float64, txBasefeeJumps, blobfeeJumps, txBlobfeeJumps float64) int {
	var (
		basefeePriority = evictionPriority1D(basefeeJumps, txBasefeeJumps)
		blobfeePriority = evictionPriority1D(blobfeeJumps, txBlobfeeJumps)
	)
	if basefeePriority < blobfeePriority {
		return basefeePriority
	}
	return blobfeePriority
}

// evictionPriority1D calculates the eviction priority based on the algorithm
// described in the BlobPool docs for a single fee component.
func evictionPriority1D(basefeeJumps float64, txfeeJumps float64) int {
	jumps := txfeeJumps - basefeeJumps
	if int(jumps) == 0 {
		return 0 // can't log2 0
	}
	if jumps < 0 {
		return -intLog2(uint(-math.Floor(jumps)))
	}
	return intLog2(uint(math.Ceil(jumps)))
}

// dynamicFeeJumps calculates the log1.125(fee), namely the number of fee jumps
// needed to reach the requested one. We only use it when calculating the jumps
// between 2 fees, so it doesn't matter from what exact number with returns.
// it returns the result from (0, 1, 1.125).
//
// This method is very expensive, taking about 75ns on a very recent laptop CPU,
// but the result does not change with the lifetime of a transaction, so it can
// be cached.
func dynamicFeeJumps(fee *uint256.Int) float64 {
	if fee.IsZero() {
		return 0 // can't log2 zero, should never happen outside tests, but don't choke
	}
	return math.Log2(fee.Float64()) / log2_1_125
}

// intLog2 is a helper to calculate the integral part of a log2 of an unsigned
// integer. It is a very specific calculation that's not particularly useful in
// general, but it's what we need here (it's fast).
func intLog2(n uint) int {
	switch {
	case n == 0:
		panic("log2(0) is undefined")

	case n < 2048:
		return bits.UintSize - bits.LeadingZeros(n) - 1

	default:
		// The input is log1.125(uint256) = log2(uint256) / log2(1.125). At the
		// most extreme, log2(uint256) will be a bit below 257, and the constant
		// log2(1.125) ~= 0.17. The larges input thus is ~257 / ~0.17 ~= ~1511.
		panic("dynamic fee jump diffs cannot reach this")
	}
}
