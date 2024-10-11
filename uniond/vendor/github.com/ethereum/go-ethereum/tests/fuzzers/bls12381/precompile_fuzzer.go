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

package bls

import (
	"bytes"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/vm"
)

const (
	blsG1Add      = byte(10)
	blsG1Mul      = byte(11)
	blsG1MultiExp = byte(12)
	blsG2Add      = byte(13)
	blsG2Mul      = byte(14)
	blsG2MultiExp = byte(15)
	blsPairing    = byte(16)
	blsMapG1      = byte(17)
	blsMapG2      = byte(18)
)

func checkInput(id byte, inputLen int) bool {
	switch id {
	case blsG1Add:
		return inputLen == 256
	case blsG1Mul:
		return inputLen == 160
	case blsG1MultiExp:
		return inputLen%160 == 0
	case blsG2Add:
		return inputLen == 512
	case blsG2Mul:
		return inputLen == 288
	case blsG2MultiExp:
		return inputLen%288 == 0
	case blsPairing:
		return inputLen%384 == 0
	case blsMapG1:
		return inputLen == 64
	case blsMapG2:
		return inputLen == 128
	}
	panic("programmer error")
}

// The function must return
//
//   - 1 if the fuzzer should increase priority of the
//     given input during subsequent fuzzing (for example, the input is lexically
//     correct and was parsed successfully);
//   - -1 if the input must not be added to corpus even if gives new coverage; and
//   - 0 otherwise
//
// other values are reserved for future use.
func fuzz(id byte, data []byte) int {
	// Even on bad input, it should not crash, so we still test the gas calc
	precompile := vm.PrecompiledContractsBLS[common.BytesToAddress([]byte{id})]
	gas := precompile.RequiredGas(data)
	if !checkInput(id, len(data)) {
		return 0
	}
	// If the gas cost is too large (25M), bail out
	if gas > 25*1000*1000 {
		return 0
	}
	cpy := make([]byte, len(data))
	copy(cpy, data)
	_, err := precompile.Run(cpy)
	if !bytes.Equal(cpy, data) {
		panic(fmt.Sprintf("input data modified, precompile %d: %x %x", id, data, cpy))
	}
	if err != nil {
		return 0
	}
	return 1
}
