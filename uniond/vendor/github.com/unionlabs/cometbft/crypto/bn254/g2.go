package bn254

import (
	"errors"
	"math/big"
	"slices"

	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	bn254fp "github.com/consensys/gnark-crypto/ecc/bn254/fp"
	bn254fr "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
	"github.com/tunabay/go-bitarray"
)


// Union whitepaper: (1), (2), M ◦ H_{mimc^4}
//
// BN254G2_XMD:MIMC-256_SVDW
// HashToG2 hashes a message to a point on the G2 curve using the SVDW map.
// Slower than EncodeToG2, but usable as a random oracle.
// dst stands for "domain separation tag", a string unique to the construction using the hash function
// https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#roadmap
func HashToG2MiMC(msg, dst []byte) (curve.G2Affine, error) {
	u, err := HashToFieldMiMC(msg, dst)
	if err != nil {
		return curve.G2Affine{}, err
	}
	Q0 := curve.MapToCurve2(&curve.E2{
		A0: u[0],
		A1: u[1],
	})
	Q1 := curve.MapToCurve2(&curve.E2{
		A0: u[2],
		A1: u[3],
	})
	var _Q0, _Q1 curve.G2Jac
	_Q0.FromAffine(&Q0)
	_Q1.FromAffine(&Q1).AddAssign(&_Q0)
	_Q1.ClearCofactor(&_Q1)
	Q1.FromJacobian(&_Q1)
	if !Q1.IsOnCurve() || Q1.IsInfinity() || !Q1.IsInSubGroup() {
		panic("impossible")
	}
	return Q1, nil
}

// Union whitepaper: (1) H_{mimc^4}
//
// Hash msg to 4 prime field elements (actually bound to scalar elements).
// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-5.2
func HashToFieldMiMC(msg, dst []byte) ([4]bn254fp.Element, error) {
	// 128 bits of security
	// L = ceil((ceil(log2(p)) + k) / 8), where k is the security parameter = 128
	const Bytes = 1 + (254-1)/8
	const L = 16 + Bytes

	pseudoRandomBytes, err := ExpandMsgXmdMiMC(msg, dst, 4*L)
	if err != nil {
		return [4]bn254fp.Element{}, err
	}

	var res [4]bn254fp.Element
	for i := 0; i < 4; i++ {
		elemBytes := pseudoRandomBytes[i*L : (i+1)*L]
		slices.Reverse(elemBytes)
		res[i].SetBytes(elemBytes)
	}

	return res, nil
}

// BN254G2_XMD
// This is not a general implementation as the input/output length are fixed.
// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-5
// https://tools.ietf.org/html/rfc8017#section-4.1 (I2OSP/O2ISP)
// NOTE: block size is 256 bits, we could use 253 bits block to avoid the modulus, not sure if this would increase security
func ExpandMsgXmdMiMC(msg, dst []byte, lenInBytes int) ([]byte, error) {
	h := mimc.NewMiMC()

	ell := (lenInBytes + h.Size() - 1) / h.Size() // ceil(len_in_bytes / b_in_bytes)
	if ell > 255 {
		return nil, errors.New("invalid lenInBytes")
	}
	if len(dst) > 255 {
		return nil, errors.New("invalid domain size (>255 bytes)")
	}

	block := bitarray.New()
	writeU8 := func(b byte) {
		// NOTE: we reverse to simplify the in-circuit implementation
		// where everything is little-endian.
		block = block.Append(bitarray.NewBufferFromByteSlice([]byte{b}).BitArray().Reverse())
	}
	write := func(bs []byte) {
		for _, b := range bs {
			writeU8(b)
		}
	}
	sum := func() []byte {
		// zero pad block
		if block.Len()%256 != 0 {
			block = block.Append(bitarray.NewZeroFilled(256 - block.Len()%256))
		}
		data := []byte{}
		for i := 0; i < block.Len(); i += 256 {
			slice := block.Slice(i, i+256)
			bits := big.NewInt(0)
			c := big.NewInt(1)
			for j := 0; j < 256; j++ {
				bits = bits.Add(bits, new(big.Int).Mul(c, big.NewInt(int64(slice.BitAt(j)))))
				c.Lsh(c, 1)
			}
			bits.Mod(bits, bn254fr.Modulus())
			bitsB := make([]byte, 32)
			bits.FillBytes(bitsB)
			data = append(data, bitsB...)
		}
		_, err := h.Write(data)
		if err != nil {
			panic(err)
		}
		block = bitarray.New()
		s := h.Sum(nil)
		h.Reset()
		slices.Reverse(s)
		return s
	}

	// Z_pad = I2OSP(0, r_in_bytes)
	write_Z_pad := func() {
		write(make([]byte, h.BlockSize()))
	}

	write_msg := func() {
		write(msg)
	}

	// l_i_b_str = I2OSP(len_in_bytes, 2)
	write_l_i_b_str := func() {
		writeU8(uint8(lenInBytes >> 8))
		writeU8(uint8(lenInBytes))
	}

	// DST_prime =  DST ∥ I2OSP(len(DST), 1)
	write_DST_prime := func() {
		write(dst)
		writeU8(uint8(len(dst)))
	}

	// Z_pad = I2OSP(0, r_in_bytes)
	// l_i_b_str = I2OSP(len_in_bytes, 2)
	// DST_prime =  DST ∥ I2OSP(len(DST), 1)
	// b₀ = H(Z_pad ∥ msg ∥ l_i_b_str ∥ I2OSP(0, 1) ∥ DST_prime)
	write_Z_pad()
	write_msg()
	write_l_i_b_str()
	writeU8(byte(0))
	write_DST_prime()
	b0 := sum()

	// b₁ = H(b₀ ∥ I2OSP(1, 1) ∥ DST_prime)
	write(b0)
	writeU8(byte(1))
	write_DST_prime()
	b1 := sum()

	res := make([]byte, lenInBytes)
	copy(res[:h.Size()], b1)

	for i := 2; i <= ell; i++ {
		// b_i = H(strxor(b₀, b_(i - 1)) ∥ I2OSP(i, 1) ∥ DST_prime)
		strxor := make([]byte, h.Size())
		for j := 0; j < h.Size(); j++ {
			strxor[j] = b0[j] ^ b1[j]
		}

		write(strxor)
		writeU8(byte(i))
		write_DST_prime()
		b1 = sum()
		copy(res[h.Size()*(i-1):min(h.Size()*i, len(res))], b1)
	}
	return res, nil
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
