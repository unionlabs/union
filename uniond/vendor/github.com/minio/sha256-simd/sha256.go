/*
 * Minio Cloud Storage, (C) 2016 Minio, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package sha256

import (
	"crypto/sha256"
	"encoding/binary"
	"hash"
	"runtime"

	"github.com/klauspost/cpuid/v2"
)

// Size - The size of a SHA256 checksum in bytes.
const Size = 32

// BlockSize - The blocksize of SHA256 in bytes.
const BlockSize = 64

const (
	chunk = BlockSize
	init0 = 0x6A09E667
	init1 = 0xBB67AE85
	init2 = 0x3C6EF372
	init3 = 0xA54FF53A
	init4 = 0x510E527F
	init5 = 0x9B05688C
	init6 = 0x1F83D9AB
	init7 = 0x5BE0CD19
)

// digest represents the partial evaluation of a checksum.
type digest struct {
	h   [8]uint32
	x   [chunk]byte
	nx  int
	len uint64
}

// Reset digest back to default
func (d *digest) Reset() {
	d.h[0] = init0
	d.h[1] = init1
	d.h[2] = init2
	d.h[3] = init3
	d.h[4] = init4
	d.h[5] = init5
	d.h[6] = init6
	d.h[7] = init7
	d.nx = 0
	d.len = 0
}

type blockfuncType int

const (
	blockfuncGeneric blockfuncType = iota
	blockfuncSha     blockfuncType = iota
	blockfuncArm     blockfuncType = iota
)

var blockfunc blockfuncType

func init() {
	blockfunc = blockfuncGeneric
	switch {
	case hasSHAExtensions():
		blockfunc = blockfuncSha
	case hasArmSha2():
		blockfunc = blockfuncArm
	default:
		blockfunc = blockfuncGeneric
	}
}

var avx512 = cpuid.CPU.Supports(cpuid.AVX512F, cpuid.AVX512DQ, cpuid.AVX512BW, cpuid.AVX512VL)

// hasSHAExtensions return  whether the cpu supports SHA extensions.
func hasSHAExtensions() bool {
	return cpuid.CPU.Supports(cpuid.SHA, cpuid.SSSE3, cpuid.SSE4) && runtime.GOARCH == "amd64"
}

// New returns a new hash.Hash computing the SHA256 checksum.
func New() hash.Hash {
	if blockfunc != blockfuncGeneric {
		d := new(digest)
		d.Reset()
		return d
	}
	// Fallback to the standard golang implementation
	// if no features were found.
	return sha256.New()
}

// Sum256 - single caller sha256 helper
func Sum256(data []byte) (result [Size]byte) {
	var d digest
	d.Reset()
	d.Write(data)
	result = d.checkSum()
	return
}

// Return size of checksum
func (d *digest) Size() int { return Size }

// Return blocksize of checksum
func (d *digest) BlockSize() int { return BlockSize }

// Write to digest
func (d *digest) Write(p []byte) (nn int, err error) {
	nn = len(p)
	d.len += uint64(nn)
	if d.nx > 0 {
		n := copy(d.x[d.nx:], p)
		d.nx += n
		if d.nx == chunk {
			block(d, d.x[:])
			d.nx = 0
		}
		p = p[n:]
	}
	if len(p) >= chunk {
		n := len(p) &^ (chunk - 1)
		block(d, p[:n])
		p = p[n:]
	}
	if len(p) > 0 {
		d.nx = copy(d.x[:], p)
	}
	return
}

// Return sha256 sum in bytes
func (d *digest) Sum(in []byte) []byte {
	// Make a copy of d0 so that caller can keep writing and summing.
	d0 := *d
	hash := d0.checkSum()
	return append(in, hash[:]...)
}

// Intermediate checksum function
func (d *digest) checkSum() (digest [Size]byte) {
	n := d.nx

	var k [64]byte
	copy(k[:], d.x[:n])

	k[n] = 0x80

	if n >= 56 {
		block(d, k[:])

		// clear block buffer - go compiles this to optimal 1x xorps + 4x movups
		// unfortunately expressing this more succinctly results in much worse code
		k[0] = 0
		k[1] = 0
		k[2] = 0
		k[3] = 0
		k[4] = 0
		k[5] = 0
		k[6] = 0
		k[7] = 0
		k[8] = 0
		k[9] = 0
		k[10] = 0
		k[11] = 0
		k[12] = 0
		k[13] = 0
		k[14] = 0
		k[15] = 0
		k[16] = 0
		k[17] = 0
		k[18] = 0
		k[19] = 0
		k[20] = 0
		k[21] = 0
		k[22] = 0
		k[23] = 0
		k[24] = 0
		k[25] = 0
		k[26] = 0
		k[27] = 0
		k[28] = 0
		k[29] = 0
		k[30] = 0
		k[31] = 0
		k[32] = 0
		k[33] = 0
		k[34] = 0
		k[35] = 0
		k[36] = 0
		k[37] = 0
		k[38] = 0
		k[39] = 0
		k[40] = 0
		k[41] = 0
		k[42] = 0
		k[43] = 0
		k[44] = 0
		k[45] = 0
		k[46] = 0
		k[47] = 0
		k[48] = 0
		k[49] = 0
		k[50] = 0
		k[51] = 0
		k[52] = 0
		k[53] = 0
		k[54] = 0
		k[55] = 0
		k[56] = 0
		k[57] = 0
		k[58] = 0
		k[59] = 0
		k[60] = 0
		k[61] = 0
		k[62] = 0
		k[63] = 0
	}
	binary.BigEndian.PutUint64(k[56:64], uint64(d.len)<<3)
	block(d, k[:])

	{
		const i = 0
		binary.BigEndian.PutUint32(digest[i*4:i*4+4], d.h[i])
	}
	{
		const i = 1
		binary.BigEndian.PutUint32(digest[i*4:i*4+4], d.h[i])
	}
	{
		const i = 2
		binary.BigEndian.PutUint32(digest[i*4:i*4+4], d.h[i])
	}
	{
		const i = 3
		binary.BigEndian.PutUint32(digest[i*4:i*4+4], d.h[i])
	}
	{
		const i = 4
		binary.BigEndian.PutUint32(digest[i*4:i*4+4], d.h[i])
	}
	{
		const i = 5
		binary.BigEndian.PutUint32(digest[i*4:i*4+4], d.h[i])
	}
	{
		const i = 6
		binary.BigEndian.PutUint32(digest[i*4:i*4+4], d.h[i])
	}
	{
		const i = 7
		binary.BigEndian.PutUint32(digest[i*4:i*4+4], d.h[i])
	}

	return
}

func block(dig *digest, p []byte) {
	if blockfunc == blockfuncSha {
		blockShaGo(dig, p)
	} else if blockfunc == blockfuncArm {
		blockArmGo(dig, p)
	} else if blockfunc == blockfuncGeneric {
		blockGeneric(dig, p)
	}
}

func blockGeneric(dig *digest, p []byte) {
	var w [64]uint32
	h0, h1, h2, h3, h4, h5, h6, h7 := dig.h[0], dig.h[1], dig.h[2], dig.h[3], dig.h[4], dig.h[5], dig.h[6], dig.h[7]
	for len(p) >= chunk {
		// Can interlace the computation of w with the
		// rounds below if needed for speed.
		for i := 0; i < 16; i++ {
			j := i * 4
			w[i] = uint32(p[j])<<24 | uint32(p[j+1])<<16 | uint32(p[j+2])<<8 | uint32(p[j+3])
		}
		for i := 16; i < 64; i++ {
			v1 := w[i-2]
			t1 := (v1>>17 | v1<<(32-17)) ^ (v1>>19 | v1<<(32-19)) ^ (v1 >> 10)
			v2 := w[i-15]
			t2 := (v2>>7 | v2<<(32-7)) ^ (v2>>18 | v2<<(32-18)) ^ (v2 >> 3)
			w[i] = t1 + w[i-7] + t2 + w[i-16]
		}

		a, b, c, d, e, f, g, h := h0, h1, h2, h3, h4, h5, h6, h7

		for i := 0; i < 64; i++ {
			t1 := h + ((e>>6 | e<<(32-6)) ^ (e>>11 | e<<(32-11)) ^ (e>>25 | e<<(32-25))) + ((e & f) ^ (^e & g)) + _K[i] + w[i]

			t2 := ((a>>2 | a<<(32-2)) ^ (a>>13 | a<<(32-13)) ^ (a>>22 | a<<(32-22))) + ((a & b) ^ (a & c) ^ (b & c))

			h = g
			g = f
			f = e
			e = d + t1
			d = c
			c = b
			b = a
			a = t1 + t2
		}

		h0 += a
		h1 += b
		h2 += c
		h3 += d
		h4 += e
		h5 += f
		h6 += g
		h7 += h

		p = p[chunk:]
	}

	dig.h[0], dig.h[1], dig.h[2], dig.h[3], dig.h[4], dig.h[5], dig.h[6], dig.h[7] = h0, h1, h2, h3, h4, h5, h6, h7
}

var _K = []uint32{
	0x428a2f98,
	0x71374491,
	0xb5c0fbcf,
	0xe9b5dba5,
	0x3956c25b,
	0x59f111f1,
	0x923f82a4,
	0xab1c5ed5,
	0xd807aa98,
	0x12835b01,
	0x243185be,
	0x550c7dc3,
	0x72be5d74,
	0x80deb1fe,
	0x9bdc06a7,
	0xc19bf174,
	0xe49b69c1,
	0xefbe4786,
	0x0fc19dc6,
	0x240ca1cc,
	0x2de92c6f,
	0x4a7484aa,
	0x5cb0a9dc,
	0x76f988da,
	0x983e5152,
	0xa831c66d,
	0xb00327c8,
	0xbf597fc7,
	0xc6e00bf3,
	0xd5a79147,
	0x06ca6351,
	0x14292967,
	0x27b70a85,
	0x2e1b2138,
	0x4d2c6dfc,
	0x53380d13,
	0x650a7354,
	0x766a0abb,
	0x81c2c92e,
	0x92722c85,
	0xa2bfe8a1,
	0xa81a664b,
	0xc24b8b70,
	0xc76c51a3,
	0xd192e819,
	0xd6990624,
	0xf40e3585,
	0x106aa070,
	0x19a4c116,
	0x1e376c08,
	0x2748774c,
	0x34b0bcb5,
	0x391c0cb3,
	0x4ed8aa4a,
	0x5b9cca4f,
	0x682e6ff3,
	0x748f82ee,
	0x78a5636f,
	0x84c87814,
	0x8cc70208,
	0x90befffa,
	0xa4506ceb,
	0xbef9a3f7,
	0xc67178f2,
}
