// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package bitcoin

import "encoding/asn1"

// IsValidSignatureEncodingBIP0066 returns true iff `data` is encoded
// per BIP-0066, including the trailing `sighash` byte.
//
// See: https://github.com/bitcoin/bips/blob/master/bip-0066.mediawiki
func IsValidSignatureEncodingBIP0066(data []byte) bool {
	// Note:
	// - Most of the checks are probably redundant to what
	//   parseASN1Signature does, but if there's a standard, follow it.
	// - Annoyingly enough while `[sighash]` isn't part of the signature
	//   proper, it still is included in all of the length accounting.

	// From BIP-0066:
	//   Format: 0x30 [total-length] 0x02 [R-length] [R] 0x02 [S-length] [S] [sighash]
	//   * total-length: 1-byte length descriptor of everything that follows,
	//     excluding the sighash byte.
	//   * R-length: 1-byte length descriptor of the R value that follows.
	//   * R: arbitrary-length big-endian encoded R value. It must use the shortest
	//     possible encoding for a positive integers (which means no null bytes at
	//     the start, except a single one when the next byte has its highest bit set).
	//   * S-length: 1-byte length descriptor of the S value that follows.
	//   * S: arbitrary-length big-endian encoded S value. The same rules apply.
	//   * sighash: 1-byte value indicating what data is hashed (not part of the DER
	//     signature)

	const asn1IsCompound = 0x20

	lenSig := len(data)

	// Minimum and maximum size constraints.
	switch {
	case lenSig < 9:
		return false
	case lenSig > 73:
		return false
	}

	// A signature is of type 0x30 (compound).
	if data[0] != (asn1.TagSequence | asn1IsCompound) {
		return false
	}

	// Make sure the length covers the entire signature.
	if int(data[1]) != lenSig-3 {
		return false
	}

	// Extract the length of the R element.
	lenR := int(data[3])

	// Make sure the length of the S element is still inside the signature.
	if 5+lenR >= lenSig {
		return false
	}

	// Extract the length of the S element.
	lenS := int(data[5+lenR])

	// Verify that the length of the signature matches the sum of the length
	// of the elements.
	if lenR+lenS+7 != lenSig {
		return false
	}

	// Check whether the R element is an integer.
	if data[2] != asn1.TagInteger {
		return false
	}

	// Zero-length integers are not allowed for R.
	if lenR == 0 {
		return false
	}

	// Negative numbers are not allowed for R.
	if data[4]&0x80 != 0x00 {
		return false
	}

	// Null bytes at the start of R are not allowed, unless R would
	// otherwise be interpreted as a negative number.
	if lenR > 1 && (data[4] == 0x00) && (data[5]&0x80 == 0x00) {
		return false
	}

	// Check whether the S element is an integer.
	if data[lenR+4] != asn1.TagInteger {
		return false
	}

	// Zero-length integers are not allowed for S.
	if lenS == 0 {
		return false
	}

	// Negative numbers are not allowed for S.
	if data[lenR+6]&0x80 != 0x00 {
		return false
	}

	// Null bytes at the start of S are not allowed, unless S would otherwise be
	// interpreted as a negative number.
	if lenS > 1 && (data[lenR+6] == 0x00) && (data[lenR+7]&0x80 == 0x00) {
		return false
	}

	return true
}
