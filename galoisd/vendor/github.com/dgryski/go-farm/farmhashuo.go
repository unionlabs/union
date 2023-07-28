package farm

import (
	"encoding/binary"
	"math/bits"
)

func uoH(x, y, mul uint64, r uint) uint64 {
	a := (x ^ y) * mul
	a ^= (a >> 47)
	b := (y ^ a) * mul
	return bits.RotateLeft64(b, -int(r)) * mul
}

// Hash64WithSeeds hashes a byte slice and two uint64 seeds and returns a uint64 hash value
func Hash64WithSeeds(s []byte, seed0, seed1 uint64) uint64 {
	slen := len(s)
	if slen <= 64 {
		return naHash64WithSeeds(s, seed0, seed1)
	}

	// For strings over 64 bytes we loop.
	// Internal state consists of 64 bytes: u, v, w, x, y, and z.
	x := seed0
	y := seed1*k2 + 113
	z := shiftMix(y*k2) * k2
	v := uint128{seed0, seed1}
	var w uint128
	u := x - z
	x *= k2
	mul := k2 + (u & 0x82)

	// Set end so that after the loop we have 1 to 64 bytes left to process.
	endIdx := ((slen - 1) / 64) * 64
	last64Idx := endIdx + ((slen - 1) & 63) - 63
	last64 := s[last64Idx:]

	for len(s) > 64 {
		a0 := binary.LittleEndian.Uint64(s[0 : 0+8])
		a1 := binary.LittleEndian.Uint64(s[8 : 8+8])
		a2 := binary.LittleEndian.Uint64(s[16 : 16+8])
		a3 := binary.LittleEndian.Uint64(s[24 : 24+8])
		a4 := binary.LittleEndian.Uint64(s[32 : 32+8])
		a5 := binary.LittleEndian.Uint64(s[40 : 40+8])
		a6 := binary.LittleEndian.Uint64(s[48 : 48+8])
		a7 := binary.LittleEndian.Uint64(s[56 : 56+8])
		x += a0 + a1
		y += a2
		z += a3
		v.lo += a4
		v.hi += a5 + a1
		w.lo += a6
		w.hi += a7

		x = bits.RotateLeft64(x, -26)
		x *= 9
		y = bits.RotateLeft64(y, -29)
		z *= mul
		v.lo = bits.RotateLeft64(v.lo, -33)
		v.hi = bits.RotateLeft64(v.hi, -30)
		w.lo ^= x
		w.lo *= 9
		z = bits.RotateLeft64(z, -32)
		z += w.hi
		w.hi += z
		z *= 9
		u, y = y, u

		z += a0 + a6
		v.lo += a2
		v.hi += a3
		w.lo += a4
		w.hi += a5 + a6
		x += a1
		y += a7

		y += v.lo
		v.lo += x - y
		v.hi += w.lo
		w.lo += v.hi
		w.hi += x - y
		x += w.hi
		w.hi = bits.RotateLeft64(w.hi, -34)
		u, z = z, u
		s = s[64:]
	}
	// Make s point to the last 64 bytes of input.
	s = last64
	u *= 9
	v.hi = bits.RotateLeft64(v.hi, -28)
	v.lo = bits.RotateLeft64(v.lo, -20)
	w.lo += (uint64(slen-1) & 63)
	u += y
	y += u
	x = bits.RotateLeft64(y-x+v.lo+binary.LittleEndian.Uint64(s[8:8+8]), -37) * mul
	y = bits.RotateLeft64(y^v.hi^binary.LittleEndian.Uint64(s[48:48+8]), -42) * mul
	x ^= w.hi * 9
	y += v.lo + binary.LittleEndian.Uint64(s[40:40+8])
	z = bits.RotateLeft64(z+w.lo, -33) * mul
	v.lo, v.hi = weakHashLen32WithSeeds(s, v.hi*mul, x+w.lo)
	w.lo, w.hi = weakHashLen32WithSeeds(s[32:], z+w.hi, y+binary.LittleEndian.Uint64(s[16:16+8]))
	return uoH(hashLen16Mul(v.lo+x, w.lo^y, mul)+z-u,
		uoH(v.hi+y, w.hi+z, k2, 30)^x,
		k2,
		31)
}

// Hash64WithSeed hashes a byte slice and a uint64 seed and returns a uint64 hash value
func Hash64WithSeed(s []byte, seed uint64) uint64 {
	if len(s) <= 64 {
		return naHash64WithSeed(s, seed)
	}
	return Hash64WithSeeds(s, 0, seed)
}

// Hash64 hashes a byte slice and returns a uint64 hash value
func uoHash64(s []byte) uint64 {
	if len(s) <= 64 {
		return naHash64(s)
	}
	return Hash64WithSeeds(s, 81, 0)
}
