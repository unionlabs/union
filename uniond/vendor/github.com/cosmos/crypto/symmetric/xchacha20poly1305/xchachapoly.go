// Package xchacha20poly1305 creates an AEAD using hchacha, chacha, and poly1305
// This allows for randomized nonces to be used in conjunction with chacha.
package xchacha20poly1305

import (
	"crypto/cipher"
	"encoding/binary"
	"errors"

	"golang.org/x/crypto/chacha20poly1305"
)

// Implements crypto.AEAD.
type xchacha20poly1305 struct {
	key [KeySize]byte
}

const (
	// KeySize is the size of the key used by this AEAD, in bytes.
	KeySize = 32
	// NonceSize is the size of the nonce used with this AEAD, in bytes.
	NonceSize = 24
	// TagSize is the size added from poly1305.
	TagSize = 16
	// MaxPlaintextSize is the max size that can be passed into a single call of Seal.
	MaxPlaintextSize = (1 << 38) - 64
	// MaxCiphertextSize is the max size that can be passed into a single call of Open,
	// this differs from plaintext size due to the tag.
	MaxCiphertextSize = (1 << 38) - 48

	// sigma are constants used in xchacha.
	// Unrolled from a slice so that they can be inlined, as slices can't be constants.
	sigma0 = uint32(0x61707865)
	sigma1 = uint32(0x3320646e)
	sigma2 = uint32(0x79622d32)
	sigma3 = uint32(0x6b206574)
)

var (
	ErrInvalidKeyLen        = errors.New("xchacha20poly1305: bad key length")
	ErrInvalidNonceLen      = errors.New("xchacha20poly1305: bad nonce length")
	ErrInvalidCipherTextLen = errors.New("xchacha20poly1305: ciphertext too large")
)

// New returns a new xchachapoly1305 AEAD.
func New(key []byte) (cipher.AEAD, error) {
	if len(key) != KeySize {
		return nil, ErrInvalidKeyLen
	}
	ret := new(xchacha20poly1305)
	copy(ret.key[:], key)
	return ret, nil
}

func (*xchacha20poly1305) NonceSize() int {
	return NonceSize
}

func (*xchacha20poly1305) Overhead() int {
	return TagSize
}

func (c *xchacha20poly1305) Seal(dst, nonce, plaintext, additionalData []byte) []byte {
	if len(nonce) != NonceSize {
		panic("xchacha20poly1305: bad nonce length passed to Seal")
	}

	if uint64(len(plaintext)) > MaxPlaintextSize {
		panic("xchacha20poly1305: plaintext too large")
	}

	var subKey [KeySize]byte
	var hNonce [16]byte
	var subNonce [chacha20poly1305.NonceSize]byte
	copy(hNonce[:], nonce[:16])

	HChaCha20(&subKey, &hNonce, &c.key)

	// This can't error because we always provide a correctly sized key
	chacha20poly1305, _ := chacha20poly1305.New(subKey[:])

	copy(subNonce[4:], nonce[16:])

	return chacha20poly1305.Seal(dst, subNonce[:], plaintext, additionalData)
}

func (c *xchacha20poly1305) Open(dst, nonce, ciphertext, additionalData []byte) ([]byte, error) {
	if len(nonce) != NonceSize {
		return nil, ErrInvalidNonceLen
	}
	if uint64(len(ciphertext)) > MaxCiphertextSize {
		return nil, ErrInvalidCipherTextLen
	}
	var subKey [KeySize]byte
	var hNonce [16]byte
	var subNonce [chacha20poly1305.NonceSize]byte
	copy(hNonce[:], nonce[:16])

	HChaCha20(&subKey, &hNonce, &c.key)

	// This can't error because we always provide a correctly sized key
	chacha20poly1305, _ := chacha20poly1305.New(subKey[:])

	copy(subNonce[4:], nonce[16:])

	return chacha20poly1305.Open(dst, subNonce[:], ciphertext, additionalData)
}

// HChaCha exported from
// https://github.com/aead/chacha20/blob/8b13a72661dae6e9e5dea04f344f0dc95ea29547/chacha/chacha_generic.go#L194
// TODO: Add support for the different assembly instructions used there.

// The MIT License (MIT)

// Copyright (c) 2016 Andreas Auernhammer

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// HChaCha20 generates 32 pseudo-random bytes from a 128 bit nonce and a 256 bit secret key.
// It can be used as a key-derivation-function (KDF).
func HChaCha20(out *[32]byte, nonce *[16]byte, key *[32]byte) { hChaCha20Generic(out, nonce, key) }

