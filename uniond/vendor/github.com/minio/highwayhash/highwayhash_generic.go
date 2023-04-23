// Copyright (c) 2017 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

package highwayhash

import (
	"encoding/binary"
)

const (
	v0   = 0
	v1   = 4
	mul0 = 8
	mul1 = 12
)

var (
	init0 = [4]uint64{0xdbe6d5d5fe4cce2f, 0xa4093822299f31d0, 0x13198a2e03707344, 0x243f6a8885a308d3}
	init1 = [4]uint64{0x3bd39e10cb0ef593, 0xc0acf169b5f18a8c, 0xbe5466cf34e90c6c, 0x452821e638d01377}
)

func initializeGeneric(state *[16]uint64, k []byte) {
	var key [4]uint64

	key[0] = binary.LittleEndian.Uint64(k[0:])
	key[1] = binary.LittleEndian.Uint64(k[8:])
	key[2] = binary.LittleEndian.Uint64(k[16:])
	key[3] = binary.LittleEndian.Uint64(k[24:])

	copy(state[mul0:], init0[:])
	copy(state[mul1:], init1[:])

	for i, k := range key {
		state[v0+i] = init0[i] ^ k
	}

	key[0] = key[0]>>32 | key[0]<<32
	key[1] = key[1]>>32 | key[1]<<32
	key[2] = key[2]>>32 | key[2]<<32
	key[3] = key[3]>>32 | key[3]<<32

	for i, k := range key {
		state[v1+i] = init1[i] ^ k
	}
}

func updateGeneric(state *[16]uint64, msg []byte) {
	for len(msg) > 0 {
		// add message
		state[v1+0] += binary.LittleEndian.Uint64(msg)
		state[v1+1] += binary.LittleEndian.Uint64(msg[8:])
		state[v1+2] += binary.LittleEndian.Uint64(msg[16:])
		state[v1+3] += binary.LittleEndian.Uint64(msg[24:])

		// v1 += mul0
		state[v1+0] += state[mul0+0]
		state[v1+1] += state[mul0+1]
		state[v1+2] += state[mul0+2]
		state[v1+3] += state[mul0+3]

		state[mul0+0] ^= uint64(uint32(state[v1+0])) * (state[v0+0] >> 32)
		state[mul0+1] ^= uint64(uint32(state[v1+1])) * (state[v0+1] >> 32)
		state[mul0+2] ^= uint64(uint32(state[v1+2])) * (state[v0+2] >> 32)
		state[mul0+3] ^= uint64(uint32(state[v1+3])) * (state[v0+3] >> 32)

		// v0 += mul1
		state[v0+0] += state[mul1+0]
		state[v0+1] += state[mul1+1]
		state[v0+2] += state[mul1+2]
		state[v0+3] += state[mul1+3]

		state[mul1+0] ^= uint64(uint32(state[v0+0])) * (state[v1+0] >> 32)
		state[mul1+1] ^= uint64(uint32(state[v0+1])) * (state[v1+1] >> 32)
		state[mul1+2] ^= uint64(uint32(state[v0+2])) * (state[v1+2] >> 32)
		state[mul1+3] ^= uint64(uint32(state[v0+3])) * (state[v1+3] >> 32)

		zipperMerge(state[v1+0], state[v1+1], &state[v0+0], &state[v0+1])
		zipperMerge(state[v1+2], state[v1+3], &state[v0+2], &state[v0+3])

		zipperMerge(state[v0+0], state[v0+1], &state[v1+0], &state[v1+1])
		zipperMerge(state[v0+2], state[v0+3], &state[v1+2], &state[v1+3])
		msg = msg[32:]
	}
}

func finalizeGeneric(out []byte, state *[16]uint64) {
	var perm [4]uint64
	var tmp [32]byte
	runs := 4
	if len(out) == 16 {
		runs = 6
	} else if len(out) == 32 {
		runs = 10
	}
	for i := 0; i < runs; i++ {
		perm[0] = state[v0+2]>>32 | state[v0+2]<<32
		perm[1] = state[v0+3]>>32 | state[v0+3]<<32
		perm[2] = state[v0+0]>>32 | state[v0+0]<<32
		perm[3] = state[v0+1]>>32 | state[v0+1]<<32

		binary.LittleEndian.PutUint64(tmp[0:], perm[0])
		binary.LittleEndian.PutUint64(tmp[8:], perm[1])
		binary.LittleEndian.PutUint64(tmp[16:], perm[2])
		binary.LittleEndian.PutUint64(tmp[24:], perm[3])

		update(state, tmp[:])
	}

	switch len(out) {
	case 8:
		binary.LittleEndian.PutUint64(out, state[v0+0]+state[v1+0]+state[mul0+0]+state[mul1+0])
	case 16:
		binary.LittleEndian.PutUint64(out, state[v0+0]+state[v1+2]+state[mul0+0]+state[mul1+2])
		binary.LittleEndian.PutUint64(out[8:], state[v0+1]+state[v1+3]+state[mul0+1]+state[mul1+3])
	case 32:
		h0, h1 := reduceMod(state[v0+0]+state[mul0+0], state[v0+1]+state[mul0+1], state[v1+0]+state[mul1+0], state[v1+1]+state[mul1+1])
		binary.LittleEndian.PutUint64(out[0:], h0)
		binary.LittleEndian.PutUint64(out[8:], h1)

		h0, h1 = reduceMod(state[v0+2]+state[mul0+2], state[v0+3]+state[mul0+3], state[v1+2]+state[mul1+2], state[v1+3]+state[mul1+3])
		binary.LittleEndian.PutUint64(out[16:], h0)
		binary.LittleEndian.PutUint64(out[24:], h1)
	}
}

func zipperMerge(v0, v1 uint64, d0, d1 *uint64) {
	m0 := v0 & (0xFF << (2 * 8))
	m1 := (v1 & (0xFF << (7 * 8))) >> 8
	m2 := ((v0 & (0xFF << (5 * 8))) + (v1 & (0xFF << (6 * 8)))) >> 16
	m3 := ((v0 & (0xFF << (3 * 8))) + (v1 & (0xFF << (4 * 8)))) >> 24
	m4 := (v0 & (0xFF << (1 * 8))) << 32
	m5 := v0 << 56

	*d0 += m0 + m1 + m2 + m3 + m4 + m5

	m0 = (v0 & (0xFF << (7 * 8))) + (v1 & (0xFF << (2 * 8)))
	m1 = (v0 & (0xFF << (6 * 8))) >> 8
	m2 = (v1 & (0xFF << (5 * 8))) >> 16
	m3 = ((v1 & (0xFF << (3 * 8))) + (v0 & (0xFF << (4 * 8)))) >> 24
	m4 = (v1 & 0xFF) << 48
	m5 = (v1 & (0xFF << (1 * 8))) << 24

	*d1 += m3 + m2 + m5 + m1 + m4 + m0
}

// reduce v = [v0, v1, v2, v3] mod the irreducible polynomial x^128 + x^2 + x
func reduceMod(v0, v1, v2, v3 uint64) (r0, r1 uint64) {
	v3 &= 0x3FFFFFFFFFFFFFFF

	r0, r1 = v2, v3

	v3 = (v3 << 1) | (v2 >> (64 - 1))
	v2 <<= 1
	r1 = (r1 << 2) | (r0 >> (64 - 2))
	r0 <<= 2

	r0 ^= v0 ^ v2
	r1 ^= v1 ^ v3
	return
}
