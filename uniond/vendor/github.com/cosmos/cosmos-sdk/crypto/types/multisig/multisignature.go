package multisig

import (
	"errors"
	"fmt"
	"strings"

	"github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// AminoMultisignature is used to represent amino multi-signatures for StdTx's.
// It is assumed that all signatures were made with SIGN_MODE_LEGACY_AMINO_JSON.
// Sigs is a list of signatures, sorted by corresponding index.
type AminoMultisignature struct {
	BitArray *types.CompactBitArray
	Sigs     [][]byte
}

// NewMultisig returns a new MultiSignatureData
func NewMultisig(n int) *signing.MultiSignatureData {
	return &signing.MultiSignatureData{
		BitArray:   types.NewCompactBitArray(n),
		Signatures: make([]signing.SignatureData, 0, n),
	}
}

// GetIndex returns the index of pk in keys. Returns -1 if not found
func getIndex(pk types.PubKey, keys []types.PubKey) int {
	for i := 0; i < len(keys); i++ {
		if pk.Equals(keys[i]) {
			return i
		}
	}
	return -1
}

// AddSignature adds a signature to the multisig, at the corresponding index. The index must
// represent the pubkey index in the LegacyAmingPubKey structure, which verifies this signature.
// If the signature already exists, replace it.
func AddSignature(mSig *signing.MultiSignatureData, sig signing.SignatureData, index int) {
	newSigIndex := mSig.BitArray.NumTrueBitsBefore(index)
	// Signature already exists, just replace the value there
	if mSig.BitArray.GetIndex(index) {
		mSig.Signatures[newSigIndex] = sig
		return
	}
	mSig.BitArray.SetIndex(index, true)
	// Optimization if the index is the greatest index
	if newSigIndex == len(mSig.Signatures) {
		mSig.Signatures = append(mSig.Signatures, sig)
		return
	}
	// Expand slice by one with a dummy element, move all elements after i
	// over by one, then place the new signature in that gap.
	mSig.Signatures = append(mSig.Signatures, &signing.SingleSignatureData{})
	copy(mSig.Signatures[newSigIndex+1:], mSig.Signatures[newSigIndex:])
	mSig.Signatures[newSigIndex] = sig
}

// AddSignatureFromPubKey adds a signature to the multisig, at the index in
// keys corresponding to the provided pubkey.
func AddSignatureFromPubKey(mSig *signing.MultiSignatureData, sig signing.SignatureData, pubkey types.PubKey, keys []types.PubKey) error {
	if mSig == nil {
		return fmt.Errorf("value of mSig is nil %v", mSig)
	}

	if sig == nil {
		return fmt.Errorf("value of sig is nil %v", sig)
	}

	if pubkey == nil {
		return fmt.Errorf("pubkey can't be nil %v", pubkey)
	}

	if len(keys) == 0 {
		return errors.New("keys can't be empty")
	}

	index := getIndex(pubkey, keys)
	if index == -1 {
		keysStr := make([]string, len(keys))
		for i, k := range keys {
			keysStr[i] = fmt.Sprintf("%X", k.Bytes())
		}

		return fmt.Errorf("provided key %X doesn't exist in pubkeys: \n%s", pubkey.Bytes(), strings.Join(keysStr, "\n"))
	}

	AddSignature(mSig, sig, index)
	return nil
}

func AddSignatureV2(mSig *signing.MultiSignatureData, sig signing.SignatureV2, keys []types.PubKey) error {
	return AddSignatureFromPubKey(mSig, sig.Data, sig.PubKey, keys)
}
