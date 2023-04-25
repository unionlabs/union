// uint256: Fixed size 256-bit math library
// Copyright 2018-2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

// Package math provides integer math utilities.

package uint256

import (
	"encoding/binary"
	"math"
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
