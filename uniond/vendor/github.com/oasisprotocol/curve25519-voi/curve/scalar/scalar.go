// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// Package scalar implements arithmetic on scalars (integers mod the group
// order).
package scalar

import (
	"bytes"
	"crypto/rand"
	"encoding/binary"
	"fmt"
	"io"

	"github.com/oasisprotocol/curve25519-voi/internal/disalloweq"
	"github.com/oasisprotocol/curve25519-voi/internal/subtle"
	_ "github.com/oasisprotocol/curve25519-voi/internal/toolchain"
)

const (
	// ScalarSize is the size of a scalar in bytes.
	ScalarSize = 32

	// ScalarWideSize is the size of a wide scalar in bytes.
	ScalarWideSize = 64
)

var (
	errScalarNotCanonical  = fmt.Errorf("curve/scalar: representative not canonical")
	errUnexpectedInputSize = fmt.Errorf("curve/scalar: unexpected input size")

	// BASEPOINT_ORDER is the order of the Ed25519 basepoint and the Ristretto
	// group.
	BASEPOINT_ORDER = func() *Scalar {
		// This is kind of ugly but the basepoint order isn't a canonical
		// scalar for reasons that should be obvious.  The bit based
		// deserialization API only masks the high bit (and does not reduce),
		// so it works to construct the constant.
		s, err := NewFromBits([]byte{
			0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
			0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
		})
		if err != nil {
			panic("curve/scalar: failed to define basepoint order constant: " + err.Error())
		}
		return s
	}()
)

// Scalar holds an integer s < 2^255 which represents an element of
// Z/L.
type Scalar struct {
	disalloweq.DisallowEqual //nolint:unused
	inner                    [ScalarSize]byte
}

// MarshalBinary encodes the scalar into a binary form and returns the
// result.
func (s *Scalar) MarshalBinary() ([]byte, error) {
	b := make([]byte, ScalarSize)
	return b, s.ToBytes(b)
}

// UnmarshalBinary decodes a binary serialized scalar.
func (s *Scalar) UnmarshalBinary(data []byte) error {
	_, err := s.SetCanonicalBytes(data)
	return err
}

// Set sets s to t, and returns s.
func (s *Scalar) Set(t *Scalar) *Scalar {
	*s = *t
	return s
}

// SetUint64 sets s to the given uint64, and returns s.
func (s *Scalar) SetUint64(x uint64) *Scalar {
	var sBytes [ScalarSize]byte
	binary.LittleEndian.PutUint64(sBytes[0:8], x)
	s.inner = sBytes

	return s
}

// SetBytesModOrder sets s to the scalar constructed by reducing a 256-bit
// little-endian integer modulo the group order L.
func (s *Scalar) SetBytesModOrder(in []byte) (*Scalar, error) {
	if len(in) != ScalarSize {
		return nil, errUnexpectedInputSize
	}

	// Temporarily allow s_unreduced.bytes > 2^255 ...
	copy(s.inner[:], in)

	// Then reduce mod the group order.
	return s.Reduce(s), nil
}

// SetBytesModOrderWide sets s to the scalar constructed by reducing a 512-bit
// little-endian integer modulo the group order L.
func (s *Scalar) SetBytesModOrderWide(in []byte) (*Scalar, error) {
	us, err := newUnpackedScalar().SetBytesWide(in)
	if err != nil {
		return nil, err
	}

	return s.pack(us), nil
}

// SetCanonicalBytes sets s from a canonical byte representation.
func (s *Scalar) SetCanonicalBytes(in []byte) (*Scalar, error) {
	candidate, err := New().SetBits(in)
	if err != nil {
		return nil, err
	}

	// Check that the high bit is not set, and that the candidate is
	// canonical.
	if in[31]>>7 != 0 || !candidate.IsCanonical() {
		return nil, errScalarNotCanonical
	}

	return s.Set(candidate), nil
}

// SetBits constructs a scalar from the low 255 bits of a 256-bit integer.
//
// This function is intended for applications like X25519 which
// require specific bit-patterns when performing scalar
// multiplication.
func (s *Scalar) SetBits(in []byte) (*Scalar, error) {
	if len(in) != ScalarSize {
		return nil, errUnexpectedInputSize
	}

	copy(s.inner[:], in)
	// Ensure that s < 2^255 by masking the high bit
	s.inner[31] &= 0x7f // 0b0111_1111

	return s, nil
}

