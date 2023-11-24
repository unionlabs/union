// uint256: Fixed size 256-bit math library
// Copyright 2018-2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

// Package math provides integer math utilities.

package uint256

import (
	"encoding/binary"
	"math"
	"math/big"
	"math/bits"
)

// Int is represented as an array of 4 uint64, in little-endian order,
// so that Int[3] is the most significant, and Int[0] is the least significant
type Int [4]uint64

// NewInt returns a new initialized Int.
func NewInt(val uint64) *Int {
	z := &Int{}
	z.SetUint64(val)
	return z
}

// SetBytes interprets buf as the bytes of a big-endian unsigned
// integer, sets z to that value, and returns z.
// If buf is larger than 32 bytes, the last 32 bytes is used. This operation
// is semantically equivalent to `FromBig(new(big.Int).SetBytes(buf))`
func (z *Int) SetBytes(buf []byte) *Int {
	switch l := len(buf); l {
	case 0:
		z.Clear()
	case 1:
		z.SetBytes1(buf)
	case 2:
		z.SetBytes2(buf)
	case 3:
		z.SetBytes3(buf)
	case 4:
		z.SetBytes4(buf)
	case 5:
		z.SetBytes5(buf)
	case 6:
		z.SetBytes6(buf)
	case 7:
		z.SetBytes7(buf)
	case 8:
		z.SetBytes8(buf)
	case 9:
		z.SetBytes9(buf)
	case 10:
		z.SetBytes10(buf)
	case 11:
		z.SetBytes11(buf)
	case 12:
		z.SetBytes12(buf)
	case 13:
		z.SetBytes13(buf)
	case 14:
		z.SetBytes14(buf)
	case 15:
		z.SetBytes15(buf)
	case 16:
		z.SetBytes16(buf)
	case 17:
		z.SetBytes17(buf)
	case 18:
		z.SetBytes18(buf)
	case 19:
		z.SetBytes19(buf)
	case 20:
		z.SetBytes20(buf)
	case 21:
		z.SetBytes21(buf)
	case 22:
		z.SetBytes22(buf)
	case 23:
		z.SetBytes23(buf)
	case 24:
		z.SetBytes24(buf)
	case 25:
		z.SetBytes25(buf)
	case 26:
		z.SetBytes26(buf)
	case 27:
		z.SetBytes27(buf)
	case 28:
		z.SetBytes28(buf)
	case 29:
		z.SetBytes29(buf)
	case 30:
		z.SetBytes30(buf)
	case 31:
		z.SetBytes31(buf)
	default:
		z.SetBytes32(buf[l-32:])
	}
	return z
}

// Bytes32 returns the value of z as a 32-byte big-endian array.
func (z *Int) Bytes32() [32]byte {
	// The PutUint64()s are inlined and we get 4x (load, bswap, store) instructions.
	var b [32]byte
	binary.BigEndian.PutUint64(b[0:8], z[3])
	binary.BigEndian.PutUint64(b[8:16], z[2])
	binary.BigEndian.PutUint64(b[16:24], z[1])
	binary.BigEndian.PutUint64(b[24:32], z[0])
	return b
}

// Bytes20 returns the value of z as a 20-byte big-endian array.
func (z *Int) Bytes20() [20]byte {
	var b [20]byte
	// The PutUint*()s are inlined and we get 3x (load, bswap, store) instructions.
	binary.BigEndian.PutUint32(b[0:4], uint32(z[2]))
	binary.BigEndian.PutUint64(b[4:12], z[1])
	binary.BigEndian.PutUint64(b[12:20], z[0])
	return b
}

// Bytes returns the value of z as a big-endian byte slice.
func (z *Int) Bytes() []byte {
	b := z.Bytes32()
	return b[32-z.ByteLen():]
}

// WriteToSlice writes the content of z into the given byteslice.
// If dest is larger than 32 bytes, z will fill the first parts, and leave
// the end untouched.
// OBS! If dest is smaller than 32 bytes, only the end parts of z will be used
// for filling the array, making it useful for filling an Address object
func (z *Int) WriteToSlice(dest []byte) {
	// ensure 32 bytes
	// A too large buffer. Fill last 32 bytes
	end := len(dest) - 1
	if end > 31 {
		end = 31
	}
	for i := 0; i <= end; i++ {
		dest[end-i] = byte(z[i/8] >> uint64(8*(i%8)))
	}
}

// WriteToArray32 writes all 32 bytes of z to the destination array, including zero-bytes
func (z *Int) WriteToArray32(dest *[32]byte) {
	for i := 0; i < 32; i++ {
		dest[31-i] = byte(z[i/8] >> uint64(8*(i%8)))
	}
}

// WriteToArray20 writes the last 20 bytes of z to the destination array, including zero-bytes
func (z *Int) WriteToArray20(dest *[20]byte) {
	for i := 0; i < 20; i++ {
		dest[19-i] = byte(z[i/8] >> uint64(8*(i%8)))
	}
}

// Uint64 returns the lower 64-bits of z
func (z *Int) Uint64() uint64 {
	return z[0]
}

// Uint64WithOverflow returns the lower 64-bits of z and bool whether overflow occurred
func (z *Int) Uint64WithOverflow() (uint64, bool) {
	return z[0], (z[1] | z[2] | z[3]) != 0
}

// Clone creates a new Int identical to z
func (z *Int) Clone() *Int {
	return &Int{z[0], z[1], z[2], z[3]}
}

// Add sets z to the sum x+y
func (z *Int) Add(x, y *Int) *Int {
	var carry uint64
	z[0], carry = bits.Add64(x[0], y[0], 0)
	z[1], carry = bits.Add64(x[1], y[1], carry)
	z[2], carry = bits.Add64(x[2], y[2], carry)
	z[3], _ = bits.Add64(x[3], y[3], carry)
	return z
}

// AddOverflow sets z to the sum x+y, and returns z and whether overflow occurred
func (z *Int) AddOverflow(x, y *Int) (*Int, bool) {
	var carry uint64
	z[0], carry = bits.Add64(x[0], y[0], 0)
	z[1], carry = bits.Add64(x[1], y[1], carry)
	z[2], carry = bits.Add64(x[2], y[2], carry)
	z[3], carry = bits.Add64(x[3], y[3], carry)
	return z, carry != 0
}

