// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"crypto"
	_ "crypto/sha512" // block()
	"encoding/binary"
	_ "unsafe" // linkname
)

// SHA512 returns the SHA-512 checksum of the bit array. It treats the bit array
// as a bit-oriented message to compute the checksum as defined in FIPS 180-4.
func (ba *BitArray) SHA512() (d512 [64]byte) {
	return ba.sha512CheckSum(crypto.SHA512)
}

// SHA384 returns the SHA-384 checksum of the bit array. It treats the bit array
// as a bit-oriented message to compute the checksum as defined in FIPS 180-4.
func (ba *BitArray) SHA384() (d384 [48]byte) {
	d512 := ba.sha512CheckSum(crypto.SHA384)
	copy(d384[:], d512[:])
	return
}

// SHA512_256 returns the SHA-512/256 checksum of the bit array. It treats the
// bit array as a bit-oriented message to compute the checksum as defined in
// FIPS 180-4.
//nolint:revive,stylecheck // not an ALL_CAPS
func (ba *BitArray) SHA512_256() (d256 [32]byte) {
	d512 := ba.sha512CheckSum(crypto.SHA512_256)
	copy(d256[:], d512[:])
	return
}

// SHA512_224 returns the SHA-512/224 checksum of the bit array. It treats the
// bit array as a bit-oriented message to compute the digest as defined in FIPS
// 180-4.
//nolint:revive,stylecheck // not an ALL_CAPS
func (ba *BitArray) SHA512_224() (d224 [28]byte) {
	d512 := ba.sha512CheckSum(crypto.SHA512_224)
	copy(d224[:], d512[:])
	return
}

func (ba *BitArray) sha512CheckSum(hash crypto.Hash) [64]byte {
	nBits := ba.Len()
	buf := NewBuffer((nBits + 1 + 128 + 1023) &^ 1023)
	buf.PutBitArrayAt(0, ba)
	buf.PutBitAt(nBits, 1)
	// binary.BigEndian.PutUint64(buf.b[len(buf.b)-16:], uint64(hi)) // for nBits > max-uint64
	binary.BigEndian.PutUint64(buf.b[len(buf.b)-8:], uint64(nBits))

	d := &sha512Digest{function: hash}
	sha512Reset(d)
	sha512Block(d, buf.b)

	n := 8
	if hash == crypto.SHA384 {
		n = 6
	}
	var d512 [64]byte
	for i := 0; i < n; i++ {
		binary.BigEndian.PutUint64(d512[i<<3:], d.h[i])
	}

	return d512
}

// crypto/sha512.digest
// TODO: if possible, use crypto/sha512.digest directly
type sha512Digest struct {
	h        [8]uint64
	x        [128]byte //nolint:structcheck,unused // for Reset()
	nx       int       //nolint:structcheck,unused // for Reset()
	len      uint64    //nolint:structcheck,unused // for Reset()
	function crypto.Hash
}

//go:linkname sha512Block crypto/sha512.block
func sha512Block(*sha512Digest, []byte)

//go:linkname sha512Reset crypto/sha512.(*digest).Reset
func sha512Reset(*sha512Digest)
