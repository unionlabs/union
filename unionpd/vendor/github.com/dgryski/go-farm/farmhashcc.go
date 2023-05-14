package farm

import (
	"encoding/binary"
	"math/bits"
)

// This file provides a 32-bit hash equivalent to CityHash32 (v1.1.1)
// and a 128-bit hash equivalent to CityHash128 (v1.1.1).  It also provides
// a seeded 32-bit hash function similar to CityHash32.

func hash32Len13to24Seed(s []byte, seed uint32) uint32 {
	slen := len(s)
	a := binary.LittleEndian.Uint32(s[-4+(slen>>1) : -4+(slen>>1)+4])
	b := binary.LittleEndian.Uint32(s[4 : 4+4])
	c := binary.LittleEndian.Uint32(s[slen-8 : slen-8+4])
	d := binary.LittleEndian.Uint32(s[(slen >> 1) : (slen>>1)+4])
	e := binary.LittleEndian.Uint32(s[0 : 0+4])
	f := binary.LittleEndian.Uint32(s[slen-4 : slen-4+4])
	h := d*c1 + uint32(slen) + seed
	a = bits.RotateLeft32(a, -12) + f
	h = mur(c, h) + a
	a = bits.RotateLeft32(a, -3) + c
	h = mur(e, h) + a
	a = bits.RotateLeft32(a+f, -12) + d
	h = mur(b^seed, h) + a
	return fmix(h)
}

func hash32Len0to4(s []byte, seed uint32) uint32 {
	slen := len(s)
	b := seed
	c := uint32(9)
	for i := 0; i < slen; i++ {
		v := int8(s[i])
		b = (b * c1) + uint32(v)
		c ^= b
	}
	return fmix(mur(b, mur(uint32(slen), c)))
}

func hash128to64(x uint128) uint64 {
	// Murmur-inspired hashing.
	const mul uint64 = 0x9ddfea08eb382d69
	a := (x.lo ^ x.hi) * mul
	a ^= (a >> 47)
	b := (x.hi ^ a) * mul
	b ^= (b >> 47)
	b *= mul
	return b
}

type uint128 struct {
	lo uint64
	hi uint64
}

// A subroutine for CityHash128().  Returns a decent 128-bit hash for strings
// of any length representable in signed long.  Based on City and Murmur.
func cityMurmur(s []byte, seed uint128) uint128 {
	slen := len(s)
	a := seed.lo
	b := seed.hi
	var c uint64
	var d uint64
	l := slen - 16
	if l <= 0 { // len <= 16
		a = shiftMix(a*k1) * k1
		c = b*k1 + hashLen0to16(s)
		if slen >= 8 {
			d = shiftMix(a + binary.LittleEndian.Uint64(s[0:0+8]))
		} else {
			d = shiftMix(a + c)
		}
	} else { // len > 16
		c = hashLen16(binary.LittleEndian.Uint64(s[slen-8:slen-8+8])+k1, a)
		d = hashLen16(b+uint64(slen), c+binary.LittleEndian.Uint64(s[slen-16:slen-16+8]))
		a += d
		for {
			a ^= shiftMix(binary.LittleEndian.Uint64(s[0:0+8])*k1) * k1
			a *= k1
			b ^= a
			c ^= shiftMix(binary.LittleEndian.Uint64(s[8:8+8])*k1) * k1
			c *= k1
			d ^= c
			s = s[16:]
			l -= 16
			if l <= 0 {
				break
			}
		}
	}
	a = hashLen16(a, c)
	b = hashLen16(d, b)
	return uint128{a ^ b, hashLen16(b, a)}
}

