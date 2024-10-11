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

package rawdb

import "github.com/ethereum/go-ethereum/ethdb"

// KeyLengthIterator is a wrapper for a database iterator that ensures only key-value pairs
// with a specific key length will be returned.
type KeyLengthIterator struct {
	requiredKeyLength int
	ethdb.Iterator
}

// NewKeyLengthIterator returns a wrapped version of the iterator that will only return key-value
// pairs where keys with a specific key length will be returned.
func NewKeyLengthIterator(it ethdb.Iterator, keyLen int) ethdb.Iterator {
	return &KeyLengthIterator{
		Iterator:          it,
		requiredKeyLength: keyLen,
	}
}

func (it *KeyLengthIterator) Next() bool {
	// Return true as soon as a key with the required key length is discovered
	for it.Iterator.Next() {
		if len(it.Iterator.Key()) == it.requiredKeyLength {
			return true
		}
	}

	// Return false when we exhaust the keys in the underlying iterator.
	return false
}
