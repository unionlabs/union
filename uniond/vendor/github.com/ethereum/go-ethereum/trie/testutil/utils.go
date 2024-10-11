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

package testutil

import (
	crand "crypto/rand"
	"encoding/binary"
	mrand "math/rand"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

// Prng is a pseudo random number generator seeded by strong randomness.
// The randomness is printed on startup in order to make failures reproducible.
var prng = initRand()

func initRand() *mrand.Rand {
	var seed [8]byte
	crand.Read(seed[:])
	rnd := mrand.New(mrand.NewSource(int64(binary.LittleEndian.Uint64(seed[:]))))
	return rnd
}

// RandBytes generates a random byte slice with specified length.
func RandBytes(n int) []byte {
	r := make([]byte, n)
	prng.Read(r)
	return r
}

// RandomHash generates a random blob of data and returns it as a hash.
func RandomHash() common.Hash {
	return common.BytesToHash(RandBytes(common.HashLength))
}

// RandomAddress generates a random blob of data and returns it as an address.
func RandomAddress() common.Address {
	return common.BytesToAddress(RandBytes(common.AddressLength))
}

// RandomNode generates a random node.
func RandomNode() *trienode.Node {
	val := RandBytes(100)
	return trienode.New(crypto.Keccak256Hash(val), val)
}
