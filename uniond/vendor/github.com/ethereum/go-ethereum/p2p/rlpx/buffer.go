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

package rlpx

import (
	"io"
)

// readBuffer implements buffering for network reads. This type is similar to bufio.Reader,
// with two crucial differences: the buffer slice is exposed, and the buffer keeps all
// read data available until reset.
//
// How to use this type:
//
// Keep a readBuffer b alongside the underlying network connection. When reading a packet
// from the connection, first call b.reset(). This empties b.data. Now perform reads
// through b.read() until the end of the packet is reached. The complete packet data is
// now available in b.data.
type readBuffer struct {
	data []byte
	end  int
}

// reset removes all processed data which was read since the last call to reset.
// After reset, len(b.data) is zero.
func (b *readBuffer) reset() {
	unprocessed := b.end - len(b.data)
	copy(b.data[:unprocessed], b.data[len(b.data):b.end])
	b.end = unprocessed
	b.data = b.data[:0]
}

// read reads at least n bytes from r, returning the bytes.
// The returned slice is valid until the next call to reset.
func (b *readBuffer) read(r io.Reader, n int) ([]byte, error) {
	offset := len(b.data)
	have := b.end - len(b.data)

	// If n bytes are available in the buffer, there is no need to read from r at all.
	if have >= n {
		b.data = b.data[:offset+n]
		return b.data[offset : offset+n], nil
	}

	// Make buffer space available.
	need := n - have
	b.grow(need)

	// Read.
	rn, err := io.ReadAtLeast(r, b.data[b.end:cap(b.data)], need)
	if err != nil {
		return nil, err
	}
	b.end += rn
	b.data = b.data[:offset+n]
	return b.data[offset : offset+n], nil
}

// grow ensures the buffer has at least n bytes of unused space.
func (b *readBuffer) grow(n int) {
	if cap(b.data)-b.end >= n {
		return
	}
	need := n - (cap(b.data) - b.end)
	offset := len(b.data)
	b.data = append(b.data[:cap(b.data)], make([]byte, need)...)
	b.data = b.data[:offset]
}

// writeBuffer implements buffering for network writes. This is essentially
// a convenience wrapper around a byte slice.
type writeBuffer struct {
	data []byte
}

func (b *writeBuffer) reset() {
	b.data = b.data[:0]
}

func (b *writeBuffer) appendZero(n int) []byte {
	offset := len(b.data)
	b.data = append(b.data, make([]byte, n)...)
	return b.data[offset : offset+n]
}

func (b *writeBuffer) Write(data []byte) (int, error) {
	b.data = append(b.data, data...)
	return len(data), nil
}

const maxUint24 = int(^uint32(0) >> 8)

func readUint24(b []byte) uint32 {
	return uint32(b[2]) | uint32(b[1])<<8 | uint32(b[0])<<16
}

func putUint24(v uint32, b []byte) {
	b[0] = byte(v >> 16)
	b[1] = byte(v >> 8)
	b[2] = byte(v)
}

// growslice ensures b has the wanted length by either expanding it to its capacity
// or allocating a new slice if b has insufficient capacity.
func growslice(b []byte, wantLength int) []byte {
	if len(b) >= wantLength {
		return b
	}
	if cap(b) >= wantLength {
		return b[:cap(b)]
	}
	return make([]byte, wantLength)
}
