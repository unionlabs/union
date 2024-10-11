// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secec

import (
	"bytes"
	"crypto/hmac"
	"crypto/sha256"
	"io"

	"gitlab.com/yawning/secp256k1-voi"
)

var readerRFC6979SHA256 = sentinelReaderRFC6979{}

type sentinelReaderRFC6979 struct{}

func (rd sentinelReaderRFC6979) Read(_ []byte) (int, error) {
	// This is just a placeholder so we know to instantiate
	// a drbgRFC6979.
	panic("secp256k1/secec: RFC6979SHA256().Read called")
}

// RFC6979SHA256 returns an [io.Reader] that will make the `Sign`
// and `SignRaw` ECDSA routines return deterministic signatures
// with the nonce generation algorithm as specified in RFC 6979,
// using SHA-256 as the hash function, when strict output
// compatibility with other implementations is required.
//
// This option is not otherwise required or recommended, and providing
// an entropy source when available is likely better.
//
// WARNING: This returns a non-functional placeholder reader that
// will panic if actually used.  The returned reader is incompatible
// with non-ECDSA use cases.
func RFC6979SHA256() io.Reader {
	return readerRFC6979SHA256
}

type drbgRFC6979 struct {
	v []byte
	k []byte

	needUpdate bool
}

func (drbg *drbgRFC6979) Read(b []byte) (int, error) {
	if len(b) != secp256k1.ScalarSize {
		panic("secp256k1/secec: invalid RFC6979 read length")
	}

	if drbg.needUpdate {
		// Step 3 from the previous Read call is delayed till the
		// next read, as it is extremely unlikely that the first k
		// to get sampled is unsuitable.
		drbg.updateK()
		drbg.updateV()
	}

	// h. Apply the following algorithm until a proper value is found for k:

	// 1.  Set T to the empty sequence.  The length of T (in bits) is
	// denoted tlen; thus, at that point, tlen = 0.

	// 2.  While tlen < qlen, do the following:
	// V = HMAC_K(V)
	// T = T || V

	drbg.updateV()
	copy(b, drbg.v) // Return T instead (Note: len(v) = qlen)

	// 3.  Compute:
	// k = bits2int(T)
	//
	// If that value of k is within the [1,q-1] range, and is
	// suitable for DSA or ECDSA (i.e., it results in an r value
	// that is not 0; see Section 3.4), then the generation of k is
	// finished.  The obtained value of k is used in DSA or ECDSA.
	// Otherwise, compute:
	//
	// K = HMAC_K(V || 0x00)
	// V = HMAC_K(V)
	//
	// and loop (try to generate a new T, and so on).

	drbg.needUpdate = true

	return len(b), nil
}

func (drbg *drbgRFC6979) updateV() {
	// V = HMAC_K(V)
	m := hmac.New(sha256.New, drbg.k)
	_, _ = m.Write(drbg.v)
	drbg.v = m.Sum(drbg.v[:0])
}

func (drbg *drbgRFC6979) updateK() {
	// K = HMAC_K(V || 0x00)
	m := hmac.New(sha256.New, drbg.k)
	_, _ = m.Write(drbg.v)
	_, _ = m.Write([]byte{0x00})
	drbg.k = m.Sum(drbg.k[:0])
}

func newDrbgRFC6979(x, e *secp256k1.Scalar) io.Reader {
	// 3.2.  Generation of k

	const kvLen = 32 // 8 * ceil(hlen/8)

	// a. Process m through the hash function H, yielding:
	// h1 = H(m) (h1 is a sequence of hlen bits).

	// b. Set: V = 0x01 0x01 0x01 ... 0x01
	// c. Set: K = 0x00 0x00 0x00 ... 0x00
	drbg := &drbgRFC6979{
		v: bytes.Repeat([]byte{0x01}, kvLen),
		k: make([]byte, kvLen),
	}

	i2oB := x.Bytes()
	b2oH1 := e.Bytes()

	// d. Set: K = HMAC_K(V || 0x00 || int2octets(x) || bits2octets(h1))
	// e. Set: V = HMAC_K(V)
	// f. Set: K = HMAC_K(V || 0x01 || int2octets(x) || bits2octets(h1))
	// g. Set: V = HMAC_K(V)

	initUpdateK := func(internalOctet byte) {
		m := hmac.New(sha256.New, drbg.k)
		_, _ = m.Write(drbg.v)
		_, _ = m.Write([]byte{internalOctet})
		_, _ = m.Write(i2oB)
		_, _ = m.Write(b2oH1)
		drbg.k = m.Sum(drbg.k[:0])
	}
	initUpdateK(0x00) // Step d
	drbg.updateV()    // Step e
	initUpdateK(0x01) // Step f
	drbg.updateV()    // Step g

	return drbg
}
