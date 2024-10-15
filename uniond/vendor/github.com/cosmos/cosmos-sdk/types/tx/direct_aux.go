package tx

import (
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// ValidateBasic performs stateless validation of the sign doc.
func (s *SignDocDirectAux) ValidateBasic() error {
	if len(s.BodyBytes) == 0 {
		return sdkerrors.ErrInvalidRequest.Wrap("body bytes cannot be empty")
	}

	if s.PublicKey == nil {
		return sdkerrors.ErrInvalidPubKey.Wrap("public key cannot be empty")
	}

	return nil
}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (s *SignDocDirectAux) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return unpacker.UnpackAny(s.PublicKey, new(cryptotypes.PubKey))
}

// ValidateBasic performs stateless validation of the auxiliary tx.
func (a *AuxSignerData) ValidateBasic() error {
	if a.Address == "" {
		return sdkerrors.ErrInvalidRequest.Wrapf("address cannot be empty")
	}

	if a.Mode != signing.SignMode_SIGN_MODE_DIRECT_AUX && a.Mode != signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON {
		return sdkerrors.ErrInvalidRequest.Wrapf("AuxTxBuilder can only sign with %s or %s", signing.SignMode_SIGN_MODE_DIRECT_AUX, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	}

	if len(a.Sig) == 0 {
		return sdkerrors.ErrNoSignatures.Wrap("signature cannot be empty")
	}

	return a.GetSignDoc().ValidateBasic()
}

// GetSignatureV2 gets the SignatureV2 of the aux signer.
func (a *AuxSignerData) GetSignatureV2() (signing.SignatureV2, error) {
	pk, ok := a.SignDoc.PublicKey.GetCachedValue().(cryptotypes.PubKey)
	if !ok {
		return signing.SignatureV2{}, sdkerrors.ErrInvalidType.Wrapf("expected %T, got %T", (cryptotypes.PubKey)(nil), pk)
	}

	return signing.SignatureV2{
		PubKey: pk,
		Data: &signing.SingleSignatureData{
			SignMode:  a.Mode,
			Signature: a.Sig,
		},
		Sequence: a.SignDoc.Sequence,
	}, nil
}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (a *AuxSignerData) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return a.GetSignDoc().UnpackInterfaces(unpacker)
}
