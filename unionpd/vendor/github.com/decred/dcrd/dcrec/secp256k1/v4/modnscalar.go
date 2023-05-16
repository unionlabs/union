// Copyright (c) 2020-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"encoding/hex"
	"math/big"
)

// References:
//   [SECG]: Recommended Elliptic Curve Domain Parameters
//     https://www.secg.org/sec2-v2.pdf
//
//   [HAC]: Handbook of Applied Cryptography Menezes, van Oorschot, Vanstone.
//     http://cacr.uwaterloo.ca/hac/

// Many elliptic curve operations require working with scalars in a finite field
// characterized by the order of the group underlying the secp256k1 curve.
// Given this precision is larger than the biggest available native type,
// obviously some form of bignum math is needed.  This code implements
// specialized fixed-precision field arithmetic rather than relying on an
// arbitrary-precision arithmetic package such as math/big for dealing with the
// math modulo the group order since the size is known.  As a result, rather
// large performance gains are achieved by taking advantage of many
// optimizations not available to arbitrary-precision arithmetic and generic
// modular arithmetic algorithms.
//
// There are various ways to internally represent each element.  For example,
// the most obvious representation would be to use an array of 4 uint64s (64
// bits * 4 = 256 bits).  However, that representation suffers from the fact
// that there is no native Go type large enough to handle the intermediate
// results while adding or multiplying two 64-bit numbers.
//
// Given the above, this implementation represents the field elements as 8
// uint32s with each word (array entry) treated as base 2^32.  This was chosen
// because most systems at the current time are 64-bit (or at least have 64-bit
// registers available for specialized purposes such as MMX) so the intermediate
// results can typically be done using a native register (and using uint64s to
// avoid the need for additional half-word arithmetic)

const (
	// These fields provide convenient access to each of the words of the
	// secp256k1 curve group order N to improve code readability.
	//
	// The group order of the curve per [SECG] is:
	// 0xffffffff ffffffff ffffffff fffffffe baaedce6 af48a03b bfd25e8c d0364141
	orderWordZero  uint32 = 0xd0364141
	orderWordOne   uint32 = 0xbfd25e8c
	orderWordTwo   uint32 = 0xaf48a03b
	orderWordThree uint32 = 0xbaaedce6
	orderWordFour  uint32 = 0xfffffffe
	orderWordFive  uint32 = 0xffffffff
	orderWordSix   uint32 = 0xffffffff
	orderWordSeven uint32 = 0xffffffff

	// These fields provide convenient access to each of the words of the two's
	// complement of the secp256k1 curve group order N to improve code
	// readability.
	//
	// The two's complement of the group order is:
	// 0x00000000 00000000 00000000 00000001 45512319 50b75fc4 402da173 2fc9bebf
	orderComplementWordZero  uint32 = (^orderWordZero) + 1
	orderComplementWordOne   uint32 = ^orderWordOne
	orderComplementWordTwo   uint32 = ^orderWordTwo
	orderComplementWordThree uint32 = ^orderWordThree
	//orderComplementWordFour  uint32 = ^orderWordFour  // unused
	//orderComplementWordFive  uint32 = ^orderWordFive  // unused
	//orderComplementWordSix   uint32 = ^orderWordSix   // unused
	//orderComplementWordSeven uint32 = ^orderWordSeven // unused

	// These fields provide convenient access to each of the words of the
	// secp256k1 curve group order N / 2 to improve code readability and avoid
	// the need to recalculate them.
	//
	// The half order of the secp256k1 curve group is:
	// 0x7fffffff ffffffff ffffffff ffffffff 5d576e73 57a4501d dfe92f46 681b20a0
	halfOrderWordZero  uint32 = 0x681b20a0
	halfOrderWordOne   uint32 = 0xdfe92f46
	halfOrderWordTwo   uint32 = 0x57a4501d
	halfOrderWordThree uint32 = 0x5d576e73
	halfOrderWordFour  uint32 = 0xffffffff
	halfOrderWordFive  uint32 = 0xffffffff
	halfOrderWordSix   uint32 = 0xffffffff
	halfOrderWordSeven uint32 = 0x7fffffff

	// uint32Mask is simply a mask with all bits set for a uint32 and is used to
	// improve the readability of the code.
	uint32Mask = 0xffffffff
)

var (
	// zero32 is an array of 32 bytes used for the purposes of zeroing and is
	// defined here to avoid extra allocations.
	zero32 = [32]byte{}
)

// ModNScalar implements optimized 256-bit constant-time fixed-precision
// arithmetic over the secp256k1 group order. This means all arithmetic is
// performed modulo:
//
//	0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141
//
// It only implements the arithmetic needed for elliptic curve operations,
// however, the operations that are not implemented can typically be worked
// around if absolutely needed.  For example, subtraction can be performed by
// adding the negation.
//
// Should it be absolutely necessary, conversion to the standard library
// math/big.Int can be accomplished by using the Bytes method, slicing the
// resulting fixed-size array, and feeding it to big.Int.SetBytes.  However,
// that should typically be avoided when possible as conversion to big.Ints
// requires allocations, is not constant time, and is slower when working modulo
// the group order.
type ModNScalar struct {
	// The scalar is represented as 8 32-bit integers in base 2^32.
	//
	// The following depicts the internal representation:
	// 	 ---------------------------------------------------------
	// 	|       n[7]     |      n[6]      | ... |      n[0]      |
	// 	| 32 bits        | 32 bits        | ... | 32 bits        |
	// 	| Mult: 2^(32*7) | Mult: 2^(32*6) | ... | Mult: 2^(32*0) |
	// 	 ---------------------------------------------------------
	//
	// For example, consider the number 2^87 + 2^42 + 1.  It would be
	// represented as:
	// 	n[0] = 1
	// 	n[1] = 2^10
	// 	n[2] = 2^23
	// 	n[3..7] = 0
	//
	// The full 256-bit value is then calculated by looping i from 7..0 and
	// doing sum(n[i] * 2^(32i)) like so:
	// 	n[7] * 2^(32*7) = 0    * 2^224 = 0
	// 	n[6] * 2^(32*6) = 0    * 2^192 = 0
	// 	...
	// 	n[2] * 2^(32*2) = 2^23 * 2^64  = 2^87
	// 	n[1] * 2^(32*1) = 2^10 * 2^32  = 2^42
	// 	n[0] * 2^(32*0) = 1    * 2^0   = 1
	// 	Sum: 0 + 0 + ... + 2^87 + 2^42 + 1 = 2^87 + 2^42 + 1
	n [8]uint32
}

