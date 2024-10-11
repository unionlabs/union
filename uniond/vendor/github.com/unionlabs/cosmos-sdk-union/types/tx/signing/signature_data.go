package signing

import (
	"github.com/cosmos/cosmos-sdk/crypto/types"
)

// SignatureData represents either a *SingleSignatureData or *MultiSignatureData.
// It is a convenience type that is easier to use in business logic than the encoded
// protobuf ModeInfo's and raw signatures.
type SignatureData interface {
	isSignatureData()
}

// SingleSignatureData represents the signature and SignMode of a single (non-multisig) signer
type SingleSignatureData struct {
	// SignMode represents the SignMode of the signature
	SignMode SignMode

	// Signature is the raw signature.
	Signature []byte
}

// MultiSignatureData represents the nested SignatureData of a multisig signature
type MultiSignatureData struct {
	// BitArray is a compact way of indicating which signers from the multisig key
	// have signed
	BitArray *types.CompactBitArray

	// Signatures is the nested SignatureData's for each signer
	Signatures []SignatureData
}

var _, _ SignatureData = &SingleSignatureData{}, &MultiSignatureData{}

func (m *SingleSignatureData) isSignatureData() {}
func (m *MultiSignatureData) isSignatureData()  {}
