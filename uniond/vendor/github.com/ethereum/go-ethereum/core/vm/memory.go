// Copyright 2015 The go-ethereum Authors
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

package vm

import (
	"github.com/holiman/uint256"
)

// Memory implements a simple memory model for the ethereum virtual machine.
type Memory struct {
	store       []byte
	lastGasCost uint64
}

// NewMemory returns a new memory model.
func NewMemory() *Memory {
	return &Memory{}
}

// Set sets offset + size to value
func (m *Memory) Set(offset, size uint64, value []byte) {
	// It's possible the offset is greater than 0 and size equals 0. This is because
	// the calcMemSize (common.go) could potentially return 0 when size is zero (NO-OP)
	if size > 0 {
		// length of store may never be less than offset + size.
		// The store should be resized PRIOR to setting the memory
		if offset+size > uint64(len(m.store)) {
			panic("invalid memory: store empty")
		}
		copy(m.store[offset:offset+size], value)
	}
}

// Set32 sets the 32 bytes starting at offset to the value of val, left-padded with zeroes to
// 32 bytes.
func (m *Memory) Set32(offset uint64, val *uint256.Int) {
	// length of store may never be less than offset + size.
	// The store should be resized PRIOR to setting the memory
	if offset+32 > uint64(len(m.store)) {
		panic("invalid memory: store empty")
	}
	// Fill in relevant bits
	b32 := val.Bytes32()
	copy(m.store[offset:], b32[:])
}

// Resize resizes the memory to size
func (m *Memory) Resize(size uint64) {
	if uint64(m.Len()) < size {
		m.store = append(m.store, make([]byte, size-uint64(m.Len()))...)
	}
}

// GetCopy returns offset + size as a new slice
func (m *Memory) GetCopy(offset, size int64) (cpy []byte) {
	if size == 0 {
		return nil
	}

	if len(m.store) > int(offset) {
		cpy = make([]byte, size)
		copy(cpy, m.store[offset:offset+size])

		return
	}

	return
}

// GetPtr returns the offset + size
func (m *Memory) GetPtr(offset, size int64) []byte {
	if size == 0 {
		return nil
	}

	if len(m.store) > int(offset) {
		return m.store[offset : offset+size]
	}

	return nil
}

// Len returns the length of the backing slice
func (m *Memory) Len() int {
	return len(m.store)
}

// Data returns the backing slice
func (m *Memory) Data() []byte {
	return m.store
}

// Copy copies data from the src position slice into the dst position.
// The source and destination may overlap.
// OBS: This operation assumes that any necessary memory expansion has already been performed,
// and this method may panic otherwise.
func (m *Memory) Copy(dst, src, len uint64) {
	if len == 0 {
		return
	}
	copy(m.store[dst:], m.store[src:src+len])
}