// String returns the scalar as a human-readable hex string.
//
// This is NOT constant time.
func (s ModNScalar) String() string {
	b := s.Bytes()
	return hex.EncodeToString(b[:])
}

// Set sets the scalar equal to a copy of the passed one in constant time.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s := new(ModNScalar).Set(s2).Add(1) so that s = s2 + 1 where s2 is not
// modified.
func (s *ModNScalar) Set(val *ModNScalar) *ModNScalar {
	*s = *val
	return s
}

// Zero sets the scalar to zero in constant time.  A newly created scalar is
// already set to zero.  This function can be useful to clear an existing scalar
// for reuse.
func (s *ModNScalar) Zero() {
	s.n[0] = 0
	s.n[1] = 0
	s.n[2] = 0
	s.n[3] = 0
	s.n[4] = 0
	s.n[5] = 0
	s.n[6] = 0
	s.n[7] = 0
}

// IsZeroBit returns 1 when the scalar is equal to zero or 0 otherwise in
// constant time.
//
// Note that a bool is not used here because it is not possible in Go to convert
// from a bool to numeric value in constant time and many constant-time
// operations require a numeric value.  See IsZero for the version that returns
// a bool.
func (s *ModNScalar) IsZeroBit() uint32 {
	// The scalar can only be zero if no bits are set in any of the words.
	bits := s.n[0] | s.n[1] | s.n[2] | s.n[3] | s.n[4] | s.n[5] | s.n[6] | s.n[7]
	return constantTimeEq(bits, 0)
}

// IsZero returns whether or not the scalar is equal to zero in constant time.
func (s *ModNScalar) IsZero() bool {
	// The scalar can only be zero if no bits are set in any of the words.
	bits := s.n[0] | s.n[1] | s.n[2] | s.n[3] | s.n[4] | s.n[5] | s.n[6] | s.n[7]
	return bits == 0
}

// SetInt sets the scalar to the passed integer in constant time.  This is a
// convenience function since it is fairly common to perform some arithmetic
// with small native integers.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s := new(ModNScalar).SetInt(2).Mul(s2) so that s = 2 * s2.
func (s *ModNScalar) SetInt(ui uint32) *ModNScalar {
	s.Zero()
	s.n[0] = ui
	return s
}

// constantTimeEq returns 1 if a == b or 0 otherwise in constant time.
func constantTimeEq(a, b uint32) uint32 {
	return uint32((uint64(a^b) - 1) >> 63)
}

// constantTimeNotEq returns 1 if a != b or 0 otherwise in constant time.
func constantTimeNotEq(a, b uint32) uint32 {
	return ^uint32((uint64(a^b)-1)>>63) & 1
}

// constantTimeLess returns 1 if a < b or 0 otherwise in constant time.
func constantTimeLess(a, b uint32) uint32 {
	return uint32((uint64(a) - uint64(b)) >> 63)
}

// constantTimeLessOrEq returns 1 if a <= b or 0 otherwise in constant time.
func constantTimeLessOrEq(a, b uint32) uint32 {
	return uint32((uint64(a) - uint64(b) - 1) >> 63)
}

// constantTimeGreater returns 1 if a > b or 0 otherwise in constant time.
func constantTimeGreater(a, b uint32) uint32 {
	return constantTimeLess(b, a)
}

// constantTimeGreaterOrEq returns 1 if a >= b or 0 otherwise in constant time.
func constantTimeGreaterOrEq(a, b uint32) uint32 {
	return constantTimeLessOrEq(b, a)
}

// constantTimeMin returns min(a,b) in constant time.
func constantTimeMin(a, b uint32) uint32 {
	return b ^ ((a ^ b) & -constantTimeLess(a, b))
}

// overflows determines if the current scalar is greater than or equal to the
// group order in constant time and returns 1 if it is or 0 otherwise.
func (s *ModNScalar) overflows() uint32 {
	// The intuition here is that the scalar is greater than the group order if
	// one of the higher individual words is greater than corresponding word of
	// the group order and all higher words in the scalar are equal to their
	// corresponding word of the group order.  Since this type is modulo the
	// group order, being equal is also an overflow back to 0.
	//
	// Note that the words 5, 6, and 7 are all the max uint32 value, so there is
	// no need to test if those individual words of the scalar exceeds them,
	// hence, only equality is checked for them.
	highWordsEqual := constantTimeEq(s.n[7], orderWordSeven)
	highWordsEqual &= constantTimeEq(s.n[6], orderWordSix)
	highWordsEqual &= constantTimeEq(s.n[5], orderWordFive)
	overflow := highWordsEqual & constantTimeGreater(s.n[4], orderWordFour)
	highWordsEqual &= constantTimeEq(s.n[4], orderWordFour)
	overflow |= highWordsEqual & constantTimeGreater(s.n[3], orderWordThree)
	highWordsEqual &= constantTimeEq(s.n[3], orderWordThree)
	overflow |= highWordsEqual & constantTimeGreater(s.n[2], orderWordTwo)
	highWordsEqual &= constantTimeEq(s.n[2], orderWordTwo)
	overflow |= highWordsEqual & constantTimeGreater(s.n[1], orderWordOne)
	highWordsEqual &= constantTimeEq(s.n[1], orderWordOne)
	overflow |= highWordsEqual & constantTimeGreaterOrEq(s.n[0], orderWordZero)

	return overflow
}

