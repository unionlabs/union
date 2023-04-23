// Package bigint provides common functions for manipulating multi-precision integers.
package bigint

import (
	"math/big"
	"math/rand"
	"strings"
)

// Zero returns 0.
func Zero() *big.Int {
	return big.NewInt(0)
}

// One returns 1.
func One() *big.Int {
	return big.NewInt(1)
}

// Hex constructs an integer from a hex string, returning the integer and a
// boolean indicating success. Underscore may be used as a separator.
func Hex(s string) (*big.Int, bool) {
	return new(big.Int).SetString(stripliteral(s), 16)
}

// MustHex constructs an integer from a hex string. It panics on error.
func MustHex(s string) *big.Int {
	x, ok := Hex(s)
	if !ok {
		panic("failed to parse hex integer")
	}
	return x
}

// Binary parses a binary string into an integer, returning the integer and a
// boolean indicating success. Underscore may be used as a separator.
func Binary(s string) (*big.Int, bool) {
	return new(big.Int).SetString(stripliteral(s), 2)
}

// MustBinary constructs an integer from a binary string. It panics on error.
func MustBinary(s string) *big.Int {
	x, ok := Binary(s)
	if !ok {
		panic("failed to parse binary integer")
	}
	return x
}

// stripliteral removes underscore spacers from a numeric literal.
func stripliteral(s string) string {
	return strings.ReplaceAll(s, "_", "")
}

// Equal returns whether x equals y.
func Equal(x, y *big.Int) bool {
	return x.Cmp(y) == 0
}

// EqualInt64 is a convenience for checking if x equals the int64 value y.
func EqualInt64(x *big.Int, y int64) bool {
	return Equal(x, big.NewInt(y))
}

// IsZero returns true if x is zero.
func IsZero(x *big.Int) bool {
	return x.Sign() == 0
}

// IsNonZero returns true if x is non-zero.
func IsNonZero(x *big.Int) bool {
	return !IsZero(x)
}

// Clone returns a copy of x.
func Clone(x *big.Int) *big.Int {
	return new(big.Int).Set(x)
}

// Pow2 returns 2ᵉ.
func Pow2(e uint) *big.Int {
	return new(big.Int).Lsh(One(), e)
}

// IsPow2 returns whether x is a power of 2.
func IsPow2(x *big.Int) bool {
	e := x.BitLen()
	if e == 0 {
		return false
	}
	return Equal(x, Pow2(uint(e-1)))
}

// Pow2UpTo returns all powers of two ⩽ x.
func Pow2UpTo(x *big.Int) []*big.Int {
	p := One()
	ps := []*big.Int{}
	for p.Cmp(x) <= 0 {
		ps = append(ps, Clone(p))
		p.Lsh(p, 1)
	}
	return ps
}

// Mask returns the integer with 1s in positions [l,h).
func Mask(l, h uint) *big.Int {
	mask := Pow2(h)
	return mask.Sub(mask, Pow2(l))
}

// Ones returns 2ⁿ - 1, the integer with n 1s in the low bits.
func Ones(n uint) *big.Int {
	return Mask(0, n)
}

// BitsSet returns the positions of set bits in x.
func BitsSet(x *big.Int) []int {
	set := []int{}
	for i := 0; i < x.BitLen(); i++ {
		if x.Bit(i) == 1 {
			set = append(set, i)
		}
	}
	return set
}

// MinMax returns the minimum and maximum of x and y.
func MinMax(x, y *big.Int) (min, max *big.Int) {
	if x.Cmp(y) < 0 {
		return x, y
	}
	return y, x
}

// Extract bits [l,h) and shift them to the low bits.
func Extract(x *big.Int, l, h uint) *big.Int {
	e := Mask(l, h)
	e.And(e, x)
	return e.Rsh(e, l)
}

// RandBits returns a random integer less than 2ⁿ.
func RandBits(r *rand.Rand, n uint) *big.Int {
	max := Pow2(n)
	return new(big.Int).Rand(r, max)
}

// Uint64s represents x in 64-bit limbs.
func Uint64s(x *big.Int) []uint64 {
	z := Clone(x)
	mask := Ones(64)
	word := new(big.Int)
	words := []uint64{}
	for IsNonZero(z) {
		word.And(z, mask)
		words = append(words, word.Uint64())
		z.Rsh(z, 64)
	}
	return words
}

// BytesLittleEndian returns the absolute value of x as a little-endian byte slice.
func BytesLittleEndian(x *big.Int) []byte {
	b := x.Bytes()
	for l, r := 0, len(b)-1; l < r; l, r = l+1, r-1 {
		b[l], b[r] = b[r], b[l]
	}
	return b
}
