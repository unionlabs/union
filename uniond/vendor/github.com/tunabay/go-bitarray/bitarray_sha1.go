// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	//nolint:gosec // intentionally provided option
	_ "crypto/sha1" // block()
	"encoding/binary"
	_ "unsafe" // linkname
)

// SHA1 returns the SHA-1 checksum of the bit array. It treats the bit array as
// a bit-oriented message to compute the checksum as defined in RFC 3174.
func (ba *BitArray) SHA1() [20]byte {
	nBits := ba.Len()
	buf := NewBuffer((nBits + 1 + 64 + 511) &^ 511)
	buf.PutBitArrayAt(0, ba)
	buf.PutBitAt(nBits, 1)
	binary.BigEndian.PutUint64(buf.b[len(buf.b)-8:], uint64(nBits))

	d := &sha1Digest{}
	sha1Reset(d)
	sha1Block(d, buf.b)

	var sha1 [20]byte
	for i := 0; i < 5; i++ {
		binary.BigEndian.PutUint32(sha1[i<<2:], d.h[i])
	}

	return sha1
}

// crypto/sha1.digest
// TODO: if possible, use crypto/sha1.digest directly
type sha1Digest struct {
	h   [5]uint32
	x   [64]byte //nolint:structcheck,unused // for Reset()
	nx  int      //nolint:structcheck,unused // for Reset()
	len uint64   //nolint:structcheck,unused // for Reset()
}

//go:linkname sha1Block crypto/sha1.block
func sha1Block(*sha1Digest, []byte)

//go:linkname sha1Reset crypto/sha1.(*digest).Reset
func sha1Reset(*sha1Digest)