// reduce256 reduces the current scalar modulo the group order in accordance
// with the overflows parameter in constant time.  The overflows parameter
// specifies whether or not the scalar is known to be greater than the group
// order and MUST either be 1 in the case it is or 0 in the case it is not for a
// correct result.
func (s *ModNScalar) reduce256(overflows uint32) {
	// Notice that since s < 2^256 < 2N (where N is the group order), the max
	// possible number of reductions required is one.  Therefore, in the case a
	// reduction is needed, it can be performed with a single subtraction of N.
	// Also, recall that subtraction is equivalent to addition by the two's
	// complement while ignoring the carry.
	//
	// When s >= N, the overflows parameter will be 1.  Conversely, it will be 0
	// when s < N.  Thus multiplying by the overflows parameter will either
	// result in 0 or the multiplicand itself.
	//
	// Combining the above along with the fact that s + 0 = s, the following is
	// a constant time implementation that works by either adding 0 or the two's
	// complement of N as needed.
	//
	// The final result will be in the range 0 <= s < N as expected.
	overflows64 := uint64(overflows)
	c := uint64(s.n[0]) + overflows64*uint64(orderComplementWordZero)
	s.n[0] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(s.n[1]) + overflows64*uint64(orderComplementWordOne)
	s.n[1] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(s.n[2]) + overflows64*uint64(orderComplementWordTwo)
	s.n[2] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(s.n[3]) + overflows64*uint64(orderComplementWordThree)
	s.n[3] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(s.n[4]) + overflows64 // * 1
	s.n[4] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(s.n[5]) // + overflows64 * 0
	s.n[5] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(s.n[6]) // + overflows64 * 0
	s.n[6] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(s.n[7]) // + overflows64 * 0
	s.n[7] = uint32(c & uint32Mask)
}

// SetBytes interprets the provided array as a 256-bit big-endian unsigned
// integer, reduces it modulo the group order, sets the scalar to the result,
// and returns either 1 if it was reduced (aka it overflowed) or 0 otherwise in
// constant time.
//
// Note that a bool is not used here because it is not possible in Go to convert
// from a bool to numeric value in constant time and many constant-time
// operations require a numeric value.
func (s *ModNScalar) SetBytes(b *[32]byte) uint32 {
	// Pack the 256 total bits across the 8 uint32 words.  This could be done
	// with a for loop, but benchmarks show this unrolled version is about 2
	// times faster than the variant that uses a loop.
	s.n[0] = uint32(b[31]) | uint32(b[30])<<8 | uint32(b[29])<<16 | uint32(b[28])<<24
	s.n[1] = uint32(b[27]) | uint32(b[26])<<8 | uint32(b[25])<<16 | uint32(b[24])<<24
	s.n[2] = uint32(b[23]) | uint32(b[22])<<8 | uint32(b[21])<<16 | uint32(b[20])<<24
	s.n[3] = uint32(b[19]) | uint32(b[18])<<8 | uint32(b[17])<<16 | uint32(b[16])<<24
	s.n[4] = uint32(b[15]) | uint32(b[14])<<8 | uint32(b[13])<<16 | uint32(b[12])<<24
	s.n[5] = uint32(b[11]) | uint32(b[10])<<8 | uint32(b[9])<<16 | uint32(b[8])<<24
	s.n[6] = uint32(b[7]) | uint32(b[6])<<8 | uint32(b[5])<<16 | uint32(b[4])<<24
	s.n[7] = uint32(b[3]) | uint32(b[2])<<8 | uint32(b[1])<<16 | uint32(b[0])<<24

	// The value might be >= N, so reduce it as required and return whether or
	// not it was reduced.
	needsReduce := s.overflows()
	s.reduce256(needsReduce)
	return needsReduce
}

// zeroArray32 zeroes the provided 32-byte buffer.
func zeroArray32(b *[32]byte) {
	copy(b[:], zero32[:])
}

// SetByteSlice interprets the provided slice as a 256-bit big-endian unsigned
// integer (meaning it is truncated to the first 32 bytes), reduces it modulo
// the group order, sets the scalar to the result, and returns whether or not
// the resulting truncated 256-bit integer overflowed in constant time.
//
// Note that since passing a slice with more than 32 bytes is truncated, it is
// possible that the truncated value is less than the order of the curve and
// hence it will not be reported as having overflowed in that case.  It is up to
// the caller to decide whether it needs to provide numbers of the appropriate
// size or it is acceptable to use this function with the described truncation
// and overflow behavior.
func (s *ModNScalar) SetByteSlice(b []byte) bool {
	var b32 [32]byte
	b = b[:constantTimeMin(uint32(len(b)), 32)]
	copy(b32[:], b32[:32-len(b)])
	copy(b32[32-len(b):], b)
	result := s.SetBytes(&b32)
	zeroArray32(&b32)
	return result != 0
}

// PutBytesUnchecked unpacks the scalar to a 32-byte big-endian value directly
// into the passed byte slice in constant time.  The target slice must must have
// at least 32 bytes available or it will panic.
//
// There is a similar function, PutBytes, which unpacks the scalar into a
// 32-byte array directly.  This version is provided since it can be useful to
// write directly into part of a larger buffer without needing a separate
// allocation.
//
// Preconditions:
//   - The target slice MUST have at least 32 bytes available
func (s *ModNScalar) PutBytesUnchecked(b []byte) {
	// Unpack the 256 total bits from the 8 uint32 words.  This could be done
	// with a for loop, but benchmarks show this unrolled version is about 2
	// times faster than the variant which uses a loop.
	b[31] = byte(s.n[0])
	b[30] = byte(s.n[0] >> 8)
	b[29] = byte(s.n[0] >> 16)
	b[28] = byte(s.n[0] >> 24)
	b[27] = byte(s.n[1])
	b[26] = byte(s.n[1] >> 8)
	b[25] = byte(s.n[1] >> 16)
	b[24] = byte(s.n[1] >> 24)
	b[23] = byte(s.n[2])
	b[22] = byte(s.n[2] >> 8)
	b[21] = byte(s.n[2] >> 16)
	b[20] = byte(s.n[2] >> 24)
	b[19] = byte(s.n[3])
	b[18] = byte(s.n[3] >> 8)
	b[17] = byte(s.n[3] >> 16)
	b[16] = byte(s.n[3] >> 24)
	b[15] = byte(s.n[4])
	b[14] = byte(s.n[4] >> 8)
	b[13] = byte(s.n[4] >> 16)
	b[12] = byte(s.n[4] >> 24)
	b[11] = byte(s.n[5])
	b[10] = byte(s.n[5] >> 8)
	b[9] = byte(s.n[5] >> 16)
	b[8] = byte(s.n[5] >> 24)
	b[7] = byte(s.n[6])
	b[6] = byte(s.n[6] >> 8)
	b[5] = byte(s.n[6] >> 16)
	b[4] = byte(s.n[6] >> 24)
	b[3] = byte(s.n[7])
	b[2] = byte(s.n[7] >> 8)
	b[1] = byte(s.n[7] >> 16)
	b[0] = byte(s.n[7] >> 24)
}