// SetRandom sets s to a scalar chosen uniformly at random using entropy
// from the user-provided io.Reader.  If rng is nil, the runtime library's
// entropy source will be used.
func (s *Scalar) SetRandom(rng io.Reader) (*Scalar, error) {
	var scalarBytes [ScalarWideSize]byte

	if rng == nil {
		rng = rand.Reader
	}
	if _, err := io.ReadFull(rng, scalarBytes[:]); err != nil {
		return nil, fmt.Errorf("curve/scalar: failed to read entropy: %w", err)
	}

	return s.SetBytesModOrderWide(scalarBytes[:])
}

// Equal returns 1 iff the s and t are equal, 0 otherwise.
// This function will execute in constant-time.
func (s *Scalar) Equal(t *Scalar) int {
	return subtle.ConstantTimeCompareBytes(s.inner[:], t.inner[:])
}

// Mul sets `s = a * b (mod l)`, and returns s.
func (s *Scalar) Mul(a, b *Scalar) *Scalar {
	unpacked := a.unpack()
	return s.pack(unpacked.Mul(unpacked, b.unpack()))
}

// Add sets `s= a + b (mod l)`, and returns s.
func (s *Scalar) Add(a, b *Scalar) *Scalar {
	unpacked := a.unpack()

	// The unpackedScalar.Add function produces reduced outputs
	// if the inputs are reduced.  However, these inputs may not
	// be reduced -- they might come from Scalar.SetBits.  So
	// after computing the sum, we explicitly reduce it mod l
	// before repacking.
	z := scalarMulInternal(unpacked.Add(unpacked, b.unpack()), &constR)
	return s.pack(unpacked.MontgomeryReduce(&z))
}

// Sub sets `s = a - b (mod l)`, and returns s.
func (s *Scalar) Sub(a, b *Scalar) *Scalar {
	unpacked, unpackedB := a.unpack(), b.unpack()

	// The unpackedScalar.Sub function requires reduced inputs
	// and produces reduced output. However, these inputs may not
	// be reduced -- they might come from Scalar.SetBits.  So
	// we explicitly reduce the inputs.
	z := scalarMulInternal(unpacked, &constR)
	unpacked.MontgomeryReduce(&z)
	z = scalarMulInternal(unpackedB, &constR)
	unpackedB.MontgomeryReduce(&z)
	return s.pack(unpacked.Sub(unpacked, unpackedB))
}

// Neg `s = -t`, and returns s.
func (s *Scalar) Neg(t *Scalar) *Scalar {
	unpacked := t.unpack()

	z := scalarMulInternal(unpacked, &constR)
	return s.pack(unpacked.Sub(newUnpackedScalar(), unpacked.MontgomeryReduce(&z)))
}

// ConditionalSelect sets s to a iff choice == 0 and b iff choice == 1.
func (s *Scalar) ConditionalSelect(a, b *Scalar, choice int) {
	// TODO/perf: This will be kind of slow, consider optimizing it
	// if the call is used frequently enough to matter.

	// Note: The rust subtle crate has inverted choice behavior for
	// select vs the Go runtime library package.
	for i := range s.inner {
		s.inner[i] = subtle.ConstantTimeSelectByte(choice, b.inner[i], a.inner[i])
	}
}

// Product sets s to the product of values, and returns s.
func (s *Scalar) Product(values []*Scalar) *Scalar {
	product := NewFromUint64(1)

	for _, v := range values {
		product.Mul(product, v)
	}

	return s.Set(product)
}

// Sum sets s to the sum of values, and returns s.
func (s *Scalar) Sum(values []*Scalar) *Scalar {
	sum := New()

	for _, v := range values {
		sum.Add(sum, v)
	}

	return s.Set(sum)
}

// ToBytes packs the scalar into 32 bytes.
func (s *Scalar) ToBytes(out []byte) error {
	if len(out) != ScalarSize {
		return fmt.Errorf("curve/scalar: unexpected output size")
	}

	copy(out, s.inner[:])

	return nil
}

// Zero sets s to zero, and returns s.
func (s *Scalar) Zero() *Scalar {
	for i := range s.inner {
		s.inner[i] = 0
	}

	return s
}

// One sets s to one, and returns s.
func (s *Scalar) One() *Scalar {
	s.inner = [ScalarSize]byte{
		1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	}

	return s
}

// Invert sets s to the multiplicative inverse of the nonzero scalar t,
// and returns s.
//
// WARNING: The scalar MUST be nonzero.  If you cannot prove that this is
// the case you MUST not use this function.
func (s *Scalar) Invert(t *Scalar) *Scalar {
	return s.pack(newUnpackedScalar().Invert(t.unpack()))
}