// AddMod sets z to the sum ( x+y ) mod m, and returns z.
// If m == 0, z is set to 0 (OBS: differs from the big.Int)
func (z *Int) AddMod(x, y, m *Int) *Int {

	// Fast path for m >= 2^192, with x and y at most slightly bigger than m.
	// This is always the case when x and y are already reduced modulo such m.

	if (m[3] != 0) && (x[3] <= m[3]) && (y[3] <= m[3]) {
		var (
			gteC1 uint64
			gteC2 uint64
			tmpX  Int
			tmpY  Int
			res   Int
		)

		// reduce x/y modulo m if they are gte m
		tmpX[0], gteC1 = bits.Sub64(x[0], m[0], gteC1)
		tmpX[1], gteC1 = bits.Sub64(x[1], m[1], gteC1)
		tmpX[2], gteC1 = bits.Sub64(x[2], m[2], gteC1)
		tmpX[3], gteC1 = bits.Sub64(x[3], m[3], gteC1)

		tmpY[0], gteC2 = bits.Sub64(y[0], m[0], gteC2)
		tmpY[1], gteC2 = bits.Sub64(y[1], m[1], gteC2)
		tmpY[2], gteC2 = bits.Sub64(y[2], m[2], gteC2)
		tmpY[3], gteC2 = bits.Sub64(y[3], m[3], gteC2)

		if gteC1 == 0 {
			x = &tmpX
		}
		if gteC2 == 0 {
			y = &tmpY
		}
		var (
			c1  uint64
			c2  uint64
			tmp Int
		)

		res[0], c1 = bits.Add64(x[0], y[0], c1)
		res[1], c1 = bits.Add64(x[1], y[1], c1)
		res[2], c1 = bits.Add64(x[2], y[2], c1)
		res[3], c1 = bits.Add64(x[3], y[3], c1)

		tmp[0], c2 = bits.Sub64(res[0], m[0], c2)
		tmp[1], c2 = bits.Sub64(res[1], m[1], c2)
		tmp[2], c2 = bits.Sub64(res[2], m[2], c2)
		tmp[3], c2 = bits.Sub64(res[3], m[3], c2)

		// final sub was unnecessary
		if c1 == 0 && c2 != 0 {
			copy((*z)[:], res[:])
			return z
		}

		copy((*z)[:], tmp[:])
		return z
	}

	if m.IsZero() {
		return z.Clear()
	}
	if z == m { // z is an alias for m and will be overwritten by AddOverflow before m is read
		m = m.Clone()
	}
	if _, overflow := z.AddOverflow(x, y); overflow {
		sum := [5]uint64{z[0], z[1], z[2], z[3], 1}
		var quot [5]uint64
		rem := udivrem(quot[:], sum[:], m)
		return z.Set(&rem)
	}
	return z.Mod(z, m)
}

// AddUint64 sets z to x + y, where y is a uint64, and returns z
func (z *Int) AddUint64(x *Int, y uint64) *Int {
	var carry uint64

	z[0], carry = bits.Add64(x[0], y, 0)
	z[1], carry = bits.Add64(x[1], 0, carry)
	z[2], carry = bits.Add64(x[2], 0, carry)
	z[3], _ = bits.Add64(x[3], 0, carry)
	return z
}

// PaddedBytes encodes a Int as a 0-padded byte slice. The length
// of the slice is at least n bytes.
// Example, z =1, n = 20 => [0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1]
func (z *Int) PaddedBytes(n int) []byte {
	b := make([]byte, n)

	for i := 0; i < 32 && i < n; i++ {
		b[n-1-i] = byte(z[i/8] >> uint64(8*(i%8)))
	}
	return b
}

// SubUint64 set z to the difference x - y, where y is a uint64, and returns z
func (z *Int) SubUint64(x *Int, y uint64) *Int {
	var carry uint64
	z[0], carry = bits.Sub64(x[0], y, carry)
	z[1], carry = bits.Sub64(x[1], 0, carry)
	z[2], carry = bits.Sub64(x[2], 0, carry)
	z[3], _ = bits.Sub64(x[3], 0, carry)
	return z
}

// SubOverflow sets z to the difference x-y and returns z and true if the operation underflowed
func (z *Int) SubOverflow(x, y *Int) (*Int, bool) {
	var carry uint64
	z[0], carry = bits.Sub64(x[0], y[0], 0)
	z[1], carry = bits.Sub64(x[1], y[1], carry)
	z[2], carry = bits.Sub64(x[2], y[2], carry)
	z[3], carry = bits.Sub64(x[3], y[3], carry)
	return z, carry != 0
}

// Sub sets z to the difference x-y
func (z *Int) Sub(x, y *Int) *Int {
	var carry uint64
	z[0], carry = bits.Sub64(x[0], y[0], 0)
	z[1], carry = bits.Sub64(x[1], y[1], carry)
	z[2], carry = bits.Sub64(x[2], y[2], carry)
	z[3], _ = bits.Sub64(x[3], y[3], carry)
	return z
}

// umulStep computes (hi * 2^64 + lo) = z + (x * y) + carry.
func umulStep(z, x, y, carry uint64) (hi, lo uint64) {
	hi, lo = bits.Mul64(x, y)
	lo, carry = bits.Add64(lo, carry, 0)
	hi, _ = bits.Add64(hi, 0, carry)
	lo, carry = bits.Add64(lo, z, 0)
	hi, _ = bits.Add64(hi, 0, carry)
	return hi, lo
}

// umulHop computes (hi * 2^64 + lo) = z + (x * y)
func umulHop(z, x, y uint64) (hi, lo uint64) {
	hi, lo = bits.Mul64(x, y)
	lo, carry := bits.Add64(lo, z, 0)
	hi, _ = bits.Add64(hi, 0, carry)
	return hi, lo
}

