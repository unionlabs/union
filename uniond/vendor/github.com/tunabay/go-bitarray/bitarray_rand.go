// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	crand "crypto/rand"
	"fmt"
	"math/rand"
)

// Rand generates a random bit array with nBits length.
//
// This is based on the crypto/rand package and cryptographically secure.
func Rand(nBits int) (*BitArray, error) {
	if nBits == 0 {
		return zeroBitArray, nil
	}

	buf := allocByteSlice((nBits + 7) >> 3)
	if _, err := crand.Read(buf); err != nil {
		return &BitArray{nBits: nBits}, fmt.Errorf("failed to read random: %w", err)
	}
	if npad := (8 - nBits&7) & 7; npad != 0 {
		buf[len(buf)-1] &= byte(0xff) << npad
	}

	return &BitArray{b: buf, nBits: nBits}, nil
}

// PseudoRand generates a random bit array with nBits length. If gen is non-nil,
// it will be used as the source instead of the default source. In this case, it
// is not safe for concurrent use by multiple goroutines. Only the default
// source is safe for concurrent use.
//
// This is based on math/rand package, and not cryptographically secure. Use
// Rand for security-sensitive data.
//
// Note that even generating less than 8 bits consumes 1 byte from the source.
// Therefore, the results are different between the case where 4 bits are
// generated twice and concatenated, and the case where 8 bits are generated at
// once, even in the same source state.
func PseudoRand(nBits int, gen *rand.Rand) *BitArray {
	if nBits == 0 {
		return zeroBitArray
	}

	buf := allocByteSlice((nBits + 7) >> 3)
	if gen == nil {
		_, _ = rand.Read(buf) //nolint:gosec // intentionally provided option
	} else {
		_, _ = gen.Read(buf)
	}
	if npad := (8 - nBits&7) & 7; npad != 0 {
		buf[len(buf)-1] &= byte(0xff) << npad
	}

	return &BitArray{b: buf, nBits: nBits}
}
