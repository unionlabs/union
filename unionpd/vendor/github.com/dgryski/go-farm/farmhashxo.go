package farm

import (
	"encoding/binary"
	"math/bits"
)

func h32(s []byte, mul uint64) uint64 {
	slen := len(s)
	a := binary.LittleEndian.Uint64(s[0:0+8]) * k1
	b := binary.LittleEndian.Uint64(s[8 : 8+8])
	c := binary.LittleEndian.Uint64(s[slen-8:slen-8+8]) * mul
	d := binary.LittleEndian.Uint64(s[slen-16:slen-16+8]) * k2
	u := bits.RotateLeft64(a+b, -43) + bits.RotateLeft64(c, -30) + d
	v := a + bits.RotateLeft64(b+k2, -18) + c
	a = shiftMix((u ^ v) * mul)
	b = shiftMix((v ^ a) * mul)
	return b
}

func h32Seeds(s []byte, mul, seed0, seed1 uint64) uint64 {
	slen := len(s)
	a := binary.LittleEndian.Uint64(s[0:0+8]) * k1
	b := binary.LittleEndian.Uint64(s[8 : 8+8])
	c := binary.LittleEndian.Uint64(s[slen-8:slen-8+8]) * mul
	d := binary.LittleEndian.Uint64(s[slen-16:slen-16+8]) * k2
	u := bits.RotateLeft64(a+b, -43) + bits.RotateLeft64(c, -30) + d + seed0
	v := a + bits.RotateLeft64(b+k2, -18) + c + seed1
	a = shiftMix((u ^ v) * mul)
	b = shiftMix((v ^ a) * mul)
	return b
}

func xohashLen33to64(s []byte) uint64 {
	slen := len(s)
	mul0 := k2 - 30
	mul1 := k2 - 30 + 2*uint64(slen)

	var h0 uint64
	{
		s := s[0:32]
		mul := mul0
		slen := len(s)
		a := binary.LittleEndian.Uint64(s[0:0+8]) * k1
		b := binary.LittleEndian.Uint64(s[8 : 8+8])
		c := binary.LittleEndian.Uint64(s[slen-8:slen-8+8]) * mul
		d := binary.LittleEndian.Uint64(s[slen-16:slen-16+8]) * k2
		u := bits.RotateLeft64(a+b, -43) + bits.RotateLeft64(c, -30) + d
		v := a + bits.RotateLeft64(b+k2, -18) + c
		a = shiftMix((u ^ v) * mul)
		b = shiftMix((v ^ a) * mul)
		h0 = b
	}

	var h1 uint64
	{
		s := s[slen-32:]
		mul := mul1
		slen := len(s)
		a := binary.LittleEndian.Uint64(s[0:0+8]) * k1
		b := binary.LittleEndian.Uint64(s[8 : 8+8])
		c := binary.LittleEndian.Uint64(s[slen-8:slen-8+8]) * mul
		d := binary.LittleEndian.Uint64(s[slen-16:slen-16+8]) * k2
		u := bits.RotateLeft64(a+b, -43) + bits.RotateLeft64(c, -30) + d
		v := a + bits.RotateLeft64(b+k2, -18) + c
		a = shiftMix((u ^ v) * mul)
		b = shiftMix((v ^ a) * mul)
		h1 = b
	}

	r := ((h1 * mul1) + h0) * mul1
	return r
}

func xohashLen65to96(s []byte) uint64 {
	slen := len(s)

	mul0 := k2 - 114
	mul1 := k2 - 114 + 2*uint64(slen)
	h0 := h32(s[:32], mul0)
	h1 := h32(s[32:64], mul1)
	h2 := h32Seeds(s[slen-32:], mul1, h0, h1)
	return (h2*9 + (h0 >> 17) + (h1 >> 21)) * mul1
}

func Hash64(s []byte) uint64 {
	slen := len(s)

	if slen <= 32 {
		if slen <= 16 {
			return hashLen0to16(s)
		} else {
			return hashLen17to32(s)
		}
	} else if slen <= 64 {
		return xohashLen33to64(s)
	} else if slen <= 96 {
		return xohashLen65to96(s)
	} else if slen <= 256 {
		return naHash64(s)
	} else {
		return uoHash64(s)
	}
}
