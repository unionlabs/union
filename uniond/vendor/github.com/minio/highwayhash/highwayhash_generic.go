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
	for len(msg) >= 32 {
		m := msg[:32]

		// add message + mul0
		// Interleave operations to hide multiplication
		state[v1+0] += binary.LittleEndian.Uint64(m) + state[mul0+0]
		state[mul0+0] ^= uint64(uint32(state[v1+0])) * (state[v0+0] >> 32)
		state[v0+0] += state[mul1+0]
		state[mul1+0] ^= uint64(uint32(state[v0+0])) * (state[v1+0] >> 32)

		state[v1+1] += binary.LittleEndian.Uint64(m[8:]) + state[mul0+1]
		state[mul0+1] ^= uint64(uint32(state[v1+1])) * (state[v0+1] >> 32)
		state[v0+1] += state[mul1+1]
		state[mul1+1] ^= uint64(uint32(state[v0+1])) * (state[v1+1] >> 32)

		state[v1+2] += binary.LittleEndian.Uint64(m[16:]) + state[mul0+2]
		state[mul0+2] ^= uint64(uint32(state[v1+2])) * (state[v0+2] >> 32)
		state[v0+2] += state[mul1+2]
		state[mul1+2] ^= uint64(uint32(state[v0+2])) * (state[v1+2] >> 32)

		state[v1+3] += binary.LittleEndian.Uint64(m[24:]) + state[mul0+3]
		state[mul0+3] ^= uint64(uint32(state[v1+3])) * (state[v0+3] >> 32)
		state[v0+3] += state[mul1+3]
		state[mul1+3] ^= uint64(uint32(state[v0+3])) * (state[v1+3] >> 32)

		// inlined: zipperMerge(state[v1+0], state[v1+1], &state[v0+0], &state[v0+1])
		{
			val0 := state[v1+0]
			val1 := state[v1+1]
			res := val0 & (0xff << (2 * 8))
			res2 := (val0 & (0xff << (7 * 8))) + (val1 & (0xff << (2 * 8)))
			res += (val1 & (0xff << (7 * 8))) >> 8
			res2 += (val0 & (0xff << (6 * 8))) >> 8
			res += ((val0 & (0xff << (5 * 8))) + (val1 & (0xff << (6 * 8)))) >> 16
			res2 += (val1 & (0xff << (5 * 8))) >> 16
			res += ((val0 & (0xff << (3 * 8))) + (val1 & (0xff << (4 * 8)))) >> 24
			res2 += ((val1 & (0xff << (3 * 8))) + (val0 & (0xff << (4 * 8)))) >> 24
			res += (val0 & (0xff << (1 * 8))) << 32
			res2 += (val1 & 0xff) << 48
			res += val0 << 56
			res2 += (val1 & (0xff << (1 * 8))) << 24

			state[v0+0] += res
			state[v0+1] += res2
		}
		// zipperMerge(state[v1+2], state[v1+3], &state[v0+2], &state[v0+3])
		{
			val0 := state[v1+2]
			val1 := state[v1+3]
			res := val0 & (0xff << (2 * 8))
			res2 := (val0 & (0xff << (7 * 8))) + (val1 & (0xff << (2 * 8)))
			res += (val1 & (0xff << (7 * 8))) >> 8
			res2 += (val0 & (0xff << (6 * 8))) >> 8
			res += ((val0 & (0xff << (5 * 8))) + (val1 & (0xff << (6 * 8)))) >> 16
			res2 += (val1 & (0xff << (5 * 8))) >> 16
			res += ((val0 & (0xff << (3 * 8))) + (val1 & (0xff << (4 * 8)))) >> 24
			res2 += ((val1 & (0xff << (3 * 8))) + (val0 & (0xff << (4 * 8)))) >> 24
			res += (val0 & (0xff << (1 * 8))) << 32
			res2 += (val1 & 0xff) << 48
			res += val0 << 56
			res2 += (val1 & (0xff << (1 * 8))) << 24

			state[v0+2] += res
			state[v0+3] += res2
		}

		// inlined: zipperMerge(state[v0+0], state[v0+1], &state[v1+0], &state[v1+1])
		{
			val0 := state[v0+0]
			val1 := state[v0+1]
			res := val0 & (0xff << (2 * 8))
			res2 := (val0 & (0xff << (7 * 8))) + (val1 & (0xff << (2 * 8)))
			res += (val1 & (0xff << (7 * 8))) >> 8
			res2 += (val0 & (0xff << (6 * 8))) >> 8
			res += ((val0 & (0xff << (5 * 8))) + (val1 & (0xff << (6 * 8)))) >> 16
			res2 += (val1 & (0xff << (5 * 8))) >> 16
			res += ((val0 & (0xff << (3 * 8))) + (val1 & (0xff << (4 * 8)))) >> 24
			res2 += ((val1 & (0xff << (3 * 8))) + (val0 & (0xff << (4 * 8)))) >> 24
			res += (val0 & (0xff << (1 * 8))) << 32
			res2 += (val1 & 0xff) << 48
			res += val0 << 56
			res2 += (val1 & (0xff << (1 * 8))) << 24

			state[v1+0] += res
			state[v1+1] += res2
		}

		//inlined: zipperMerge(state[v0+2], state[v0+3], &state[v1+2], &state[v1+3])
		{
			val0 := state[v0+2]
			val1 := state[v0+3]
			res := val0 & (0xff << (2 * 8))
			res2 := (val0 & (0xff << (7 * 8))) + (val1 & (0xff << (2 * 8)))
			res += (val1 & (0xff << (7 * 8))) >> 8
			res2 += (val0 & (0xff << (6 * 8))) >> 8
			res += ((val0 & (0xff << (5 * 8))) + (val1 & (0xff << (6 * 8)))) >> 16
			res2 += (val1 & (0xff << (5 * 8))) >> 16
			res += ((val0 & (0xff << (3 * 8))) + (val1 & (0xff << (4 * 8)))) >> 24
			res2 += ((val1 & (0xff << (3 * 8))) + (val0 & (0xff << (4 * 8)))) >> 24
			res += (val0 & (0xff << (1 * 8))) << 32
			res2 += (val1 & 0xff) << 48
			res += val0 << 56
			res2 += (val1 & (0xff << (1 * 8))) << 24

			state[v1+2] += res
			state[v1+3] += res2
		}
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

// Experiments on variations left for future reference...
/*
func zipperMerge(v0, v1 uint64, d0, d1 *uint64) {
	if true {
		// fastest. original interleaved...
		res := v0 & (0xff << (2 * 8))
		res2 := (v0 & (0xff << (7 * 8))) + (v1 & (0xff << (2 * 8)))
		res += (v1 & (0xff << (7 * 8))) >> 8
		res2 += (v0 & (0xff << (6 * 8))) >> 8
		res += ((v0 & (0xff << (5 * 8))) + (v1 & (0xff << (6 * 8)))) >> 16
		res2 += (v1 & (0xff << (5 * 8))) >> 16
		res += ((v0 & (0xff << (3 * 8))) + (v1 & (0xff << (4 * 8)))) >> 24
		res2 += ((v1 & (0xff << (3 * 8))) + (v0 & (0xff << (4 * 8)))) >> 24
		res += (v0 & (0xff << (1 * 8))) << 32
		res2 += (v1 & 0xff) << 48
		res += v0 << 56
		res2 += (v1 & (0xff << (1 * 8))) << 24

		*d0 += res
		*d1 += res2
	} else if false {
		// Reading bytes and combining into uint64
		var v0b [8]byte
		binary.LittleEndian.PutUint64(v0b[:], v0)
		var v1b [8]byte
		binary.LittleEndian.PutUint64(v1b[:], v1)
		var res, res2 uint64

		res = uint64(v0b[0]) << (7 * 8)
		res2 = uint64(v1b[0]) << (6 * 8)
		res |= uint64(v0b[1]) << (5 * 8)
		res2 |= uint64(v1b[1]) << (4 * 8)
		res |= uint64(v0b[2]) << (2 * 8)
		res2 |= uint64(v1b[2]) << (2 * 8)
		res |= uint64(v0b[3])
		res2 |= uint64(v0b[4]) << (1 * 8)
		res |= uint64(v0b[5]) << (3 * 8)
		res2 |= uint64(v0b[6]) << (5 * 8)
		res |= uint64(v1b[4]) << (1 * 8)
		res2 |= uint64(v0b[7]) << (7 * 8)
		res |= uint64(v1b[6]) << (4 * 8)
		res2 |= uint64(v1b[3])
		res |= uint64(v1b[7]) << (6 * 8)
		res2 |= uint64(v1b[5]) << (3 * 8)

		*d0 += res
		*d1 += res2

	} else if false {
		// bytes to bytes shuffle
		var v0b [8]byte
		binary.LittleEndian.PutUint64(v0b[:], v0)
		var v1b [8]byte
		binary.LittleEndian.PutUint64(v1b[:], v1)
		var res [8]byte

		//res += ((v0 & (0xff << (3 * 8))) + (v1 & (0xff << (4 * 8)))) >> 24
		res[0] = v0b[3]
		res[1] = v1b[4]

		// res := v0 & (0xff << (2 * 8))
		res[2] = v0b[2]

		//res += ((v0 & (0xff << (5 * 8))) + (v1 & (0xff << (6 * 8)))) >> 16
		res[3] = v0b[5]
		res[4] = v1b[6]

		//res += (v0 & (0xff << (1 * 8))) << 32
		res[5] = v0b[1]

		//res += (v1 & (0xff << (7 * 8))) >> 8
		res[6] += v1b[7]

		//res += v0 << 56
		res[7] = v0b[0]
		v0 = binary.LittleEndian.Uint64(res[:])
		*d0 += v0

		//res += ((v1 & (0xff << (3 * 8))) + (v0 & (0xff << (4 * 8)))) >> 24
		res[0] = v1b[3]
		res[1] = v0b[4]

		res[2] = v1b[2]

		// res += (v1 & (0xff << (5 * 8))) >> 16
		res[3] = v1b[5]

		//res += (v1 & (0xff << (1 * 8))) << 24
		res[4] = v1b[1]

		// res += (v0 & (0xff << (6 * 8))) >> 8
		res[5] = v0b[6]

		//res := (v0 & (0xff << (7 * 8))) + (v1 & (0xff << (2 * 8)))
		res[7] = v0b[7]

		//res += (v1 & 0xff) << 48
		res[6] = v1b[0]

		v0 = binary.LittleEndian.Uint64(res[:])
		*d1 += v0
	} else {
		// original.
		res := v0 & (0xff << (2 * 8))
		res += (v1 & (0xff << (7 * 8))) >> 8
		res += ((v0 & (0xff << (5 * 8))) + (v1 & (0xff << (6 * 8)))) >> 16
		res += ((v0 & (0xff << (3 * 8))) + (v1 & (0xff << (4 * 8)))) >> 24
		res += (v0 & (0xff << (1 * 8))) << 32
		res += v0 << 56

		*d0 += res

		res = (v0 & (0xff << (7 * 8))) + (v1 & (0xff << (2 * 8)))
		res += (v0 & (0xff << (6 * 8))) >> 8
		res += (v1 & (0xff << (5 * 8))) >> 16
		res += ((v1 & (0xff << (3 * 8))) + (v0 & (0xff << (4 * 8)))) >> 24
		res += (v1 & 0xff) << 48
		res += (v1 & (0xff << (1 * 8))) << 24

		*d1 += res
	}
}
*/

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
