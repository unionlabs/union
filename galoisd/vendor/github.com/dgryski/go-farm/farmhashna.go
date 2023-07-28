package farm

import (
	"encoding/binary"
	"math/bits"
)

func shiftMix(val uint64) uint64 {
	return val ^ (val >> 47)
}

func hashLen16(u, v uint64) uint64 {
	return hash128to64(uint128{u, v})
}

func hashLen16Mul(u, v, mul uint64) uint64 {
	// Murmur-inspired hashing.
	a := (u ^ v) * mul
	a ^= (a >> 47)
	b := (v ^ a) * mul
	b ^= (b >> 47)
	b *= mul
	return b
}

func hashLen0to16(s []byte) uint64 {
	slen := uint64(len(s))
	if slen >= 8 {
		mul := k2 + slen*2
		a := binary.LittleEndian.Uint64(s[0:0+8]) + k2
		b := binary.LittleEndian.Uint64(s[int(slen-8) : int(slen-8)+8])
		c := bits.RotateLeft64(b, -37)*mul + a
		d := (bits.RotateLeft64(a, -25) + b) * mul
		return hashLen16Mul(c, d, mul)
	}

	if slen >= 4 {
		mul := k2 + slen*2
		a := binary.LittleEndian.Uint32(s[0 : 0+4])
		return hashLen16Mul(slen+(uint64(a)<<3), uint64(binary.LittleEndian.Uint32(s[int(slen-4):int(slen-4)+4])), mul)
	}
	if slen > 0 {
		a := s[0]
		b := s[slen>>1]
		c := s[slen-1]
		y := uint32(a) + (uint32(b) << 8)
		z := uint32(slen) + (uint32(c) << 2)
		return shiftMix(uint64(y)*k2^uint64(z)*k0) * k2
	}
	return k2
}

// This probably works well for 16-byte strings as well, but it may be overkill
// in that case.
func hashLen17to32(s []byte) uint64 {
	slen := len(s)
	mul := k2 + uint64(slen*2)
	a := binary.LittleEndian.Uint64(s[0:0+8]) * k1
	b := binary.LittleEndian.Uint64(s[8 : 8+8])
	c := binary.LittleEndian.Uint64(s[slen-8:slen-8+8]) * mul
	d := binary.LittleEndian.Uint64(s[slen-16:slen-16+8]) * k2
	return hashLen16Mul(bits.RotateLeft64(a+b, -43)+bits.RotateLeft64(c, -30)+d, a+bits.RotateLeft64(b+k2, -18)+c, mul)
}

// Return a 16-byte hash for 48 bytes.  Quick and dirty.
// Callers do best to use "random-looking" values for a and b.
func weakHashLen32WithSeedsWords(w, x, y, z, a, b uint64) (uint64, uint64) {
	a += w
	b = bits.RotateLeft64(b+a+z, -21)
	c := a
	a += x
	a += y
	b += bits.RotateLeft64(a, -44)
	return a + z, b + c
}

// Return a 16-byte hash for s[0] ... s[31], a, and b.  Quick and dirty.
func weakHashLen32WithSeeds(s []byte, a, b uint64) (uint64, uint64) {
	return weakHashLen32WithSeedsWords(binary.LittleEndian.Uint64(s[0:0+8]),
		binary.LittleEndian.Uint64(s[8:8+8]),
		binary.LittleEndian.Uint64(s[16:16+8]),
		binary.LittleEndian.Uint64(s[24:24+8]),
		a,
		b)
}

