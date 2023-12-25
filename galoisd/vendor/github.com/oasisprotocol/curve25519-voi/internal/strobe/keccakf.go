// Copyright (c) 2014 The Go Authors. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//    * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//go:build !amd64 || purego || !gc

package strobe

import "encoding/binary"

// rc stores the round constants for use in the ι step.
var rc = [24]uint64{
	0x0000000000000001,
	0x0000000000008082,
	0x800000000000808A,
	0x8000000080008000,
	0x000000000000808B,
	0x0000000080000001,
	0x8000000080008081,
	0x8000000000008009,
	0x000000000000008A,
	0x0000000000000088,
	0x0000000080008009,
	0x000000008000000A,
	0x000000008000808B,
	0x800000000000008B,
	0x8000000000008089,
	0x8000000000008003,
	0x8000000000008002,
	0x8000000000000080,
	0x000000000000800A,
	0x800000008000000A,
	0x8000000080008081,
	0x8000000000008080,
	0x0000000080000001,
	0x8000000080008008,
}

// keccakF1600 applies the Keccak permutation to a 1600b-wide
// state represented as a slice of 25 uint64s.
func keccakF1600(a *[25]uint64) {
	// Implementation translated from Keccak-inplace.c
	// in the keccak reference code.
	var t, bc0, bc1, bc2, bc3, bc4, d0, d1, d2, d3, d4 uint64

	for i := 0; i < 24; i += 4 {
		// Combines the 5 steps in each round into 2 steps.
		// Unrolls 4 rounds per loop and spreads some steps across rounds.

		// Round 1
		bc0 = a[0] ^ a[5] ^ a[10] ^ a[15] ^ a[20]
		bc1 = a[1] ^ a[6] ^ a[11] ^ a[16] ^ a[21]
		bc2 = a[2] ^ a[7] ^ a[12] ^ a[17] ^ a[22]
		bc3 = a[3] ^ a[8] ^ a[13] ^ a[18] ^ a[23]
		bc4 = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24]
		d0 = bc4 ^ (bc1<<1 | bc1>>63)
		d1 = bc0 ^ (bc2<<1 | bc2>>63)
		d2 = bc1 ^ (bc3<<1 | bc3>>63)
		d3 = bc2 ^ (bc4<<1 | bc4>>63)
		d4 = bc3 ^ (bc0<<1 | bc0>>63)

		bc0 = a[0] ^ d0
		t = a[6] ^ d1
		bc1 = t<<44 | t>>(64-44)
		t = a[12] ^ d2
		bc2 = t<<43 | t>>(64-43)
		t = a[18] ^ d3
		bc3 = t<<21 | t>>(64-21)
		t = a[24] ^ d4
		bc4 = t<<14 | t>>(64-14)
		a[0] = bc0 ^ (bc2 &^ bc1) ^ rc[i]
		a[6] = bc1 ^ (bc3 &^ bc2)
		a[12] = bc2 ^ (bc4 &^ bc3)
		a[18] = bc3 ^ (bc0 &^ bc4)
		a[24] = bc4 ^ (bc1 &^ bc0)

		t = a[10] ^ d0
		bc2 = t<<3 | t>>(64-3)
		t = a[16] ^ d1
		bc3 = t<<45 | t>>(64-45)
		t = a[22] ^ d2
		bc4 = t<<61 | t>>(64-61)
		t = a[3] ^ d3
		bc0 = t<<28 | t>>(64-28)
		t = a[9] ^ d4
		bc1 = t<<20 | t>>(64-20)
		a[10] = bc0 ^ (bc2 &^ bc1)
		a[16] = bc1 ^ (bc3 &^ bc2)
		a[22] = bc2 ^ (bc4 &^ bc3)
		a[3] = bc3 ^ (bc0 &^ bc4)
		a[9] = bc4 ^ (bc1 &^ bc0)

		t = a[20] ^ d0
		bc4 = t<<18 | t>>(64-18)
		t = a[1] ^ d1
		bc0 = t<<1 | t>>(64-1)
		t = a[7] ^ d2
		bc1 = t<<6 | t>>(64-6)
		t = a[13] ^ d3
		bc2 = t<<25 | t>>(64-25)
		t = a[19] ^ d4
		bc3 = t<<8 | t>>(64-8)
		a[20] = bc0 ^ (bc2 &^ bc1)
		a[1] = bc1 ^ (bc3 &^ bc2)
		a[7] = bc2 ^ (bc4 &^ bc3)
		a[13] = bc3 ^ (bc0 &^ bc4)
		a[19] = bc4 ^ (bc1 &^ bc0)

		t = a[5] ^ d0
		bc1 = t<<36 | t>>(64-36)
		t = a[11] ^ d1
		bc2 = t<<10 | t>>(64-10)
		t = a[17] ^ d2
		bc3 = t<<15 | t>>(64-15)
		t = a[23] ^ d3
		bc4 = t<<56 | t>>(64-56)
		t = a[4] ^ d4
		bc0 = t<<27 | t>>(64-27)
		a[5] = bc0 ^ (bc2 &^ bc1)
		a[11] = bc1 ^ (bc3 &^ bc2)
		a[17] = bc2 ^ (bc4 &^ bc3)
		a[23] = bc3 ^ (bc0 &^ bc4)
		a[4] = bc4 ^ (bc1 &^ bc0)

		t = a[15] ^ d0
		bc3 = t<<41 | t>>(64-41)
		t = a[21] ^ d1
		bc4 = t<<2 | t>>(64-2)
		t = a[2] ^ d2
		bc0 = t<<62 | t>>(64-62)
		t = a[8] ^ d3
		bc1 = t<<55 | t>>(64-55)
		t = a[14] ^ d4
		bc2 = t<<39 | t>>(64-39)
		a[15] = bc0 ^ (bc2 &^ bc1)
		a[21] = bc1 ^ (bc3 &^ bc2)
		a[2] = bc2 ^ (bc4 &^ bc3)
		a[8] = bc3 ^ (bc0 &^ bc4)
		a[14] = bc4 ^ (bc1 &^ bc0)

		// Round 2
		bc0 = a[0] ^ a[5] ^ a[10] ^ a[15] ^ a[20]
		bc1 = a[1] ^ a[6] ^ a[11] ^ a[16] ^ a[21]
		bc2 = a[2] ^ a[7] ^ a[12] ^ a[17] ^ a[22]
		bc3 = a[3] ^ a[8] ^ a[13] ^ a[18] ^ a[23]
		bc4 = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24]
		d0 = bc4 ^ (bc1<<1 | bc1>>63)
		d1 = bc0 ^ (bc2<<1 | bc2>>63)
		d2 = bc1 ^ (bc3<<1 | bc3>>63)
		d3 = bc2 ^ (bc4<<1 | bc4>>63)
		d4 = bc3 ^ (bc0<<1 | bc0>>63)

		bc0 = a[0] ^ d0
		t = a[16] ^ d1
		bc1 = t<<44 | t>>(64-44)
		t = a[7] ^ d2
		bc2 = t<<43 | t>>(64-43)
		t = a[23] ^ d3
		bc3 = t<<21 | t>>(64-21)
		t = a[14] ^ d4
		bc4 = t<<14 | t>>(64-14)
		a[0] = bc0 ^ (bc2 &^ bc1) ^ rc[i+1]
		a[16] = bc1 ^ (bc3 &^ bc2)
		a[7] = bc2 ^ (bc4 &^ bc3)
		a[23] = bc3 ^ (bc0 &^ bc4)
		a[14] = bc4 ^ (bc1 &^ bc0)

		t = a[20] ^ d0
		bc2 = t<<3 | t>>(64-3)
		t = a[11] ^ d1
		bc3 = t<<45 | t>>(64-45)
		t = a[2] ^ d2
		bc4 = t<<61 | t>>(64-61)
		t = a[18] ^ d3
		bc0 = t<<28 | t>>(64-28)
		t = a[9] ^ d4
		bc1 = t<<20 | t>>(64-20)
		a[20] = bc0 ^ (bc2 &^ bc1)
		a[11] = bc1 ^ (bc3 &^ bc2)
		a[2] = bc2 ^ (bc4 &^ bc3)
		a[18] = bc3 ^ (bc0 &^ bc4)
		a[9] = bc4 ^ (bc1 &^ bc0)

		t = a[15] ^ d0
		bc4 = t<<18 | t>>(64-18)
		t = a[6] ^ d1
		bc0 = t<<1 | t>>(64-1)
		t = a[22] ^ d2
		bc1 = t<<6 | t>>(64-6)
		t = a[13] ^ d3
		bc2 = t<<25 | t>>(64-25)
		t = a[4] ^ d4
		bc3 = t<<8 | t>>(64-8)
		a[15] = bc0 ^ (bc2 &^ bc1)
		a[6] = bc1 ^ (bc3 &^ bc2)
		a[22] = bc2 ^ (bc4 &^ bc3)
		a[13] = bc3 ^ (bc0 &^ bc4)
		a[4] = bc4 ^ (bc1 &^ bc0)

		t = a[10] ^ d0
		bc1 = t<<36 | t>>(64-36)
		t = a[1] ^ d1
		bc2 = t<<10 | t>>(64-10)
		t = a[17] ^ d2
		bc3 = t<<15 | t>>(64-15)
		t = a[8] ^ d3
		bc4 = t<<56 | t>>(64-56)
		t = a[24] ^ d4
		bc0 = t<<27 | t>>(64-27)
		a[10] = bc0 ^ (bc2 &^ bc1)
		a[1] = bc1 ^ (bc3 &^ bc2)
		a[17] = bc2 ^ (bc4 &^ bc3)
		a[8] = bc3 ^ (bc0 &^ bc4)
		a[24] = bc4 ^ (bc1 &^ bc0)

		t = a[5] ^ d0
		bc3 = t<<41 | t>>(64-41)
		t = a[21] ^ d1
		bc4 = t<<2 | t>>(64-2)
		t = a[12] ^ d2
		bc0 = t<<62 | t>>(64-62)
		t = a[3] ^ d3
		bc1 = t<<55 | t>>(64-55)
		t = a[19] ^ d4
		bc2 = t<<39 | t>>(64-39)
		a[5] = bc0 ^ (bc2 &^ bc1)
		a[21] = bc1 ^ (bc3 &^ bc2)
		a[12] = bc2 ^ (bc4 &^ bc3)
		a[3] = bc3 ^ (bc0 &^ bc4)
		a[19] = bc4 ^ (bc1 &^ bc0)

		// Round 3
		bc0 = a[0] ^ a[5] ^ a[10] ^ a[15] ^ a[20]
		bc1 = a[1] ^ a[6] ^ a[11] ^ a[16] ^ a[21]
		bc2 = a[2] ^ a[7] ^ a[12] ^ a[17] ^ a[22]
		bc3 = a[3] ^ a[8] ^ a[13] ^ a[18] ^ a[23]
		bc4 = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24]
		d0 = bc4 ^ (bc1<<1 | bc1>>63)
		d1 = bc0 ^ (bc2<<1 | bc2>>63)
		d2 = bc1 ^ (bc3<<1 | bc3>>63)
		d3 = bc2 ^ (bc4<<1 | bc4>>63)
		d4 = bc3 ^ (bc0<<1 | bc0>>63)

		bc0 = a[0] ^ d0
		t = a[11] ^ d1
		bc1 = t<<44 | t>>(64-44)
		t = a[22] ^ d2
		bc2 = t<<43 | t>>(64-43)
		t = a[8] ^ d3
		bc3 = t<<21 | t>>(64-21)
		t = a[19] ^ d4
		bc4 = t<<14 | t>>(64-14)
		a[0] = bc0 ^ (bc2 &^ bc1) ^ rc[i+2]
		a[11] = bc1 ^ (bc3 &^ bc2)
		a[22] = bc2 ^ (bc4 &^ bc3)
		a[8] = bc3 ^ (bc0 &^ bc4)
		a[19] = bc4 ^ (bc1 &^ bc0)

		t = a[15] ^ d0
		bc2 = t<<3 | t>>(64-3)
		t = a[1] ^ d1
		bc3 = t<<45 | t>>(64-45)
		t = a[12] ^ d2
		bc4 = t<<61 | t>>(64-61)
		t = a[23] ^ d3
		bc0 = t<<28 | t>>(64-28)
		t = a[9] ^ d4
		bc1 = t<<20 | t>>(64-20)
		a[15] = bc0 ^ (bc2 &^ bc1)
		a[1] = bc1 ^ (bc3 &^ bc2)
		a[12] = bc2 ^ (bc4 &^ bc3)
		a[23] = bc3 ^ (bc0 &^ bc4)
		a[9] = bc4 ^ (bc1 &^ bc0)

		t = a[5] ^ d0
		bc4 = t<<18 | t>>(64-18)
		t = a[16] ^ d1
		bc0 = t<<1 | t>>(64-1)
		t = a[2] ^ d2
		bc1 = t<<6 | t>>(64-6)
		t = a[13] ^ d3
		bc2 = t<<25 | t>>(64-25)
		t = a[24] ^ d4
		bc3 = t<<8 | t>>(64-8)
		a[5] = bc0 ^ (bc2 &^ bc1)
		a[16] = bc1 ^ (bc3 &^ bc2)
		a[2] = bc2 ^ (bc4 &^ bc3)
		a[13] = bc3 ^ (bc0 &^ bc4)
		a[24] = bc4 ^ (bc1 &^ bc0)

		t = a[20] ^ d0
		bc1 = t<<36 | t>>(64-36)
		t = a[6] ^ d1
		bc2 = t<<10 | t>>(64-10)
		t = a[17] ^ d2
		bc3 = t<<15 | t>>(64-15)
		t = a[3] ^ d3
		bc4 = t<<56 | t>>(64-56)
		t = a[14] ^ d4
		bc0 = t<<27 | t>>(64-27)
		a[20] = bc0 ^ (bc2 &^ bc1)
		a[6] = bc1 ^ (bc3 &^ bc2)
		a[17] = bc2 ^ (bc4 &^ bc3)
		a[3] = bc3 ^ (bc0 &^ bc4)
		a[14] = bc4 ^ (bc1 &^ bc0)

		t = a[10] ^ d0
		bc3 = t<<41 | t>>(64-41)
		t = a[21] ^ d1
		bc4 = t<<2 | t>>(64-2)
		t = a[7] ^ d2
		bc0 = t<<62 | t>>(64-62)
		t = a[18] ^ d3
		bc1 = t<<55 | t>>(64-55)
		t = a[4] ^ d4
		bc2 = t<<39 | t>>(64-39)
		a[10] = bc0 ^ (bc2 &^ bc1)
		a[21] = bc1 ^ (bc3 &^ bc2)
		a[7] = bc2 ^ (bc4 &^ bc3)
		a[18] = bc3 ^ (bc0 &^ bc4)
		a[4] = bc4 ^ (bc1 &^ bc0)

		// Round 4
		bc0 = a[0] ^ a[5] ^ a[10] ^ a[15] ^ a[20]
		bc1 = a[1] ^ a[6] ^ a[11] ^ a[16] ^ a[21]
		bc2 = a[2] ^ a[7] ^ a[12] ^ a[17] ^ a[22]
		bc3 = a[3] ^ a[8] ^ a[13] ^ a[18] ^ a[23]
		bc4 = a[4] ^ a[9] ^ a[14] ^ a[19] ^ a[24]
		d0 = bc4 ^ (bc1<<1 | bc1>>63)
		d1 = bc0 ^ (bc2<<1 | bc2>>63)
		d2 = bc1 ^ (bc3<<1 | bc3>>63)
		d3 = bc2 ^ (bc4<<1 | bc4>>63)
		d4 = bc3 ^ (bc0<<1 | bc0>>63)

		bc0 = a[0] ^ d0
		t = a[1] ^ d1
		bc1 = t<<44 | t>>(64-44)
		t = a[2] ^ d2
		bc2 = t<<43 | t>>(64-43)
		t = a[3] ^ d3
		bc3 = t<<21 | t>>(64-21)
		t = a[4] ^ d4
		bc4 = t<<14 | t>>(64-14)
		a[0] = bc0 ^ (bc2 &^ bc1) ^ rc[i+3]
		a[1] = bc1 ^ (bc3 &^ bc2)
		a[2] = bc2 ^ (bc4 &^ bc3)
		a[3] = bc3 ^ (bc0 &^ bc4)
		a[4] = bc4 ^ (bc1 &^ bc0)

		t = a[5] ^ d0
		bc2 = t<<3 | t>>(64-3)
		t = a[6] ^ d1
		bc3 = t<<45 | t>>(64-45)
		t = a[7] ^ d2
		bc4 = t<<61 | t>>(64-61)
		t = a[8] ^ d3
		bc0 = t<<28 | t>>(64-28)
		t = a[9] ^ d4
		bc1 = t<<20 | t>>(64-20)
		a[5] = bc0 ^ (bc2 &^ bc1)
		a[6] = bc1 ^ (bc3 &^ bc2)
		a[7] = bc2 ^ (bc4 &^ bc3)
		a[8] = bc3 ^ (bc0 &^ bc4)
		a[9] = bc4 ^ (bc1 &^ bc0)

		t = a[10] ^ d0
		bc4 = t<<18 | t>>(64-18)
		t = a[11] ^ d1
		bc0 = t<<1 | t>>(64-1)
		t = a[12] ^ d2
		bc1 = t<<6 | t>>(64-6)
		t = a[13] ^ d3
		bc2 = t<<25 | t>>(64-25)
		t = a[14] ^ d4
		bc3 = t<<8 | t>>(64-8)
		a[10] = bc0 ^ (bc2 &^ bc1)
		a[11] = bc1 ^ (bc3 &^ bc2)
		a[12] = bc2 ^ (bc4 &^ bc3)
		a[13] = bc3 ^ (bc0 &^ bc4)
		a[14] = bc4 ^ (bc1 &^ bc0)

		t = a[15] ^ d0
		bc1 = t<<36 | t>>(64-36)
		t = a[16] ^ d1
		bc2 = t<<10 | t>>(64-10)
		t = a[17] ^ d2
		bc3 = t<<15 | t>>(64-15)
		t = a[18] ^ d3
		bc4 = t<<56 | t>>(64-56)
		t = a[19] ^ d4
		bc0 = t<<27 | t>>(64-27)
		a[15] = bc0 ^ (bc2 &^ bc1)
		a[16] = bc1 ^ (bc3 &^ bc2)
		a[17] = bc2 ^ (bc4 &^ bc3)
		a[18] = bc3 ^ (bc0 &^ bc4)
		a[19] = bc4 ^ (bc1 &^ bc0)

		t = a[20] ^ d0
		bc3 = t<<41 | t>>(64-41)
		t = a[21] ^ d1
		bc4 = t<<2 | t>>(64-2)
		t = a[22] ^ d2
		bc0 = t<<62 | t>>(64-62)
		t = a[23] ^ d3
		bc1 = t<<55 | t>>(64-55)
		t = a[24] ^ d4
		bc2 = t<<39 | t>>(64-39)
		a[20] = bc0 ^ (bc2 &^ bc1)
		a[21] = bc1 ^ (bc3 &^ bc2)
		a[22] = bc2 ^ (bc4 &^ bc3)
		a[23] = bc3 ^ (bc0 &^ bc4)
		a[24] = bc4 ^ (bc1 &^ bc0)
	}
}

func keccakF1600Bytes(s *[25 * 8]byte) {
	// TODO/perf: This could be split out to a separate set of files so
	// that architectures that don't care about memory alignment that
	// aren't amd64 do not have to take the added overhead.
	//
	// But if there are people using this library on 386, ppc64,
	// ppc64le or s390x, they can submit a PR for it.

	var a [25]uint64

	for i := 0; i < 25; i++ {
		a[i] = binary.LittleEndian.Uint64(s[i*8:])
	}

	keccakF1600(&a)

	for i := 0; i < 25; i++ {
		binary.LittleEndian.PutUint64(s[i*8:], a[i])
	}
}