func cityHash128WithSeed(s []byte, seed uint128) uint128 {
	slen := len(s)
	if slen < 128 {
		return cityMurmur(s, seed)
	}

	endIdx := ((slen - 1) / 128) * 128
	lastBlockIdx := endIdx + ((slen - 1) & 127) - 127
	last := s[lastBlockIdx:]

	// We expect len >= 128 to be the common case.  Keep 56 bytes of state:
	// v, w, x, y, and z.
	var v1, v2 uint64
	var w1, w2 uint64
	x := seed.lo
	y := seed.hi
	z := uint64(slen) * k1
	v1 = bits.RotateLeft64(y^k1, -49)*k1 + binary.LittleEndian.Uint64(s[0:0+8])
	v2 = bits.RotateLeft64(v1, -42)*k1 + binary.LittleEndian.Uint64(s[8:8+8])
	w1 = bits.RotateLeft64(y+z, -35)*k1 + x
	w2 = bits.RotateLeft64(x+binary.LittleEndian.Uint64(s[88:88+8]), -53) * k1

	// This is the same inner loop as CityHash64(), manually unrolled.
	for {
		x = bits.RotateLeft64(x+y+v1+binary.LittleEndian.Uint64(s[8:8+8]), -37) * k1
		y = bits.RotateLeft64(y+v2+binary.LittleEndian.Uint64(s[48:48+8]), -42) * k1
		x ^= w2
		y += v1 + binary.LittleEndian.Uint64(s[40:40+8])
		z = bits.RotateLeft64(z+w1, -33) * k1
		v1, v2 = weakHashLen32WithSeeds(s, v2*k1, x+w1)
		w1, w2 = weakHashLen32WithSeeds(s[32:], z+w2, y+binary.LittleEndian.Uint64(s[16:16+8]))
		z, x = x, z
		s = s[64:]
		x = bits.RotateLeft64(x+y+v1+binary.LittleEndian.Uint64(s[8:8+8]), -37) * k1
		y = bits.RotateLeft64(y+v2+binary.LittleEndian.Uint64(s[48:48+8]), -42) * k1
		x ^= w2
		y += v1 + binary.LittleEndian.Uint64(s[40:40+8])
		z = bits.RotateLeft64(z+w1, -33) * k1
		v1, v2 = weakHashLen32WithSeeds(s, v2*k1, x+w1)
		w1, w2 = weakHashLen32WithSeeds(s[32:], z+w2, y+binary.LittleEndian.Uint64(s[16:16+8]))
		z, x = x, z
		s = s[64:]
		slen -= 128
		if slen < 128 {
			break
		}
	}
	x += bits.RotateLeft64(v1+z, -49) * k0
	y = y*k0 + bits.RotateLeft64(w2, -37)
	z = z*k0 + bits.RotateLeft64(w1, -27)
	w1 *= 9
	v1 *= k0
	// If 0 < len < 128, hash up to 4 chunks of 32 bytes each from the end of s.
	for tailDone := 0; tailDone < slen; {
		tailDone += 32
		y = bits.RotateLeft64(x+y, -42)*k0 + v2
		w1 += binary.LittleEndian.Uint64(last[128-tailDone+16 : 128-tailDone+16+8])
		x = x*k0 + w1
		z += w2 + binary.LittleEndian.Uint64(last[128-tailDone:128-tailDone+8])
		w2 += v1
		v1, v2 = weakHashLen32WithSeeds(last[128-tailDone:], v1+z, v2)
		v1 *= k0
	}

	// At this point our 56 bytes of state should contain more than
	// enough information for a strong 128-bit hash.  We use two
	// different 56-byte-to-8-byte hashes to get a 16-byte final result.
	x = hashLen16(x, v1)
	y = hashLen16(y+z, w1)
	return uint128{hashLen16(x+v2, w2) + y,
		hashLen16(x+w2, y+v2)}
}

func cityHash128(s []byte) uint128 {
	slen := len(s)
	if slen >= 16 {
		return cityHash128WithSeed(s[16:], uint128{binary.LittleEndian.Uint64(s[0 : 0+8]), binary.LittleEndian.Uint64(s[8:8+8]) + k0})
	}
	return cityHash128WithSeed(s, uint128{k0, k1})
}

// Fingerprint128 is a 128-bit fingerprint function for byte-slices
func Fingerprint128(s []byte) (lo, hi uint64) {
	h := cityHash128(s)
	return h.lo, h.hi
}

// Hash128 is a 128-bit hash function for byte-slices
func Hash128(s []byte) (lo, hi uint64) {
	return Fingerprint128(s)
}

// Hash128WithSeed is a 128-bit hash function for byte-slices and a 128-bit seed
func Hash128WithSeed(s []byte, seed0, seed1 uint64) (lo, hi uint64) {
	h := cityHash128WithSeed(s, uint128{seed0, seed1})
	return h.lo, h.hi
}
