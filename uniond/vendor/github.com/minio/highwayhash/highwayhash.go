// Copyright (c) 2017 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

// Package highwayhash implements the pseudo-random-function (PRF) HighwayHash.
// HighwayHash is a fast hash function designed to defend hash-flooding attacks
// or to authenticate short-lived messages.
//
// HighwayHash is not a general purpose cryptographic hash function and does not
// provide (strong) collision resistance.
package highwayhash

import (
	"encoding/binary"
	"errors"
	"hash"
)

const (
	// Size is the size of HighwayHash-256 checksum in bytes.
	Size = 32
	// Size128 is the size of HighwayHash-128 checksum in bytes.
	Size128 = 16
	// Size64 is the size of HighwayHash-64 checksum in bytes.
	Size64 = 8
)

var errKeySize = errors.New("highwayhash: invalid key size")

// New returns a hash.Hash computing the HighwayHash-256 checksum.
// It returns a non-nil error if the key is not 32 bytes long.
func New(key []byte) (hash.Hash, error) {
	if len(key) != Size {
		return nil, errKeySize
	}
	h := &digest{size: Size}
	copy(h.key[:], key)
	h.Reset()
	return h, nil
}

// New128 returns a hash.Hash computing the HighwayHash-128 checksum.
// It returns a non-nil error if the key is not 32 bytes long.
func New128(key []byte) (hash.Hash, error) {
	if len(key) != Size {
		return nil, errKeySize
	}
	h := &digest{size: Size128}
	copy(h.key[:], key)
	h.Reset()
	return h, nil
}

// New64 returns a hash.Hash computing the HighwayHash-64 checksum.
// It returns a non-nil error if the key is not 32 bytes long.
func New64(key []byte) (hash.Hash64, error) {
	if len(key) != Size {
		return nil, errKeySize
	}
	h := new(digest64)
	h.size = Size64
	copy(h.key[:], key)
	h.Reset()
	return h, nil
}

// Sum computes the HighwayHash-256 checksum of data.
// It panics if the key is not 32 bytes long.
func Sum(data, key []byte) [Size]byte {
	if len(key) != Size {
		panic(errKeySize)
	}
	var state [16]uint64
	initialize(&state, key)
	if n := len(data) & (^(Size - 1)); n > 0 {
		update(&state, data[:n])
		data = data[n:]
	}
	if len(data) > 0 {
		var block [Size]byte
		offset := copy(block[:], data)
		hashBuffer(&state, &block, offset)
	}
	var hash [Size]byte
	finalize(hash[:], &state)
	return hash
}

// Sum128 computes the HighwayHash-128 checksum of data.
// It panics if the key is not 32 bytes long.
func Sum128(data, key []byte) [Size128]byte {
	if len(key) != Size {
		panic(errKeySize)
	}
	var state [16]uint64
	initialize(&state, key)
	if n := len(data) & (^(Size - 1)); n > 0 {
		update(&state, data[:n])
		data = data[n:]
	}
	if len(data) > 0 {
		var block [Size]byte
		offset := copy(block[:], data)
		hashBuffer(&state, &block, offset)
	}
	var hash [Size128]byte
	finalize(hash[:], &state)
	return hash
}

// Sum64 computes the HighwayHash-64 checksum of data.
// It panics if the key is not 32 bytes long.
func Sum64(data, key []byte) uint64 {
	if len(key) != Size {
		panic(errKeySize)
	}
	var state [16]uint64
	initialize(&state, key)
	if n := len(data) & (^(Size - 1)); n > 0 {
		update(&state, data[:n])
		data = data[n:]
	}
	if len(data) > 0 {
		var block [Size]byte
		offset := copy(block[:], data)
		hashBuffer(&state, &block, offset)
	}
	var hash [Size64]byte
	finalize(hash[:], &state)
	return binary.LittleEndian.Uint64(hash[:])
}

type digest64 struct{ digest }

func (d *digest64) Sum64() uint64 {
	state := d.state
	if d.offset > 0 {
		hashBuffer(&state, &d.buffer, d.offset)
	}
	var hash [8]byte
	finalize(hash[:], &state)
	return binary.LittleEndian.Uint64(hash[:])
}

type digest struct {
	state [16]uint64 // v0 | v1 | mul0 | mul1

	key, buffer [Size]byte
	offset      int

	size int
}

func (d *digest) Size() int { return d.size }

func (d *digest) BlockSize() int { return Size }

func (d *digest) Reset() {
	initialize(&d.state, d.key[:])
	d.offset = 0
}

func (d *digest) Write(p []byte) (n int, err error) {
	n = len(p)
	if d.offset > 0 {
		remaining := Size - d.offset
		if n < remaining {
			d.offset += copy(d.buffer[d.offset:], p)
			return
		}
		copy(d.buffer[d.offset:], p[:remaining])
		update(&d.state, d.buffer[:])
		p = p[remaining:]
		d.offset = 0
	}
	if nn := len(p) & (^(Size - 1)); nn > 0 {
		update(&d.state, p[:nn])
		p = p[nn:]
	}
	if len(p) > 0 {
		d.offset = copy(d.buffer[d.offset:], p)
	}
	return
}

func (d *digest) Sum(b []byte) []byte {
	state := d.state
	if d.offset > 0 {
		hashBuffer(&state, &d.buffer, d.offset)
	}
	var hash [Size]byte
	finalize(hash[:d.size], &state)
	return append(b, hash[:d.size]...)
}

func hashBuffer(state *[16]uint64, buffer *[32]byte, offset int) {
	var block [Size]byte
	mod32 := (uint64(offset) << 32) + uint64(offset)
	for i := range state[:4] {
		state[i] += mod32
	}
	for i := range state[4:8] {
		t0 := uint32(state[i+4])
		t0 = (t0 << uint(offset)) | (t0 >> uint(32-offset))

		t1 := uint32(state[i+4] >> 32)
		t1 = (t1 << uint(offset)) | (t1 >> uint(32-offset))

		state[i+4] = (uint64(t1) << 32) | uint64(t0)
	}

	mod4 := offset & 3
	remain := offset - mod4

	copy(block[:], buffer[:remain])
	if offset >= 16 {
		copy(block[28:], buffer[offset-4:])
	} else if mod4 != 0 {
		last := uint32(buffer[remain])
		last += uint32(buffer[remain+mod4>>1]) << 8
		last += uint32(buffer[offset-1]) << 16
		binary.LittleEndian.PutUint32(block[16:], last)
	}
	update(state, block[:])
}