// umul computes full 256 x 256 -> 512 multiplication.
func umul(x, y *Int) [8]uint64 {
	var (
		res                           [8]uint64
		carry, carry4, carry5, carry6 uint64
		res1, res2, res3, res4, res5  uint64
	)

	carry, res[0] = bits.Mul64(x[0], y[0])
	carry, res1 = umulHop(carry, x[1], y[0])
	carry, res2 = umulHop(carry, x[2], y[0])
	carry4, res3 = umulHop(carry, x[3], y[0])

	carry, res[1] = umulHop(res1, x[0], y[1])
	carry, res2 = umulStep(res2, x[1], y[1], carry)
	carry, res3 = umulStep(res3, x[2], y[1], carry)
	carry5, res4 = umulStep(carry4, x[3], y[1], carry)

	carry, res[2] = umulHop(res2, x[0], y[2])
	carry, res3 = umulStep(res3, x[1], y[2], carry)
	carry, res4 = umulStep(res4, x[2], y[2], carry)
	carry6, res5 = umulStep(carry5, x[3], y[2], carry)

	carry, res[3] = umulHop(res3, x[0], y[3])
	carry, res[4] = umulStep(res4, x[1], y[3], carry)
	carry, res[5] = umulStep(res5, x[2], y[3], carry)
	res[7], res[6] = umulStep(carry6, x[3], y[3], carry)

	return res
}

// Mul sets z to the product x*y
func (z *Int) Mul(x, y *Int) *Int {
	var (
		res              Int
		carry            uint64
		res1, res2, res3 uint64
	)

	carry, res[0] = bits.Mul64(x[0], y[0])
	carry, res1 = umulHop(carry, x[1], y[0])
	carry, res2 = umulHop(carry, x[2], y[0])
	res3 = x[3]*y[0] + carry

	carry, res[1] = umulHop(res1, x[0], y[1])
	carry, res2 = umulStep(res2, x[1], y[1], carry)
	res3 = res3 + x[2]*y[1] + carry

	carry, res[2] = umulHop(res2, x[0], y[2])
	res3 = res3 + x[1]*y[2] + carry

	res[3] = res3 + x[0]*y[3]

	return z.Set(&res)
}

// MulOverflow sets z to the product x*y, and returns z and  whether overflow occurred
func (z *Int) MulOverflow(x, y *Int) (*Int, bool) {
	p := umul(x, y)
	copy(z[:], p[:4])
	return z, (p[4] | p[5] | p[6] | p[7]) != 0
}

func (z *Int) squared() {
	var (
		res                    Int
		carry0, carry1, carry2 uint64
		res1, res2             uint64
	)

	carry0, res[0] = bits.Mul64(z[0], z[0])
	carry0, res1 = umulHop(carry0, z[0], z[1])
	carry0, res2 = umulHop(carry0, z[0], z[2])

	carry1, res[1] = umulHop(res1, z[0], z[1])
	carry1, res2 = umulStep(res2, z[1], z[1], carry1)

	carry2, res[2] = umulHop(res2, z[0], z[2])

	res[3] = 2*(z[0]*z[3]+z[1]*z[2]) + carry0 + carry1 + carry2

	z.Set(&res)
}

// isBitSet returns true if bit n-th is set, where n = 0 is LSB.
// The n must be <= 255.
func (z *Int) isBitSet(n uint) bool {
	return (z[n/64] & (1 << (n % 64))) != 0
}

// addTo computes x += y.
// Requires len(x) >= len(y).
func addTo(x, y []uint64) uint64 {
	var carry uint64
	for i := 0; i < len(y); i++ {
		x[i], carry = bits.Add64(x[i], y[i], carry)
	}
	return carry
}

// subMulTo computes x -= y * multiplier.
// Requires len(x) >= len(y).
func subMulTo(x, y []uint64, multiplier uint64) uint64 {

	var borrow uint64
	for i := 0; i < len(y); i++ {
		s, carry1 := bits.Sub64(x[i], borrow, 0)
		ph, pl := bits.Mul64(y[i], multiplier)
		t, carry2 := bits.Sub64(s, pl, 0)
		x[i] = t
		borrow = ph + carry1 + carry2
	}
	return borrow
}

// udivremBy1 divides u by single normalized word d and produces both quotient and remainder.
// The quotient is stored in provided quot.
func udivremBy1(quot, u []uint64, d uint64) (rem uint64) {
	reciprocal := reciprocal2by1(d)
	rem = u[len(u)-1] // Set the top word as remainder.
	for j := len(u) - 2; j >= 0; j-- {
		quot[j], rem = udivrem2by1(rem, u[j], d, reciprocal)
	}
	return rem
}

// udivremKnuth implements the division of u by normalized multiple word d from the Knuth's division algorithm.
// The quotient is stored in provided quot - len(u)-len(d) words.
// Updates u to contain the remainder - len(d) words.
func udivremKnuth(quot, u, d []uint64) {
	dh := d[len(d)-1]
	dl := d[len(d)-2]
	reciprocal := reciprocal2by1(dh)

	for j := len(u) - len(d) - 1; j >= 0; j-- {
		u2 := u[j+len(d)]
		u1 := u[j+len(d)-1]
		u0 := u[j+len(d)-2]

		var qhat, rhat uint64
		if u2 >= dh { // Division overflows.
			qhat = ^uint64(0)
			// TODO: Add "qhat one to big" adjustment (not needed for correctness, but helps avoiding "add back" case).
		} else {
			qhat, rhat = udivrem2by1(u2, u1, dh, reciprocal)
			ph, pl := bits.Mul64(qhat, dl)
			if ph > rhat || (ph == rhat && pl > u0) {
				qhat--
				// TODO: Add "qhat one to big" adjustment (not needed for correctness, but helps avoiding "add back" case).
			}
		}

		// Multiply and subtract.
		borrow := subMulTo(u[j:], d, qhat)
		u[j+len(d)] = u2 - borrow
		if u2 < borrow { // Too much subtracted, add back.
			qhat--
			u[j+len(d)] += addTo(u[j:], d)
		}

		quot[j] = qhat // Store quotient digit.
	}
}