// PutBytes unpacks the scalar to a 32-byte big-endian value using the passed
// byte array in constant time.
//
// There is a similar function, PutBytesUnchecked, which unpacks the scalar into
// a slice that must have at least 32 bytes available.  This version is provided
// since it can be useful to write directly into an array that is type checked.
//
// Alternatively, there is also Bytes, which unpacks the scalar into a new array
// and returns that which can sometimes be more ergonomic in applications that
// aren't concerned about an additional copy.
func (s *ModNScalar) PutBytes(b *[32]byte) {
	s.PutBytesUnchecked(b[:])
}

// Bytes unpacks the scalar to a 32-byte big-endian value in constant time.
//
// See PutBytes and PutBytesUnchecked for variants that allow an array or slice
// to be passed which can be useful to cut down on the number of allocations
// by allowing the caller to reuse a buffer or write directly into part of a
// larger buffer.
func (s *ModNScalar) Bytes() [32]byte {
	var b [32]byte
	s.PutBytesUnchecked(b[:])
	return b
}

// IsOdd returns whether or not the scalar is an odd number in constant time.
func (s *ModNScalar) IsOdd() bool {
	// Only odd numbers have the bottom bit set.
	return s.n[0]&1 == 1
}

// Equals returns whether or not the two scalars are the same in constant time.
func (s *ModNScalar) Equals(val *ModNScalar) bool {
	// Xor only sets bits when they are different, so the two scalars can only
	// be the same if no bits are set after xoring each word.
	bits := (s.n[0] ^ val.n[0]) | (s.n[1] ^ val.n[1]) | (s.n[2] ^ val.n[2]) |
		(s.n[3] ^ val.n[3]) | (s.n[4] ^ val.n[4]) | (s.n[5] ^ val.n[5]) |
		(s.n[6] ^ val.n[6]) | (s.n[7] ^ val.n[7])

	return bits == 0
}

// Add2 adds the passed two scalars together modulo the group order in constant
// time and stores the result in s.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s3.Add2(s, s2).AddInt(1) so that s3 = s + s2 + 1.
func (s *ModNScalar) Add2(val1, val2 *ModNScalar) *ModNScalar {
	c := uint64(val1.n[0]) + uint64(val2.n[0])
	s.n[0] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(val1.n[1]) + uint64(val2.n[1])
	s.n[1] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(val1.n[2]) + uint64(val2.n[2])
	s.n[2] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(val1.n[3]) + uint64(val2.n[3])
	s.n[3] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(val1.n[4]) + uint64(val2.n[4])
	s.n[4] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(val1.n[5]) + uint64(val2.n[5])
	s.n[5] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(val1.n[6]) + uint64(val2.n[6])
	s.n[6] = uint32(c & uint32Mask)
	c = (c >> 32) + uint64(val1.n[7]) + uint64(val2.n[7])
	s.n[7] = uint32(c & uint32Mask)

	// The result is now 256 bits, but it might still be >= N, so use the
	// existing normal reduce method for 256-bit values.
	s.reduce256(uint32(c>>32) + s.overflows())
	return s
}

// Add adds the passed scalar to the existing one modulo the group order in
// constant time and stores the result in s.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s.Add(s2).AddInt(1) so that s = s + s2 + 1.
func (s *ModNScalar) Add(val *ModNScalar) *ModNScalar {
	return s.Add2(s, val)
}

// accumulator96 provides a 96-bit accumulator for use in the intermediate
// calculations requiring more than 64-bits.
type accumulator96 struct {
	n [3]uint32
}

// Add adds the passed unsigned 64-bit value to the accumulator.
func (a *accumulator96) Add(v uint64) {
	low := uint32(v & uint32Mask)
	hi := uint32(v >> 32)
	a.n[0] += low
	hi += constantTimeLess(a.n[0], low) // Carry if overflow in n[0].
	a.n[1] += hi
	a.n[2] += constantTimeLess(a.n[1], hi) // Carry if overflow in n[1].
}

// Rsh32 right shifts the accumulator by 32 bits.
func (a *accumulator96) Rsh32() {
	a.n[0] = a.n[1]
	a.n[1] = a.n[2]
	a.n[2] = 0
}

