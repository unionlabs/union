//go:build !libsecp256k1_sdk
// +build !libsecp256k1_sdk

package secp256k1

import (
	"errors"

	"github.com/cometbft/cometbft/crypto"
	secp256k1 "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa"
)

// Sign creates an ECDSA signature on curve Secp256k1, using SHA256 on the msg.
// The returned signature will be of the form R || S (in lower-S form).
func (privKey *PrivKey) Sign(msg []byte) ([]byte, error) {
	priv := secp256k1.PrivKeyFromBytes(privKey.Key)
	sig := ecdsa.SignCompact(priv, crypto.Sha256(msg), false)

	// remove the first byte which is compactSigRecoveryCode
	return sig[1:], nil
}

// VerifyBytes verifies a signature of the form R || S.
// It rejects signatures which are not in lower-S form.
func (pubKey *PubKey) VerifySignature(msg, sigStr []byte) bool {
	if len(sigStr) != 64 {
		return false
	}
	pub, err := secp256k1.ParsePubKey(pubKey.Key)
	if err != nil {
		return false
	}
	// parse the signature, will return error if it is not in lower-S form
	signature, err := signatureFromBytes(sigStr)
	if err != nil {
		return false
	}
	return signature.Verify(crypto.Sha256(msg), pub)
}

// Read Signature struct from R || S. Caller needs to ensure
// that len(sigStr) == 64.
// Rejects malleable signatures (if S value if it is over half order).
func signatureFromBytes(sigStr []byte) (*ecdsa.Signature, error) {
	var r secp256k1.ModNScalar
	r.SetByteSlice(sigStr[:32])
	var s secp256k1.ModNScalar
	s.SetByteSlice(sigStr[32:64])
	if s.IsOverHalfOrder() {
		return nil, errors.New("signature is not in lower-S form")
	}

	return ecdsa.NewSignature(&r, &s), nil
}
