// Copyright 2021 The go-ethereum Authors
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

package eip1559

import (
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/params"
)

// copyConfig does a _shallow_ copy of a given config. Safe to set new values, but
// do not use e.g. SetInt() on the numbers. For testing only
func copyConfig(original *params.ChainConfig) *params.ChainConfig {
	return &params.ChainConfig{
		ChainID:                 original.ChainID,
		HomesteadBlock:          original.HomesteadBlock,
		DAOForkBlock:            original.DAOForkBlock,
		DAOForkSupport:          original.DAOForkSupport,
		EIP150Block:             original.EIP150Block,
		EIP155Block:             original.EIP155Block,
		EIP158Block:             original.EIP158Block,
		ByzantiumBlock:          original.ByzantiumBlock,
		ConstantinopleBlock:     original.ConstantinopleBlock,
		PetersburgBlock:         original.PetersburgBlock,
		IstanbulBlock:           original.IstanbulBlock,
		MuirGlacierBlock:        original.MuirGlacierBlock,
		BerlinBlock:             original.BerlinBlock,
		LondonBlock:             original.LondonBlock,
		TerminalTotalDifficulty: original.TerminalTotalDifficulty,
		Ethash:                  original.Ethash,
		Clique:                  original.Clique,
	}
}

func config() *params.ChainConfig {
	config := copyConfig(params.TestChainConfig)
	config.LondonBlock = big.NewInt(5)
	return config
}

// TestBlockGasLimits tests the gasLimit checks for blocks both across
// the EIP-1559 boundary and post-1559 blocks
func TestBlockGasLimits(t *testing.T) {
	initial := new(big.Int).SetUint64(params.InitialBaseFee)

	for i, tc := range []struct {
		pGasLimit uint64
		pNum      int64
		gasLimit  uint64
		ok        bool
	}{
		// Transitions from non-london to london
		{10000000, 4, 20000000, true},  // No change
		{10000000, 4, 20019530, true},  // Upper limit
		{10000000, 4, 20019531, false}, // Upper +1
		{10000000, 4, 19980470, true},  // Lower limit
		{10000000, 4, 19980469, false}, // Lower limit -1
		// London to London
		{20000000, 5, 20000000, true},
		{20000000, 5, 20019530, true},  // Upper limit
		{20000000, 5, 20019531, false}, // Upper limit +1
		{20000000, 5, 19980470, true},  // Lower limit
		{20000000, 5, 19980469, false}, // Lower limit -1
		{40000000, 5, 40039061, true},  // Upper limit
		{40000000, 5, 40039062, false}, // Upper limit +1
		{40000000, 5, 39960939, true},  // lower limit
		{40000000, 5, 39960938, false}, // Lower limit -1
	} {
		parent := &types.Header{
			GasUsed:  tc.pGasLimit / 2,
			GasLimit: tc.pGasLimit,
			BaseFee:  initial,
			Number:   big.NewInt(tc.pNum),
		}
		header := &types.Header{
			GasUsed:  tc.gasLimit / 2,
			GasLimit: tc.gasLimit,
			BaseFee:  initial,
			Number:   big.NewInt(tc.pNum + 1),
		}
		err := VerifyEIP1559Header(config(), parent, header)
		if tc.ok && err != nil {
			t.Errorf("test %d: Expected valid header: %s", i, err)
		}
		if !tc.ok && err == nil {
			t.Errorf("test %d: Expected invalid header", i)
		}
	}
}

// TestCalcBaseFee assumes all blocks are 1559-blocks
func TestCalcBaseFee(t *testing.T) {
	tests := []struct {
		parentBaseFee   int64
		parentGasLimit  uint64
		parentGasUsed   uint64
		expectedBaseFee int64
	}{
		{params.InitialBaseFee, 20000000, 10000000, params.InitialBaseFee}, // usage == target
		{params.InitialBaseFee, 20000000, 9000000, 987500000},              // usage below target
		{params.InitialBaseFee, 20000000, 11000000, 1012500000},            // usage above target
	}
	for i, test := range tests {
		parent := &types.Header{
			Number:   common.Big32,
			GasLimit: test.parentGasLimit,
			GasUsed:  test.parentGasUsed,
			BaseFee:  big.NewInt(test.parentBaseFee),
		}
		if have, want := CalcBaseFee(config(), parent), big.NewInt(test.expectedBaseFee); have.Cmp(want) != 0 {
			t.Errorf("test %d: have %d  want %d, ", i, have, want)
		}
	}
}
