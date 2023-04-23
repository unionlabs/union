package legacytx

import (
	"fmt"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// stdTxSignModeHandler is a SignModeHandler that handles SIGN_MODE_LEGACY_AMINO_JSON
type stdTxSignModeHandler struct{}

func NewStdTxSignModeHandler() signing.SignModeHandler {
	return &stdTxSignModeHandler{}
}

// assert interface implementation
var _ signing.SignModeHandler = stdTxSignModeHandler{}

// DefaultMode implements SignModeHandler.DefaultMode
func (h stdTxSignModeHandler) DefaultMode() signingtypes.SignMode {
	return signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON
}

// Modes implements SignModeHandler.Modes
func (stdTxSignModeHandler) Modes() []signingtypes.SignMode {
	return []signingtypes.SignMode{signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON}
}

// DefaultMode implements SignModeHandler.GetSignBytes
func (stdTxSignModeHandler) GetSignBytes(mode signingtypes.SignMode, data signing.SignerData, tx sdk.Tx) ([]byte, error) {
	if mode != signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON {
		return nil, fmt.Errorf("expected %s, got %s", signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON, mode)
	}

	stdTx, ok := tx.(StdTx)
	if !ok {
		return nil, fmt.Errorf("expected %T, got %T", StdTx{}, tx)
	}

	return StdSignBytes(
		data.ChainID, data.AccountNumber, data.Sequence, stdTx.GetTimeoutHeight(), StdFee{Amount: stdTx.GetFee(), Gas: stdTx.GetGas()}, tx.GetMsgs(), stdTx.GetMemo(), nil,
	), nil
}

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
			return multisig.AminoMultisignature{}, sdkerrors.Wrapf(err, "Unable to convert Signature Data to signature %d", i)
		}
	}

	return multisig.AminoMultisignature{
		BitArray: mSig.BitArray,
		Sigs:     sigs,
	}, nil
}