// reduce385 reduces the 385-bit intermediate result in the passed terms modulo
// the group order in constant time and stores the result in s.
func (s *ModNScalar) reduce385(t0, t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12 uint64) {
	// At this point, the intermediate result in the passed terms has been
	// reduced to fit within 385 bits, so reduce it again using the same method
	// described in reduce512.  As before, the intermediate result will end up
	// being reduced by another 127 bits to 258 bits, thus 9 32-bit terms are
	// needed for this iteration.  The reduced terms are assigned back to t0
	// through t8.
	//
	// Note that several of the intermediate calculations require adding 64-bit
	// products together which would overflow a uint64, so a 96-bit accumulator
	// is used instead until the value is reduced enough to use native uint64s.

	// Terms for 2^(32*0).
	var acc accumulator96
	acc.n[0] = uint32(t0) // == acc.Add(t0) because acc is guaranteed to be 0.
	acc.Add(t8 * uint64(orderComplementWordZero))
	t0 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*1).
	acc.Add(t1)
	acc.Add(t8 * uint64(orderComplementWordOne))
	acc.Add(t9 * uint64(orderComplementWordZero))
	t1 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*2).
	acc.Add(t2)
	acc.Add(t8 * uint64(orderComplementWordTwo))
	acc.Add(t9 * uint64(orderComplementWordOne))
	acc.Add(t10 * uint64(orderComplementWordZero))
	t2 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*3).
	acc.Add(t3)
	acc.Add(t8 * uint64(orderComplementWordThree))
	acc.Add(t9 * uint64(orderComplementWordTwo))
	acc.Add(t10 * uint64(orderComplementWordOne))
	acc.Add(t11 * uint64(orderComplementWordZero))
	t3 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*4).
	acc.Add(t4)
	acc.Add(t8) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t9 * uint64(orderComplementWordThree))
	acc.Add(t10 * uint64(orderComplementWordTwo))
	acc.Add(t11 * uint64(orderComplementWordOne))
	acc.Add(t12 * uint64(orderComplementWordZero))
	t4 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*5).
	acc.Add(t5)
	// acc.Add(t8 * uint64(orderComplementWordFive)) // 0
	acc.Add(t9) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t10 * uint64(orderComplementWordThree))
	acc.Add(t11 * uint64(orderComplementWordTwo))
	acc.Add(t12 * uint64(orderComplementWordOne))
	t5 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*6).
	acc.Add(t6)
	// acc.Add(t8 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t9 * uint64(orderComplementWordFive)) // 0
	acc.Add(t10) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t11 * uint64(orderComplementWordThree))
	acc.Add(t12 * uint64(orderComplementWordTwo))
	t6 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*7).
	acc.Add(t7)
	// acc.Add(t8 * uint64(orderComplementWordSeven)) // 0
	// acc.Add(t9 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t10 * uint64(orderComplementWordFive)) // 0
	acc.Add(t11) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t12 * uint64(orderComplementWordThree))
	t7 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*8).
	// acc.Add(t9 * uint64(orderComplementWordSeven)) // 0
	// acc.Add(t10 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t11 * uint64(orderComplementWordFive)) // 0
	acc.Add(t12) // * uint64(orderComplementWordFour) // * 1
	t8 = uint64(acc.n[0])
	// acc.Rsh32() // No need since not used after this.  Guaranteed to be 0.

	// NOTE: All of the remaining multiplications for this iteration result in 0
	// as they all involve multiplying by combinations of the fifth, sixth, and
	// seventh words of the two's complement of N, which are 0, so skip them.

	// At this point, the result is reduced to fit within 258 bits, so reduce it
	// again using a slightly modified version of the same method.  The maximum
	// value in t8 is 2 at this point and therefore multiplying it by each word
	// of the two's complement of N and adding it to a 32-bit term will result
	// in a maximum requirement of 33 bits, so it is safe to use native uint64s
	// here for the intermediate term carry propagation.
	//
	// Also, since the maximum value in t8 is 2, this ends up reducing by
	// another 2 bits to 256 bits.
	c := t0 + t8*uint64(orderComplementWordZero)
	s.n[0] = uint32(c & uint32Mask)
	c = (c >> 32) + t1 + t8*uint64(orderComplementWordOne)
	s.n[1] = uint32(c & uint32Mask)
	c = (c >> 32) + t2 + t8*uint64(orderComplementWordTwo)
	s.n[2] = uint32(c & uint32Mask)
	c = (c >> 32) + t3 + t8*uint64(orderComplementWordThree)
	s.n[3] = uint32(c & uint32Mask)
	c = (c >> 32) + t4 + t8 // * uint64(orderComplementWordFour) == * 1
	s.n[4] = uint32(c & uint32Mask)
	c = (c >> 32) + t5 // + t8*uint64(orderComplementWordFive) == 0
	s.n[5] = uint32(c & uint32Mask)
	c = (c >> 32) + t6 // + t8*uint64(orderComplementWordSix) == 0
	s.n[6] = uint32(c & uint32Mask)
	c = (c >> 32) + t7 // + t8*uint64(orderComplementWordSeven) == 0
	s.n[7] = uint32(c & uint32Mask)

	// The result is now 256 bits, but it might still be >= N, so use the
	// existing normal reduce method for 256-bit values.
	s.reduce256(uint32(c>>32) + s.overflows())
}

