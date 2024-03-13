package sha2

import (
	"github.com/consensys/gnark/std/math/uints"
)

var _K = uints.NewU32Array([]uint32{
	0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
	0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
	0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
	0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
	0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
	0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
	0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
	0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
})

func Permute(uapi *uints.BinaryField[uints.U32], currentHash [8]uints.U32, p [64]uints.U8) (newHash [8]uints.U32) {
	var w [64]uints.U32

	for i := 0; i < 16; i++ {
		w[i] = uapi.PackMSB(p[4*i], p[4*i+1], p[4*i+2], p[4*i+3])
	}

	for i := 16; i < 64; i++ {
		v1 := w[i-2]
		t1 := uapi.Xor(
			uapi.Lrot(v1, -17),
			uapi.Lrot(v1, -19),
			uapi.Rshift(v1, 10),
		)
		v2 := w[i-15]
		t2 := uapi.Xor(
			uapi.Lrot(v2, -7),
			uapi.Lrot(v2, -18),
			uapi.Rshift(v2, 3),
		)

		w[i] = uapi.Add(t1, w[i-7], t2, w[i-16])
	}

	a, b, c, d, e, f, g, h := currentHash[0], currentHash[1], currentHash[2], currentHash[3], currentHash[4], currentHash[5], currentHash[6], currentHash[7]

	for i := 0; i < 64; i++ {
		t1 := uapi.Add(
			h,
			uapi.Xor(
				uapi.Lrot(e, -6),
				uapi.Lrot(e, -11),
				uapi.Lrot(e, -25)),
			uapi.Xor(
				uapi.And(e, f),
				uapi.And(
					uapi.Not(e),
					g)),
			_K[i],
			w[i],
		)
		t2 := uapi.Add(
			uapi.Xor(
				uapi.Lrot(a, -2),
				uapi.Lrot(a, -13),
				uapi.Lrot(a, -22)),
			uapi.Xor(
				uapi.And(a, b),
				uapi.And(a, c),
				uapi.And(b, c)),
		)

		h = g
		g = f
		f = e
		e = uapi.Add(d, t1)
		d = c
		c = b
		b = a
		a = uapi.Add(t1, t2)
	}

	currentHash[0] = uapi.Add(currentHash[0], a)
	currentHash[1] = uapi.Add(currentHash[1], b)
	currentHash[2] = uapi.Add(currentHash[2], c)
	currentHash[3] = uapi.Add(currentHash[3], d)
	currentHash[4] = uapi.Add(currentHash[4], e)
	currentHash[5] = uapi.Add(currentHash[5], f)
	currentHash[6] = uapi.Add(currentHash[6], g)
	currentHash[7] = uapi.Add(currentHash[7], h)

	return currentHash
}
