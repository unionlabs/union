package tx

import (
	"context"
	"fmt"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"

	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	txsigning "cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/textual"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// signModeTextualHandler defines the SIGN_MODE_TEXTUAL SignModeHandler.
// It is currently not enabled by default, but you can enable it manually
// for TESTING purposes. It will be enabled once SIGN_MODE_TEXTUAL is fully
// released, see https://github.com/cosmos/cosmos-sdk/issues/11970.
type signModeTextualHandler struct {
	t textual.SignModeHandler
}

var _ signing.SignModeHandler = signModeTextualHandler{}

// DefaultMode implements SignModeHandler.DefaultMode
func (signModeTextualHandler) DefaultMode() signingtypes.SignMode {
	return signingtypes.SignMode_SIGN_MODE_TEXTUAL
}

// Modes implements SignModeHandler.Modes
func (signModeTextualHandler) Modes() []signingtypes.SignMode {
	return []signingtypes.SignMode{signingtypes.SignMode_SIGN_MODE_TEXTUAL}
}

// GetSignBytes implements SignModeHandler.GetSignBytes
func (h signModeTextualHandler) GetSignBytes(mode signingtypes.SignMode, data signing.SignerData, tx sdk.Tx) ([]byte, error) {
	panic("SIGN_MODE_TEXTUAL needs GetSignBytesWithContext")
}

// GetSignBytesWithContext implements SignModeHandler.GetSignBytesWithContext
func (h signModeTextualHandler) GetSignBytesWithContext(ctx context.Context, mode signingtypes.SignMode, data signing.SignerData, tx sdk.Tx) ([]byte, error) {
	if mode != signingtypes.SignMode_SIGN_MODE_TEXTUAL {
		return nil, fmt.Errorf("expected %s, got %s", signingtypes.SignMode_SIGN_MODE_TEXTUAL, mode)
	}

	protoTx, ok := tx.(*wrapper)
	if !ok {
		return nil, fmt.Errorf("can only handle a protobuf Tx, got %T", tx)
	}

	pbAny, err := codectypes.NewAnyWithValue(data.PubKey)
	if err != nil {
		return nil, err
	}

	txBody := &txv1beta1.TxBody{}
	txAuthInfo := &txv1beta1.AuthInfo{}
	err = proto.Unmarshal(protoTx.getBodyBytes(), txBody)
	if err != nil {
		return nil, err
	}
	err = proto.Unmarshal(protoTx.getAuthInfoBytes(), txAuthInfo)
	if err != nil {
		return nil, err
	}

	txData := txsigning.TxData{
		Body:          txBody,
		AuthInfo:      txAuthInfo,
		BodyBytes:     protoTx.getBodyBytes(),
		AuthInfoBytes: protoTx.getAuthInfoBytes(),
	}

	return h.t.GetSignBytes(ctx, txsigning.SignerData{
		Address:       data.Address,
		ChainID:       data.ChainID,
		AccountNumber: data.AccountNumber,
		Sequence:      data.Sequence,
		PubKey: &anypb.Any{
			TypeUrl: pbAny.TypeUrl,
			Value:   pbAny.Value,
		},
	}, txData)
}