// reduce512 reduces the 512-bit intermediate result in the passed terms modulo
// the group order down to 385 bits in constant time and stores the result in s.
func (s *ModNScalar) reduce512(t0, t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12, t13, t14, t15 uint64) {
	// At this point, the intermediate result in the passed terms is grouped
	// into the respective bases.
	//
	// Per [HAC] section 14.3.4: Reduction method of moduli of special form,
	// when the modulus is of the special form m = b^t - c, where log_2(c) < t,
	// highly efficient reduction can be achieved per the provided algorithm.
	//
	// The secp256k1 group order fits this criteria since it is:
	//   2^256 - 432420386565659656852420866394968145599
	//
	// Technically the max possible value here is (N-1)^2 since the two scalars
	// being multiplied are always mod N.  Nevertheless, it is safer to consider
	// it to be (2^256-1)^2 = 2^512 - 2^256 + 1 since it is the product of two
	// 256-bit values.
	//
	// The algorithm is to reduce the result modulo the prime by subtracting
	// multiples of the group order N.  However, in order simplify carry
	// propagation, this adds with the two's complement of N to achieve the same
	// result.
	//
	// Since the two's complement of N has 127 leading zero bits, this will end
	// up reducing the intermediate result from 512 bits to 385 bits, resulting
	// in 13 32-bit terms.  The reduced terms are assigned back to t0 through
	// t12.
	//
	// Note that several of the intermediate calculations require adding 64-bit
	// products together which would overflow a uint64, so a 96-bit accumulator
	// is used instead.

	// Terms for 2^(32*0).
	var acc accumulator96
	acc.n[0] = uint32(t0) // == acc.Add(t0) because acc is guaranteed to be 0.
	acc.Add(t8 * uint64(orderComplementWordZero))
	t0 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*1).
	acc.Add(t1)
	acc.Add(t8 * uint64(orderComplementWordOne))
	acc.Add(t9 * uint64(orderComplementWordZero))
	t1 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*2).
	acc.Add(t2)
	acc.Add(t8 * uint64(orderComplementWordTwo))
	acc.Add(t9 * uint64(orderComplementWordOne))
	acc.Add(t10 * uint64(orderComplementWordZero))
	t2 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*3).
	acc.Add(t3)
	acc.Add(t8 * uint64(orderComplementWordThree))
	acc.Add(t9 * uint64(orderComplementWordTwo))
	acc.Add(t10 * uint64(orderComplementWordOne))
	acc.Add(t11 * uint64(orderComplementWordZero))
	t3 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*4).
	acc.Add(t4)
	acc.Add(t8) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t9 * uint64(orderComplementWordThree))
	acc.Add(t10 * uint64(orderComplementWordTwo))
	acc.Add(t11 * uint64(orderComplementWordOne))
	acc.Add(t12 * uint64(orderComplementWordZero))
	t4 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*5).
	acc.Add(t5)
	// acc.Add(t8 * uint64(orderComplementWordFive)) // 0
	acc.Add(t9) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t10 * uint64(orderComplementWordThree))
	acc.Add(t11 * uint64(orderComplementWordTwo))
	acc.Add(t12 * uint64(orderComplementWordOne))
	acc.Add(t13 * uint64(orderComplementWordZero))
	t5 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*6).
	acc.Add(t6)
	// acc.Add(t8 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t9 * uint64(orderComplementWordFive)) // 0
	acc.Add(t10) // * uint64(orderComplementWordFour)) // * 1
	acc.Add(t11 * uint64(orderComplementWordThree))
	acc.Add(t12 * uint64(orderComplementWordTwo))
	acc.Add(t13 * uint64(orderComplementWordOne))
	acc.Add(t14 * uint64(orderComplementWordZero))
	t6 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*7).
	acc.Add(t7)
	// acc.Add(t8 * uint64(orderComplementWordSeven)) // 0
	// acc.Add(t9 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t10 * uint64(orderComplementWordFive)) // 0
	acc.Add(t11) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t12 * uint64(orderComplementWordThree))
	acc.Add(t13 * uint64(orderComplementWordTwo))
	acc.Add(t14 * uint64(orderComplementWordOne))
	acc.Add(t15 * uint64(orderComplementWordZero))
	t7 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*8).
	// acc.Add(t9 * uint64(orderComplementWordSeven)) // 0
	// acc.Add(t10 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t11 * uint64(orderComplementWordFive)) // 0
	acc.Add(t12) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t13 * uint64(orderComplementWordThree))
	acc.Add(t14 * uint64(orderComplementWordTwo))
	acc.Add(t15 * uint64(orderComplementWordOne))
	t8 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*9).
	// acc.Add(t10 * uint64(orderComplementWordSeven)) // 0
	// acc.Add(t11 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t12 * uint64(orderComplementWordFive)) // 0
	acc.Add(t13) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t14 * uint64(orderComplementWordThree))
	acc.Add(t15 * uint64(orderComplementWordTwo))
	t9 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*10).
	// acc.Add(t11 * uint64(orderComplementWordSeven)) // 0
	// acc.Add(t12 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t13 * uint64(orderComplementWordFive)) // 0
	acc.Add(t14) // * uint64(orderComplementWordFour) // * 1
	acc.Add(t15 * uint64(orderComplementWordThree))
	t10 = uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*11).
	// acc.Add(t12 * uint64(orderComplementWordSeven)) // 0
	// acc.Add(t13 * uint64(orderComplementWordSix)) // 0
	// acc.Add(t14 * uint64(orderComplementWordFive)) // 0
	acc.Add(t15) // * uint64(orderComplementWordFour) // * 1
	t11 = uint64(acc.n[0])
	acc.Rsh32()

	// NOTE: All of the remaining multiplications for this iteration result in 0
	// as they all involve multiplying by combinations of the fifth, sixth, and
	// seventh words of the two's complement of N, which are 0, so skip them.

	// Terms for 2^(32*12).
	t12 = uint64(acc.n[0])
	// acc.Rsh32() // No need since not used after this.  Guaranteed to be 0.

	// At this point, the result is reduced to fit within 385 bits, so reduce it
	// again using the same method accordingly.
	s.reduce385(t0, t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12)
}

