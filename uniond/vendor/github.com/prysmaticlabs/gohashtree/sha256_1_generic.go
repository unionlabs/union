/*
MIT License

Copyright (c) 2021-2022 Prysmatic Labs

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
package gohashtree

import (
	"encoding/binary"
	"math/bits"
)

const (
	init0 = uint32(0x6A09E667)
	init1 = uint32(0xBB67AE85)
	init2 = uint32(0x3C6EF372)
	init3 = uint32(0xA54FF53A)
	init4 = uint32(0x510E527F)
	init5 = uint32(0x9B05688C)
	init6 = uint32(0x1F83D9AB)
	init7 = uint32(0x5BE0CD19)
)

var _P = []uint32{
	0xc28a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
	0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
	0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
	0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf374,
	0x649b69c1, 0xf0fe4786, 0x0fe1edc6, 0x240cf254,
	0x4fe9346f, 0x6cc984be, 0x61b9411e, 0x16f988fa,
	0xf2c65152, 0xa88e5a6d, 0xb019fc65, 0xb9d99ec7,
	0x9a1231c3, 0xe70eeaa0, 0xfdb1232b, 0xc7353eb0,
	0x3069bad5, 0xcb976d5f, 0x5a0f118f, 0xdc1eeefd,
	0x0a35b689, 0xde0b7a04, 0x58f4ca9d, 0xe15d5b16,
	0x007f3e86, 0x37088980, 0xa507ea32, 0x6fab9537,
	0x17406110, 0x0d8cd6f1, 0xcdaa3b6d, 0xc0bbbe37,
	0x83613bda, 0xdb48a363, 0x0b02e931, 0x6fd15ca7,
	0x521afaca, 0x31338431, 0x6ed41a95, 0x6d437890,
	0xc39c91f2, 0x9eccabbd, 0xb5c9a0e6, 0x532fb63c,
	0xd2c741c6, 0x07237ea3, 0xa4954b68, 0x4c191d76,
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

func sha256_1_generic(digests [][32]byte, p [][32]byte) {
	var w [16]uint32
	for k := 0; k < len(p)/2; k++ {
		// First 16 rounds
		a, b, c, d, e, f, g, h := init0, init1, init2, init3, init4, init5, init6, init7
		for i := 0; i < 8; i++ {
			j := i * 4
			w[i] = uint32(p[2*k][j])<<24 | uint32(p[2*k][j+1])<<16 | uint32(p[2*k][j+2])<<8 | uint32(p[2*k][j+3])
			t1 := h + ((bits.RotateLeft32(e, -6)) ^ (bits.RotateLeft32(e, -11)) ^ (bits.RotateLeft32(e, -25))) + ((e & f) ^ (^e & g)) + _K[i] + w[i]

			t2 := ((bits.RotateLeft32(a, -2)) ^ (bits.RotateLeft32(a, -13)) ^ (bits.RotateLeft32(a, -22))) + ((a & b) ^ (a & c) ^ (b & c))

			h = g
			g = f
			f = e
			e = d + t1
			d = c
			c = b
			b = a
			a = t1 + t2
		}
		for i := 8; i < 16; i++ {
			j := (i - 8) * 4
			w[i] = uint32(p[2*k+1][j])<<24 | uint32(p[2*k+1][j+1])<<16 | uint32(p[2*k+1][j+2])<<8 | uint32(p[2*k+1][j+3])
			t1 := h + ((bits.RotateLeft32(e, -6)) ^ (bits.RotateLeft32(e, -11)) ^ (bits.RotateLeft32(e, -25))) + ((e & f) ^ (^e & g)) + _K[i] + w[i]

			t2 := ((bits.RotateLeft32(a, -2)) ^ (bits.RotateLeft32(a, -13)) ^ (bits.RotateLeft32(a, -22))) + ((a & b) ^ (a & c) ^ (b & c))

			h = g
			g = f
			f = e
			e = d + t1
			d = c
			c = b
			b = a
			a = t1 + t2
		}
		// Last 48 rounds
		for i := 16; i < 64; i++ {
			v1 := w[(i-2)%16]
			t1 := (bits.RotateLeft32(v1, -17)) ^ (bits.RotateLeft32(v1, -19)) ^ (v1 >> 10)
			v2 := w[(i-15)%16]
			t2 := (bits.RotateLeft32(v2, -7)) ^ (bits.RotateLeft32(v2, -18)) ^ (v2 >> 3)
			w[i%16] += t1 + w[(i-7)%16] + t2

			t1 = h + ((bits.RotateLeft32(e, -6)) ^ (bits.RotateLeft32(e, -11)) ^ (bits.RotateLeft32(e, -25))) + ((e & f) ^ (^e & g)) + _K[i] + w[i%16]
			t2 = ((bits.RotateLeft32(a, -2)) ^ (bits.RotateLeft32(a, -13)) ^ (bits.RotateLeft32(a, -22))) + ((a & b) ^ (a & c) ^ (b & c))
			h = g
			g = f
			f = e
			e = d + t1
			d = c
			c = b
			b = a
			a = t1 + t2
		}
		// Add original digest
		a += init0
		b += init1
		c += init2
		d += init3
		e += init4
		f += init5
		g += init6
		h += init7

		h0, h1, h2, h3, h4, h5, h6, h7 := a, b, c, d, e, f, g, h
		// Rounds with padding
		for i := 0; i < 64; i++ {
			t1 := h + ((bits.RotateLeft32(e, -6)) ^ (bits.RotateLeft32(e, -11)) ^ (bits.RotateLeft32(e, -25))) + ((e & f) ^ (^e & g)) + _P[i]

			t2 := ((bits.RotateLeft32(a, -2)) ^ (bits.RotateLeft32(a, -13)) ^ (bits.RotateLeft32(a, -22))) + ((a & b) ^ (a & c) ^ (b & c))

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

		var dig [32]byte
		binary.BigEndian.PutUint32(dig[0:4], h0)
		binary.BigEndian.PutUint32(dig[4:8], h1)
		binary.BigEndian.PutUint32(dig[8:12], h2)
		binary.BigEndian.PutUint32(dig[12:16], h3)
		binary.BigEndian.PutUint32(dig[16:20], h4)
		binary.BigEndian.PutUint32(dig[20:24], h5)
		binary.BigEndian.PutUint32(dig[24:28], h6)
		binary.BigEndian.PutUint32(dig[28:32], h7)
		(digests)[k] = dig
	}
}
