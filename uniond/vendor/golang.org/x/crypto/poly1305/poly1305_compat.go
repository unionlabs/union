// Copyright 2012 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package poly1305 implements Poly1305 one-time message authentication code as
// specified in https://cr.yp.to/mac/poly1305-20050329.pdf.
//
// Poly1305 is a fast, one-time authentication function. It is infeasible for an
// attacker to generate an authenticator for a message without the key. However, a
// key must only be used for a single message. Authenticating two different
// messages with the same key allows an attacker to forge authenticators for other
// messages with the same key.
//
// Poly1305 was originally coupled with AES in order to make Poly1305-AES. AES was
// used with a fixed key in order to generate one-time keys from an nonce.
// However, in this package AES isn't used and the one-time key is specified
// directly.
//
// Deprecated: Poly1305 as implemented by this package is a cryptographic
// building block that is not safe for general purpose use.
// For encryption, use the full ChaCha20-Poly1305 construction implemented by
// golang.org/x/crypto/chacha20poly1305. For authentication, use a general
// purpose MAC such as HMAC implemented by crypto/hmac.
package poly1305

import "golang.org/x/crypto/internal/poly1305"

// TagSize is the size, in bytes, of a poly1305 authenticator.
//
// For use with golang.org/x/crypto/chacha20poly1305, chacha20poly1305.Overhead
// can be used instead.
const TagSize = 16

// Sum generates an authenticator for msg using a one-time key and puts the
// 16-byte result into out. Authenticating two different messages with the same
// key allows an attacker to forge messages at will.
func Sum(out *[16]byte, m []byte, key *[32]byte) {
	poly1305.Sum(out, m, key)
}

// Verify returns true if mac is a valid authenticator for m with the given key.
func Verify(mac *[16]byte, m []byte, key *[32]byte) bool {
	return poly1305.Verify(mac, m, key)
}

// New returns a new MAC computing an authentication
// tag of all data written to it with the given key.
// This allows writing the message progressively instead
// of passing it as a single slice. Common users should use
// the Sum function instead.
//
// The key must be unique for each message, as authenticating
// two different messages with the same key allows an attacker
// to forge messages at will.
func New(key *[32]byte) *MAC {
	return &MAC{mac: poly1305.New(key)}
}

// MAC is an io.Writer computing an authentication tag
// of the data written to it.
//
// MAC cannot be used like common hash.Hash implementations,
// because using a poly1305 key twice breaks its security.
// Therefore writing data to a running MAC after calling
// Sum or Verify causes it to panic.
type MAC struct {
	mac *poly1305.MAC
}

// Size returns the number of bytes Sum will return.
func (h *MAC) Size() int { return TagSize }

// Write adds more data to the running message authentication code.
// It never returns an error.
//
// It must not be called after the first call of Sum or Verify.
func (h *MAC) Write(p []byte) (n int, err error) {
	return h.mac.Write(p)
}

// Sum computes the authenticator of all data written to the
// message authentication code.
func (h *MAC) Sum(b []byte) []byte {
	return h.mac.Sum(b)
}

// Verify returns whether the authenticator of all data written to
// the message authentication code matches the expected value.
func (h *MAC) Verify(expected []byte) bool {
	return h.mac.Verify(expected)
}