// udivrem divides u by d and produces both quotient and remainder.
// The quotient is stored in provided quot - len(u)-len(d)+1 words.
// It loosely follows the Knuth's division algorithm (sometimes referenced as "schoolbook" division) using 64-bit words.
// See Knuth, Volume 2, section 4.3.1, Algorithm D.
func udivrem(quot, u []uint64, d *Int) (rem Int) {
	var dLen int
	for i := len(d) - 1; i >= 0; i-- {
		if d[i] != 0 {
			dLen = i + 1
			break
		}
	}

	shift := uint(bits.LeadingZeros64(d[dLen-1]))

	var dnStorage Int
	dn := dnStorage[:dLen]
	for i := dLen - 1; i > 0; i-- {
		dn[i] = (d[i] << shift) | (d[i-1] >> (64 - shift))
	}
	dn[0] = d[0] << shift

	var uLen int
	for i := len(u) - 1; i >= 0; i-- {
		if u[i] != 0 {
			uLen = i + 1
			break
		}
	}

	if uLen < dLen {
		copy(rem[:], u)
		return rem
	}

	var unStorage [9]uint64
	un := unStorage[:uLen+1]
	un[uLen] = u[uLen-1] >> (64 - shift)
	for i := uLen - 1; i > 0; i-- {
		un[i] = (u[i] << shift) | (u[i-1] >> (64 - shift))
	}
	un[0] = u[0] << shift

	// TODO: Skip the highest word of numerator if not significant.

	if dLen == 1 {
		r := udivremBy1(quot, un, dn[0])
		rem.SetUint64(r >> shift)
		return rem
	}

	udivremKnuth(quot, un, dn)

	for i := 0; i < dLen-1; i++ {
		rem[i] = (un[i] >> shift) | (un[i+1] << (64 - shift))
	}
	rem[dLen-1] = un[dLen-1] >> shift

	return rem
}

// Div sets z to the quotient x/y for returns z.
// If y == 0, z is set to 0
func (z *Int) Div(x, y *Int) *Int {
	if y.IsZero() || y.Gt(x) {
		return z.Clear()
	}
	if x.Eq(y) {
		return z.SetOne()
	}
	// Shortcut some cases
	if x.IsUint64() {
		return z.SetUint64(x.Uint64() / y.Uint64())
	}

	// At this point, we know
	// x/y ; x > y > 0

	var quot Int
	udivrem(quot[:], x[:], y)
	return z.Set(&quot)
}

// Mod sets z to the modulus x%y for y != 0 and returns z.
// If y == 0, z is set to 0 (OBS: differs from the big.Int)
func (z *Int) Mod(x, y *Int) *Int {
	if x.IsZero() || y.IsZero() {
		return z.Clear()
	}
	switch x.Cmp(y) {
	case -1:
		// x < y
		copy(z[:], x[:])
		return z
	case 0:
		// x == y
		return z.Clear() // They are equal
	}

	// At this point:
	// x != 0
	// y != 0
	// x > y

	// Shortcut trivial case
	if x.IsUint64() {
		return z.SetUint64(x.Uint64() % y.Uint64())
	}

	var quot Int
	*z = udivrem(quot[:], x[:], y)
	return z
}

// DivMod sets z to the quotient x div y and m to the modulus x mod y and returns the pair (z, m) for y != 0.
// If y == 0, both z and m are set to 0 (OBS: differs from the big.Int)
func (z *Int) DivMod(x, y, m *Int) (*Int, *Int) {
	if y.IsZero() {
		return z.Clear(), m.Clear()
	}
	var quot Int
	*m = udivrem(quot[:], x[:], y)
	*z = quot
	return z, m
}

// SMod interprets x and y as two's complement signed integers,
// sets z to (sign x) * { abs(x) modulus abs(y) }
// If y == 0, z is set to 0 (OBS: differs from the big.Int)
func (z *Int) SMod(x, y *Int) *Int {
	ys := y.Sign()
	xs := x.Sign()

	// abs x
	if xs == -1 {
		x = new(Int).Neg(x)
	}
	// abs y
	if ys == -1 {
		y = new(Int).Neg(y)
	}
	z.Mod(x, y)
	if xs == -1 {
		z.Neg(z)
	}
	return z
}

// MulModWithReciprocal calculates the modulo-m multiplication of x and y
// and returns z, using the reciprocal of m provided as the mu parameter.
// Use uint256.Reciprocal to calculate mu from m.
// If m == 0, z is set to 0 (OBS: differs from the big.Int)
func (z *Int) MulModWithReciprocal(x, y, m *Int, mu *[5]uint64) *Int {
	if x.IsZero() || y.IsZero() || m.IsZero() {
		return z.Clear()
	}
	p := umul(x, y)

	if m[3] != 0 {
		r := reduce4(p, m, *mu)
		return z.Set(&r)
	}

	var (
		pl Int
		ph Int
	)
	copy(pl[:], p[:4])
	copy(ph[:], p[4:])

	// If the multiplication is within 256 bits use Mod().
	if ph.IsZero() {
		return z.Mod(&pl, m)
	}

	var quot [8]uint64
	rem := udivrem(quot[:], p[:], m)
	return z.Set(&rem)
}

// MulMod calculates the modulo-m multiplication of x and y and
// returns z.
// If m == 0, z is set to 0 (OBS: differs from the big.Int)
func (z *Int) MulMod(x, y, m *Int) *Int {
	if x.IsZero() || y.IsZero() || m.IsZero() {
		return z.Clear()
	}
	p := umul(x, y)

	if m[3] != 0 {
		mu := Reciprocal(m)
		r := reduce4(p, m, mu)
		return z.Set(&r)
	}

	var (
		pl Int
		ph Int
	)
	copy(pl[:], p[:4])
	copy(ph[:], p[4:])

	// If the multiplication is within 256 bits use Mod().
	if ph.IsZero() {
		return z.Mod(&pl, m)
	}

	var quot [8]uint64
	rem := udivrem(quot[:], p[:], m)
	return z.Set(&rem)
}

// MulDivOverflow calculates (x*y)/d with full precision, returns z and whether overflow occurred in multiply process (result does not fit to 256-bit).
// computes 512-bit multiplication and 512 by 256 division.
func (z *Int) MulDivOverflow(x, y, d *Int) (*Int, bool) {
	if x.IsZero() || y.IsZero() || d.IsZero() {
		return z.Clear(), false
	}
	p := umul(x, y)

	var quot [8]uint64
	udivrem(quot[:], p[:], d)

	copy(z[:], quot[:4])

	return z, (quot[4] | quot[5] | quot[6] | quot[7]) != 0
}

