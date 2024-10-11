// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

// Package field implements arithmetic modulo p = 2^256 - 2^32 - 977.
package field

import (
	"crypto/rand"
	"encoding/hex"
	"errors"

	"gitlab.com/yawning/secp256k1-voi/internal/disalloweq"
	fiat "gitlab.com/yawning/secp256k1-voi/internal/fiat/secp256k1montgomery"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

const (
	// ElementSize is the size of a field element in bytes.
	ElementSize = 32

	// WideElementSize is the size of a wide field element in bytes.
	WideElementSize = 64
)

var (
	mSat = func() [5]uint64 {
		var m [5]uint64
		fiat.Msat(&m)
		return m
	}()

	feZero Element
	feOne  = NewElement().One()

	errNonCanonicalEncoding = errors.New("internal/field: value out of range")
)

// Element is a field element.  All arguments and receivers are allowed
// to alias.  The zero value is a valid zero element.
type Element struct {
	_ disalloweq.DisallowEqual
	m fiat.MontgomeryDomainFieldElement
}

// Zero sets `fe = 0` and returns `fe`.
func (fe *Element) Zero() *Element {
	for i := range fe.m {
		fe.m[i] = 0
	}
	return fe
}

// One sets `fe = 1` and returns `fe`.
func (fe *Element) One() *Element {
	fiat.SetOne(&fe.m)
	return fe
}

// Add sets `fe = a + b` and returns `fe`.
func (fe *Element) Add(a, b *Element) *Element {
	fiat.Add(&fe.m, &a.m, &b.m)
	return fe
}

// Subtract sets `fe = a - b` and returns `fe`.
func (fe *Element) Subtract(a, b *Element) *Element {
	fiat.Sub(&fe.m, &a.m, &b.m)
	return fe
}

// Negate sets `fe = -a` and returns `fe`.
func (fe *Element) Negate(a *Element) *Element {
	fiat.Opp(&fe.m, &a.m)
	return fe
}

// Multiply sets `fe = a * b` and returns `fe`.
func (fe *Element) Multiply(a, b *Element) *Element {
	fiat.Mul(&fe.m, &a.m, &b.m)
	return fe
}

// Square sets `fe = a * a` and returns `fe`.
func (fe *Element) Square(a *Element) *Element {
	fiat.Square(&fe.m, &a.m)
	return fe
}

// Pow2k sets `fe = a ^ (2 * k)` and returns `fe`.  k MUST be non-zero.
func (fe *Element) Pow2k(a *Element, k uint) *Element {
	if k == 0 {
		// This could just set fe = a, but "don't do that".
		panic("internal/field: k out of bounds")
	}

	// XXX/perf: It might be worth inlining this manually at some point.
	fiat.Square(&fe.m, &a.m)
	for i := uint(1); i < k; i++ {
		fiat.Square(&fe.m, &fe.m)
	}

	return fe
}

// Set sets `fe = a` and returns `fe`.
func (fe *Element) Set(a *Element) *Element {
	copy(fe.m[:], a.m[:])
	return fe
}

// SetBytes sets `fe = src`, where `src` is a 32-byte big-endian encoding
// of `fe`, and returns `fe, 0`.  If `src` is not a canonical encoding of
// `fe`, `src` is reduced modulo p, and SetBytes returns `fe, 1`.
func (fe *Element) SetBytes(src *[ElementSize]byte) (*Element, uint64) {
	l := helpers.BytesToSaturated(src)

	didReduce := reduceSaturated(&l, &l)
	fe.uncheckedSetSaturated(&l)

	return fe, didReduce
}

// SetCanonicalBytes sets `fe = src`, where `src` is a 32-byte big-endian
// encoding of `fe`, and returns `fe`.  If `src` is not a canonical
// encoding of `fe`, SetCanonicalBytes returns nil and an error, and the
// receiver is unchanged.
func (fe *Element) SetCanonicalBytes(src *[ElementSize]byte) (*Element, error) {
	l := helpers.BytesToSaturated(src)

	if reduceSaturated(&l, &l) != 0 {
		return nil, errNonCanonicalEncoding
	}
	fe.uncheckedSetSaturated(&l)

	return fe, nil
}

// MustSetCanonicalBytes sets `fe = src`, where `src` MUST be the 32-byte
// big-endian canonical encoding of `fe`, and returns `fe`.  All errors
// will panic.
func (fe *Element) MustSetCanonicalBytes(src *[ElementSize]byte) *Element {
	if _, err := fe.SetCanonicalBytes(src); err != nil {
		panic(err)
	}
	return fe
}

// Bytes returns the canonical big-endian encoding of `fe`.
func (fe *Element) Bytes() []byte {
	// Blah outline blah escape analysis blah.
	var dst [ElementSize]byte
	return fe.getBytes(&dst)
}

func (fe *Element) getBytes(dst *[ElementSize]byte) []byte {
	var nm fiat.NonMontgomeryDomainFieldElement
	fiat.FromMontgomery(&nm, &fe.m)
	return helpers.PutSaturatedToBytes(dst, (*[4]uint64)(&nm))
}

// ConditionalNegate sets `fe = a` iff `ctrl == 0`, `fe = -a` otherwise,
// and returns `fe`.
func (fe *Element) ConditionalNegate(a *Element, ctrl uint64) *Element {
	feNeg := NewElement().Negate(a)

	return fe.ConditionalSelect(a, feNeg, ctrl)
}

// ConditionalSelect sets `fe = a` iff `ctrl == 0`, `fe = b` otherwise,
// and returns `fe`.
func (fe *Element) ConditionalSelect(a, b *Element, ctrl uint64) *Element {
	fiat.Selectznz((*[4]uint64)(&fe.m), fiat.Uint64ToUint1(ctrl), (*[4]uint64)(&a.m), (*[4]uint64)(&b.m))
	return fe
}

// Equal returns 1 iff `fe == a`, 0 otherwise.
func (fe *Element) Equal(a *Element) uint64 {
	return helpers.FiatLimbsAreEqual((*[4]uint64)(&fe.m), (*[4]uint64)(&a.m))
}

// IsZero returns 1 iff `fe == 0`, 0 otherwise.
func (fe *Element) IsZero() uint64 {
	var ctrl uint64
	fiat.Nonzero(&ctrl, (*[4]uint64)(&fe.m))

	return helpers.Uint64IsZero(ctrl)
}

// IsOdd returns 1 iff `fe % 2 == 1`, 0 otherwise.
func (fe *Element) IsOdd() uint64 {
	// XXX/perf: Can't this just be done in the Montgomery domain?
	var nm fiat.NonMontgomeryDomainFieldElement
	fiat.FromMontgomery(&nm, &fe.m)

	return helpers.Uint64IsNonzero(nm[0] & 1)
}

// String returns the big-endian hex representation of `fe`.
func (fe *Element) String() string {
	return hex.EncodeToString(fe.Bytes())
}

func (fe *Element) uncheckedSetSaturated(a *[4]uint64) *Element {
	fiat.ToMontgomery(&fe.m, (*fiat.NonMontgomeryDomainFieldElement)(a))
	return fe
}

// DebugMustRandomizeNonZero randomizes and returns `fe`, or panics.
func (fe *Element) DebugMustRandomizeNonZero() *Element {
	var b [ElementSize]byte
	for {
		if _, err := rand.Read(b[:]); err != nil {
			panic("internal/field: entropy source failure")
		}
		if _, err := fe.SetCanonicalBytes(&b); err != nil {
			continue
		}
		if fe.IsZero() == 1 {
			continue
		}
		return fe
	}
}

// NewElement returns a new zero Element.
func NewElement() *Element {
	return &Element{}
}

// NewElementFrom creates a new Element from another.
func NewElementFrom(other *Element) *Element {
	return NewElement().Set(other)
}

// NewElementFromUint64 creates a new Element from a uint64.
func NewElementFromUint64(l0 uint64) *Element {
	return NewElement().uncheckedSetSaturated(&[4]uint64{l0, 0, 0, 0})
}

// NewElementFromCanonicalBytes creates a new Element from the canonical
// big-endian byte representation.
func NewElementFromCanonicalBytes(src *[ElementSize]byte) (*Element, error) {
	fe, err := NewElement().SetCanonicalBytes(src)
	if err != nil {
		return nil, err
	}

	return fe, nil
}

// NewElementFromCanonicalHex creates a new Element from the canonical hex
// big-endian byte representation.  The hex string MUST be less than or
// equal to `2*ElementSize`.
func NewElementFromCanonicalHex(str string) *Element {
	fe, err := NewElementFromCanonicalBytes(helpers.Must256BitsFromHex(str))
	if err != nil {
		panic(err)
	}

	return fe
}

// BytesAreCanonical returns true iff `src` represents a canonically
// encoded field element in big-endian byte order.
func BytesAreCanonical(src *[ElementSize]byte) bool {
	l := helpers.BytesToSaturated(src)
	return reduceSaturated(&l, &l) == 0
}
