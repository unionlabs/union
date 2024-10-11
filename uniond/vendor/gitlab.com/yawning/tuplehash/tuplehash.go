// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

// Package tuplehash implements TupleHash and TupleHashXOF from NIST
// SP 800-185.
//
// See: https://csrc.nist.gov/publications/detail/sp/800-185/final
package tuplehash

import (
	"encoding/binary"
	"errors"
	"io"
	"math"
	"math/bits"

	"golang.org/x/crypto/sha3"
)

const (
	rate128 = 168
	rate256 = 136
)

var (
	errNotXOF      = errors.New("tuplehash: Hasher is not a TupleHashXOF")
	errIsXOF       = errors.New("tuplehash: Hasher is a TupleHashXOF")
	errSOverflow   = errors.New("tuplehash: S is oversized")
	errLUnderflow  = errors.New("tuplehash: L must be greater than or equal to 0")
	errSecStrength = errors.New("tuplehash: unsupported security strength")

	constN = []byte("TupleHash")
)

// Hasher is a TupleHash or TupleHashXOF instance.
type Hasher struct {
	inner      sha3.ShakeHash
	rate       int
	outputSize int // L
	isXOF      bool
	didRead    bool
}

// Write writes the byte-encoded tuple to the TupleHash (Hasher.Write is
// thus very different from ShakeHash.Write/Hash.Write).
// It never returns an error.
//
// It panics if input is written to it after output has been read from
// it.
func (h *Hasher) Write(b []byte) (int, error) {
	encodeString(h.inner, b)
	return len(b), nil
}

// Read reads more output from the hash; reading affects the hash's
// state (Hasher.Read is thus very different from Hash.Sum).  It never
// returns an error.
//
// It panics if the Hasher is not a TupleHashXOF.
func (h *Hasher) Read(b []byte) (int, error) {
	if !h.isXOF {
		panic(errNotXOF)
	}

	if !h.didRead {
		h.didRead = true
		rightEncodeByteLen(h.inner, 0)
	}

	return h.inner.Read(b)
}

// Sum appends the current hash to b and returns the resulting slice.
// It does not change the underlying hash state.
//
// It panics if the Hasher is a TupleHashXOF.
func (h *Hasher) Sum(b []byte) []byte {
	if h.isXOF {
		panic(errIsXOF)
	}

	inner := h.inner.Clone()
	rightEncodeByteLen(inner, h.outputSize)

	ret, dst := sliceForAppend(b, h.outputSize)
	_, _ = inner.Read(dst)
	return ret
}

// Size returns the number of bytes Sum will return.
//
// It panics if the Hasher is a TupleHashXOF.
func (h *Hasher) Size() int {
	if h.isXOF {
		panic(errIsXOF)
	}

	return h.outputSize
}

// BlockSize returns the hash's underlying block size.  This value is
// meaningless in the context of TupleHash, as the Hasher expects each
// value to be written in one call to Write.
func (h *Hasher) BlockSize() int {
	return h.rate
}

// Clone returns a copy of the Hasher in its current state.
func (h *Hasher) Clone() sha3.ShakeHash { //nolint:ireturn // x/crypto/sha3.ShakeHash requires this
	return &Hasher{
		inner:      h.inner.Clone(),
		rate:       h.rate,
		outputSize: h.outputSize,
		isXOF:      h.isXOF,
		didRead:    h.didRead,
	}
}

// Reset resets the Hasher to its initial state.
func (h *Hasher) Reset() {
	h.inner.Reset()
	h.didRead = false
}

// NewTupleHash128 constructs a new TupleHash128 instance with the optional
// customization string S, and the specified output size in bytes.
func NewTupleHash128(S []byte, outputSize int) *Hasher { //nolint:gocritic
	return newHasher(S, outputSize, 128)
}

// NewTupleHash256 constructs a new TupleHash256 instance with the optional
// customization string S, and the specified output size in bytes.
func NewTupleHash256(S []byte, outputSize int) *Hasher { //nolint:gocritic
	return newHasher(S, outputSize, 256)
}

// NewTupleHashXOF128 constructs a new TupleHashXOF128 instance with the
// optional customization string S.
func NewTupleHashXOF128(S []byte) *Hasher { //nolint:gocritic
	h := newHasher(S, 0, 128)
	h.isXOF = true
	return h
}

