package legacytx

import (
	"fmt"

	"cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// SignatureDataToAminoSignature converts a SignatureData to amino-encoded signature bytes.
// Only SIGN_MODE_LEGACY_AMINO_JSON is supported.
func SignatureDataToAminoSignature(cdc *codec.LegacyAmino, data signingtypes.SignatureData) ([]byte, error) {
	switch data := data.(type) {
	case *signingtypes.SingleSignatureData:
		if data.SignMode != signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON {
			return nil, fmt.Errorf("wrong SignMode. Expected %s, got %s",
				signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON, data.SignMode)
		}

		return data.Signature, nil
	case *signingtypes.MultiSignatureData:
		aminoMSig, err := MultiSignatureDataToAminoMultisignature(cdc, data)
		if err != nil {
			return nil, err
		}

		return cdc.Marshal(aminoMSig)
	default:
		return nil, fmt.Errorf("unexpected signature data %T", data)
	}
}

// MultiSignatureDataToAminoMultisignature converts a MultiSignatureData to an AminoMultisignature.
// Only SIGN_MODE_LEGACY_AMINO_JSON is supported.
func MultiSignatureDataToAminoMultisignature(cdc *codec.LegacyAmino, mSig *signingtypes.MultiSignatureData) (multisig.AminoMultisignature, error) {
	n := len(mSig.Signatures)
	sigs := make([][]byte, n)

	for i := 0; i < n; i++ {
		var err error
		sigs[i], err = SignatureDataToAminoSignature(cdc, mSig.Signatures[i])
		if err != nil {
			return multisig.AminoMultisignature{}, errors.Wrapf(err, "Unable to convert Signature Data to signature %d", i)
		}
	}

	return multisig.AminoMultisignature{
		BitArray: mSig.BitArray,
		Sigs:     sigs,
	}, nil
}
