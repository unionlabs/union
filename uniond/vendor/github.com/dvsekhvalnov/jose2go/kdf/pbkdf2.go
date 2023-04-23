// package kdf contains implementations of various key derivation functions
package kdf

import (
	"crypto/hmac"
	"fmt"
	"hash"
	"math"

	"github.com/dvsekhvalnov/jose2go/arrays"
)

// DerivePBKDF2 implements Password Based Key Derivation Function 2, RFC 2898. Derives key of keyBitLength size, given password, salt, iteration count and hash function
func DerivePBKDF2(password, salt []byte, iterationCount, keyBitLength int, h func() hash.Hash) []byte {

	prf := hmac.New(h, password)
	hLen := prf.Size()
	dkLen := keyBitLength >> 3 //size of derived key in bytes

	l := int(math.Ceil(float64(dkLen) / float64(hLen))) // l = CEIL (dkLen / hLen)
	r := dkLen - (l-1)*hLen

	// 1. If dkLen > (2^32 - 1) * hLen, output "derived key too long" and stop.
	if dkLen > MaxInt {
		panic(fmt.Sprintf("kdf.DerivePBKDF2: expects derived key size to be not more that (2^32-1) bits, but was requested %v bits.", keyBitLength))
	}

	dk := make([]byte, 0, dkLen)

	for i := 0; i < l; i++ {

		t := f(salt, iterationCount, i+1, prf) // T_l = F (P, S, c, l)

		if i == (l - 1) {
			t = t[:r]
		} // truncate last block to r bits

		dk = append(dk, t...) // DK = T_1 || T_2 ||  ...  || T_l<0..r-1>
	}

	return dk
}

func f(salt []byte, iterationCount, blockIndex int, prf hash.Hash) []byte {

	prf.Reset()
	prf.Write(salt)
	prf.Write(arrays.UInt32ToBytes(uint32(blockIndex)))

	u := prf.Sum(nil) // U_1 = PRF (P, S || INT (i))

	result := u

	for i := 2; i <= iterationCount; i++ {
		prf.Reset()
		prf.Write(u)

		u = prf.Sum(nil)               // U_c = PRF (P, U_{c-1}) .
		result = arrays.Xor(result, u) // U_1 \xor U_2 \xor ... \xor U_c
	}

	return result
}
