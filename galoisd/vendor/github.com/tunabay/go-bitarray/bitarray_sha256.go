// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	_ "crypto/sha256" // block()
	"encoding/binary"
	_ "unsafe" // linkname
)

// SHA256 returns the SHA-256 checksum of the bit array. It treats the bit array
// as a bit-oriented message to compute the checksum as defined in FIPS 180-4.
func (ba *BitArray) SHA256() (d256 [32]byte) {
	return ba.sha256CheckSum(false)
}

// SHA224 returns the SHA-224 checksum of the bit array. It treats the bit array
// as a bit-oriented message to compute the checksum as defined in FIPS 180-4.
func (ba *BitArray) SHA224() (d224 [28]byte) {
	d256 := ba.sha256CheckSum(true)
	copy(d224[:], d256[:])
	return
}

func (ba *BitArray) sha256CheckSum(is224 bool) [32]byte {
	nBits := ba.Len()
	buf := NewBuffer((nBits + 1 + 64 + 511) &^ 511)
	buf.PutBitArrayAt(0, ba)
	buf.PutBitAt(nBits, 1)
	binary.BigEndian.PutUint64(buf.b[len(buf.b)-8:], uint64(nBits))

	d := &sha256Digest{is224: is224}
	sha256Reset(d)
	sha256Block(d, buf.b)

	n := 8
	if is224 {
		n = 7
	}
	var d256 [32]byte
	for i := 0; i < n; i++ {
		binary.BigEndian.PutUint32(d256[i<<2:], d.h[i])
	}

	return d256
}

// crypto/sha256.digest
// TODO: if possible, use crypto/sha256.digest directly
type sha256Digest struct {
	h     [8]uint32
	x     [64]byte //nolint:structcheck,unused // for Reset()
	nx    int      //nolint:structcheck,unused // for Reset()
	len   uint64   //nolint:structcheck,unused // for Reset()
	is224 bool
}

//go:linkname sha256Block crypto/sha256.block
func sha256Block(*sha256Digest, []byte)

//go:linkname sha256Reset crypto/sha256.(*digest).Reset
func sha256Reset(*sha256Digest)