// NewTupleHashXOF256 constructs a new TupleHashXOF256 instance with the
// optional customization string S.
func NewTupleHashXOF256(S []byte) *Hasher { //nolint:gocritic
	h := newHasher(S, 0, 256)
	h.isXOF = true
	return h
}

func newHasher(S []byte, L int, secStrength int) *Hasher { //nolint:gocritic
	// x/crypto/sha3 will silently misbehave if this invariant is
	// violated.  Realistically this is unlikely in my lifetime,
	// and not currently possible given hardware/software limitations.
	if hi, _ := lenToBits(len(S)); hi != 0 {
		panic(errSOverflow)
	}

	if L < 0 {
		panic(errLUnderflow)
	}

	h := &Hasher{
		outputSize: L,
	}
	switch secStrength {
	case 128:
		h.inner = sha3.NewCShake128(constN, S)
		h.rate = rate128
	case 256:
		h.inner = sha3.NewCShake256(constN, S)
		h.rate = rate256
	default:
		panic(errSecStrength)
	}
	return h
}

// right_encode and left_encode are defined to support 0 <= x < 2^2040
// however, the largest value we will ever need to encode is
// `math.MaxInt * 8`.
//
// This is unfortunate as the extreme upper edge is larger than
// [math.MaxUint64].  While such values are impractical at present,
// they are possible, ie: https://arxiv.org/pdf/quant-ph/9908043.pdf
//
// Thus we support 0 <= x < 2^128.

func lenToBits(x int) (uint64, uint64) {
	hi, lo := bits.Mul64(uint64(x), 8)
	return hi, lo
}

func rightEncodeByteLen(w io.Writer, l int) {
	hi, lo := lenToBits(l)
	rightEncode(w, hi, lo)
}

func encodeString(w io.Writer, s []byte) {
	// The spec is bit-oriented.
	hi, lo := lenToBits(len(s))

	// 1. Return left_encode(len(S)) || S.
	leftEncode(w, hi, lo)
	_, _ = w.Write(s)
}

func leftEncode(w io.Writer, hi, lo uint64) {
	leftRightEncode(w, hi, lo, true)
}

func rightEncode(w io.Writer, hi, lo uint64) {
	leftRightEncode(w, hi, lo, false)
}

func leftRightEncode(w io.Writer, hi, lo uint64, isLeft bool) {
	// Fast-path: Values under 2^16 are likely to be common.
	if lo <= math.MaxUint16 && hi == 0 {
		var buf [1 + 2 + 1]byte

		binary.BigEndian.PutUint16(buf[1:], uint16(lo))

		n := byte(1)
		if lo > math.MaxUint8 {
			n = 2
		}

		var b []byte
		switch isLeft {
		case true:
			buf[2-n] = n
			b = buf[2-n : 3]
		case false:
			buf[3] = n
			b = buf[3-n:]
		}
		_, _ = w.Write(b)
		return
	}

	// Generic implementation.
	const (
		hiOffset    = 1
		loOffset    = hiOffset + 8
		rightOffset = loOffset + 8

		bufLen = rightOffset + 1
	)

	var buf [bufLen]byte // prefix + largest uint + postfix

	// 1. Encode as a big-endian integer.
	binary.BigEndian.PutUint64(buf[hiOffset:], hi)
	binary.BigEndian.PutUint64(buf[loOffset:], lo)

	// 2. Strip leading `0x00` bytes.
	var off int
	for off = hiOffset; off < rightOffset-1; off++ { // Note: Minimum size is 1.
		if buf[off] != 0 {
			break
		}
	}
	n := byte(rightOffset - off)

	// 3. Prefix (left_encode) or postfix (right_encode) the length in bytes.
	var b []byte
	switch isLeft {
	case true:
		buf[off-1] = n // n | x
		b = buf[off-1 : rightOffset]
	case false:
		buf[rightOffset] = n // x | n
		b = buf[off:]
	}

	_, _ = w.Write(b)
}

// Stolen from the stdlib.
func sliceForAppend(in []byte, n int) (head, tail []byte) { //nolint:nonamedreturns
	if total := len(in) + n; cap(in) >= total {
		head = in[:total]
	} else {
		head = make([]byte, total)
		copy(head, in)
	}
	tail = head[len(in):]
	return
}

func init() { //nolint:gochecknoinits
	// This is a stupid invariant to have to check, but someone may
	// port Go to an exotic architecture.
	if math.MaxInt > math.MaxInt64 {
		panic("tuplehash: unsupported architecture")
	}
}
