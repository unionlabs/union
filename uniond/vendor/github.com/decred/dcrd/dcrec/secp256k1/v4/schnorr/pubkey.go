// Copyright (c) 2013-2014 The btcsuite developers
// Copyright (c) 2015-2020 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package schnorr

import (
	"fmt"

	"github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// These constants define the lengths of serialized public keys.
const (
	PubKeyBytesLen = 33
)

const (
	// pubkeyCompressed is the header byte for a compressed secp256k1 pubkey.
	pubkeyCompressed byte = 0x2 // y_bit + x coord
)

// ParsePubKey parses a public key for a koblitz curve from a bytestring into a
// ecdsa.Publickey, verifying that it is valid. It supports compressed signature
// formats only.
func ParsePubKey(pubKeyStr []byte) (key *secp256k1.PublicKey, err error) {
	if pubKeyStr == nil {
		err = fmt.Errorf("nil pubkey byte string")
		return
	}
	if len(pubKeyStr) != PubKeyBytesLen {
		err = fmt.Errorf("bad pubkey byte string size (want %v, have %v)",
			PubKeyBytesLen, len(pubKeyStr))
		return
	}
	format := pubKeyStr[0]
	format &= ^byte(0x1)
	if format != pubkeyCompressed {
		err = fmt.Errorf("wrong pubkey type (not compressed)")
		return
	}

	return secp256k1.ParsePubKey(pubKeyStr)
}