// BatchInvert computes the inverses of slice of `Scalar`s in a batch,
// and sets s to the product of all inverses, and returns s.  Each
// element of the input slice is replaced by its inverse.
//
// WARNING: The input scalars MUST be nonzero.  If you cannot prove
// that this is the case you MUST not use this function.
func (s *Scalar) BatchInvert(inputs []*Scalar) *Scalar {
	n := len(inputs)
	unpackedOne := func() unpackedScalar {
		us := newUnpackedScalar().ToMontgomery(One().unpack())
		return *us
	}()

	// TODO: In theory this should be sanitized.
	scratch := make([]unpackedScalar, 0, n)

	// Keep an accumulator of all of the previous products.
	acc := unpackedOne
	for i, input := range inputs {
		scratch = append(scratch, acc)

		// Avoid unnecessary Montgomery multiplication in second pass by
		// keeping inputs in Montgomery form.
		tmp := newUnpackedScalar().ToMontgomery(input.unpack())
		inputs[i].pack(tmp)
		acc.MontgomeryMul(&acc, tmp)
	}

	// Compute the inverse of all products.
	acc.MontgomeryInvert()
	acc.FromMontgomery(&acc)

	// We need to return the product of all inverses later.
	ret := New().pack(&acc)

	// Pass through the vector backwards to compute the inverses
	// in place.
	for i := n - 1; i >= 0; i-- {
		input, scratch := inputs[i], scratch[i]
		tmp := newUnpackedScalar().MontgomeryMul(&acc, input.unpack())
		tmp2 := newUnpackedScalar().MontgomeryMul(&acc, &scratch)
		inputs[i].pack(tmp2)
		acc = *tmp
	}

	return s.Set(ret)
}

// Bits gets the bits of the scalar.
func (s *Scalar) Bits() [8 * ScalarSize]byte {
	var out [8 * ScalarSize]byte

	for i := range out {
		out[i] = (s.inner[i>>3] >> (i & 7)) & 1
	}

	return out
}

// NonAdjacentForm returns a width-w "Non-Adjacent Form" of this scalar.
func (s *Scalar) NonAdjacentForm(w uint) [256]int8 {
	if w < 2 || w > 8 {
		panic("curve/scalar: invalid width parameter")
	}

	var (
		naf [256]int8
		x   [5]uint64
	)
	for i := 0; i < 4; i++ {
		x[i] = binary.LittleEndian.Uint64(s.inner[i*8:])
	}

	width := uint64(1 << w)
	windowMask := uint64(width - 1)

	var (
		pos   uint
		carry uint64
	)
	for pos < 256 {
		// Construct a buffer of bits of the scalar, starting at bit `pos`
		idx := pos / 64
		bitIdx := pos % 64
		var bitBuf uint64
		if bitIdx < 64-w {
			// This window's bits are contained in a single u64
			bitBuf = x[idx] >> bitIdx
		} else {
			// Combine the current u64's bits with the bits from the next u64
			bitBuf = (x[idx] >> bitIdx) | (x[1+idx] << (64 - bitIdx))
		}

		// Add the carry into the current window
		window := carry + (bitBuf & windowMask)

		if window&1 == 0 {
			// If the window value is even, preserve the carry and continue.
			// Why is the carry preserved?
			// If carry == 0 and window & 1 == 0, then the next carry should be 0
			// If carry == 1 and window & 1 == 0, then bit_buf & 1 == 1 so the next carry should be 1
			pos += 1
			continue
		}

		if window < width/2 {
			carry = 0
			naf[pos] = int8(window)
		} else {
			carry = 1
			naf[pos] = int8(window) - int8(width)
		}

		pos += w
	}

	return naf
}

// ToRadix16 returns the scalar in radix 16, with coefficients in [-8,8).
func (s *Scalar) ToRadix16() [64]int8 {
	var output [64]int8

	// Step 1: change radix.
	// Convert from radix 256 (bytes) to radix 16 (nibbles)
	botHalf := func(x uint8) uint8 {
		return (x >> 0) & 15
	}
	topHalf := func(x uint8) uint8 {
		return (x >> 4) & 15
	}

	for i := 0; i < 32; i++ {
		output[2*i] = int8(botHalf(s.inner[i]))
		output[2*i+1] = int8(topHalf(s.inner[i]))
	}
	// Precondition note: since self[31] <= 127, output[63] <= 7

	// Step 2: recenter coefficients from [0,16) to [-8,8)
	for i := 0; i < 63; i++ {
		carry := (output[i] + 8) >> 4
		output[i] -= carry << 4
		output[i+1] += carry
	}
	// Precondition note: output[63] is not recentered.  It
	// increases by carry <= 1.  Thus output[63] <= 8.

	return output
}

// ToRadix2wSizeHint returns a size hint indicating how many entries of
// the return value of ToRadix2w are nonzero.
func ToRadix2wSizeHint(w uint) uint {
	switch w {
	case 6, 7:
		return (256 + w - 1) / w
	case 8:
		// See comment in toRadix2w on handling the terminal carry.
		return (256+w-1)/w + 1
	default:
		panic("curve/scalar: invalid radix parameter")
	}
}

