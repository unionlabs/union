package offchain

import (
	apitxsigning "cosmossdk.io/api/cosmos/tx/signing/v1beta1"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

type SignatureData interface {
	isSignatureData()
}

func (m *SingleSignatureData) isSignatureData() {}

type SingleSignatureData struct {
	// SignMode represents the SignMode of the signature
	SignMode apitxsigning.SignMode

	// Signature is the raw signature.
	Signature []byte
}

type OffchainSignature struct {
	// PubKey is the public key to use for verifying the signature
	PubKey cryptotypes.PubKey

	// Data is the actual data of the signature which includes SignMode's and
	// the signatures themselves for either single or multi-signatures.
	Data SignatureData

	// Sequence is the sequence of this account. Only populated in
	// SIGN_MODE_DIRECT.
	Sequence uint64
}