// Abs interprets x as a two's complement signed number,
// and sets z to the absolute value
//
//	Abs(0)        = 0
//	Abs(1)        = 1
//	Abs(2**255)   = -2**255
//	Abs(2**256-1) = -1
func (z *Int) Abs(x *Int) *Int {
	if x[3] < 0x8000000000000000 {
		return z.Set(x)
	}
	return z.Sub(new(Int), x)
}

// Neg returns -x mod 2**256.
func (z *Int) Neg(x *Int) *Int {
	return z.Sub(new(Int), x)
}

// SDiv interprets n and d as two's complement signed integers,
// does a signed division on the two operands and sets z to the result.
// If d == 0, z is set to 0
func (z *Int) SDiv(n, d *Int) *Int {
	if n.Sign() > 0 {
		if d.Sign() > 0 {
			// pos / pos
			z.Div(n, d)
			return z
		} else {
			// pos / neg
			z.Div(n, new(Int).Neg(d))
			return z.Neg(z)
		}
	}

	if d.Sign() < 0 {
		// neg / neg
		z.Div(new(Int).Neg(n), new(Int).Neg(d))
		return z
	}
	// neg / pos
	z.Div(new(Int).Neg(n), d)
	return z.Neg(z)
}

// Sign returns:
//
//	-1 if z <  0
//	 0 if z == 0
//	+1 if z >  0
//
// Where z is interpreted as a two's complement signed number
func (z *Int) Sign() int {
	if z.IsZero() {
		return 0
	}
	if z[3] < 0x8000000000000000 {
		return 1
	}
	return -1
}

// BitLen returns the number of bits required to represent z
func (z *Int) BitLen() int {
	switch {
	case z[3] != 0:
		return 192 + bits.Len64(z[3])
	case z[2] != 0:
		return 128 + bits.Len64(z[2])
	case z[1] != 0:
		return 64 + bits.Len64(z[1])
	default:
		return bits.Len64(z[0])
	}
}

// ByteLen returns the number of bytes required to represent z
func (z *Int) ByteLen() int {
	return (z.BitLen() + 7) / 8
}

func (z *Int) lsh64(x *Int) *Int {
	z[3], z[2], z[1], z[0] = x[2], x[1], x[0], 0
	return z
}
func (z *Int) lsh128(x *Int) *Int {
	z[3], z[2], z[1], z[0] = x[1], x[0], 0, 0
	return z
}
func (z *Int) lsh192(x *Int) *Int {
	z[3], z[2], z[1], z[0] = x[0], 0, 0, 0
	return z
}
func (z *Int) rsh64(x *Int) *Int {
	z[3], z[2], z[1], z[0] = 0, x[3], x[2], x[1]
	return z
}
func (z *Int) rsh128(x *Int) *Int {
	z[3], z[2], z[1], z[0] = 0, 0, x[3], x[2]
	return z
}
func (z *Int) rsh192(x *Int) *Int {
	z[3], z[2], z[1], z[0] = 0, 0, 0, x[3]
	return z
}
func (z *Int) srsh64(x *Int) *Int {
	z[3], z[2], z[1], z[0] = math.MaxUint64, x[3], x[2], x[1]
	return z
}
func (z *Int) srsh128(x *Int) *Int {
	z[3], z[2], z[1], z[0] = math.MaxUint64, math.MaxUint64, x[3], x[2]
	return z
}
func (z *Int) srsh192(x *Int) *Int {
	z[3], z[2], z[1], z[0] = math.MaxUint64, math.MaxUint64, math.MaxUint64, x[3]
	return z
}

// Not sets z = ^x and returns z.
func (z *Int) Not(x *Int) *Int {
	z[3], z[2], z[1], z[0] = ^x[3], ^x[2], ^x[1], ^x[0]
	return z
}

// Gt returns true if z > x
func (z *Int) Gt(x *Int) bool {
	return x.Lt(z)
}

// Slt interprets z and x as signed integers, and returns
// true if z < x
func (z *Int) Slt(x *Int) bool {

	zSign := z.Sign()
	xSign := x.Sign()

	switch {
	case zSign >= 0 && xSign < 0:
		return false
	case zSign < 0 && xSign >= 0:
		return true
	default:
		return z.Lt(x)
	}
}

// Sgt interprets z and x as signed integers, and returns
// true if z > x
func (z *Int) Sgt(x *Int) bool {
	zSign := z.Sign()
	xSign := x.Sign()

	switch {
	case zSign >= 0 && xSign < 0:
		return true
	case zSign < 0 && xSign >= 0:
		return false
	default:
		return z.Gt(x)
	}
}

// Lt returns true if z < x
func (z *Int) Lt(x *Int) bool {
	// z < x <=> z - x < 0 i.e. when subtraction overflows.
	_, carry := bits.Sub64(z[0], x[0], 0)
	_, carry = bits.Sub64(z[1], x[1], carry)
	_, carry = bits.Sub64(z[2], x[2], carry)
	_, carry = bits.Sub64(z[3], x[3], carry)
	return carry != 0
}

// SetUint64 sets z to the value x
func (z *Int) SetUint64(x uint64) *Int {
	z[3], z[2], z[1], z[0] = 0, 0, 0, x
	return z
}

// Eq returns true if z == x
func (z *Int) Eq(x *Int) bool {
	return (z[0] == x[0]) && (z[1] == x[1]) && (z[2] == x[2]) && (z[3] == x[3])
}

// Cmp compares z and x and returns:
//
//	-1 if z <  x
//	 0 if z == x
//	+1 if z >  x
func (z *Int) Cmp(x *Int) (r int) {
	// z < x <=> z - x < 0 i.e. when subtraction overflows.
	d0, carry := bits.Sub64(z[0], x[0], 0)
	d1, carry := bits.Sub64(z[1], x[1], carry)
	d2, carry := bits.Sub64(z[2], x[2], carry)
	d3, carry := bits.Sub64(z[3], x[3], carry)
	if carry == 1 {
		return -1
	}
	if d0|d1|d2|d3 == 0 {
		return 0
	}
	return 1
}

