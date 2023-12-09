package direct

import (
	"context"

	"google.golang.org/protobuf/proto"

	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/signing"
)

// SignModeHandler is the SIGN_MODE_DIRECT implementation of signing.SignModeHandler.
type SignModeHandler struct{}

// Mode implements signing.SignModeHandler.Mode.
func (h SignModeHandler) Mode() signingv1beta1.SignMode {
	return signingv1beta1.SignMode_SIGN_MODE_DIRECT
}

// GetSignBytes implements signing.SignModeHandler.GetSignBytes.
func (SignModeHandler) GetSignBytes(_ context.Context, signerData signing.SignerData, txData signing.TxData) ([]byte, error) {
	return proto.Marshal(&txv1beta1.SignDoc{
		BodyBytes:     txData.BodyBytes,
		AuthInfoBytes: txData.AuthInfoBytes,
		ChainId:       signerData.ChainID,
		AccountNumber: signerData.AccountNumber,
	})
}

var _ signing.SignModeHandler = SignModeHandler{}