func hChaCha20Generic(out *[32]byte, nonce *[16]byte, key *[32]byte) {
	v00 := sigma0
	v01 := sigma1
	v02 := sigma2
	v03 := sigma3
	v04 := binary.LittleEndian.Uint32(key[0:])
	v05 := binary.LittleEndian.Uint32(key[4:])
	v06 := binary.LittleEndian.Uint32(key[8:])
	v07 := binary.LittleEndian.Uint32(key[12:])
	v08 := binary.LittleEndian.Uint32(key[16:])
	v09 := binary.LittleEndian.Uint32(key[20:])
	v10 := binary.LittleEndian.Uint32(key[24:])
	v11 := binary.LittleEndian.Uint32(key[28:])
	v12 := binary.LittleEndian.Uint32(nonce[0:])
	v13 := binary.LittleEndian.Uint32(nonce[4:])
	v14 := binary.LittleEndian.Uint32(nonce[8:])
	v15 := binary.LittleEndian.Uint32(nonce[12:])

	for i := 0; i < 20; i += 2 {
		v00 += v04
		v12 ^= v00
		v12 = (v12 << 16) | (v12 >> 16)
		v08 += v12
		v04 ^= v08
		v04 = (v04 << 12) | (v04 >> 20)
		v00 += v04
		v12 ^= v00
		v12 = (v12 << 8) | (v12 >> 24)
		v08 += v12
		v04 ^= v08
		v04 = (v04 << 7) | (v04 >> 25)
		v01 += v05
		v13 ^= v01
		v13 = (v13 << 16) | (v13 >> 16)
		v09 += v13
		v05 ^= v09
		v05 = (v05 << 12) | (v05 >> 20)
		v01 += v05
		v13 ^= v01
		v13 = (v13 << 8) | (v13 >> 24)
		v09 += v13
		v05 ^= v09
		v05 = (v05 << 7) | (v05 >> 25)
		v02 += v06
		v14 ^= v02
		v14 = (v14 << 16) | (v14 >> 16)
		v10 += v14
		v06 ^= v10
		v06 = (v06 << 12) | (v06 >> 20)
		v02 += v06
		v14 ^= v02
		v14 = (v14 << 8) | (v14 >> 24)
		v10 += v14
		v06 ^= v10
		v06 = (v06 << 7) | (v06 >> 25)
		v03 += v07
		v15 ^= v03
		v15 = (v15 << 16) | (v15 >> 16)
		v11 += v15
		v07 ^= v11
		v07 = (v07 << 12) | (v07 >> 20)
		v03 += v07
		v15 ^= v03
		v15 = (v15 << 8) | (v15 >> 24)
		v11 += v15
		v07 ^= v11
		v07 = (v07 << 7) | (v07 >> 25)
		v00 += v05
		v15 ^= v00
		v15 = (v15 << 16) | (v15 >> 16)
		v10 += v15
		v05 ^= v10
		v05 = (v05 << 12) | (v05 >> 20)
		v00 += v05
		v15 ^= v00
		v15 = (v15 << 8) | (v15 >> 24)
		v10 += v15
		v05 ^= v10
		v05 = (v05 << 7) | (v05 >> 25)
		v01 += v06
		v12 ^= v01
		v12 = (v12 << 16) | (v12 >> 16)
		v11 += v12
		v06 ^= v11
		v06 = (v06 << 12) | (v06 >> 20)
		v01 += v06
		v12 ^= v01
		v12 = (v12 << 8) | (v12 >> 24)
		v11 += v12
		v06 ^= v11
		v06 = (v06 << 7) | (v06 >> 25)
		v02 += v07
		v13 ^= v02
		v13 = (v13 << 16) | (v13 >> 16)
		v08 += v13
		v07 ^= v08
		v07 = (v07 << 12) | (v07 >> 20)
		v02 += v07
		v13 ^= v02
		v13 = (v13 << 8) | (v13 >> 24)
		v08 += v13
		v07 ^= v08
		v07 = (v07 << 7) | (v07 >> 25)
		v03 += v04
		v14 ^= v03
		v14 = (v14 << 16) | (v14 >> 16)
		v09 += v14
		v04 ^= v09
		v04 = (v04 << 12) | (v04 >> 20)
		v03 += v04
		v14 ^= v03
		v14 = (v14 << 8) | (v14 >> 24)
		v09 += v14
		v04 ^= v09
		v04 = (v04 << 7) | (v04 >> 25)
	}

	binary.LittleEndian.PutUint32(out[0:], v00)
	binary.LittleEndian.PutUint32(out[4:], v01)
	binary.LittleEndian.PutUint32(out[8:], v02)
	binary.LittleEndian.PutUint32(out[12:], v03)
	binary.LittleEndian.PutUint32(out[16:], v12)
	binary.LittleEndian.PutUint32(out[20:], v13)
	binary.LittleEndian.PutUint32(out[24:], v14)
	binary.LittleEndian.PutUint32(out[28:], v15)
}