// Return an 8-byte hash for 33 to 64 bytes.
func hashLen33to64(s []byte) uint64 {
	slen := len(s)
	mul := k2 + uint64(slen)*2
	a := binary.LittleEndian.Uint64(s[0:0+8]) * k2
	b := binary.LittleEndian.Uint64(s[8 : 8+8])
	c := binary.LittleEndian.Uint64(s[slen-8:slen-8+8]) * mul
	d := binary.LittleEndian.Uint64(s[slen-16:slen-16+8]) * k2
	y := bits.RotateLeft64(a+b, -43) + bits.RotateLeft64(c, -30) + d
	z := hashLen16Mul(y, a+bits.RotateLeft64(b+k2, -18)+c, mul)
	e := binary.LittleEndian.Uint64(s[16:16+8]) * mul
	f := binary.LittleEndian.Uint64(s[24 : 24+8])
	g := (y + binary.LittleEndian.Uint64(s[slen-32:slen-32+8])) * mul
	h := (z + binary.LittleEndian.Uint64(s[slen-24:slen-24+8])) * mul
	return hashLen16Mul(bits.RotateLeft64(e+f, -43)+bits.RotateLeft64(g, -30)+h, e+bits.RotateLeft64(f+a, -18)+g, mul)
}

func naHash64(s []byte) uint64 {
	slen := len(s)
	var seed uint64 = 81
	if slen <= 32 {
		if slen <= 16 {
			return hashLen0to16(s)
		}
		return hashLen17to32(s)
	}
	if slen <= 64 {
		return hashLen33to64(s)
	}
	// For strings over 64 bytes we loop.
	// Internal state consists of 56 bytes: v, w, x, y, and z.
	v := uint128{0, 0}
	w := uint128{0, 0}
	x := seed*k2 + binary.LittleEndian.Uint64(s[0:0+8])
	y := seed*k1 + 113
	z := shiftMix(y*k2+113) * k2
	// Set end so that after the loop we have 1 to 64 bytes left to process.
	endIdx := ((slen - 1) / 64) * 64
	last64Idx := endIdx + ((slen - 1) & 63) - 63
	last64 := s[last64Idx:]
	for len(s) > 64 {
		x = bits.RotateLeft64(x+y+v.lo+binary.LittleEndian.Uint64(s[8:8+8]), -37) * k1
		y = bits.RotateLeft64(y+v.hi+binary.LittleEndian.Uint64(s[48:48+8]), -42) * k1
		x ^= w.hi
		y += v.lo + binary.LittleEndian.Uint64(s[40:40+8])
		z = bits.RotateLeft64(z+w.lo, -33) * k1
		v.lo, v.hi = weakHashLen32WithSeeds(s, v.hi*k1, x+w.lo)
		w.lo, w.hi = weakHashLen32WithSeeds(s[32:], z+w.hi, y+binary.LittleEndian.Uint64(s[16:16+8]))
		x, z = z, x
		s = s[64:]
	}
	mul := k1 + ((z & 0xff) << 1)
	// Make s point to the last 64 bytes of input.
	s = last64
	w.lo += (uint64(slen-1) & 63)
	v.lo += w.lo
	w.lo += v.lo
	x = bits.RotateLeft64(x+y+v.lo+binary.LittleEndian.Uint64(s[8:8+8]), -37) * mul
	y = bits.RotateLeft64(y+v.hi+binary.LittleEndian.Uint64(s[48:48+8]), -42) * mul
	x ^= w.hi * 9
	y += v.lo*9 + binary.LittleEndian.Uint64(s[40:40+8])
	z = bits.RotateLeft64(z+w.lo, -33) * mul
	v.lo, v.hi = weakHashLen32WithSeeds(s, v.hi*mul, x+w.lo)
	w.lo, w.hi = weakHashLen32WithSeeds(s[32:], z+w.hi, y+binary.LittleEndian.Uint64(s[16:16+8]))
	x, z = z, x
	return hashLen16Mul(hashLen16Mul(v.lo, w.lo, mul)+shiftMix(y)*k0+z, hashLen16Mul(v.hi, w.hi, mul)+x, mul)
}

func naHash64WithSeed(s []byte, seed uint64) uint64 {
	return naHash64WithSeeds(s, k2, seed)
}

func naHash64WithSeeds(s []byte, seed0, seed1 uint64) uint64 {
	return hashLen16(naHash64(s)-seed0, seed1)
}