// CmpUint64 compares z and x and returns:
//
//	-1 if z <  x
//	 0 if z == x
//	+1 if z >  x
func (z *Int) CmpUint64(x uint64) int {
	if z[0] > x || (z[1]|z[2]|z[3]) != 0 {
		return 1
	}
	if z[0] == x {
		return 0
	}
	return -1
}

// CmpBig compares z and x and returns:
//
//	-1 if z <  x
//	 0 if z == x
//	+1 if z >  x
func (z *Int) CmpBig(x *big.Int) (r int) {
	// If x is negative, it's surely smaller (z > x)
	if x.Sign() == -1 {
		return 1
	}
	y := new(Int)
	if y.SetFromBig(x) { // overflow
		// z < x
		return -1
	}
	return z.Cmp(y)
}

// LtUint64 returns true if z is smaller than n
func (z *Int) LtUint64(n uint64) bool {
	return z[0] < n && (z[1]|z[2]|z[3]) == 0
}

// GtUint64 returns true if z is larger than n
func (z *Int) GtUint64(n uint64) bool {
	return z[0] > n || (z[1]|z[2]|z[3]) != 0
}

// IsUint64 reports whether z can be represented as a uint64.
func (z *Int) IsUint64() bool {
	return (z[1] | z[2] | z[3]) == 0
}

// IsZero returns true if z == 0
func (z *Int) IsZero() bool {
	return (z[0] | z[1] | z[2] | z[3]) == 0
}

// Clear sets z to 0
func (z *Int) Clear() *Int {
	z[3], z[2], z[1], z[0] = 0, 0, 0, 0
	return z
}

// SetAllOne sets all the bits of z to 1
func (z *Int) SetAllOne() *Int {
	z[3], z[2], z[1], z[0] = math.MaxUint64, math.MaxUint64, math.MaxUint64, math.MaxUint64
	return z
}

// SetOne sets z to 1
func (z *Int) SetOne() *Int {
	z[3], z[2], z[1], z[0] = 0, 0, 0, 1
	return z
}

// Lsh sets z = x << n and returns z.
func (z *Int) Lsh(x *Int, n uint) *Int {
	// n % 64 == 0
	if n&0x3f == 0 {
		switch n {
		case 0:
			return z.Set(x)
		case 64:
			return z.lsh64(x)
		case 128:
			return z.lsh128(x)
		case 192:
			return z.lsh192(x)
		default:
			return z.Clear()
		}
	}
	var (
		a, b uint64
	)
	// Big swaps first
	switch {
	case n > 192:
		if n > 256 {
			return z.Clear()
		}
		z.lsh192(x)
		n -= 192
		goto sh192
	case n > 128:
		z.lsh128(x)
		n -= 128
		goto sh128
	case n > 64:
		z.lsh64(x)
		n -= 64
		goto sh64
	default:
		z.Set(x)
	}

	// remaining shifts
	a = z[0] >> (64 - n)
	z[0] = z[0] << n

sh64:
	b = z[1] >> (64 - n)
	z[1] = (z[1] << n) | a

sh128:
	a = z[2] >> (64 - n)
	z[2] = (z[2] << n) | b

sh192:
	z[3] = (z[3] << n) | a

	return z
}

// Rsh sets z = x >> n and returns z.
func (z *Int) Rsh(x *Int, n uint) *Int {
	// n % 64 == 0
	if n&0x3f == 0 {
		switch n {
		case 0:
			return z.Set(x)
		case 64:
			return z.rsh64(x)
		case 128:
			return z.rsh128(x)
		case 192:
			return z.rsh192(x)
		default:
			return z.Clear()
		}
	}
	var (
		a, b uint64
	)
	// Big swaps first
	switch {
	case n > 192:
		if n > 256 {
			return z.Clear()
		}
		z.rsh192(x)
		n -= 192
		goto sh192
	case n > 128:
		z.rsh128(x)
		n -= 128
		goto sh128
	case n > 64:
		z.rsh64(x)
		n -= 64
		goto sh64
	default:
		z.Set(x)
	}

	// remaining shifts
	a = z[3] << (64 - n)
	z[3] = z[3] >> n

sh64:
	b = z[2] << (64 - n)
	z[2] = (z[2] >> n) | a

sh128:
	a = z[1] << (64 - n)
	z[1] = (z[1] >> n) | b

sh192:
	z[0] = (z[0] >> n) | a

	return z
}

// SRsh (Signed/Arithmetic right shift)
// considers z to be a signed integer, during right-shift
// and sets z = x >> n and returns z.
func (z *Int) SRsh(x *Int, n uint) *Int {
	// If the MSB is 0, SRsh is same as Rsh.
	if !x.isBitSet(255) {
		return z.Rsh(x, n)
	}
	if n%64 == 0 {
		switch n {
		case 0:
			return z.Set(x)
		case 64:
			return z.srsh64(x)
		case 128:
			return z.srsh128(x)
		case 192:
			return z.srsh192(x)
		default:
			return z.SetAllOne()
		}
	}
	var (
		a uint64 = math.MaxUint64 << (64 - n%64)
	)
	// Big swaps first
	switch {
	case n > 192:
		if n > 256 {
			return z.SetAllOne()
		}
		z.srsh192(x)
		n -= 192
		goto sh192
	case n > 128:
		z.srsh128(x)
		n -= 128
		goto sh128
	case n > 64:
		z.srsh64(x)
		n -= 64
		goto sh64
	default:
		z.Set(x)
	}

	// remaining shifts
	z[3], a = (z[3]>>n)|a, z[3]<<(64-n)

sh64:
	z[2], a = (z[2]>>n)|a, z[2]<<(64-n)

sh128:
	z[1], a = (z[1]>>n)|a, z[1]<<(64-n)

sh192:
	z[0] = (z[0] >> n) | a

	return z
}

// Set sets z to x and returns z.
func (z *Int) Set(x *Int) *Int {
	*z = *x
	return z
}

// Or sets z = x | y and returns z.
func (z *Int) Or(x, y *Int) *Int {
	z[0] = x[0] | y[0]
	z[1] = x[1] | y[1]
	z[2] = x[2] | y[2]
	z[3] = x[3] | y[3]
	return z
}

