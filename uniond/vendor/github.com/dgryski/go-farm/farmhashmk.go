package farm

import (
	"encoding/binary"
	"math/bits"
)

func hash32Len5to12(s []byte, seed uint32) uint32 {
	slen := len(s)
	a := uint32(len(s))
	b := uint32(len(s) * 5)
	c := uint32(9)
	d := b + seed
	a += binary.LittleEndian.Uint32(s[0 : 0+4])
	b += binary.LittleEndian.Uint32(s[slen-4 : slen-4+4])
	c += binary.LittleEndian.Uint32(s[((slen >> 1) & 4) : ((slen>>1)&4)+4])
	return fmix(seed ^ mur(c, mur(b, mur(a, d))))
}

// Hash32 hashes a byte slice and returns a uint32 hash value
func Hash32(s []byte) uint32 {

	slen := len(s)

	if slen <= 24 {
		if slen <= 12 {
			if slen <= 4 {
				return hash32Len0to4(s, 0)
			}
			return hash32Len5to12(s, 0)
		}
		return hash32Len13to24Seed(s, 0)
	}

	// len > 24
	h := uint32(slen)
	g := c1 * uint32(slen)
	f := g
	a0 := bits.RotateLeft32(binary.LittleEndian.Uint32(s[slen-4:slen-4+4])*c1, -17) * c2
	a1 := bits.RotateLeft32(binary.LittleEndian.Uint32(s[slen-8:slen-8+4])*c1, -17) * c2
	a2 := bits.RotateLeft32(binary.LittleEndian.Uint32(s[slen-16:slen-16+4])*c1, -17) * c2
	a3 := bits.RotateLeft32(binary.LittleEndian.Uint32(s[slen-12:slen-12+4])*c1, -17) * c2
	a4 := bits.RotateLeft32(binary.LittleEndian.Uint32(s[slen-20:slen-20+4])*c1, -17) * c2
	h ^= a0
	h = bits.RotateLeft32(h, -19)
	h = h*5 + 0xe6546b64
	h ^= a2
	h = bits.RotateLeft32(h, -19)
	h = h*5 + 0xe6546b64
	g ^= a1
	g = bits.RotateLeft32(g, -19)
	g = g*5 + 0xe6546b64
	g ^= a3
	g = bits.RotateLeft32(g, -19)
	g = g*5 + 0xe6546b64
	f += a4
	f = bits.RotateLeft32(f, -19) + 113
	for len(s) > 20 {
		a := binary.LittleEndian.Uint32(s[0 : 0+4])
		b := binary.LittleEndian.Uint32(s[4 : 4+4])
		c := binary.LittleEndian.Uint32(s[8 : 8+4])
		d := binary.LittleEndian.Uint32(s[12 : 12+4])
		e := binary.LittleEndian.Uint32(s[16 : 16+4])
		h += a
		g += b
		f += c
		h = mur(d, h) + e
		g = mur(c, g) + a
		f = mur(b+e*c1, f) + d
		f += g
		g += f
		s = s[20:]
	}
	g = bits.RotateLeft32(g, -11) * c1
	g = bits.RotateLeft32(g, -17) * c1
	f = bits.RotateLeft32(f, -11) * c1
	f = bits.RotateLeft32(f, -17) * c1
	h = bits.RotateLeft32(h+g, -19)
	h = h*5 + 0xe6546b64
	h = bits.RotateLeft32(h, -17) * c1
	h = bits.RotateLeft32(h+f, -19)
	h = h*5 + 0xe6546b64
	h = bits.RotateLeft32(h, -17) * c1
	return h
}

// Hash32WithSeed hashes a byte slice and a uint32 seed and returns a uint32 hash value
func Hash32WithSeed(s []byte, seed uint32) uint32 {
	slen := len(s)

	if slen <= 24 {
		if slen >= 13 {
			return hash32Len13to24Seed(s, seed*c1)
		}
		if slen >= 5 {
			return hash32Len5to12(s, seed)
		}
		return hash32Len0to4(s, seed)
	}
	h := hash32Len13to24Seed(s[:24], seed^uint32(slen))
	return mur(Hash32(s[24:])+seed, h)
}
