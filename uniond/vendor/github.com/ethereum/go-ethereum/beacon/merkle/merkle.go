// Copyright 2022 The go-ethereum Authors
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

// Package merkle implements proof verifications in binary merkle trees.
package merkle

import (
	"crypto/sha256"
	"errors"
	"reflect"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

// Value represents either a 32 byte leaf value or hash node in a binary merkle tree/partial proof.
type Value [32]byte

// Values represent a series of merkle tree leaves/nodes.
type Values []Value

var valueT = reflect.TypeOf(Value{})

// UnmarshalJSON parses a merkle value in hex syntax.
func (m *Value) UnmarshalJSON(input []byte) error {
	return hexutil.UnmarshalFixedJSON(valueT, input, m[:])
}

// VerifyProof verifies a Merkle proof branch for a single value in a
// binary Merkle tree (index is a generalized tree index).
func VerifyProof(root common.Hash, index uint64, branch Values, value Value) error {
	hasher := sha256.New()
	for _, sibling := range branch {
		hasher.Reset()
		if index&1 == 0 {
			hasher.Write(value[:])
			hasher.Write(sibling[:])
		} else {
			hasher.Write(sibling[:])
			hasher.Write(value[:])
		}
		hasher.Sum(value[:0])
		if index >>= 1; index == 0 {
			return errors.New("branch has extra items")
		}
	}
	if index != 1 {
		return errors.New("branch is missing items")
	}
	if common.Hash(value) != root {
		return errors.New("root mismatch")
	}
	return nil
}