// ToRadix2w returns a representation of a scalar in radix 64, 128, or 256.
func (s *Scalar) ToRadix2w(w uint) [43]int8 {
	_ = ToRadix2wSizeHint(w)

	// Scalar formatted as four `uint64`s with the carry bit packed
	// into the highest bit.
	var scalar64x4 [4]uint64
	for i := 0; i < 4; i++ {
		scalar64x4[i] = binary.LittleEndian.Uint64(s.inner[i*8:])
	}

	radix := uint64(1 << w)
	windowMask := radix - 1
	digitsCount := (254 + w - 1) / w

	var (
		carry  uint64
		digits [43]int8
	)
	for i := uint(0); i < digitsCount; i++ {
		// Construct a buffer of bits of the scalar, starting at `bitOffset`.
		bitOffset := i * w
		u64Idx := bitOffset / 64
		bitIdx := bitOffset % 64

		// Read the bits from the scalar.
		var bitBuf uint64
		if bitIdx < 64-w || u64Idx == 3 {
			// This window's bits are contained in a single uint64,
			// or it's the last uint64 anyway.
			bitBuf = scalar64x4[u64Idx] >> bitIdx
		} else {
			// Combine the current u64's bits with the bits from the next u64
			bitBuf = (scalar64x4[u64Idx] >> bitIdx) | (scalar64x4[1+u64Idx] << (64 - bitIdx))
		}

		// Read the actual coefficient value from the window
		coef := carry + (bitBuf & windowMask) // coef = [0, 2^r)

		// Recenter coefficients from [0,2^w) to [-2^w/2, 2^w/2)
		carry = (coef + (radix / 2)) >> w
		digits[i] = int8(int64(coef) - int64(carry<<w))
	}

	// When w < 8, we can fold the final carry onto the last digit d,
	// because d < 2^w/2 so d + carry*2^w = d + 1*2^w < 2^(w+1) < 2^8.
	//
	// When w = 8, we can't fit carry*2^w into an i8.  This should
	// not happen anyways, because the final carry will be 0 for
	// reduced scalars, but the Scalar invariant allows 255-bit scalars.
	// To handle this, we expand the size_hint by 1 when w=8,
	// and accumulate the final carry onto another digit.
	switch w {
	case 8:
		digits[digitsCount] += int8(carry)
	default:
		digits[digitsCount-1] += int8(carry << w)
	}

	return digits
}

// Reduce reduces t modulo L, and returns s.
func (s *Scalar) Reduce(t *Scalar) *Scalar {
	xR := scalarMulInternal(t.unpack(), &constR)
	return s.pack(newUnpackedScalar().MontgomeryReduce(&xR))
}

// IsCanonical checks if this scalar is the canonical representative mod L.
//
// This is intended for uses like input validation, where variable-time code
// is acceptable.
func (s *Scalar) IsCanonical() bool {
	sReduced := New().Reduce(s)
	return bytes.Equal(s.inner[:], sReduced.inner[:])
}

func (s *Scalar) unpack() *unpackedScalar {
	return newUnpackedScalar().SetBytes(s.inner[:])
}

func (s *Scalar) pack(us *unpackedScalar) *Scalar {
	us.ToBytes(s.inner[:])
	return s
}

// New returns a scalar set to zero.
func New() *Scalar {
	return &Scalar{}
}

// NewFromBytesModOrder constructs a scalar by reducing a 256-bit
// little-endian integer modulo the group order L.
func NewFromBytesModOrder(in []byte) (*Scalar, error) {
	return New().SetBytesModOrder(in)
}

// NewFromBytesModOrderWide constructs a scalar by reducing a 512-bit
// little-endian integer modulo the group order L.
func NewFromBytesModOrderWide(in []byte) (*Scalar, error) {
	return New().SetBytesModOrderWide(in)
}

// NewFromCanonicalBytes attempts to construct a scalar from a canoical
// byte representation.
func NewFromCanonicalBytes(in []byte) (*Scalar, error) {
	return New().SetCanonicalBytes(in)
}

// NewFromBits constructs a scalar from the low 255 bits of a 256-bit integer.
//
// This function is intended for applications like X25519 which
// require specific bit-patterns when performing scalar
// multiplication.
func NewFromBits(in []byte) (*Scalar, error) {
	return New().SetBits(in)
}

// NewFromUint64 returns a scalar set to the given uint64.
func NewFromUint64(x uint64) *Scalar {
	return New().SetUint64(x)
}

// One returns a scalar set to 1.
func One() *Scalar {
	return New().One()
}
