package solomachine

import (
	errorsmod "cosmossdk.io/errors"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// VerifySignature verifies if the the provided public key generated the signature
// over the given data. Single and Multi signature public keys are supported.
// The signature data type must correspond to the public key type. An error is
// returned if signature verification fails or an invalid SignatureData type is
// provided.
func VerifySignature(pubKey cryptotypes.PubKey, signBytes []byte, sigData signing.SignatureData) error {
	switch pubKey := pubKey.(type) {
	case multisig.PubKey:
		data, ok := sigData.(*signing.MultiSignatureData)
		if !ok {
			return errorsmod.Wrapf(ErrSignatureVerificationFailed, "invalid signature data type, expected %T, got %T", (*signing.MultiSignatureData)(nil), data)
		}

		// The function supplied fulfills the VerifyMultisignature interface. No special
		// adjustments need to be made to the sign bytes based on the sign mode.
		if err := pubKey.VerifyMultisignature(func(signing.SignMode) ([]byte, error) {
			return signBytes, nil
		}, data); err != nil {
			return err
		}

	default:
		data, ok := sigData.(*signing.SingleSignatureData)
		if !ok {
			return errorsmod.Wrapf(ErrSignatureVerificationFailed, "invalid signature data type, expected %T, got %T", (*signing.SingleSignatureData)(nil), data)
		}

		if !pubKey.VerifySignature(signBytes, data.Signature) {
			return ErrSignatureVerificationFailed
		}
	}

	return nil
}