// Mul2 multiplies the passed two scalars together modulo the group order in
// constant time and stores the result in s.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s3.Mul2(s, s2).AddInt(1) so that s3 = (s * s2) + 1.
func (s *ModNScalar) Mul2(val, val2 *ModNScalar) *ModNScalar {
	// This could be done with for loops and an array to store the intermediate
	// terms, but this unrolled version is significantly faster.

	// The overall strategy employed here is:
	// 1) Calculate the 512-bit product of the two scalars using the standard
	//    pencil-and-paper method.
	// 2) Reduce the result modulo the prime by effectively subtracting
	//    multiples of the group order N (actually performed by adding multiples
	//    of the two's complement of N to avoid implementing subtraction).
	// 3) Repeat step 2 noting that each iteration reduces the required number
	//    of bits by 127 because the two's complement of N has 127 leading zero
	//    bits.
	// 4) Once reduced to 256 bits, call the existing reduce method to perform
	//    a final reduction as needed.
	//
	// Note that several of the intermediate calculations require adding 64-bit
	// products together which would overflow a uint64, so a 96-bit accumulator
	// is used instead.

	// Terms for 2^(32*0).
	var acc accumulator96
	acc.Add(uint64(val.n[0]) * uint64(val2.n[0]))
	t0 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*1).
	acc.Add(uint64(val.n[0]) * uint64(val2.n[1]))
	acc.Add(uint64(val.n[1]) * uint64(val2.n[0]))
	t1 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*2).
	acc.Add(uint64(val.n[0]) * uint64(val2.n[2]))
	acc.Add(uint64(val.n[1]) * uint64(val2.n[1]))
	acc.Add(uint64(val.n[2]) * uint64(val2.n[0]))
	t2 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*3).
	acc.Add(uint64(val.n[0]) * uint64(val2.n[3]))
	acc.Add(uint64(val.n[1]) * uint64(val2.n[2]))
	acc.Add(uint64(val.n[2]) * uint64(val2.n[1]))
	acc.Add(uint64(val.n[3]) * uint64(val2.n[0]))
	t3 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*4).
	acc.Add(uint64(val.n[0]) * uint64(val2.n[4]))
	acc.Add(uint64(val.n[1]) * uint64(val2.n[3]))
	acc.Add(uint64(val.n[2]) * uint64(val2.n[2]))
	acc.Add(uint64(val.n[3]) * uint64(val2.n[1]))
	acc.Add(uint64(val.n[4]) * uint64(val2.n[0]))
	t4 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*5).
	acc.Add(uint64(val.n[0]) * uint64(val2.n[5]))
	acc.Add(uint64(val.n[1]) * uint64(val2.n[4]))
	acc.Add(uint64(val.n[2]) * uint64(val2.n[3]))
	acc.Add(uint64(val.n[3]) * uint64(val2.n[2]))
	acc.Add(uint64(val.n[4]) * uint64(val2.n[1]))
	acc.Add(uint64(val.n[5]) * uint64(val2.n[0]))
	t5 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*6).
	acc.Add(uint64(val.n[0]) * uint64(val2.n[6]))
	acc.Add(uint64(val.n[1]) * uint64(val2.n[5]))
	acc.Add(uint64(val.n[2]) * uint64(val2.n[4]))
	acc.Add(uint64(val.n[3]) * uint64(val2.n[3]))
	acc.Add(uint64(val.n[4]) * uint64(val2.n[2]))
	acc.Add(uint64(val.n[5]) * uint64(val2.n[1]))
	acc.Add(uint64(val.n[6]) * uint64(val2.n[0]))
	t6 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*7).
	acc.Add(uint64(val.n[0]) * uint64(val2.n[7]))
	acc.Add(uint64(val.n[1]) * uint64(val2.n[6]))
	acc.Add(uint64(val.n[2]) * uint64(val2.n[5]))
	acc.Add(uint64(val.n[3]) * uint64(val2.n[4]))
	acc.Add(uint64(val.n[4]) * uint64(val2.n[3]))
	acc.Add(uint64(val.n[5]) * uint64(val2.n[2]))
	acc.Add(uint64(val.n[6]) * uint64(val2.n[1]))
	acc.Add(uint64(val.n[7]) * uint64(val2.n[0]))
	t7 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*8).
	acc.Add(uint64(val.n[1]) * uint64(val2.n[7]))
	acc.Add(uint64(val.n[2]) * uint64(val2.n[6]))
	acc.Add(uint64(val.n[3]) * uint64(val2.n[5]))
	acc.Add(uint64(val.n[4]) * uint64(val2.n[4]))
	acc.Add(uint64(val.n[5]) * uint64(val2.n[3]))
	acc.Add(uint64(val.n[6]) * uint64(val2.n[2]))
	acc.Add(uint64(val.n[7]) * uint64(val2.n[1]))
	t8 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*9).
	acc.Add(uint64(val.n[2]) * uint64(val2.n[7]))
	acc.Add(uint64(val.n[3]) * uint64(val2.n[6]))
	acc.Add(uint64(val.n[4]) * uint64(val2.n[5]))
	acc.Add(uint64(val.n[5]) * uint64(val2.n[4]))
	acc.Add(uint64(val.n[6]) * uint64(val2.n[3]))
	acc.Add(uint64(val.n[7]) * uint64(val2.n[2]))
	t9 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*10).
	acc.Add(uint64(val.n[3]) * uint64(val2.n[7]))
	acc.Add(uint64(val.n[4]) * uint64(val2.n[6]))
	acc.Add(uint64(val.n[5]) * uint64(val2.n[5]))
	acc.Add(uint64(val.n[6]) * uint64(val2.n[4]))
	acc.Add(uint64(val.n[7]) * uint64(val2.n[3]))
	t10 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*11).
	acc.Add(uint64(val.n[4]) * uint64(val2.n[7]))
	acc.Add(uint64(val.n[5]) * uint64(val2.n[6]))
	acc.Add(uint64(val.n[6]) * uint64(val2.n[5]))
	acc.Add(uint64(val.n[7]) * uint64(val2.n[4]))
	t11 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*12).
	acc.Add(uint64(val.n[5]) * uint64(val2.n[7]))
	acc.Add(uint64(val.n[6]) * uint64(val2.n[6]))
	acc.Add(uint64(val.n[7]) * uint64(val2.n[5]))
	t12 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*13).
	acc.Add(uint64(val.n[6]) * uint64(val2.n[7]))
	acc.Add(uint64(val.n[7]) * uint64(val2.n[6]))
	t13 := uint64(acc.n[0])
	acc.Rsh32()

	// Terms for 2^(32*14).
	acc.Add(uint64(val.n[7]) * uint64(val2.n[7]))
	t14 := uint64(acc.n[0])
	acc.Rsh32()

	// What's left is for 2^(32*15).
	t15 := uint64(acc.n[0])
	// acc.Rsh32() // No need since not used after this.  Guaranteed to be 0.

	// At this point, all of the terms are grouped into their respective base
	// and occupy up to 512 bits.  Reduce the result accordingly.
	s.reduce512(t0, t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12, t13, t14,
		t15)
	return s
}

// Mul multiplies the passed scalar with the existing one modulo the group order
// in constant time and stores the result in s.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s.Mul(s2).AddInt(1) so that s = (s * s2) + 1.
func (s *ModNScalar) Mul(val *ModNScalar) *ModNScalar {
	return s.Mul2(s, val)
}

