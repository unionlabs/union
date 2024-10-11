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

package snapshot

import (
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/ethdb"
)

// holdableIterator is a wrapper of underlying database iterator. It extends
// the basic iterator interface by adding Hold which can hold the element
// locally where the iterator is currently located and serve it up next time.
type holdableIterator struct {
	it     ethdb.Iterator
	key    []byte
	val    []byte
	atHeld bool
}

// newHoldableIterator initializes the holdableIterator with the given iterator.
func newHoldableIterator(it ethdb.Iterator) *holdableIterator {
	return &holdableIterator{it: it}
}

// Hold holds the element locally where the iterator is currently located which
// can be served up next time.
func (it *holdableIterator) Hold() {
	if it.it.Key() == nil {
		return // nothing to hold
	}
	it.key = common.CopyBytes(it.it.Key())
	it.val = common.CopyBytes(it.it.Value())
	it.atHeld = false
}

// Next moves the iterator to the next key/value pair. It returns whether the
// iterator is exhausted.
func (it *holdableIterator) Next() bool {
	if !it.atHeld && it.key != nil {
		it.atHeld = true
	} else if it.atHeld {
		it.atHeld = false
		it.key = nil
		it.val = nil
	}
	if it.key != nil {
		return true // shifted to locally held value
	}
	return it.it.Next()
}

// Error returns any accumulated error. Exhausting all the key/value pairs
// is not considered to be an error.
func (it *holdableIterator) Error() error { return it.it.Error() }

// Release releases associated resources. Release should always succeed and can
// be called multiple times without causing error.
func (it *holdableIterator) Release() {
	it.atHeld = false
	it.key = nil
	it.val = nil
	it.it.Release()
}

// Key returns the key of the current key/value pair, or nil if done. The caller
// should not modify the contents of the returned slice, and its contents may
// change on the next call to Next.
func (it *holdableIterator) Key() []byte {
	if it.key != nil {
		return it.key
	}
	return it.it.Key()
}

// Value returns the value of the current key/value pair, or nil if done. The
// caller should not modify the contents of the returned slice, and its contents
// may change on the next call to Next.
func (it *holdableIterator) Value() []byte {
	if it.val != nil {
		return it.val
	}
	return it.it.Value()
}