// And sets z = x & y and returns z.
func (z *Int) And(x, y *Int) *Int {
	z[0] = x[0] & y[0]
	z[1] = x[1] & y[1]
	z[2] = x[2] & y[2]
	z[3] = x[3] & y[3]
	return z
}

// Xor sets z = x ^ y and returns z.
func (z *Int) Xor(x, y *Int) *Int {
	z[0] = x[0] ^ y[0]
	z[1] = x[1] ^ y[1]
	z[2] = x[2] ^ y[2]
	z[3] = x[3] ^ y[3]
	return z
}

// Byte sets z to the value of the byte at position n,
// with 'z' considered as a big-endian 32-byte integer
// if 'n' > 32, f is set to 0
// Example: f = '5', n=31 => 5
func (z *Int) Byte(n *Int) *Int {
	// in z, z[0] is the least significant
	//
	if number, overflow := n.Uint64WithOverflow(); !overflow {
		if number < 32 {
			number := z[4-1-number/8]
			offset := (n[0] & 0x7) << 3 // 8*(n.d % 8)
			z[0] = (number & (0xff00000000000000 >> offset)) >> (56 - offset)
			z[3], z[2], z[1] = 0, 0, 0
			return z
		}
	}
	return z.Clear()
}

// Exp sets z = base**exponent mod 2**256, and returns z.
func (z *Int) Exp(base, exponent *Int) *Int {
	res := Int{1, 0, 0, 0}
	multiplier := *base
	expBitLen := exponent.BitLen()

	curBit := 0
	word := exponent[0]
	for ; curBit < expBitLen && curBit < 64; curBit++ {
		if word&1 == 1 {
			res.Mul(&res, &multiplier)
		}
		multiplier.squared()
		word >>= 1
	}

	word = exponent[1]
	for ; curBit < expBitLen && curBit < 128; curBit++ {
		if word&1 == 1 {
			res.Mul(&res, &multiplier)
		}
		multiplier.squared()
		word >>= 1
	}

	word = exponent[2]
	for ; curBit < expBitLen && curBit < 192; curBit++ {
		if word&1 == 1 {
			res.Mul(&res, &multiplier)
		}
		multiplier.squared()
		word >>= 1
	}

	word = exponent[3]
	for ; curBit < expBitLen && curBit < 256; curBit++ {
		if word&1 == 1 {
			res.Mul(&res, &multiplier)
		}
		multiplier.squared()
		word >>= 1
	}
	return z.Set(&res)
}

// ExtendSign extends length of two’s complement signed integer,
// sets z to
//   - x if byteNum > 31
//   - x interpreted as a signed number with sign-bit at (byteNum*8+7), extended to the full 256 bits
//
// and returns z.
func (z *Int) ExtendSign(x, byteNum *Int) *Int {
	if byteNum.GtUint64(31) {
		return z.Set(x)
	}
	bit := uint(byteNum.Uint64()*8 + 7)

	mask := new(Int).SetOne()
	mask.Lsh(mask, bit)
	mask.SubUint64(mask, 1)
	if x.isBitSet(bit) {
		z.Or(x, mask.Not(mask))
	} else {
		z.And(x, mask)
	}
	return z
}

// Sqrt sets z to ⌊√x⌋, the largest integer such that z² ≤ x, and returns z.
func (z *Int) Sqrt(x *Int) *Int {
	// This implementation of Sqrt is based on big.Int (see math/big/nat.go).
	if x.LtUint64(2) {
		return z.Set(x)
	}
	var (
		z1 = &Int{1, 0, 0, 0}
		z2 = &Int{}
	)
	// Start with value known to be too large and repeat "z = ⌊(z + ⌊x/z⌋)/2⌋" until it stops getting smaller.
	z1 = z1.Lsh(z1, uint(x.BitLen()+1)/2) // must be ≥ √x
	for {
		z2 = z2.Div(x, z1)
		z2 = z2.Add(z2, z1)
		{ //z2 = z2.Rsh(z2, 1) -- the code below does a 1-bit rsh faster
			a := z2[3] << 63
			z2[3] = z2[3] >> 1
			b := z2[2] << 63
			z2[2] = (z2[2] >> 1) | a
			a = z2[1] << 63
			z2[1] = (z2[1] >> 1) | b
			z2[0] = (z2[0] >> 1) | a
		}
		// end of inlined bitshift

		if z2.Cmp(z1) >= 0 {
			// z1 is answer.
			return z.Set(z1)
		}
		z1, z2 = z2, z1
	}
}