// SquareVal squares the passed scalar modulo the group order in constant time
// and stores the result in s.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s3.SquareVal(s).Mul(s) so that s3 = s^2 * s = s^3.
func (s *ModNScalar) SquareVal(val *ModNScalar) *ModNScalar {
	// This could technically be optimized slightly to take advantage of the
	// fact that many of the intermediate calculations in squaring are just
	// doubling, however, benchmarking has shown that due to the need to use a
	// 96-bit accumulator, any savings are essentially offset by that and
	// consequently there is no real difference in performance over just
	// multiplying the value by itself to justify the extra code for now.  This
	// can be revisited in the future if it becomes a bottleneck in practice.

	return s.Mul2(val, val)
}

// Square squares the scalar modulo the group order in constant time.  The
// existing scalar is modified.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s.Square().Mul(s2) so that s = s^2 * s2.
func (s *ModNScalar) Square() *ModNScalar {
	return s.SquareVal(s)
}

// NegateVal negates the passed scalar modulo the group order and stores the
// result in s in constant time.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s.NegateVal(s2).AddInt(1) so that s = -s2 + 1.
func (s *ModNScalar) NegateVal(val *ModNScalar) *ModNScalar {
	// Since the scalar is already in the range 0 <= val < N, where N is the
	// group order, negation modulo the group order is just the group order
	// minus the value.  This implies that the result will always be in the
	// desired range with the sole exception of 0 because N - 0 = N itself.
	//
	// Therefore, in order to avoid the need to reduce the result for every
	// other case in order to achieve constant time, this creates a mask that is
	// all 0s in the case of the scalar being negated is 0 and all 1s otherwise
	// and bitwise ands that mask with each word.
	//
	// Finally, to simplify the carry propagation, this adds the two's
	// complement of the scalar to N in order to achieve the same result.
	bits := val.n[0] | val.n[1] | val.n[2] | val.n[3] | val.n[4] | val.n[5] |
		val.n[6] | val.n[7]
	mask := uint64(uint32Mask * constantTimeNotEq(bits, 0))
	c := uint64(orderWordZero) + (uint64(^val.n[0]) + 1)
	s.n[0] = uint32(c & mask)
	c = (c >> 32) + uint64(orderWordOne) + uint64(^val.n[1])
	s.n[1] = uint32(c & mask)
	c = (c >> 32) + uint64(orderWordTwo) + uint64(^val.n[2])
	s.n[2] = uint32(c & mask)
	c = (c >> 32) + uint64(orderWordThree) + uint64(^val.n[3])
	s.n[3] = uint32(c & mask)
	c = (c >> 32) + uint64(orderWordFour) + uint64(^val.n[4])
	s.n[4] = uint32(c & mask)
	c = (c >> 32) + uint64(orderWordFive) + uint64(^val.n[5])
	s.n[5] = uint32(c & mask)
	c = (c >> 32) + uint64(orderWordSix) + uint64(^val.n[6])
	s.n[6] = uint32(c & mask)
	c = (c >> 32) + uint64(orderWordSeven) + uint64(^val.n[7])
	s.n[7] = uint32(c & mask)
	return s
}

// Negate negates the scalar modulo the group order in constant time.  The
// existing scalar is modified.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s.Negate().AddInt(1) so that s = -s + 1.
func (s *ModNScalar) Negate() *ModNScalar {
	return s.NegateVal(s)
}

// InverseValNonConst finds the modular multiplicative inverse of the passed
// scalar and stores result in s in *non-constant* time.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s3.InverseVal(s1).Mul(s2) so that s3 = s1^-1 * s2.
func (s *ModNScalar) InverseValNonConst(val *ModNScalar) *ModNScalar {
	// This is making use of big integers for now.  Ideally it will be replaced
	// with an implementation that does not depend on big integers.
	valBytes := val.Bytes()
	bigVal := new(big.Int).SetBytes(valBytes[:])
	bigVal.ModInverse(bigVal, curveParams.N)
	s.SetByteSlice(bigVal.Bytes())
	return s
}

// InverseNonConst finds the modular multiplicative inverse of the scalar in
// *non-constant* time.  The existing scalar is modified.
//
// The scalar is returned to support chaining.  This enables syntax like:
// s.Inverse().Mul(s2) so that s = s^-1 * s2.
func (s *ModNScalar) InverseNonConst() *ModNScalar {
	return s.InverseValNonConst(s)
}

// IsOverHalfOrder returns whether or not the scalar exceeds the group order
// divided by 2 in constant time.
func (s *ModNScalar) IsOverHalfOrder() bool {
	// The intuition here is that the scalar is greater than half of the group
	// order if one of the higher individual words is greater than the
	// corresponding word of the half group order and all higher words in the
	// scalar are equal to their corresponding word of the half group order.
	//
	// Note that the words 4, 5, and 6 are all the max uint32 value, so there is
	// no need to test if those individual words of the scalar exceeds them,
	// hence, only equality is checked for them.
	result := constantTimeGreater(s.n[7], halfOrderWordSeven)
	highWordsEqual := constantTimeEq(s.n[7], halfOrderWordSeven)
	highWordsEqual &= constantTimeEq(s.n[6], halfOrderWordSix)
	highWordsEqual &= constantTimeEq(s.n[5], halfOrderWordFive)
	highWordsEqual &= constantTimeEq(s.n[4], halfOrderWordFour)
	result |= highWordsEqual & constantTimeGreater(s.n[3], halfOrderWordThree)
	highWordsEqual &= constantTimeEq(s.n[3], halfOrderWordThree)
	result |= highWordsEqual & constantTimeGreater(s.n[2], halfOrderWordTwo)
	highWordsEqual &= constantTimeEq(s.n[2], halfOrderWordTwo)
	result |= highWordsEqual & constantTimeGreater(s.n[1], halfOrderWordOne)
	highWordsEqual &= constantTimeEq(s.n[1], halfOrderWordOne)
	result |= highWordsEqual & constantTimeGreater(s.n[0], halfOrderWordZero)

	return result != 0
}