var (
	// lut is a lookuptable of bitlength -> log10, used in Log10().
	lut = [257]int8{0, 0, 0, 0, -1, 1, 1, -2, 2, 2, -3, 3, 3, 3, -4, 4, 4, -5, 5, 5, -6, 6, 6, 6, -7, 7, 7, -8, 8, 8, -9, 9, 9, 9, -10, 10, 10, -11, 11, 11, -12, 12, 12, 12, -13, 13, 13, -14, 14, 14, -15, 15, 15, 15, -16, 16, 16, -17, 17, 17, -18, 18, 18, 18, -19, 19, 19, -20, 20, 20, -21, 21, 21, 21, -22, 22, 22, -23, 23, 23, -24, 24, 24, 24, -25, 25, 25, -26, 26, 26, -27, 27, 27, 27, -28, 28, 28, -29, 29, 29, -30, 30, 30, -31, 31, 31, 31, -32, 32, 32, -33, 33, 33, -34, 34, 34, 34, -35, 35, 35, -36, 36, 36, -37, 37, 37, 37, -38, 38, 38, -39, 39, 39, -40, 40, 40, 40, -41, 41, 41, -42, 42, 42, -43, 43, 43, 43, -44, 44, 44, -45, 45, 45, -46, 46, 46, 46, -47, 47, 47, -48, 48, 48, -49, 49, 49, 49, -50, 50, 50, -51, 51, 51, -52, 52, 52, 52, -53, 53, 53, -54, 54, 54, -55, 55, 55, 55, -56, 56, 56, -57, 57, 57, -58, 58, 58, -59, 59, 59, 59, -60, 60, 60, -61, 61, 61, -62, 62, 62, 62, -63, 63, 63, -64, 64, 64, -65, 65, 65, 65, -66, 66, 66, -67, 67, 67, -68, 68, 68, 68, -69, 69, 69, -70, 70, 70, -71, 71, 71, 71, -72, 72, 72, -73, 73, 73, -74, 74, 74, 74, -75, 75, 75, -76, 76, 76, -77}

	// pows64 contains 10^0 ... 10^19
	pows64 = [20]uint64{
		1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16, 1e17, 1e18, 1e19,
	}
	// pows contain 1 ** 20 ... 10 ** 80
	pows = [60]Int{
		Int{7766279631452241920, 5, 0, 0}, Int{3875820019684212736, 54, 0, 0}, Int{1864712049423024128, 542, 0, 0}, Int{200376420520689664, 5421, 0, 0}, Int{2003764205206896640, 54210, 0, 0}, Int{1590897978359414784, 542101, 0, 0}, Int{15908979783594147840, 5421010, 0, 0}, Int{11515845246265065472, 54210108, 0, 0}, Int{4477988020393345024, 542101086, 0, 0}, Int{7886392056514347008, 5421010862, 0, 0}, Int{5076944270305263616, 54210108624, 0, 0}, Int{13875954555633532928, 542101086242, 0, 0}, Int{9632337040368467968, 5421010862427, 0, 0},
		Int{4089650035136921600, 54210108624275, 0, 0}, Int{4003012203950112768, 542101086242752, 0, 0}, Int{3136633892082024448, 5421010862427522, 0, 0}, Int{12919594847110692864, 54210108624275221, 0, 0}, Int{68739955140067328, 542101086242752217, 0, 0}, Int{687399551400673280, 5421010862427522170, 0, 0}, Int{6873995514006732800, 17316620476856118468, 2, 0}, Int{13399722918938673152, 7145508105175220139, 29, 0}, Int{4870020673419870208, 16114848830623546549, 293, 0}, Int{11806718586779598848, 13574535716559052564, 2938, 0},
		Int{7386721425538678784, 6618148649623664334, 29387, 0}, Int{80237960548581376, 10841254275107988496, 293873, 0}, Int{802379605485813760, 16178822382532126880, 2938735, 0}, Int{8023796054858137600, 14214271235644855872, 29387358, 0}, Int{6450984253743169536, 13015503840481697412, 293873587, 0}, Int{9169610316303040512, 1027829888850112811, 2938735877, 0}, Int{17909126868192198656, 10278298888501128114, 29387358770, 0}, Int{13070572018536022016, 10549268516463523069, 293873587705, 0}, Int{1578511669393358848, 13258964796087472617, 2938735877055, 0}, Int{15785116693933588480, 3462439444907864858, 29387358770557, 0},
		Int{10277214349659471872, 16177650375369096972, 293873587705571, 0}, Int{10538423128046960640, 14202551164014556797, 2938735877055718, 0}, Int{13150510911921848320, 12898303124178706663, 29387358770557187, 0}, Int{2377900603251621888, 18302566799529756941, 293873587705571876, 0}, Int{5332261958806667264, 17004971331911604867, 2938735877055718769, 0}, Int{16429131440647569408, 4029016655730084128, 10940614696847636083, 1}, Int{16717361816799281152, 3396678409881738056, 17172426599928602752, 15}, Int{1152921504606846976, 15520040025107828953, 5703569335900062977, 159}, Int{11529215046068469760, 7626447661401876602, 1695461137871974930, 1593}, Int{4611686018427387904, 2477500319180559562, 16954611378719749304, 15930}, Int{9223372036854775808, 6328259118096044006, 3525417123811528497, 159309},
		Int{0, 7942358959831785217, 16807427164405733357, 1593091}, Int{0, 5636613303479645706, 2053574980671369030, 15930919}, Int{0, 1025900813667802212, 2089005733004138687, 159309191}, Int{0, 10259008136678022120, 2443313256331835254, 1593091911}, Int{0, 10356360998232463120, 5986388489608800929, 15930919111}, Int{0, 11329889613776873120, 4523652674959354447, 159309191113}, Int{0, 2618431695511421504, 8343038602174441244, 1593091911132}, Int{0, 7737572881404663424, 9643409726906205977, 15930919111324}, Int{0, 3588752519208427776, 4200376900514301694, 159309191113245}, Int{0, 17440781118374726144, 5110280857723913709, 1593091911132452}, Int{0, 8387114520361296896, 14209320429820033867, 15930919111324522}, Int{0, 10084168908774762496, 12965995782233477362, 159309191113245227}, Int{0, 8607968719199866880, 532749306367912313, 1593091911132452277}, Int{0, 12292710897160462336, 5327493063679123134, 15930919111324522770}, Int{0, 12246644529347313664, 16381442489372128114, 11735238523568814774}, Int{0, 11785980851215826944, 16240472304044868218, 6671920793430838052},
	}
)

// Log10 returns the log in base 10, floored to nearest integer.
// **OBS** This method returns '0' for '0', not `-Inf`.
func (z *Int) Log10() uint {
	// For some bit-lengths, there's only one possible value. Example:
	// three bits can only represent [100 ... 111], or [4 ... 7]
	// Ergo, bitlen:3 -> log10 == 0
	res := lut[z.BitLen()%257]
	if res >= 0 {
		return uint(res)
	}
	// It was negative, which is a signal that we need to do one more check
	// do determine which log it is. First remove the negation
	res = -res

	// We now lookup via the power of tens. Example:
	// bitlen 4, [1000 ... 1111], or [8 .. 15]
	// In order to figure out if it is '0' or '1', we only need to do one comparison,
	// is it larger or smaller than '10'?

	// For bitlengths < 20, we can use the uint64-space
	if res < 20 {
		// Uint64-space
		if z.CmpUint64(pows64[res]) < 0 {
			return uint(res - 1)
		}
		return uint(res)
	}
	// Non-uint64 space
	if z.Cmp(&pows[res-20]) < 0 {
		return uint(res - 1)
	}
	return uint(res)
}
